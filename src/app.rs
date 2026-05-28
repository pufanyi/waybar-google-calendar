use crate::model::{Config, Mode};
use crate::{agenda, month, paths, single_instance, theme};
use adw::prelude::*;
use gtk::gio;
use std::fs;

const APP_ID: &str = "io.github.pufanyi.waybar_google_calendar";

pub fn run(config: Config) -> Result<(), String> {
    single_instance::toggle_existing_instance(config.mode)?;
    let css = theme::load_css(config.theme_path.as_deref())?;

    let pid_file = paths::pid_file(config.mode);
    fs::write(&pid_file, std::process::id().to_string())
        .map_err(|err| format!("Could not write pid file {}: {err}", pid_file.display()))?;

    let app_id = match config.mode {
        Mode::Agenda => format!("{APP_ID}.agenda"),
        Mode::Month => format!("{APP_ID}.month"),
    };

    let app = adw::Application::builder()
        .application_id(&app_id)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();

    let pid_file_for_shutdown = pid_file.clone();
    app.connect_shutdown(move |_| {
        let _ = fs::remove_file(&pid_file_for_shutdown);
    });

    app.connect_activate(move |app| {
        theme::apply_css(&css);
        match config.mode {
            Mode::Agenda => agenda::build_window(app, config.days),
            Mode::Month => month::build_window(app),
        }
    });

    app.run_with_args::<&str>(&[]);
    let _ = fs::remove_file(pid_file);
    Ok(())
}
