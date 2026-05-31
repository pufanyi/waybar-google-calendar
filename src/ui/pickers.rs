use super::widgets::{block_scroll_changes, drop_down, set_drop_down_strings};
use crate::i18n::translate;
use crate::storage::settings::Language;
use adw::prelude::*;
use chrono::{Datelike, NaiveDate, NaiveTime, Timelike};
use chrono_tz::TZ_VARIANTS;
use gtk::glib;

#[derive(Clone)]
pub struct DateTimePicker {
    pub container: gtk::Box,
    calendar: gtk::Calendar,
    hour: gtk::SpinButton,
    minute: gtk::SpinButton,
}

impl DateTimePicker {
    pub fn new(date: NaiveDate, time: NaiveTime) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 8);
        container.add_css_class("datetime-picker");

        let calendar = gtk::Calendar::new();
        calendar.add_css_class("datetime-calendar");
        calendar.set_hexpand(true);
        if let Ok(date_time) = glib::DateTime::from_local(
            date.year(),
            date.month() as i32,
            date.day() as i32,
            0,
            0,
            0.0,
        ) {
            calendar.set_date(Some(&date_time));
        }
        container.append(&calendar);

        let time_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        time_row.add_css_class("datetime-time-row");
        let hour = time_spin(0, 23, 1, time.hour());
        let minute = time_spin(0, 59, 5, time.minute());
        let separator = gtk::Label::new(Some(":"));
        separator.add_css_class("datetime-separator");
        time_row.append(&hour);
        time_row.append(&separator);
        time_row.append(&minute);
        container.append(&time_row);

        Self {
            container,
            calendar,
            hour,
            minute,
        }
    }

    pub fn date_string(&self) -> String {
        let date = self.calendar.date();
        format!(
            "{:04}-{:02}-{:02}",
            date.year(),
            date.month(),
            date.day_of_month()
        )
    }

    pub fn time_string(&self) -> String {
        format!(
            "{:02}:{:02}",
            self.hour.value_as_int(),
            self.minute.value_as_int()
        )
    }

    pub fn set_time_sensitive(&self, sensitive: bool) {
        self.hour.set_sensitive(sensitive);
        self.minute.set_sensitive(sensitive);
    }
}

pub fn time_zone_drop_down(current: Option<&str>, lang: Language) -> gtk::DropDown {
    let widget = drop_down();
    widget.set_enable_search(true);
    set_time_zone_options(&widget, current, lang);
    widget
}

pub fn set_time_zone_options(dropdown: &gtk::DropDown, current: Option<&str>, lang: Language) {
    let current = normalized_time_zone(current);
    let mut labels = Vec::with_capacity(TZ_VARIANTS.len() + 2);
    labels.push(translate(lang, "local_timezone").to_string());
    labels.extend(TZ_VARIANTS.iter().map(ToString::to_string));

    if let Some(current) = current
        && !labels.iter().any(|label| label == current)
    {
        labels.push(current.to_string());
    }

    let selected = current
        .and_then(|current| labels.iter().position(|label| label == current))
        .unwrap_or_default();
    let label_refs = labels.iter().map(String::as_str).collect::<Vec<_>>();
    set_drop_down_strings(dropdown, &label_refs, selected);
}

pub fn selected_time_zone(dropdown: &gtk::DropDown) -> Option<String> {
    if dropdown.selected() == 0 {
        return None;
    }

    dropdown
        .selected_item()
        .and_downcast::<gtk::StringObject>()
        .map(|item| item.string().to_string())
        .filter(|value| !value.trim().is_empty())
}

fn time_spin(min: u32, max: u32, step: u32, value: u32) -> gtk::SpinButton {
    let spin = gtk::SpinButton::with_range(min as f64, max as f64, step as f64);
    spin.add_css_class("datetime-spin");
    spin.set_digits(0);
    spin.set_numeric(true);
    spin.set_wrap(true);
    spin.set_width_chars(2);
    spin.set_value(value.clamp(min, max) as f64);
    block_scroll_changes(&spin);
    spin
}

fn normalized_time_zone(timezone: Option<&str>) -> Option<&str> {
    timezone
        .map(str::trim)
        .filter(|timezone| !timezone.is_empty())
}
