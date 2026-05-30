use super::{AgendaApp, AgendaMsg};
use crate::storage::paths;
use crate::storage::settings::{Language, UserSettings, translate};
use crate::ui::{classed_button, icon_button, label};
use adw::prelude::*;
use relm4::ComponentSender;

pub(super) struct SettingsWidgets {
    pub(super) panel: gtk::Box,
    pub(super) calendar_entry: gtk::Entry,
    pub(super) timezone_entry: gtk::Entry,
    pub(super) theme_entry: gtk::Entry,
    pub(super) language_combo: gtk::ComboBoxText,
    title: gtk::Label,
    close_button: gtk::Button,
    cal_tz_title: gtk::Label,
    calendar_label: gtk::Label,
    timezone_label: gtk::Label,
    appearance_title: gtk::Label,
    theme_label: gtk::Label,
    language_label: gtk::Label,
    account_title: gtk::Label,
    account_status_label: gtk::Label,
    account_status_badge: gtk::Label,
    login_button: gtk::Button,
    logout_button: gtk::Button,
    message_label: gtk::Label,
    cancel_button: gtk::Button,
    save_button: gtk::Button,
}

pub(super) fn build(
    user_settings: &UserSettings,
    lang: Language,
    sender: ComponentSender<AgendaApp>,
) -> SettingsWidgets {
    let panel = gtk::Box::new(gtk::Orientation::Vertical, 12);
    panel.add_css_class("settings-panel");
    panel.set_hexpand(true);
    panel.set_vexpand(true);

    let topbar = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    topbar.add_css_class("topbar");
    topbar.add_css_class("settings-topbar");

    let title_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    title_box.add_css_class("settings-title-box");
    title_box.append(&icon_tile(
        "emblem-system-symbolic",
        "settings-title-icon",
        "tint-general",
    ));
    let title = label(translate(lang, "settings"), &["title"], 0.0, false);
    title_box.append(&title);
    topbar.append(&title_box);

    let top_spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    top_spacer.set_hexpand(true);
    topbar.append(&top_spacer);

    let close_button = icon_button(
        "go-previous-symbolic",
        &["close-button", "icon-button"],
        translate(lang, "close"),
    );
    {
        let sender = sender.clone();
        close_button.connect_clicked(move |_| sender.input(AgendaMsg::CloseSettings));
    }
    topbar.append(&close_button);
    panel.append(&topbar);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.add_css_class("settings-content");

    let cal_tz_title = label(
        translate(lang, "calendar_timezone"),
        &["event-title"],
        0.0,
        false,
    );
    let calendar_section = section(&cal_tz_title, "x-office-calendar-symbolic", "tint-calendar");

    let calendar_label = label(translate(lang, "calendar_id"), &["field-label"], 0.0, false);
    let calendar_entry = gtk::Entry::builder()
        .text(user_settings.calendar.as_deref().unwrap_or(""))
        .placeholder_text("primary")
        .build();
    calendar_section.append(&field_row(&calendar_label, &calendar_entry));

    let timezone_label = label(translate(lang, "timezone"), &["field-label"], 0.0, false);
    let timezone_entry = gtk::Entry::builder()
        .text(user_settings.timezone.as_deref().unwrap_or(""))
        .placeholder_text("Local")
        .build();
    calendar_section.append(&field_row(&timezone_label, &timezone_entry));
    content.append(&calendar_section);

    let appearance_title = label(translate(lang, "appearance"), &["event-title"], 0.0, false);
    let appearance_section = section(
        &appearance_title,
        "preferences-desktop-theme-symbolic",
        "tint-appearance",
    );

    let theme_label = label(translate(lang, "theme_path"), &["field-label"], 0.0, false);
    let theme_entry = gtk::Entry::builder()
        .text(
            user_settings
                .theme_path
                .as_ref()
                .map(|path| path.to_string_lossy())
                .as_deref()
                .unwrap_or(""),
        )
        .placeholder_text("~/.config/waybar-google-calendar/style.css")
        .build();
    appearance_section.append(&field_row(&theme_label, &theme_entry));

    let language_label = label(translate(lang, "language"), &["field-label"], 0.0, false);
    let language_combo = gtk::ComboBoxText::new();
    set_language_combo_options(
        &language_combo,
        lang,
        user_settings.language.unwrap_or_default(),
    );
    appearance_section.append(&field_row(&language_label, &language_combo));
    content.append(&appearance_section);

    let account_title = label(
        translate(lang, "google_account"),
        &["event-title"],
        0.0,
        false,
    );
    let account_section = section(&account_title, "avatar-default-symbolic", "tint-account");

    let account_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    account_row.add_css_class("settings-form-row");
    let account_status_label = label(
        translate(lang, "account_status"),
        &["path-label", "muted"],
        0.0,
        false,
    );
    account_status_label.set_hexpand(true);
    let account_status_badge = label("", &["status-badge"], 0.5, false);
    account_row.append(&account_status_label);
    account_row.append(&account_status_badge);
    account_section.append(&account_row);

    let account_actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    account_actions.add_css_class("settings-inline-actions");
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
    account_actions.append(&login_button);
    account_actions.append(&logout_button);
    account_section.append(&account_actions);
    content.append(&account_section);

    let scroll = gtk::ScrolledWindow::new();
    scroll.add_css_class("settings-scroll");
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_vexpand(true);
    scroll.set_child(Some(&content));
    panel.append(&scroll);

    let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    buttons.add_css_class("settings-footer");
    let message_label = label("", &["muted", "settings-message"], 0.0, true);
    message_label.set_hexpand(true);
    buttons.append(&message_label);

    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    buttons.append(&spacer);

    let cancel_button = classed_button(translate(lang, "cancel"), &["action-button"]);
    {
        let sender = sender.clone();
        cancel_button.connect_clicked(move |_| sender.input(AgendaMsg::CloseSettings));
    }
    buttons.append(&cancel_button);

    let save_button = classed_button(translate(lang, "save"), &["action-button"]);
    save_button.add_css_class("primary-action");
    {
        let sender = sender.clone();
        let calendar_entry = calendar_entry.clone();
        let timezone_entry = timezone_entry.clone();
        let theme_entry = theme_entry.clone();
        let language_combo = language_combo.clone();
        save_button.connect_clicked(move |_| {
            let calendar = calendar_entry.text().trim().to_string();
            let timezone = timezone_entry.text().trim().to_string();
            let theme_path = theme_entry.text().trim().to_string();
            sender.input(AgendaMsg::SaveSettings {
                calendar: if calendar.is_empty() {
                    None
                } else {
                    Some(calendar)
                },
                timezone: if timezone.is_empty() {
                    None
                } else {
                    Some(timezone)
                },
                theme_path: if theme_path.is_empty() {
                    None
                } else {
                    Some(theme_path)
                },
                language: selected_language(&language_combo),
            });
        });
    }
    buttons.append(&save_button);
    panel.append(&buttons);

    SettingsWidgets {
        panel,
        calendar_entry,
        timezone_entry,
        theme_entry,
        language_combo,
        title,
        close_button,
        cal_tz_title,
        calendar_label,
        timezone_label,
        appearance_title,
        theme_label,
        language_label,
        account_title,
        account_status_label,
        account_status_badge,
        login_button,
        logout_button,
        message_label,
        cancel_button,
        save_button,
    }
}

pub(super) fn update_text(widgets: &SettingsWidgets, lang: Language) {
    widgets.title.set_text(translate(lang, "settings"));
    widgets
        .close_button
        .set_tooltip_text(Some(translate(lang, "close")));
    widgets
        .cal_tz_title
        .set_text(translate(lang, "calendar_timezone"));
    widgets
        .calendar_label
        .set_text(translate(lang, "calendar_id"));
    widgets.timezone_label.set_text(translate(lang, "timezone"));
    widgets
        .appearance_title
        .set_text(translate(lang, "appearance"));
    widgets.theme_label.set_text(translate(lang, "theme_path"));
    widgets.language_label.set_text(translate(lang, "language"));
    widgets
        .account_title
        .set_text(translate(lang, "google_account"));
    widgets
        .account_status_label
        .set_text(translate(lang, "account_status"));
    widgets.cancel_button.set_label(translate(lang, "cancel"));
    widgets.save_button.set_label(translate(lang, "save"));
    update_language_options(widgets, lang);
}

pub(super) fn update_state(
    widgets: &SettingsWidgets,
    lang: Language,
    authenticating: bool,
    message: Option<&str>,
) {
    let token_exists = paths::oauth_token_file().exists();

    widgets.account_status_badge.remove_css_class("success");
    widgets.account_status_badge.remove_css_class("warning");
    widgets.account_status_badge.remove_css_class("info");

    if authenticating {
        widgets
            .account_status_badge
            .set_text(translate(lang, "authenticating"));
        widgets.account_status_badge.add_css_class("info");
        widgets.login_button.set_sensitive(false);
        widgets.logout_button.set_sensitive(false);
        widgets
            .login_button
            .set_label(translate(lang, "authenticating"));
    } else if token_exists {
        widgets
            .account_status_badge
            .set_text(translate(lang, "authenticated"));
        widgets.account_status_badge.add_css_class("success");
        widgets.login_button.set_sensitive(false);
        widgets.logout_button.set_sensitive(true);
        widgets.login_button.set_label(translate(lang, "login"));
    } else {
        widgets
            .account_status_badge
            .set_text(translate(lang, "missing_token"));
        widgets.account_status_badge.add_css_class("warning");
        widgets.login_button.set_sensitive(true);
        widgets.logout_button.set_sensitive(false);
        widgets.login_button.set_label(translate(lang, "login"));
    }

    widgets.logout_button.set_label(translate(lang, "logout"));

    if let Some(message) = message {
        widgets.message_label.set_text(match message {
            "Logged out successfully." => translate(lang, "logged_out_success"),
            "Logged out. Please authenticate." => translate(lang, "logged_out_please_auth"),
            _ => message,
        });
    } else {
        widgets.message_label.set_text("");
    }
}

fn section(title: &gtk::Label, icon_name: &str, tint: &str) -> gtk::Box {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 10);
    section.add_css_class("settings-section");

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    header.add_css_class("settings-section-header");
    header.append(&icon_tile(icon_name, "settings-section-icon", tint));
    header.append(title);
    section.append(&header);

    section
}

/// A rounded, colour-filled tile holding a centered symbolic icon (iOS style).
fn icon_tile(icon_name: &str, icon_class: &str, tint: &str) -> gtk::Box {
    let tile = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    tile.add_css_class("settings-icon-tile");
    tile.add_css_class(tint);
    tile.set_halign(gtk::Align::Center);
    tile.set_valign(gtk::Align::Center);

    let icon = gtk::Image::from_icon_name(icon_name);
    icon.add_css_class(icon_class);
    tile.append(&icon);

    tile
}

fn field_row(label: &gtk::Label, input: &impl IsA<gtk::Widget>) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    row.add_css_class("settings-form-row");
    label.set_size_request(150, -1);
    row.append(label);
    input.as_ref().add_css_class("text-entry");
    input.as_ref().set_hexpand(true);
    row.append(input);
    row
}

fn selected_language(combo: &gtk::ComboBoxText) -> Language {
    match combo.active_id().as_deref() {
        Some("chinese") => Language::Chinese,
        _ => Language::English,
    }
}

fn update_language_options(widgets: &SettingsWidgets, lang: Language) {
    let selected = selected_language(&widgets.language_combo);
    set_language_combo_options(&widgets.language_combo, lang, selected);
}

fn set_language_combo_options(combo: &gtk::ComboBoxText, lang: Language, selected: Language) {
    combo.remove_all();
    combo.append(Some("english"), translate(lang, "english"));
    combo.append(Some("chinese"), translate(lang, "chinese"));
    combo.set_active_id(Some(language_id(selected)));
}

fn language_id(language: Language) -> &'static str {
    match language {
        Language::English => "english",
        Language::Chinese => "chinese",
    }
}
