use chrono::{DateTime, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const CACHE_TTL_SECONDS: i64 = 300;
pub const FETCH_TIMEOUT_SECONDS: u64 = 25;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Agenda,
    Month,
    Auth,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub mode: Mode,
    pub calendar: Option<String>,
    pub timezone: Option<String>,
    pub theme_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Event {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub calendar_id: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub calendar: String,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, rename = "htmlLink")]
    pub html_link: String,
    #[serde(default)]
    pub start: String,
    #[serde(default)]
    pub end: String,
}

impl Event {
    pub fn key(&self) -> Option<EventKey> {
        if self.id.is_empty() || self.calendar_id.is_empty() {
            return None;
        }

        Some(EventKey {
            calendar_id: self.calendar_id.clone(),
            event_id: self.id.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventKey {
    pub calendar_id: String,
    pub event_id: String,
}

#[derive(Debug, Clone)]
pub struct EventMutation {
    pub summary: String,
    pub location: String,
    pub description: String,
    pub start_date: String,
    pub start_time: String,
    pub end_date: String,
    pub end_time: String,
    pub all_day: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CachePayload {
    pub version: u32,
    pub calendar: Option<String>,
    pub timezone: Option<String>,
    pub start: String,
    pub end_exclusive: String,
    pub fetched_at: String,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone)]
pub struct CachedEvents {
    pub events: Vec<Event>,
    pub fetched_at: DateTime<Local>,
}

#[derive(Debug, Clone)]
pub struct AgendaResult {
    pub range: DateRange,
    pub events: Vec<Event>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AgendaQuery {
    pub calendar: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end_exclusive: NaiveDate,
}

#[derive(Debug, Clone)]
pub struct AgendaState {
    pub range: DateRange,
    pub loading_range: Option<DateRange>,
    pub events: Vec<Event>,
    pub error: Option<String>,
    pub fetched_at: Option<DateTime<Local>>,
    pub loading: bool,
    pub cached: bool,
}
