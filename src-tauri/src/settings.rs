//! Persistent user settings (JSON in the app config dir). Rust owns them so the
//! window/tray logic works even while the UI is hidden.

use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    // General
    pub lang: String,
    pub theme: String,
    pub hide_hubs: bool,
    pub sidebar_collapsed: bool,
    // Window
    pub always_on_top: bool,
    pub close_to_tray: bool,
    pub start_with_windows: bool,
    pub start_minimized: bool,
    // Notifications
    pub system_notifications: bool,
    pub notify_on_removal: bool,
    pub sound_on_arrival: bool,
    pub sound_on_removal: bool,
    /// "chime" | "voice"
    pub sound_style: String,
    pub sound_volume: u8,
    // Storage devices
    pub bring_to_front_on_storage: bool,
    pub open_explorer_on_storage: bool,
    // Updates
    pub check_updates_on_start: bool,
    /// Version whose release notes the user has seen (drives the "What's new" dialog).
    pub last_seen_version: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            lang: "tr".into(),
            theme: "dark".into(),
            hide_hubs: true,
            sidebar_collapsed: false,
            always_on_top: false,
            close_to_tray: true,
            start_with_windows: false,
            start_minimized: false,
            system_notifications: true,
            notify_on_removal: true,
            sound_on_arrival: true,
            sound_on_removal: false,
            sound_style: "chime".into(),
            sound_volume: 70,
            bring_to_front_on_storage: false,
            open_explorer_on_storage: false,
            check_updates_on_start: true,
            last_seen_version: String::new(),
        }
    }
}

impl Settings {
    /// Clamp / normalise values coming from the UI or a hand-edited file.
    pub fn sanitized(mut self) -> Self {
        if self.lang != "tr" && self.lang != "en" {
            self.lang = "tr".into();
        }
        if self.theme != "dark" && self.theme != "light" {
            self.theme = "dark".into();
        }
        if self.sound_style != "chime" && self.sound_style != "voice" {
            self.sound_style = "chime".into();
        }
        self.sound_volume = self.sound_volume.min(100);
        self
    }
}

pub struct SettingsStore {
    path: PathBuf,
    current: Mutex<Settings>,
}

impl SettingsStore {
    pub fn load(path: PathBuf) -> Self {
        let existing = std::fs::read_to_string(&path).ok();
        let mut current = existing
            .as_deref()
            .and_then(|s| serde_json::from_str::<Settings>(s).ok())
            .unwrap_or_default()
            .sanitized();
        // Settings saved by a version older than 0.2.4 have no last_seen_version:
        // that is an update (not a fresh install), so the UI should show what's new.
        if existing.is_some() && current.last_seen_version.is_empty() {
            current.last_seen_version = "legacy".into();
        }
        Self {
            path,
            current: Mutex::new(current),
        }
    }

    pub fn get(&self) -> Settings {
        self.current.lock().map(|s| s.clone()).unwrap_or_default()
    }

    pub fn set(&self, new: Settings) -> Result<Settings, String> {
        let new = new.sanitized();
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(&new).map_err(|e| e.to_string())?;
        std::fs::write(&self.path, json).map_err(|e| e.to_string())?;
        if let Ok(mut cur) = self.current.lock() {
            *cur = new.clone();
        }
        Ok(new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_use_defaults_and_bad_values_are_fixed() {
        let s: Settings = serde_json::from_str(r#"{"lang":"de","alwaysOnTop":true,"soundVolume":250}"#).unwrap();
        let s = s.sanitized();
        assert_eq!(s.lang, "tr");
        assert!(s.always_on_top);
        assert!(s.close_to_tray);
        assert_eq!(s.sound_volume, 100);
    }

    #[test]
    fn marks_settings_from_older_versions() {
        let dir = std::env::temp_dir().join(format!("usblens-legacy-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("settings.json"), r#"{"lang":"tr"}"#).unwrap();
        assert_eq!(SettingsStore::load(dir.join("settings.json")).get().last_seen_version, "legacy");
        assert_eq!(SettingsStore::load(dir.join("missing.json")).get().last_seen_version, "");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn round_trips_through_disk() {
        let dir = std::env::temp_dir().join(format!("usblens-test-{}", std::process::id()));
        let store = SettingsStore::load(dir.join("settings.json"));
        let mut s = store.get();
        s.open_explorer_on_storage = true;
        store.set(s).unwrap();
        let again = SettingsStore::load(dir.join("settings.json"));
        assert!(again.get().open_explorer_on_storage);
        let _ = std::fs::remove_dir_all(dir);
    }
}
