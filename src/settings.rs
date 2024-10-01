//! settings 设置
use log::error;
use std::{cell::RefCell, rc};
use slint::ComponentHandle;

use crate::{App, Settings};

pub fn init(app_weak: rc::Weak<RefCell<App>>) -> Option<()> {
    let app = app_weak.upgrade()?;
    let ui = Settings::new().ok()?;
    let ui_weak = ui.as_weak();

    // init
    ui.set_assets_source(app.borrow().config.assets_source.clone().into());
    ui.set_authors(env!("CARGO_PKG_AUTHORS").into());
    ui.set_close_after_launch(app.borrow().config.close_after_launch.clone());
    ui.set_config_height(app.borrow().config.height.clone().into());
    ui.set_config_width(app.borrow().config.width.clone().into());
    ui.set_fabric_source(app.borrow().config.fabric_source.clone().into());
    ui.set_forge_source(app.borrow().config.forge_source.clone().into());
    ui.set_game_path(app.borrow().config.game_path.clone().into());
    ui.set_game_source(app.borrow().config.game_source.clone().into());
    ui.set_java_path(app.borrow().config.java_path.clone().into());
    ui.set_libraries_source(app.borrow().config.libraries_source.clone().into());
    ui.set_version(env!("CARGO_PKG_VERSION").into());
    ui.set_xms(app.borrow().config.xms.clone().into());
    ui.set_xmx(app.borrow().config.xmx.clone().into());
    
    let ui_weak_clone = ui_weak.clone();
    ui.on_apply_clicked(move || {
        if let (Some(app), Some(ui)) = (app_weak.upgrade(), ui_weak_clone.upgrade())
        {
            let old_game_path = app.borrow().config.game_path.clone();
            let new_game_path = ui.get_game_path().to_string();

            app.borrow_mut().config.assets_source = ui.get_assets_source().into();
            app.borrow_mut().config.close_after_launch = ui.get_close_after_launch();
            app.borrow_mut().config.fabric_source = ui.get_fabric_source().into();
            app.borrow_mut().config.forge_source = ui.get_forge_source().into();
            app.borrow_mut().config.game_path = new_game_path.clone();
            app.borrow_mut().config.game_source = ui.get_game_source().into();
            app.borrow_mut().config.height = ui.get_config_height().into();
            app.borrow_mut().config.java_path = ui.get_java_path().into();
            app.borrow_mut().config.libraries_source = ui.get_libraries_source().into();
            app.borrow_mut().config.width = ui.get_config_width().into();
            app.borrow_mut().config.xms = ui.get_xms().into();
            app.borrow_mut().config.xmx = ui.get_xmx().into();
            
            app.borrow().save_config().unwrap();

            if old_game_path != new_game_path {
                app.borrow_mut().load_game_list().unwrap();
                app.borrow_mut().refresh_ui_game_list();
            }
            
            ui.hide().unwrap();
        } else {
            error!("Failed to upgrade a weak pointer.");
        }
    });

    ui.on_cancel_clicked(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.hide().unwrap();
        } else {
            error!("Failed to upgrade a weak pointer");
        }
    });

    ui.show().ok()
}
