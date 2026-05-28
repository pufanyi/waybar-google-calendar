use crate::date::parse_event_start;
use crate::model::{Event, FETCH_TIMEOUT_SECONDS, GwsAgenda};
use std::env;
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

pub fn auth_calendar() -> Result<i32, String> {
    let status = Command::new("gws")
        .args(["auth", "login", "--services", "calendar", "--readonly"])
        .status()
        .map_err(|err| format!("Could not run gws auth login: {err}"))?;
    Ok(status.code().unwrap_or(1))
}

pub fn fetch_events(days: u32) -> Result<Vec<Event>, String> {
    let timeout = env::var("GCAL_FETCH_TIMEOUT")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(FETCH_TIMEOUT_SECONDS);

    let mut child = Command::new("gws")
        .args([
            "calendar",
            "+agenda",
            "--days",
            &days.to_string(),
            "--format",
            "json",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| format!("Could not run gws. Is googleworkspace-cli installed? {err}"))?;

    match child
        .wait_timeout(Duration::from_secs(timeout))
        .map_err(|err| format!("Could not wait for gws: {err}"))?
    {
        Some(_) => {
            let output = child
                .wait_with_output()
                .map_err(|err| format!("Could not read gws output: {err}"))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                let message = stderr
                    .lines()
                    .chain(stdout.lines())
                    .find(|line| !line.trim().is_empty())
                    .unwrap_or("gws returned an error")
                    .trim()
                    .to_string();
                return Err(message);
            }

            let payload: GwsAgenda = serde_json::from_slice(&output.stdout)
                .map_err(|err| format!("Could not parse gws JSON output: {err}"))?;
            let mut events = payload.events;
            events.sort_by_key(|event| parse_event_start(&event.start));
            Ok(events)
        }
        None => {
            let _ = child.kill();
            let _ = child.wait();
            Err("Timed out while reading Google Calendar.".to_string())
        }
    }
}
