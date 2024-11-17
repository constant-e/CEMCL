//! 设置

use log::error;
use std::sync::{self, Mutex};
use slint::ComponentHandle;

use crate::{App, Settings};

pub fn init(app_weak: sync::Weak<Mutex<App>>) -> Option<()> {
    let app = app_weak.upgrade()?;
    let ui = Settings::new().ok()?;
    let ui_weak = ui.as_weak();

    // init
    if let Ok(app) = app.lock() {
        ui.set_assets_source(app.config.assets_source.clone().into());
        ui.set_authors(env!("CARGO_PKG_AUTHORS").into());
        ui.set_close_after_launch(app.config.close_after_launch.clone());
        ui.set_concurrency(app.config.concurrency as i32);
        ui.set_config_height(app.config.height.clone().into());
        ui.set_config_width(app.config.width.clone().into());
        ui.set_fabric_source(app.config.fabric_source.clone().into());
        ui.set_forge_source(app.config.forge_source.clone().into());
        ui.set_game_path(app.config.game_path.clone().into());
        ui.set_game_source(app.config.game_source.clone().into());
        ui.set_java_path(app.config.java_path.clone().into());
        ui.set_libraries_source(app.config.libraries_source.clone().into());
        ui.set_version(env!("CARGO_PKG_VERSION").into());
        ui.set_xms(app.config.xms.clone().into());
        ui.set_xmx(app.config.xmx.clone().into());
    } else {
        error!("Failed to lock a mutex.");
        return None;
    }
    
    let ui_weak_clone = ui_weak.clone();
    ui.on_apply_clicked(move || {
        if let (Some(app), Some(ui)) = (app_weak.upgrade(), ui_weak_clone.upgrade()) {
            if let Ok(mut app) = app.lock() {
                let old_game_path = app.config.game_path.clone();
                let new_game_path = ui.get_game_path().to_string();

                app.config.assets_source = ui.get_assets_source().into();
                app.config.close_after_launch = ui.get_close_after_launch();
                // In slint, it is between 0 and 100, so don't need to check.
                app.config.concurrency = ui.get_concurrency() as usize;
                app.config.fabric_source = ui.get_fabric_source().into();
                app.config.forge_source = ui.get_forge_source().into();
                app.config.game_path = new_game_path.clone();
                app.config.game_source = ui.get_game_source().into();
                app.config.height = ui.get_config_height().into();
                app.config.java_path = ui.get_java_path().into();
                app.config.libraries_source = ui.get_libraries_source().into();
                app.config.width = ui.get_config_width().into();
                app.config.xms = ui.get_xms().into();
                app.config.xmx = ui.get_xmx().into();
                
                app.save_config().unwrap();

                if old_game_path != new_game_path {
                    app.load_game_list().unwrap();
                    app.refresh_ui_game_list();
                }
            } else {
                error!("Failed to lock a mutex.");
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
