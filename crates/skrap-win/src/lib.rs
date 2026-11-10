//! What Skrap needs from Windows, behind plain functions: the folder variables rules start from,
//! the running programs, the folders to protect, the disks, the Recycle Bin.
//!
//! Everything here reads; the only changes are emptying the Recycle Bin (when the user cleans
//! it) and hiding a Reserve folder.

#[cfg(windows)]
mod sys;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::Serialize;
use skrap_core::{Context, Protected, Special, Specials};

/// A fixed disk, as the home screen shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Disk {
    /// `C:\`
    pub root: String,
    pub label: String,
    pub total: u64,
    pub free: u64,
    /// The disk Windows runs from.
    pub system: bool,
}

/// Everything the engine needs to scan or clean now. `extra_protected`: more folders to keep out
/// of reach (the Reserve itself).
pub fn context(extra_protected: &[PathBuf]) -> Context {
    let mut protected = protected_folders();
    protected.extend(extra_protected.iter().cloned());
    Context {
        vars: variables(),
        running: running_programs(),
        protected: Protected::new(protected),
        specials: Box::new(WindowsSpecials),
        now: SystemTime::now(),
    }
}

/// The folder variables rules can start from, keys in upper case. Next to Windows' own: the
/// LocalLow folder (`LOCALAPPDATALOW`) and Steam's folder (`STEAM`), when installed.
pub fn variables() -> HashMap<String, String> {
    let mut vars: HashMap<String, String> = std::env::vars_os()
        .filter_map(|(k, v)| {
            Some((
                k.into_string().ok()?.to_ascii_uppercase(),
                v.into_string().ok()?,
            ))
        })
        .collect();
    if let Some(profile) = vars.get("USERPROFILE").cloned() {
        vars.insert(
            "LOCALAPPDATALOW".into(),
            Path::new(&profile)
                .join("AppData")
                .join("LocalLow")
                .display()
                .to_string(),
        );
    }
    if let Some(steam) = steam_folder() {
        vars.insert("STEAM".into(), steam.display().to_string());
    }
    // Folders as they really are: long names instead of 8.3 ones (`C:\Users\JEAN-P~1`),
    // junctions followed, so they compare with the protected folders, resolved the same way.
    for name in FOLDER_VARIABLES {
        if let Some(value) = vars.get_mut(*name) {
            *value = real_path(Path::new(value.as_str())).display().to_string();
        }
    }
    vars
}

/// The folders that hold the user's own files. Known folders come from Windows, so a Desktop or
/// Documents moved elsewhere (OneDrive, another disk) is protected where it really is.
pub fn protected_folders() -> Vec<PathBuf> {
    let mut folders = Vec::new();
    #[cfg(windows)]
    folders.extend(sys::known_folders());
    let env = |name: &str| std::env::var_os(name).map(PathBuf::from);
    // Cloud folders.
    for name in ["OneDrive", "OneDriveConsumer", "OneDriveCommercial"] {
        folders.extend(env(name));
    }
    if let Some(profile) = env("USERPROFILE") {
        for name in [
            "Dropbox",
            "Google Drive",
            "iCloudDrive",
            "Saved Games",
            "OneDrive",
        ] {
            folders.push(profile.join(name));
        }
        // Game saves outside Saved Games: the usual places.
        folders.push(profile.join("Documents").join("My Games"));
    }
    if let Some(local) = env("LOCALAPPDATA") {
        // Google Drive's local copy and Dropbox's settings: emptying them breaks the sync.
        folders.push(local.join("Google").join("DriveFS"));
        folders.push(local.join("Dropbox"));
    }
    folders.extend(dropbox_folders());
    // The Krak library, wherever the user put it.
    folders.extend(krak_library());
    folders.retain(|folder| folder.is_absolute());
    // Each folder both as written and as it really is (short names, junctions resolved).
    let real: Vec<PathBuf> = folders.iter().map(|f| real_path(f)).collect();
    folders.extend(real);
    folders
}

/// The folder variables rules start from (`%SYSTEMDRIVE%` aside: a bare drive).
const FOLDER_VARIABLES: &[&str] = &[
    "TEMP",
    "TMP",
    "LOCALAPPDATA",
    "LOCALAPPDATALOW",
    "APPDATA",
    "PROGRAMDATA",
    "USERPROFILE",
    "WINDIR",
    "PROGRAMFILES",
    "PROGRAMFILES(X86)",
    "STEAM",
];

/// `path` with its long names and its junctions resolved, without the `\\?\` prefix; as is
/// when it does not exist.
pub fn real_path(path: &Path) -> PathBuf {
    let Ok(real) = std::fs::canonicalize(path) else {
        return path.to_path_buf();
    };
    let text = real.to_string_lossy();
    if let Some(unc) = text.strip_prefix(r"\\?\UNC\") {
        PathBuf::from(format!(r"\\{unc}"))
    } else if let Some(local) = text.strip_prefix(r"\\?\") {
        PathBuf::from(local)
    } else {
        real
    }
}

/// Executable names of the running programs, in lower case.
pub fn running_programs() -> HashSet<String> {
    #[cfg(windows)]
    {
        sys::running_programs()
    }
    #[cfg(not(windows))]
    {
        HashSet::new()
    }
}

/// The fixed disks, the system one first.
pub fn disks() -> Vec<Disk> {
    #[cfg(windows)]
    {
        let mut disks = sys::disks();
        disks.sort_by_key(|d| (!d.system, d.root.clone()));
        disks
    }
    #[cfg(not(windows))]
    {
        Vec::new()
    }
}

/// Free space on the disk that holds `path`.
pub fn free_space(path: &Path) -> Option<u64> {
    #[cfg(windows)]
    {
        sys::free_space(path)
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        None
    }
}

/// Whether Skrap runs with administrator rights.
pub fn is_elevated() -> bool {
    #[cfg(windows)]
    {
        sys::is_elevated()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// Marks a folder hidden (the Reserve at a disk's root).
pub fn hide(path: &Path) {
    #[cfg(windows)]
    sys::hide(path);
    #[cfg(not(windows))]
    let _ = path;
}

/// The Recycle Bin, measured and emptied through the shell.
pub struct WindowsSpecials;

impl Specials for WindowsSpecials {
    fn measure(&self, special: Special) -> Option<(u64, u64)> {
        match special {
            #[cfg(windows)]
            Special::RecycleBin => sys::recycle_bin(),
            #[cfg(not(windows))]
            Special::RecycleBin => None,
        }
    }

    fn clean(&self, special: Special) -> Result<(), String> {
        match special {
            #[cfg(windows)]
            Special::RecycleBin => sys::empty_recycle_bin(),
            #[cfg(not(windows))]
            Special::RecycleBin => Err("unsupported".into()),
        }
    }
}

/// Steam's install folder, from its registry entry.
fn steam_folder() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let path = sys::user_registry_string(r"Software\Valve\Steam", "SteamPath")?;
        let path = PathBuf::from(path.replace('/', "\\"));
        path.is_dir().then_some(path)
    }
    #[cfg(not(windows))]
    {
        None
    }
}

/// Dropbox folders moved away from the profile, from Dropbox's own `info.json`.
fn dropbox_folders() -> Vec<PathBuf> {
    let mut found = Vec::new();
    for var in ["APPDATA", "LOCALAPPDATA"] {
        let Some(base) = std::env::var_os(var) else {
            continue;
        };
        let info = PathBuf::from(base).join("Dropbox").join("info.json");
        let Ok(text) = std::fs::read_to_string(info) else {
            continue;
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        if let Some(accounts) = json.as_object() {
            for account in accounts.values() {
                if let Some(path) = account.get("path").and_then(|p| p.as_str()) {
                    found.push(PathBuf::from(path));
                }
            }
        }
    }
    found
}

/// Krak's clips folder (Krak: `%APPDATA%\com.kryostudio.krak\settings.json`, `clipsDir`), and its
/// default place.
fn krak_library() -> Vec<PathBuf> {
    let mut found = Vec::new();
    if let Some(appdata) = std::env::var_os("APPDATA") {
        let settings = PathBuf::from(appdata)
            .join("com.kryostudio.krak")
            .join("settings.json");
        if let Ok(text) = std::fs::read_to_string(settings)
            && let Ok(json) = serde_json::from_str::<serde_json::Value>(&text)
            && let Some(dir) = json.get("clipsDir").and_then(|d| d.as_str())
        {
            found.push(PathBuf::from(dir));
        }
    }
    found
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn reads_this_pc() {
        let vars = variables();
        assert!(vars.contains_key("LOCALAPPDATA"));
        assert!(vars.contains_key("LOCALAPPDATALOW"));

        let running = running_programs();
        assert!(!running.is_empty());
        assert!(running.iter().all(|exe| exe == &exe.to_lowercase()));

        let protected = protected_folders();
        let profile = PathBuf::from(std::env::var("USERPROFILE").unwrap());
        assert!(protected.iter().any(|p| p.starts_with(&profile)));

        let disks = disks();
        assert!(
            disks
                .first()
                .is_some_and(|d| d.system && d.total >= d.free && d.total > 0)
        );
        assert!(free_space(&profile).is_some());
        // Measuring the Recycle Bin never fails on a real PC.
        assert!(WindowsSpecials.measure(Special::RecycleBin).is_some());
    }

    #[test]
    fn the_context_protects_documents() {
        let ctx = context(&[]);
        let docs = sys::known_folders();
        assert!(!docs.is_empty());
        assert!(docs.iter().all(|d| ctx.protected.covers(d)));
    }
}

/// Started without a console (the installed app), what Skrap prints goes to `<dir>/skrap.log`,
/// the previous run's kept as `skrap.old.log`. Never sent anywhere. Run from a terminal, it
/// stays there.
pub fn log_to_file(dir: &Path) {
    #[cfg(windows)]
    {
        let _ = std::fs::create_dir_all(dir);
        let path = dir.join("skrap.log");
        let _ = std::fs::rename(&path, dir.join("skrap.old.log"));
        if let Ok(file) = std::fs::File::create(&path) {
            sys::stderr_to(file);
        }
    }
    #[cfg(not(windows))]
    let _ = dir;
}
