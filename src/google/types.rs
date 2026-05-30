use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(super) struct CalendarListResponse {
    #[serde(default)]
    pub(super) items: Vec<CalendarInfo>,
}

#[derive(Debug, Deserialize)]
pub(super) struct CalendarInfo {
    #[serde(default)]
    pub(super) id: String,
    #[serde(default)]
    pub(super) summary: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct EventsListResponse {
    #[serde(default)]
    pub(super) items: Vec<RawEvent>,
    #[serde(default, rename = "nextPageToken")]
    pub(super) next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct RawEvent {
    #[serde(default)]
    pub(super) status: Option<String>,
    #[serde(default)]
    pub(super) summary: Option<String>,
    #[serde(default)]
    pub(super) location: Option<String>,
    #[serde(default)]
    pub(super) start: RawEventTime,
    #[serde(default)]
    pub(super) end: RawEventTime,
}

#[derive(Debug, Default, Deserialize)]
pub(super) struct RawEventTime {
    #[serde(default, rename = "dateTime")]
    date_time: Option<String>,
    #[serde(default)]
    date: Option<String>,
}

impl RawEventTime {
    pub(super) fn value(self) -> String {
        self.date_time.or(self.date).unwrap_or_default()
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct GoogleErrorResponse {
    pub(super) error: Option<GoogleError>,
}

#[derive(Debug, Deserialize)]
pub(super) struct GoogleError {
    pub(super) message: String,
}

#[derive(Serialize)]
pub(super) struct ClientSecretFile<'a> {
    pub(super) installed: InstalledClientSecret<'a>,
}

#[derive(Serialize)]
pub(super) struct InstalledClientSecret<'a> {
    pub(super) client_id: &'a str,
    pub(super) project_id: &'a str,
    pub(super) auth_uri: &'a str,
    pub(super) token_uri: &'a str,
    pub(super) auth_provider_x509_cert_url: &'a str,
    pub(super) client_secret: &'a str,
    pub(super) redirect_uris: &'a [&'a str],
}
