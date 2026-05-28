mod agenda;
mod app;
mod cache;
mod cli;
mod date;
mod google;
mod model;
mod month;
mod paths;
mod single_instance;
mod theme;
mod ui;

use cli::CliCommand;
use std::env;

fn main() {
    match cli::parse_args(env::args().skip(1).collect()) {
        Ok(CliCommand::Help) => cli::print_help(),
        Ok(CliCommand::PrintTheme) => print!("{}", theme::builtin_css()),
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
            cli::print_help();
            std::process::exit(2);
        }
    }
}
