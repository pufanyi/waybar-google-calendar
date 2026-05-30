mod form;
mod pages;
mod status;
mod widgets;

use super::AgendaApp;
use crate::calendar::model::AgendaState;
use crate::google;
use crate::storage::paths;
use adw::prelude::*;
use gtk::gio;
use relm4::ComponentSender;
use std::fs;
use std::path::Path;

pub(super) const GOOGLE_CLOUD_CREDENTIALS_URL: &str =
    "https://console.cloud.google.com/auth/clients";
pub(super) const GOOGLE_CALENDAR_API_URL: &str =
    "https://console.cloud.google.com/apis/library/calendar-json.googleapis.com";
pub(super) const SETUP_GUIDE_URL: &str =
    "https://github.com/pufanyi/waybar-google-calendar/blob/main/docs/google-oauth.md";

pub(super) fn prompt_card(
    error: &str,
    authenticating: bool,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    let secret_present = paths::client_secret_file().exists();
    let token_present = paths::oauth_token_file().exists();

    let card = gtk::Box::new(gtk::Orientation::Vertical, 12);
    card.add_css_class("empty-card");
    card.add_css_class("auth-prompt");
    card.append(&header(secret_present, token_present));
    card.append(&status::current(error, secret_present, token_present));
    card.append(&pages::build(
        secret_present,
        token_present,
        authenticating,
        sender.clone(),
    ));
    card.append(&widgets::utility_actions(authenticating, sender));
    card
}

pub(super) fn open_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path)
        .map_err(|err| format!("Could not create folder {}: {err}", path.display()))?;
    let file = gio::File::for_path(path);
    let uri = file.uri();
    google::open_external_uri(uri.as_str())
}

pub(super) fn open_setup_guide() -> Result<(), String> {
    google::open_external_uri(SETUP_GUIDE_URL)
}

pub(super) fn should_focus(state: &AgendaState, authenticating: bool) -> bool {
    if authenticating || status::setup_incomplete() {
        return true;
    }

    state
        .error
        .as_deref()
        .map(status::should_show_prompt)
        .unwrap_or(false)
}

fn header(secret_present: bool, token_present: bool) -> gtk::Box {
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    header.add_css_class("auth-header");
    header.append(&crate::ui::label(
        "Connect Google Calendar",
        &["event-title"],
        0.0,
        false,
    ));
    let header_spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    header_spacer.set_hexpand(true);
    header.append(&header_spacer);
    header.append(&status::badge(
        if token_present {
            "Connected"
        } else if secret_present {
            "Authorize"
        } else {
            "Setup"
        },
        if token_present {
            "success"
        } else if secret_present {
            "info"
        } else {
            "warning"
        },
    ));
    header
}
