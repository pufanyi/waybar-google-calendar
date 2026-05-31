use crate::calendar::model::{DateRange, Event};
use crate::i18n::translate;
use crate::storage::settings::{Language, WeekStart};
use chrono::{DateTime, Datelike, Duration as ChronoDuration, Local, NaiveDate, NaiveTime};
use chrono_tz::Tz;
use std::collections::BTreeSet;

pub fn parse_event_start_for_timezone(
    raw: &str,
    timezone: Option<&str>,
) -> Option<(NaiveDate, Option<NaiveTime>)> {
    if raw.contains('T') {
        let dt = DateTime::parse_from_rfc3339(raw).ok()?;
        if let Some(tz) = timezone.and_then(|timezone| timezone.parse::<Tz>().ok()) {
            let dt = dt.with_timezone(&tz);
            Some((dt.date_naive(), Some(dt.time())))
        } else {
            let dt = dt.with_timezone(&Local);
            Some((dt.date_naive(), Some(dt.time())))
        }
    } else {
        NaiveDate::parse_from_str(raw, "%Y-%m-%d")
            .ok()
            .map(|date| (date, None))
    }
}

pub fn format_day_label_for_timezone(
    date: NaiveDate,
    timezone: Option<&str>,
    lang: Language,
) -> String {
    let today = today_for_timezone(timezone);
    if date == today {
        translate(lang, "today").to_string()
    } else if date == today + ChronoDuration::days(1) {
        translate(lang, "tomorrow").to_string()
    } else {
        date.format("%a %b %-d").to_string()
    }
}

pub fn format_time_label_for_timezone(
    start: &str,
    end: &str,
    timezone: Option<&str>,
    lang: Language,
) -> String {
    let Some((_, start_time)) = parse_event_start_for_timezone(start, timezone) else {
        return translate(lang, "time_unavailable").to_string();
    };
    let Some(start_time) = start_time else {
        return translate(lang, "all_day").to_string();
    };
    let end_time = parse_event_start_for_timezone(end, timezone)
        .and_then(|(_, time)| time)
        .unwrap_or(start_time);
    format!(
        "{}-{}",
        start_time.format("%-I:%M %p"),
        end_time.format("%-I:%M %p")
    )
}

pub fn event_days_for_timezone(events: &[Event], timezone: Option<&str>) -> BTreeSet<NaiveDate> {
    events
        .iter()
        .filter_map(|event| event_date_for_timezone(event, timezone))
        .collect()
}

pub fn event_date_for_timezone(event: &Event, timezone: Option<&str>) -> Option<NaiveDate> {
    parse_event_start_for_timezone(&event.start, timezone).map(|(date, _)| date)
}

pub fn today_for_timezone(timezone: Option<&str>) -> NaiveDate {
    if let Some(tz) = timezone.and_then(|timezone| timezone.parse::<Tz>().ok()) {
        Local::now().with_timezone(&tz).date_naive()
    } else {
        Local::now().date_naive()
    }
}

pub fn month_dates(year: i32, month: u32, week_start: WeekStart) -> Vec<NaiveDate> {
    let first = NaiveDate::from_ymd_opt(year, month, 1).expect("valid month");
    let offset = (first.weekday().num_days_from_monday() + 7 - week_start.days_from_monday()) % 7;
    let offset = offset as i64;
    let start = first - ChronoDuration::days(offset);
    (0..42)
        .map(|day| start + ChronoDuration::days(day))
        .collect()
}

pub fn visible_month_range(year: i32, month: u32, week_start: WeekStart) -> DateRange {
    let dates = month_dates(year, month, week_start);
    DateRange {
        start: *dates.first().expect("month grid has a start date"),
        end_exclusive: *dates.last().expect("month grid has an end date") + ChronoDuration::days(1),
    }
}

pub fn shift_month(year: i32, month: u32, delta: i32) -> (i32, u32) {
    assert!((1..=12).contains(&month), "valid month");
    let month_index = year * 12 + month as i32 - 1 + delta;
    let shifted_year = month_index.div_euclid(12);
    let shifted_month = month_index.rem_euclid(12) as u32 + 1;
    (shifted_year, shifted_month)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_day_event_start() {
        let parsed = parse_event_start_for_timezone("2026-05-28", None).expect("date should parse");
        assert_eq!(parsed.0, NaiveDate::from_ymd_opt(2026, 5, 28).unwrap());
        assert_eq!(parsed.1, None);
    }

    #[test]
    fn parses_timed_event_in_requested_timezone() {
        let parsed = parse_event_start_for_timezone("2026-05-30T23:30:00Z", Some("Asia/Singapore"))
            .expect("date time should parse");

        assert_eq!(parsed.0, NaiveDate::from_ymd_opt(2026, 5, 31).unwrap());
        assert_eq!(parsed.1, Some(NaiveTime::from_hms_opt(7, 30, 0).unwrap()));
    }

    #[test]
    fn formats_all_day_time_label() {
        assert_eq!(
            format_time_label_for_timezone("2026-05-28", "2026-05-29", None, Language::English),
            "All day"
        );
    }

    #[test]
    fn month_grid_has_six_weeks_starting_monday() {
        let dates = month_dates(2026, 5, WeekStart::Monday);
        assert_eq!(dates.len(), 42);
        assert_eq!(dates.first().unwrap().weekday().number_from_monday(), 1);
        assert!(
            dates
                .iter()
                .any(|date| *date == NaiveDate::from_ymd_opt(2026, 5, 1).unwrap())
        );
    }

    #[test]
    fn visible_month_range_covers_rendered_grid() {
        let range = visible_month_range(2026, 5, WeekStart::Monday);
        assert_eq!(range.start, NaiveDate::from_ymd_opt(2026, 4, 27).unwrap());
        assert_eq!(
            range.end_exclusive,
            NaiveDate::from_ymd_opt(2026, 6, 8).unwrap()
        );
    }

    #[test]
    fn month_grid_can_start_on_sunday() {
        let dates = month_dates(2026, 5, WeekStart::Sunday);

        assert_eq!(dates.len(), 42);
        assert_eq!(dates.first().unwrap().weekday().number_from_sunday(), 1);
        assert_eq!(
            *dates.first().unwrap(),
            NaiveDate::from_ymd_opt(2026, 4, 26).unwrap()
        );
    }

    #[test]
    fn shifts_months_across_year_boundaries() {
        assert_eq!(shift_month(2026, 1, -1), (2025, 12));
        assert_eq!(shift_month(2026, 12, 1), (2027, 1));
        assert_eq!(shift_month(2026, 5, -12), (2025, 5));
        assert_eq!(shift_month(2026, 5, 12), (2027, 5));
    }
}
