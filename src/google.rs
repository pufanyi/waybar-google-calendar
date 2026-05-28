use crate::date::parse_event_start;
use crate::model::{AgendaQuery, DateRange, Event, FETCH_TIMEOUT_SECONDS};
use crate::paths;
use chrono::{Datelike, Local, NaiveDate, TimeZone};
use chrono_tz::Tz;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use reqwest::StatusCode;
use serde::Deserialize;
use std::env;
use std::fs;
use std::time::Duration;
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};

const CALENDAR_SCOPE: &str = "https://www.googleapis.com/auth/calendar.readonly";
const CALENDAR_API: &str = "https://www.googleapis.com/calendar/v3";

pub fn auth_calendar() -> Result<(), String> {
    runtime()?.block_on(async {
        let timeout = fetch_timeout();
        let _ = access_token(timeout, false).await?;
        Ok(())
    })
}

pub fn fetch_events(query: &AgendaQuery, range: DateRange) -> Result<Vec<Event>, String> {
    runtime()?.block_on(fetch_events_async(query, range))
}

async fn fetch_events_async(query: &AgendaQuery, range: DateRange) -> Result<Vec<Event>, String> {
    let timeout = fetch_timeout();
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

    events.sort_by_key(|event| parse_event_start(&event.start));
    Ok(events)
}

async fn access_token(timeout: u64, require_existing_token: bool) -> Result<String, String> {
    let secret_file = paths::client_secret_file();
    if !secret_file.exists() {
        return Err(format!(
            "Missing Google OAuth client secret. Put it at {} or set WAYBAR_GCAL_CLIENT_SECRET, then run `waybar-gcal auth`.",
            secret_file.display()
        ));
    }

    let token_file = paths::oauth_token_file();
    if require_existing_token && !token_file.exists() {
        return Err(
            "Google Calendar is not authenticated. Run `waybar-gcal auth` first.".to_string(),
        );
    }

    if let Some(parent) = token_file.parent() {
        create_secure_dir(parent)?;
    }

    let secret = yup_oauth2::read_application_secret(&secret_file)
        .await
        .map_err(|err| {
            format!(
                "Could not read Google OAuth client secret {}: {err}",
                secret_file.display()
            )
        })?;

    let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
        .persist_tokens_to_disk(token_file)
        .with_timeout(Duration::from_secs(timeout))
        .build()
        .await
        .map_err(|err| format!("Could not initialize Google OAuth: {err}"))?;

    let token = auth
        .token(&[CALENDAR_SCOPE])
        .await
        .map_err(|err| format!("Could not authenticate Google Calendar: {err}"))?
        .token()
        .map(ToOwned::to_owned)
        .ok_or_else(|| "Google OAuth did not return an access token.".to_string())?;
    secure_token_file();
    Ok(token)
}

async fn fetch_calendars(
    client: &reqwest::Client,
    token: &str,
    query: &AgendaQuery,
) -> Result<Vec<CalendarInfo>, String> {
    let payload: CalendarListResponse = request_json(
        client
            .get(format!("{CALENDAR_API}/users/me/calendarList"))
            .bearer_auth(token)
            .query(&[("minAccessRole", "reader")]),
    )
    .await?;

    let calendars = payload
        .items
        .into_iter()
        .map(|mut calendar| {
            if calendar.summary.is_empty() {
                calendar.summary = calendar.id.clone();
            }
            calendar
        })
        .filter(|calendar| {
            query
                .calendar
                .as_deref()
                .map(|filter| calendar.id == filter || calendar.summary.contains(filter))
                .unwrap_or(true)
        })
        .collect();
    Ok(calendars)
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
                summary: event.summary.unwrap_or_default(),
                calendar: calendar.summary.clone(),
                location: event.location.unwrap_or_default(),
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

async fn request_json<T: for<'de> Deserialize<'de>>(
    request: reqwest::RequestBuilder,
) -> Result<T, String> {
    let response = request
        .send()
        .await
        .map_err(|err| format!("Google Calendar request failed: {err}"))?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(api_error_message(status, &body));
    }
    response
        .json::<T>()
        .await
        .map_err(|err| format!("Could not parse Google Calendar response: {err}"))
}

fn api_error_message(status: StatusCode, body: &str) -> String {
    if let Ok(payload) = serde_json::from_str::<GoogleErrorResponse>(body)
        && let Some(error) = payload.error
    {
        return format!("Google Calendar API returned {status}: {}", error.message);
    }
    let detail = body
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("empty response")
        .trim();
    format!("Google Calendar API returned {status}: {detail}")
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

fn runtime() -> Result<tokio::runtime::Runtime, String> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|err| format!("Could not start async runtime: {err}"))
}

fn fetch_timeout() -> u64 {
    env::var("GCAL_FETCH_TIMEOUT")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(FETCH_TIMEOUT_SECONDS)
}

fn secure_token_file() {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        if let Ok(metadata) = fs::metadata(paths::oauth_token_file()) {
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600);
            let _ = fs::set_permissions(paths::oauth_token_file(), permissions);
        }
    }
}

fn create_secure_dir(path: &std::path::Path) -> Result<(), String> {
    fs::create_dir_all(path)
        .map_err(|err| format!("Could not create token directory {}: {err}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(path)
            .map_err(|err| format!("Could not read token directory {}: {err}", path.display()))?
            .permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(path, permissions).map_err(|err| {
            format!(
                "Could not secure token directory permissions {}: {err}",
                path.display()
            )
        })?;
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct CalendarListResponse {
    #[serde(default)]
    items: Vec<CalendarInfo>,
}

#[derive(Debug, Deserialize)]
struct CalendarInfo {
    #[serde(default)]
    id: String,
    #[serde(default)]
    summary: String,
}

#[derive(Debug, Deserialize)]
struct EventsListResponse {
    #[serde(default)]
    items: Vec<RawEvent>,
    #[serde(default, rename = "nextPageToken")]
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawEvent {
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    location: Option<String>,
    #[serde(default)]
    start: RawEventTime,
    #[serde(default)]
    end: RawEventTime,
}

#[derive(Debug, Default, Deserialize)]
struct RawEventTime {
    #[serde(default, rename = "dateTime")]
    date_time: Option<String>,
    #[serde(default)]
    date: Option<String>,
}

impl RawEventTime {
    fn value(self) -> String {
        self.date_time.or(self.date).unwrap_or_default()
    }
}

#[derive(Debug, Deserialize)]
struct GoogleErrorResponse {
    error: Option<GoogleError>,
}

#[derive(Debug, Deserialize)]
struct GoogleError {
    message: String,
}
