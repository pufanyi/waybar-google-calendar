use crate::cache::{cache_is_fresh, read_cache, write_cache};
use crate::date::{
    event_date, event_days, format_day_label, format_time_label, month_dates, month_name,
    parse_event_start,
};
use crate::gws::fetch_events;
use crate::model::{AgendaQuery, AgendaResult, AgendaState, Event};
use crate::ui::{add_escape_to_close, classed_button, clear_box, label};
use adw::prelude::*;
use chrono::{DateTime, Datelike, Local, NaiveDate};
use relm4::{Component, ComponentParts, ComponentSender};
use std::collections::BTreeSet;

#[derive(Debug)]
pub struct AgendaInit {
    pub query: AgendaQuery,
}

#[derive(Debug)]
pub struct AgendaApp {
    query: AgendaQuery,
    state: AgendaState,
    calendar_year: i32,
    calendar_month: u32,
    selected_day: Option<NaiveDate>,
}

#[derive(Debug)]
pub enum AgendaMsg {
    Refresh,
    PreviousMonth,
    NextMonth,
    Today,
    ClearSelection,
    SelectDay(NaiveDate),
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

        let today = Local::now().date_naive();
        let initial_cache = read_cache(&init.query);
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
            query: init.query,
            state,
            calendar_year: today.year(),
            calendar_month: today.month(),
            selected_day: None,
        };

        let mut widgets = AgendaWidgets {
            content,
            status_label,
            refresh,
        };
        render_agenda(&model, &mut widgets, sender.clone());

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

                let query = self.query.clone();
                sender.spawn_oneshot_command(move || match fetch_events(&query) {
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
            AgendaMsg::PreviousMonth => {
                if self.calendar_month == 1 {
                    self.calendar_month = 12;
                    self.calendar_year -= 1;
                } else {
                    self.calendar_month -= 1;
                }
            }
            AgendaMsg::NextMonth => {
                if self.calendar_month == 12 {
                    self.calendar_month = 1;
                    self.calendar_year += 1;
                } else {
                    self.calendar_month += 1;
                }
            }
            AgendaMsg::Today => {
                let today = Local::now().date_naive();
                self.calendar_year = today.year();
                self.calendar_month = today.month();
                self.selected_day = Some(today);
            }
            AgendaMsg::ClearSelection => {
                self.selected_day = None;
            }
            AgendaMsg::SelectDay(day) => {
                self.calendar_year = day.year();
                self.calendar_month = day.month();
                self.selected_day = Some(day);
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
            write_cache(&self.query, &result.events, fetched_at);
            self.state.events = result.events;
            self.state.error = None;
            self.state.fetched_at = Some(fetched_at);
            self.state.cached = false;
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
        render_agenda(self, widgets, sender);
    }
}

fn render_agenda(
    model: &AgendaApp,
    widgets: &mut AgendaWidgets,
    sender: ComponentSender<AgendaApp>,
) {
    clear_box(&widgets.content);
    widgets.content.append(&build_event_calendar(
        model,
        &event_days(&model.state.events),
        sender,
    ));
    widgets.content.append(&build_agenda_list(
        &model.query,
        &model.state,
        model.selected_day,
    ));

    widgets.refresh.set_sensitive(!model.state.loading);
    widgets.refresh.set_label(if model.state.loading {
        "Refreshing"
    } else {
        "Refresh"
    });
    widgets.status_label.set_text(&agenda_status(&model.state));
}

fn build_event_calendar(
    model: &AgendaApp,
    event_days: &BTreeSet<NaiveDate>,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    let today = Local::now().date_naive();
    let pane = gtk::Box::new(gtk::Orientation::Vertical, 12);
    pane.add_css_class("left-pane");
    pane.set_size_request(292, -1);
    pane.set_halign(gtk::Align::Start);
    pane.set_hexpand(false);

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let previous = classed_button("<", &["nav-button", "icon-button"]);
    let next = classed_button(">", &["nav-button", "icon-button"]);
    let title = label(
        &format!(
            "{} {}",
            month_name(model.calendar_month),
            model.calendar_year
        ),
        &["month-title"],
        0.5,
        false,
    );
    title.set_hexpand(true);

    {
        let sender = sender.clone();
        previous.connect_clicked(move |_| sender.input(AgendaMsg::PreviousMonth));
    }
    {
        let sender = sender.clone();
        next.connect_clicked(move |_| sender.input(AgendaMsg::NextMonth));
    }

    header.append(&previous);
    header.append(&title);
    header.append(&next);
    pane.append(&header);

    let grid = gtk::Grid::builder()
        .column_spacing(5)
        .row_spacing(5)
        .build();
    grid.set_halign(gtk::Align::Center);

    for (col, weekday) in ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"]
        .iter()
        .enumerate()
    {
        let item = label(weekday, &["weekday"], 0.5, false);
        item.set_size_request(30, 22);
        grid.attach(&item, col as i32, 0, 1, 1);
    }

    for (index, day) in month_dates(model.calendar_year, model.calendar_month)
        .iter()
        .enumerate()
    {
        let row = index / 7 + 1;
        let col = index % 7;
        let has_event = event_days.contains(day);
        let item = calendar_day_button(day.day(), has_event);
        item.set_size_request(34, 34);
        if day.month() != model.calendar_month {
            item.add_css_class("outside");
        }
        if day.weekday().number_from_monday() >= 6 {
            item.add_css_class("weekend");
        }
        if has_event {
            item.add_css_class("event-day");
        }
        if *day == today {
            item.add_css_class("today");
        }
        if Some(*day) == model.selected_day {
            item.add_css_class("selected");
        }

        let selected_day = *day;
        let sender = sender.clone();
        item.connect_clicked(move |_| sender.input(AgendaMsg::SelectDay(selected_day)));

        grid.attach(&item, col as i32, row as i32, 1, 1);
    }

    pane.append(&grid);

    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actions.add_css_class("calendar-actions");
    let all = classed_button("All", &["action-button"]);
    if model.selected_day.is_none() {
        all.add_css_class("selected");
    }
    let today_button = classed_button("Today", &["action-button"]);
    if model.selected_day == Some(today) {
        today_button.add_css_class("selected");
    }

    {
        let sender = sender.clone();
        all.connect_clicked(move |_| sender.input(AgendaMsg::ClearSelection));
    }
    {
        let sender = sender.clone();
        today_button.connect_clicked(move |_| sender.input(AgendaMsg::Today));
    }

    actions.append(&all);
    actions.append(&today_button);
    pane.append(&actions);
    pane
}

fn calendar_day_button(day: u32, has_event: bool) -> gtk::Button {
    let button = gtk::Button::new();
    button.add_css_class("date-cell");

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.add_css_class("date-cell-content");
    content.set_halign(gtk::Align::Center);
    content.set_valign(gtk::Align::Center);

    let number = label(&day.to_string(), &["date-number"], 0.5, false);
    number.set_halign(gtk::Align::Center);
    content.append(&number);

    let dot = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    dot.add_css_class("event-dot");
    if !has_event {
        dot.add_css_class("empty");
    }
    dot.set_halign(gtk::Align::Center);
    content.append(&dot);

    button.set_child(Some(&content));
    button
}

fn build_agenda_list(
    query: &AgendaQuery,
    state: &AgendaState,
    selected_day: Option<NaiveDate>,
) -> gtk::Box {
    let right = gtk::Box::new(gtk::Orientation::Vertical, 10);
    right.set_hexpand(true);
    let visible_events = visible_events(&state.events, selected_day);

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    header.append(&label("Agenda", &["agenda-header"], 0.0, false));
    let range_text = selected_day
        .map(|day| day.format("%a %b %-d").to_string())
        .unwrap_or_else(|| format!("Next {} days", query.days));
    header.append(&label(&range_text, &["subtle"], 0.0, false));
    if let Some(calendar) = &query.calendar {
        header.append(&label(calendar, &["pill"], 0.0, false));
    }
    header.append(&label(
        &format!("{} events", visible_events.len()),
        &["accent"],
        0.0,
        false,
    ));
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
            for event in visible_events {
                list.append(&event_card(event));
            }
        }
    } else if state.loading {
        list.append(&message_card(
            "Refreshing",
            Some("Showing cached events while Google Calendar updates."),
            true,
        ));
        for event in visible_events {
            list.append(&event_card(event));
        }
    } else if visible_events.is_empty() {
        let detail = selected_day
            .map(|day| format!("No loaded events for {}.", day.format("%A, %B %-d")))
            .unwrap_or_else(|| format!("You are clear for the next {} days.", query.days));
        list.append(&message_card("No upcoming events", Some(&detail), false));
    } else {
        for event in visible_events {
            list.append(&event_card(event));
        }
    }

    scroll.set_child(Some(&list));
    right.append(&scroll);
    right
}

fn visible_events(events: &[Event], selected_day: Option<NaiveDate>) -> Vec<&Event> {
    events
        .iter()
        .filter(|event| {
            selected_day
                .map(|day| event_date(event) == Some(day))
                .unwrap_or(true)
        })
        .collect()
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
