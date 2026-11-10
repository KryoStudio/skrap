//! Texts written in the rules themselves, in every language they come with.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A text in several languages, `{ en = "...", fr = "..." }`. English is required: it is what a
/// language without its own version shows.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Text(BTreeMap<String, String>);

impl Text {
    /// The text in `lang` (a two-letter code), else in English.
    pub fn get(&self, lang: &str) -> &str {
        self.0
            .get(lang)
            .or_else(|| self.0.get("en"))
            .map_or("", String::as_str)
    }

    pub fn has(&self, lang: &str) -> bool {
        self.0.get(lang).is_some_and(|text| !text.trim().is_empty())
    }

    pub fn languages(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(String::as_str)
    }
}

impl<const N: usize> From<[(&str, &str); N]> for Text {
    fn from(entries: [(&str, &str); N]) -> Self {
        Self(
            entries
                .into_iter()
                .map(|(lang, text)| (lang.to_owned(), text.to_owned()))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falls_back_to_english() {
        let text = Text::from([("en", "Cache"), ("fr", "Cache FR")]);
        assert_eq!(text.get("fr"), "Cache FR");
        assert_eq!(text.get("de"), "Cache");
        assert_eq!(Text::default().get("fr"), "");
        assert!(text.has("fr"));
        assert!(!text.has("ja"));
    }
}
