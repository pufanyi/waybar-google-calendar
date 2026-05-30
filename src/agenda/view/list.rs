use super::{cards, status};
use crate::agenda::{AgendaApp, auth_prompt};
use crate::calendar::date::event_date;
use crate::calendar::model::{AgendaQuery, AgendaState, Event};
use crate::ui::label;
use adw::prelude::*;
use chrono::NaiveDate;
use relm4::ComponentSender;

pub(super) fn build(
    query: &AgendaQuery,
    state: &AgendaState,
    selected_day: Option<NaiveDate>,
    authenticating: bool,
    auth_page: usize,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    let right = gtk::Box::new(gtk::Orientation::Vertical, 10);
    right.set_hexpand(true);

    let focus_auth_prompt = auth_prompt::should_focus(state, authenticating);
    let visible_events = if focus_auth_prompt {
        Vec::new()
    } else {
        visible_events(&state.events, selected_day)
    };

    right.append(&header(
        query,
        state,
        selected_day,
        focus_auth_prompt,
        visible_events.len(),
    ));
    right.append(&scrolling_body(
        state,
        selected_day,
        authenticating,
        auth_page,
        focus_auth_prompt,
        visible_events,
        sender,
    ));
    right
}

fn header(
    query: &AgendaQuery,
    state: &AgendaState,
    selected_day: Option<NaiveDate>,
    focus_auth_prompt: bool,
    visible_event_count: usize,
) -> gtk::Box {
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    header.append(&label("Agenda", &["agenda-header"], 0.0, false));
    let range_text = selected_day
        .map(|day| day.format("%a %b %-d").to_string())
        .unwrap_or_else(|| status::range(state.range));
    header.append(&label(&range_text, &["subtle"], 0.0, false));
    if let Some(calendar) = &query.calendar {
        header.append(&label(calendar, &["pill"], 0.0, false));
    }
    let status_text = if focus_auth_prompt {
        "Action required".to_string()
    } else {
        format!("{visible_event_count} events")
    };
    header.append(&label(&status_text, &["accent"], 0.0, false));
    header
}

fn scrolling_body(
    state: &AgendaState,
    selected_day: Option<NaiveDate>,
    authenticating: bool,
    auth_page: usize,
    focus_auth_prompt: bool,
    visible_events: Vec<&Event>,
    sender: ComponentSender<AgendaApp>,
) -> gtk::ScrolledWindow {
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_vexpand(true);
    scroll.set_child(Some(&body(
        state,
        selected_day,
        authenticating,
        auth_page,
        focus_auth_prompt,
        visible_events,
        sender,
    )));
    scroll
}

fn body(
    state: &AgendaState,
    selected_day: Option<NaiveDate>,
    authenticating: bool,
    auth_page: usize,
    focus_auth_prompt: bool,
    visible_events: Vec<&Event>,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    let list = gtk::Box::new(gtk::Orientation::Vertical, 8);
    if focus_auth_prompt {
        list.append(&auth_prompt::prompt_card(
            state.error.as_deref().unwrap_or_default(),
            authenticating,
            auth_page,
            sender,
        ));
    } else if state.loading && state.events.is_empty() {
        list.append(&cards::message(
            "Loading Google Calendar",
            Some("The window is ready while agenda data updates."),
            true,
        ));
    } else if let Some(error) = &state.error {
        append_error_state(&list, error, visible_events);
    } else if state.loading {
        list.append(&cards::message(
            "Refreshing",
            Some("Showing cached events while Google Calendar updates."),
            true,
        ));
        append_events(&list, visible_events);
    } else if visible_events.is_empty() {
        let detail = selected_day
            .map(|day| format!("No loaded events for {}.", day.format("%A, %B %-d")))
            .unwrap_or_else(|| "No loaded events for this calendar view.".to_string());
        list.append(&cards::message("No upcoming events", Some(&detail), false));
    } else {
        append_events(&list, visible_events);
    }
    list
}

fn append_error_state(list: &gtk::Box, error: &str, visible_events: Vec<&Event>) {
    if visible_events.is_empty() {
        list.append(&cards::message("Refresh failed", Some(error), false));
    } else {
        list.append(&cards::message("Refresh failed", Some(error), false));
        append_events(list, visible_events);
    }
}

fn append_events(list: &gtk::Box, events: Vec<&Event>) {
    for event in events {
        list.append(&cards::event(event));
    }
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
