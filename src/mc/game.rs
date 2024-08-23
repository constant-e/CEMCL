use log::warn;
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;
use serde_json::{json, Value};
use crate::Config;
use crate::file_tools::{exists, list_dir};
use super::Game;

slint::include_modules!();

pub fn add_dialog(game_list: &Rc<RefCell<Vec<Game>>>, app: &crate::AppWindow) {
    let ui = AddGameDialog::new().unwrap();
    
    ui.on_ok_clicked({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            // TODO: Save changes
            ui.hide();
        }
    });

    ui.on_cancel_clicked({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.hide();
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
