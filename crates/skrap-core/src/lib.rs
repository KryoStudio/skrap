//! Skrap's cleaning engine.
//!
//! Rules say what can go and why ([`rule`]); patterns turn them into real paths ([`pattern`]); a
//! scan measures what they target ([`scan`]); a clean acts on what the scan found and nothing
//! else ([`clean`]), deleting what rebuilds itself and moving the rest to the Reserve
//! ([`reserve`]). Folders that hold the user's own files are never reached ([`protect`]).
//!
//! The engine knows nothing about Windows APIs: what it needs from the system comes in a
//! [`Context`], so the app, the command line and the tests all run the same code.

pub mod clean;
pub mod fsx;
pub mod pattern;
pub mod progress;
pub mod protect;
pub mod reserve;
pub mod rule;
pub mod scan;
pub mod text;

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

pub use clean::{CleanReport, FailReason, Failure, clean};
pub use progress::{Progress, ProgressSnapshot};
pub use protect::Protected;
pub use reserve::{BatchSummary, Reserve, RestoreReport};
pub use rule::{Category, Risk, Rule, RuleError, RuleSet, Special};
pub use scan::{Finding, Item, ScanReport, Skip, scan, scan_with};
pub use text::Text;

/// What the engine needs to know about the machine it runs on.
pub struct Context {
    /// Folder variables (`LOCALAPPDATA`, `TEMP`, `STEAM`...), keys in upper case.
    pub vars: HashMap<String, String>,
    /// Executable names of the running processes, in lower case (`chrome.exe`).
    pub running: HashSet<String>,
    /// Folders that are never scanned nor cleaned.
    pub protected: Protected,
    /// What only the system can measure or clean (the Recycle Bin).
    pub specials: Box<dyn Specials>,
    pub now: SystemTime,
}

impl Context {
    /// The value of a folder variable, by any case (`%LocalAppData%`).
    pub fn var(&self, name: &str) -> Option<&str> {
        self.vars
            .get(&name.to_ascii_uppercase())
            .map(String::as_str)
    }

    /// Whether a program is running (`chrome.exe`), by any case.
    pub fn is_running(&self, exe: &str) -> bool {
        self.running.contains(&exe.to_ascii_lowercase())
    }

    pub fn now_ms(&self) -> u64 {
        unix_ms(self.now)
    }
}

/// Things measured and cleaned through the system rather than file by file.
pub trait Specials: Send + Sync {
    /// Size in bytes and number of items, `None` when it cannot be measured.
    fn measure(&self, special: Special) -> Option<(u64, u64)>;
    /// Empties it.
    fn clean(&self, special: Special) -> Result<(), String>;
}

/// No special handling (tests, or a platform without one).
pub struct NoSpecials;

impl Specials for NoSpecials {
    fn measure(&self, _: Special) -> Option<(u64, u64)> {
        None
    }
    fn clean(&self, _: Special) -> Result<(), String> {
        Err("unsupported".into())
    }
}

/// Milliseconds since the Unix epoch (0 for a time before it).
pub fn unix_ms(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A context with these folder variables, nothing running and nothing protected.
    pub fn context(vars: &[(&str, &str)]) -> Context {
        Context {
            vars: vars
                .iter()
                .map(|(k, v)| (k.to_ascii_uppercase(), (*v).to_owned()))
                .collect(),
            running: HashSet::new(),
            protected: Protected::default(),
            specials: Box::new(NoSpecials),
            now: SystemTime::now(),
        }
    }

    /// A Safe rule over `paths`.
    pub fn rule(id: &str, paths: &[&str]) -> Rule {
        Rule {
            id: id.to_owned(),
            category: Category::Apps,
            risk: Risk::Safe,
            group: Text::from([("en", "Test")]),
            name: Text::from([("en", "Test files")]),
            explain: Text::from([("en", "Rebuilt on their own.")]),
            paths: paths.iter().map(|p| (*p).to_owned()).collect(),
            exclude: Vec::new(),
            running: Vec::new(),
            older_than_hours: None,
            special: None,
            admin: false,
        }
    }
}
