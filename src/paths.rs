use crate::model::Mode;
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

pub fn config_theme_file() -> PathBuf {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(env::temp_dir)
        .join("waybar-google-calendar")
        .join("style.css")
}

pub fn pid_file(mode: Mode) -> PathBuf {
    let suffix = match mode {
        Mode::Agenda => "agenda",
        Mode::Month => "month",
    };
    env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir)
        .join(format!("waybar-google-calendar-{suffix}.pid"))
}
