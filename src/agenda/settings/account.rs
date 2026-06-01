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
    client_label: gtk::Label,
    client_badge: gtk::Label,
    token_label: gtk::Label,
    token_badge: gtk::Label,
    setup_button: gtk::Button,
    login_button: gtk::Button,
    logout_button: gtk::Button,
}

pub(super) fn build(lang: Language, sender: ComponentSender<AgendaApp>) -> AccountWidgets {
    let title = section_title(translate(lang, "google_account"));
    let section = section(&title, SettingsIcon::Account, "tint-account");

    let (client_row, client_label, client_badge) =
        status_row(translate(lang, "oauth_client_status"));
    section.append(&client_row);

    let (token_row, token_label, token_badge) = status_row(translate(lang, "oauth_token_status"));
    section.append(&token_row);

    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actions.add_css_class("settings-inline-actions");
    actions.set_halign(gtk::Align::End);
    let setup_button = classed_button(translate(lang, "setup_guide"), &["action-button"]);
    let login_button = classed_button(translate(lang, "login"), &["action-button"]);
    let logout_button = classed_button(translate(lang, "logout"), &["action-button"]);
    {
        let sender = sender.clone();
        setup_button.connect_clicked(move |_| sender.input(AgendaMsg::OpenSetupGuide));
    }
    {
        let sender = sender.clone();
        login_button.connect_clicked(move |_| sender.input(AgendaMsg::StartAuth));
    }
    {
        let sender = sender.clone();
        logout_button.connect_clicked(move |_| sender.input(AgendaMsg::Logout));
    }
    actions.append(&setup_button);
    actions.append(&login_button);
    actions.append(&logout_button);
    section.append(&actions);

    AccountWidgets {
        section,
        title,
        client_label,
        client_badge,
        token_label,
        token_badge,
        setup_button,
        login_button,
        logout_button,
    }
}

pub(super) fn update_text(widgets: &AccountWidgets, lang: Language) {
    widgets.title.set_text(translate(lang, "google_account"));
    widgets
        .client_label
        .set_text(translate(lang, "oauth_client_status"));
    widgets
        .token_label
        .set_text(translate(lang, "oauth_token_status"));
    widgets
        .setup_button
        .set_label(translate(lang, "setup_guide"));
}

pub(super) fn update_state(widgets: &AccountWidgets, lang: Language, authenticating: bool) {
    let client_exists = paths::client_secret_file().exists();
    let token_exists = paths::oauth_token_file().exists();

    set_client_badge(&widgets.client_badge, client_exists, lang);
    set_token_badge(&widgets.token_badge, token_exists, lang);

    if authenticating {
        set_info_badge(&widgets.token_badge, translate(lang, "authenticating"));
        widgets.setup_button.set_sensitive(false);
        widgets.login_button.set_sensitive(false);
        widgets.logout_button.set_sensitive(false);
        widgets
            .login_button
            .set_label(translate(lang, "authenticating"));
    } else if token_exists {
        widgets.setup_button.set_sensitive(true);
        widgets.login_button.set_sensitive(false);
        widgets.logout_button.set_sensitive(true);
        widgets.login_button.set_label(translate(lang, "login"));
    } else if client_exists {
        widgets.setup_button.set_sensitive(true);
        widgets.login_button.set_sensitive(true);
        widgets.logout_button.set_sensitive(false);
        widgets.login_button.set_label(translate(lang, "login"));
    } else {
        widgets.setup_button.set_sensitive(true);
        widgets.login_button.set_sensitive(false);
        widgets.logout_button.set_sensitive(false);
        widgets.login_button.set_label(translate(lang, "login"));
    }

    widgets.logout_button.set_label(translate(lang, "logout"));
}

fn status_row(title: &str) -> (gtk::Box, gtk::Label, gtk::Label) {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    row.add_css_class("settings-form-row");
    let row_label = label(title, &["path-label", "muted"], 0.0, false);
    row_label.set_hexpand(true);
    let badge = label("", &["status-badge"], 0.5, false);
    row.append(&row_label);
    row.append(&badge);
    (row, row_label, badge)
}

fn set_client_badge(widget: &gtk::Label, present: bool, lang: Language) {
    reset_badge(widget);
    if present {
        widget.set_text(translate(lang, "saved"));
        widget.add_css_class("info");
    } else {
        widget.set_text(translate(lang, "setup"));
        widget.add_css_class("warning");
    }
}

fn set_token_badge(widget: &gtk::Label, present: bool, lang: Language) {
    reset_badge(widget);
    if present {
        widget.set_text(translate(lang, "authenticated"));
        widget.add_css_class("success");
    } else {
        widget.set_text(translate(lang, "missing_token"));
        widget.add_css_class("warning");
    }
}

fn set_info_badge(widget: &gtk::Label, text: &str) {
    reset_badge(widget);
    widget.set_text(text);
    widget.add_css_class("info");
}

fn reset_badge(widget: &gtk::Label) {
    widget.remove_css_class("success");
    widget.remove_css_class("warning");
    widget.remove_css_class("info");
    widget.remove_css_class("neutral");
}
