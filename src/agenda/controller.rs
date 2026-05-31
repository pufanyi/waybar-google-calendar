use super::{AgendaApp, AgendaCommandOutput, AgendaMsg, SettingsChanges, auth_prompt};
use crate::calendar::date::{shift_month, today_for_timezone, visible_month_range};
use crate::calendar::model::{AgendaResult, DateRange};
use crate::calendar::view::{CalendarViewMode, YEAR_PAGE_STEP};
use crate::google::{self, fetch_events};
use crate::i18n::translate;
use crate::storage::cache::{cache_is_fresh, clear_agenda_cache, read_cache, write_cache};
use crate::storage::paths;
use chrono::{Datelike, Local};
use relm4::ComponentSender;
use std::io;

impl AgendaApp {
    pub(super) fn current_range(&self) -> DateRange {
        visible_month_range(self.calendar_year, self.calendar_month, self.week_start())
    }

    pub(super) fn load_visible_range(&mut self, sender: ComponentSender<Self>, force: bool) {
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

    pub(super) fn handle_input(&mut self, message: AgendaMsg, sender: ComponentSender<Self>) {
        match message {
            AgendaMsg::Refresh => {
                self.load_visible_range(sender, true);
            }
            AgendaMsg::LoadVisibleRange => {
                self.load_visible_range(sender, false);
            }
            AgendaMsg::PreviousCalendarPage => {
                self.move_calendar_page(-1);
                self.load_visible_range(sender, false);
            }
            AgendaMsg::NextCalendarPage => {
                self.move_calendar_page(1);
                self.load_visible_range(sender, false);
            }
            AgendaMsg::CycleCalendarView => {
                self.calendar_view = self.calendar_view.next_level();
            }
            AgendaMsg::SelectMonth(month) => self.select_month(month, sender),
            AgendaMsg::SelectYear(year) => self.select_year(year, sender),
            AgendaMsg::Today => {
                self.select_today();
                self.load_visible_range(sender, false);
            }
            AgendaMsg::ClearSelection => {
                self.selected_day = None;
            }
            AgendaMsg::SelectDay(day) => {
                let previous_range = self.current_range();
                self.calendar_year = day.year();
                self.calendar_month = day.month();
                self.calendar_view = CalendarViewMode::Days;
                self.selected_day = Some(day);
                if self.current_range() != previous_range {
                    self.load_visible_range(sender, false);
                }
            }
            AgendaMsg::StartAuth => self.start_auth(sender),
            AgendaMsg::SaveAndStartAuth {
                client_id,
                client_secret,
            } => self.save_and_start_auth(client_id, client_secret, sender),
            AgendaMsg::OpenConfigDir => {
                let lang = self.language();
                self.state.error = Some(
                    auth_prompt::open_dir(&paths::config_dir())
                        .map(|_| translate(lang, "config_folder_opened").to_string())
                        .unwrap_or_else(|error| error),
                );
            }
            AgendaMsg::OpenTokenDir => {
                let lang = self.language();
                self.state.error = Some(
                    auth_prompt::open_dir(&paths::data_dir())
                        .map(|_| translate(lang, "token_folder_opened").to_string())
                        .unwrap_or_else(|error| error),
                );
            }
            AgendaMsg::OpenSetupGuide => {
                let lang = self.language();
                self.state.error = Some(
                    auth_prompt::open_setup_guide()
                        .map(|_| translate(lang, "setup_guide_opened").to_string())
                        .unwrap_or_else(|error| error),
                );
            }
            AgendaMsg::OpenGoogleCloud => {
                let lang = self.language();
                self.open_external(
                    auth_prompt::GOOGLE_CLOUD_CREDENTIALS_URL,
                    translate(lang, "google_cloud_opened"),
                );
            }
            AgendaMsg::OpenCalendarApi => {
                let lang = self.language();
                self.open_external(
                    auth_prompt::GOOGLE_CALENDAR_API_URL,
                    translate(lang, "google_calendar_api_opened"),
                );
            }
            AgendaMsg::OpenSettings => {
                match crate::storage::settings::read_settings() {
                    Ok(settings) => {
                        self.settings_form = settings;
                        self.settings_msg = None;
                    }
                    Err(error) => {
                        self.settings_msg = Some(error);
                    }
                }
                self.settings_open = true;
            }
            AgendaMsg::CloseSettings => {
                self.settings_msg = None;
                self.settings_open = false;
            }
            AgendaMsg::Close => {}
            AgendaMsg::EscapePressed => {}
            AgendaMsg::ApplySettings(changes) => self.save_settings(changes, sender, false),
            AgendaMsg::SaveSettings(changes) => self.save_settings(changes, sender, true),
            AgendaMsg::Logout => {
                let token_file = paths::oauth_token_file();
                let mut cleanup_errors = Vec::new();
                match std::fs::remove_file(&token_file) {
                    Ok(()) => {}
                    Err(err) if err.kind() == io::ErrorKind::NotFound => {}
                    Err(err) => {
                        cleanup_errors
                            .push(format!("Could not delete {}: {err}", token_file.display()));
                    }
                }

                if let Err(err) = clear_agenda_cache() {
                    cleanup_errors.push(err);
                }

                self.state.events.clear();
                self.state.fetched_at = None;
                self.state.cached = false;
                if cleanup_errors.is_empty() {
                    let lang = self.language();
                    self.state.error = Some(translate(lang, "logged_out_please_auth").to_string());
                    self.settings_msg = Some(translate(lang, "logged_out_success").to_string());
                } else {
                    let lang = self.language();
                    let message = format!(
                        "{}: {}",
                        translate(lang, "logged_out_cleanup_incomplete"),
                        cleanup_errors.join("; ")
                    );
                    self.state.error = Some(message.clone());
                    self.settings_msg = Some(message);
                }
            }
        }
    }

    pub(super) fn handle_command(
        &mut self,
        output: AgendaCommandOutput,
        sender: ComponentSender<Self>,
    ) {
        match output {
            AgendaCommandOutput::Auth(Ok(())) => {
                self.authenticating = false;
                self.state.error =
                    Some(translate(self.language(), "google_account_authenticated").to_string());
                self.load_visible_range(sender, true);
            }
            AgendaCommandOutput::Auth(Err(error)) => {
                self.authenticating = false;
                self.state.error = Some(error);
            }
            AgendaCommandOutput::Events(result) => self.apply_events_result(result),
        }
    }

    fn move_calendar_page(&mut self, direction: i32) {
        let month_delta = match self.calendar_view {
            CalendarViewMode::Days => direction,
            CalendarViewMode::Months => direction * 12,
            CalendarViewMode::Years => direction * YEAR_PAGE_STEP * 12,
        };
        self.move_month(month_delta);
    }

    fn move_month(&mut self, delta: i32) {
        let (year, month) = shift_month(self.calendar_year, self.calendar_month, delta);
        self.calendar_year = year;
        self.calendar_month = month;
        self.selected_day = None;
    }

    fn select_month(&mut self, month: u32, sender: ComponentSender<Self>) {
        let previous_range = self.current_range();
        self.calendar_month = month;
        self.calendar_view = CalendarViewMode::Days;
        self.selected_day = None;
        if self.current_range() != previous_range {
            self.load_visible_range(sender, false);
        }
    }

    fn select_year(&mut self, year: i32, sender: ComponentSender<Self>) {
        let previous_range = self.current_range();
        self.calendar_year = year;
        self.calendar_view = CalendarViewMode::Months;
        self.selected_day = None;
        if self.current_range() != previous_range {
            self.load_visible_range(sender, false);
        }
    }

    fn select_today(&mut self) {
        let today = today_for_timezone(self.query.timezone.as_deref());
        self.calendar_year = today.year();
        self.calendar_month = today.month();
        self.calendar_view = CalendarViewMode::Days;
        self.selected_day = Some(today);
    }

    fn start_auth(&mut self, sender: ComponentSender<Self>) {
        if self.authenticating {
            return;
        }
        self.authenticating = true;
        self.state.error = Some(translate(self.language(), "opening_browser_oauth").to_string());
        sender.spawn_oneshot_command(|| AgendaCommandOutput::Auth(google::auth_calendar()));
    }

    fn save_settings(
        &mut self,
        changes: SettingsChanges,
        sender: ComponentSender<Self>,
        close_after_save: bool,
    ) {
        use crate::storage::settings::{UserSettings, write_settings};
        use std::path::PathBuf;

        let calendar = changes.calendar.trim().to_string();
        let timezone = changes.timezone.trim().to_string();
        let theme_path = changes.theme_path.trim().to_string();
        let theme_buf = (!theme_path.is_empty()).then(|| PathBuf::from(theme_path));
        let new_settings = UserSettings {
            calendar: (!calendar.is_empty()).then_some(calendar),
            timezone: (!timezone.is_empty()).then_some(timezone),
            theme_path: theme_buf,
            language: Some(changes.language),
            week_start: Some(changes.week_start),
        };

        let css = match crate::ui::theme::load_css(new_settings.theme_path.as_deref()) {
            Ok(css) => css,
            Err(err) => {
                self.settings_msg = Some(format!(
                    "{}: {err}",
                    translate(changes.language, "failed_load_theme")
                ));
                self.settings_open = true;
                return;
            }
        };

        if let Err(err) = write_settings(&new_settings) {
            self.settings_msg = Some(format!(
                "{}: {err}",
                translate(changes.language, "failed_save_settings")
            ));
            self.settings_open = true;
            return;
        }

        self.settings_form = new_settings.clone();
        self.user_settings = new_settings;
        self.query.calendar = self.user_settings.calendar.clone();
        self.query.timezone = self.user_settings.timezone.clone();

        crate::ui::theme::apply_css(&css);
        self.settings_msg = if close_after_save {
            None
        } else {
            Some(translate(changes.language, "settings_applied").to_string())
        };
        self.settings_open = !close_after_save;
        self.load_visible_range(sender, true);
    }

    fn save_and_start_auth(
        &mut self,
        client_id: String,
        client_secret: String,
        sender: ComponentSender<Self>,
    ) {
        if self.authenticating {
            return;
        }
        match google::save_client_secret(&client_id, &client_secret) {
            Ok(path) => {
                let lang = self.language();
                self.state.error = Some(format!(
                    "{} {}. {}",
                    translate(lang, "oauth_client_saved_to"),
                    path.display(),
                    translate(lang, "opening_browser_oauth")
                ));
                sender.input(AgendaMsg::StartAuth);
            }
            Err(error) => {
                self.state.error = Some(error);
            }
        }
    }

    fn open_external(&mut self, uri: &str, success: &str) {
        self.state.error = Some(
            google::open_external_uri(uri)
                .map(|_| success.to_string())
                .unwrap_or_else(|error| error),
        );
    }

    fn apply_events_result(&mut self, result: AgendaResult) {
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
            let cache_error =
                write_cache(&self.query, result.range, &result.events, fetched_at).err();
            self.state.events = result.events;
            self.state.error = cache_error
                .map(|error| format!("Events loaded, but cache could not be saved: {error}"));
            self.state.fetched_at = Some(fetched_at);
            self.state.cached = false;
        }
    }
}
