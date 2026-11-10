//! Progress shared between a running scan or clean and whoever shows it. The worker only bumps
//! atomics; the app reads a snapshot about 30 times a second, whatever the number of files.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering::Relaxed};

use serde::Serialize;

#[derive(Debug, Default)]
pub struct Progress {
    files: AtomicU64,
    bytes: AtomicU64,
    steps_done: AtomicUsize,
    steps_total: AtomicUsize,
    current: Mutex<String>,
    cancelled: AtomicBool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressSnapshot {
    pub files: u64,
    pub bytes: u64,
    pub steps_done: usize,
    pub steps_total: usize,
    /// Where the worker is (a folder while scanning, a file while cleaning).
    pub current: String,
}

impl Progress {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&self, steps: usize) {
        self.steps_total.store(steps, Relaxed);
    }

    pub fn add(&self, files: u64, bytes: u64) {
        self.files.fetch_add(files, Relaxed);
        self.bytes.fetch_add(bytes, Relaxed);
    }

    pub fn step_done(&self) {
        self.steps_done.fetch_add(1, Relaxed);
    }

    /// Where the worker is. Skipped when another thread is writing it: it is only shown.
    pub fn at(&self, place: &std::path::Path) {
        if let Ok(mut current) = self.current.try_lock() {
            current.clear();
            current.push_str(&place.to_string_lossy());
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Relaxed)
    }

    pub fn snapshot(&self) -> ProgressSnapshot {
        ProgressSnapshot {
            files: self.files.load(Relaxed),
            bytes: self.bytes.load(Relaxed),
            steps_done: self.steps_done.load(Relaxed),
            steps_total: self.steps_total.load(Relaxed),
            current: self.current.lock().map(|c| c.clone()).unwrap_or_default(),
        }
    }
}
