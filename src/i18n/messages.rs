use crate::storage::settings::Language;

pub fn translate(lang: Language, key: &'static str) -> &'static str {
    match lang {
        Language::Chinese => chinese(key),
        Language::English => english(key),
    }
}

fn chinese(key: &'static str) -> &'static str {
    match key {
        "action_required" => "需要操作",
        "account_status" => "Google 账号状态",
        "advanced" => "高级",
        "agenda" => "日程",
        "all" => "全部",
        "all_day" => "全天",
        "appearance" => "外观",
        "apply" => "应用",
        "authenticated" => "已登录",
        "authenticating" => "正在登录",
        "authorize" => "授权",
        "authorize_calendar_access" => "授权日历访问",
        "browser_token_saved_at" => "浏览器令牌保存于",
        "cached" => "已缓存",
        "calendar" => "日历",
        "calendar_api" => "Calendar API",
        "calendar_id" => "日历 ID:",
        "calendar_timezone" => "日历与时区",
        "cancel" => "取消",
        "change_calendar_view" => "切换日历视图",
        "choose_month" => "选择月份",
        "choose_year" => "选择年份",
        "chinese" => "中文",
        "client_id" => "Client ID",
        "client_secret" => "Client Secret",
        "close" => "关闭",
        "config_folder" => "配置文件夹",
        "config_folder_opened" => "配置文件夹已打开。",
        "connect_google_calendar" => "连接 Google 日历",
        "connected" => "已连接",
        "create_google_oauth_client" => "创建 Google OAuth 客户端",
        "current_status" => "当前状态",
        "english" => "英文",
        "event" => "个事件",
        "events" => "个事件",
        "failed_load_theme" => "加载主题失败",
        "failed_save_settings" => "保存设置失败",
        "google_account" => "Google 账号",
        "google_account_authenticated" => "Google Calendar 已认证，正在加载事件...",
        "google_calendar" => "Google 日历",
        "google_calendar_api_opened" => "Google Calendar API 页面已在浏览器中打开。",
        "google_calendar_credentials_saved" => {
            "Google Calendar 凭据已保存。如果事件仍未加载，请刷新或重新认证。"
        }
        "google_cloud" => "Google Cloud",
        "google_cloud_opened" => "Google Cloud 已在浏览器中打开。",
        "language" => "语言:",
        "loading" => "正在加载",
        "loading_google_calendar" => "正在加载 Google Calendar",
        "login" => "登录",
        "logged_out_cleanup_incomplete" => "已退出登录，但清理不完整",
        "logged_out_please_auth" => "已退出登录，请重新认证。",
        "logged_out_success" => "退出登录成功。",
        "logout" => "退出登录",
        "missing_token" => "未登录",
        "next_month" => "下个月",
        "next_year" => "下一年",
        "next_years" => "后几年",
        "no_loaded_events_day" => "没有已加载的事件：",
        "no_loaded_events_view" => "当前日历视图没有已加载的事件。",
        "no_oauth_client_saved" => "还没有保存 OAuth 客户端。按下面的步骤完成一次性设置。",
        "no_upcoming_events" => "没有即将到来的事件",
        "oauth_client_saved_at" => "OAuth 客户端保存于",
        "oauth_client_saved_authorize" => "OAuth 客户端已保存。请在最后一步完成浏览器授权。",
        "oauth_client_saved_browser" => "OAuth 客户端已保存。开始浏览器授权以加载日历。",
        "oauth_client_saved_to" => "OAuth 客户端已保存到",
        "open_setup_guide_detail" => {
            "打开设置指南，创建 Desktop app OAuth 客户端，然后在这里粘贴对应的值。"
        }
        "opening_browser_oauth" => "正在打开浏览器进行 Google OAuth...",
        "previous_month" => "上个月",
        "previous_year" => "上一年",
        "previous_years" => "前几年",
        "reauthenticate" => "重新认证",
        "refresh" => "刷新",
        "refresh_failed" => "刷新失败",
        "refresh_status" => "刷新状态",
        "refreshing" => "正在刷新",
        "replace_authenticate" => "替换并认证",
        "replace_oauth_client" => "替换 OAuth 客户端",
        "return_to_days" => "返回日期",
        "save" => "保存",
        "save_authenticate" => "保存并认证",
        "settings" => "设置",
        "settings_applied" => "设置已应用并保存。",
        "settings_open" => "设置已打开",
        "setup" => "设置",
        "setup_guide" => "设置指南",
        "setup_guide_opened" => "设置指南已打开。",
        "showing_cached_refreshing" => "显示缓存事件，同时更新 Google Calendar。",
        "start_authentication" => "开始认证",
        "theme_path" => "主题 CSS 路径:",
        "time_unavailable" => "时间不可用",
        "timezone" => "时区:",
        "today" => "今天",
        "token_folder" => "令牌文件夹",
        "token_folder_opened" => "令牌文件夹已打开。",
        "tomorrow" => "明天",
        "untitled_event" => "未命名事件",
        "upcoming" => "即将到来",
        "updated" => "已更新",
        "week_start" => "每周起始日:",
        "window_ready_updates" => "窗口已就绪，日程数据正在更新。",
        _ => key,
    }
}

fn english(key: &'static str) -> &'static str {
    match key {
        "action_required" => "Action required",
        "account_status" => "Google Account Status",
        "advanced" => "Advanced",
        "agenda" => "Agenda",
        "all" => "All",
        "all_day" => "All day",
        "appearance" => "Appearance",
        "apply" => "Apply",
        "authenticated" => "Authenticated",
        "authenticating" => "Authenticating",
        "authorize" => "Authorize",
        "authorize_calendar_access" => "Authorize calendar access",
        "browser_token_saved_at" => "Browser token saved at",
        "cached" => "Cached",
        "calendar" => "Calendar",
        "calendar_api" => "Calendar API",
        "calendar_id" => "Calendar ID:",
        "calendar_timezone" => "Calendar & Timezone",
        "cancel" => "Cancel",
        "change_calendar_view" => "Change calendar view",
        "choose_month" => "Choose month",
        "choose_year" => "Choose year",
        "chinese" => "Chinese",
        "client_id" => "Client ID",
        "client_secret" => "Client Secret",
        "close" => "Close",
        "config_folder" => "Config Folder",
        "config_folder_opened" => "Config folder opened.",
        "connect_google_calendar" => "Connect Google Calendar",
        "connected" => "Connected",
        "create_google_oauth_client" => "Create a Google OAuth client",
        "current_status" => "Current status",
        "english" => "English",
        "event" => "event",
        "events" => "events",
        "failed_load_theme" => "Failed to load theme",
        "failed_save_settings" => "Failed to save settings",
        "google_account" => "Google Account",
        "google_account_authenticated" => "Google Calendar authenticated. Loading events...",
        "google_calendar" => "Google Calendar",
        "google_calendar_api_opened" => "Google Calendar API page opened in your browser.",
        "google_calendar_credentials_saved" => {
            "Google Calendar credentials are saved. Refresh or re-authenticate if events still do not load."
        }
        "google_cloud" => "Google Cloud",
        "google_cloud_opened" => "Google Cloud opened in your browser.",
        "language" => "Language:",
        "loading" => "Loading",
        "loading_google_calendar" => "Loading Google Calendar",
        "login" => "Log In",
        "logged_out_cleanup_incomplete" => "Logged out, but cleanup was incomplete",
        "logged_out_please_auth" => "Logged out. Please authenticate.",
        "logged_out_success" => "Logged out successfully.",
        "logout" => "Log Out",
        "missing_token" => "Missing Token",
        "next_month" => "Next month",
        "next_year" => "Next year",
        "next_years" => "Next years",
        "no_loaded_events_day" => "No loaded events for ",
        "no_loaded_events_view" => "No loaded events for this calendar view.",
        "no_oauth_client_saved" => {
            "No OAuth client is saved yet. Follow the pages below; this is a one-time setup."
        }
        "no_upcoming_events" => "No upcoming events",
        "oauth_client_saved_at" => "OAuth client saved at",
        "oauth_client_saved_authorize" => {
            "OAuth client is saved. Finish browser authorization on the last page."
        }
        "oauth_client_saved_browser" => {
            "The OAuth client is saved. Start browser authorization to load your calendar."
        }
        "oauth_client_saved_to" => "OAuth client saved to",
        "open_setup_guide_detail" => {
            "Open the setup guide, create a Desktop app OAuth client, then paste the values here."
        }
        "opening_browser_oauth" => "Opening browser for Google OAuth...",
        "previous_month" => "Previous month",
        "previous_year" => "Previous year",
        "previous_years" => "Previous years",
        "reauthenticate" => "Re-authenticate",
        "refresh" => "Refresh",
        "refresh_failed" => "Refresh failed",
        "refresh_status" => "Refresh Status",
        "refreshing" => "Refreshing",
        "replace_authenticate" => "Replace & Authenticate",
        "replace_oauth_client" => "Replace OAuth client",
        "return_to_days" => "Return to days",
        "save" => "Save",
        "save_authenticate" => "Save & Authenticate",
        "settings" => "Settings",
        "settings_applied" => "Settings applied and saved.",
        "settings_open" => "Settings open",
        "setup" => "Setup",
        "setup_guide" => "Setup Guide",
        "setup_guide_opened" => "Setup guide opened.",
        "showing_cached_refreshing" => "Showing cached events while Google Calendar updates.",
        "start_authentication" => "Start Authentication",
        "theme_path" => "Theme CSS Path:",
        "time_unavailable" => "Time unavailable",
        "timezone" => "Timezone:",
        "today" => "Today",
        "token_folder" => "Token Folder",
        "token_folder_opened" => "Token folder opened.",
        "tomorrow" => "Tomorrow",
        "untitled_event" => "Untitled event",
        "upcoming" => "Upcoming",
        "updated" => "Updated",
        "week_start" => "Week Starts On:",
        "window_ready_updates" => "The window is ready while agenda data updates.",
        _ => key,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_uses_language_specific_copy_and_falls_back_to_key() {
        assert_eq!(translate(Language::English, "settings"), "Settings");
        assert_eq!(translate(Language::Chinese, "settings"), "设置");
        assert_eq!(translate(Language::English, "unknown_key"), "unknown_key");
    }

    #[test]
    fn chinese_settings_labels_do_not_embed_english_parentheticals() {
        for key in [
            "account_status",
            "appearance",
            "apply",
            "calendar_id",
            "calendar_timezone",
            "cancel",
            "chinese",
            "english",
            "google_account",
            "language",
            "login",
            "logout",
            "save",
            "theme_path",
            "timezone",
            "week_start",
        ] {
            let text = translate(Language::Chinese, key);
            assert!(!text.contains('('), "{key} contains '(' in {text}");
            assert!(!text.contains(')'), "{key} contains ')' in {text}");
        }
    }
}
