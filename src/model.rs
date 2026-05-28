use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const DEFAULT_DAYS: u32 = 7;
pub const CACHE_TTL_SECONDS: i64 = 300;
pub const FETCH_TIMEOUT_SECONDS: u64 = 25;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Agenda,
    Month,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub mode: Mode,
    pub days: u32,
    pub theme_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Event {
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub calendar: String,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub start: String,
    #[serde(default)]
    pub end: String,
}

#[derive(Debug, Deserialize)]
pub struct GwsAgenda {
    #[serde(default)]
    pub events: Vec<Event>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CachePayload {
    pub version: u32,
    pub days: u32,
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
    pub events: Vec<Event>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AgendaState {
    pub events: Vec<Event>,
    pub error: Option<String>,
    pub fetched_at: Option<DateTime<Local>>,
    pub loading: bool,
    pub cached: bool,
}
