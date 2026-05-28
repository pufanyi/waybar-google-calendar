use crate::model::{Config, DEFAULT_DAYS, Mode};
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
    let mut days = env::var("GCAL_DAYS")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(DEFAULT_DAYS);
    let mut theme_path = env_path("WAYBAR_GCAL_THEME");

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "auth" => return Ok(CliCommand::Auth),
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
                days = raw
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
            unknown => return Err(format!("Unknown argument: {unknown}")),
        }
    }

    Ok(CliCommand::Run(Config {
        mode,
        days,
        theme_path,
    }))
}

pub fn print_help() {
    println!(
        "Usage:
  waybar-gcal agenda [--days N] [--theme PATH]
  waybar-gcal month [--theme PATH]
  waybar-gcal auth
  waybar-gcal print-theme

Theme:
  Default user theme path: ~/.config/waybar-google-calendar/style.css
  --theme PATH overrides the default user theme path

Environment:
  GCAL_DAYS              Default agenda window, in days (default: 7)
  GCAL_CACHE_TTL         Cache freshness in seconds (default: 300)
  GCAL_FETCH_TIMEOUT     gws fetch timeout in seconds (default: 25)
  WAYBAR_GCAL_THEME      CSS file appended after the built-in theme"
    );
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
