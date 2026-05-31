use crate::calendar::model::{AgendaState, DateRange};
use crate::i18n::{month_name, translate};
use crate::storage::settings::Language;
use chrono::{DateTime, Datelike, Duration as ChronoDuration, Local};

pub(super) fn agenda(state: &AgendaState, lang: Language) -> String {
    if state.loading && state.events.is_empty() {
        return translate(lang, "loading").to_string();
    }
    if let Some(error) = &state.error {
        if state.events.is_empty() {
            return translate(lang, "refresh_failed").to_string();
        }
        return format!(
            "{}; {}",
            cache_status(state.fetched_at, lang, Some(error)),
            translate(lang, "refresh_failed").to_ascii_lowercase()
        );
    }
    if state.loading {
        return format!(
            "{}; {}",
            cache_status(state.fetched_at, lang, None),
            translate(lang, "refreshing").to_ascii_lowercase()
        );
    }
    if state.cached {
        return cache_status(state.fetched_at, lang, None);
    }
    state
        .fetched_at
        .map(|time| format!("{} {}", translate(lang, "updated"), time.format("%H:%M")))
        .unwrap_or_else(|| Local::now().format("%a, %b %-d  %-I:%M %p").to_string())
}

pub(super) fn range(range: DateRange, lang: Language) -> String {
    let end = range.end_exclusive - ChronoDuration::days(1);
    if range.start.year() == end.year() {
        if range.start.month() == end.month() {
            return format!(
                "{} {}-{}",
                month_name(lang, range.start.month()),
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

fn cache_status(
    fetched_at: Option<DateTime<Local>>,
    lang: Language,
    suffix: Option<&str>,
) -> String {
    let prefix = translate(lang, "cached");
    let base = match fetched_at {
        Some(time) => {
            let age = (Local::now() - time).num_seconds().max(0);
            if age < 60 {
                if lang == Language::Chinese {
                    format!("{prefix}刚刚")
                } else {
                    format!("{prefix} just now")
                }
            } else if age < 3600 {
                if lang == Language::Chinese {
                    format!("{prefix} {} 分钟前", age / 60)
                } else {
                    format!("{prefix} {} min ago", age / 60)
                }
            } else {
                if lang == Language::Chinese {
                    format!("{prefix} {} 小时前", age / 3600)
                } else {
                    format!("{prefix} {} h ago", age / 3600)
                }
            }
        }
        None => prefix.to_string(),
    };
    match suffix {
        Some(suffix) if !suffix.is_empty() => format!("{base}: {suffix}"),
        _ => base,
    }
}
