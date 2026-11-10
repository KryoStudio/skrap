//! The folders Skrap never touches: the user's own files. This list lives in the code, not in
//! the rules, so that no rule can get around it.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct Protected {
    /// Normalized: backslashes, no trailing separator, lower case.
    roots: Vec<String>,
}

impl Protected {
    pub fn new(roots: impl IntoIterator<Item = PathBuf>) -> Self {
        let mut roots: Vec<String> = roots
            .into_iter()
            .map(|root| normalize(&root))
            .filter(|root| root.len() > 3) // never a whole drive (`c:`)
            .collect();
        roots.sort();
        roots.dedup();
        Self { roots }
    }

    pub fn roots(&self) -> impl Iterator<Item = &str> {
        self.roots.iter().map(String::as_str)
    }

    /// `path` is a protected folder or inside one.
    pub fn covers(&self, path: &Path) -> bool {
        let path = normalize(path);
        self.roots.iter().any(|root| within(&path, root))
    }

    /// A protected folder is inside `dir` (cleaning all of `dir` would reach it).
    pub fn inside(&self, dir: &Path) -> bool {
        let dir = normalize(dir);
        self.roots
            .iter()
            .any(|root| root.len() > dir.len() && within(root, &dir))
    }
}

fn within(path: &str, root: &str) -> bool {
    path == root
        || (path.len() > root.len()
            && path.starts_with(root)
            && path.as_bytes()[root.len()] == b'\\')
}

fn normalize(path: &Path) -> String {
    let text = path.to_string_lossy().replace('/', "\\");
    let text = text.strip_prefix(r"\\?\").unwrap_or(&text);
    text.trim_end_matches('\\').to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_the_folder_and_what_is_inside() {
        let protected = Protected::new([
            PathBuf::from(r"C:\Users\Me\Documents\"),
            PathBuf::from(r"D:\OneDrive"),
            PathBuf::from(r"C:\"),
        ]);
        assert!(protected.covers(Path::new(r"C:\Users\Me\Documents")));
        assert!(protected.covers(Path::new(r"c:\users\me\documents\My Games\save.dat")));
        assert!(protected.covers(Path::new(r"\\?\C:\Users\Me\Documents\x")));
        assert!(protected.covers(Path::new("D:/OneDrive/x")));
        assert!(!protected.covers(Path::new(r"C:\Users\Me\Documents2\x")));
        assert!(!protected.covers(Path::new(r"C:\Users\Me\AppData")));
        // A whole drive is never a protected root (everything would be).
        assert!(!protected.covers(Path::new(r"C:\Windows\Temp")));
    }

    #[test]
    fn knows_the_folders_that_hold_a_protected_one() {
        let protected = Protected::new([PathBuf::from(r"C:\Users\Me\Documents")]);
        assert!(protected.inside(Path::new(r"C:\Users\Me")));
        assert!(protected.inside(Path::new(r"C:\")));
        assert!(!protected.inside(Path::new(r"C:\Users\Me\Documents")));
        assert!(!protected.inside(Path::new(r"C:\Users\Me\AppData")));
        assert!(!protected.inside(Path::new(r"C:\Users\M")));
    }
}
