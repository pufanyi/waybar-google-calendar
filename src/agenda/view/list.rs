mod timeline;

use super::{cards, editor, status};
use crate::agenda::{AgendaApp, AgendaEditor, AgendaMsg, AgendaViewMode, auth_prompt};
use crate::calendar::date::{
    event_date_for_timezone, format_now_date_for_timezone, format_now_time_for_timezone,
    format_time_label_for_timezone, now_parts_for_timezone, parse_event_start_for_timezone,
};
use crate::calendar::model::{AgendaState, Event};
use crate::i18n::translate;
use crate::storage::settings::Language;
use crate::ui::{classed_button, icon_button, label};
use adw::prelude::*;
use chrono::{Datelike, NaiveDate, NaiveTime};
use relm4::ComponentSender;

pub(super) fn build(model: &AgendaApp, sender: ComponentSender<AgendaApp>) -> gtk::Box {
    let right = gtk::Box::new(gtk::Orientation::Vertical, 10);
    right.add_css_class("agenda-pane");
    right.set_hexpand(true);

    let lang = model.language();
    let timezone = model.query.timezone.as_deref();
    let focus_auth_prompt = auth_prompt::should_focus(&model.state, model.authenticating);
    let upcoming = upcoming_events(&model.state.events, timezone);
    let visible_events = if focus_auth_prompt {
        Vec::new()
    } else {
        events_for_view(model)
    };

    right.append(&context_bar(
        model,
        upcoming.first().copied(),
        !focus_auth_prompt,
        sender.clone(),
    ));
    if !focus_auth_prompt && model.event_editor == AgendaEditor::None {
        right.append(&view_tabs(model, sender.clone()));
        right.append(&header(
            model,
            focus_auth_prompt,
            visible_events.len(),
            lang,
        ));
    }
    right.append(&scrolling_body(BodyRender {
        model,
        state: &model.state,
        selected_day: model.selected_day,
        authenticating: model.authenticating,
        focus_auth_prompt,
        visible_events,
        timezone,
        lang,
        sender,
        view: model.agenda_view,
    }));
    right
}

struct BodyRender<'a> {
    model: &'a AgendaApp,
    state: &'a AgendaState,
    selected_day: Option<NaiveDate>,
    authenticating: bool,
    focus_auth_prompt: bool,
    visible_events: Vec<&'a Event>,
    timezone: Option<&'a str>,
    lang: Language,
    sender: ComponentSender<AgendaApp>,
    view: AgendaViewMode,
}

fn context_bar(
    model: &AgendaApp,
    next_event: Option<&Event>,
    can_edit: bool,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    let lang = model.language();
    let timezone = model.query.timezone.as_deref();
    let panel = gtk::Box::new(gtk::Orientation::Horizontal, 14);
    panel.add_css_class("agenda-context-bar");

    let clock = gtk::Box::new(gtk::Orientation::Vertical, 3);
    clock.add_css_class("agenda-context-date");
    clock.append(&label(translate(lang, "today"), &["subtle"], 0.0, false));
    clock.append(&label(
        &format_now_date_for_timezone(timezone, lang),
        &["agenda-context-title"],
        0.0,
        false,
    ));
    clock.append(&label(
        &format_now_time_for_timezone(timezone, lang),
        &["agenda-now-pill"],
        0.0,
        false,
    ));
    panel.append(&clock);

    let next = gtk::Box::new(gtk::Orientation::Vertical, 5);
    next.add_css_class("agenda-context-next");
    next.set_hexpand(true);
    next.append(&label(translate(lang, "next"), &["subtle"], 0.0, false));
    if let Some(event) = next_event {
        next.append(&label(
            event_summary(event, lang),
            &["event-title"],
            0.0,
            true,
        ));
        next.append(&label(
            &format_time_label_for_timezone(&event.start, &event.end, timezone, lang),
            &["event-time"],
            0.0,
            false,
        ));
    } else {
        next.append(&label(
            translate(lang, "no_upcoming_events"),
            &["event-title"],
            0.0,
            true,
        ));
        next.append(&label(
            translate(lang, "no_upcoming_events_detail"),
            &["muted"],
            0.0,
            true,
        ));
    }
    panel.append(&next);

    let add = icon_button(
        "list-add-symbolic",
        &["action-button", "icon-button"],
        translate(lang, "add_event"),
    );
    add.set_sensitive(can_edit && !model.mutating_event);
    add.connect_clicked(move |_| sender.input(AgendaMsg::ShowAddEvent));
    panel.append(&add);

    panel
}

fn view_tabs(model: &AgendaApp, sender: ComponentSender<AgendaApp>) -> gtk::Box {
    let tabs = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    tabs.add_css_class("agenda-view-tabs");
    for view in AgendaViewMode::ALL {
        let button = classed_button(view_label(view, model.language()), &["agenda-view-tab"]);
        if view == model.agenda_view {
            button.add_css_class("selected");
        }
        let sender = sender.clone();
        button.connect_clicked(move |_| sender.input(AgendaMsg::SetAgendaView(view)));
        tabs.append(&button);
    }
    tabs
}

fn header(
    model: &AgendaApp,
    focus_auth_prompt: bool,
    visible_event_count: usize,
    lang: Language,
) -> gtk::Box {
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    header.add_css_class("agenda-list-header");
    header.append(&label(
        view_title(model.agenda_view, lang),
        &["agenda-header"],
        0.0,
        false,
    ));
    header.append(&label(&range_text(model, lang), &["subtle"], 0.0, false));
    if let Some(calendar) = &model.query.calendar {
        header.append(&label(calendar, &["pill"], 0.0, false));
    }
    let status_text = if focus_auth_prompt {
        translate(lang, "action_required").to_string()
    } else {
        event_count_label(visible_event_count, lang)
    };
    header.append(&label(&status_text, &["accent"], 0.0, false));
    if model.state.loading
        && !model.state.events.is_empty()
        && model.event_editor == AgendaEditor::None
    {
        header.append(&inline_refresh(lang));
    }
    header
}

fn inline_refresh(lang: Language) -> gtk::Box {
    let container = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    container.add_css_class("agenda-inline-refresh");
    container.set_valign(gtk::Align::Center);

    let spinner = gtk::Spinner::new();
    spinner.add_css_class("agenda-inline-spinner");
    spinner.start();
    container.append(&spinner);
    container.append(&label(
        translate(lang, "refreshing"),
        &["agenda-inline-refresh-label"],
        0.0,
        false,
    ));
    container
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
    list.add_css_class("agenda-list");
    if render.model.event_editor != AgendaEditor::None {
        list.append(&editor::build(render.model, render.sender));
    } else if render.focus_auth_prompt {
        list.append(&auth_prompt::prompt_card(
            render.state.error.as_deref().unwrap_or_default(),
            render.authenticating,
            render.lang,
            render.sender,
        ));
    } else if let Some(message) = &render.state.error {
        let kind = status::state_message_kind(message, render.lang);
        if kind == status::StateMessageKind::Error {
            append_error_state(&list, message, &render);
        } else {
            append_status_banner(&list, message, kind, render.lang);
            append_regular_body(&list, &render);
        }
    } else {
        append_regular_body(&list, &render);
    }
    list
}

fn append_regular_body(list: &gtk::Box, render: &BodyRender<'_>) {
    if render.state.loading && render.state.events.is_empty() {
        list.append(&cards::message(
            translate(render.lang, "loading_google_calendar"),
            Some(translate(render.lang, "window_ready_updates")),
            true,
        ));
    } else if render.state.loading {
        let context = timeline_context(render);
        append_events(list, render.visible_events.clone(), context);
    } else if render.visible_events.is_empty() {
        timeline::append_empty_now_reference(
            list,
            render.selected_day,
            render.timezone,
            render.lang,
            render.view,
        );
        let detail = empty_detail(render.selected_day, render.view, render.lang);
        list.append(&cards::message(
            empty_title(render.view, render.lang),
            Some(&detail),
            false,
        ));
        append_empty_action(list, render.lang, render.sender.clone());
    } else {
        let context = timeline_context(render);
        append_events(list, render.visible_events.clone(), context);
    }
}

fn append_error_state(list: &gtk::Box, error: &str, render: &BodyRender<'_>) {
    if render.visible_events.is_empty() {
        list.append(&cards::message(
            translate(render.lang, "refresh_failed"),
            Some(error),
            false,
        ));
        return;
    }

    append_status_banner(list, error, status::StateMessageKind::Error, render.lang);
    let context = timeline_context(render);
    append_events(list, render.visible_events.clone(), context);
}

fn append_empty_action(list: &gtk::Box, lang: Language, sender: ComponentSender<AgendaApp>) {
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actions.set_halign(gtk::Align::Start);
    let add = classed_button(translate(lang, "add_event"), &["action-button"]);
    add.add_css_class("primary-action");
    add.connect_clicked(move |_| sender.input(AgendaMsg::ShowAddEvent));
    actions.append(&add);
    list.append(&actions);
}

fn append_status_banner(
    list: &gtk::Box,
    message: &str,
    kind: status::StateMessageKind,
    lang: Language,
) {
    let banner = gtk::Box::new(gtk::Orientation::Vertical, 4);
    banner.add_css_class("status-banner");
    match kind {
        status::StateMessageKind::Error => {
            banner.add_css_class("error");
            banner.append(&label(
                translate(lang, "refresh_failed"),
                &["status-banner-title"],
                0.0,
                false,
            ));
            banner.append(&label(message, &["muted"], 0.0, true));
        }
        status::StateMessageKind::Info => {
            banner.add_css_class("info");
            banner.append(&label(message, &["status-banner-title"], 0.0, true));
        }
        status::StateMessageKind::Success => {
            banner.add_css_class("success");
            banner.append(&label(message, &["status-banner-title"], 0.0, true));
        }
    }
    list.append(&banner);
}

fn append_events(list: &gtk::Box, events: Vec<&Event>, context: timeline::TimelineContext<'_>) {
    timeline::append_events(list, events, context);
}

fn timeline_context<'a>(render: &BodyRender<'a>) -> timeline::TimelineContext<'a> {
    timeline::TimelineContext {
        selected_day: render.selected_day,
        timezone: render.timezone,
        lang: render.lang,
        view: render.view,
        sender: render.sender.clone(),
    }
}

fn events_for_view(model: &AgendaApp) -> Vec<&Event> {
    let timezone = model.query.timezone.as_deref();
    match model.agenda_view {
        AgendaViewMode::Now => upcoming_events(&model.state.events, timezone)
            .into_iter()
            .take(5)
            .collect(),
        AgendaViewMode::Upcoming => upcoming_events(&model.state.events, timezone),
        AgendaViewMode::Day => {
            let day = model
                .selected_day
                .unwrap_or_else(|| now_parts_for_timezone(timezone).0);
            day_events(&model.state.events, day, timezone)
        }
        AgendaViewMode::Month => month_events(&model.state.events, timezone),
    }
}

fn month_events<'a>(events: &'a [Event], timezone: Option<&str>) -> Vec<&'a Event> {
    let mut events: Vec<_> = events.iter().collect();
    events.sort_by_key(|event| event_start_key(event, timezone));
    events
}

fn upcoming_events<'a>(events: &'a [Event], timezone: Option<&str>) -> Vec<&'a Event> {
    let mut events: Vec<_> = events
        .iter()
        .filter(|event| event_has_not_ended(event, timezone))
        .collect();
    events.sort_by_key(|event| event_start_key(event, timezone));
    events
}

fn day_events<'a>(events: &'a [Event], day: NaiveDate, timezone: Option<&str>) -> Vec<&'a Event> {
    let mut events: Vec<_> = events
        .iter()
        .filter(|event| event_date_for_timezone(event, timezone) == Some(day))
        .collect();
    events.sort_by_key(|event| event_start_key(event, timezone));
    events
}

fn event_has_not_ended(event: &Event, timezone: Option<&str>) -> bool {
    let (today, now) = now_parts_for_timezone(timezone);
    if let Some((end_date, end_time)) = parse_event_start_for_timezone(&event.end, timezone) {
        return match end_time {
            Some(end_time) => (end_date, end_time) >= (today, now),
            None => end_date > today,
        };
    }

    let Some((start_date, start_time)) = parse_event_start_for_timezone(&event.start, timezone)
    else {
        return true;
    };
    match start_time {
        Some(start_time) => (start_date, start_time) >= (today, now),
        None => start_date >= today,
    }
}

fn event_start_key(event: &Event, timezone: Option<&str>) -> (NaiveDate, NaiveTime) {
    parse_event_start_for_timezone(&event.start, timezone)
        .map(|(date, time)| (date, time.unwrap_or_else(midnight)))
        .unwrap_or((NaiveDate::MAX, midnight()))
}

fn midnight() -> NaiveTime {
    NaiveTime::from_hms_opt(0, 0, 0).expect("midnight is valid")
}

fn event_summary(event: &Event, lang: Language) -> &str {
    if event.summary.trim().is_empty() {
        translate(lang, "untitled_event")
    } else {
        event.summary.trim()
    }
}

fn event_count_label(count: usize, lang: Language) -> String {
    if lang == Language::Chinese {
        return format!("{}{}", count, translate(lang, "events"));
    }

    let key = if count == 1 { "event" } else { "events" };
    format!("{} {}", count, translate(lang, key))
}

fn empty_title(view: AgendaViewMode, lang: Language) -> &'static str {
    match view {
        AgendaViewMode::Day => translate(lang, "no_events_day"),
        _ => translate(lang, "no_upcoming_events"),
    }
}

fn empty_detail(selected_day: Option<NaiveDate>, view: AgendaViewMode, lang: Language) -> String {
    match view {
        AgendaViewMode::Day => selected_day
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
            .unwrap_or_else(|| translate(lang, "no_loaded_events_view").to_string()),
        AgendaViewMode::Month => translate(lang, "no_loaded_events_view").to_string(),
        _ => translate(lang, "no_upcoming_events_detail").to_string(),
    }
}

fn range_text(model: &AgendaApp, lang: Language) -> String {
    match model.agenda_view {
        AgendaViewMode::Day => model
            .selected_day
            .map(|day| format_short_day(day, lang))
            .unwrap_or_else(|| format_now_date_for_timezone(model.query.timezone.as_deref(), lang)),
        AgendaViewMode::Month => status::range(model.state.range, lang),
        _ => translate(lang, "coming_up").to_string(),
    }
}

fn format_short_day(day: NaiveDate, lang: Language) -> String {
    if lang == Language::Chinese {
        format!("{}月{}日", day.month(), day.day())
    } else {
        day.format("%a %b %-d").to_string()
    }
}

fn view_label(view: AgendaViewMode, lang: Language) -> &'static str {
    match view {
        AgendaViewMode::Now => translate(lang, "view_now"),
        AgendaViewMode::Upcoming => translate(lang, "view_upcoming"),
        AgendaViewMode::Day => translate(lang, "view_day"),
        AgendaViewMode::Month => translate(lang, "view_month"),
    }
}

fn view_title(view: AgendaViewMode, lang: Language) -> &'static str {
    match view {
        AgendaViewMode::Now => translate(lang, "coming_up"),
        AgendaViewMode::Upcoming => translate(lang, "upcoming"),
        AgendaViewMode::Day => translate(lang, "view_day"),
        AgendaViewMode::Month => translate(lang, "view_month"),
    }
}
