use crate::calendar::model::{Config, Mode};
use crate::storage::settings::{UserSettings, read_settings};
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
    if args.first().is_some_and(|arg| arg == "auth") {
        return Ok(CliCommand::Auth);
    }
    if args.first().is_some_and(|arg| arg == "print-theme") {
        return Ok(CliCommand::PrintTheme);
    }

    parse_args_with_sources(args, read_settings()?, env_string, env_path)
}

fn parse_args_with_sources<F, G>(
    args: Vec<String>,
    user_settings: UserSettings,
    env_string: F,
    env_path: G,
) -> Result<CliCommand, String>
where
    F: Fn(&str) -> Option<String>,
    G: Fn(&str) -> Option<PathBuf>,
{
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        return Ok(CliCommand::Help);
    }

    let mut mode = Mode::Agenda;
    let mut calendar = env_string("GCAL_CALENDAR")
        .filter(|value| !value.is_empty())
        .or(user_settings.calendar);
    let mut timezone = env_string("GCAL_TIMEZONE")
        .filter(|value| !value.is_empty())
        .or(user_settings.timezone);
    let mut theme_path = env_path("WAYBAR_GCAL_THEME")
        .filter(|value| !value.as_os_str().is_empty())
        .or(user_settings.theme_path);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::settings::{Language, WeekStart};
    use std::path::Path;

    fn settings() -> UserSettings {
        UserSettings {
            calendar: Some("saved-calendar".to_string()),
            timezone: Some("Asia/Singapore".to_string()),
            theme_path: Some(PathBuf::from("/saved/theme.css")),
            language: Some(Language::Chinese),
            week_start: Some(WeekStart::Monday),
        }
    }

    fn no_env_string(_: &str) -> Option<String> {
        None
    }

    fn no_env_path(_: &str) -> Option<PathBuf> {
        None
    }

    fn run_config(command: CliCommand) -> Config {
        match command {
            CliCommand::Run(config) => config,
            other => panic!("expected run command, got {other:?}"),
        }
    }

    #[test]
    fn saved_settings_are_defaults_for_agenda() {
        let config = run_config(
            parse_args_with_sources(vec![], settings(), no_env_string, no_env_path).unwrap(),
        );

        assert_eq!(config.mode, Mode::Agenda);
        assert_eq!(config.calendar.as_deref(), Some("saved-calendar"));
        assert_eq!(config.timezone.as_deref(), Some("Asia/Singapore"));
        assert_eq!(
            config.theme_path.as_deref(),
            Some(Path::new("/saved/theme.css"))
        );
    }

    #[test]
    fn environment_overrides_saved_settings() {
        let config = run_config(
            parse_args_with_sources(
                vec![],
                settings(),
                |name| match name {
                    "GCAL_CALENDAR" => Some("env-calendar".to_string()),
                    "GCAL_TIMEZONE" => Some("UTC".to_string()),
                    _ => None,
                },
                |name| match name {
                    "WAYBAR_GCAL_THEME" => Some(PathBuf::from("/env/theme.css")),
                    _ => None,
                },
            )
            .unwrap(),
        );

        assert_eq!(config.calendar.as_deref(), Some("env-calendar"));
        assert_eq!(config.timezone.as_deref(), Some("UTC"));
        assert_eq!(
            config.theme_path.as_deref(),
            Some(Path::new("/env/theme.css"))
        );
    }

    #[test]
    fn arguments_override_environment_and_saved_settings() {
        let config = run_config(
            parse_args_with_sources(
                vec![
                    "--calendar".to_string(),
                    "arg-calendar".to_string(),
                    "--timezone".to_string(),
                    "Europe/London".to_string(),
                    "--theme".to_string(),
                    "/arg/theme.css".to_string(),
                ],
                settings(),
                |name| match name {
                    "GCAL_CALENDAR" => Some("env-calendar".to_string()),
                    "GCAL_TIMEZONE" => Some("UTC".to_string()),
                    _ => None,
                },
                |name| match name {
                    "WAYBAR_GCAL_THEME" => Some(PathBuf::from("/env/theme.css")),
                    _ => None,
                },
            )
            .unwrap(),
        );

        assert_eq!(config.calendar.as_deref(), Some("arg-calendar"));
        assert_eq!(config.timezone.as_deref(), Some("Europe/London"));
        assert_eq!(
            config.theme_path.as_deref(),
            Some(Path::new("/arg/theme.css"))
        );
    }

    #[test]
    fn empty_environment_values_do_not_override_saved_settings() {
        let config = run_config(
            parse_args_with_sources(
                vec![],
                settings(),
                |name| match name {
                    "GCAL_CALENDAR" | "GCAL_TIMEZONE" => Some(String::new()),
                    _ => None,
                },
                |name| match name {
                    "WAYBAR_GCAL_THEME" => Some(PathBuf::new()),
                    _ => None,
                },
            )
            .unwrap(),
        );

        assert_eq!(config.calendar.as_deref(), Some("saved-calendar"));
        assert_eq!(config.timezone.as_deref(), Some("Asia/Singapore"));
        assert_eq!(
            config.theme_path.as_deref(),
            Some(Path::new("/saved/theme.css"))
        );
    }

    #[test]
    fn mode_commands_keep_saved_theme() {
        let config = run_config(
            parse_args_with_sources(
                vec!["month".to_string()],
                settings(),
                no_env_string,
                no_env_path,
            )
            .unwrap(),
        );

        assert_eq!(config.mode, Mode::Month);
        assert_eq!(
            config.theme_path.as_deref(),
            Some(Path::new("/saved/theme.css"))
        );
    }
}
