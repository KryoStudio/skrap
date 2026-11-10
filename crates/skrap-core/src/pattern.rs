//! From a rule's pattern (`%LOCALAPPDATA%\Google\Chrome\User Data\*\Cache`) to the folders and
//! files it names on this PC.

use std::fs;
use std::path::{Path, PathBuf};

use crate::Context;
use crate::fsx;

pub fn has_wildcard(part: &str) -> bool {
    part.contains(['*', '?'])
}

/// Whether `name` matches `pattern` (`*` any run of characters, `?` one), ignoring case like
/// Windows does.
pub fn matches(pattern: &str, name: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().flat_map(char::to_lowercase).collect();
    let name: Vec<char> = name.chars().flat_map(char::to_lowercase).collect();
    let (mut p, mut n) = (0, 0);
    // Where to resume after the last `*`: (pattern position after it, name position).
    let mut star: Option<(usize, usize)> = None;
    while n < name.len() {
        match pattern.get(p) {
            Some('*') => {
                star = Some((p + 1, n));
                p += 1;
            }
            Some(&c) if c == '?' || c == name[n] => {
                p += 1;
                n += 1;
            }
            _ => match star {
                Some((after, from)) => {
                    p = after;
                    n = from + 1;
                    star = Some((after, from + 1));
                }
                None => return false,
            },
        }
    }
    pattern[p..].iter().all(|&c| c == '*')
}

/// Replaces the leading `%VARIABLE%`. `None` when the variable is unknown on this PC (its app or
/// folder is not there), which simply means the pattern names nothing.
pub fn expand(pattern: &str, ctx: &Context) -> Option<String> {
    let Some(rest) = pattern.strip_prefix('%') else {
        return Some(pattern.to_owned());
    };
    let (name, rest) = rest.split_once('%')?;
    let value = ctx.var(name)?;
    if value.is_empty() {
        return None;
    }
    Some(format!("{}{rest}", value.trim_end_matches(['\\', '/'])))
}

/// The existing files and folders an expanded pattern names. Wildcards can be in any part; no
/// junction or symbolic link is ever followed or returned.
pub fn resolve(expanded: &str) -> Vec<PathBuf> {
    let (root, parts) = split_root(expanded);
    let Some(root) = root else {
        return Vec::new();
    };
    let mut found = vec![root];
    for part in parts {
        let mut next = Vec::new();
        for dir in &found {
            if has_wildcard(part) {
                let Ok(entries) = fs::read_dir(dir) else {
                    continue;
                };
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let Some(name) = name.to_str() else { continue };
                    if matches(part, name) && entry.metadata().is_ok_and(|m| !fsx::is_link(&m)) {
                        next.push(entry.path());
                    }
                }
            } else {
                let path = dir.join(part);
                if fs::symlink_metadata(&path).is_ok_and(|m| !fsx::is_link(&m)) {
                    next.push(path);
                }
            }
        }
        // Only folders can be gone through; the last part may name files.
        found = next;
        found.sort();
        found.dedup();
        if found.is_empty() {
            break;
        }
    }
    found
}

/// Splits `C:\a\b` into `C:\` and `[a, b]` (also `\\server\share\...` and `/a/b`).
fn split_root(path: &str) -> (Option<PathBuf>, Vec<&str>) {
    let parts: Vec<&str> = path.split(['\\', '/']).collect();
    let (root, skip) = match parts.as_slice() {
        ["", "", server, share, ..] if !server.is_empty() && !share.is_empty() => {
            (format!(r"\\{server}\{share}\"), 4)
        }
        [drive, ..] if drive.len() == 2 && drive.ends_with(':') => (format!("{drive}\\"), 1),
        ["", ..] if cfg!(not(windows)) => ("/".to_owned(), 1),
        _ => return (None, Vec::new()),
    };
    let rest = parts[skip..]
        .iter()
        .copied()
        .filter(|p| !p.is_empty())
        .collect();
    (Some(PathBuf::from(root)), rest)
}

/// Whether `path` is matched by an `exclude` entry: a name pattern (`*.lock`) is checked against
/// the file name, a full pattern (with separators, already expanded) against the whole path.
pub fn excluded(path: &Path, excludes: &[String]) -> bool {
    excludes.iter().any(|exclude| {
        if exclude.contains(['\\', '/']) {
            let whole = path.to_string_lossy();
            matches(&exclude.replace('/', "\\"), &whole.replace('/', "\\"))
        } else {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| matches(exclude, name))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::context;

    #[test]
    fn wildcards() {
        assert!(matches("*", ""));
        assert!(matches("*", "anything"));
        assert!(matches("thumbcache_*.db", "thumbcache_1280.db"));
        assert!(matches("Thumbcache_*.DB", "thumbcache_idx.db"));
        assert!(!matches("thumbcache_*.db", "iconcache_1280.db"));
        assert!(matches("a?c", "abc"));
        assert!(!matches("a?c", "ac"));
        assert!(matches("*cache*", "GPUCache"));
        assert!(matches("*.log", "a.b.log"));
        assert!(!matches("*.log", "a.log.txt"));
        assert!(matches("**a", "bba"));
        assert!(matches("Profile *", "profile 2"));
        assert!(matches("É*", "école"));
    }

    #[test]
    fn expands_the_leading_variable() {
        let ctx = context(&[("LOCALAPPDATA", r"C:\Users\me\AppData\Local\")]);
        assert_eq!(
            expand(r"%LocalAppData%\NVIDIA\DXCache", &ctx).as_deref(),
            Some(r"C:\Users\me\AppData\Local\NVIDIA\DXCache")
        );
        assert_eq!(expand(r"%STEAM%\logs", &ctx), None);
    }

    #[test]
    fn resolves_wildcards_in_any_part() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path();
        for profile in ["Default", "Profile 1", "System Profile"] {
            fs::create_dir_all(base.join("User Data").join(profile).join("Cache")).unwrap();
        }
        fs::create_dir_all(base.join("User Data").join("Profile 2")).unwrap();
        fs::write(base.join("User Data").join("Local State"), "x").unwrap();

        let pattern = format!(r"{}\User Data\*\Cache", base.display());
        let found = resolve(&pattern);
        assert_eq!(found.len(), 3, "{found:?}");
        assert!(found.iter().all(|p| p.ends_with("Cache")));

        let pattern = format!(r"{}\User Data\Profile *\Cache", base.display());
        assert_eq!(resolve(&pattern).len(), 1);

        let pattern = format!(r"{}\User Data\Local*", base.display());
        assert_eq!(
            resolve(&pattern),
            vec![base.join("User Data").join("Local State")]
        );

        let pattern = format!(r"{}\Nope\*", base.display());
        assert!(resolve(&pattern).is_empty());
    }

    #[test]
    fn excludes_by_name_or_path() {
        let path = Path::new(r"C:\Cache\data\index.lock");
        assert!(excluded(path, &["*.lock".into()]));
        assert!(!excluded(path, &["*.tmp".into()]));
        assert!(excluded(path, &[r"C:\cache\data\*".into()]));
        assert!(!excluded(path, &[r"C:\cache\other\*".into()]));
    }
}
