use crate::storage::paths;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserSettings {
    pub calendar: Option<String>,
    pub timezone: Option<String>,
    pub theme_path: Option<PathBuf>,
}

pub fn read_settings() -> UserSettings {
    let file = paths::settings_file();
    if !file.exists() {
        return UserSettings::default();
    }
    fs::read_to_string(file)
        .ok()
        .and_then(|content| serde_json::from_str::<UserSettings>(&content).ok())
        .unwrap_or_default()
}

pub fn write_settings(settings: &UserSettings) -> Result<(), String> {
    let file = paths::settings_file();
    if let Some(parent) = file.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let content = serde_json::to_string_pretty(settings)
        .map_err(|err| format!("Could not serialize settings: {err}"))?;
    fs::write(file, content).map_err(|err| format!("Could not write settings to file: {err}"))?;
    Ok(())
}
