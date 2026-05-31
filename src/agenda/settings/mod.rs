mod account;
mod appearance;
mod calendar;
mod footer;
mod language;
mod layout;

use crate::agenda::AgendaApp;
use crate::storage::settings::{Language, UserSettings};
use adw::prelude::*;
use relm4::ComponentSender;

pub(super) struct SettingsWidgets {
    pub(super) panel: gtk::Box,
    calendar: calendar::CalendarWidgets,
    appearance: appearance::AppearanceWidgets,
    account: account::AccountWidgets,
    footer: footer::FooterWidgets,
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

    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.add_css_class("settings-content");

    let calendar = calendar::build(user_settings, lang);
    content.append(&calendar.section);

    let appearance = appearance::build(user_settings, lang);
    content.append(&appearance.section);

    let account = account::build(lang, sender.clone());
    content.append(&account.section);

    let scroll = gtk::ScrolledWindow::new();
    scroll.add_css_class("settings-scroll");
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_vexpand(true);
    scroll.set_child(Some(&content));
    panel.append(&scroll);

    let footer = footer::build(lang, &calendar, &appearance, sender);
    panel.append(&footer.container);

    SettingsWidgets {
        panel,
        calendar,
        appearance,
        account,
        footer,
    }
}

pub(super) fn update_text(widgets: &SettingsWidgets, lang: Language) {
    calendar::update_text(&widgets.calendar, lang);
    appearance::update_text(&widgets.appearance, lang);
    account::update_text(&widgets.account, lang);
    footer::update_text(&widgets.footer, lang);
}

pub(super) fn update_state(
    widgets: &SettingsWidgets,
    lang: Language,
    authenticating: bool,
    message: Option<&str>,
) {
    account::update_state(&widgets.account, lang, authenticating);
    footer::update_message(&widgets.footer, lang, message);
}

pub(super) fn populate_form(widgets: &SettingsWidgets, settings: &UserSettings) {
    calendar::populate_form(&widgets.calendar, settings);
    appearance::populate_form(&widgets.appearance, settings);
}
