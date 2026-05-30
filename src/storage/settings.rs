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

pub fn read_settings() -> UserSettings {
    read_settings_from(&paths::settings_file())
}

fn read_settings_from(file: &Path) -> UserSettings {
    if !file.exists() {
        return UserSettings::default();
    }
    fs::read_to_string(file)
        .ok()
        .and_then(|content| serde_json::from_str::<UserSettings>(&content).ok())
        .unwrap_or_default()
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

pub fn translate(lang: Language, key: &'static str) -> &'static str {
    match lang {
        Language::Chinese => match key {
            "settings" => "设置",
            "calendar_timezone" => "日历与时区 (Calendar & Timezone)",
            "appearance" => "外观 (Appearance)",
            "google_account" => "Google 账号 (Google Account)",
            "calendar_id" => "日历 ID (Calendar ID):",
            "timezone" => "时区 (Timezone):",
            "theme_path" => "主题 CSS 路径 (Theme CSS Path):",
            "account_status" => "Google 账号状态 (Status)",
            "login" => "登录 (Log In)",
            "logout" => "退出登录 (Log Out)",
            "cancel" => "取消 (Cancel)",
            "save" => "保存 (Save)",
            "authenticated" => "已登录",
            "missing_token" => "未登录",
            "authenticating" => "正在登录...",
            "logged_out_please_auth" => "已退出登录，请重新认证。",
            "logged_out_success" => "退出登录成功。",
            "language" => "语言 (Language):",
            "english" => "英文 (English)",
            "chinese" => "中文 (Chinese)",
            "refresh" => "刷新",
            "close" => "关闭",
            "google_calendar" => "谷歌日历 (Google Calendar)",
            _ => key,
        },
        Language::English => match key {
            "settings" => "Settings",
            "calendar_timezone" => "Calendar & Timezone",
            "appearance" => "Appearance",
            "google_account" => "Google Account",
            "calendar_id" => "Calendar ID:",
            "timezone" => "Timezone:",
            "theme_path" => "Theme CSS Path:",
            "account_status" => "Google Account Status",
            "login" => "Log In",
            "logout" => "Log Out",
            "cancel" => "Cancel",
            "save" => "Save",
            "authenticated" => "Authenticated",
            "missing_token" => "Missing Token",
            "authenticating" => "Authenticating...",
            "logged_out_please_auth" => "Logged out. Please authenticate.",
            "logged_out_success" => "Logged out successfully.",
            "language" => "Language:",
            "english" => "English",
            "chinese" => "Chinese",
            "refresh" => "Refresh",
            "close" => "Close",
            "google_calendar" => "Google Calendar",
            _ => key,
        },
    }
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

        let settings = read_settings_from(&file);

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
        let restored = read_settings_from(&file);
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

        let settings = read_settings_from(&file);
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
    fn invalid_json_returns_defaults() {
        let file = unique_settings_file("invalid-json");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, "{not valid json").unwrap();

        let settings = read_settings_from(&file);
        cleanup(&file);

        assert!(settings.calendar.is_none());
        assert!(settings.timezone.is_none());
        assert!(settings.theme_path.is_none());
        assert_eq!(settings.language, None);
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

    #[test]
    fn translate_uses_language_specific_copy_and_falls_back_to_key() {
        assert_eq!(translate(Language::English, "settings"), "Settings");
        assert_eq!(translate(Language::Chinese, "settings"), "设置");
        assert_eq!(translate(Language::English, "unknown_key"), "unknown_key");
    }
}
