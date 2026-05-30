use crate::calendar::date::month_name;
use crate::calendar::model::{AgendaState, DateRange};
use chrono::{DateTime, Datelike, Duration as ChronoDuration, Local};

pub(super) fn agenda(state: &AgendaState) -> String {
    if state.loading && state.events.is_empty() {
        return "Loading".to_string();
    }
    if let Some(error) = &state.error {
        if state.events.is_empty() {
            return "Refresh failed".to_string();
        }
        return format!(
            "{}; refresh failed",
            cache_status(state.fetched_at, "Cached", Some(error))
        );
    }
    if state.loading {
        return format!(
            "{}; refreshing",
            cache_status(state.fetched_at, "Cached", None)
        );
    }
    if state.cached {
        return cache_status(state.fetched_at, "Cached", None);
    }
    state
        .fetched_at
        .map(|time| format!("Updated {}", time.format("%H:%M")))
        .unwrap_or_else(|| Local::now().format("%a, %b %-d  %-I:%M %p").to_string())
}

pub(super) fn range(range: DateRange) -> String {
    let end = range.end_exclusive - ChronoDuration::days(1);
    if range.start.year() == end.year() {
        if range.start.month() == end.month() {
            return format!(
                "{} {}-{}",
                month_name(range.start.month()),
                range.start.day(),
                end.day()
            );
        }
        return format!(
            "{} {}-{} {}",
            range.start.format("%b"),
            range.start.day(),
            end.format("%b"),
            end.day()
        );
    }
    format!(
        "{}-{}",
        range.start.format("%b %-d %Y"),
        end.format("%b %-d %Y")
    )
}

fn cache_status(fetched_at: Option<DateTime<Local>>, prefix: &str, suffix: Option<&str>) -> String {
    let base = match fetched_at {
        Some(time) => {
            let age = (Local::now() - time).num_seconds().max(0);
            if age < 60 {
                format!("{prefix} just now")
            } else if age < 3600 {
                format!("{prefix} {} min ago", age / 60)
            } else {
                format!("{prefix} {} h ago", age / 3600)
            }
        }
        None => prefix.to_string(),
    };
    match suffix {
        Some(suffix) if !suffix.is_empty() => format!("{base}: {suffix}"),
        _ => base,
    }
}
