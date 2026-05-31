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
    #[serde(default)]
    pub(super) primary: bool,
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
    pub(super) id: String,
    #[serde(default)]
    pub(super) status: Option<String>,
    #[serde(default, rename = "htmlLink")]
    pub(super) html_link: Option<String>,
    #[serde(default)]
    pub(super) summary: Option<String>,
    #[serde(default)]
    pub(super) description: Option<String>,
    #[serde(default)]
    pub(super) location: Option<String>,
    #[serde(default)]
    pub(super) start: RawEventTime,
    #[serde(default)]
    pub(super) end: RawEventTime,
}

#[derive(Serialize)]
pub(super) struct EventWritePayload<'a> {
    pub(super) summary: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    pub(super) location: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    pub(super) description: &'a str,
    pub(super) start: EventWriteTime,
    pub(super) end: EventWriteTime,
}

#[derive(Serialize)]
#[serde(untagged)]
pub(super) enum EventWriteTime {
    AllDay {
        date: String,
    },
    Timed {
        #[serde(rename = "dateTime")]
        date_time: String,
    },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calendar_list_response_deserialization() {
        let json =
            r#"{"items": [{"id": "cal1", "summary": "Work", "primary": true}, {"id": "cal2"}]}"#;
        let response: CalendarListResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.items.len(), 2);
        assert_eq!(response.items[0].id, "cal1");
        assert_eq!(response.items[0].summary, "Work");
        assert!(response.items[0].primary);
        assert_eq!(response.items[1].id, "cal2");
        assert_eq!(response.items[1].summary, "");
    }

    #[test]
    fn test_events_list_response_deserialization() {
        let json = r#"{
            "items": [
                {
                    "id": "event1",
                    "status": "confirmed",
                    "htmlLink": "https://calendar.google.com/event?eid=event1",
                    "summary": "Meeting",
                    "description": "Discuss roadmap",
                    "location": "Room A",
                    "start": {"dateTime": "2026-05-30T10:00:00Z"},
                    "end": {"date": "2026-05-31"}
                }
            ],
            "nextPageToken": "token123"
        }"#;
        let response: EventsListResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.items.len(), 1);
        assert_eq!(response.next_page_token.as_deref(), Some("token123"));
        let event = &response.items[0];
        assert_eq!(event.id, "event1");
        assert_eq!(event.status.as_deref(), Some("confirmed"));
        assert_eq!(
            event.html_link.as_deref(),
            Some("https://calendar.google.com/event?eid=event1")
        );
        assert_eq!(event.summary.as_deref(), Some("Meeting"));
        assert_eq!(event.description.as_deref(), Some("Discuss roadmap"));
        assert_eq!(event.location.as_deref(), Some("Room A"));
    }

    #[test]
    fn test_raw_event_time_value() {
        let t1 = RawEventTime {
            date_time: Some("2026-05-30T10:00:00Z".to_string()),
            date: None,
        };
        assert_eq!(t1.value(), "2026-05-30T10:00:00Z");

        let t2 = RawEventTime {
            date_time: None,
            date: Some("2026-05-30".to_string()),
        };
        assert_eq!(t2.value(), "2026-05-30");

        let t3 = RawEventTime {
            date_time: Some("2026-05-30T10:00:00Z".to_string()),
            date: Some("2026-05-30".to_string()),
        };
        assert_eq!(t3.value(), "2026-05-30T10:00:00Z");

        let t4 = RawEventTime {
            date_time: None,
            date: None,
        };
        assert_eq!(t4.value(), "");
    }

    #[test]
    fn test_google_error_response_deserialization() {
        let json = r#"{"error": {"message": "Invalid API Key"}}"#;
        let response: GoogleErrorResponse = serde_json::from_str(json).unwrap();
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().message, "Invalid API Key");
    }

    #[test]
    fn test_client_secret_file_serialization() {
        let payload = ClientSecretFile {
            installed: InstalledClientSecret {
                client_id: "id123",
                project_id: "proj123",
                auth_uri: "https://auth",
                token_uri: "https://token",
                auth_provider_x509_cert_url: "https://cert",
                client_secret: "sec123",
                redirect_uris: &["http://localhost"],
            },
        };
        let serialized = serde_json::to_string(&payload).unwrap();
        assert!(serialized.contains(r#""client_id":"id123""#));
        assert!(serialized.contains(r#""client_secret":"sec123""#));
    }
}
