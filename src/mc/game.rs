use log::warn;
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;
use serde_json::{json, Value};
use slint::ComponentHandle;
use crate::{AddGameDialog, AppWindow, Config, EditGameDialog, ui_game_list};
use crate::dialogs::ask_dialog;
use crate::file_tools::{exists, list_dir};
use super::Game;

pub fn add_dialog(game_list: &Rc<RefCell<Vec<Game>>>, app: &AppWindow) {
    let ui = AddGameDialog::new().unwrap();
    
    ui.on_ok_clicked({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            // TODO: Save changes
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

    ui.show().unwrap();
}

pub fn edit_dialog(game_list: &Rc<RefCell<Vec<Game>>>, index: usize, game_path: &String, app: &AppWindow) {
    let ui = EditGameDialog::new().unwrap();
    let game = &game_list.borrow()[index.clone()];
    ui.set_args(game.args.borrow().clone().into());
    ui.set_config_height(game.height.borrow().clone().into());
    ui.set_config_width(game.width.borrow().clone().into());
    ui.set_description(game.description.borrow().clone().into());
    ui.set_java_path(game.java_path.borrow().clone().into());
    ui.set_separated(game.separated.borrow().clone());
    ui.set_xms(game.xms.borrow().clone().into());
    ui.set_xmx(game.xmx.borrow().clone().into());
    
    ui.on_ok_clicked({
        let app_handle = app.as_weak();
        let game_list_handle = Rc::downgrade(game_list);
        let game_path = game_path.clone();
        let ui_handle = ui.as_weak();
        move || {
            let game_list = game_list_handle.upgrade().unwrap();
            let app = app_handle.unwrap();
            let ui = ui_handle.unwrap();
            let game = &game_list.borrow()[index];
            *game.args.borrow_mut() = ui.get_args().into();
            *game.description.borrow_mut() = ui.get_description().into();
            *game.height.borrow_mut() = ui.get_config_height().into();
            *game.java_path.borrow_mut() = ui.get_java_path().into();
            *game.separated.borrow_mut() = ui.get_separated();
            *game.width.borrow_mut() = ui.get_config_width().into();
            *game.xms.borrow_mut() = ui.get_xms().into();
            *game.xmx.borrow_mut() = ui.get_xmx().into();
            let path = game_path.clone() + "/versions/" + game.version.borrow().as_ref();
            save(&path, game);
            app.set_game_list(ui_game_list(game_list.borrow().as_ref()));
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

    ui.on_click_del_btn({
        let app_handle = app.as_weak();
        let game_list_handle = Rc::downgrade(game_list);
        let game_path = game_path.clone();
        let ui_handle = ui.as_weak();
        move || {
            let app = app_handle.unwrap();
            let game_list = game_list_handle.upgrade().unwrap();
            let ui = ui_handle.unwrap();
            ask_dialog("Warning", "All the files under this game's dir will be deleted. Continue?", {
                let app_handle = app.as_weak();
                let game_list_handle = Rc::downgrade(&game_list);
                let ui_handle = ui.as_weak();
                let path = game_path.clone() + "/versions/" + &game_list.borrow()[index].version.borrow().as_ref();
                move || {
                    let app = app_handle.unwrap();
                    let game_list = game_list_handle.upgrade().unwrap();
                    let ui = ui_handle.unwrap();
                    fs::remove_dir_all(path.clone()).unwrap();
                    game_list.borrow_mut().remove(index);
                    app.set_game_list(ui_game_list(game_list.borrow().as_ref()));
                    ui.hide().unwrap();
                }
            });
        }
    });

    ui.show().unwrap();
}

pub fn load(config: &Config) -> Option<Vec<Game>> {
    let mut game_list: Vec<Game> = Vec::new();
    let dir = config.game_path.borrow().clone() + "/versions";

    if !exists(&dir) {
        // 空目录
        warn!("{dir} is empty.");
        return Some(game_list);
    }

    for version in list_dir(&dir)? {
        let mut game: Game;
        let path = dir.clone() + "/" + version.as_str();
        
        // 先加载原版json
        if let Ok(json) = serde_json::from_str::<Value>(&fs::read_to_string(&(path.clone() + "/" + &version.as_str() + ".json")).ok()?.as_str()) {
            game = Game {
                args: RefCell::from(String::from("")),
                description: RefCell::from(String::from("")),
                height: config.height.clone(),
                java_path: config.java_path.clone(),
                separated: RefCell::from(false),
                game_type: RefCell::from(String::from(json["type"].as_str()?)),
                version: RefCell::from(version),
                width: config.width.clone(),
                xms: config.xms.clone(),
                xmx: config.xmx.clone(),
            };
        } else {
            // 异常，跳过此次加载
            warn!("Failed to load {version}.json.");
            continue;
        }
        
        // 若config.json存在，覆盖原版json
        let cfg_path = path.clone() + "/" + "config.json";
        if exists(&cfg_path) {
            if let Ok(json) = serde_json::from_str::<Value>(&fs::read_to_string(&cfg_path).ok()?.as_str()) {
                game.args = RefCell::from(String::from(json["args"].as_str()?));
                game.description = RefCell::from(String::from(json["description"].as_str()?));
                game.height = RefCell::from(String::from(json["height"].as_str()?));
                game.java_path = RefCell::from(String::from(json["java_path"].as_str()?));
                game.separated = RefCell::from(json["separated"].as_bool()?);
                game.width = RefCell::from(String::from(json["width"].as_str()?));
                game.xms = RefCell::from(String::from(json["xms"].as_str()?));
                game.xmx = RefCell::from(String::from(json["xmx"].as_str()?));
            } else {
                warn!("Failed to load {cfg_path}.");
                continue;
            }
        }
        game_list.push(game);
    };
    Some(game_list)
}

fn save(path: &str, game: &Game) -> Option<()> {
    let json = json!(
        {
            "args": *game.args.borrow(),
            "description": *game.description.borrow(),
            "height": *game.height.borrow(),
            "java_path": *game.java_path.borrow(),
            "separated": *game.separated.borrow(),
            "width": *game.width.borrow(),
            "xms": *game.xms.borrow(),
            "xmx": *game.xmx.borrow(),
        }
    );
    fs::write(path, json.to_string()).ok()
}
