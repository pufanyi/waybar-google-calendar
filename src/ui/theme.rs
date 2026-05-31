use crate::storage::paths;
use gtk::gdk;
use std::cell::RefCell;
use std::fs;
use std::path::Path;

const BUILTIN_CSS: &str = include_str!("../../assets/themes/apple-light.css");

thread_local! {
    static ACTIVE_PROVIDER: RefCell<Option<gtk::CssProvider>> = const { RefCell::new(None) };
}

pub fn builtin_css() -> &'static str {
    BUILTIN_CSS
}

pub fn load_css(explicit_path: Option<&Path>) -> Result<String, String> {
    let mut css = String::from(BUILTIN_CSS);

    if let Some(path) = explicit_path {
        append_theme_file(&mut css, path)?;
        return Ok(css);
    }

    let path = paths::config_theme_file();
    if path.exists() {
        append_theme_file(&mut css, &path)?;
    }

    Ok(css)
}

pub fn apply_css(css: &str) {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(css);
    if let Some(display) = gdk::Display::default() {
        ACTIVE_PROVIDER.with(|active_provider| {
            let mut active_provider = active_provider.borrow_mut();
            if let Some(provider) = active_provider.take() {
                gtk::style_context_remove_provider_for_display(&display, &provider);
            }
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
            *active_provider = Some(provider);
        });
    }
}

fn append_theme_file(css: &mut String, path: &Path) -> Result<(), String> {
    let custom = fs::read_to_string(path)
        .map_err(|err| format!("Could not read theme CSS {}: {err}", path.display()))?;
    css.push_str("\n\n/* User theme overrides */\n");
    css.push_str(&custom);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    struct EnvGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
        original_config: Option<std::ffi::OsString>,
        temp_dir: PathBuf,
    }

    impl EnvGuard {
        fn new(name: &str) -> Self {
            let lock = crate::test_env::ENV_LOCK
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let original_config = env::var_os("XDG_CONFIG_HOME");
            let temp_dir = env::temp_dir().join(format!("gcal-test-theme-{}", name));
            let _ = fs::remove_dir_all(&temp_dir);
            let _ = fs::create_dir_all(&temp_dir);
            unsafe {
                env::set_var("XDG_CONFIG_HOME", &temp_dir);
            }
            Self {
                _lock: lock,
                original_config,
                temp_dir,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            unsafe {
                if let Some(v) = &self.original_config {
                    env::set_var("XDG_CONFIG_HOME", v);
                } else {
                    env::remove_var("XDG_CONFIG_HOME");
                }
            }
            let _ = fs::remove_dir_all(&self.temp_dir);
        }
    }

    #[test]
    fn test_builtin_css() {
        assert!(builtin_css().contains("window"));
        assert!(builtin_css().contains("calendar"));
    }

    #[test]
    fn test_load_css_no_custom() {
        let _guard = EnvGuard::new("no-custom");
        let css = load_css(None).unwrap();
        assert_eq!(css, builtin_css());
    }

    #[test]
    fn test_load_css_explicit_path() {
        let _guard = EnvGuard::new("explicit");
        let custom_file = _guard.temp_dir.join("custom.css");
        fs::write(&custom_file, ".my-class { color: red; }").unwrap();

        let css = load_css(Some(&custom_file)).unwrap();
        assert!(css.contains(".my-class { color: red; }"));
        assert!(css.contains("/* User theme overrides */"));
    }

    #[test]
    fn test_load_css_config_theme_file() {
        let _guard = EnvGuard::new("config-theme");
        let custom_theme_dir = _guard.temp_dir.join("waybar-google-calendar");
        fs::create_dir_all(&custom_theme_dir).unwrap();
        let custom_file = custom_theme_dir.join("style.css");
        fs::write(&custom_file, ".config-class { color: blue; }").unwrap();

        let css = load_css(None).unwrap();
        assert!(css.contains(".config-class { color: blue; }"));
    }

    #[test]
    fn test_load_css_invalid_file() {
        let _guard = EnvGuard::new("invalid");
        let non_existent = _guard.temp_dir.join("nonexistent.css");
        assert!(load_css(Some(&non_existent)).is_err());
    }
}
