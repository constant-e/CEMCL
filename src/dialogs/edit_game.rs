//! 修改MC版本

use std::sync::{self, Mutex};
use log::error;
use slint::ComponentHandle;

use crate::{app::App, dialogs::msg_box::err_dialog, EditGameDialog, Messages};

use super::msg_box::ask_dialog;

pub fn edit_game_dialog(app_weak: sync::Weak<Mutex<App>>) -> Result<(), slint::PlatformError> {
    let ui = EditGameDialog::new()?;
    let ui_weak = ui.as_weak();

    let index = if let Some(app) = app_weak.upgrade() {
        if let Ok(app) = app.lock() {
            if let Some(index) = app.get_game_index() {
                let game = &app.game_list[index];
                ui.set_config_height(game.height.clone().into());
                ui.set_config_width(game.width.clone().into());
                ui.set_description(game.description.clone().into());
                ui.set_java_path(game.java_path.clone().into());
                ui.set_separated(game.separated.clone());
                ui.set_version(game.version.clone().into());
                ui.set_xms(game.xms.clone().into());
                ui.set_xmx(game.xmx.clone().into());
                index
            } else {
                err_dialog(&app.ui_weak.upgrade()
                    .ok_or(slint::PlatformError::Other(String::from("Failed to upgrade a weak pointer")))?
                    .global::<Messages>().get_game_not_selected());
                return Err(slint::PlatformError::Other(String::from("Failed to get the index of game_list")));
            }
        } else {
            error!("Failed to lock a mutex.");
            return Err(slint::PlatformError::Other(String::from("Failed to lock a mutex")));
        }
    } else {
        error!("Failed to upgrade a weak pointer.");
        return Err(slint::PlatformError::Other(String::from("Failed to upgrade a weak pointer")));
    };
    
    let app_weak_clone = app_weak.clone();
    let ui_weak_clone = ui_weak.clone();
    ui.on_ok_clicked(move || {
        if let (Some(app), Some(ui)) = (app_weak_clone.upgrade(), ui_weak_clone.upgrade()) {
            if let Ok(mut app) = app.lock() {
                let mut game = app.game_list[index].clone();
                game.description = ui.get_description().into();
                game.height = ui.get_config_height().into();
                game.java_path = ui.get_java_path().into();
                game.separated = ui.get_separated();
                game.width = ui.get_config_width().into();
                game.xms = ui.get_xms().into();
                game.xmx = ui.get_xmx().into();

                app.edit_game(index, game.clone());
            } else {
                error!("Failed to lock a mutex.");
            }
            
            ui.hide().unwrap();
        } else {
            error!("Failed to upgrade a weak pointer.");
        }
    });

    let ui_weak_clone = ui_weak.clone();
    ui.on_cancel_clicked(move || {
        if let Some(ui) = ui_weak_clone.upgrade() {
            ui.hide().unwrap();
        } else {
            error!("Failed to upgrade a weak pointer.");
        }
    });

    ui.on_click_del_btn(move || {
        if let (Some(app), Some(ui)) = (app_weak.upgrade(), ui_weak.upgrade()) {
            if let Ok(app) = app.lock() {
                let (title, msg) = if let Some(app_ui) = app.ui_weak.upgrade() {
                    (app_ui.global::<Messages>().get_warn(), app_ui.global::<Messages>().get_del_game_confirm())
                } else {
                    error!("Failed to upgrade a weak pointer.");
                    return;
                };
                let app_weak = app_weak.clone();
                ask_dialog(&title, &msg, move || {
                    if let Some(app) = app_weak.upgrade() {
                        if let Ok(mut app) = app.lock() {
                            app.del_game(index);
                            ui.hide().unwrap();
                        }
                    }
                });
            } else {
                error!("Failed to lock a mutex.");
            }
        } else {
            error!("Failed to upgrade a weak pointer.");
        }
    });

    ui.show()
}
