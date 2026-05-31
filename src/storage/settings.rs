use crate::storage::paths;
use serde::Deserializer;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    English,
    Chinese,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserSettings {
    pub calendar: Option<String>,
    pub timezone: Option<String>,
    pub theme_path: Option<PathBuf>,
    #[serde(default, deserialize_with = "deserialize_language")]
    pub language: Option<Language>,
}

pub fn read_settings() -> Result<UserSettings, String> {
    read_settings_from(&paths::settings_file())
}

fn read_settings_from(file: &Path) -> Result<UserSettings, String> {
    if !file.exists() {
        return Ok(UserSettings::default());
    }

    let content = fs::read_to_string(file)
        .map_err(|err| format!("Could not read settings file {}: {err}", file.display()))?;
    serde_json::from_str::<UserSettings>(&content)
        .map_err(|err| format!("Could not parse settings file {}: {err}", file.display()))
}

pub fn write_settings(settings: &UserSettings) -> Result<(), String> {
    write_settings_to(&paths::settings_file(), settings)
}

fn write_settings_to(file: &Path, settings: &UserSettings) -> Result<(), String> {
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "Could not create settings folder {}: {err}",
                parent.display()
            )
        })?;
    }
    let content = serde_json::to_string_pretty(settings)
        .map_err(|err| format!("Could not serialize settings: {err}"))?;
    fs::write(file, content).map_err(|err| format!("Could not write settings to file: {err}"))?;
    Ok(())
}

fn deserialize_language<'de, D>(deserializer: D) -> Result<Option<Language>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    let Some(serde_json::Value::String(raw)) = value else {
        return Ok(None);
    };

    Ok(match raw.as_str() {
        "english" => Some(Language::English),
        "chinese" => Some(Language::Chinese),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_settings_file(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir()
            .join(format!(
                "waybar-google-calendar-settings-test-{}-{suffix}",
                std::process::id()
            ))
            .join(name)
            .join("config.json")
    }

    fn cleanup(path: &Path) {
        if let Some(root) = path.parent().and_then(Path::parent) {
            let _ = fs::remove_dir_all(root);
        }
    }

    #[test]
    fn missing_settings_file_returns_defaults() {
        let file = unique_settings_file("missing");
        cleanup(&file);

        let settings = read_settings_from(&file).unwrap();

        assert!(settings.calendar.is_none());
        assert!(settings.timezone.is_none());
        assert!(settings.theme_path.is_none());
        assert_eq!(settings.language, None);
    }

    #[test]
    fn settings_round_trip_preserves_all_fields() {
        let file = unique_settings_file("round-trip");
        let settings = UserSettings {
            calendar: Some("team@example.com".to_string()),
            timezone: Some("Asia/Singapore".to_string()),
            theme_path: Some(PathBuf::from("/tmp/theme.css")),
            language: Some(Language::Chinese),
        };

        write_settings_to(&file, &settings).unwrap();
        let restored = read_settings_from(&file).unwrap();
        cleanup(&file);

        assert_eq!(restored.calendar.as_deref(), Some("team@example.com"));
        assert_eq!(restored.timezone.as_deref(), Some("Asia/Singapore"));
        assert_eq!(
            restored.theme_path.as_deref(),
            Some(Path::new("/tmp/theme.css"))
        );
        assert_eq!(restored.language, Some(Language::Chinese));
    }

    #[test]
    fn invalid_language_does_not_discard_other_settings() {
        let file = unique_settings_file("invalid-language");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(
            &file,
            r#"{
  "calendar": "primary",
  "timezone": "UTC",
  "theme_path": "/tmp/custom.css",
  "language": "klingon"
}"#,
        )
        .unwrap();

        let settings = read_settings_from(&file).unwrap();
        cleanup(&file);

        assert_eq!(settings.calendar.as_deref(), Some("primary"));
        assert_eq!(settings.timezone.as_deref(), Some("UTC"));
        assert_eq!(
            settings.theme_path.as_deref(),
            Some(Path::new("/tmp/custom.css"))
        );
        assert_eq!(settings.language, None);
    }

    #[test]
    fn invalid_json_returns_error() {
        let file = unique_settings_file("invalid-json");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, "{not valid json").unwrap();

        let error = read_settings_from(&file).unwrap_err();
        cleanup(&file);

        assert!(error.contains("Could not parse settings file"));
    }

    #[test]
    fn write_settings_reports_parent_creation_errors() {
        let file = unique_settings_file("blocked-parent");
        let blocked_parent = file.parent().unwrap();
        fs::create_dir_all(blocked_parent.parent().unwrap()).unwrap();
        fs::write(blocked_parent, "not a directory").unwrap();

        let error = write_settings_to(&file, &UserSettings::default()).unwrap_err();
        cleanup(&file);

        assert!(error.contains("Could not create settings folder"));
    }
}
