#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod account;
mod errors;
mod runtime;
mod version;

use log::error;

pub use errors::LauncherError;
use frontend::AppWindow;
use runtime::AppRuntime;

#[tokio::main]
async fn main() {
    // initialize
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    if let Some(mut path) = std::env::home_dir() {
        let cemcl_path = path.join(".cemcl");
        if !cemcl_path.exists() {
            if let Err(e) = std::fs::create_dir(&cemcl_path) {
                error!("Failed to create directory. Reason: {e}.");
            } else {
                path = cemcl_path;
            }
        } else {
            path = cemcl_path;
        }

        if let Err(e) = std::env::set_current_dir(&path) {
            error!("Failed to set current directory. Reason: {e}.");
        }
    } else {
        error!("Failed to get home directory.");
    }

    let mut app_window = AppWindow::new().unwrap();
    let update_sender = app_window.get_update_sender();
    let cmd_receiver = app_window.take_cmd_receiver().unwrap();
    let mut rt = AppRuntime::new(update_sender, cmd_receiver).unwrap();

    tokio::spawn(async move {
        if let Err(e) = rt.run().await {
            error!("{e}")
        }
    });

    app_window.run().unwrap();
}
