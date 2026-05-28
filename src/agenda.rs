use crate::cache::{cache_is_fresh, read_cache, write_cache};
use crate::date::{
    event_days, format_day_label, format_time_label, month_dates, parse_event_start,
};
use crate::gws::fetch_events;
use crate::model::{AgendaResult, AgendaState, Event};
use crate::ui::{add_escape_to_close, clear_box, label};
use adw::prelude::*;
use chrono::{DateTime, Datelike, Local, NaiveDate};
use relm4::{Component, ComponentParts, ComponentSender};
use std::collections::BTreeSet;

#[derive(Debug)]
pub struct AgendaInit {
    pub days: u32,
}

#[derive(Debug)]
pub struct AgendaApp {
    days: u32,
    state: AgendaState,
}

#[derive(Debug)]
pub enum AgendaMsg {
    Refresh,
}

pub struct AgendaWidgets {
    content: gtk::Box,
    status_label: gtk::Label,
    refresh: gtk::Button,
}

impl Component for AgendaApp {
    type Init = AgendaInit;
    type Input = AgendaMsg;
    type Output = ();
    type CommandOutput = AgendaResult;
    type Root = adw::ApplicationWindow;
    type Widgets = AgendaWidgets;

    fn init_root() -> Self::Root {
        let root = adw::ApplicationWindow::builder()
            .title("Google Calendar")
            .default_width(900)
            .default_height(500)
            .resizable(false)
            .build();
        root.set_decorated(false);
        root
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let root_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
        root_box.add_css_class("panel");

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
        {
            let sender = sender.clone();
            refresh.connect_clicked(move |_| sender.input(AgendaMsg::Refresh));
        }
        topbar.append(&refresh);

        let close = gtk::Button::with_label("x");
        close.add_css_class("close-button");
        {
            let root = root.clone();
            close.connect_clicked(move |_| root.close());
        }
        topbar.append(&close);
        root_box.append(&topbar);

        let content = gtk::Box::new(gtk::Orientation::Horizontal, 14);
        root_box.append(&content);
        root.set_content(Some(&root_box));
        add_escape_to_close(&root);

        let initial_cache = read_cache(init.days);
        let state = match &initial_cache {
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
        };
        let model = AgendaApp {
            days: init.days,
            state,
        };

        let mut widgets = AgendaWidgets {
            content,
            status_label,
            refresh,
        };
        render_agenda(&model, &mut widgets);

        let should_fetch = initial_cache
            .as_ref()
            .map(|cache| !cache_is_fresh(cache.fetched_at))
            .unwrap_or(true);
        if should_fetch {
            sender.input(AgendaMsg::Refresh);
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            AgendaMsg::Refresh => {
                if self.state.loading {
                    return;
                }
                self.state.loading = true;
                self.state.error = None;

                let days = self.days;
                sender.spawn_oneshot_command(move || match fetch_events(days) {
                    Ok(events) => AgendaResult {
                        events,
                        error: None,
                    },
                    Err(error) => AgendaResult {
                        events: Vec::new(),
                        error: Some(error),
                    },
                });
            }
        }
    }

    fn update_cmd(
        &mut self,
        result: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        self.state.loading = false;
        if let Some(error) = result.error {
            self.state.error = Some(error);
            self.state.cached = !self.state.events.is_empty();
        } else {
            let fetched_at = Local::now();
            write_cache(self.days, &result.events, fetched_at);
            self.state.events = result.events;
            self.state.error = None;
            self.state.fetched_at = Some(fetched_at);
            self.state.cached = false;
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        render_agenda(self, widgets);
    }
}

fn render_agenda(model: &AgendaApp, widgets: &mut AgendaWidgets) {
    clear_box(&widgets.content);
    widgets
        .content
        .append(&build_event_calendar(&event_days(&model.state.events)));
    widgets
        .content
        .append(&build_agenda_list(model.days, &model.state));

    widgets.refresh.set_sensitive(!model.state.loading);
    widgets.refresh.set_label(if model.state.loading {
        "Refreshing"
    } else {
        "Refresh"
    });
    widgets.status_label.set_text(&agenda_status(&model.state));
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

    for (index, day) in month_dates(today.year(), today.month()).iter().enumerate() {
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
