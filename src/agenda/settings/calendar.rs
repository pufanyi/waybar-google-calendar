use super::layout::{SettingsIcon, field_row, section, section_title};
use crate::i18n::{translate, week_start_name};
use crate::storage::settings::{Language, UserSettings, WeekStart};
use crate::ui::label;
use adw::prelude::*;

pub(super) struct CalendarWidgets {
    pub(super) section: gtk::Box,
    pub(super) calendar_entry: gtk::Entry,
    pub(super) timezone_entry: gtk::Entry,
    pub(super) week_start_combo: gtk::ComboBoxText,
    title: gtk::Label,
    calendar_label: gtk::Label,
    timezone_label: gtk::Label,
    week_start_label: gtk::Label,
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

    let week_start_label = label(translate(lang, "week_start"), &["field-label"], 0.0, false);
    let week_start_combo = gtk::ComboBoxText::new();
    set_week_start_options(
        &week_start_combo,
        lang,
        settings.week_start.unwrap_or_default(),
    );
    section.append(&field_row(&week_start_label, &week_start_combo));

    CalendarWidgets {
        section,
        calendar_entry,
        timezone_entry,
        week_start_combo,
        title,
        calendar_label,
        timezone_label,
        week_start_label,
    }
}

pub(super) fn update_text(widgets: &CalendarWidgets, lang: Language) {
    widgets.title.set_text(translate(lang, "calendar_timezone"));
    widgets
        .calendar_label
        .set_text(translate(lang, "calendar_id"));
    widgets.timezone_label.set_text(translate(lang, "timezone"));
    widgets
        .week_start_label
        .set_text(translate(lang, "week_start"));
    update_week_start_options(widgets, lang);
}

pub(super) fn populate_form(widgets: &CalendarWidgets, settings: &UserSettings) {
    widgets
        .calendar_entry
        .set_text(settings.calendar.as_deref().unwrap_or(""));
    widgets
        .timezone_entry
        .set_text(settings.timezone.as_deref().unwrap_or(""));
    widgets
        .week_start_combo
        .set_active_id(Some(week_start_id(settings.week_start.unwrap_or_default())));
}

pub(super) fn selected_week_start(widgets: &CalendarWidgets) -> WeekStart {
    selected_week_start_from_combo(&widgets.week_start_combo)
}

pub(super) fn selected_week_start_from_combo(combo: &gtk::ComboBoxText) -> WeekStart {
    match combo.active_id().as_deref() {
        Some("monday") => WeekStart::Monday,
        Some("tuesday") => WeekStart::Tuesday,
        Some("wednesday") => WeekStart::Wednesday,
        Some("thursday") => WeekStart::Thursday,
        Some("friday") => WeekStart::Friday,
        Some("saturday") => WeekStart::Saturday,
        Some("sunday") => WeekStart::Sunday,
        _ => WeekStart::default(),
    }
}

fn update_week_start_options(widgets: &CalendarWidgets, lang: Language) {
    let selected = selected_week_start(widgets);
    set_week_start_options(&widgets.week_start_combo, lang, selected);
}

fn set_week_start_options(combo: &gtk::ComboBoxText, lang: Language, selected: WeekStart) {
    combo.remove_all();
    for week_start in WeekStart::SETTINGS_ORDER {
        combo.append(
            Some(week_start_id(week_start)),
            week_start_name(lang, week_start),
        );
    }
    combo.set_active_id(Some(week_start_id(selected)));
}

fn week_start_id(week_start: WeekStart) -> &'static str {
    match week_start {
        WeekStart::Monday => "monday",
        WeekStart::Tuesday => "tuesday",
        WeekStart::Wednesday => "wednesday",
        WeekStart::Thursday => "thursday",
        WeekStart::Friday => "friday",
        WeekStart::Saturday => "saturday",
        WeekStart::Sunday => "sunday",
    }
}
