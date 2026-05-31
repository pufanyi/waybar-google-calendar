use super::{cards, status};
use crate::agenda::{AgendaApp, auth_prompt};
use crate::calendar::date::event_date_for_timezone;
use crate::calendar::model::{AgendaQuery, AgendaState, Event};
use crate::i18n::translate;
use crate::storage::settings::Language;
use crate::ui::label;
use adw::prelude::*;
use chrono::NaiveDate;
use relm4::ComponentSender;

pub(super) fn build(
    query: &AgendaQuery,
    state: &AgendaState,
    selected_day: Option<NaiveDate>,
    authenticating: bool,
    lang: Language,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    let right = gtk::Box::new(gtk::Orientation::Vertical, 10);
    right.set_hexpand(true);

    let focus_auth_prompt = auth_prompt::should_focus(state, authenticating);
    let visible_events = if focus_auth_prompt {
        Vec::new()
    } else {
        visible_events(&state.events, selected_day, query.timezone.as_deref())
    };

    right.append(&header(
        query,
        state,
        selected_day,
        focus_auth_prompt,
        visible_events.len(),
        lang,
    ));
    right.append(&scrolling_body(BodyRender {
        state,
        selected_day,
        authenticating,
        focus_auth_prompt,
        visible_events,
        timezone: query.timezone.as_deref(),
        lang,
        sender,
    }));
    right
}

struct BodyRender<'a> {
    state: &'a AgendaState,
    selected_day: Option<NaiveDate>,
    authenticating: bool,
    focus_auth_prompt: bool,
    visible_events: Vec<&'a Event>,
    timezone: Option<&'a str>,
    lang: Language,
    sender: ComponentSender<AgendaApp>,
}

fn header(
    query: &AgendaQuery,
    state: &AgendaState,
    selected_day: Option<NaiveDate>,
    focus_auth_prompt: bool,
    visible_event_count: usize,
    lang: Language,
) -> gtk::Box {
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    header.append(&label(
        translate(lang, "agenda"),
        &["agenda-header"],
        0.0,
        false,
    ));
    let range_text = selected_day
        .map(|day| day.format("%a %b %-d").to_string())
        .unwrap_or_else(|| status::range(state.range, lang));
    header.append(&label(&range_text, &["subtle"], 0.0, false));
    if let Some(calendar) = &query.calendar {
        header.append(&label(calendar, &["pill"], 0.0, false));
    }
    let status_text = if focus_auth_prompt {
        translate(lang, "action_required").to_string()
    } else {
        event_count_label(visible_event_count, lang)
    };
    header.append(&label(&status_text, &["accent"], 0.0, false));
    header
}

fn scrolling_body(render: BodyRender<'_>) -> gtk::ScrolledWindow {
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_vexpand(true);
    scroll.set_child(Some(&body(render)));
    scroll
}

fn body(render: BodyRender<'_>) -> gtk::Box {
    let list = gtk::Box::new(gtk::Orientation::Vertical, 8);
    if render.focus_auth_prompt {
        list.append(&auth_prompt::prompt_card(
            render.state.error.as_deref().unwrap_or_default(),
            render.authenticating,
            render.lang,
            render.sender,
        ));
    } else if render.state.loading && render.state.events.is_empty() {
        list.append(&cards::message(
            translate(render.lang, "loading_google_calendar"),
            Some(translate(render.lang, "window_ready_updates")),
            true,
        ));
    } else if let Some(error) = &render.state.error {
        append_error_state(
            &list,
            error,
            render.visible_events,
            render.timezone,
            render.lang,
        );
    } else if render.state.loading {
        list.append(&cards::message(
            translate(render.lang, "refreshing"),
            Some(translate(render.lang, "showing_cached_refreshing")),
            true,
        ));
        append_events(&list, render.visible_events, render.timezone, render.lang);
    } else if render.visible_events.is_empty() {
        let detail = empty_detail(render.selected_day, render.lang);
        list.append(&cards::message(
            translate(render.lang, "no_upcoming_events"),
            Some(&detail),
            false,
        ));
    } else {
        append_events(&list, render.visible_events, render.timezone, render.lang);
    }
    list
}

fn append_error_state(
    list: &gtk::Box,
    error: &str,
    visible_events: Vec<&Event>,
    timezone: Option<&str>,
    lang: Language,
) {
    if visible_events.is_empty() {
        list.append(&cards::message(
            translate(lang, "refresh_failed"),
            Some(error),
            false,
        ));
    } else {
        list.append(&cards::message(
            translate(lang, "refresh_failed"),
            Some(error),
            false,
        ));
        append_events(list, visible_events, timezone, lang);
    }
}

fn append_events(list: &gtk::Box, events: Vec<&Event>, timezone: Option<&str>, lang: Language) {
    for event in events {
        list.append(&cards::event(event, timezone, lang));
    }
}

fn event_count_label(count: usize, lang: Language) -> String {
    if lang == Language::Chinese {
        return format!("{}{}", count, translate(lang, "events"));
    }

    let key = if count == 1 { "event" } else { "events" };
    format!("{} {}", count, translate(lang, key))
}

fn empty_detail(selected_day: Option<NaiveDate>, lang: Language) -> String {
    selected_day
        .map(|day| {
            if lang == Language::Chinese {
                format!(
                    "{}{}。",
                    translate(lang, "no_loaded_events_day"),
                    day.format("%Y-%m-%d")
                )
            } else {
                format!(
                    "{}{}.",
                    translate(lang, "no_loaded_events_day"),
                    day.format("%A, %B %-d")
                )
            }
        })
        .unwrap_or_else(|| translate(lang, "no_loaded_events_view").to_string())
}

fn visible_events<'a>(
    events: &'a [Event],
    selected_day: Option<NaiveDate>,
    timezone: Option<&str>,
) -> Vec<&'a Event> {
    events
        .iter()
        .filter(|event| {
            selected_day
                .map(|day| event_date_for_timezone(event, timezone) == Some(day))
                .unwrap_or(true)
        })
        .collect()
}
