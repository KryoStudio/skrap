//! Cleaning rules: what can go, where it is, how risky it is, and why.
//!
//! Every source (Skrap's own rules, and later BleachBit's and winapp2's) is converted to this one
//! format, readable and reviewed like code. See `docs/RULES.md`.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::pattern;
use crate::text::Text;

/// How careful to be with what a rule targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Risk {
    /// Rebuilds itself (temporary files, caches). Ticked by default, deleted for good.
    Safe,
    /// Fine to remove, but with a cost or a reason to look first (shader caches make the next
    /// game launch stutter, a cleaned history is gone). Unticked, and kept in the Reserve.
    Review,
    /// System changes. Unticked and hidden by default; a restore point comes first.
    Expert,
}

impl Risk {
    pub fn ticked_by_default(self) -> bool {
        self == Self::Safe
    }

    /// Whether cleaned files go to the Reserve instead of being deleted.
    pub fn keeps_in_reserve(self) -> bool {
        self != Self::Safe
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    System,
    Browsers,
    Gaming,
    Apps,
}

/// Targets measured and cleaned through the system rather than file by file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Special {
    RecycleBin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    /// Stable identifier, `category.app.what` (`gaming.nvidia.dxcache`).
    pub id: String,
    pub category: Category,
    pub risk: Risk,
    /// The app or place it belongs to, shown above its rules ("NVIDIA", "Chrome").
    pub group: Text,
    pub name: Text,
    /// One sentence on what it is and what happens once it is gone.
    pub explain: Text,
    /// Folder patterns: `%LOCALAPPDATA%\NVIDIA\DXCache`. A folder means everything inside it
    /// (the folder itself stays); wildcards work in any part.
    #[serde(default)]
    pub paths: Vec<String>,
    /// Names (`*.lock`) or full patterns of what to leave alone inside the targets.
    #[serde(default)]
    pub exclude: Vec<String>,
    /// Programs that use these files: while one runs, the rule is skipped.
    #[serde(default)]
    pub running: Vec<String>,
    /// Only files untouched for at least this long (a running installer's temp files stay).
    #[serde(default)]
    pub older_than_hours: Option<u32>,
    #[serde(default)]
    pub special: Option<Special>,
    /// Usually needs administrator rights (files Windows owns). Without them, what cannot be
    /// cleaned is reported.
    #[serde(default)]
    pub admin: bool,
}

impl Rule {
    pub fn ticked_by_default(&self) -> bool {
        self.risk.ticked_by_default()
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RuleError {
    #[error("{file}: {message}")]
    Parse { file: String, message: String },
    #[error("rule {id}: {message}")]
    Invalid { id: String, message: String },
    #[error("rule {0} is defined twice")]
    Duplicate(String),
}

/// Folder variables a rule can start from. The ones marked junk are junk as a whole, so a
/// wildcard may follow them directly (`%TEMP%\*`); any other needs a named folder first, so that
/// no rule can sweep a whole profile.
pub(crate) const VARIABLES: &[(&str, bool)] = &[
    ("TEMP", true),
    ("TMP", true),
    ("LOCALAPPDATA", false),
    ("LOCALAPPDATALOW", false),
    ("APPDATA", false),
    ("PROGRAMDATA", false),
    ("USERPROFILE", false),
    ("WINDIR", false),
    ("SYSTEMDRIVE", false),
    ("PROGRAMFILES", false),
    ("PROGRAMFILES(X86)", false),
    ("STEAM", false),
];

/// Whether a variable names a folder that is junk as a whole (`%TEMP%`).
pub(crate) fn is_junk_root(name: &str) -> bool {
    VARIABLES
        .iter()
        .any(|(known, junk)| *junk && known.eq_ignore_ascii_case(name))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleFile {
    #[serde(rename = "rule", default)]
    rules: Vec<Rule>,
}

/// A validated set of rules, in the order of their files.
#[derive(Debug, Clone, Default)]
pub struct RuleSet {
    rules: Vec<Rule>,
}

impl RuleSet {
    /// Parses and validates rule files, given as `(file name, TOML)`.
    pub fn from_toml(files: &[(&str, &str)]) -> Result<Self, RuleError> {
        let mut rules = Vec::new();
        for (name, source) in files {
            let file: RuleFile = toml::from_str(source).map_err(|err| RuleError::Parse {
                file: (*name).to_owned(),
                message: err.to_string(),
            })?;
            rules.extend(file.rules);
        }
        Self::new(rules)
    }

    pub fn new(rules: Vec<Rule>) -> Result<Self, RuleError> {
        let mut seen = HashSet::new();
        for rule in &rules {
            validate(rule)?;
            if !seen.insert(rule.id.as_str()) {
                return Err(RuleError::Duplicate(rule.id.clone()));
            }
        }
        Ok(Self { rules })
    }

    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    pub fn get(&self, id: &str) -> Option<&Rule> {
        self.rules.iter().find(|rule| rule.id == id)
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

fn validate(rule: &Rule) -> Result<(), RuleError> {
    let invalid = |message: String| RuleError::Invalid {
        id: rule.id.clone(),
        message,
    };
    let id_ok = !rule.id.is_empty()
        && rule.id.split('.').count() >= 2
        && rule
            .id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_'));
    if !id_ok {
        return Err(invalid(
            "the id must look like category.app.what (a-z, 0-9, _)".into(),
        ));
    }
    for (field, text) in [
        ("group", &rule.group),
        ("name", &rule.name),
        ("explain", &rule.explain),
    ] {
        if !text.has("en") {
            return Err(invalid(format!("{field} needs an English text")));
        }
    }
    match (rule.special, rule.paths.is_empty()) {
        (None, true) => return Err(invalid("no paths".into())),
        (Some(_), false) => return Err(invalid("a special rule takes no paths".into())),
        _ => {}
    }
    for path in &rule.paths {
        check_pattern(path).map_err(|message| invalid(format!("{path}: {message}")))?;
    }
    for exclude in &rule.exclude {
        if exclude.contains(['\\', '/']) {
            check_pattern(exclude).map_err(|message| invalid(format!("{exclude}: {message}")))?;
        } else if exclude.is_empty() {
            return Err(invalid("empty exclude".into()));
        }
    }
    for exe in &rule.running {
        if !exe.to_ascii_lowercase().ends_with(".exe") || exe.contains(['\\', '/']) {
            return Err(invalid(format!(
                "{exe}: running wants a program name like app.exe"
            )));
        }
    }
    Ok(())
}

/// A path pattern starts with a known variable, and only a junk folder may be followed by a
/// wildcard right away.
fn check_pattern(path: &str) -> Result<(), String> {
    let Some(rest) = path.strip_prefix('%') else {
        return Err("must start with a folder variable like %LOCALAPPDATA%".into());
    };
    let Some((name, rest)) = rest.split_once('%') else {
        return Err("unclosed variable".into());
    };
    let Some(&(_, junk)) = VARIABLES
        .iter()
        .find(|(known, _)| known.eq_ignore_ascii_case(name))
    else {
        return Err(format!("unknown variable %{name}%"));
    };
    if rest.contains('%') {
        return Err("only one variable, at the start".into());
    }
    let mut parts = rest.split(['\\', '/']);
    if parts.next() != Some("") {
        return Err("a separator must follow the variable".into());
    }
    let parts: Vec<&str> = parts.collect();
    if parts
        .iter()
        .any(|part| part.is_empty() || *part == "." || *part == "..")
    {
        return Err("empty, . or .. part".into());
    }
    match parts.first() {
        None if !junk => Err("targets a whole folder variable".into()),
        Some(first) if !junk && pattern::has_wildcard(first) => {
            Err("needs a named folder before any wildcard".into())
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(paths: &[&str]) -> Rule {
        Rule {
            id: "system.test.cache".into(),
            category: Category::System,
            risk: Risk::Safe,
            group: Text::from([("en", "Test")]),
            name: Text::from([("en", "Cache")]),
            explain: Text::from([("en", "Rebuilds itself.")]),
            paths: paths.iter().map(|p| (*p).to_owned()).collect(),
            exclude: vec![],
            running: vec![],
            older_than_hours: None,
            special: None,
            admin: false,
        }
    }

    #[test]
    fn parses_a_rule_file() {
        let toml = r#"
            [[rule]]
            id = "gaming.nvidia.dxcache"
            category = "gaming"
            risk = "review"
            group = { en = "NVIDIA" }
            name = { en = "DirectX shader cache", fr = "Cache de shaders DirectX" }
            explain = { en = "Rebuilt by the next game launch." }
            paths = ['%LOCALAPPDATA%\NVIDIA\DXCache']
            running = ["steam.exe"]
        "#;
        let set = RuleSet::from_toml(&[("gaming.toml", toml)]).unwrap();
        let rule = set.get("gaming.nvidia.dxcache").unwrap();
        assert_eq!(rule.risk, Risk::Review);
        assert!(!rule.ticked_by_default());
        assert!(rule.risk.keeps_in_reserve());
        assert_eq!(rule.name.get("fr"), "Cache de shaders DirectX");
    }

    #[test]
    fn unknown_fields_are_errors() {
        let toml = r#"
            [[rule]]
            id = "a.b"
            categry = "system"
        "#;
        assert!(matches!(
            RuleSet::from_toml(&[("x.toml", toml)]),
            Err(RuleError::Parse { .. })
        ));
    }

    #[test]
    fn rejects_broad_or_odd_patterns() {
        for bad in [
            r"C:\Temp",
            r"%NOPE%\x",
            r"%USERPROFILE%",
            r"%USERPROFILE%\*",
            r"%LOCALAPPDATA%\*\Cache",
            r"%LOCALAPPDATA%\..\x",
            r"%LOCALAPPDATA%\a\\b",
            r"%LOCALAPPDATA%x",
            r"%LOCALAPPDATA%\a\%TEMP%",
        ] {
            assert!(RuleSet::new(vec![rule(&[bad])]).is_err(), "{bad} accepted");
        }
        for good in [
            r"%TEMP%\*",
            r"%TEMP%",
            r"%LocalAppData%\Google\Chrome\User Data\*\Cache",
            r"%PROGRAMFILES(X86)%\Steam\logs",
        ] {
            assert!(RuleSet::new(vec![rule(&[good])]).is_ok(), "{good} refused");
        }
    }

    #[test]
    fn rejects_duplicates_and_bad_ids() {
        let a = rule(&[r"%TEMP%"]);
        assert_eq!(
            RuleSet::new(vec![a.clone(), a.clone()]).unwrap_err(),
            RuleError::Duplicate(a.id.clone())
        );
        let mut bad = a.clone();
        bad.id = "System.Cache".into();
        assert!(RuleSet::new(vec![bad]).is_err());
        let mut lonely = a;
        lonely.id = "cache".into();
        assert!(RuleSet::new(vec![lonely]).is_err());
    }

    #[test]
    fn special_rules_take_no_paths() {
        let mut bin = rule(&[]);
        bin.special = Some(Special::RecycleBin);
        assert!(RuleSet::new(vec![bin.clone()]).is_ok());
        bin.paths.push(r"%TEMP%".into());
        assert!(RuleSet::new(vec![bin]).is_err());
        assert!(RuleSet::new(vec![rule(&[])]).is_err());
    }

    #[test]
    fn running_wants_program_names() {
        let mut r = rule(&[r"%TEMP%"]);
        r.running = vec![r"C:\x\chrome.exe".into()];
        assert!(RuleSet::new(vec![r.clone()]).is_err());
        r.running = vec!["Chrome.EXE".into()];
        assert!(RuleSet::new(vec![r]).is_ok());
    }
}
