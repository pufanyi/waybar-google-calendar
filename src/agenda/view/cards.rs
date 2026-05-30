use crate::calendar::date::{format_day_label, format_time_label, parse_event_start};
use crate::calendar::model::Event;
use crate::ui::label;
use adw::prelude::*;

pub(super) fn event(event: &Event) -> gtk::Box {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 6);
    card.add_css_class("agenda-card");

    card.append(&event_meta(event));
    card.append(&event_title(event));
    card.append(&event_details(event));
    card
}

pub(super) fn message(title: &str, detail: Option<&str>, spinner: bool) -> gtk::Box {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 6);
    card.add_css_class("empty-card");

    if spinner {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let progress = gtk::Spinner::new();
        progress.start();
        row.append(&progress);
        row.append(&label(title, &["event-title"], 0.0, false));
        card.append(&row);
    } else {
        card.append(&label(title, &["event-title"], 0.0, false));
    }

    if let Some(detail) = detail {
        card.append(&label(detail, &["muted"], 0.0, true));
    }
    card
}

fn event_meta(event: &Event) -> gtk::Box {
    let start = parse_event_start(&event.start);
    let meta = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    meta.append(&label(
        &start
            .map(|(date, _)| format_day_label(date))
            .unwrap_or_else(|| "Upcoming".to_string()),
        &["event-date"],
        0.0,
        false,
    ));
    meta.append(&label(
        &format_time_label(&event.start, &event.end),
        &["event-time"],
        0.0,
        false,
    ));
    meta
}

fn event_title(event: &Event) -> gtk::Label {
    let title = if event.summary.trim().is_empty() {
        "Untitled event"
    } else {
        event.summary.trim()
    };
    let title_label = label(title, &["event-title"], 0.0, true);
    title_label.set_max_width_chars(54);
    title_label
}

fn event_details(event: &Event) -> gtk::Box {
    let details = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let calendar = if event.calendar.trim().is_empty() {
        "Calendar"
    } else {
        event.calendar.trim()
    };
    let pill = label(calendar, &["pill"], 0.0, false);
    pill.set_max_width_chars(26);
    details.append(&pill);
    if !event.location.trim().is_empty() {
        let place = label(
            &format!("@ {}", event.location.trim()),
            &["muted"],
            0.0,
            false,
        );
        place.set_max_width_chars(34);
        details.append(&place);
    }
    details
}
