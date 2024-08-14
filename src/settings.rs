use crate::Config;

slint::include_modules!();

pub fn init(config: &Config) {
    let ui = Settings::new().unwrap();
    let config = config.clone();

    // init
    ui.set_authors(env!("CARGO_PKG_AUTHORS").into());
    ui.set_close_after_launch(config.close_after_launch.clone().into());
    ui.set_config_height(config.height.to_string().into());
    ui.set_config_width(config.width.to_string().into());
    ui.set_fabric_source(config.fabric_source.clone().into());
    ui.set_forge_source(config.forge_source.clone().into());
    ui.set_game_path(config.game_path.clone().into());
    ui.set_game_source(config.game_source.clone().into());
    ui.set_java_path(config.java_path.clone().into());
    ui.set_optifine_source(config.optifine_source.clone().into());
    ui.set_version(env!("CARGO_PKG_VERSION").into());
    ui.set_xms(config.xms.clone().into());
    ui.set_xmx(config.xmx.clone().into());
    
    ui.on_click_apply_btn({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            // TODO: Save changes
            ui.hide();
        }
    });

    ui.on_click_cancel_btn({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.hide();
        }
    });

    ui.show().unwrap();
}
