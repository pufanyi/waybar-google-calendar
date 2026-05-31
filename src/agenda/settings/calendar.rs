use super::layout::{SettingsIcon, field_row, section, section_title};
use crate::i18n::translate;
use crate::storage::settings::{Language, UserSettings};
use crate::ui::label;
use adw::prelude::*;

pub(super) struct CalendarWidgets {
    pub(super) section: gtk::Box,
    pub(super) calendar_entry: gtk::Entry,
    pub(super) timezone_entry: gtk::Entry,
    title: gtk::Label,
    calendar_label: gtk::Label,
    timezone_label: gtk::Label,
}

pub(super) fn build(settings: &UserSettings, lang: Language) -> CalendarWidgets {
    let title = section_title(translate(lang, "calendar_timezone"));
    let section = section(&title, SettingsIcon::Calendar, "tint-calendar");

    let calendar_label = label(translate(lang, "calendar_id"), &["field-label"], 0.0, false);
    let calendar_entry = gtk::Entry::builder()
        .text(settings.calendar.as_deref().unwrap_or(""))
        .placeholder_text("primary")
        .build();
    section.append(&field_row(&calendar_label, &calendar_entry));

    let timezone_label = label(translate(lang, "timezone"), &["field-label"], 0.0, false);
    let timezone_entry = gtk::Entry::builder()
        .text(settings.timezone.as_deref().unwrap_or(""))
        .placeholder_text("Local")
        .build();
    section.append(&field_row(&timezone_label, &timezone_entry));

    CalendarWidgets {
        section,
        calendar_entry,
        timezone_entry,
        title,
        calendar_label,
        timezone_label,
    }
}

pub(super) fn update_text(widgets: &CalendarWidgets, lang: Language) {
    widgets.title.set_text(translate(lang, "calendar_timezone"));
    widgets
        .calendar_label
        .set_text(translate(lang, "calendar_id"));
    widgets.timezone_label.set_text(translate(lang, "timezone"));
}

pub(super) fn populate_form(widgets: &CalendarWidgets, settings: &UserSettings) {
    widgets
        .calendar_entry
        .set_text(settings.calendar.as_deref().unwrap_or(""));
    widgets
        .timezone_entry
        .set_text(settings.timezone.as_deref().unwrap_or(""));
}
