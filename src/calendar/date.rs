use crate::calendar::model::{DateRange, Event};
use chrono::{DateTime, Datelike, Duration as ChronoDuration, Local, NaiveDate, NaiveTime};
use std::collections::BTreeSet;

pub fn parse_event_start(raw: &str) -> Option<(NaiveDate, Option<NaiveTime>)> {
    if raw.contains('T') {
        DateTime::parse_from_rfc3339(raw)
            .ok()
            .map(|dt| dt.with_timezone(&Local))
            .map(|dt| (dt.date_naive(), Some(dt.time())))
    } else {
        NaiveDate::parse_from_str(raw, "%Y-%m-%d")
            .ok()
            .map(|date| (date, None))
    }
}

pub fn format_day_label(date: NaiveDate) -> String {
    let today = Local::now().date_naive();
    if date == today {
        "Today".to_string()
    } else if date == today + ChronoDuration::days(1) {
        "Tomorrow".to_string()
    } else {
        date.format("%a %b %-d").to_string()
    }
}

pub fn format_time_label(start: &str, end: &str) -> String {
    let Some((_, start_time)) = parse_event_start(start) else {
        return "Time unavailable".to_string();
    };
    let Some(start_time) = start_time else {
        return "All day".to_string();
    };
    let end_time = parse_event_start(end)
        .and_then(|(_, time)| time)
        .unwrap_or(start_time);
    format!(
        "{}-{}",
        start_time.format("%-I:%M %p"),
        end_time.format("%-I:%M %p")
    )
}

pub fn event_days(events: &[Event]) -> BTreeSet<NaiveDate> {
    events.iter().filter_map(event_date).collect()
}

pub fn event_date(event: &Event) -> Option<NaiveDate> {
    parse_event_start(&event.start).map(|(date, _)| date)
}

pub fn month_dates(year: i32, month: u32) -> Vec<NaiveDate> {
    let first = NaiveDate::from_ymd_opt(year, month, 1).expect("valid month");
    let offset = first.weekday().num_days_from_monday() as i64;
    let start = first - ChronoDuration::days(offset);
    (0..42)
        .map(|day| start + ChronoDuration::days(day))
        .collect()
}

pub fn visible_month_range(year: i32, month: u32) -> DateRange {
    let dates = month_dates(year, month);
    DateRange {
        start: *dates.first().expect("month grid has a start date"),
        end_exclusive: *dates.last().expect("month grid has an end date") + ChronoDuration::days(1),
    }
}

pub fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Calendar",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_day_event_start() {
        let parsed = parse_event_start("2026-05-28").expect("date should parse");
        assert_eq!(parsed.0, NaiveDate::from_ymd_opt(2026, 5, 28).unwrap());
        assert_eq!(parsed.1, None);
    }

    #[test]
    fn formats_all_day_time_label() {
        assert_eq!(format_time_label("2026-05-28", "2026-05-29"), "All day");
    }

    #[test]
    fn month_grid_has_six_weeks_starting_monday() {
        let dates = month_dates(2026, 5);
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
        let range = visible_month_range(2026, 5);
        assert_eq!(range.start, NaiveDate::from_ymd_opt(2026, 4, 27).unwrap());
        assert_eq!(
            range.end_exclusive,
            NaiveDate::from_ymd_opt(2026, 6, 8).unwrap()
        );
    }
}
