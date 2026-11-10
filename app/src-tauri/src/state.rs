//! What the app keeps between commands.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use skrap_core::{Progress, Reserve, RuleSet, ScanReport};
use tauri::{AppHandle, Manager};

use crate::persist;
use crate::settings::{Settings, Stats};

pub struct AppState {
    pub settings: Mutex<Settings>,
    pub settings_path: PathBuf,
    pub stats: Mutex<Stats>,
    pub stats_path: PathBuf,
    pub rules: RuleSet,
    pub reserve: Reserve,
    /// The last scan with every file it found, until it is cleaned or replaced.
    pub scan: Mutex<Option<ScanReport>>,
    /// The scan or clean running now (to cancel it); `None` when idle.
    pub job: Mutex<Option<Arc<Progress>>>,
}

/// Locks a mutex, recovering the data if a previous holder panicked.
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl AppState {
    pub fn load(app: &AppHandle) -> tauri::Result<Self> {
        let paths = app.path();
        let settings_path = paths.app_config_dir()?.join("settings.json");
        let stats_path = paths.app_data_dir()?.join("stats.json");
        // The Reserve on the system disk: `%LOCALAPPDATA%\com.kryostudio.skrap\reserve`.
        let reserve = Reserve::new(paths.app_local_data_dir()?.join("reserve"), skrap_win::hide);
        let rules = skrap_rules::builtin().unwrap_or_else(|err| {
            // The tests make sure this never ships; if it did, Skrap cleans nothing.
            eprintln!("skrap: built-in rules are broken: {err}");
            RuleSet::default()
        });
        Ok(Self {
            settings: Mutex::new(persist::load(&settings_path)),
            settings_path,
            stats: Mutex::new(persist::load(&stats_path)),
            stats_path,
            rules,
            reserve,
            scan: Mutex::default(),
            job: Mutex::default(),
        })
    }

    pub fn settings(&self) -> Settings {
        lock(&self.settings).clone()
    }

    pub fn update_stats(&self, change: impl FnOnce(&mut Stats)) {
        let mut stats = lock(&self.stats);
        change(&mut stats);
        if let Err(err) = persist::save(&self.stats_path, &*stats) {
            eprintln!("skrap: cannot save stats: {err}");
        }
    }

    /// The folders a scan never enters besides the user's own: the Reserve itself.
    pub fn extra_protected(&self) -> Vec<PathBuf> {
        vec![self.reserve.home().to_path_buf()]
    }
}
