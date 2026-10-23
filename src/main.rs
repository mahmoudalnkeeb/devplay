#![windows_subsystem = "windows"]

mod actions;
mod config;
mod docker;
mod log;
mod sys;
mod tray;

use actions::Action;

fn main() {
    let Some(arg) = std::env::args().nth(1) else {
        tray::run();
        return;
    };
    // A windows-subsystem app has no console, so errors go to the log.
    match arg.parse::<Action>() {
        Ok(action) => {
            action.run();
        }
        Err(err) => log::write(&err.to_string()),
    }
}
