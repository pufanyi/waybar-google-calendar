#![allow(deprecated)]
mod auth_prompt;
mod controller;
mod view;

use crate::calendar::date::visible_month_range;
use crate::calendar::model::{AgendaQuery, AgendaResult, AgendaState};
use crate::calendar::view::CalendarViewMode;
use crate::storage::cache::{cache_is_fresh, read_cache};
use crate::storage::paths;
use crate::storage::settings::{Language, UserSettings, read_settings, translate};
use crate::ui::{add_escape_to_close, classed_button, icon_button, label};
use adw::prelude::*;
use chrono::{Datelike, Local, NaiveDate};
use gtk::gdk;
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
    user_settings: UserSettings,
    settings_msg: Option<String>,
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
    OpenSettings,
    #[allow(dead_code)]
    CloseSettings,
    SaveSettings {
        calendar: Option<String>,
        timezone: Option<String>,
        theme_path: Option<String>,
        language: Language,
    },
    Logout,
}

#[derive(Debug)]
pub enum AgendaCommandOutput {
    Events(AgendaResult),
    Auth(Result<(), String>),
}

#[allow(dead_code)]
pub struct AgendaWidgets {
    content: gtk::Box,
    status_label: gtk::Label,
    refresh: gtk::Button,
    settings_button: gtk::Button,
    settings_window: adw::Window,
    calendar_entry: gtk::Entry,
    timezone_entry: gtk::Entry,
    theme_entry: gtk::Entry,
    account_status_label: gtk::Label,
    account_status_badge: gtk::Label,
    login_button: gtk::Button,
    logout_button: gtk::Button,
    settings_error_label: gtk::Label,
    title_label: gtk::Label,
    settings_title: gtk::Label,
    settings_cal_tz_title: gtk::Label,
    settings_calendar_label: gtk::Label,
    settings_timezone_label: gtk::Label,
    settings_appearance_title: gtk::Label,
    settings_theme_label: gtk::Label,
    settings_language_label: gtk::Label,
    settings_account_title: gtk::Label,
    settings_cancel_button: gtk::Button,
    settings_save_button: gtk::Button,
    language_combo: gtk::ComboBoxText,
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

        let user_settings = read_settings();
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

        // Settings modal window setup
        let settings_window = adw::Window::builder()
            .title(translate(lang, "settings"))
            .default_width(500)
            .default_height(450)
            .transient_for(&root)
            .modal(true)
            .resizable(false)
            .build();
        settings_window.set_decorated(false);
        settings_window.connect_close_request(|window| {
            window.set_visible(false);
            gtk::glib::Propagation::Stop
        });

        let key_controller = gtk::EventControllerKey::new();
        {
            let win = settings_window.clone();
            key_controller.connect_key_pressed(move |_, key, _, _| {
                if key == gdk::Key::Escape {
                    win.set_visible(false);
                    gtk::glib::Propagation::Stop
                } else {
                    gtk::glib::Propagation::Proceed
                }
            });
        }
        settings_window.add_controller(key_controller);

        let settings_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
        settings_box.add_css_class("panel");

        let settings_topbar = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        settings_topbar.add_css_class("topbar");

        let settings_title = label(translate(lang, "settings"), &["title"], 0.0, false);
        settings_topbar.append(&settings_title);

        let settings_top_spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        settings_top_spacer.set_hexpand(true);
        settings_topbar.append(&settings_top_spacer);

        let settings_close = icon_button(
            "window-close-symbolic",
            &["close-button", "icon-button"],
            translate(lang, "close"),
        );
        {
            let win = settings_window.clone();
            settings_close.connect_clicked(move |_| win.set_visible(false));
        }
        settings_topbar.append(&settings_close);
        settings_box.append(&settings_topbar);

        let settings_content = gtk::Box::new(gtk::Orientation::Vertical, 10);
        settings_content.add_css_class("settings-card");

        let settings_cal_tz_title = label(
            translate(lang, "calendar_timezone"),
            &["event-title"],
            0.0,
            false,
        );
        settings_content.append(&settings_cal_tz_title);

        let calendar_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let settings_calendar_label =
            label(translate(lang, "calendar_id"), &["field-label"], 0.0, false);
        calendar_row.append(&settings_calendar_label);
        let calendar_entry = gtk::Entry::builder()
            .text(user_settings.calendar.as_deref().unwrap_or(""))
            .placeholder_text("primary")
            .build();
        calendar_entry.add_css_class("text-entry");
        calendar_entry.set_hexpand(true);
        calendar_row.append(&calendar_entry);
        settings_content.append(&calendar_row);

        let timezone_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let settings_timezone_label =
            label(translate(lang, "timezone"), &["field-label"], 0.0, false);
        timezone_row.append(&settings_timezone_label);
        let timezone_entry = gtk::Entry::builder()
            .text(user_settings.timezone.as_deref().unwrap_or(""))
            .placeholder_text("Local")
            .build();
        timezone_entry.add_css_class("text-entry");
        timezone_entry.set_hexpand(true);
        timezone_row.append(&timezone_entry);
        settings_content.append(&timezone_row);

        let settings_appearance_title =
            label(translate(lang, "appearance"), &["event-title"], 0.0, false);
        settings_content.append(&settings_appearance_title);

        let theme_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let settings_theme_label =
            label(translate(lang, "theme_path"), &["field-label"], 0.0, false);
        theme_row.append(&settings_theme_label);
        let theme_entry = gtk::Entry::builder()
            .text(
                user_settings
                    .theme_path
                    .as_ref()
                    .map(|p| p.to_string_lossy())
                    .as_deref()
                    .unwrap_or(""),
            )
            .placeholder_text("~/.config/waybar-google-calendar/style.css")
            .build();
        theme_entry.add_css_class("text-entry");
        theme_entry.set_hexpand(true);
        theme_row.append(&theme_entry);
        settings_content.append(&theme_row);

        // Language Dropdown
        let language_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let settings_language_label =
            label(translate(lang, "language"), &["field-label"], 0.0, false);
        language_row.append(&settings_language_label);

        let language_combo = gtk::ComboBoxText::new();
        language_combo.append(Some("english"), "English");
        language_combo.append(Some("chinese"), "中文 (Chinese)");
        language_combo.set_active_id(Some(match lang {
            Language::English => "english",
            Language::Chinese => "chinese",
        }));
        language_combo.add_css_class("text-entry");
        language_combo.set_hexpand(true);
        language_row.append(&language_combo);
        settings_content.append(&language_row);

        let settings_account_title = label(
            translate(lang, "google_account"),
            &["event-title"],
            0.0,
            false,
        );
        settings_content.append(&settings_account_title);

        let account_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let account_status_label = label(
            translate(lang, "account_status"),
            &["path-label", "muted"],
            0.0,
            false,
        );
        account_status_label.set_hexpand(true);
        let account_status_badge = label("", &["status-badge"], 0.5, false);
        account_row.append(&account_status_label);
        account_row.append(&account_status_badge);
        settings_content.append(&account_row);

        let account_actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let login_button = classed_button(translate(lang, "login"), &["action-button"]);
        let logout_button = classed_button(translate(lang, "logout"), &["action-button"]);
        {
            let sender = sender.clone();
            login_button.connect_clicked(move |_| sender.input(AgendaMsg::StartAuth));
        }
        {
            let sender = sender.clone();
            logout_button.connect_clicked(move |_| sender.input(AgendaMsg::Logout));
        }
        account_actions.append(&login_button);
        account_actions.append(&logout_button);
        settings_content.append(&account_actions);

        settings_box.append(&settings_content);

        let settings_error_label = label("", &["muted"], 0.0, true);
        settings_box.append(&settings_error_label);

        let settings_buttons = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let settings_save_spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        settings_save_spacer.set_hexpand(true);
        settings_buttons.append(&settings_save_spacer);

        let settings_cancel_button = classed_button(translate(lang, "cancel"), &["action-button"]);
        {
            let win = settings_window.clone();
            settings_cancel_button.connect_clicked(move |_| win.set_visible(false));
        }
        settings_buttons.append(&settings_cancel_button);

        let settings_save_button = classed_button(translate(lang, "save"), &["action-button"]);
        {
            let sender = sender.clone();
            let c_entry = calendar_entry.clone();
            let t_entry = timezone_entry.clone();
            let th_entry = theme_entry.clone();
            let l_combo = language_combo.clone();
            let win = settings_window.clone();
            settings_save_button.connect_clicked(move |_| {
                let cal = c_entry.text().to_string();
                let tz = t_entry.text().to_string();
                let th = th_entry.text().to_string();
                let l_id = l_combo
                    .active_id()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "english".to_string());
                let saved_lang = match l_id.as_str() {
                    "chinese" => Language::Chinese,
                    _ => Language::English,
                };
                sender.input(AgendaMsg::SaveSettings {
                    calendar: if cal.is_empty() { None } else { Some(cal) },
                    timezone: if tz.is_empty() { None } else { Some(tz) },
                    theme_path: if th.is_empty() { None } else { Some(th) },
                    language: saved_lang,
                });
                win.set_visible(false);
            });
        }
        settings_buttons.append(&settings_save_button);
        settings_box.append(&settings_buttons);

        settings_window.set_content(Some(&settings_box));

        let settings_button = icon_button(
            "emblem-system-symbolic",
            &["action-button", "icon-button"],
            translate(lang, "settings"),
        );
        {
            let sender = sender.clone();
            let c_entry = calendar_entry.clone();
            let t_entry = timezone_entry.clone();
            let th_entry = theme_entry.clone();
            let l_combo = language_combo.clone();
            let win = settings_window.clone();
            settings_button.connect_clicked(move |_| {
                let current = read_settings();
                c_entry.set_text(current.calendar.as_deref().unwrap_or(""));
                t_entry.set_text(current.timezone.as_deref().unwrap_or(""));
                th_entry.set_text(
                    current
                        .theme_path
                        .as_ref()
                        .map(|p| p.to_string_lossy())
                        .as_deref()
                        .unwrap_or(""),
                );
                l_combo.set_active_id(Some(match current.language.unwrap_or_default() {
                    Language::English => "english",
                    Language::Chinese => "chinese",
                }));
                win.present();
                sender.input(AgendaMsg::OpenSettings);
            });
        }
        topbar.append(&settings_button);

        let close = icon_button(
            "window-close-symbolic",
            &["close-button", "icon-button"],
            translate(lang, "close"),
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
            user_settings,
            settings_msg: None,
        };

        let mut widgets = AgendaWidgets {
            content,
            status_label,
            refresh,
            settings_button,
            settings_window,
            calendar_entry,
            timezone_entry,
            theme_entry,
            account_status_label,
            account_status_badge,
            login_button,
            logout_button,
            settings_error_label,
            title_label,
            settings_title,
            settings_cal_tz_title,
            settings_calendar_label,
            settings_timezone_label,
            settings_appearance_title,
            settings_theme_label,
            settings_language_label,
            settings_account_title,
            settings_cancel_button,
            settings_save_button,
            language_combo,
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

        // Update settings window labels dynamically
        widgets.settings_title.set_text(translate(lang, "settings"));
        widgets
            .settings_cal_tz_title
            .set_text(translate(lang, "calendar_timezone"));
        widgets
            .settings_calendar_label
            .set_text(translate(lang, "calendar_id"));
        widgets
            .settings_timezone_label
            .set_text(translate(lang, "timezone"));
        widgets
            .settings_appearance_title
            .set_text(translate(lang, "appearance"));
        widgets
            .settings_theme_label
            .set_text(translate(lang, "theme_path"));
        widgets
            .settings_language_label
            .set_text(translate(lang, "language"));
        widgets
            .settings_account_title
            .set_text(translate(lang, "google_account"));
        widgets
            .account_status_label
            .set_text(translate(lang, "account_status"));

        widgets
            .settings_cancel_button
            .set_label(translate(lang, "cancel"));
        widgets
            .settings_save_button
            .set_label(translate(lang, "save"));

        // Update settings dialog state
        let token_exists = paths::oauth_token_file().exists();

        widgets.account_status_badge.remove_css_class("success");
        widgets.account_status_badge.remove_css_class("warning");
        widgets.account_status_badge.remove_css_class("info");

        if self.authenticating {
            widgets
                .account_status_badge
                .set_text(translate(lang, "authenticating"));
            widgets.account_status_badge.add_css_class("info");
            widgets.login_button.set_sensitive(false);
            widgets.logout_button.set_sensitive(false);
            widgets
                .login_button
                .set_label(translate(lang, "authenticating"));
        } else if token_exists {
            widgets
                .account_status_badge
                .set_text(translate(lang, "authenticated"));
            widgets.account_status_badge.add_css_class("success");
            widgets.login_button.set_sensitive(false);
            widgets.logout_button.set_sensitive(true);
            widgets.login_button.set_label(translate(lang, "login"));
        } else {
            widgets
                .account_status_badge
                .set_text(translate(lang, "missing_token"));
            widgets.account_status_badge.add_css_class("warning");
            widgets.login_button.set_sensitive(true);
            widgets.logout_button.set_sensitive(false);
            widgets.login_button.set_label(translate(lang, "login"));
        }

        widgets.logout_button.set_label(translate(lang, "logout"));

        if let Some(msg) = &self.settings_msg {
            let display_msg = match msg.as_str() {
                "Logged out successfully." => translate(lang, "logged_out_success"),
                "Logged out. Please authenticate." => translate(lang, "logged_out_please_auth"),
                _ => msg,
            };
            widgets.settings_error_label.set_text(display_msg);
        } else {
            widgets.settings_error_label.set_text("");
        }
    }
}
