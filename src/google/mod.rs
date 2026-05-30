mod api;
mod auth;
mod transport;
mod types;

use std::env;

const CALENDAR_SCOPE: &str = "https://www.googleapis.com/auth/calendar.readonly";
const CALENDAR_API: &str = "https://www.googleapis.com/calendar/v3";

pub use api::fetch_events;
pub use auth::{auth_calendar, save_client_secret};
pub use transport::open_external_uri;

fn runtime() -> Result<tokio::runtime::Runtime, String> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|err| format!("Could not start async runtime: {err}"))
}

fn fetch_timeout(default: u64) -> u64 {
    env::var("GCAL_FETCH_TIMEOUT")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}
