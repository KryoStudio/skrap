//! The Reserve: cleaned files are moved here, not deleted, and can come back in one click until
//! they are freed for good (after 7 days by default, or sooner when the disk runs low).
//!
//! A move on the same disk is instant whatever the size, so each disk has its own Reserve: on
//! the system disk, Skrap's local app data folder; on the others, a hidden `.skrap-reserve`
//! folder at the root. Each clean is a batch: `<store>/<id>/batch.json` (when), `items.jsonl`
//! (one line per file: where it came from), and `files/` (the files, numbered). The line is
//! written before the file is moved, so a crash never leaves a file nobody knows the origin of;
//! the small JSON files are written through a temporary file, so a crash never leaves half of
//! one. Putting files back only ever removes what went back: a file the list does not account
//! for stays where it is.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, ErrorKind, Write};
use std::path::{Component, Path, PathBuf, Prefix};

use serde::{Deserialize, Serialize};

use crate::fsx::{self, FailReason};

const DISK_DIR: &str = ".skrap-reserve";
const STORES: &str = "stores.json";
const BATCH: &str = "batch.json";
const ITEMS: &str = "items.jsonl";
const FILES: &str = "files";

pub struct Reserve {
    home: PathBuf,
    /// Hides a Reserve folder created at a disk's root (a Windows attribute).
    hide: fn(&Path),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchInfo {
    id: String,
    created: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub original: PathBuf,
    /// File name under `files/`.
    pub stored: String,
    pub size: u64,
    pub rule: String,
}

/// A batch as the app shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchSummary {
    pub id: String,
    /// Unix ms.
    pub created: u64,
    pub bytes: u64,
    pub files: u64,
    /// The rules its files came from.
    pub rules: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreReport {
    pub files: u64,
    pub bytes: u64,
    pub failures: Vec<(PathBuf, FailReason)>,
}

impl Reserve {
    /// `home`: the Reserve on the system disk (inside Skrap's local app data).
    pub fn new(home: PathBuf, hide: fn(&Path)) -> Self {
        Self { home, hide }
    }

    pub fn home(&self) -> &Path {
        &self.home
    }

    /// Every Reserve folder, on every disk. No scan ever goes in.
    pub fn store_dirs(&self) -> Vec<PathBuf> {
        self.stores()
    }

    /// Starts a batch. Nothing is written until the first file comes in.
    pub fn deposit(&self, created: u64) -> Deposit<'_> {
        let mut id = created.to_string();
        let mut n = 1;
        while self.stores().iter().any(|store| store.join(&id).exists()) {
            n += 1;
            id = format!("{created}-{n}");
        }
        Deposit {
            reserve: self,
            info: BatchInfo { id, created },
            parts: HashMap::new(),
        }
    }

    /// Every batch, newest first.
    pub fn list(&self) -> Vec<BatchSummary> {
        summarize(self.parts())
    }

    /// The batches' parts on the system disk only, newest first: freeing them is what gives
    /// that disk space back.
    pub fn home_list(&self) -> Vec<BatchSummary> {
        summarize(
            self.parts()
                .into_iter()
                .filter(|p| self.is_home(p))
                .collect(),
        )
    }

    /// The files of a batch, where they came from.
    pub fn entries(&self, id: &str) -> Vec<Entry> {
        self.parts()
            .iter()
            .filter(|part| part.info.id == id)
            .flat_map(present)
            .collect()
    }

    /// Puts a batch's files back where they were (`only`: just these originals). A file whose
    /// place is taken stays in the Reserve; nothing already there is ever replaced.
    pub fn restore(&self, id: &str, only: Option<&[PathBuf]>) -> RestoreReport {
        let mut report = RestoreReport::default();
        for part in self.parts().into_iter().filter(|p| p.info.id == id) {
            let mut kept = Vec::new();
            for entry in read_items(&part.dir) {
                let stored = part.dir.join(FILES).join(&entry.stored);
                if !stored.exists() {
                    continue; // never moved (a crash), or already back
                }
                if only.is_some_and(|only| !only.contains(&entry.original)) {
                    kept.push(entry);
                    continue;
                }
                match put_back(&stored, &entry.original) {
                    Ok(()) => {
                        report.files += 1;
                        report.bytes += entry.size;
                    }
                    Err(reason) => {
                        report.failures.push((entry.original.clone(), reason));
                        kept.push(entry);
                    }
                }
            }
            // The batch's folder goes only once its `files/` is empty: a file the list does not
            // account for (a damaged line) is never deleted by putting the others back.
            if kept.is_empty() && fs::remove_dir(part.dir.join(FILES)).is_ok() {
                let _ = fs::remove_file(part.dir.join(ITEMS));
                let _ = fs::remove_file(part.dir.join(BATCH));
                let _ = fs::remove_dir(&part.dir);
            } else {
                let _ = write_items(&part.dir, &kept);
            }
        }
        report
    }

    /// Frees a batch for good, on every disk. Returns the bytes freed.
    pub fn purge(&self, id: &str) -> u64 {
        self.parts()
            .iter()
            .filter(|part| part.info.id == id)
            .map(remove_part)
            .sum()
    }

    /// Frees, for good, a batch's part on the system disk only. Returns the bytes freed.
    pub fn purge_home(&self, id: &str) -> u64 {
        self.parts()
            .iter()
            .filter(|part| part.info.id == id && self.is_home(part))
            .map(remove_part)
            .sum()
    }

    /// Frees the batches created before `before` (Unix ms). Returns the bytes freed.
    pub fn purge_before(&self, before: u64) -> u64 {
        self.list()
            .iter()
            .filter(|batch| batch.created < before)
            .map(|batch| self.purge(&batch.id))
            .sum()
    }

    pub fn purge_all(&self) -> u64 {
        self.list().iter().map(|batch| self.purge(&batch.id)).sum()
    }

    fn is_home(&self, part: &Part) -> bool {
        part.dir.parent() == Some(self.home.as_path())
    }

    /// The Reserve folders in use: the home one and those on other disks.
    fn stores(&self) -> Vec<PathBuf> {
        let mut stores = vec![self.home.clone()];
        stores.extend(self.other_stores().unwrap_or_default());
        stores
    }

    /// The Reserve folders on other disks, `None` when their list cannot be read (then no new
    /// one is made, rather than writing a list that would lose track of the others).
    fn other_stores(&self) -> Option<Vec<PathBuf>> {
        match fs::read_to_string(self.home.join(STORES)) {
            Ok(text) => serde_json::from_str(&text).ok(),
            Err(err) if err.kind() == ErrorKind::NotFound => Some(Vec::new()),
            Err(_) => None,
        }
    }

    fn parts(&self) -> Vec<Part> {
        let mut parts = Vec::new();
        for store in self.stores() {
            let Ok(entries) = fs::read_dir(&store) else {
                continue;
            };
            for entry in entries.flatten() {
                if !entry.file_type().is_ok_and(|t| t.is_dir()) {
                    continue;
                }
                let dir = entry.path();
                let info = fs::read_to_string(dir.join(BATCH))
                    .ok()
                    .and_then(|text| serde_json::from_str::<BatchInfo>(&text).ok())
                    // Its own description is unreadable but its list is there: the folder's
                    // name is the id and its date the batch's.
                    .or_else(|| {
                        dir.join(ITEMS).is_file().then(|| BatchInfo {
                            id: entry.file_name().to_string_lossy().into_owned(),
                            created: entry
                                .metadata()
                                .ok()
                                .and_then(|m| fsx::modified_ms(&m))
                                .unwrap_or(0),
                        })
                    });
                if let Some(info) = info {
                    parts.push(Part { dir, info });
                }
            }
        }
        parts
    }

    /// The Reserve folder for a file, on its own disk, created if needed.
    fn store_for(&self, path: &Path) -> Option<PathBuf> {
        let disk = volume(path)?;
        if volume(&self.home).as_deref() == Some(disk.as_str()) {
            return Some(self.home.clone());
        }
        let mut known = self.other_stores()?;
        let store = PathBuf::from(format!("{disk}\\")).join(DISK_DIR);
        if !store.is_dir() {
            fs::create_dir_all(&store).ok()?;
            (self.hide)(&store);
        }
        if !known.contains(&store) {
            known.push(store.clone());
            fs::create_dir_all(&self.home).ok()?;
            let json = serde_json::to_vec_pretty(&known).ok()?;
            fsx::write_atomic(&self.home.join(STORES), &json).ok()?;
        }
        Some(store)
    }
}

struct Part {
    dir: PathBuf,
    info: BatchInfo,
}

/// Batches as the app shows them, from their parts on every disk, newest first. A batch with
/// no file left is not one.
fn summarize(parts: Vec<Part>) -> Vec<BatchSummary> {
    let mut batches: BTreeMap<String, BatchSummary> = BTreeMap::new();
    let mut rules: HashMap<String, BTreeSet<String>> = HashMap::new();
    for part in &parts {
        let summary = batches
            .entry(part.info.id.clone())
            .or_insert_with(|| BatchSummary {
                id: part.info.id.clone(),
                created: part.info.created,
                bytes: 0,
                files: 0,
                rules: Vec::new(),
            });
        let names = rules.entry(part.info.id.clone()).or_default();
        for entry in present(part) {
            summary.bytes += entry.size;
            summary.files += 1;
            names.insert(entry.rule);
        }
    }
    let mut list: Vec<BatchSummary> = batches
        .into_values()
        .map(|mut summary| {
            summary.rules = rules
                .remove(&summary.id)
                .unwrap_or_default()
                .into_iter()
                .collect();
            summary
        })
        .filter(|summary| summary.files > 0)
        .collect();
    list.sort_by(|a, b| b.created.cmp(&a.created).then(b.id.cmp(&a.id)));
    list
}

/// A part's files that are really in the Reserve.
fn present(part: &Part) -> Vec<Entry> {
    read_items(&part.dir)
        .into_iter()
        .filter(|e| part.dir.join(FILES).join(&e.stored).exists())
        .collect()
}

/// Deletes a part with its files. Returns the bytes it held, or 0 when it could not go whole.
fn remove_part(part: &Part) -> u64 {
    let bytes = present(part).iter().map(|e| e.size).sum();
    if fs::remove_dir_all(&part.dir).is_ok() {
        bytes
    } else {
        0
    }
}

/// A batch being filled by a clean.
pub struct Deposit<'a> {
    reserve: &'a Reserve,
    info: BatchInfo,
    /// Per Reserve folder: the batch's folder there, its open item list, and the next number.
    parts: HashMap<PathBuf, OpenPart>,
}

struct OpenPart {
    dir: PathBuf,
    items: File,
    next: u64,
}

impl Deposit<'_> {
    pub fn id(&self) -> &str {
        &self.info.id
    }

    /// Moves `path` into the Reserve.
    pub fn put(&mut self, path: &Path, size: u64, rule: &str) -> Result<(), FailReason> {
        let store = self.reserve.store_for(path).ok_or(FailReason::NoReserve)?;
        if !self.parts.contains_key(&store) {
            let part = self.open(&store).map_err(|e| fsx::classify(&e))?;
            self.parts.insert(store.clone(), part);
        }
        let part = self.parts.get_mut(&store).ok_or(FailReason::Other)?;
        let stored = part.next.to_string();
        part.next += 1;
        let entry = Entry {
            original: path.to_path_buf(),
            stored: stored.clone(),
            size,
            rule: rule.to_owned(),
        };
        let mut line = serde_json::to_vec(&entry).map_err(|_| FailReason::Other)?;
        line.push(b'\n');
        part.items
            .write_all(&line)
            .and_then(|()| part.items.flush())
            .map_err(|e| fsx::classify(&e))?;
        fs::rename(path, part.dir.join(FILES).join(stored)).map_err(|e| fsx::classify(&e))
    }

    /// The batch's folder in a Reserve folder: always a new one, never written into twice.
    fn open(&self, store: &Path) -> io::Result<OpenPart> {
        fs::create_dir_all(store)?;
        let dir = store.join(&self.info.id);
        fs::create_dir(&dir)?;
        fs::create_dir(dir.join(FILES))?;
        fsx::write_atomic(&dir.join(BATCH), &serde_json::to_vec_pretty(&self.info)?)?;
        let items = OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(dir.join(ITEMS))?;
        Ok(OpenPart {
            dir,
            items,
            next: 0,
        })
    }

    /// The batch id, if anything went in.
    pub fn finish(self) -> Option<String> {
        (!self.parts.is_empty()).then_some(self.info.id)
    }
}

fn read_items(dir: &Path) -> Vec<Entry> {
    let Ok(file) = File::open(dir.join(ITEMS)) else {
        return Vec::new();
    };
    BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| serde_json::from_str(&line).ok())
        .collect()
}

fn write_items(dir: &Path, items: &[Entry]) -> io::Result<()> {
    let mut text = Vec::new();
    for item in items {
        serde_json::to_writer(&mut text, item)?;
        text.push(b'\n');
    }
    fsx::write_atomic(&dir.join(ITEMS), &text)
}

/// Moves a file back to its place, which must be free: the name is taken first (a file that
/// appears there meanwhile makes this fail), then the file replaces that empty placeholder.
fn put_back(stored: &Path, original: &Path) -> Result<(), FailReason> {
    if fs::symlink_metadata(original).is_ok() {
        return Err(FailReason::Exists);
    }
    if let Some(parent) = original.parent() {
        fs::create_dir_all(parent).map_err(|e| fsx::classify(&e))?;
    }
    File::create_new(original).map_err(|e| match e.kind() {
        ErrorKind::AlreadyExists => FailReason::Exists,
        _ => fsx::classify(&e),
    })?;
    fs::rename(stored, original).map_err(|e| {
        let _ = fs::remove_file(original);
        fsx::classify(&e)
    })
}

/// `C:` for a path on drive C, `None` for a network share.
fn volume(path: &Path) -> Option<String> {
    match path.components().next() {
        Some(Component::Prefix(prefix)) => match prefix.kind() {
            Prefix::Disk(d) | Prefix::VerbatimDisk(d) => {
                Some(format!("{}:", char::from(d).to_ascii_uppercase()))
            }
            _ => None,
        },
        Some(Component::RootDir) if cfg!(not(windows)) => Some(String::new()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (tempfile::TempDir, Reserve, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let reserve = Reserve::new(dir.path().join("reserve"), |_| {});
        let data = dir.path().join("data");
        fs::create_dir_all(data.join("cache")).unwrap();
        (dir, reserve, data)
    }

    #[test]
    fn moves_files_in_and_back() {
        let (_dir, reserve, data) = setup();
        let a = data.join("cache").join("a.bin");
        let b = data.join("b.bin");
        fs::write(&a, vec![1u8; 10]).unwrap();
        fs::write(&b, vec![2u8; 5]).unwrap();

        let mut deposit = reserve.deposit(1_000);
        deposit.put(&a, 10, "x.a").unwrap();
        deposit.put(&b, 5, "x.b").unwrap();
        let id = deposit.finish().unwrap();
        assert!(!a.exists() && !b.exists());

        let list = reserve.list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, id);
        assert_eq!((list[0].bytes, list[0].files), (15, 2));
        assert_eq!(list[0].rules, vec!["x.a".to_owned(), "x.b".to_owned()]);

        // The folder it came from is gone too: it comes back with the file.
        fs::remove_dir_all(data.join("cache")).unwrap();
        let report = reserve.restore(&id, None);
        assert_eq!((report.files, report.bytes), (2, 15));
        assert!(report.failures.is_empty());
        assert_eq!(fs::read(&a).unwrap(), vec![1u8; 10]);
        assert!(b.exists());
        assert!(reserve.list().is_empty());
    }

    #[test]
    fn restores_one_file_and_keeps_what_is_taken() {
        let (_dir, reserve, data) = setup();
        let a = data.join("a.tmp");
        let b = data.join("b.tmp");
        fs::write(&a, "a").unwrap();
        fs::write(&b, "b").unwrap();
        let mut deposit = reserve.deposit(1);
        deposit.put(&a, 1, "r.x").unwrap();
        deposit.put(&b, 1, "r.x").unwrap();
        let id = deposit.finish().unwrap();

        let report = reserve.restore(&id, Some(std::slice::from_ref(&a)));
        assert_eq!(report.files, 1);
        assert!(a.exists() && !b.exists());

        fs::write(&b, "new b").unwrap();
        let report = reserve.restore(&id, None);
        assert_eq!(report.failures, vec![(b.clone(), FailReason::Exists)]);
        assert_eq!(fs::read_to_string(&b).unwrap(), "new b");
        assert_eq!(reserve.list()[0].files, 1);
    }

    #[test]
    fn purges_old_batches() {
        let (_dir, reserve, data) = setup();
        for (n, created) in [(0, 100u64), (1, 200), (2, 300)] {
            let file = data.join(format!("{n}.tmp"));
            fs::write(&file, "xx").unwrap();
            let mut deposit = reserve.deposit(created);
            deposit.put(&file, 2, "r.x").unwrap();
            deposit.finish().unwrap();
        }
        assert_eq!(reserve.purge_before(250), 4);
        let left = reserve.list();
        assert_eq!(left.len(), 1);
        assert_eq!(left[0].created, 300);
        assert_eq!(reserve.purge_all(), 2);
        assert!(reserve.list().is_empty());
    }

    #[test]
    fn same_time_batches_get_their_own_id() {
        let (_dir, reserve, data) = setup();
        let mut ids = Vec::new();
        for n in 0..2 {
            let file = data.join(format!("{n}.tmp"));
            fs::write(&file, "x").unwrap();
            let mut deposit = reserve.deposit(42);
            deposit.put(&file, 1, "r.x").unwrap();
            ids.push(deposit.finish().unwrap());
        }
        assert_ne!(ids[0], ids[1]);
        assert_eq!(reserve.list().len(), 2);
    }

    #[test]
    fn putting_back_never_deletes_what_the_list_does_not_account_for() {
        let (_dir, reserve, data) = setup();
        let mut deposit = reserve.deposit(5);
        for n in 0..3 {
            let file = data.join(format!("{n}.tmp"));
            fs::write(&file, "x").unwrap();
            deposit.put(&file, 1, "r.x").unwrap();
        }
        let id = deposit.finish().unwrap();
        // Damage the second line of the list (a power cut, a bad disk).
        let items = reserve.home().join(&id).join(ITEMS);
        let text = fs::read_to_string(&items).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        fs::write(
            &items,
            format!(
                "{}
{{broken
{}
",
                lines[0], lines[2]
            ),
        )
        .unwrap();

        let report = reserve.restore(&id, None);
        assert_eq!(report.files, 2);
        assert!(data.join("0.tmp").exists() && data.join("2.tmp").exists());
        // The file the list lost track of is still in the Reserve, not deleted.
        assert!(reserve.home().join(&id).join(FILES).join("1").exists());
    }

    #[test]
    fn a_batch_whose_description_is_lost_is_still_listed() {
        let (_dir, reserve, data) = setup();
        let file = data.join("a.tmp");
        fs::write(&file, "abc").unwrap();
        let mut deposit = reserve.deposit(9);
        deposit.put(&file, 3, "r.x").unwrap();
        let id = deposit.finish().unwrap();
        fs::write(reserve.home().join(&id).join(BATCH), "{torn").unwrap();
        let list = reserve.list();
        assert_eq!(list.len(), 1);
        assert_eq!((list[0].id.as_str(), list[0].bytes), (id.as_str(), 3));
        assert_eq!(reserve.restore(&id, None).files, 1);
    }

    #[test]
    fn the_system_disk_part_is_listed_and_freed_on_its_own() {
        let (_dir, reserve, data) = setup();
        let file = data.join("a.tmp");
        fs::write(&file, "abcd").unwrap();
        let mut deposit = reserve.deposit(1);
        deposit.put(&file, 4, "r.x").unwrap();
        let id = deposit.finish().unwrap();
        assert_eq!(reserve.home_list().len(), 1);
        assert_eq!(reserve.purge_home(&id), 4);
        assert!(reserve.list().is_empty());
        assert!(reserve.store_dirs().contains(&reserve.home().to_path_buf()));
    }

    #[test]
    fn an_empty_deposit_leaves_nothing() {
        let (_dir, reserve, _data) = setup();
        assert_eq!(reserve.deposit(7).finish(), None);
        assert!(!reserve.home().exists());
    }
}
