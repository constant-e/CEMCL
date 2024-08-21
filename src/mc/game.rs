use log::warn;
use std::cell::RefCell;
use std::fs;
use serde_json::{json, Value};
use crate::Config;
use crate::file_tools::{exists, list_dir};
use super::Game;

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
                seperated: RefCell::from(false),
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
                for item in json.as_array()? {
                    game.args = RefCell::from(String::from(item["args"].as_str()?));
                    game.description = RefCell::from(String::from(item["description"].as_str()?));
                    game.height = RefCell::from(String::from(item["height"].as_str()?));
                    game.java_path = RefCell::from(String::from(item["java_path"].as_str()?));
                    game.seperated = RefCell::from(item["seperated"].as_bool()?);
                    game.width = RefCell::from(String::from(item["width"].as_str()?));
                    game.xms = RefCell::from(String::from(item["xms"].as_str()?));
                    game.xmx = RefCell::from(String::from(item["xmx"].as_str()?));
                }
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
            "seperated": *game.seperated.borrow(),
            "width": *game.width.borrow(),
            "xms": *game.xms.borrow(),
            "xmx": *game.xmx.borrow(),
        }
    );
    fs::write(path, json.to_string()).ok()
}