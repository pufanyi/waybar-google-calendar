use crate::model::{CACHE_TTL_SECONDS, CachePayload, CachedEvents, Event};
use crate::paths;
use chrono::{DateTime, Local};
use std::env;
use std::fs;

pub fn read_cache(days: u32) -> Option<CachedEvents> {
    let file = paths::cache_file(days);
    let payload: CachePayload = serde_json::from_str(&fs::read_to_string(file).ok()?).ok()?;
    if payload.version != 1 || payload.days != days {
        return None;
    }
    let fetched_at = DateTime::parse_from_rfc3339(&payload.fetched_at)
        .ok()?
        .with_timezone(&Local);
    Some(CachedEvents {
        events: payload.events,
        fetched_at,
    })
}

pub fn write_cache(days: u32, events: &[Event], fetched_at: DateTime<Local>) {
    let file = paths::cache_file(days);
    if let Some(parent) = file.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let payload = CachePayload {
        version: 1,
        days,
        fetched_at: fetched_at.to_rfc3339(),
        events: events.to_vec(),
    };
    if let Ok(json) = serde_json::to_string(&payload) {
        let _ = fs::write(file, json);
    }
}

pub fn cache_is_fresh(fetched_at: DateTime<Local>) -> bool {
    let ttl = env::var("GCAL_CACHE_TTL")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(CACHE_TTL_SECONDS);
    (Local::now() - fetched_at).num_seconds() <= ttl
}
