mod auth_prompt;
mod controller;
mod view;

use crate::calendar::date::visible_month_range;
use crate::calendar::model::{AgendaQuery, AgendaResult, AgendaState};
use crate::calendar::view::CalendarViewMode;
use crate::storage::cache::{cache_is_fresh, read_cache};
use crate::ui::{add_escape_to_close, icon_button, label};
use adw::prelude::*;
use chrono::{Datelike, Local, NaiveDate};
use relm4::{Component, ComponentParts, ComponentSender};

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
    calendar_view: CalendarViewMode,
    selected_day: Option<NaiveDate>,
    authenticating: bool,
}

#[derive(Debug)]
pub enum AgendaMsg {
    Refresh,
    LoadVisibleRange,
    PreviousCalendarPage,
    NextCalendarPage,
    CycleCalendarView,
    SelectMonth(u32),
    SelectYear(i32),
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
    OpenSetupGuide,
    OpenGoogleCloud,
    OpenCalendarApi,
}

#[derive(Debug)]
pub enum AgendaCommandOutput {
    Events(AgendaResult),
    Auth(Result<(), String>),
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

        let refresh = icon_button(
            "view-refresh-symbolic",
            &["action-button", "icon-button"],
            "Refresh",
        );
        {
            let sender = sender.clone();
            refresh.connect_clicked(move |_| sender.input(AgendaMsg::Refresh));
        }
        topbar.append(&refresh);

        let close = icon_button(
            "window-close-symbolic",
            &["close-button", "icon-button"],
            "Close",
        );
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
            calendar_view: CalendarViewMode::Days,
            selected_day: None,
            authenticating: false,
        };

        let mut widgets = AgendaWidgets {
            content,
            status_label,
            refresh,
        };
        view::render(&model, &mut widgets, sender.clone());

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
        self.handle_input(message, sender);
    }

    fn update_cmd(
        &mut self,
        output: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        self.handle_command(output, sender);
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
        view::render(self, widgets, sender);
    }
}
