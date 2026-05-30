use crate::cache::{cache_is_fresh, read_cache, write_cache};
use crate::date::{
    event_date, event_days, format_day_label, format_time_label, month_dates, month_name,
    parse_event_start, visible_month_range,
};
use crate::google::{self, fetch_events};
use crate::model::{AgendaQuery, AgendaResult, AgendaState, DateRange, Event};
use crate::paths;
use crate::ui::{add_escape_to_close, classed_button, clear_box, label};
use adw::prelude::*;
use chrono::{DateTime, Datelike, Duration as ChronoDuration, Local, NaiveDate};
use gtk::gio;
use relm4::{Component, ComponentParts, ComponentSender};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const GOOGLE_CLOUD_CREDENTIALS_URL: &str = "https://console.cloud.google.com/auth/clients";
const GOOGLE_CALENDAR_API_URL: &str =
    "https://console.cloud.google.com/apis/library/calendar-json.googleapis.com";
const AUTH_WIZARD_PAGE_COUNT: usize = 6;
const AUTH_WIZARD_LAST_PAGE: usize = AUTH_WIZARD_PAGE_COUNT - 1;

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
    authenticating: bool,
    auth_page: usize,
}

#[derive(Debug)]
pub enum AgendaMsg {
    Refresh,
    LoadVisibleRange,
    PreviousMonth,
    NextMonth,
    Today,
    ClearSelection,
    SelectDay(NaiveDate),
    StartAuth,
    SaveAndStartAuth {
        client_id: String,
        client_secret: String,
    },
    OpenConfigDir,
    OpenTokenDir,
    OpenGoogleCloud,
    OpenCalendarApi,
    PreviousAuthPage,
    NextAuthPage,
}

#[derive(Debug)]
pub enum AgendaCommandOutput {
    Events(AgendaResult),
    Auth(Result<(), String>),
}

impl AgendaApp {
    fn current_range(&self) -> DateRange {
        visible_month_range(self.calendar_year, self.calendar_month)
    }

    fn load_visible_range(&mut self, sender: ComponentSender<Self>, force: bool) {
        let range = self.current_range();
        self.state.range = range;

        if let Some(cache) = read_cache(&self.query, range) {
            self.state.events = cache.events;
            self.state.error = None;
            self.state.fetched_at = Some(cache.fetched_at);
            self.state.cached = true;
            if !force && cache_is_fresh(cache.fetched_at) {
                self.state.loading = false;
                self.state.loading_range = None;
                return;
            }
        } else {
            self.state.events.clear();
            self.state.fetched_at = None;
            self.state.cached = false;
        }

        if self.state.loading_range == Some(range) {
            return;
        }

        self.state.loading = true;
        self.state.loading_range = Some(range);
        self.state.error = None;

        let query = self.query.clone();
        sender.spawn_oneshot_command(move || {
            AgendaCommandOutput::Events(match fetch_events(&query, range) {
                Ok(events) => AgendaResult {
                    range,
                    events,
                    error: None,
                },
                Err(error) => AgendaResult {
                    range,
                    events: Vec::new(),
                    error: Some(error),
                },
            })
        });
    }
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
    type CommandOutput = AgendaCommandOutput;
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
        let initial_range = visible_month_range(today.year(), today.month());
        let initial_cache = read_cache(&init.query, initial_range);
        let state = match &initial_cache {
            Some(cache) => AgendaState {
                range: initial_range,
                loading_range: None,
                events: cache.events.clone(),
                error: None,
                fetched_at: Some(cache.fetched_at),
                loading: false,
                cached: true,
            },
            None => AgendaState {
                range: initial_range,
                loading_range: None,
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
            authenticating: false,
            auth_page: initial_auth_wizard_page(),
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
            sender.input(AgendaMsg::LoadVisibleRange);
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            AgendaMsg::Refresh => {
                self.load_visible_range(sender, true);
            }
            AgendaMsg::LoadVisibleRange => {
                self.load_visible_range(sender, false);
            }
            AgendaMsg::PreviousMonth => {
                if self.calendar_month == 1 {
                    self.calendar_month = 12;
                    self.calendar_year -= 1;
                } else {
                    self.calendar_month -= 1;
                }
                self.selected_day = None;
                self.load_visible_range(sender, false);
            }
            AgendaMsg::NextMonth => {
                if self.calendar_month == 12 {
                    self.calendar_month = 1;
                    self.calendar_year += 1;
                } else {
                    self.calendar_month += 1;
                }
                self.selected_day = None;
                self.load_visible_range(sender, false);
            }
            AgendaMsg::Today => {
                let today = Local::now().date_naive();
                self.calendar_year = today.year();
                self.calendar_month = today.month();
                self.selected_day = Some(today);
                self.load_visible_range(sender, false);
            }
            AgendaMsg::ClearSelection => {
                self.selected_day = None;
            }
            AgendaMsg::SelectDay(day) => {
                let previous_range = self.current_range();
                self.calendar_year = day.year();
                self.calendar_month = day.month();
                self.selected_day = Some(day);
                if self.current_range() != previous_range {
                    self.load_visible_range(sender, false);
                }
            }
            AgendaMsg::StartAuth => {
                if self.authenticating {
                    return;
                }
                self.auth_page = AUTH_WIZARD_LAST_PAGE;
                self.authenticating = true;
                self.state.error = Some("Opening browser for Google OAuth...".to_string());
                sender.spawn_oneshot_command(|| AgendaCommandOutput::Auth(google::auth_calendar()));
            }
            AgendaMsg::SaveAndStartAuth {
                client_id,
                client_secret,
            } => {
                if self.authenticating {
                    return;
                }
                match google::save_client_secret(&client_id, &client_secret) {
                    Ok(path) => {
                        self.auth_page = AUTH_WIZARD_LAST_PAGE;
                        self.state.error = Some(format!(
                            "OAuth client saved to {}. Opening browser for Google OAuth...",
                            path.display()
                        ));
                        sender.input(AgendaMsg::StartAuth);
                    }
                    Err(error) => {
                        self.state.error = Some(error);
                    }
                }
            }
            AgendaMsg::OpenConfigDir => {
                self.state.error = Some(
                    open_dir(&paths::config_dir())
                        .map(|_| "Config folder opened.".to_string())
                        .unwrap_or_else(|error| error),
                );
            }
            AgendaMsg::OpenTokenDir => {
                self.state.error = Some(
                    open_dir(&paths::data_dir())
                        .map(|_| "Token folder opened.".to_string())
                        .unwrap_or_else(|error| error),
                );
            }
            AgendaMsg::OpenGoogleCloud => {
                self.state.error = Some(
                    google::open_external_uri(GOOGLE_CLOUD_CREDENTIALS_URL)
                        .map(|_| "Google Cloud opened in your browser.".to_string())
                        .unwrap_or_else(|error| error),
                );
            }
            AgendaMsg::OpenCalendarApi => {
                self.state.error = Some(
                    google::open_external_uri(GOOGLE_CALENDAR_API_URL)
                        .map(|_| "Google Calendar API page opened in your browser.".to_string())
                        .unwrap_or_else(|error| error),
                );
            }
            AgendaMsg::PreviousAuthPage => {
                self.auth_page = self.auth_page.saturating_sub(1);
            }
            AgendaMsg::NextAuthPage => {
                self.auth_page = (self.auth_page + 1).min(AUTH_WIZARD_LAST_PAGE);
            }
        }
    }

    fn update_cmd(
        &mut self,
        output: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match output {
            AgendaCommandOutput::Auth(Ok(())) => {
                self.authenticating = false;
                self.state.error =
                    Some("Google Calendar authenticated. Loading events...".to_string());
                self.load_visible_range(sender, true);
            }
            AgendaCommandOutput::Auth(Err(error)) => {
                self.authenticating = false;
                self.state.error = Some(error);
            }
            AgendaCommandOutput::Events(result) => {
                if result.range != self.current_range() {
                    return;
                }

                self.state.loading = false;
                self.state.loading_range = None;
                self.state.range = result.range;
                if let Some(error) = result.error {
                    self.state.error = Some(error);
                    self.state.cached = !self.state.events.is_empty();
                } else {
                    let fetched_at = Local::now();
                    write_cache(&self.query, result.range, &result.events, fetched_at);
                    self.state.events = result.events;
                    self.state.error = None;
                    self.state.fetched_at = Some(fetched_at);
                    self.state.cached = false;
                }
            }
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
    let focus_auth_prompt = should_focus_auth_prompt(&model.state, model.authenticating);
    let calendar_event_days = if focus_auth_prompt {
        BTreeSet::new()
    } else {
        event_days(&model.state.events)
    };
    widgets.content.append(&build_event_calendar(
        model,
        &calendar_event_days,
        sender.clone(),
    ));
    widgets.content.append(&build_agenda_list(
        &model.query,
        &model.state,
        model.selected_day,
        model.authenticating,
        model.auth_page,
        sender,
    ));

    widgets
        .refresh
        .set_sensitive(!model.state.loading && !model.authenticating);
    widgets.refresh.set_label(if model.authenticating {
        "Authenticating"
    } else if model.state.loading {
        "Refreshing"
    } else {
        "Refresh"
    });
    let status = if model.authenticating {
        "Authenticating".to_string()
    } else {
        agenda_status(&model.state)
    };
    widgets.status_label.set_text(&status);
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
    authenticating: bool,
    auth_page: usize,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    let right = gtk::Box::new(gtk::Orientation::Vertical, 10);
    right.set_hexpand(true);
    let focus_auth_prompt = should_focus_auth_prompt(state, authenticating);
    let visible_events = if focus_auth_prompt {
        Vec::new()
    } else {
        visible_events(&state.events, selected_day)
    };

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    header.append(&label("Agenda", &["agenda-header"], 0.0, false));
    let range_text = selected_day
        .map(|day| day.format("%a %b %-d").to_string())
        .unwrap_or_else(|| range_label(state.range));
    header.append(&label(&range_text, &["subtle"], 0.0, false));
    if let Some(calendar) = &query.calendar {
        header.append(&label(calendar, &["pill"], 0.0, false));
    }
    let status_text = if focus_auth_prompt {
        "Action required".to_string()
    } else {
        format!("{} events", visible_events.len())
    };
    header.append(&label(&status_text, &["accent"], 0.0, false));
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
        if focus_auth_prompt {
            list.append(&auth_prompt_card(error, authenticating, auth_page, sender));
        } else if state.events.is_empty() {
            list.append(&message_card("Refresh failed", Some(error), false));
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
            .unwrap_or_else(|| "No loaded events for this calendar view.".to_string());
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

fn auth_prompt_card(
    error: &str,
    authenticating: bool,
    auth_page: usize,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    let secret_present = paths::client_secret_file().exists();
    let token_present = paths::oauth_token_file().exists();
    let page = auth_page.min(AUTH_WIZARD_LAST_PAGE);

    let card = gtk::Box::new(gtk::Orientation::Vertical, 12);
    card.add_css_class("empty-card");
    card.add_css_class("auth-prompt");

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    header.add_css_class("auth-header");
    header.append(&label(
        "Connect Google Calendar",
        &["event-title"],
        0.0,
        false,
    ));
    let header_spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    header_spacer.set_hexpand(true);
    header.append(&header_spacer);
    header.append(&auth_status_badge(
        if token_present {
            "Connected"
        } else if secret_present {
            "Authorize"
        } else {
            "Setup"
        },
        if token_present {
            "success"
        } else if secret_present {
            "info"
        } else {
            "warning"
        },
    ));
    card.append(&header);
    card.append(&auth_current_status(error, secret_present, token_present));
    card.append(&auth_wizard_progress(page));
    card.append(&auth_wizard_page(
        page,
        secret_present,
        token_present,
        authenticating,
        sender.clone(),
    ));
    card.append(&auth_wizard_navigation(
        page,
        authenticating,
        sender.clone(),
    ));
    card.append(&auth_utility_actions(authenticating, sender));
    card
}

fn initial_auth_wizard_page() -> usize {
    if paths::client_secret_file().exists() {
        AUTH_WIZARD_LAST_PAGE
    } else {
        0
    }
}

fn auth_current_status(error: &str, secret_present: bool, token_present: bool) -> gtk::Box {
    let status = gtk::Box::new(gtk::Orientation::Vertical, 3);
    status.add_css_class("auth-current-status");
    status.append(&label("Current status", &["field-label"], 0.0, false));
    status.append(&label(
        &friendly_auth_status(error, secret_present, token_present),
        &["muted"],
        0.0,
        true,
    ));
    status
}

fn friendly_auth_status(error: &str, secret_present: bool, token_present: bool) -> String {
    let error = error.trim();
    let lower = error.to_ascii_lowercase();
    if lower.contains("opened")
        || lower.contains("opening browser")
        || lower.contains("saved to")
        || lower.contains("authenticated")
    {
        return error.to_string();
    }
    if lower.contains("missing google oauth client secret") {
        return "No OAuth client is saved yet. Follow the pages below; this is a one-time setup."
            .to_string();
    }
    if lower.contains("not authenticated") {
        return "OAuth client is saved. Finish browser authorization on the last page.".to_string();
    }
    if !error.is_empty() {
        return error.to_string();
    }
    if !secret_present {
        return "No OAuth client is saved yet. Follow the pages below; this is a one-time setup."
            .to_string();
    }
    if !token_present {
        return "OAuth client is saved. Finish browser authorization on the last page.".to_string();
    }
    "Google Calendar credentials are saved. Refresh or re-authenticate if events still do not load."
        .to_string()
}

fn auth_wizard_progress(current_page: usize) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 7);
    row.add_css_class("auth-wizard-progress");
    for index in 0..AUTH_WIZARD_PAGE_COUNT {
        let dot = label(&(index + 1).to_string(), &["auth-progress-dot"], 0.5, false);
        if index == current_page {
            dot.add_css_class("active");
        } else if index < current_page {
            dot.add_css_class("completed");
        }
        row.append(&dot);
    }
    row
}

fn auth_wizard_page(
    page: usize,
    secret_present: bool,
    token_present: bool,
    authenticating: bool,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    match page {
        0 => auth_intro_page(),
        1 => auth_open_cloud_page(sender),
        2 => auth_enable_api_page(sender),
        3 => auth_consent_screen_page(sender),
        4 => auth_create_client_page(sender),
        _ => auth_finish_page(secret_present, token_present, authenticating, sender),
    }
}

fn auth_intro_page() -> gtk::Box {
    let (page, body) = auth_page_shell(
        "Before you start",
        "You will create a private Google OAuth client, then let this app read your calendar.",
    );
    body.append(&instruction_list(&[
        "Keep this calendar window open while the browser opens Google Cloud.",
        "Use the Google account whose calendar you want to show in Waybar.",
        "This app asks for read-only Calendar access and saves the credentials on this computer.",
        "If a Google page looks different, choose the closest option with the same name.",
    ]));
    page
}

fn auth_open_cloud_page(sender: ComponentSender<AgendaApp>) -> gtk::Box {
    let (page, body) = auth_page_shell(
        "Open Google Cloud",
        "First open the Google Auth Platform page and choose a project for this local setup.",
    );
    body.append(&instruction_list(&[
        "Click Google Cloud below. Your browser should open a Google Cloud page.",
        "Sign in if Google asks.",
        "If Google asks for a project, create one named Waybar Calendar or choose an existing personal project.",
        "If you see a Get started button for Google Auth Platform, click it.",
    ]));
    let actions = auth_action_row();
    let open_cloud = classed_button("Google Cloud", &["action-button"]);
    {
        let sender = sender.clone();
        open_cloud.connect_clicked(move |_| sender.input(AgendaMsg::OpenGoogleCloud));
    }
    actions.append(&open_cloud);
    body.append(&actions);
    page
}

fn auth_enable_api_page(sender: ComponentSender<AgendaApp>) -> gtk::Box {
    let (page, body) = auth_page_shell(
        "Enable Calendar API",
        "Google will not allow calendar access until the Calendar API is enabled for the project.",
    );
    body.append(&instruction_list(&[
        "Click Open Calendar API below.",
        "Check the project selector near the top of the page. It should be the project you chose for this app.",
        "Click Enable. If the button says Manage, the API is already enabled.",
        "When it finishes, return here and click Next.",
    ]));
    let actions = auth_action_row();
    let open_api = classed_button("Open Calendar API", &["action-button"]);
    {
        let sender = sender.clone();
        open_api.connect_clicked(move |_| sender.input(AgendaMsg::OpenCalendarApi));
    }
    actions.append(&open_api);
    body.append(&actions);
    page
}

fn auth_consent_screen_page(sender: ComponentSender<AgendaApp>) -> gtk::Box {
    let (page, body) = auth_page_shell(
        "Set up app details",
        "Google uses these details on the permission screen that appears when you sign in.",
    );
    body.append(&instruction_list(&[
        "If Google asks for app information, use Waybar Google Calendar as the app name.",
        "For User support email and Developer contact email, choose your own Google email.",
        "Choose External for a personal Gmail account. Choose Internal only for a Google Workspace organization.",
        "If there is a Test users page, add the same Google email you will use for Calendar.",
        "Save or continue until Google lets you create OAuth clients.",
    ]));
    let actions = auth_action_row();
    let open_cloud = classed_button("Google Cloud", &["action-button"]);
    {
        let sender = sender.clone();
        open_cloud.connect_clicked(move |_| sender.input(AgendaMsg::OpenGoogleCloud));
    }
    actions.append(&open_cloud);
    body.append(&actions);
    page
}

fn auth_create_client_page(sender: ComponentSender<AgendaApp>) -> gtk::Box {
    let (page, body) = auth_page_shell(
        "Create a Desktop client",
        "This creates the Client ID and Client Secret that this app needs.",
    );
    body.append(&instruction_list(&[
        "Open the Google Auth Platform Clients page.",
        "Click Create client.",
        "For Application type, choose Desktop app.",
        "For Name, enter Waybar Google Calendar, then click Create.",
        "Copy both the Client ID and Client Secret before closing the result window.",
    ]));
    let note = label(
        "Google may only show the full client secret when it is created. Copy it immediately.",
        &["muted", "auth-note"],
        0.0,
        true,
    );
    body.append(&note);
    let actions = auth_action_row();
    let open_cloud = classed_button("Google Cloud", &["action-button"]);
    {
        let sender = sender.clone();
        open_cloud.connect_clicked(move |_| sender.input(AgendaMsg::OpenGoogleCloud));
    }
    actions.append(&open_cloud);
    body.append(&actions);
    page
}

fn auth_finish_page(
    secret_present: bool,
    token_present: bool,
    authenticating: bool,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    let (page, body) = auth_page_shell(
        "Paste and authorize",
        "Finish by saving the OAuth client, then approve read-only Calendar access in your browser.",
    );

    if secret_present {
        body.append(&auth_path_summary(
            "OAuth client saved at",
            &paths::client_secret_file(),
        ));
        let replace = gtk::Expander::new(Some("Replace OAuth client"));
        replace.add_css_class("auth-expander");
        replace.set_child(Some(&auth_credentials_form(
            authenticating,
            "Replace & Authenticate",
            sender.clone(),
        )));
        body.append(&replace);
    } else {
        body.append(&instruction_list(&[
            "Paste the Client ID exactly as Google shows it.",
            "Paste the Client Secret exactly as Google shows it.",
            "Click Save & Authenticate. Your browser will open the Google permission screen.",
        ]));
        body.append(&auth_credentials_form(
            authenticating,
            "Save & Authenticate",
            sender.clone(),
        ));
    }

    if token_present {
        body.append(&auth_path_summary(
            "Browser token saved at",
            &paths::oauth_token_file(),
        ));
    }

    if secret_present {
        let actions = auth_action_row();
        let start_auth = classed_button(
            if authenticating {
                "Authenticating"
            } else if token_present {
                "Re-authenticate"
            } else {
                "Start Authentication"
            },
            &["action-button"],
        );
        start_auth.set_sensitive(!authenticating);
        {
            let sender = sender.clone();
            start_auth.connect_clicked(move |_| sender.input(AgendaMsg::StartAuth));
        }
        actions.append(&start_auth);
        body.append(&actions);
    }

    page
}

fn auth_wizard_navigation(
    page: usize,
    authenticating: bool,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    let navigation = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    navigation.add_css_class("auth-wizard-navigation");

    let previous = classed_button("Previous", &["action-button"]);
    previous.set_sensitive(page > 0 && !authenticating);
    let next = classed_button("Next", &["action-button"]);
    next.set_sensitive(page < AUTH_WIZARD_LAST_PAGE && !authenticating);
    let page_label = label(
        &format!("Step {} of {}", page + 1, AUTH_WIZARD_PAGE_COUNT),
        &["muted", "auth-page-label"],
        0.5,
        false,
    );
    page_label.set_hexpand(true);

    {
        let sender = sender.clone();
        previous.connect_clicked(move |_| sender.input(AgendaMsg::PreviousAuthPage));
    }
    {
        let sender = sender.clone();
        next.connect_clicked(move |_| sender.input(AgendaMsg::NextAuthPage));
    }

    navigation.append(&previous);
    navigation.append(&page_label);
    navigation.append(&next);
    navigation
}

fn auth_page_shell(title: &str, detail: &str) -> (gtk::Box, gtk::Box) {
    let page = gtk::Box::new(gtk::Orientation::Vertical, 10);
    page.add_css_class("auth-wizard-page");
    page.append(&label(
        title,
        &["event-title", "auth-wizard-title"],
        0.0,
        false,
    ));
    page.append(&label(detail, &["muted", "auth-step-detail"], 0.0, true));

    let body = gtk::Box::new(gtk::Orientation::Vertical, 9);
    body.add_css_class("auth-wizard-body");
    page.append(&body);
    (page, body)
}

fn instruction_list(items: &[&str]) -> gtk::Box {
    let list = gtk::Box::new(gtk::Orientation::Vertical, 7);
    list.add_css_class("auth-instruction-list");
    for (index, item) in items.iter().enumerate() {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        row.add_css_class("auth-instruction-row");
        row.append(&label(
            &(index + 1).to_string(),
            &["auth-instruction-index"],
            0.5,
            false,
        ));
        let text = label(item, &["auth-instruction-text", "muted"], 0.0, true);
        text.set_hexpand(true);
        row.append(&text);
        list.append(&row);
    }
    list
}

fn auth_action_row() -> gtk::Box {
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actions.add_css_class("auth-step-actions");
    actions
}

fn auth_utility_actions(authenticating: bool, sender: ComponentSender<AgendaApp>) -> gtk::Expander {
    let expander = gtk::Expander::new(Some("Advanced"));
    expander.add_css_class("auth-expander");

    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actions.add_css_class("auth-helper-actions");

    let refresh = classed_button("Refresh Status", &["action-button"]);
    let open_config = classed_button("Config Folder", &["action-button"]);
    let open_token = classed_button("Token Folder", &["action-button"]);

    refresh.set_sensitive(!authenticating);

    {
        let sender = sender.clone();
        refresh.connect_clicked(move |_| sender.input(AgendaMsg::Refresh));
    }
    {
        let sender = sender.clone();
        open_config.connect_clicked(move |_| sender.input(AgendaMsg::OpenConfigDir));
    }
    {
        let sender = sender.clone();
        open_token.connect_clicked(move |_| sender.input(AgendaMsg::OpenTokenDir));
    }

    actions.append(&refresh);
    actions.append(&open_config);
    actions.append(&open_token);
    expander.set_child(Some(&actions));
    expander
}

fn auth_path_summary(title: &str, path: &Path) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Vertical, 3);
    row.add_css_class("auth-path-row");
    row.append(&label(title, &["field-label"], 0.0, false));
    let path_label = label(
        &path.display().to_string(),
        &["path-label", "muted"],
        0.0,
        false,
    );
    path_label.set_selectable(true);
    row.append(&path_label);
    row
}

fn auth_status_badge(text: &str, state: &str) -> gtk::Label {
    let badge = label(text, &["status-badge"], 0.5, false);
    badge.add_css_class(state);
    badge
}

fn auth_credentials_form(
    authenticating: bool,
    button_label: &str,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    let form = gtk::Box::new(gtk::Orientation::Vertical, 8);
    form.add_css_class("auth-form");

    let client_id = credential_entry("Client ID");
    let client_secret = credential_entry("Client Secret");
    client_secret.set_visibility(false);

    form.append(&field_row("Client ID", &client_id));
    form.append(&field_row("Client Secret", &client_secret));

    let save = classed_button(button_label, &["action-button"]);
    save.set_sensitive(!authenticating);
    {
        let client_id = client_id.clone();
        let client_secret = client_secret.clone();
        save.connect_clicked(move |_| {
            sender.input(AgendaMsg::SaveAndStartAuth {
                client_id: client_id.text().trim().to_string(),
                client_secret: client_secret.text().trim().to_string(),
            });
        });
    }
    form.append(&save);
    form
}

fn credential_entry(placeholder: &str) -> gtk::Entry {
    let entry = gtk::Entry::new();
    entry.add_css_class("text-entry");
    entry.set_placeholder_text(Some(placeholder));
    entry.set_hexpand(true);
    entry
}

fn field_row(title: &str, entry: &gtk::Entry) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let title = label(title, &["field-label"], 0.0, false);
    title.set_size_request(104, -1);
    row.append(&title);
    row.append(entry);
    row
}

fn open_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path)
        .map_err(|err| format!("Could not create folder {}: {err}", path.display()))?;
    let file = gio::File::for_path(path);
    let uri = file.uri();
    google::open_external_uri(uri.as_str())
}

fn needs_auth_action(error: &str) -> bool {
    let error = error.to_ascii_lowercase();
    error.contains("oauth")
        || error.contains("client secret")
        || error.contains("not authenticated")
        || error.contains("access token")
        || error.contains("invalid_grant")
        || error.contains("401")
}

fn should_show_auth_prompt(error: &str) -> bool {
    needs_auth_action(error) || auth_setup_incomplete()
}

fn should_focus_auth_prompt(state: &AgendaState, authenticating: bool) -> bool {
    state
        .error
        .as_deref()
        .map(|error| authenticating || should_show_auth_prompt(error))
        .unwrap_or(false)
}

fn auth_setup_incomplete() -> bool {
    !paths::client_secret_file().exists() || !paths::oauth_token_file().exists()
}

fn range_label(range: DateRange) -> String {
    let end = range.end_exclusive - ChronoDuration::days(1);
    if range.start.year() == end.year() {
        if range.start.month() == end.month() {
            return format!(
                "{} {}-{}",
                month_name(range.start.month()),
                range.start.day(),
                end.day()
            );
        }
        return format!(
            "{} {}-{} {}",
            range.start.format("%b"),
            range.start.day(),
            end.format("%b"),
            end.day()
        );
    }
    format!(
        "{}-{}",
        range.start.format("%b %-d %Y"),
        end.format("%b %-d %Y")
    )
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
