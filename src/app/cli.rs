use crate::calendar::model::{Config, Mode};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum CliCommand {
    Run(Config),
    Auth,
    PrintTheme,
    Help,
}

pub fn parse_args(args: Vec<String>) -> Result<CliCommand, String> {
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        return Ok(CliCommand::Help);
    }

    let mut mode = Mode::Agenda;
    let mut calendar = env_string("GCAL_CALENDAR");
    let mut timezone = env_string("GCAL_TIMEZONE");
    let mut theme_path = env_path("WAYBAR_GCAL_THEME");

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "auth" => return Ok(CliCommand::Auth),
            "auth-ui" | "auth-gui" | "settings" => {
                mode = Mode::Auth;
                index += 1;
            }
            "print-theme" => return Ok(CliCommand::PrintTheme),
            "agenda" => {
                mode = Mode::Agenda;
                index += 1;
            }
            "month" | "calendar" => {
                mode = Mode::Month;
                index += 1;
            }
            "--days" => {
                let raw = args
                    .get(index + 1)
                    .ok_or_else(|| "--days requires a positive integer".to_string())?;
                let days = raw
                    .parse::<u32>()
                    .map_err(|_| format!("Invalid --days value: {raw}"))?;
                if days == 0 || days > 90 {
                    return Err("--days must be between 1 and 90".to_string());
                }
                index += 2;
            }
            "--theme" => {
                let raw = args
                    .get(index + 1)
                    .ok_or_else(|| "--theme requires a CSS file path".to_string())?;
                theme_path = Some(PathBuf::from(raw));
                index += 2;
            }
            "--calendar" => {
                let raw = args
                    .get(index + 1)
                    .ok_or_else(|| "--calendar requires a calendar name or ID".to_string())?;
                calendar = Some(raw.clone());
                index += 2;
            }
            "--timezone" | "--tz" => {
                let raw = args
                    .get(index + 1)
                    .ok_or_else(|| "--timezone requires an IANA timezone".to_string())?;
                timezone = Some(raw.clone());
                index += 2;
            }
            unknown => return Err(format!("Unknown argument: {unknown}")),
        }
    }

    Ok(CliCommand::Run(Config {
        mode,
        calendar,
        timezone,
        theme_path,
    }))
}

pub fn print_help() {
    println!(
        "Usage:
  waybar-gcal agenda [--calendar NAME_OR_ID] [--timezone TZ] [--theme PATH]
  waybar-gcal month [--theme PATH]
  waybar-gcal auth
  waybar-gcal auth-ui [--theme PATH]
  waybar-gcal print-theme

Agenda:
  Events are fetched for the visible calendar grid. Changing months refreshes
  the Google Calendar range for the month currently on screen.
  --days N is accepted for older Waybar configs but no longer controls fetching.

Theme:
  Default user theme path: ~/.config/waybar-google-calendar/style.css
  --theme PATH overrides the default user theme path

Environment:
  GCAL_DAYS              Deprecated; accepted for older configs
  GCAL_CALENDAR          Calendar name or ID filter for agenda
  GCAL_TIMEZONE          IANA timezone override for agenda
  GCAL_CACHE_TTL         Cache freshness in seconds (default: 300)
  GCAL_FETCH_TIMEOUT     Google API request/auth timeout in seconds (default: 25)
  WAYBAR_GCAL_CLIENT_SECRET
                         OAuth client secret JSON path
  WAYBAR_GCAL_THEME      CSS file appended after the built-in theme"
    );
}

fn env_string(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.is_empty())
}

fn env_path(name: &str) -> Option<PathBuf> {
    env::var_os(name).and_then(|value| {
        if value.is_empty() {
            None
        } else {
            Some(PathBuf::from(value))
        }
    })
}
