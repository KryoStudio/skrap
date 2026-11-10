//! Small file system helpers shared by the scan, the clean and the Reserve.

use std::fs::{self, Metadata};
use std::io;
use std::path::Path;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::unix_ms;

/// A junction, a symbolic link, or any other reparse point (cloud placeholders included): never
/// followed, never cleaned.
pub fn is_link(meta: &Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 || meta.file_type().is_symlink()
    }
    #[cfg(not(windows))]
    {
        meta.file_type().is_symlink()
    }
}

/// Last write time in Unix milliseconds.
pub fn modified_ms(meta: &Metadata) -> Option<u64> {
    meta.modified().ok().map(unix_ms)
}

pub fn age_hours(meta: &Metadata, now: SystemTime) -> Option<u64> {
    let modified = meta.modified().ok()?;
    Some(
        now.duration_since(modified)
            .map_or(0, |d| d.as_secs() / 3600),
    )
}

/// Why a file could not be cleaned or restored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailReason {
    /// Open in a program (or locked by Windows).
    InUse,
    /// Needs rights Skrap does not have (usually administrator).
    Denied,
    /// Changed since the scan: left alone.
    Changed,
    /// Inside a protected folder.
    Protected,
    /// No Reserve on this disk (a network share): nothing is deleted without one.
    NoReserve,
    /// Something is already where the file goes back.
    Exists,
    /// The program that uses it is running.
    Running,
    Other,
}

pub fn classify(err: &io::Error) -> FailReason {
    // ERROR_SHARING_VIOLATION, ERROR_LOCK_VIOLATION, ERROR_USER_MAPPED_FILE
    match err.raw_os_error() {
        Some(32 | 33 | 1224) => return FailReason::InUse,
        Some(5) => return FailReason::Denied,
        _ => {}
    }
    match err.kind() {
        io::ErrorKind::PermissionDenied => FailReason::Denied,
        io::ErrorKind::AlreadyExists => FailReason::Exists,
        _ => FailReason::Other,
    }
}

/// Writes a file through a temporary one and a rename, so a crash never leaves half of it.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    let tmp = path.with_file_name(name);
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)
}

/// Deletes a file, clearing its read-only flag if that is what stands in the way.
pub fn remove_file(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Err(err) if err.kind() == io::ErrorKind::PermissionDenied => {
            let meta = fs::symlink_metadata(path)?;
            let mut permissions = meta.permissions();
            if !permissions.readonly() {
                return Err(err);
            }
            #[allow(clippy::permissions_set_readonly_false)]
            permissions.set_readonly(false);
            fs::set_permissions(path, permissions)?;
            fs::remove_file(path)
        }
        result => result,
    }
}

/// Removes the empty folders under `root` (not `root` itself), deepest first. Links and anything
/// `keep` says to keep are left alone.
pub fn remove_empty_dirs(root: &Path, keep: &dyn Fn(&Path) -> bool) {
    fn visit(dir: &Path, keep: &dyn Fn(&Path) -> bool) -> bool {
        let Ok(entries) = fs::read_dir(dir) else {
            return false;
        };
        let mut empty = true;
        for entry in entries.flatten() {
            let path = entry.path();
            let is_real_dir = entry.metadata().is_ok_and(|m| m.is_dir() && !is_link(&m));
            if is_real_dir && !keep(&path) && visit(&path, keep) && fs::remove_dir(&path).is_ok() {
                continue;
            }
            empty = false;
        }
        empty
    }
    if fs::symlink_metadata(root).is_ok_and(|m| m.is_dir() && !is_link(&m)) {
        visit(root, keep);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_read_only_files() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("ro.tmp");
        fs::write(&file, "x").unwrap();
        let mut permissions = fs::metadata(&file).unwrap().permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&file, permissions).unwrap();
        remove_file(&file).unwrap();
        assert!(!file.exists());
    }

    #[test]
    fn removes_only_empty_folders_below_the_root() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("cache");
        fs::create_dir_all(root.join("a/b/c")).unwrap();
        fs::create_dir_all(root.join("full/inner")).unwrap();
        fs::write(root.join("full/inner/file"), "x").unwrap();
        fs::create_dir_all(root.join("kept/empty")).unwrap();
        remove_empty_dirs(&root, &|p| p.ends_with("kept"));
        assert!(root.is_dir());
        assert!(!root.join("a").exists());
        assert!(root.join("full/inner/file").exists());
        assert!(root.join("kept/empty").is_dir());
    }
}
