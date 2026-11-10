//! The scan: what each rule targets on this PC, file by file, without changing anything.

use std::fs;
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use serde::Serialize;

use crate::fsx;
use crate::pattern;
use crate::progress::Progress;
use crate::rule::{Rule, RuleSet};
use crate::{Context, unix_ms};

/// One file a rule found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub path: PathBuf,
    pub size: u64,
    /// Last write time (Unix ms): a file changed since the scan is left alone.
    pub modified: Option<u64>,
}

/// Why a rule found something but cannot be cleaned now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Skip {
    /// The program that uses these files is running (`exe` as in the rule).
    Running { exe: String },
}

/// What one rule found.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub rule: String,
    pub bytes: u64,
    pub files: u64,
    #[serde(skip)]
    pub items: Vec<Item>,
    /// The folders the rule's patterns named: emptied folders below them go too.
    #[serde(skip)]
    pub roots: Vec<PathBuf>,
    pub skip: Option<Skip>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanReport {
    pub findings: Vec<Finding>,
    pub started: u64,
    pub duration_ms: u64,
    pub cancelled: bool,
}

impl ScanReport {
    pub fn finding(&self, rule: &str) -> Option<&Finding> {
        self.findings.iter().find(|f| f.rule == rule)
    }

    pub fn total_bytes(&self) -> u64 {
        self.findings.iter().map(|f| f.bytes).sum()
    }
}

/// Scans `rules` (the ones `wanted` keeps), in parallel. Rules that find nothing are left out of
/// the report.
pub fn scan(
    rules: &RuleSet,
    wanted: &dyn Fn(&Rule) -> bool,
    ctx: &Context,
    progress: &Progress,
) -> ScanReport {
    let started = std::time::Instant::now();
    let selected: Vec<&Rule> = rules.rules().iter().filter(|r| wanted(r)).collect();
    progress.start(selected.len());
    let findings: Vec<Finding> = selected
        .par_iter()
        .filter_map(|rule| {
            let finding = scan_rule(rule, ctx, progress);
            progress.step_done();
            finding.filter(|f| f.files > 0 || f.bytes > 0)
        })
        .collect();
    ScanReport {
        findings,
        started: unix_ms(ctx.now),
        duration_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        cancelled: progress.is_cancelled(),
    }
}

fn scan_rule(rule: &Rule, ctx: &Context, progress: &Progress) -> Option<Finding> {
    if progress.is_cancelled() {
        return None;
    }
    let skip = rule
        .running
        .iter()
        .find(|exe| ctx.is_running(exe))
        .map(|exe| Skip::Running { exe: exe.clone() });

    if let Some(special) = rule.special {
        let (bytes, files) = ctx.specials.measure(special)?;
        progress.add(files, bytes);
        return Some(Finding {
            rule: rule.id.clone(),
            bytes,
            files,
            items: Vec::new(),
            roots: Vec::new(),
            skip,
        });
    }

    let excludes = crate::clean::expand_excludes(&rule.exclude, ctx);
    let walker = Walker {
        ctx,
        progress,
        excludes: &excludes,
        min_age: rule.older_than_hours.map(u64::from),
    };

    let mut roots = Vec::new();
    let mut items = Vec::new();
    for path in &rule.paths {
        let Some(expanded) = pattern::expand(path, ctx) else {
            continue;
        };
        for target in pattern::resolve(&expanded) {
            // A target that is, or holds, a protected folder is not touched at all.
            if ctx.protected.covers(&target) || ctx.protected.inside(&target) {
                continue;
            }
            let Ok(meta) = fs::symlink_metadata(&target) else {
                continue;
            };
            if meta.is_dir() {
                walker.walk(&target, &mut items);
                roots.push(target);
            } else {
                walker.file(&target, &meta, &mut items);
            }
        }
    }
    // Two patterns may name the same files.
    items.sort_by(|a, b| a.path.cmp(&b.path));
    items.dedup_by(|a, b| a.path == b.path);
    let bytes = items.iter().map(|i| i.size).sum();
    Some(Finding {
        rule: rule.id.clone(),
        bytes,
        files: items.len() as u64,
        items,
        roots,
        skip,
    })
}

struct Walker<'a> {
    ctx: &'a Context,
    progress: &'a Progress,
    excludes: &'a [String],
    min_age: Option<u64>,
}

impl Walker<'_> {
    fn walk(&self, dir: &Path, out: &mut Vec<Item>) {
        if self.progress.is_cancelled() || self.ctx.protected.covers(dir) {
            return;
        }
        if pattern::excluded(dir, self.excludes) {
            return;
        }
        self.progress.at(dir);
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            // On Windows this comes with the directory listing: no extra system call.
            let Ok(meta) = entry.metadata() else {
                continue;
            };
            if fsx::is_link(&meta) {
                continue;
            }
            let path = entry.path();
            if meta.is_dir() {
                self.walk(&path, out);
            } else {
                self.file(&path, &meta, out);
            }
        }
    }

    fn file(&self, path: &Path, meta: &fs::Metadata, out: &mut Vec<Item>) {
        if fsx::is_link(meta) || self.ctx.protected.covers(path) {
            return;
        }
        if pattern::excluded(path, self.excludes) {
            return;
        }
        if let Some(min_age) = self.min_age
            && fsx::age_hours(meta, self.ctx.now).is_none_or(|age| age < min_age)
        {
            return;
        }
        let size = meta.len();
        self.progress.add(1, size);
        out.push(Item {
            path: path.to_path_buf(),
            size,
            modified: fsx::modified_ms(meta),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{context, rule};
    use std::time::{Duration, SystemTime};

    #[test]
    fn finds_files_below_targets_except_excluded_ones() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path();
        let cache = base.join("App").join("Cache");
        fs::create_dir_all(cache.join("sub")).unwrap();
        fs::write(cache.join("a.bin"), vec![0u8; 100]).unwrap();
        fs::write(cache.join("sub").join("b.bin"), vec![0u8; 50]).unwrap();
        fs::write(cache.join("keep.lock"), "x").unwrap();
        // Protected, but next to the target: no effect on it.
        fs::create_dir_all(base.join("App").join("Saves")).unwrap();

        let mut ctx = context(&[("LOCALAPPDATA", &base.display().to_string())]);
        ctx.protected = crate::Protected::new([base.join("App").join("Saves")]);
        let mut r = rule("apps.test.cache", &[r"%LOCALAPPDATA%\App\Cache"]);
        r.exclude = vec!["*.lock".into()];
        let rules = RuleSet::new(vec![r]).unwrap();
        let progress = Progress::new();
        let report = scan(&rules, &|_| true, &ctx, &progress);

        let finding = report.finding("apps.test.cache").unwrap();
        assert_eq!(finding.files, 2);
        assert_eq!(finding.bytes, 150);
        assert_eq!(finding.roots, vec![cache.clone()]);
        assert_eq!(progress.snapshot().bytes, 150);
        assert_eq!(progress.snapshot().steps_done, 1);
    }

    #[test]
    fn a_protected_folder_is_never_scanned() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path();
        fs::create_dir_all(base.join("Docs")).unwrap();
        fs::write(base.join("Docs").join("cv.pdf"), "x").unwrap();
        let mut ctx = context(&[("LOCALAPPDATA", &base.display().to_string())]);
        ctx.protected = crate::Protected::new([base.join("Docs")]);
        let rules =
            RuleSet::new(vec![rule("apps.test.docs", &[r"%LOCALAPPDATA%\Docs\*"])]).unwrap();
        assert!(
            scan(&rules, &|_| true, &ctx, &Progress::new())
                .findings
                .is_empty()
        );
    }

    #[test]
    fn a_target_holding_a_protected_folder_is_left_out() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path();
        fs::create_dir_all(base.join("Tool").join("Documents")).unwrap();
        fs::write(base.join("Tool").join("junk.tmp"), "x").unwrap();
        let mut ctx = context(&[("LOCALAPPDATA", &base.display().to_string())]);
        ctx.protected = crate::Protected::new([base.join("Tool").join("Documents")]);
        let rules = RuleSet::new(vec![rule("apps.test.tool", &[r"%LOCALAPPDATA%\Tool"])]).unwrap();
        let report = scan(&rules, &|_| true, &ctx, &Progress::new());
        assert!(report.findings.is_empty());
    }

    #[test]
    fn running_programs_and_age() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path();
        fs::create_dir_all(base.join("T")).unwrap();
        fs::write(base.join("T").join("new.tmp"), "x").unwrap();

        let mut ctx = context(&[("TEMP", &base.join("T").display().to_string())]);
        let mut r = rule("system.test.temp", &[r"%TEMP%\*"]);
        r.running = vec!["Chrome.exe".into()];
        r.older_than_hours = Some(24);
        let rules = RuleSet::new(vec![r]).unwrap();

        // Fresh files are not old enough.
        assert!(
            scan(&rules, &|_| true, &ctx, &Progress::new())
                .findings
                .is_empty()
        );

        // A day later they are; Chrome running marks the finding as skipped.
        ctx.now = SystemTime::now() + Duration::from_secs(25 * 3600);
        ctx.running.insert("chrome.exe".into());
        let report = scan(&rules, &|_| true, &ctx, &Progress::new());
        let finding = report.finding("system.test.temp").unwrap();
        assert_eq!(finding.files, 1);
        assert_eq!(
            finding.skip,
            Some(Skip::Running {
                exe: "Chrome.exe".into()
            })
        );
    }

    #[test]
    fn cancelled_scans_stop() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.tmp"), "x").unwrap();
        let ctx = context(&[("TEMP", &dir.path().display().to_string())]);
        let rules = RuleSet::new(vec![rule("system.test.temp", &[r"%TEMP%"])]).unwrap();
        let progress = Progress::new();
        progress.cancel();
        let report = scan(&rules, &|_| true, &ctx, &progress);
        assert!(report.cancelled);
        assert!(report.findings.is_empty());
    }
}
