use super::{AgendaApp, AgendaCommandOutput, AgendaMsg, auth_prompt};
use crate::calendar::date::{shift_month, visible_month_range};
use crate::calendar::model::{AgendaResult, DateRange};
use crate::calendar::view::{CalendarViewMode, YEAR_PAGE_STEP};
use crate::google::{self, fetch_events};
use crate::storage::cache::{cache_is_fresh, read_cache, write_cache};
use crate::storage::paths;
use chrono::{Datelike, Local};
use relm4::ComponentSender;

impl AgendaApp {
    pub(super) fn current_range(&self) -> DateRange {
        visible_month_range(self.calendar_year, self.calendar_month)
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
                self.state.error = Some(
                    auth_prompt::open_dir(&paths::config_dir())
                        .map(|_| "Config folder opened.".to_string())
                        .unwrap_or_else(|error| error),
                );
            }
            AgendaMsg::OpenTokenDir => {
                self.state.error = Some(
                    auth_prompt::open_dir(&paths::data_dir())
                        .map(|_| "Token folder opened.".to_string())
                        .unwrap_or_else(|error| error),
                );
            }
            AgendaMsg::OpenSetupGuide => {
                self.state.error = Some(
                    auth_prompt::open_setup_guide()
                        .map(|_| "Setup guide opened.".to_string())
                        .unwrap_or_else(|error| error),
                );
            }
            AgendaMsg::OpenGoogleCloud => {
                self.open_external(
                    auth_prompt::GOOGLE_CLOUD_CREDENTIALS_URL,
                    "Google Cloud opened in your browser.",
                );
            }
            AgendaMsg::OpenCalendarApi => {
                self.open_external(
                    auth_prompt::GOOGLE_CALENDAR_API_URL,
                    "Google Calendar API page opened in your browser.",
                );
            }
            AgendaMsg::OpenSettings => {
                self.settings_form = crate::storage::settings::read_settings();
                self.settings_msg = None;
                self.settings_open = true;
            }
            AgendaMsg::CloseSettings => {
                self.settings_msg = None;
                self.settings_open = false;
            }
            AgendaMsg::Close => {}
            AgendaMsg::SaveSettings {
                calendar,
                timezone,
                theme_path,
                language,
            } => {
                use crate::storage::settings::{UserSettings, write_settings};
                use std::path::PathBuf;

                let calendar = calendar.trim().to_string();
                let timezone = timezone.trim().to_string();
                let theme_path = theme_path.trim().to_string();
                let theme_buf = (!theme_path.is_empty()).then(|| PathBuf::from(theme_path));
                let new_settings = UserSettings {
                    calendar: (!calendar.is_empty()).then_some(calendar),
                    timezone: (!timezone.is_empty()).then_some(timezone),
                    theme_path: theme_buf,
                    language: Some(language),
                };

                if let Err(err) = write_settings(&new_settings) {
                    self.settings_msg = Some(format!("Failed to save settings: {err}"));
                    self.settings_open = true;
                } else {
                    self.settings_form = new_settings.clone();
                    self.user_settings = new_settings;
                    self.query.calendar = self.user_settings.calendar.clone();
                    self.query.timezone = self.user_settings.timezone.clone();

                    // Dynamic theme reload
                    if let Ok(css) =
                        crate::ui::theme::load_css(self.user_settings.theme_path.as_deref())
                    {
                        crate::ui::theme::apply_css(&css);
                    }

                    self.settings_msg = None;
                    self.settings_open = false;
                    self.load_visible_range(sender, true);
                }
            }
            AgendaMsg::Logout => {
                let token_file = paths::oauth_token_file();
                if token_file.exists() {
                    let _ = std::fs::remove_file(token_file);
                }

                // Clear active events and cache
                let range = self.current_range();
                let cache_file =
                    paths::cache_file(&crate::storage::cache::cache_key(&self.query, range));
                if cache_file.exists() {
                    let _ = std::fs::remove_file(cache_file);
                }

                self.state.events.clear();
                self.state.fetched_at = None;
                self.state.cached = false;
                self.state.error = Some("Logged out. Please authenticate.".to_string());
                self.settings_msg = Some("Logged out successfully.".to_string());
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
                    Some("Google Calendar authenticated. Loading events...".to_string());
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
        let today = Local::now().date_naive();
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
        self.state.error = Some("Opening browser for Google OAuth...".to_string());
        sender.spawn_oneshot_command(|| AgendaCommandOutput::Auth(google::auth_calendar()));
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
            write_cache(&self.query, result.range, &result.events, fetched_at);
            self.state.events = result.events;
            self.state.error = None;
            self.state.fetched_at = Some(fetched_at);
            self.state.cached = false;
        }
    }
}
