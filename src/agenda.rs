use crate::cache::{cache_is_fresh, read_cache, write_cache};
use crate::date::{event_days, format_day_label, format_time_label, parse_event_start};
use crate::gws::fetch_events;
use crate::model::{AgendaResult, AgendaState, Event};
use crate::ui::{add_escape_to_close, clear_box, label};
use adw::prelude::*;
use chrono::{DateTime, Datelike, Local, NaiveDate};
use gtk::glib;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;

pub fn build_window(app: &adw::Application, days: u32) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Google Calendar")
        .default_width(900)
        .default_height(500)
        .resizable(false)
        .build();
    window.set_decorated(false);

    let root = gtk::Box::new(gtk::Orientation::Vertical, 12);
    root.add_css_class("panel");

    let topbar = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    topbar.add_css_class("topbar");
    topbar.append(&label("Google Calendar", &["title"], 0.0, false));

    let status_label = label(
        &Local::now().format("%a, %b %-d  %-I:%M %p").to_string(),
        &["muted"],
        0.0,
        false,
    );
    topbar.append(&status_label);

    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    topbar.append(&spacer);

    let refresh = gtk::Button::with_label("Refresh");
    refresh.add_css_class("action-button");
    topbar.append(&refresh);

    let close = gtk::Button::with_label("x");
    close.add_css_class("close-button");
    {
        let window = window.clone();
        close.connect_clicked(move |_| window.close());
    }
    topbar.append(&close);
    root.append(&topbar);

    let content = gtk::Box::new(gtk::Orientation::Horizontal, 14);
    root.append(&content);
    window.set_content(Some(&root));
    add_escape_to_close(&window);
    window.present();

    let initial_cache = read_cache(days);
    let state = Rc::new(RefCell::new(match &initial_cache {
        Some(cache) => AgendaState {
            events: cache.events.clone(),
            error: None,
            fetched_at: Some(cache.fetched_at),
            loading: false,
            cached: true,
        },
        None => AgendaState {
            events: Vec::new(),
            error: None,
            fetched_at: None,
            loading: false,
            cached: false,
        },
    }));

    {
        let content = content.clone();
        let status_label = status_label.clone();
        let refresh = refresh.clone();
        let state = state.clone();
        refresh.clone().connect_clicked(move |_| {
            start_fetch(
                days,
                content.clone(),
                status_label.clone(),
                refresh.clone(),
                state.clone(),
            );
        });
    }

    let should_fetch = initial_cache
        .as_ref()
        .map(|cache| !cache_is_fresh(cache.fetched_at))
        .unwrap_or(true);
    if should_fetch {
        start_fetch(days, content, status_label, refresh, state);
    } else {
        render_agenda(&content, &status_label, &refresh, days, &state.borrow());
    }
}

fn start_fetch(
    days: u32,
    content: gtk::Box,
    status_label: gtk::Label,
    refresh: gtk::Button,
    state: Rc<RefCell<AgendaState>>,
) {
    if state.borrow().loading {
        return;
    }

    {
        let mut state = state.borrow_mut();
        state.loading = true;
        state.error = None;
    }
    render_agenda(&content, &status_label, &refresh, days, &state.borrow());

    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let result = match fetch_events(days) {
            Ok(events) => AgendaResult {
                events,
                error: None,
            },
            Err(error) => AgendaResult {
                events: Vec::new(),
                error: Some(error),
            },
        };
        let _ = sender.send(result);
    });

    glib::timeout_add_local(Duration::from_millis(80), move || {
        match receiver.try_recv() {
            Ok(result) => {
                {
                    let mut state = state.borrow_mut();
                    state.loading = false;
                    if let Some(error) = result.error {
                        state.error = Some(error);
                        state.cached = !state.events.is_empty();
                    } else {
                        let fetched_at = Local::now();
                        write_cache(days, &result.events, fetched_at);
                        state.events = result.events;
                        state.error = None;
                        state.fetched_at = Some(fetched_at);
                        state.cached = false;
                    }
                }
                render_agenda(&content, &status_label, &refresh, days, &state.borrow());
                glib::ControlFlow::Break
            }
            Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(mpsc::TryRecvError::Disconnected) => {
                {
                    let mut state = state.borrow_mut();
                    state.loading = false;
                    state.error = Some("Calendar refresh worker stopped unexpectedly.".to_string());
                }
                render_agenda(&content, &status_label, &refresh, days, &state.borrow());
                glib::ControlFlow::Break
            }
        }
    });
}

fn render_agenda(
    content: &gtk::Box,
    status_label: &gtk::Label,
    refresh: &gtk::Button,
    days: u32,
    state: &AgendaState,
) {
    clear_box(content);
    content.append(&build_event_calendar(&event_days(&state.events)));
    content.append(&build_agenda_list(days, state));

    refresh.set_sensitive(!state.loading);
    refresh.set_label(if state.loading {
        "Refreshing"
    } else {
        "Refresh"
    });
    status_label.set_text(&agenda_status(state));
}

fn build_event_calendar(event_days: &BTreeSet<NaiveDate>) -> gtk::Box {
    let today = Local::now().date_naive();
    let pane = gtk::Box::new(gtk::Orientation::Vertical, 12);
    pane.add_css_class("left-pane");
    pane.set_size_request(285, -1);

    pane.append(&label(
        &today.format("%B %Y").to_string(),
        &["month-title"],
        0.0,
        false,
    ));

    let grid = gtk::Grid::builder()
        .column_spacing(6)
        .row_spacing(7)
        .build();

    for (col, weekday) in ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"]
        .iter()
        .enumerate()
    {
        let item = label(weekday, &["weekday"], 0.5, false);
        item.set_size_request(30, 22);
        grid.attach(&item, col as i32, 0, 1, 1);
    }

    for (index, day) in crate::date::month_dates(today.year(), today.month())
        .iter()
        .enumerate()
    {
        let row = index / 7 + 1;
        let col = index % 7;
        let text = if day.month() == today.month() {
            day.day().to_string()
        } else {
            String::new()
        };
        let item = label(&text, &["date-cell"], 0.5, false);
        item.set_size_request(30, 30);
        if day.weekday().number_from_monday() >= 6 {
            item.add_css_class("weekend");
        }
        if event_days.contains(day) {
            item.add_css_class("event-day");
        }
        if *day == today {
            item.add_css_class("today");
        }
        grid.attach(&item, col as i32, row as i32, 1, 1);
    }

    pane.append(&grid);
    pane.append(&label(
        "Highlighted days have agenda items.",
        &["muted"],
        0.0,
        true,
    ));
    pane
}

fn build_agenda_list(days: u32, state: &AgendaState) -> gtk::Box {
    let right = gtk::Box::new(gtk::Orientation::Vertical, 10);
    right.set_hexpand(true);

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    header.append(&label("Agenda", &["agenda-header"], 0.0, false));
    header.append(&label(
        &format!("Next {days} days"),
        &["subtle"],
        0.0,
        false,
    ));
    header.append(&label(
        &format!("{} events", state.events.len()),
        &["accent"],
        0.0,
        false,
    ));
    if state.loading {
        header.append(&label("Loading", &["subtle"], 0.0, false));
    } else if state.cached {
        header.append(&label("Cached", &["subtle"], 0.0, false));
    }
    right.append(&header);

    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_vexpand(true);

    let list = gtk::Box::new(gtk::Orientation::Vertical, 8);
    if state.loading && state.events.is_empty() {
        list.append(&message_card(
            "Loading Google Calendar",
            Some("The window is ready while agenda data updates."),
            true,
        ));
    } else if let Some(error) = &state.error {
        if state.events.is_empty() {
            list.append(&message_card(
                "Could not read Google Calendar",
                Some(error),
                false,
            ));
        } else {
            list.append(&message_card("Refresh failed", Some(error), false));
            for event in &state.events {
                list.append(&event_card(event));
            }
        }
    } else if state.loading {
        list.append(&message_card(
            "Refreshing",
            Some("Showing cached events while Google Calendar updates."),
            true,
        ));
        for event in &state.events {
            list.append(&event_card(event));
        }
    } else if state.events.is_empty() {
        list.append(&message_card(
            "No upcoming events",
            Some(&format!("You are clear for the next {days} days.")),
            false,
        ));
    } else {
        for event in &state.events {
            list.append(&event_card(event));
        }
    }

    scroll.set_child(Some(&list));
    right.append(&scroll);
    right
}

fn event_card(event: &Event) -> gtk::Box {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 6);
    card.add_css_class("agenda-card");

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
    card.append(&meta);

    let title = if event.summary.trim().is_empty() {
        "Untitled event"
    } else {
        event.summary.trim()
    };
    let title_label = label(title, &["event-title"], 0.0, true);
    title_label.set_max_width_chars(54);
    card.append(&title_label);

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
    card.append(&details);
    card
}

fn message_card(title: &str, detail: Option<&str>, spinner: bool) -> gtk::Box {
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

fn agenda_status(state: &AgendaState) -> String {
    if state.loading && state.events.is_empty() {
        return "Loading".to_string();
    }
    if let Some(error) = &state.error {
        if state.events.is_empty() {
            return "Refresh failed".to_string();
        }
        return format!(
            "{}; refresh failed",
            cache_status(state.fetched_at, "Cached", Some(error))
        );
    }
    if state.loading {
        return format!(
            "{}; refreshing",
            cache_status(state.fetched_at, "Cached", None)
        );
    }
    if state.cached {
        return cache_status(state.fetched_at, "Cached", None);
    }
    state
        .fetched_at
        .map(|time| format!("Updated {}", time.format("%H:%M")))
        .unwrap_or_else(|| Local::now().format("%a, %b %-d  %-I:%M %p").to_string())
}

fn cache_status(fetched_at: Option<DateTime<Local>>, prefix: &str, suffix: Option<&str>) -> String {
    let base = match fetched_at {
        Some(time) => {
            let age = (Local::now() - time).num_seconds().max(0);
            if age < 60 {
                format!("{prefix} just now")
            } else if age < 3600 {
                format!("{prefix} {} min ago", age / 60)
            } else {
                format!("{prefix} {} h ago", age / 3600)
            }
        }
        None => prefix.to_string(),
    };
    match suffix {
        Some(suffix) if !suffix.is_empty() => format!("{base}: {suffix}"),
        _ => base,
    }
}
