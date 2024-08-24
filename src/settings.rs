use std::rc::Rc;
use slint::ComponentHandle;

use crate::{Config, Settings};

pub fn init(config: &Rc<Config>) -> Option<()> {
    let ui = Settings::new().ok()?;

    // init
    ui.set_authors(env!("CARGO_PKG_AUTHORS").into());
    ui.set_close_after_launch(config.close_after_launch.borrow().clone());
    ui.set_config_height(config.height.borrow().clone().into());
    ui.set_config_width(config.width.borrow().clone().into());
    ui.set_fabric_source(config.fabric_source.borrow().clone().into());
    ui.set_forge_source(config.forge_source.borrow().clone().into());
    ui.set_game_path(config.game_path.borrow().clone().into());
    ui.set_game_source(config.game_source.borrow().clone().into());
    ui.set_java_path(config.java_path.borrow().clone().into());
    ui.set_optifine_source(config.optifine_source.borrow().clone().into());
    ui.set_version(env!("CARGO_PKG_VERSION").into());
    ui.set_xms(config.xms.borrow().clone().into());
    ui.set_xmx(config.xmx.borrow().clone().into());
    
    ui.on_apply_clicked({
        let ui_handle = ui.as_weak();
        let config_handle = Rc::downgrade(config);
        move || {
            let ui = ui_handle.unwrap();
            if let Some(config) = config_handle.upgrade() {
                // TODO: Save changes
                *config.close_after_launch.borrow_mut() = ui.get_close_after_launch();
                *config.fabric_source.borrow_mut() = ui.get_fabric_source().into();
                *config.forge_source.borrow_mut() = ui.get_forge_source().into();
                *config.game_path.borrow_mut() = ui.get_game_path().into();
                *config.game_source.borrow_mut() = ui.get_game_source().into();
                *config.height.borrow_mut() = ui.get_config_height().into();
                *config.java_path.borrow_mut() = ui.get_java_path().into();
                *config.optifine_source.borrow_mut() = ui.get_optifine_source().into();
                *config.width.borrow_mut() = ui.get_config_width().into();
                *config.xms.borrow_mut() = ui.get_xms().into();
                *config.xmx.borrow_mut() = ui.get_xmx().into();
            } else {

            }
            ui.hide().unwrap();
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
