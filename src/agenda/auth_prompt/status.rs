use crate::storage::paths;
use crate::ui::label;
use adw::prelude::*;

pub(super) fn current(error: &str, secret_present: bool, token_present: bool) -> gtk::Box {
    let status = gtk::Box::new(gtk::Orientation::Vertical, 3);
    status.add_css_class("auth-current-status");
    status.append(&label("Current status", &["field-label"], 0.0, false));
    status.append(&label(
        &friendly_message(error, secret_present, token_present),
        &["muted"],
        0.0,
        true,
    ));
    status
}

pub(super) fn badge(text: &str, state: &str) -> gtk::Label {
    let badge = label(text, &["status-badge"], 0.5, false);
    badge.add_css_class(state);
    badge
}

pub(super) fn should_show_prompt(error: &str) -> bool {
    needs_auth_action(error) || setup_incomplete()
}

fn friendly_message(error: &str, secret_present: bool, token_present: bool) -> String {
    let error = error.trim();
    let lower = error.to_ascii_lowercase();
    if lower.contains("opened")
        || lower.contains("opening browser")
        || lower.contains("saved to")
        || lower.contains("authenticated")
    {
        return error.to_string();
    }
    if lower.contains("missing google oauth client secret") {
        return "No OAuth client is saved yet. Follow the pages below; this is a one-time setup."
            .to_string();
    }
    if lower.contains("not authenticated") {
        return "OAuth client is saved. Finish browser authorization on the last page.".to_string();
    }
    if !error.is_empty() {
        return error.to_string();
    }
    if !secret_present {
        return "No OAuth client is saved yet. Follow the pages below; this is a one-time setup."
            .to_string();
    }
    if !token_present {
        return "OAuth client is saved. Finish browser authorization on the last page.".to_string();
    }
    "Google Calendar credentials are saved. Refresh or re-authenticate if events still do not load."
        .to_string()
}

fn needs_auth_action(error: &str) -> bool {
    let error = error.to_ascii_lowercase();
    error.contains("oauth")
        || error.contains("client secret")
        || error.contains("not authenticated")
        || error.contains("access token")
        || error.contains("invalid_grant")
        || error.contains("401")
}

pub(super) fn setup_incomplete() -> bool {
    !paths::client_secret_file().exists() || !paths::oauth_token_file().exists()
}
