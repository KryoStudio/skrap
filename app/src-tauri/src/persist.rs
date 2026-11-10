//! The app's small JSON files: tolerant reads, atomic writes, a backup of a corrupt file.

use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Reads `path`, or the default when it is missing. A file that does not parse is kept as
/// `<name>.bak` before the defaults take its place.
pub fn load<T: DeserializeOwned + Default>(path: &Path) -> T {
    match fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|err| {
            eprintln!(
                "skrap: {} is unreadable ({err}), using defaults",
                path.display()
            );
            let _ = fs::copy(path, sibling(path, "bak"));
            T::default()
        }),
        Err(err) if err.kind() == ErrorKind::NotFound => T::default(),
        Err(err) => {
            eprintln!("skrap: cannot read {}: {err}", path.display());
            T::default()
        }
    }
}

/// Writes `value` as pretty JSON through a temporary file, so a crash never leaves half a file.
pub fn save<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let json = serde_json::to_vec_pretty(value).map_err(io::Error::other)?;
    let tmp = sibling(path, "tmp");
    fs::write(&tmp, json)?;
    fs::rename(&tmp, path)
}

/// `settings.json` -> `settings.json.<suffix>`.
fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".");
    name.push(suffix);
    path.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Serialize, serde::Deserialize, Default, PartialEq, Debug)]
    struct Thing {
        n: u32,
    }

    #[test]
    fn round_trips_and_survives_corruption() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub").join("thing.json");
        assert_eq!(load::<Thing>(&path), Thing::default());
        save(&path, &Thing { n: 3 }).unwrap();
        assert_eq!(load::<Thing>(&path), Thing { n: 3 });
        fs::write(&path, "{ not json").unwrap();
        assert_eq!(load::<Thing>(&path), Thing::default());
        assert!(dir.path().join("sub").join("thing.json.bak").exists());
    }
}
