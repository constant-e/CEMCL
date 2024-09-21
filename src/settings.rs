//! settings 设置
use log::error;
use std::{cell::RefCell, rc::Rc};
use slint::ComponentHandle;

use crate::{game, mc::Game, save_config, ui_game_list, AppWindow, Config, Settings};

pub fn init(config: &Rc<Config>, game_list: &Rc<RefCell<Vec<Game>>>, app: &AppWindow) -> Option<()> {
    let ui = Settings::new().ok()?;

    // init
    ui.set_assets_source(config.assets_source.borrow().clone().into());
    ui.set_authors(env!("CARGO_PKG_AUTHORS").into());
    ui.set_close_after_launch(config.close_after_launch.borrow().clone());
    ui.set_config_height(config.height.borrow().clone().into());
    ui.set_config_width(config.width.borrow().clone().into());
    ui.set_fabric_source(config.fabric_source.borrow().clone().into());
    ui.set_forge_source(config.forge_source.borrow().clone().into());
    ui.set_game_path(config.game_path.borrow().clone().into());
    ui.set_game_source(config.game_source.borrow().clone().into());
    ui.set_java_path(config.java_path.borrow().clone().into());
    ui.set_libraries_source(config.libraries_source.borrow().clone().into());
    ui.set_optifine_source(config.optifine_source.borrow().clone().into());
    ui.set_version(env!("CARGO_PKG_VERSION").into());
    ui.set_xms(config.xms.borrow().clone().into());
    ui.set_xmx(config.xmx.borrow().clone().into());
    
    ui.on_apply_clicked({
        let app_handle = app.as_weak();
        let ui_handle = ui.as_weak();
        let config_handle = Rc::downgrade(config);
        let game_list_handle = Rc::downgrade(game_list);
        move || {
            if let (Some(app), Some(ui), Some(config), Some(game_list)) =
                (app_handle.upgrade(), ui_handle.upgrade(), config_handle.upgrade(), game_list_handle.upgrade())
            {
                *config.assets_source.borrow_mut() = ui.get_assets_source().into();
                *config.close_after_launch.borrow_mut() = ui.get_close_after_launch();
                *config.fabric_source.borrow_mut() = ui.get_fabric_source().into();
                *config.forge_source.borrow_mut() = ui.get_forge_source().into();
                *config.game_path.borrow_mut() = ui.get_game_path().into();
                *config.game_source.borrow_mut() = ui.get_game_source().into();
                *config.height.borrow_mut() = ui.get_config_height().into();
                *config.java_path.borrow_mut() = ui.get_java_path().into();
                *config.libraries_source.borrow_mut() = ui.get_libraries_source().into();
                *config.optifine_source.borrow_mut() = ui.get_optifine_source().into();
                *config.width.borrow_mut() = ui.get_config_width().into();
                *config.xms.borrow_mut() = ui.get_xms().into();
                *config.xmx.borrow_mut() = ui.get_xmx().into();
                save_config(&config);
                *game_list.borrow_mut() = game::load(&config).unwrap();
                app.set_game_list(ui_game_list(game_list.borrow().as_ref()));
                ui.hide().unwrap();
            } else {
                error!("Failed to get config.");
            }
        }
    });

    ui.on_cancel_clicked({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.hide().unwrap();
        }
    });

    ui.show().ok()
}
