use crate::calendar::model::{
    AgendaQuery, CACHE_TTL_SECONDS, CachePayload, CachedEvents, DateRange, Event,
};
use crate::storage::paths;
use chrono::{DateTime, Local};
use std::env;
use std::fs;

const CACHE_VERSION: u32 = 3;

pub fn read_cache(query: &AgendaQuery, range: DateRange) -> Option<CachedEvents> {
    let file = paths::cache_file(&cache_key(query, range));
    let payload: CachePayload = serde_json::from_str(&fs::read_to_string(file).ok()?).ok()?;
    if payload.version != CACHE_VERSION
        || payload.calendar != query.calendar
        || payload.timezone != query.timezone
        || payload.start != range.start.to_string()
        || payload.end_exclusive != range.end_exclusive.to_string()
    {
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

pub fn write_cache(
    query: &AgendaQuery,
    range: DateRange,
    events: &[Event],
    fetched_at: DateTime<Local>,
) {
    let file = paths::cache_file(&cache_key(query, range));
    if let Some(parent) = file.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let payload = CachePayload {
        version: CACHE_VERSION,
        calendar: query.calendar.clone(),
        timezone: query.timezone.clone(),
        start: range.start.to_string(),
        end_exclusive: range.end_exclusive.to_string(),
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

pub fn cache_key(query: &AgendaQuery, range: DateRange) -> String {
    let mut key = format!("{}-{}", range.start, range.end_exclusive);
    if let Some(calendar) = &query.calendar {
        key.push_str("-cal-");
        key.push_str(&sanitize_key_part(calendar));
    }
    if let Some(timezone) = &query.timezone {
        key.push_str("-tz-");
        key.push_str(&sanitize_key_part(timezone));
    }
    key
}

fn sanitize_key_part(value: &str) -> String {
    let mut sanitized = String::with_capacity(value.len());
    for character in value.chars() {
        if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
            sanitized.push(character);
        } else {
            sanitized.push('-');
        }
    }
    sanitized.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_includes_query_filters() {
        let query = AgendaQuery {
            calendar: Some("Work Calendar".to_string()),
            timezone: Some("Asia/Singapore".to_string()),
        };
        let range = DateRange {
            start: chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
            end_exclusive: chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        };

        assert_eq!(
            cache_key(&query, range),
            "2026-05-01-2026-06-01-cal-Work-Calendar-tz-Asia-Singapore"
        );
    }
}
