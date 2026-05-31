use super::layout::{SettingsIcon, field_row, section, section_title};
use crate::i18n::{translate, week_start_name};
use crate::storage::settings::{Language, UserSettings, WeekStart};
use crate::ui::{
    drop_down, label, selected_time_zone, set_drop_down_strings, set_time_zone_options,
    time_zone_drop_down,
};
use adw::prelude::*;

pub(super) struct CalendarWidgets {
    pub(super) section: gtk::Box,
    pub(super) calendar_entry: gtk::Entry,
    pub(super) timezone_dropdown: gtk::DropDown,
    pub(super) week_start_dropdown: gtk::DropDown,
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
    let timezone_dropdown = time_zone_drop_down(settings.timezone.as_deref(), lang);
    section.append(&field_row(&timezone_label, &timezone_dropdown));

    let week_start_label = label(translate(lang, "week_start"), &["field-label"], 0.0, false);
    let week_start_dropdown = drop_down();
    set_week_start_options(
        &week_start_dropdown,
        lang,
        settings.week_start.unwrap_or_default(),
    );
    section.append(&field_row(&week_start_label, &week_start_dropdown));

    CalendarWidgets {
        section,
        calendar_entry,
        timezone_dropdown,
        week_start_dropdown,
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
    update_timezone_options(widgets, lang);
    update_week_start_options(widgets, lang);
}

pub(super) fn populate_form(widgets: &CalendarWidgets, settings: &UserSettings) {
    widgets
        .calendar_entry
        .set_text(settings.calendar.as_deref().unwrap_or(""));
    let lang = settings.language.unwrap_or_default();
    set_time_zone_options(
        &widgets.timezone_dropdown,
        settings.timezone.as_deref(),
        lang,
    );
    widgets
        .week_start_dropdown
        .set_selected(week_start_index(settings.week_start.unwrap_or_default()) as u32);
}

pub(super) fn selected_week_start(widgets: &CalendarWidgets) -> WeekStart {
    selected_week_start_from_dropdown(&widgets.week_start_dropdown)
}

pub(super) fn selected_week_start_from_dropdown(dropdown: &gtk::DropDown) -> WeekStart {
    WeekStart::SETTINGS_ORDER
        .get(dropdown.selected() as usize)
        .copied()
        .unwrap_or_default()
}

fn update_timezone_options(widgets: &CalendarWidgets, lang: Language) {
    let selected = selected_time_zone(&widgets.timezone_dropdown);
    set_time_zone_options(&widgets.timezone_dropdown, selected.as_deref(), lang);
}

fn update_week_start_options(widgets: &CalendarWidgets, lang: Language) {
    let selected = selected_week_start(widgets);
    set_week_start_options(&widgets.week_start_dropdown, lang, selected);
}

fn set_week_start_options(dropdown: &gtk::DropDown, lang: Language, selected: WeekStart) {
    let labels = WeekStart::SETTINGS_ORDER.map(|week_start| week_start_name(lang, week_start));
    set_drop_down_strings(dropdown, &labels, week_start_index(selected));
}

fn week_start_index(week_start: WeekStart) -> usize {
    WeekStart::SETTINGS_ORDER
        .iter()
        .position(|candidate| *candidate == week_start)
        .unwrap_or_default()
}
