use super::types::GoogleErrorResponse;
use reqwest::StatusCode;
use serde::Deserialize;

pub(super) async fn request_json<T: for<'de> Deserialize<'de>>(
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

pub(super) async fn request_empty(request: reqwest::RequestBuilder) -> Result<(), String> {
    let response = request
        .send()
        .await
        .map_err(|err| format!("Google Calendar request failed: {err}"))?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(api_error_message(status, &body));
    }
    Ok(())
}

pub fn open_external_uri(uri: &str) -> Result<(), String> {
    gtk::gio::AppInfo::launch_default_for_uri(uri, None::<&gtk::gio::AppLaunchContext>)
        .map_err(|err| format!("Could not open {uri}: {err}"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;

    #[test]
    fn test_api_error_message_valid_json() {
        let status = StatusCode::BAD_REQUEST;
        let body = r#"{"error": {"message": "Invalid API key"}}"#;
        let msg = api_error_message(status, body);
        assert_eq!(
            msg,
            "Google Calendar API returned 400 Bad Request: Invalid API key"
        );
    }

    #[test]
    fn test_api_error_message_non_json() {
        let status = StatusCode::INTERNAL_SERVER_ERROR;
        let body = "Service is currently unavailable.\nPlease try again later.";
        let msg = api_error_message(status, body);
        assert_eq!(
            msg,
            "Google Calendar API returned 500 Internal Server Error: Service is currently unavailable."
        );
    }

    #[test]
    fn test_api_error_message_empty_body() {
        let status = StatusCode::UNAUTHORIZED;
        let body = "   \n  \n  ";
        let msg = api_error_message(status, body);
        assert_eq!(
            msg,
            "Google Calendar API returned 401 Unauthorized: empty response"
        );
    }
}
