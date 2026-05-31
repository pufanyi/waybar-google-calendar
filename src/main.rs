mod agenda;
mod app;
mod auth_ui;
mod calendar;
mod google;
mod i18n;
mod month;
mod storage;
mod ui;

use app::cli::CliCommand;
use std::env;

fn main() {
    match app::cli::parse_args(env::args().skip(1).collect()) {
        Ok(CliCommand::Help) => app::cli::print_help(),
        Ok(CliCommand::PrintTheme) => print!("{}", ui::theme::builtin_css()),
        Ok(CliCommand::Auth) => match google::auth_calendar() {
            Ok(()) => println!("Google Calendar authenticated."),
            Err(message) => {
                eprintln!("{message}");
                std::process::exit(1);
            }
        },
        Ok(CliCommand::Run(config)) => {
            if let Err(message) = app::run(config) {
                eprintln!("{message}");
                std::process::exit(1);
            }
        }
        Err(message) => {
            eprintln!("{message}");
            app::cli::print_help();
            std::process::exit(2);
        }
    }
}
