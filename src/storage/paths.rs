use crate::calendar::model::Mode;
use std::env;
use std::path::PathBuf;

pub fn cache_file(name: &str) -> PathBuf {
    cache_dir().join(format!("agenda-{name}.json"))
}

pub fn cache_dir() -> PathBuf {
    env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .unwrap_or_else(env::temp_dir)
        .join("waybar-google-calendar")
}

pub fn config_dir() -> PathBuf {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(env::temp_dir)
        .join("waybar-google-calendar")
}

pub fn config_theme_file() -> PathBuf {
    config_dir().join("style.css")
}

pub fn settings_file() -> PathBuf {
    config_dir().join("config.json")
}

pub fn client_secret_file() -> PathBuf {
    env::var_os("WAYBAR_GCAL_CLIENT_SECRET")
        .map(PathBuf::from)
        .unwrap_or_else(|| config_dir().join("client_secret.json"))
}

pub fn data_dir() -> PathBuf {
    env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .unwrap_or_else(env::temp_dir)
        .join("waybar-google-calendar")
}

pub fn oauth_token_file() -> PathBuf {
    data_dir().join("oauth-token.json")
}

pub fn pid_file(mode: Mode) -> PathBuf {
    let suffix = match mode {
        Mode::Agenda => "agenda",
        Mode::Month => "month",
        Mode::Auth => "auth",
    };
    env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir)
        .join(format!("waybar-google-calendar-{suffix}.pid"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::model::Mode;
    use std::sync::LazyLock;
    use std::sync::Mutex;

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    struct EnvGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
        vars: Vec<(&'static str, Option<std::ffi::OsString>)>,
    }

    impl EnvGuard {
        fn new(keys: &[&'static str]) -> Self {
            let lock = ENV_LOCK.lock().unwrap();
            let vars = keys.iter().map(|&key| (key, env::var_os(key))).collect();
            Self { _lock: lock, vars }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, val) in &self.vars {
                unsafe {
                    if let Some(v) = val {
                        env::set_var(key, v);
                    } else {
                        env::remove_var(key);
                    }
                }
            }
        }
    }

    #[test]
    fn test_cache_dir_xdg() {
        let _guard = EnvGuard::new(&["XDG_CACHE_HOME", "HOME"]);
        unsafe {
            env::set_var("XDG_CACHE_HOME", "/tmp/custom_cache");
        }
        assert_eq!(
            cache_dir(),
            PathBuf::from("/tmp/custom_cache/waybar-google-calendar")
        );
        assert_eq!(
            cache_file("test"),
            PathBuf::from("/tmp/custom_cache/waybar-google-calendar/agenda-test.json")
        );
    }

    #[test]
    fn test_cache_dir_fallback() {
        let _guard = EnvGuard::new(&["XDG_CACHE_HOME", "HOME"]);
        unsafe {
            env::remove_var("XDG_CACHE_HOME");
            env::set_var("HOME", "/tmp/home");
        }
        assert_eq!(
            cache_dir(),
            PathBuf::from("/tmp/home/.cache/waybar-google-calendar")
        );
    }

    #[test]
    fn test_config_dir_xdg() {
        let _guard = EnvGuard::new(&["XDG_CONFIG_HOME", "HOME"]);
        unsafe {
            env::set_var("XDG_CONFIG_HOME", "/tmp/custom_config");
        }
        assert_eq!(
            config_dir(),
            PathBuf::from("/tmp/custom_config/waybar-google-calendar")
        );
        assert_eq!(
            config_theme_file(),
            PathBuf::from("/tmp/custom_config/waybar-google-calendar/style.css")
        );
        assert_eq!(
            settings_file(),
            PathBuf::from("/tmp/custom_config/waybar-google-calendar/config.json")
        );
    }

    #[test]
    fn test_config_dir_fallback() {
        let _guard = EnvGuard::new(&["XDG_CONFIG_HOME", "HOME"]);
        unsafe {
            env::remove_var("XDG_CONFIG_HOME");
            env::set_var("HOME", "/tmp/home");
        }
        assert_eq!(
            config_dir(),
            PathBuf::from("/tmp/home/.config/waybar-google-calendar")
        );
    }

    #[test]
    fn test_client_secret_env() {
        let _guard = EnvGuard::new(&["WAYBAR_GCAL_CLIENT_SECRET"]);
        unsafe {
            env::set_var("WAYBAR_GCAL_CLIENT_SECRET", "/tmp/custom_secret.json");
        }
        assert_eq!(
            client_secret_file(),
            PathBuf::from("/tmp/custom_secret.json")
        );
    }

    #[test]
    fn test_client_secret_fallback() {
        let _guard = EnvGuard::new(&["WAYBAR_GCAL_CLIENT_SECRET", "XDG_CONFIG_HOME", "HOME"]);
        unsafe {
            env::remove_var("WAYBAR_GCAL_CLIENT_SECRET");
            env::set_var("XDG_CONFIG_HOME", "/tmp/custom_config");
        }
        assert_eq!(
            client_secret_file(),
            PathBuf::from("/tmp/custom_config/waybar-google-calendar/client_secret.json")
        );
    }

    #[test]
    fn test_data_dir_xdg() {
        let _guard = EnvGuard::new(&["XDG_DATA_HOME", "HOME"]);
        unsafe {
            env::set_var("XDG_DATA_HOME", "/tmp/custom_data");
        }
        assert_eq!(
            data_dir(),
            PathBuf::from("/tmp/custom_data/waybar-google-calendar")
        );
        assert_eq!(
            oauth_token_file(),
            PathBuf::from("/tmp/custom_data/waybar-google-calendar/oauth-token.json")
        );
    }

    #[test]
    fn test_data_dir_fallback() {
        let _guard = EnvGuard::new(&["XDG_DATA_HOME", "HOME"]);
        unsafe {
            env::remove_var("XDG_DATA_HOME");
            env::set_var("HOME", "/tmp/home");
        }
        assert_eq!(
            data_dir(),
            PathBuf::from("/tmp/home/.local/share/waybar-google-calendar")
        );
    }

    #[test]
    fn test_pid_file() {
        let _guard = EnvGuard::new(&["XDG_RUNTIME_DIR"]);
        unsafe {
            env::set_var("XDG_RUNTIME_DIR", "/tmp/runtime");
        }
        assert_eq!(
            pid_file(Mode::Agenda),
            PathBuf::from("/tmp/runtime/waybar-google-calendar-agenda.pid")
        );
        assert_eq!(
            pid_file(Mode::Month),
            PathBuf::from("/tmp/runtime/waybar-google-calendar-month.pid")
        );
        assert_eq!(
            pid_file(Mode::Auth),
            PathBuf::from("/tmp/runtime/waybar-google-calendar-auth.pid")
        );
    }
}
