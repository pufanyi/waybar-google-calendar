use crate::storage::paths;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
    pub language: Option<Language>,
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
