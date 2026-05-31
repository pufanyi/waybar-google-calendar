use crate::i18n::translate;
use crate::storage::paths;
use crate::storage::settings::Language;
use crate::ui::label;
use adw::prelude::*;

pub(super) fn current(
    error: &str,
    secret_present: bool,
    token_present: bool,
    lang: Language,
) -> gtk::Box {
    let status = gtk::Box::new(gtk::Orientation::Vertical, 3);
    status.add_css_class("auth-current-status");
    status.append(&label(
        translate(lang, "current_status"),
        &["field-label"],
        0.0,
        false,
    ));
    status.append(&label(
        &friendly_message(error, secret_present, token_present, lang),
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

fn friendly_message(
    error: &str,
    secret_present: bool,
    token_present: bool,
    lang: Language,
) -> String {
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
        return translate(lang, "no_oauth_client_saved").to_string();
    }
    if lower.contains("not authenticated") {
        return translate(lang, "oauth_client_saved_authorize").to_string();
    }
    if !error.is_empty() {
        return error.to_string();
    }
    if !secret_present {
        return translate(lang, "no_oauth_client_saved").to_string();
    }
    if !token_present {
        return translate(lang, "oauth_client_saved_authorize").to_string();
    }
    translate(lang, "google_calendar_credentials_saved").to_string()
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
