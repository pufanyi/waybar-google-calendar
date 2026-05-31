use crate::calendar::model::Mode;
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

pub fn cache_file(name: &str) -> PathBuf {
    cache_dir().join(format!("agenda-{name}.json"))
}

pub fn cache_dir() -> PathBuf {
    cache_dir_from(|name| env::var_os(name), env::temp_dir())
}

pub fn config_dir() -> PathBuf {
    config_dir_from(|name| env::var_os(name), env::temp_dir())
}

pub fn config_theme_file() -> PathBuf {
    config_dir().join("style.css")
}

pub fn settings_file() -> PathBuf {
    config_dir().join("config.json")
}

pub fn client_secret_file() -> PathBuf {
    client_secret_file_from(|name| env::var_os(name), env::temp_dir())
}

pub fn data_dir() -> PathBuf {
    data_dir_from(|name| env::var_os(name), env::temp_dir())
}

pub fn oauth_token_file() -> PathBuf {
    data_dir().join("oauth-token.json")
}

pub fn pid_file(mode: Mode) -> PathBuf {
    pid_file_from(mode, |name| env::var_os(name), env::temp_dir())
}

fn cache_dir_from<F>(env_var: F, temp_dir: PathBuf) -> PathBuf
where
    F: Fn(&str) -> Option<OsString>,
{
    env_var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| env_var("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .unwrap_or(temp_dir)
        .join("waybar-google-calendar")
}

fn config_dir_from<F>(env_var: F, temp_dir: PathBuf) -> PathBuf
where
    F: Fn(&str) -> Option<OsString>,
{
    env_var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env_var("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or(temp_dir)
        .join("waybar-google-calendar")
}

fn client_secret_file_from<F>(env_var: F, temp_dir: PathBuf) -> PathBuf
where
    F: Fn(&str) -> Option<OsString> + Copy,
{
    env_var("WAYBAR_GCAL_CLIENT_SECRET")
        .map(PathBuf::from)
        .unwrap_or_else(|| config_dir_from(env_var, temp_dir).join("client_secret.json"))
}

fn data_dir_from<F>(env_var: F, temp_dir: PathBuf) -> PathBuf
where
    F: Fn(&str) -> Option<OsString>,
{
    env_var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| env_var("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .unwrap_or(temp_dir)
        .join("waybar-google-calendar")
}

fn pid_file_from<F>(mode: Mode, env_var: F, temp_dir: PathBuf) -> PathBuf
where
    F: Fn(&str) -> Option<OsString>,
{
    let suffix = match mode {
        Mode::Agenda => "agenda",
        Mode::Month => "month",
        Mode::Auth => "auth",
    };
    env_var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or(temp_dir)
        .join(format!("waybar-google-calendar-{suffix}.pid"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::model::Mode;

    fn env_fixture<'a>(
        vars: &'a [(&'a str, &'a str)],
    ) -> impl Fn(&str) -> Option<OsString> + Copy + 'a {
        move |name| {
            vars.iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| OsString::from(value))
        }
    }

    #[test]
    fn test_cache_dir_xdg() {
        assert_eq!(
            cache_dir_from(
                env_fixture(&[("XDG_CACHE_HOME", "/tmp/custom_cache")]),
                PathBuf::from("/tmp")
            ),
            PathBuf::from("/tmp/custom_cache/waybar-google-calendar")
        );
        assert_eq!(
            cache_dir_from(
                env_fixture(&[("XDG_CACHE_HOME", "/tmp/custom_cache")]),
                PathBuf::from("/tmp")
            )
            .join("agenda-test.json"),
            PathBuf::from("/tmp/custom_cache/waybar-google-calendar/agenda-test.json"),
        );
    }

    #[test]
    fn test_cache_dir_fallback() {
        assert_eq!(
            cache_dir_from(env_fixture(&[("HOME", "/tmp/home")]), PathBuf::from("/tmp")),
            PathBuf::from("/tmp/home/.cache/waybar-google-calendar")
        );
    }

    #[test]
    fn test_config_dir_xdg() {
        let config_dir = config_dir_from(
            env_fixture(&[("XDG_CONFIG_HOME", "/tmp/custom_config")]),
            PathBuf::from("/tmp"),
        );
        assert_eq!(
            config_dir,
            PathBuf::from("/tmp/custom_config/waybar-google-calendar")
        );
        assert_eq!(
            config_dir.join("style.css"),
            PathBuf::from("/tmp/custom_config/waybar-google-calendar/style.css")
        );
        assert_eq!(
            config_dir.join("config.json"),
            PathBuf::from("/tmp/custom_config/waybar-google-calendar/config.json")
        );
    }

    #[test]
    fn test_config_dir_fallback() {
        assert_eq!(
            config_dir_from(env_fixture(&[("HOME", "/tmp/home")]), PathBuf::from("/tmp")),
            PathBuf::from("/tmp/home/.config/waybar-google-calendar")
        );
    }

    #[test]
    fn test_client_secret_env() {
        assert_eq!(
            client_secret_file_from(
                env_fixture(&[("WAYBAR_GCAL_CLIENT_SECRET", "/tmp/custom_secret.json")]),
                PathBuf::from("/tmp"),
            ),
            PathBuf::from("/tmp/custom_secret.json")
        );
    }

    #[test]
    fn test_client_secret_fallback() {
        assert_eq!(
            client_secret_file_from(
                env_fixture(&[("XDG_CONFIG_HOME", "/tmp/custom_config")]),
                PathBuf::from("/tmp"),
            ),
            PathBuf::from("/tmp/custom_config/waybar-google-calendar/client_secret.json")
        );
    }

    #[test]
    fn test_data_dir_xdg() {
        let data_dir = data_dir_from(
            env_fixture(&[("XDG_DATA_HOME", "/tmp/custom_data")]),
            PathBuf::from("/tmp"),
        );
        assert_eq!(
            data_dir,
            PathBuf::from("/tmp/custom_data/waybar-google-calendar")
        );
        assert_eq!(
            data_dir.join("oauth-token.json"),
            PathBuf::from("/tmp/custom_data/waybar-google-calendar/oauth-token.json")
        );
    }

    #[test]
    fn test_data_dir_fallback() {
        assert_eq!(
            data_dir_from(env_fixture(&[("HOME", "/tmp/home")]), PathBuf::from("/tmp")),
            PathBuf::from("/tmp/home/.local/share/waybar-google-calendar")
        );
    }

    #[test]
    fn test_pid_file() {
        let env = env_fixture(&[("XDG_RUNTIME_DIR", "/tmp/runtime")]);
        assert_eq!(
            pid_file_from(Mode::Agenda, env, PathBuf::from("/tmp")),
            PathBuf::from("/tmp/runtime/waybar-google-calendar-agenda.pid")
        );
        assert_eq!(
            pid_file_from(Mode::Month, env, PathBuf::from("/tmp")),
            PathBuf::from("/tmp/runtime/waybar-google-calendar-month.pid")
        );
        assert_eq!(
            pid_file_from(Mode::Auth, env, PathBuf::from("/tmp")),
            PathBuf::from("/tmp/runtime/waybar-google-calendar-auth.pid")
        );
    }
}
