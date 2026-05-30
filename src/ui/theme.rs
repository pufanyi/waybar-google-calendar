use crate::storage::paths;
use gtk::gdk;
use std::fs;
use std::path::Path;

const BUILTIN_CSS: &str = include_str!("../../assets/themes/apple-light.css");

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
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn append_theme_file(css: &mut String, path: &Path) -> Result<(), String> {
    let custom = fs::read_to_string(path)
        .map_err(|err| format!("Could not read theme CSS {}: {err}", path.display()))?;
    css.push_str("\n\n/* User theme overrides */\n");
    css.push_str(&custom);
    Ok(())
}
