//! The rules Skrap ships with: `rules/*.toml`, embedded in the binary and validated by the
//! tests below, so a broken rule never reaches a release.

use skrap_core::{RuleError, RuleSet};

/// The rule files, in the order the app shows their categories.
pub const FILES: &[(&str, &str)] = &[
    ("system.toml", include_str!("../../../rules/system.toml")),
    ("gaming.toml", include_str!("../../../rules/gaming.toml")),
    (
        "browsers.toml",
        include_str!("../../../rules/browsers.toml"),
    ),
    ("apps.toml", include_str!("../../../rules/apps.toml")),
];

/// The languages every built-in rule speaks.
pub const LANGUAGES: &[&str] = &["en", "fr"];

pub fn builtin() -> Result<RuleSet, RuleError> {
    RuleSet::from_toml(FILES)
}

#[cfg(test)]
mod tests {
    use super::*;
    use skrap_core::{Category, Risk};

    #[test]
    fn every_rule_loads() {
        let rules = builtin().unwrap();
        assert!(rules.len() >= 30, "{} rules", rules.len());
    }

    #[test]
    fn every_rule_speaks_every_language() {
        for rule in builtin().unwrap().rules() {
            for lang in LANGUAGES {
                for (field, text) in [
                    ("group", &rule.group),
                    ("name", &rule.name),
                    ("explain", &rule.explain),
                ] {
                    assert!(text.has(lang), "{}: {field} has no {lang} text", rule.id);
                }
            }
            let explain = rule.explain.get("en");
            assert!(
                explain.ends_with('.'),
                "{}: explain should be a sentence",
                rule.id
            );
        }
    }

    #[test]
    fn ids_start_with_their_category() {
        for rule in builtin().unwrap().rules() {
            let prefix = match rule.category {
                Category::System => "system.",
                Category::Browsers => "browsers.",
                Category::Gaming => "gaming.",
                Category::Apps => "apps.",
            };
            assert!(rule.id.starts_with(prefix), "{}", rule.id);
        }
    }

    #[test]
    fn nothing_expert_is_shipped_yet() {
        // Expert rules need the restore point and the admin helper (phase 2).
        assert!(
            builtin()
                .unwrap()
                .rules()
                .iter()
                .all(|r| r.risk != Risk::Expert)
        );
    }

    #[test]
    fn no_rule_aims_at_the_users_own_folders() {
        let forbidden = [
            "documents",
            "desktop",
            "pictures",
            "videos",
            "music",
            "downloads",
            "saved games",
            "onedrive",
            "dropbox",
            "google drive",
            "my games",
            "krak",
        ];
        for rule in builtin().unwrap().rules() {
            for path in &rule.paths {
                let lower = path.to_lowercase();
                for word in forbidden {
                    assert!(
                        !lower.split('\\').any(|part| part == word),
                        "{}: {path} names {word}",
                        rule.id
                    );
                }
            }
        }
    }
}
