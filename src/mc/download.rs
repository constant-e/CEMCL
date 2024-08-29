//! mc::download.rs 下载文件

use std::fs;
use std::rc::Rc;
use serde_json::Value;

use crate::{Config, Game};
use super::GameUrl;

fn download_assets() {

}

fn download_libraries() {

}

/// 下载MC入口
pub fn download_game(config: &Rc<Config>, version: &str) -> Option<()> {
    let game_path = config.game_path.borrow().clone() + "/versions/" + version;
    let json = serde_json::from_str::<Value>(fs::read_to_string(&(game_path.clone() + "/" + version + ".json")).ok()?.as_str()).ok()?;

    // 下载游戏本体jar

    
    Some(())
}

/// 初始化版本（创建json文件）
pub fn init_game(path: &str, version: &str, url: &str) -> Option<()> {
    let dir = String::from(path) + "/versions/" + version;
    fs::create_dir(&dir).ok()?;

    let text = reqwest::blocking::get(url).ok()?.text().ok()?;
    fs::write(dir + "/" + version + ".json", text).ok()?;

    Some(())
}

/// 获取下载列表
pub fn list_game() -> Option<Vec<GameUrl>> {
    let mut game_list = Vec::new();

    // 下载列表
    let text = reqwest::blocking::get("http://launchermeta.mojang.com/mc/game/version_manifest_v2.json").ok()?.text().ok()?;
    // // 储存json，与官启保持一致
    // fs::write(String::from(path) + "/version_manifest_v2.json", &text).ok()?;

    // 开始解析
    let json = serde_json::from_str::<Value>(&text).ok()?;

    for version in json["versions"].as_array()? {
        let game = GameUrl {
            game_type: version["type"].as_str()?.to_string(),
            url: version["url"].as_str()?.to_string(),
            version: version["id"].as_str()?.to_string(),
        };
        game_list.push(game);
    }

    Some(game_list)
}
