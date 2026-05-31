use super::layout::{SettingsIcon, section, section_title};
use crate::agenda::{AgendaApp, AgendaMsg};
use crate::i18n::translate;
use crate::storage::paths;
use crate::storage::settings::Language;
use crate::ui::{classed_button, label};
use adw::prelude::*;
use relm4::ComponentSender;

pub(super) struct AccountWidgets {
    pub(super) section: gtk::Box,
    title: gtk::Label,
    status_label: gtk::Label,
    status_badge: gtk::Label,
    login_button: gtk::Button,
    logout_button: gtk::Button,
}

pub(super) fn build(lang: Language, sender: ComponentSender<AgendaApp>) -> AccountWidgets {
    let title = section_title(translate(lang, "google_account"));
    let section = section(&title, SettingsIcon::Account, "tint-account");

    let status_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    status_row.add_css_class("settings-form-row");
    let status_label = label(
        translate(lang, "account_status"),
        &["path-label", "muted"],
        0.0,
        false,
    );
    status_label.set_hexpand(true);
    let status_badge = label("", &["status-badge"], 0.5, false);
    status_row.append(&status_label);
    status_row.append(&status_badge);
    section.append(&status_row);

    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actions.add_css_class("settings-inline-actions");
    actions.set_halign(gtk::Align::End);
    let login_button = classed_button(translate(lang, "login"), &["action-button"]);
    let logout_button = classed_button(translate(lang, "logout"), &["action-button"]);
    {
        let sender = sender.clone();
        login_button.connect_clicked(move |_| sender.input(AgendaMsg::StartAuth));
    }
    {
        let sender = sender.clone();
        logout_button.connect_clicked(move |_| sender.input(AgendaMsg::Logout));
    }
    actions.append(&login_button);
    actions.append(&logout_button);
    section.append(&actions);

    AccountWidgets {
        section,
        title,
        status_label,
        status_badge,
        login_button,
        logout_button,
    }
}

pub(super) fn update_text(widgets: &AccountWidgets, lang: Language) {
    widgets.title.set_text(translate(lang, "google_account"));
    widgets
        .status_label
        .set_text(translate(lang, "account_status"));
}

pub(super) fn update_state(widgets: &AccountWidgets, lang: Language, authenticating: bool) {
    let token_exists = paths::oauth_token_file().exists();

    widgets.status_badge.remove_css_class("success");
    widgets.status_badge.remove_css_class("warning");
    widgets.status_badge.remove_css_class("info");

    if authenticating {
        widgets
            .status_badge
            .set_text(translate(lang, "authenticating"));
        widgets.status_badge.add_css_class("info");
        widgets.login_button.set_sensitive(false);
        widgets.logout_button.set_sensitive(false);
        widgets
            .login_button
            .set_label(translate(lang, "authenticating"));
    } else if token_exists {
        widgets
            .status_badge
            .set_text(translate(lang, "authenticated"));
        widgets.status_badge.add_css_class("success");
        widgets.login_button.set_sensitive(false);
        widgets.logout_button.set_sensitive(true);
        widgets.login_button.set_label(translate(lang, "login"));
    } else {
        widgets
            .status_badge
            .set_text(translate(lang, "missing_token"));
        widgets.status_badge.add_css_class("warning");
        widgets.login_button.set_sensitive(true);
        widgets.logout_button.set_sensitive(false);
        widgets.login_button.set_label(translate(lang, "login"));
    }

    widgets.logout_button.set_label(translate(lang, "logout"));
}
