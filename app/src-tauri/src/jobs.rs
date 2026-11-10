//! Scans and cleans run on their own thread; their progress reaches the page as events, about 30
//! times a second at most, however many files go by.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use serde::Serialize;
use skrap_core::{CleanReport, Finding, Progress, Risk, unix_ms};
use tauri::{AppHandle, Emitter, Manager};

use crate::state::{AppState, lock};

pub const SCAN_PROGRESS: &str = "scan-progress";
/// One rule's finding, as soon as the scan has it.
pub const SCAN_FOUND: &str = "scan-found";
pub const SCAN_DONE: &str = "scan-done";
pub const CLEAN_PROGRESS: &str = "clean-progress";
pub const CLEAN_DONE: &str = "clean-done";
/// The Reserve changed on its own (expired batches freed, low disk).
pub const RESERVE_CHANGED: &str = "reserve-changed";

const TICK: Duration = Duration::from_millis(33);

/// Below this share of free space, the Reserve frees its oldest batches...
const LOW_DISK: f64 = 0.10;
/// ...those more than a day old.
const LOW_DISK_GRACE_MS: u64 = 24 * 60 * 60 * 1000;

pub fn start_scan(app: &AppHandle) -> Result<(), &'static str> {
    let progress = begin(app)?;
    let app = app.clone();
    thread::spawn(move || {
        let ticker = Ticker::start(&app, SCAN_PROGRESS, &progress);
        let state = app.state::<AppState>();
        let ctx = skrap_win::context(&state.extra_protected());
        let wanted = |rule: &skrap_core::Rule| rule.risk != Risk::Expert;
        let found = |finding: &Finding| {
            let _ = app.emit(SCAN_FOUND, finding);
        };
        let report = skrap_core::scan_with(&state.rules, &wanted, &ctx, &progress, &found);
        ticker.stop();
        state.update_stats(|stats| stats.last_scan = Some(report.started));
        let _ = app.emit(SCAN_DONE, &report);
        *lock(&state.scan) = (!report.cancelled).then_some(report);
        *lock(&state.job) = None;
    });
    Ok(())
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanDone {
    #[serde(flatten)]
    report: CleanReport,
    /// When the Reserve frees this clean's batch for good (Unix ms).
    reserve_until: Option<u64>,
}

/// Cleans what the last scan found for `rules`, in that order (the page animates them so).
pub fn start_clean(app: &AppHandle, rules: Vec<String>) -> Result<(), &'static str> {
    let state = app.state::<AppState>();
    let Some(report) = lock(&state.scan).take() else {
        return Err("no_scan");
    };
    let progress = match begin(app) {
        Ok(progress) => progress,
        Err(err) => {
            *lock(&state.scan) = Some(report);
            return Err(err);
        }
    };
    let app = app.clone();
    thread::spawn(move || {
        let ticker = Ticker::start(&app, CLEAN_PROGRESS, &progress);
        let state = app.state::<AppState>();
        let ctx = skrap_win::context(&state.extra_protected());
        let findings: Vec<&Finding> = rules.iter().filter_map(|id| report.finding(id)).collect();
        let cleaned = skrap_core::clean(&state.rules, &findings, &ctx, &state.reserve, &progress);
        ticker.stop();

        let now = unix_ms(ctx.now);
        state.update_stats(|stats| {
            stats.cleaned_bytes += cleaned.freed + cleaned.reserved;
            stats.cleans += 1;
            stats.last_clean = Some(now);
        });
        let reserve_until = cleaned
            .batch
            .as_ref()
            .map(|_| now + state.settings().reserve_ms());
        let _ = app.emit(
            CLEAN_DONE,
            CleanDone {
                report: cleaned,
                reserve_until,
            },
        );
        *lock(&state.job) = None;
        maintain_reserve(&app);
    });
    Ok(())
}

pub fn cancel(app: &AppHandle) {
    if let Some(progress) = lock(&app.state::<AppState>().job).as_ref() {
        progress.cancel();
    }
}

/// Frees the Reserve's batches past their delay. Then, while the Windows disk is under 10% free,
/// frees its oldest batches on that disk, one at a time: only those more than a day old (what
/// was just cleaned can still come back), and only on that disk (freeing another changes
/// nothing to it). Skipped while a scan, a clean or a restore runs. Tells the page what went.
pub fn maintain_reserve(app: &AppHandle) {
    let freed = exclusive(app, || {
        let state = app.state::<AppState>();
        let now = unix_ms(std::time::SystemTime::now());
        let mut freed = state
            .reserve
            .purge_before(now.saturating_sub(state.settings().reserve_ms()));

        let home = state.reserve.home();
        let total = skrap_win::disks()
            .into_iter()
            .find(|d| d.system)
            .map_or(0, |d| d.total);
        // A disk of unknown size is never taken for a full one.
        let low = || {
            let free = skrap_win::free_space(home).unwrap_or(u64::MAX);
            total > 0 && (free as f64) < (total as f64) * LOW_DISK
        };
        let mut candidates: Vec<_> = state
            .reserve
            .home_list()
            .into_iter()
            .filter(|batch| batch.created < now.saturating_sub(LOW_DISK_GRACE_MS))
            .collect();
        while low() {
            // The list is newest first: the oldest is last.
            let Some(oldest) = candidates.pop() else {
                break;
            };
            let gone = state.reserve.purge_home(&oldest.id);
            if gone == 0 {
                break; // could not be freed: no point going on
            }
            freed += gone;
        }
        freed
    });
    if let Ok(freed) = freed
        && freed > 0
    {
        let _ = app.emit(RESERVE_CHANGED, freed);
    }
}

/// Runs `work` while no scan, clean or other Reserve change can start, or refuses when one runs.
pub fn exclusive<T>(app: &AppHandle, work: impl FnOnce() -> T) -> Result<T, &'static str> {
    begin(app)?;
    let result = work();
    *lock(&app.state::<AppState>().job) = None;
    Ok(result)
}

/// Marks a job as running, or refuses when one already is.
fn begin(app: &AppHandle) -> Result<Arc<Progress>, &'static str> {
    let state = app.state::<AppState>();
    let mut job = lock(&state.job);
    if job.is_some() {
        return Err("busy");
    }
    let progress = Arc::new(Progress::new());
    *job = Some(Arc::clone(&progress));
    Ok(progress)
}

/// Sends a job's progress while it runs, only when it moved, and once more at the end.
struct Ticker {
    done: Arc<AtomicBool>,
    handle: JoinHandle<()>,
}

impl Ticker {
    fn start(app: &AppHandle, event: &'static str, progress: &Arc<Progress>) -> Self {
        let done = Arc::new(AtomicBool::new(false));
        let handle = {
            let (app, progress, done) = (app.clone(), Arc::clone(progress), Arc::clone(&done));
            thread::spawn(move || {
                let mut last = None;
                loop {
                    let finished = done.load(Ordering::Relaxed);
                    let snapshot = progress.snapshot();
                    if last.as_ref() != Some(&snapshot) {
                        let _ = app.emit(event, &snapshot);
                        last = Some(snapshot);
                    }
                    if finished {
                        break;
                    }
                    thread::sleep(TICK);
                }
            })
        };
        Self { done, handle }
    }

    fn stop(self) {
        self.done.store(true, Ordering::Relaxed);
        let _ = self.handle.join();
    }
}
