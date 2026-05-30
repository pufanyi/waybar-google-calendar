use super::{AgendaApp, AgendaCommandOutput, AgendaMsg, auth_prompt};
use crate::calendar::date::visible_month_range;
use crate::calendar::model::{AgendaResult, DateRange};
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
            AgendaMsg::PreviousMonth => {
                self.move_month(-1);
                self.load_visible_range(sender, false);
            }
            AgendaMsg::NextMonth => {
                self.move_month(1);
                self.load_visible_range(sender, false);
            }
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
            AgendaMsg::PreviousAuthPage => {
                self.auth_page = self.auth_page.saturating_sub(1);
            }
            AgendaMsg::NextAuthPage => {
                self.auth_page = (self.auth_page + 1).min(auth_prompt::LAST_PAGE);
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

    fn move_month(&mut self, delta: i32) {
        let month = self.calendar_month as i32 + delta;
        if month < 1 {
            self.calendar_month = 12;
            self.calendar_year -= 1;
        } else if month > 12 {
            self.calendar_month = 1;
            self.calendar_year += 1;
        } else {
            self.calendar_month = month as u32;
        }
        self.selected_day = None;
    }

    fn select_today(&mut self) {
        let today = Local::now().date_naive();
        self.calendar_year = today.year();
        self.calendar_month = today.month();
        self.selected_day = Some(today);
    }

    fn start_auth(&mut self, sender: ComponentSender<Self>) {
        if self.authenticating {
            return;
        }
        self.auth_page = auth_prompt::LAST_PAGE;
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
                self.auth_page = auth_prompt::LAST_PAGE;
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
