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
    load_css_with_config_theme(explicit_path, &paths::config_theme_file())
}

fn load_css_with_config_theme(
    explicit_path: Option<&Path>,
    config_theme_file: &Path,
) -> Result<String, String> {
    let mut css = String::from(BUILTIN_CSS);

    if let Some(path) = explicit_path {
        append_theme_file(&mut css, path)?;
        return Ok(css);
    }

    if config_theme_file.exists() {
        append_theme_file(&mut css, config_theme_file)?;
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
    use std::fs;
    use std::path::PathBuf;

    fn temp_theme_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("gcal-test-theme-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_builtin_css() {
        assert!(builtin_css().contains("window"));
        assert!(builtin_css().contains("calendar"));
    }

    #[test]
    fn test_load_css_no_custom() {
        let dir = temp_theme_dir("no-custom");
        let css = load_css_with_config_theme(None, &dir.join("missing.css")).unwrap();
        let _ = fs::remove_dir_all(dir);
        assert_eq!(css, builtin_css());
    }

    #[test]
    fn test_load_css_explicit_path() {
        let dir = temp_theme_dir("explicit");
        let custom_file = dir.join("custom.css");
        fs::write(&custom_file, ".my-class { color: red; }").unwrap();

        let css = load_css(Some(&custom_file)).unwrap();
        let _ = fs::remove_dir_all(dir);
        assert!(css.contains(".my-class { color: red; }"));
        assert!(css.contains("/* User theme overrides */"));
    }

    #[test]
    fn test_load_css_config_theme_file() {
        let dir = temp_theme_dir("config-theme");
        let custom_file = dir.join("style.css");
        fs::write(&custom_file, ".config-class { color: blue; }").unwrap();

        let css = load_css_with_config_theme(None, &custom_file).unwrap();
        let _ = fs::remove_dir_all(dir);
        assert!(css.contains(".config-class { color: blue; }"));
    }

    #[test]
    fn test_load_css_invalid_file() {
        let dir = temp_theme_dir("invalid");
        let non_existent = dir.join("nonexistent.css");
        assert!(load_css(Some(&non_existent)).is_err());
        let _ = fs::remove_dir_all(dir);
    }
}
