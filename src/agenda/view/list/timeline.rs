use super::super::cards;
use super::{event_count_label, event_start_key};
use crate::agenda::AgendaViewMode;
use crate::calendar::date::{
    event_date_for_timezone, format_now_time_for_timezone, now_parts_for_timezone,
    parse_event_start_for_timezone,
};
use crate::calendar::model::Event;
use crate::i18n::translate;
use crate::storage::settings::Language;
use crate::ui::label;
use adw::prelude::*;
use chrono::{Datelike, NaiveDate};
use std::collections::BTreeMap;

pub(super) fn append_events(
    list: &gtk::Box,
    events: Vec<&Event>,
    selected_day: Option<NaiveDate>,
    timezone: Option<&str>,
    lang: Language,
    view: AgendaViewMode,
) {
    let groups = grouped_by_day(events, timezone);
    let today = now_parts_for_timezone(timezone).0;
    if view_should_show_now_line(view, selected_day, today) && !groups.contains_key(&Some(today)) {
        list.append(&day_header(Some(today), 0, timezone, lang));
        list.append(&now_marker(timezone, lang));
    }

    for (date, mut events) in groups {
        events.sort_by_key(|event| event_start_key(event, timezone));
        list.append(&day_header(date, events.len(), timezone, lang));
        append_day_events(list, &events, date, selected_day, timezone, lang, view);
    }
}

pub(super) fn append_empty_now_reference(
    list: &gtk::Box,
    selected_day: Option<NaiveDate>,
    timezone: Option<&str>,
    lang: Language,
    view: AgendaViewMode,
) {
    let today = now_parts_for_timezone(timezone).0;
    if !view_should_show_now_line(view, selected_day, today) {
        return;
    }

    list.append(&day_header(Some(today), 0, timezone, lang));
    list.append(&now_marker(timezone, lang));
}

fn grouped_by_day<'a>(
    events: Vec<&'a Event>,
    timezone: Option<&str>,
) -> BTreeMap<Option<NaiveDate>, Vec<&'a Event>> {
    let mut groups: BTreeMap<Option<NaiveDate>, Vec<&Event>> = BTreeMap::new();
    for event in events {
        groups
            .entry(event_date_for_timezone(event, timezone))
            .or_default()
            .push(event);
    }
    groups
}

fn append_day_events(
    list: &gtk::Box,
    events: &[&Event],
    date: Option<NaiveDate>,
    selected_day: Option<NaiveDate>,
    timezone: Option<&str>,
    lang: Language,
    view: AgendaViewMode,
) {
    let today = now_parts_for_timezone(timezone).0;
    let show_now_line = date == Some(today) && view_should_show_now_line(view, selected_day, today);
    let mut marker_inserted = false;

    for event in all_day_events(events, timezone) {
        list.append(&event_row(event, timezone, lang));
    }

    for event in timed_events(events, timezone) {
        if show_now_line && !marker_inserted && should_insert_now_before(event, today, timezone) {
            list.append(&now_marker(timezone, lang));
            marker_inserted = true;
        }

        let row = event_row(event, timezone, lang);
        if event_is_current(event, today, timezone) {
            row.add_css_class("current-event");
        }
        list.append(&row);
    }

    if show_now_line && !marker_inserted {
        list.append(&now_marker(timezone, lang));
    }
}

fn all_day_events<'a>(events: &[&'a Event], timezone: Option<&str>) -> Vec<&'a Event> {
    events
        .iter()
        .copied()
        .filter(|event| {
            matches!(
                parse_event_start_for_timezone(&event.start, timezone),
                Some((_, None))
            )
        })
        .collect()
}

fn timed_events<'a>(events: &[&'a Event], timezone: Option<&str>) -> Vec<&'a Event> {
    events
        .iter()
        .copied()
        .filter(|event| {
            !matches!(
                parse_event_start_for_timezone(&event.start, timezone),
                Some((_, None))
            )
        })
        .collect()
}

fn event_row(event: &Event, timezone: Option<&str>, lang: Language) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    row.add_css_class("agenda-timeline-row");

    let rail = gtk::Box::new(gtk::Orientation::Vertical, 4);
    rail.add_css_class("agenda-time-rail");
    rail.append(&label(
        &rail_time_label(event, timezone, lang),
        &["agenda-rail-label"],
        1.0,
        false,
    ));
    let line = gtk::Box::new(gtk::Orientation::Vertical, 0);
    line.add_css_class("agenda-rail-line");
    line.set_halign(gtk::Align::End);
    line.set_vexpand(true);
    rail.append(&line);
    row.append(&rail);

    let slot = gtk::Box::new(gtk::Orientation::Vertical, 0);
    slot.add_css_class("agenda-event-slot");
    slot.set_hexpand(true);
    let card = cards::event(event, timezone, lang);
    card.set_hexpand(true);
    slot.append(&card);
    row.append(&slot);

    row
}

fn now_marker(timezone: Option<&str>, lang: Language) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    row.add_css_class("agenda-now-marker");

    row.append(&label(
        &format_now_time_for_timezone(timezone, lang),
        &["agenda-now-chip"],
        1.0,
        false,
    ));

    let track = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    track.add_css_class("agenda-now-track");
    track.set_hexpand(true);

    let dot = gtk::Box::new(gtk::Orientation::Vertical, 0);
    dot.add_css_class("agenda-now-dot");
    dot.set_valign(gtk::Align::Center);
    track.append(&dot);

    let line = gtk::Box::new(gtk::Orientation::Vertical, 0);
    line.add_css_class("agenda-now-line");
    line.set_hexpand(true);
    line.set_valign(gtk::Align::Center);
    track.append(&line);

    row.append(&track);
    row
}

fn day_header(
    date: Option<NaiveDate>,
    count: usize,
    timezone: Option<&str>,
    lang: Language,
) -> gtk::Box {
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    header.add_css_class("agenda-day-section");
    header.append(&label(
        &day_title(date, timezone, lang),
        &["agenda-section-title"],
        0.0,
        false,
    ));
    header.append(&label(
        &event_count_label(count, lang),
        &["subtle"],
        0.0,
        false,
    ));
    header
}

fn day_title(date: Option<NaiveDate>, timezone: Option<&str>, lang: Language) -> String {
    let Some(date) = date else {
        return translate(lang, "upcoming").to_string();
    };

    let today = now_parts_for_timezone(timezone).0;
    if date == today {
        return translate(lang, "today").to_string();
    }
    if date == today + chrono::Duration::days(1) {
        return translate(lang, "tomorrow").to_string();
    }
    if lang == Language::Chinese {
        format!("{}月{}日", date.month(), date.day())
    } else {
        date.format("%A, %B %-d").to_string()
    }
}

fn rail_time_label(event: &Event, timezone: Option<&str>, lang: Language) -> String {
    match parse_event_start_for_timezone(&event.start, timezone) {
        Some((_, Some(time))) if lang == Language::Chinese => time.format("%H:%M").to_string(),
        Some((_, Some(time))) => time.format("%-I:%M").to_string(),
        Some((_, None)) => translate(lang, "all_day").to_string(),
        None => translate(lang, "time_unavailable").to_string(),
    }
}

fn view_should_show_now_line(
    view: AgendaViewMode,
    selected_day: Option<NaiveDate>,
    today: NaiveDate,
) -> bool {
    match view {
        AgendaViewMode::Now | AgendaViewMode::Upcoming => true,
        AgendaViewMode::Day => selected_day.unwrap_or(today) == today,
        AgendaViewMode::Month => false,
    }
}

fn should_insert_now_before(event: &Event, today: NaiveDate, timezone: Option<&str>) -> bool {
    let (_, now) = now_parts_for_timezone(timezone);
    if event_is_current(event, today, timezone) {
        return true;
    }

    matches!(
        parse_event_start_for_timezone(&event.start, timezone),
        Some((date, Some(start_time))) if date == today && start_time >= now
    )
}

fn event_is_current(event: &Event, today: NaiveDate, timezone: Option<&str>) -> bool {
    let (_, now) = now_parts_for_timezone(timezone);
    let Some((start_date, Some(start_time))) =
        parse_event_start_for_timezone(&event.start, timezone)
    else {
        return false;
    };
    if start_date != today || start_time > now {
        return false;
    }

    let Some((end_date, end_time)) = parse_event_start_for_timezone(&event.end, timezone) else {
        return false;
    };
    match end_time {
        Some(end_time) => (end_date, end_time) > (today, now),
        None => end_date > today,
    }
}
