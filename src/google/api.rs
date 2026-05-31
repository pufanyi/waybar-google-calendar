use super::auth::access_token;
use super::transport::{request_empty, request_json};
use super::types::{
    CalendarInfo, CalendarListResponse, EventWritePayload, EventWriteTime, EventsListResponse,
};
use super::{CALENDAR_API, fetch_timeout, runtime};
use crate::calendar::date::parse_event_start_for_timezone;
use crate::calendar::model::{
    AgendaQuery, DateRange, Event, EventKey, EventMutation, FETCH_TIMEOUT_SECONDS,
};
use chrono::{
    Datelike, Duration as ChronoDuration, Local, NaiveDate, NaiveTime, TimeZone, Timelike,
};
use chrono_tz::Tz;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use std::time::Duration;

pub fn fetch_events(query: &AgendaQuery, range: DateRange) -> Result<Vec<Event>, String> {
    runtime()?.block_on(fetch_events_async(query, range))
}

async fn fetch_events_async(query: &AgendaQuery, range: DateRange) -> Result<Vec<Event>, String> {
    let timeout = fetch_timeout(FETCH_TIMEOUT_SECONDS);
    let token = access_token(timeout, true).await?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout))
        .build()
        .map_err(|err| format!("Could not build HTTP client: {err}"))?;

    let time_min = range_boundary(range.start, query.timezone.as_deref())?;
    let time_max = range_boundary(range.end_exclusive, query.timezone.as_deref())?;
    let calendars = fetch_calendars(&client, &token, query).await?;
    let mut events = Vec::new();
    let mut successes = 0;
    let mut failures = Vec::new();

    for calendar in calendars {
        match fetch_calendar_events(&client, &token, &calendar, &time_min, &time_max, query).await {
            Ok(calendar_events) => {
                successes += 1;
                events.extend(calendar_events);
            }
            Err(error) => failures.push(format!("{}: {error}", calendar.summary)),
        }
    }

    if successes == 0 && !failures.is_empty() {
        return Err(failures.join("; "));
    }

    events.sort_by_key(|event| {
        parse_event_start_for_timezone(&event.start, query.timezone.as_deref())
    });
    Ok(events)
}

pub fn create_event(query: &AgendaQuery, changes: EventMutation) -> Result<(), String> {
    runtime()?.block_on(create_event_async(query, changes))
}

async fn create_event_async(query: &AgendaQuery, changes: EventMutation) -> Result<(), String> {
    let timeout = fetch_timeout(FETCH_TIMEOUT_SECONDS);
    let token = access_token(timeout, true).await?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout))
        .build()
        .map_err(|err| format!("Could not build HTTP client: {err}"))?;
    let calendar = writable_calendar(&client, &token, query).await?;
    let payload = event_write_payload(&changes, query.timezone.as_deref())?;

    let encoded_id = utf8_percent_encode(&calendar.id, NON_ALPHANUMERIC).to_string();
    let url = format!("{CALENDAR_API}/calendars/{encoded_id}/events");
    let _: serde_json::Value = request_json(
        client
            .post(url)
            .bearer_auth(token)
            .query(&[("sendUpdates", "none")])
            .json(&payload),
    )
    .await?;
    Ok(())
}

pub fn update_event(
    query: &AgendaQuery,
    key: EventKey,
    changes: EventMutation,
) -> Result<(), String> {
    runtime()?.block_on(update_event_async(query, key, changes))
}

async fn update_event_async(
    query: &AgendaQuery,
    key: EventKey,
    changes: EventMutation,
) -> Result<(), String> {
    let timeout = fetch_timeout(FETCH_TIMEOUT_SECONDS);
    let token = access_token(timeout, true).await?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout))
        .build()
        .map_err(|err| format!("Could not build HTTP client: {err}"))?;
    let payload = event_write_payload(&changes, query.timezone.as_deref())?;

    let calendar_id = utf8_percent_encode(&key.calendar_id, NON_ALPHANUMERIC).to_string();
    let event_id = utf8_percent_encode(&key.event_id, NON_ALPHANUMERIC).to_string();
    let url = format!("{CALENDAR_API}/calendars/{calendar_id}/events/{event_id}");
    let _: serde_json::Value = request_json(
        client
            .patch(url)
            .bearer_auth(token)
            .query(&[("sendUpdates", "none")])
            .json(&payload),
    )
    .await?;
    Ok(())
}

pub fn delete_event(key: EventKey) -> Result<(), String> {
    runtime()?.block_on(delete_event_async(key))
}

async fn delete_event_async(key: EventKey) -> Result<(), String> {
    let timeout = fetch_timeout(FETCH_TIMEOUT_SECONDS);
    let token = access_token(timeout, true).await?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout))
        .build()
        .map_err(|err| format!("Could not build HTTP client: {err}"))?;

    let calendar_id = utf8_percent_encode(&key.calendar_id, NON_ALPHANUMERIC).to_string();
    let event_id = utf8_percent_encode(&key.event_id, NON_ALPHANUMERIC).to_string();
    let url = format!("{CALENDAR_API}/calendars/{calendar_id}/events/{event_id}");
    request_empty(
        client
            .delete(url)
            .bearer_auth(token)
            .query(&[("sendUpdates", "none")]),
    )
    .await
}

async fn fetch_calendars(
    client: &reqwest::Client,
    token: &str,
    query: &AgendaQuery,
) -> Result<Vec<CalendarInfo>, String> {
    let calendars = fetch_calendars_with_role(client, token, query, "reader").await?;
    if calendars.is_empty()
        && let Some(filter) = query
            .calendar
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
    {
        return Err(format!(
            "No readable Google Calendar matched \"{filter}\". Check the calendar name or ID."
        ));
    }

    Ok(calendars)
}

async fn writable_calendar(
    client: &reqwest::Client,
    token: &str,
    query: &AgendaQuery,
) -> Result<CalendarInfo, String> {
    let mut calendars = fetch_calendars_with_role(client, token, query, "writer").await?;
    calendars
        .iter()
        .position(|calendar| calendar.primary)
        .or_else(|| (!calendars.is_empty()).then_some(0))
        .map(|index| calendars.remove(index))
        .ok_or_else(|| {
            if let Some(calendar) = &query.calendar {
                format!("No writable Google Calendar matched \"{calendar}\".")
            } else {
                "No writable Google Calendar is available.".to_string()
            }
        })
}

async fn fetch_calendars_with_role(
    client: &reqwest::Client,
    token: &str,
    query: &AgendaQuery,
    min_access_role: &str,
) -> Result<Vec<CalendarInfo>, String> {
    let payload: CalendarListResponse = request_json(
        client
            .get(format!("{CALENDAR_API}/users/me/calendarList"))
            .bearer_auth(token)
            .query(&[("minAccessRole", min_access_role)]),
    )
    .await?;

    Ok(filter_calendars(payload.items, query))
}

fn filter_calendars(calendars: Vec<CalendarInfo>, query: &AgendaQuery) -> Vec<CalendarInfo> {
    let filter = query
        .calendar
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let filter_lower = filter.map(str::to_ascii_lowercase);
    calendars
        .into_iter()
        .map(|mut calendar| {
            if calendar.summary.is_empty() {
                calendar.summary = calendar.id.clone();
            }
            calendar
        })
        .filter(|calendar| {
            filter
                .zip(filter_lower.as_deref())
                .map(|(filter, filter_lower)| {
                    calendar.id == filter
                        || filter_lower == "primary" && calendar.primary
                        || calendar.summary.to_ascii_lowercase().contains(filter_lower)
                })
                .unwrap_or(true)
        })
        .collect()
}

async fn fetch_calendar_events(
    client: &reqwest::Client,
    token: &str,
    calendar: &CalendarInfo,
    time_min: &str,
    time_max: &str,
    query: &AgendaQuery,
) -> Result<Vec<Event>, String> {
    let encoded_id = utf8_percent_encode(&calendar.id, NON_ALPHANUMERIC).to_string();
    let url = format!("{CALENDAR_API}/calendars/{encoded_id}/events");
    let mut events = Vec::new();
    let mut page_token: Option<String> = None;

    loop {
        let mut request = client.get(&url).bearer_auth(token).query(&[
            ("timeMin", time_min),
            ("timeMax", time_max),
            ("singleEvents", "true"),
            ("orderBy", "startTime"),
            ("maxResults", "2500"),
        ]);

        if let Some(timezone) = &query.timezone {
            request = request.query(&[("timeZone", timezone.as_str())]);
        }
        if let Some(page_token) = &page_token {
            request = request.query(&[("pageToken", page_token.as_str())]);
        }

        let payload: EventsListResponse = request_json(request).await?;
        events.extend(payload.items.into_iter().filter_map(|event| {
            if event.status.as_deref() == Some("cancelled") {
                return None;
            }
            Some(Event {
                id: event.id,
                calendar_id: calendar.id.clone(),
                summary: event.summary.unwrap_or_default(),
                calendar: calendar.summary.clone(),
                location: event.location.unwrap_or_default(),
                description: event.description.unwrap_or_default(),
                html_link: event.html_link.unwrap_or_default(),
                start: event.start.value(),
                end: event.end.value(),
            })
        }));

        page_token = payload.next_page_token;
        if page_token.is_none() {
            break;
        }
    }

    Ok(events)
}

fn event_write_payload<'a>(
    changes: &'a EventMutation,
    timezone: Option<&str>,
) -> Result<EventWritePayload<'a>, String> {
    let summary = changes.summary.trim();
    if summary.is_empty() {
        return Err("Event title is empty.".to_string());
    }

    let start = if changes.all_day {
        EventWriteTime::AllDay {
            date: parse_date(&changes.start_date)?.to_string(),
        }
    } else {
        EventWriteTime::Timed {
            date_time: date_time_value(&changes.start_date, &changes.start_time, timezone)?,
        }
    };
    let end = if changes.all_day {
        let start_date = parse_date(&changes.start_date)?;
        let end_date = parse_date(&changes.end_date)?;
        if end_date < start_date {
            return Err("End date must be on or after start date.".to_string());
        }
        EventWriteTime::AllDay {
            date: (end_date + ChronoDuration::days(1)).to_string(),
        }
    } else {
        let start_key = parse_date_time_key(&changes.start_date, &changes.start_time)?;
        let end_key = parse_date_time_key(&changes.end_date, &changes.end_time)?;
        if end_key <= start_key {
            return Err("End time must be after start time.".to_string());
        }
        EventWriteTime::Timed {
            date_time: date_time_value(&changes.end_date, &changes.end_time, timezone)?,
        }
    };

    Ok(EventWritePayload {
        summary,
        location: changes.location.trim(),
        description: changes.description.trim(),
        start,
        end,
    })
}

fn parse_date(raw: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d")
        .map_err(|_| "Use dates in YYYY-MM-DD format.".to_string())
}

fn parse_time(raw: &str) -> Result<NaiveTime, String> {
    NaiveTime::parse_from_str(raw.trim(), "%H:%M")
        .map_err(|_| "Use times in HH:MM format.".to_string())
}

fn parse_date_time_key(date: &str, time: &str) -> Result<(NaiveDate, NaiveTime), String> {
    Ok((parse_date(date)?, parse_time(time)?))
}

fn date_time_value(date: &str, time: &str, timezone: Option<&str>) -> Result<String, String> {
    let date = parse_date(date)?;
    let time = parse_time(time)?;
    if let Some(timezone) = timezone {
        let tz = timezone
            .parse::<Tz>()
            .map_err(|_| format!("Invalid timezone: {timezone}"))?;
        return tz
            .with_ymd_and_hms(
                date.year(),
                date.month(),
                date.day(),
                time.hour(),
                time.minute(),
                0,
            )
            .earliest()
            .map(|date_time| date_time.to_rfc3339())
            .ok_or_else(|| format!("Could not build event time for {date} in {timezone}"));
    }

    Local
        .with_ymd_and_hms(
            date.year(),
            date.month(),
            date.day(),
            time.hour(),
            time.minute(),
            0,
        )
        .earliest()
        .map(|date_time| date_time.to_rfc3339())
        .ok_or_else(|| format!("Could not build local event time for {date}"))
}

fn range_boundary(date: NaiveDate, timezone: Option<&str>) -> Result<String, String> {
    if let Some(timezone) = timezone {
        let tz = timezone
            .parse::<Tz>()
            .map_err(|_| format!("Invalid timezone: {timezone}"))?;
        return tz
            .with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
            .earliest()
            .map(|date_time| date_time.to_rfc3339())
            .ok_or_else(|| format!("Could not build range boundary for {date} in {timezone}"));
    }

    Local
        .with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
        .earliest()
        .map(|date_time| date_time.to_rfc3339())
        .ok_or_else(|| format!("Could not build local range boundary for {date}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_range_boundary_valid_timezone() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 30).unwrap();
        let boundary = range_boundary(date, Some("Asia/Shanghai")).unwrap();
        assert_eq!(boundary, "2026-05-30T00:00:00+08:00");

        let boundary_utc = range_boundary(date, Some("UTC")).unwrap();
        assert_eq!(boundary_utc, "2026-05-30T00:00:00+00:00");
    }

    #[test]
    fn test_range_boundary_invalid_timezone() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 30).unwrap();
        let err = range_boundary(date, Some("Invalid/Timezone")).unwrap_err();
        assert!(err.contains("Invalid timezone: Invalid/Timezone"));
    }

    #[test]
    fn test_range_boundary_none_timezone_fallback() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 30).unwrap();
        let boundary = range_boundary(date, None).unwrap();
        assert!(boundary.contains("2026-05-30T00:00:00"));
    }

    #[test]
    fn event_write_payload_serializes_timed_event() {
        let changes = EventMutation {
            summary: "Planning".to_string(),
            location: "Room A".to_string(),
            description: "Discuss roadmap".to_string(),
            start_date: "2026-05-30".to_string(),
            start_time: "09:00".to_string(),
            end_date: "2026-05-30".to_string(),
            end_time: "10:00".to_string(),
            all_day: false,
        };

        let payload = event_write_payload(&changes, Some("UTC")).unwrap();
        let json = serde_json::to_value(payload).unwrap();
        assert_eq!(json["summary"], "Planning");
        assert_eq!(json["location"], "Room A");
        assert_eq!(json["description"], "Discuss roadmap");
        assert_eq!(json["start"]["dateTime"], "2026-05-30T09:00:00+00:00");
        assert_eq!(json["end"]["dateTime"], "2026-05-30T10:00:00+00:00");
    }

    #[test]
    fn event_write_payload_serializes_all_day_event_with_exclusive_end() {
        let changes = EventMutation {
            summary: "OOO".to_string(),
            location: String::new(),
            description: String::new(),
            start_date: "2026-05-30".to_string(),
            start_time: String::new(),
            end_date: "2026-05-31".to_string(),
            end_time: String::new(),
            all_day: true,
        };

        let payload = event_write_payload(&changes, Some("UTC")).unwrap();
        let json = serde_json::to_value(payload).unwrap();
        assert_eq!(json["start"]["date"], "2026-05-30");
        assert_eq!(json["end"]["date"], "2026-06-01");
        assert!(json.get("location").is_none());
        assert!(json.get("description").is_none());
    }

    #[test]
    fn filter_calendars_matches_primary_alias() {
        let query = AgendaQuery {
            calendar: Some("primary".to_string()),
            timezone: None,
        };
        let calendars = filter_calendars(
            vec![
                CalendarInfo {
                    id: "work@example.com".to_string(),
                    summary: "Work".to_string(),
                    primary: false,
                },
                CalendarInfo {
                    id: "me@example.com".to_string(),
                    summary: "Personal".to_string(),
                    primary: true,
                },
            ],
            &query,
        );

        assert_eq!(calendars.len(), 1);
        assert_eq!(calendars[0].id, "me@example.com");
    }
}
