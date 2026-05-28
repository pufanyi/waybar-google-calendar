use crate::model::{Config, Mode};
use crate::{agenda, month, paths, single_instance, theme};
use adw::prelude::*;
use gtk::gio;
use relm4::RelmApp;
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

    app.connect_startup(move |_| {
        theme::apply_css(&css);
    });

    match config.mode {
        Mode::Agenda => {
            let relm: RelmApp<agenda::AgendaMsg> = RelmApp::from_app(app).with_args(Vec::new());
            relm.allow_multiple_instances(true);
            relm.run::<agenda::AgendaApp>(agenda::AgendaInit {
                query: crate::model::AgendaQuery {
                    days: config.days,
                    calendar: config.calendar,
                    timezone: config.timezone,
                },
            });
        }
        Mode::Month => {
            let relm: RelmApp<month::MonthMsg> = RelmApp::from_app(app).with_args(Vec::new());
            relm.allow_multiple_instances(true);
            relm.run::<month::MonthApp>(());
        }
    }

    let _ = fs::remove_file(pid_file);
    Ok(())
}
