//! The clean: acts on what the scan found, and nothing else.
//!
//! Each file is checked again first: still there, still a plain file, same size and date as at
//! the scan, outside the protected folders. *Safe* files are deleted, the others go to the
//! Reserve. What could not be cleaned is kept with its reason, so the app can say it.

use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::time::Instant;

use serde::Serialize;

use crate::fsx;
pub use crate::fsx::FailReason;
use crate::pattern;
use crate::progress::Progress;
use crate::reserve::Reserve;
use crate::rule::RuleSet;
use crate::scan::Finding;
use crate::{Context, unix_ms};

/// Files of one rule that could not be cleaned, for one reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Failure {
    pub rule: String,
    pub reason: FailReason,
    pub files: u64,
    pub bytes: u64,
    /// The program, for [`FailReason::Running`].
    pub exe: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanReport {
    /// Deleted for good.
    pub freed: u64,
    /// Moved to the Reserve (freed once the Reserve lets them go).
    pub reserved: u64,
    pub files: u64,
    /// The Reserve batch, when something went there.
    pub batch: Option<String>,
    pub failures: Vec<Failure>,
    pub duration_ms: u64,
    pub cancelled: bool,
}

/// Cleans what `findings` hold.
pub fn clean(
    rules: &RuleSet,
    findings: &[&Finding],
    ctx: &Context,
    reserve: &Reserve,
    progress: &Progress,
) -> CleanReport {
    let started = Instant::now();
    let mut report = CleanReport::default();
    let mut failures: BTreeMap<(String, FailReason), Failure> = BTreeMap::new();
    let mut fail = |rule: &str, reason: FailReason, files: u64, bytes: u64, exe: Option<&str>| {
        let failure = failures
            .entry((rule.to_owned(), reason))
            .or_insert_with(|| Failure {
                rule: rule.to_owned(),
                reason,
                files: 0,
                bytes: 0,
                exe: exe.map(str::to_owned),
            });
        failure.files += files;
        failure.bytes += bytes;
    };
    let mut deposit = reserve.deposit(unix_ms(ctx.now));
    progress.start(findings.len());

    for finding in findings {
        if progress.is_cancelled() {
            report.cancelled = true;
            break;
        }
        let Some(rule) = rules.get(&finding.rule) else {
            continue;
        };
        if let Some(exe) = rule.running.iter().find(|exe| ctx.is_running(exe)) {
            fail(
                &rule.id,
                FailReason::Running,
                finding.files,
                finding.bytes,
                Some(exe),
            );
            progress.step_done();
            continue;
        }
        if let Some(special) = rule.special {
            match ctx.specials.clean(special) {
                Ok(()) => {
                    report.freed += finding.bytes;
                    report.files += finding.files;
                    progress.add(finding.files, finding.bytes);
                }
                Err(_) => fail(
                    &rule.id,
                    FailReason::Other,
                    finding.files,
                    finding.bytes,
                    None,
                ),
            }
            progress.step_done();
            continue;
        }

        let to_reserve = rule.risk.keeps_in_reserve();
        for item in &finding.items {
            if progress.is_cancelled() {
                break;
            }
            progress.at(&item.path);
            if ctx.protected.covers(&item.path) {
                fail(&rule.id, FailReason::Protected, 1, item.size, None);
                continue;
            }
            let meta = match fs::symlink_metadata(&item.path) {
                Ok(meta) => meta,
                Err(err) if err.kind() == ErrorKind::NotFound => continue, // already gone
                Err(err) => {
                    fail(&rule.id, fsx::classify(&err), 1, item.size, None);
                    continue;
                }
            };
            if fsx::is_link(&meta)
                || !meta.is_file()
                || meta.len() != item.size
                || fsx::modified_ms(&meta) != item.modified
            {
                fail(&rule.id, FailReason::Changed, 1, item.size, None);
                continue;
            }
            let result = if to_reserve {
                deposit.put(&item.path, item.size, &rule.id)
            } else {
                fsx::remove_file(&item.path).map_err(|e| fsx::classify(&e))
            };
            match result {
                Ok(()) => {
                    if to_reserve {
                        report.reserved += item.size;
                    } else {
                        report.freed += item.size;
                    }
                    report.files += 1;
                    progress.add(1, item.size);
                }
                Err(reason) => fail(&rule.id, reason, 1, item.size, None),
            }
        }

        // Folders emptied by the clean (or already empty) go as well; the targets themselves,
        // and anything protected or excluded, stay.
        let excludes = expand_excludes(&rule.exclude, ctx);
        for root in &finding.roots {
            fsx::remove_empty_dirs(root, &|dir| {
                ctx.protected.covers(dir) || pattern::excluded(dir, &excludes)
            });
        }
        progress.step_done();
    }

    report.batch = deposit.finish();
    report.failures = failures.into_values().collect();
    report.duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    report.cancelled |= progress.is_cancelled();
    report
}

/// A rule's excludes, the full-path ones expanded.
pub(crate) fn expand_excludes(excludes: &[String], ctx: &Context) -> Vec<String> {
    excludes
        .iter()
        .filter_map(|e| {
            if e.contains(['\\', '/']) {
                pattern::expand(e, ctx)
            } else {
                Some(e.clone())
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::Risk;
    use crate::scan::scan;
    use crate::tests::{context, rule};

    struct Setup {
        _dir: tempfile::TempDir,
        base: std::path::PathBuf,
        ctx: Context,
        reserve: Reserve,
    }

    fn setup() -> Setup {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("local");
        fs::create_dir_all(&base).unwrap();
        let ctx = context(&[("LOCALAPPDATA", &base.display().to_string())]);
        let reserve = Reserve::new(dir.path().join("reserve"), |_| {});
        Setup {
            base,
            ctx,
            reserve,
            _dir: dir,
        }
    }

    fn write(path: &std::path::Path, len: usize) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, vec![7u8; len]).unwrap();
    }

    #[test]
    fn safe_files_are_deleted_review_files_reserved() {
        let s = setup();
        write(&s.base.join("Safe/sub/a.tmp"), 10);
        write(&s.base.join("Review/b.bin"), 20);
        let safe = rule("system.test.safe", &[r"%LOCALAPPDATA%\Safe"]);
        let mut review = rule("gaming.test.review", &[r"%LOCALAPPDATA%\Review"]);
        review.risk = Risk::Review;
        let rules = RuleSet::new(vec![safe, review]).unwrap();
        let found = scan(&rules, &|_| true, &s.ctx, &Progress::new());
        let findings: Vec<&Finding> = found.findings.iter().collect();

        let report = clean(&rules, &findings, &s.ctx, &s.reserve, &Progress::new());
        assert_eq!((report.freed, report.reserved, report.files), (10, 20, 2));
        assert!(report.failures.is_empty(), "{:?}", report.failures);
        // The targets stay, emptied; the folder below them goes.
        assert!(s.base.join("Safe").is_dir());
        assert!(!s.base.join("Safe/sub").exists());
        assert!(!s.base.join("Review/b.bin").exists());

        let batch = report.batch.unwrap();
        assert_eq!(s.reserve.list()[0].bytes, 20);
        s.reserve.restore(&batch, None);
        assert_eq!(fs::read(s.base.join("Review/b.bin")).unwrap().len(), 20);
    }

    #[test]
    fn files_changed_since_the_scan_stay() {
        let s = setup();
        write(&s.base.join("Cache/a.tmp"), 10);
        write(&s.base.join("Cache/b.tmp"), 10);
        let rules =
            RuleSet::new(vec![rule("apps.test.cache", &[r"%LOCALAPPDATA%\Cache"])]).unwrap();
        let found = scan(&rules, &|_| true, &s.ctx, &Progress::new());
        // After the scan: one file grows, one is replaced by a folder... by a new file.
        write(&s.base.join("Cache/a.tmp"), 11);
        fs::write(s.base.join("Cache/new.tmp"), "new").unwrap();

        let findings: Vec<&Finding> = found.findings.iter().collect();
        let report = clean(&rules, &findings, &s.ctx, &s.reserve, &Progress::new());
        assert_eq!(report.files, 1);
        assert_eq!(
            report.failures,
            vec![Failure {
                rule: "apps.test.cache".into(),
                reason: FailReason::Changed,
                files: 1,
                bytes: 10,
                exe: None,
            }]
        );
        assert!(s.base.join("Cache/a.tmp").exists());
        assert!(
            s.base.join("Cache/new.tmp").exists(),
            "not in the scan, not cleaned"
        );
    }

    #[test]
    fn a_program_started_since_the_scan_stops_its_rule() {
        let mut s = setup();
        write(&s.base.join("Discord/Cache/a"), 5);
        let mut r = rule("apps.discord.cache", &[r"%LOCALAPPDATA%\Discord\Cache"]);
        r.running = vec!["Discord.exe".into()];
        let rules = RuleSet::new(vec![r]).unwrap();
        let found = scan(&rules, &|_| true, &s.ctx, &Progress::new());
        s.ctx.running.insert("discord.exe".into());
        let findings: Vec<&Finding> = found.findings.iter().collect();
        let report = clean(&rules, &findings, &s.ctx, &s.reserve, &Progress::new());
        assert_eq!(report.files, 0);
        assert_eq!(report.failures[0].reason, FailReason::Running);
        assert_eq!(report.failures[0].exe.as_deref(), Some("Discord.exe"));
        assert!(s.base.join("Discord/Cache/a").exists());
    }

    #[test]
    fn a_folder_protected_after_the_scan_is_still_safe() {
        let mut s = setup();
        write(&s.base.join("Cache/Keep/a"), 5);
        let rules =
            RuleSet::new(vec![rule("apps.test.cache", &[r"%LOCALAPPDATA%\Cache"])]).unwrap();
        let found = scan(&rules, &|_| true, &s.ctx, &Progress::new());
        s.ctx.protected = crate::Protected::new([s.base.join("Cache/Keep")]);
        let findings: Vec<&Finding> = found.findings.iter().collect();
        let report = clean(&rules, &findings, &s.ctx, &s.reserve, &Progress::new());
        assert_eq!(report.failures[0].reason, FailReason::Protected);
        assert!(s.base.join("Cache/Keep/a").exists());
    }
}
