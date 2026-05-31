use super::language;
use super::layout::{SettingsIcon, field_row, section, section_title};
use crate::i18n::translate;
use crate::storage::settings::{Language, UserSettings};
use crate::ui::{drop_down, label};
use adw::prelude::*;

pub(super) struct AppearanceWidgets {
    pub(super) section: gtk::Box,
    pub(super) theme_entry: gtk::Entry,
    pub(super) language_dropdown: gtk::DropDown,
    title: gtk::Label,
    theme_label: gtk::Label,
    language_label: gtk::Label,
}

pub(super) fn build(settings: &UserSettings, lang: Language) -> AppearanceWidgets {
    let title = section_title(translate(lang, "appearance"));
    let section = section(&title, SettingsIcon::Sparkle, "tint-appearance");

    let theme_label = label(translate(lang, "theme_path"), &["field-label"], 0.0, false);
    let theme_entry = gtk::Entry::builder()
        .text(
            settings
                .theme_path
                .as_ref()
                .map(|path| path.to_string_lossy())
                .as_deref()
                .unwrap_or(""),
        )
        .placeholder_text("~/.config/waybar-google-calendar/style.css")
        .build();
    section.append(&field_row(&theme_label, &theme_entry));

    let language_label = label(translate(lang, "language"), &["field-label"], 0.0, false);
    let language_dropdown = drop_down();
    language::set_options(
        &language_dropdown,
        lang,
        settings.language.unwrap_or_default(),
    );
    section.append(&field_row(&language_label, &language_dropdown));

    AppearanceWidgets {
        section,
        theme_entry,
        language_dropdown,
        title,
        theme_label,
        language_label,
    }
}

pub(super) fn update_text(widgets: &AppearanceWidgets, lang: Language) {
    widgets.title.set_text(translate(lang, "appearance"));
    widgets.theme_label.set_text(translate(lang, "theme_path"));
    widgets.language_label.set_text(translate(lang, "language"));
    language::update_options(&widgets.language_dropdown, lang);
}

pub(super) fn populate_form(widgets: &AppearanceWidgets, settings: &UserSettings) {
    widgets.theme_entry.set_text(
        settings
            .theme_path
            .as_ref()
            .map(|path| path.to_string_lossy())
            .as_deref()
            .unwrap_or(""),
    );
    widgets
        .language_dropdown
        .set_selected(language::language_index(settings.language.unwrap_or_default()) as u32);
}
