//! CEMCL 入口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod dialogs;
mod downloader;
mod file_tools;
mod mc;

use app::App;
use dialogs::{login_dialog, add_game_dialog, edit_game_dialog};
use log::error;
use std::sync::{Arc, Mutex};
use std::thread;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    env_logger::builder()
        .filter_module(
            "cemcl",
            if cfg!(debug_assertions) {
                log::LevelFilter::Debug
            } else {
                log::LevelFilter::Info
            },
        )
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

    let ui = AppWindow::new()?;
    ui.show()?; // dialogs in app should show later than appwindow
    let app = Arc::new(Mutex::new(App::new(ui.as_weak()).unwrap()));
    let app_weak = Arc::downgrade(&app);

    let app_weak_clone = app_weak.clone();
    ui.on_click_add_acc_btn(move || {
        let app_weak_clone = app_weak_clone.clone();
        if let Err(e) = login_dialog(app_weak_clone) {
            error!("Failed to start add_acc. Reason: {e}.");
        }
    });

    let app_weak_clone = app_weak.clone();
    ui.on_click_add_game_btn(move || {
        let app_weak_clone = app_weak_clone.clone();
        if let Err(e) = slint::spawn_local(async move {
            if let Err(e) = add_game_dialog(app_weak_clone).await {
                error!("Failed to start add_game. Reason: {e}.");
            }
        }) {
            error!("Failed to call spawn_local. Reason: {e}.");
        }
    });

    let app_weak_clone = app_weak.clone();
    ui.on_switch_acc(move |index| {
        if app_weak_clone.upgrade().and_then(|app| {
            let mut app = app.lock().unwrap();
            app.current_acc_index = index as usize;
            if let Err(e) = app.save_acc_list() {
                error!("{e}");
            }
            Some(())
        }).is_none() {
            error!("Failed to upgrade a weak pointer.");
        }
    });

    let app_weak_clone = app_weak.clone();
    ui.on_del_acc(move |index| {
        if app_weak_clone.upgrade().and_then(|app| {
            let mut app = app.lock().unwrap();
            if let Err(e) = app.del_account(index as usize) {
                error!("Failed to delete account {index}.");
            }
            Some(())
        }).is_none() {
            error!("Failed to upgrade a weak pointer.");
        }
    });

    let app_weak_clone = app_weak.clone();
    ui.on_edit_acc(move |index, account| {
        if app_weak_clone.upgrade().and_then(|app| {
            let mut app = app.lock().unwrap();
            if let Err(e) = app.edit_account(index as usize, account.into()) {
                error!("Failed to edit account {index}. Reason: {e}");
            }
            Some(())
        }).is_none() {
            error!("Failed to upgrade a weak pointer.");
        }
    });

    let app_weak_clone = app_weak.clone();
    ui.on_click_edit_game_btn(move |index| {
        if let Err(e) = edit_game_dialog(app_weak_clone.clone()) {
            error!("Failed to start edit_game. Reason: {e}.");
        }
    });

    let app_weak_clone = app_weak.clone();
    ui.on_apply_settings(move || {
        if app_weak_clone.upgrade().and_then(|app| {
            let mut app = app.lock().unwrap();
            if let Err(e) = app.set_config() {
                error!("Failed to apply settings. Reason: {e}.");
            }
            Some(())
        }).is_none() {
            error!("Failed to upgrade a weak pointer.");
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_click_start_btn(move |game_index| {
        if let Some(ui) = ui_weak.upgrade() {
            let app_weak = app_weak.clone();
            let acc_index = ui.get_acc_index() as usize;
            thread::spawn(move || {
                if let Some(app) = app_weak.upgrade() {
                    if let Ok(mut app) = app.try_lock() {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        let _tokio = rt.enter();
                        rt.block_on(app.launch(acc_index, game_index as usize));
                    } else {
                        error!("Failed to lock a mutex.");
                    }
                } else {
                    error!("Failed to upgrade weak pointer.");
                }
            });
        } else {
            error!("Failed to upgrade weak pointer.");
        }
    });

    slint::run_event_loop()?;
    ui.hide()
}
