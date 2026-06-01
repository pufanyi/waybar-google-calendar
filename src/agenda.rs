mod auth_prompt;
mod controller;
mod settings;
mod view;

use crate::calendar::date::{today_for_timezone, visible_month_range};
use crate::calendar::model::{AgendaQuery, AgendaResult, AgendaState, EventKey, EventMutation};
use crate::calendar::view::CalendarViewMode;
use crate::i18n::translate;
use crate::storage::cache::{cache_is_fresh, read_cache};
use crate::storage::settings::{Language, UserSettings, WeekStart, read_settings};
use crate::ui::{icon_button, label};
use adw::prelude::*;
use chrono::{Datelike, Local, NaiveDate};
use gtk::{gdk, glib};
use relm4::{Component, ComponentParts, ComponentSender};

const AGENDA_WINDOW_WIDTH: i32 = 1060;
const AGENDA_WINDOW_HEIGHT: i32 = 620;
const SETTINGS_WINDOW_WIDTH: i32 = 1080;
const SETTINGS_WINDOW_HEIGHT: i32 = 680;
const AGENDA_WINDOW_MIN_WIDTH: i32 = 780;
const AGENDA_WINDOW_MIN_HEIGHT: i32 = 480;
const SETTINGS_WINDOW_MIN_WIDTH: i32 = 820;
const SETTINGS_WINDOW_MIN_HEIGHT: i32 = 520;

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
    agenda_view: AgendaViewMode,
    selected_day: Option<NaiveDate>,
    authenticating: bool,
    user_settings: UserSettings,
    settings_form: UserSettings,
    settings_msg: Option<String>,
    settings_open: bool,
    event_editor: AgendaEditor,
    event_editor_msg: Option<String>,
    mutating_event: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgendaViewMode {
    Now,
    Upcoming,
    Day,
    Month,
}

impl AgendaViewMode {
    pub const ALL: [Self; 4] = [Self::Now, Self::Upcoming, Self::Day, Self::Month];
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgendaEditor {
    None,
    Add,
    Detail(EventKey),
    Edit(EventKey),
    ConfirmDelete(EventKey),
}

impl AgendaEditor {
    fn is_form(&self) -> bool {
        matches!(self, Self::Add | Self::Edit(_))
    }
}

#[derive(Debug, Clone)]
pub struct SettingsChanges {
    pub calendar: String,
    pub timezone: String,
    pub theme_path: String,
    pub language: Language,
    pub week_start: WeekStart,
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
    SetAgendaView(AgendaViewMode),
    ShowAddEvent,
    ShowEventDetail(EventKey),
    EditEvent(EventKey),
    ConfirmDelete(EventKey),
    CloseEventEditor,
    CreateEvent(EventMutation),
    UpdateEvent(EventKey, EventMutation),
    DeleteEvent(EventKey),
    OpenEventLink(String),
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
    OpenSettings,
    CloseSettings,
    Close,
    EscapePressed,
    Tick,
    ApplySettings(SettingsChanges),
    SaveSettings(SettingsChanges),
    Logout,
}

#[derive(Debug)]
pub enum AgendaCommandOutput {
    Events(AgendaResult),
    Auth(Result<(), String>),
    EventMutation(Result<&'static str, String>),
}

#[allow(dead_code)]
pub struct AgendaWidgets {
    content: gtk::Box,
    status_label: gtk::Label,
    refresh: gtk::Button,
    settings_button: gtk::Button,
    title_label: gtk::Label,
    settings: settings::SettingsWidgets,
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
            .default_width(AGENDA_WINDOW_WIDTH)
            .default_height(AGENDA_WINDOW_HEIGHT)
            .resizable(false)
            .build();
        root.set_decorated(false);
        root.set_size_request(AGENDA_WINDOW_MIN_WIDTH, AGENDA_WINDOW_MIN_HEIGHT);
        root
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let root_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
        root_box.add_css_class("panel");

        let (user_settings, settings_msg) = match read_settings() {
            Ok(settings) => (settings, None),
            Err(error) => (UserSettings::default(), Some(error)),
        };
        let lang = user_settings.language.unwrap_or_default();

        let topbar = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        topbar.add_css_class("topbar");

        let title_label = label(translate(lang, "google_calendar"), &["title"], 0.0, false);
        topbar.append(&title_label);

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
            translate(lang, "refresh"),
        );
        {
            let sender = sender.clone();
            refresh.connect_clicked(move |_| sender.input(AgendaMsg::Refresh));
        }
        topbar.append(&refresh);

        let settings_widgets = settings::build(&user_settings, lang, sender.clone());

        let settings_button = icon_button(
            "emblem-system-symbolic",
            &["action-button", "icon-button"],
            translate(lang, "settings"),
        );
        {
            let sender = sender.clone();
            settings_button.connect_clicked(move |_| sender.input(AgendaMsg::OpenSettings));
        }
        topbar.append(&settings_button);

        let close = icon_button(
            "window-close-symbolic",
            &["close-button", "icon-button"],
            translate(lang, "close"),
        );
        {
            let sender = sender.clone();
            close.connect_clicked(move |_| sender.input(AgendaMsg::Close));
        }
        topbar.append(&close);
        root_box.append(&topbar);

        let content = gtk::Box::new(gtk::Orientation::Horizontal, 14);
        root_box.append(&content);
        root.set_content(Some(&root_box));

        let key_controller = gtk::EventControllerKey::new();
        {
            let sender = sender.clone();
            key_controller.connect_key_pressed(move |_, key, _, _| {
                if key == gdk::Key::Escape {
                    sender.input(AgendaMsg::EscapePressed);
                    gtk::glib::Propagation::Stop
                } else {
                    gtk::glib::Propagation::Proceed
                }
            });
        }
        root.add_controller(key_controller);

        let today = today_for_timezone(init.query.timezone.as_deref());
        let initial_range = visible_month_range(
            today.year(),
            today.month(),
            user_settings.week_start.unwrap_or_default(),
        );
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
            agenda_view: AgendaViewMode::Now,
            selected_day: None,
            authenticating: false,
            user_settings: user_settings.clone(),
            settings_form: user_settings,
            settings_msg,
            settings_open: false,
            event_editor: AgendaEditor::None,
            event_editor_msg: None,
            mutating_event: false,
        };

        let mut widgets = AgendaWidgets {
            content,
            status_label,
            refresh,
            settings_button,
            title_label,
            settings: settings_widgets,
        };
        view::render(&model, &mut widgets, sender.clone());

        let should_fetch = initial_cache
            .as_ref()
            .map(|cache| !cache_is_fresh(cache.fetched_at))
            .unwrap_or(true);
        if should_fetch {
            sender.input(AgendaMsg::LoadVisibleRange);
        }

        {
            let sender = sender.clone();
            glib::timeout_add_seconds_local(30, move || {
                sender.input(AgendaMsg::Tick);
                glib::ControlFlow::Continue
            });
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            AgendaMsg::Close => root.close(),
            AgendaMsg::EscapePressed if self.settings_open => {
                self.handle_input(AgendaMsg::CloseSettings, sender);
            }
            AgendaMsg::EscapePressed => root.close(),
            message => self.handle_input(message, sender),
        }
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
        view::render(self, widgets, sender.clone());

        let lang = self.user_settings.language.unwrap_or_default();

        // Update topbar texts dynamically
        widgets
            .title_label
            .set_text(translate(lang, "google_calendar"));
        widgets
            .refresh
            .set_tooltip_text(Some(translate(lang, "refresh")));
        widgets
            .settings_button
            .set_tooltip_text(Some(translate(lang, "settings")));
        widgets.settings_button.set_sensitive(!self.settings_open);

        settings::update_text(&widgets.settings, lang);
        settings::update_state(
            &widgets.settings,
            lang,
            self.authenticating,
            self.settings_msg.as_deref(),
        );
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        let was_settings_open = self.settings_open;
        let should_sync_settings = matches!(message, AgendaMsg::OpenSettings);
        let should_close = matches!(message, AgendaMsg::Close)
            || matches!(message, AgendaMsg::EscapePressed) && !self.settings_open;
        let should_skip_view = matches!(message, AgendaMsg::Tick) && self.event_editor.is_form();

        self.update(message, sender.clone(), root);
        if should_close {
            return;
        }

        if was_settings_open != self.settings_open {
            set_window_size(root, self.settings_open);
        }
        if should_skip_view {
            return;
        }

        self.update_view(widgets, sender);
        if should_sync_settings {
            settings::populate_form(&widgets.settings, &self.settings_form);
        }
    }
}

impl AgendaApp {
    pub(super) fn language(&self) -> Language {
        self.user_settings.language.unwrap_or_default()
    }

    pub(super) fn week_start(&self) -> WeekStart {
        self.user_settings.week_start.unwrap_or_default()
    }
}

fn set_window_size(root: &adw::ApplicationWindow, settings_open: bool) {
    let (width, height, min_width, min_height) = if settings_open {
        (
            SETTINGS_WINDOW_WIDTH,
            SETTINGS_WINDOW_HEIGHT,
            SETTINGS_WINDOW_MIN_WIDTH,
            SETTINGS_WINDOW_MIN_HEIGHT,
        )
    } else {
        (
            AGENDA_WINDOW_WIDTH,
            AGENDA_WINDOW_HEIGHT,
            AGENDA_WINDOW_MIN_WIDTH,
            AGENDA_WINDOW_MIN_HEIGHT,
        )
    };
    root.set_default_size(width, height);
    root.set_size_request(min_width, min_height);
}
