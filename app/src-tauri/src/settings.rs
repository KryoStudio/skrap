//! What the user chose, saved in `<config>/settings.json`, and what Skrap counts for them in
//! `<data>/stats.json`. Unknown or missing fields take their defaults, so an older or newer file
//! always loads.

use serde::{Deserialize, Serialize};

pub const RESERVE_DAYS: &[u32] = &[3, 7, 14, 30];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// A language code, or `None` to follow Windows.
    pub language: Option<String>,
    /// Look for an update at launch (the only network call Skrap makes).
    pub check_updates: bool,
    /// How long the Reserve keeps what was cleaned.
    pub reserve_days: u32,
    /// `auto` follows Windows' animation setting, `reduced` keeps simple fades.
    pub motion: Motion,
    /// The welcome was shown.
    pub onboarded: bool,
    /// The version whose "What's new" was shown, so it shows once after each update.
    pub last_seen_version: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: None,
            check_updates: true,
            reserve_days: 7,
            motion: Motion::Auto,
            onboarded: false,
            last_seen_version: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Motion {
    Auto,
    Reduced,
}

/// A change from the page: only the fields it sends.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPatch {
    #[serde(default, deserialize_with = "double_option")]
    pub language: Option<Option<String>>,
    pub check_updates: Option<bool>,
    pub reserve_days: Option<u32>,
    pub motion: Option<Motion>,
    pub onboarded: Option<bool>,
    pub last_seen_version: Option<String>,
}

/// `"language": null` (follow Windows) differs from no `language` at all (unchanged).
fn double_option<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<Option<String>>, D::Error> {
    Option::<String>::deserialize(d).map(Some)
}

impl Settings {
    pub fn merge(&self, patch: SettingsPatch) -> Result<Self, &'static str> {
        let mut next = self.clone();
        if let Some(language) = patch.language {
            next.language = language.filter(|l| !l.is_empty());
        }
        if let Some(check) = patch.check_updates {
            next.check_updates = check;
        }
        if let Some(days) = patch.reserve_days {
            if !RESERVE_DAYS.contains(&days) {
                return Err("invalid_reserve_days");
            }
            next.reserve_days = days;
        }
        if let Some(motion) = patch.motion {
            next.motion = motion;
        }
        if let Some(onboarded) = patch.onboarded {
            next.onboarded = onboarded;
        }
        if let Some(version) = patch.last_seen_version {
            next.last_seen_version = Some(version);
        }
        Ok(next)
    }

    /// The Reserve's delay in milliseconds.
    pub fn reserve_ms(&self) -> u64 {
        u64::from(self.reserve_days) * 24 * 60 * 60 * 1000
    }
}

/// What Skrap counts for the home screen.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Stats {
    /// Everything cleaned since installation, Reserve included.
    pub cleaned_bytes: u64,
    pub cleans: u32,
    /// Unix ms.
    pub last_scan: Option<u64>,
    pub last_clean: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_take_defaults() {
        let settings: Settings =
            serde_json::from_str(r#"{"reserveDays": 14, "future": 1}"#).unwrap();
        assert_eq!(settings.reserve_days, 14);
        assert!(settings.check_updates);
        assert_eq!(settings.motion, Motion::Auto);
    }

    #[test]
    fn patches_only_what_they_carry() {
        let base = Settings {
            language: Some("fr".into()),
            ..Settings::default()
        };
        let patch: SettingsPatch = serde_json::from_str(r#"{"checkUpdates": false}"#).unwrap();
        let next = base.merge(patch).unwrap();
        assert_eq!(next.language.as_deref(), Some("fr"));
        assert!(!next.check_updates);

        let patch: SettingsPatch = serde_json::from_str(r#"{"language": null}"#).unwrap();
        assert_eq!(base.merge(patch).unwrap().language, None);

        let patch: SettingsPatch = serde_json::from_str(r#"{"reserveDays": 5}"#).unwrap();
        assert!(base.merge(patch).is_err());
    }
}
