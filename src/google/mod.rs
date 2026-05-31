mod api;
mod auth;
mod transport;
mod types;

use std::{env, sync::OnceLock};

const CALENDAR_SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/calendar.readonly",
    "https://www.googleapis.com/auth/calendar.events",
];
const CALENDAR_API: &str = "https://www.googleapis.com/calendar/v3";

pub use api::{create_event, delete_event, fetch_events, update_event};
pub use auth::{auth_calendar, save_client_secret};
pub use transport::open_external_uri;

fn runtime() -> Result<&'static tokio::runtime::Runtime, String> {
    static RUNTIME: OnceLock<Result<tokio::runtime::Runtime, String>> = OnceLock::new();

    RUNTIME
        .get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .map_err(|err| format!("Could not start async runtime: {err}"))
        })
        .as_ref()
        .map_err(Clone::clone)
}

fn fetch_timeout(default: u64) -> u64 {
    env::var("GCAL_FETCH_TIMEOUT")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}
