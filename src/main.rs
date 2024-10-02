//! CEMCL 入口

mod app;
mod dialogs;
mod file_tools;
mod mc;
mod settings;

use app::App;
use dialogs::{add_acc_dialog, add_game_dialog, edit_acc_dialog, edit_game_dialog};
use log::error;
use std::cell::RefCell;
use std::rc::Rc;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    env_logger::builder().filter_module("cemcl",
        if cfg!(debug_assertions) { log::LevelFilter::Debug } else { log::LevelFilter::Info }
    ).init();

    let ui = AppWindow::new()?;
    ui.show()?; // dialogs in app should show later than appwindow
    let app = Rc::new(RefCell::new(App::new(ui.as_weak()).unwrap()));
    let app_weak = Rc::downgrade(&app);

    let app_weak_clone = app_weak.clone();
    ui.on_click_add_acc_btn(move || {
        if let Err(e) = add_acc_dialog(app_weak_clone.clone()) {
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
    ui.on_click_edit_acc_btn(move || {
        if let Err(e) = edit_acc_dialog(app_weak_clone.clone()) {
            error!("Failed to start edit_acc. Reason: {e}.");
        }
    });

    let app_weak_clone = app_weak.clone();
    ui.on_click_edit_game_btn(move || {
        if let Err(e) = edit_game_dialog(app_weak_clone.clone()) {
            error!("Failed to start edit_game. Reason: {e}.");
        }
    });

    let app_weak_clone = app_weak.clone();
    ui.on_click_settings_btn(move || {
        settings::init(app_weak_clone.clone());
    });
    
    ui.on_click_start_btn(move || {
        if let Some(app) = app_weak.upgrade() {
            app.borrow_mut().launch();
        } else {
            error!("Failed to upgrade weak pointer.");
        }
    });

    slint::run_event_loop()?;
    return ui.hide();
}
