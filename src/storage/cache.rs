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
) -> Result<(), String> {
    let file = paths::cache_file(&cache_key(query, range));
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("Could not create cache folder {}: {err}", parent.display()))?;
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
    let json = serde_json::to_string(&payload)
        .map_err(|err| format!("Could not serialize agenda cache: {err}"))?;
    fs::write(&file, json).map_err(|err| format!("Could not write cache {}: {err}", file.display()))
}

pub fn clear_agenda_cache() -> Result<usize, String> {
    let dir = paths::cache_dir();
    if !dir.exists() {
        return Ok(0);
    }

    let entries = fs::read_dir(&dir)
        .map_err(|err| format!("Could not read cache folder {}: {err}", dir.display()))?;
    let mut removed = 0;
    let mut errors = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                errors.push(format!("Could not read cache entry: {err}"));
                continue;
            }
        };
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.starts_with("agenda-") || !name.ends_with(".json") {
            continue;
        }
        match fs::remove_file(&path) {
            Ok(()) => removed += 1,
            Err(err) => errors.push(format!("{}: {err}", path.display())),
        }
    }

    if errors.is_empty() {
        Ok(removed)
    } else {
        Err(format!(
            "Could not clear some cache files: {}",
            errors.join("; ")
        ))
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
    use std::path::PathBuf;

    struct EnvGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
        original_xdg: Option<std::ffi::OsString>,
        original_ttl: Option<std::ffi::OsString>,
        temp_dir: PathBuf,
    }

    impl EnvGuard {
        fn new(name: &str) -> Self {
            let lock = crate::test_env::ENV_LOCK
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let original_xdg = env::var_os("XDG_CACHE_HOME");
            let original_ttl = env::var_os("GCAL_CACHE_TTL");
            let temp_dir = env::temp_dir().join(format!("gcal-test-cache-{}", name));
            let _ = fs::remove_dir_all(&temp_dir);
            unsafe {
                env::set_var("XDG_CACHE_HOME", &temp_dir);
            }
            Self {
                _lock: lock,
                original_xdg,
                original_ttl,
                temp_dir,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            unsafe {
                if let Some(v) = &self.original_xdg {
                    env::set_var("XDG_CACHE_HOME", v);
                } else {
                    env::remove_var("XDG_CACHE_HOME");
                }
                if let Some(v) = &self.original_ttl {
                    env::set_var("GCAL_CACHE_TTL", v);
                } else {
                    env::remove_var("GCAL_CACHE_TTL");
                }
            }
            let _ = fs::remove_dir_all(&self.temp_dir);
        }
    }

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

    #[test]
    fn test_cache_is_fresh() {
        let _guard = EnvGuard::new("fresh");

        let now = Local::now();
        assert!(cache_is_fresh(now));

        let old = now - chrono::Duration::seconds(CACHE_TTL_SECONDS + 10);
        assert!(!cache_is_fresh(old));

        // Test custom TTL override
        unsafe {
            env::set_var("GCAL_CACHE_TTL", "10");
        }
        assert!(cache_is_fresh(now - chrono::Duration::seconds(5)));
        assert!(!cache_is_fresh(now - chrono::Duration::seconds(15)));
    }

    #[test]
    fn test_read_write_cache_roundtrip() {
        let _guard = EnvGuard::new("roundtrip");

        let query = AgendaQuery {
            calendar: Some("my-calendar".to_string()),
            timezone: Some("UTC".to_string()),
        };
        let range = DateRange {
            start: chrono::NaiveDate::from_ymd_opt(2026, 5, 30).unwrap(),
            end_exclusive: chrono::NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(),
        };
        let events = vec![Event {
            summary: "Meeting".to_string(),
            calendar: "my-calendar".to_string(),
            location: "Office".to_string(),
            start: "2026-05-30T10:00:00Z".to_string(),
            end: "2026-05-30T11:00:00Z".to_string(),
        }];

        let fetched_at = Local::now();
        write_cache(&query, range, &events, fetched_at).unwrap();

        let cached = read_cache(&query, range).expect("Should successfully read cache");
        assert_eq!(cached.events.len(), 1);
        assert_eq!(cached.events[0].summary, "Meeting");
        assert_eq!(cached.fetched_at.timestamp(), fetched_at.timestamp());

        // Check timezone mismatch
        let query_diff_tz = AgendaQuery {
            calendar: Some("my-calendar".to_string()),
            timezone: Some("EST".to_string()),
        };
        assert!(read_cache(&query_diff_tz, range).is_none());

        // Check range mismatch
        let range_diff = DateRange {
            start: chrono::NaiveDate::from_ymd_opt(2026, 5, 30).unwrap(),
            end_exclusive: chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        };
        assert!(read_cache(&query, range_diff).is_none());
    }

    #[test]
    fn test_sanitize_key_part() {
        assert_eq!(sanitize_key_part("abc-123_XYZ"), "abc-123_XYZ");
        assert_eq!(
            sanitize_key_part("hello/world@domain.com"),
            "hello-world-domain-com"
        );
        assert_eq!(sanitize_key_part("---test---"), "test");
    }

    #[test]
    fn clear_agenda_cache_only_removes_agenda_json_files() {
        let _guard = EnvGuard::new("clear");
        let dir = paths::cache_dir();
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("agenda-one.json"), "{}").unwrap();
        fs::write(dir.join("agenda-two.json"), "{}").unwrap();
        fs::write(dir.join("other.json"), "{}").unwrap();

        assert_eq!(clear_agenda_cache().unwrap(), 2);
        assert!(!dir.join("agenda-one.json").exists());
        assert!(!dir.join("agenda-two.json").exists());
        assert!(dir.join("other.json").exists());
    }
}
