//! mc::download.rs 下载文件

use std::fs;
use std::rc::Rc;
use log::info;
use serde_json::Value;

use crate::{file_tools::exists, Config, Game};
use super::{GameUrl, check_rules, env};

/// 下载assets index
fn init_assets() {

}

/// 下载assets
fn download_assets(path: &str, id: &str, mirror: &str) -> Option<()> {
    let assets_dir = path.to_string() + "/assets";
    
    let json = serde_json::from_str::<Value>(&fs::read_to_string(assets_dir.clone() + "/indexes/" + id).ok()?).ok()?;
    for (key, node) in json["objects"].as_object()? {
        let hash = node["hash"].as_str()?;
        let dl_path = hash[0..2].to_string() + "/" + hash;
        let local_path = assets_dir.clone() + "/objects/" + &dl_path;
        if !exists(&local_path) {
            info!("Downloading {key}...");
            let url = mirror.to_string() + "/" + &dl_path;
            let mut response = reqwest::blocking::get(&url);
            let mut c = 0; // retry times
            while response.is_err() {
                if c == 3 { return None; } // retry times: 3 TODO: support change this value
                response = reqwest::blocking::get(&url);
                c += 1;
            }
            fs::write(local_path, response.unwrap().bytes().ok()?).ok()?;
        }
    }
    
    Some(())
}

/// 下载libraries node: mc json["libraries"]
fn download_libraries(node: &Value, path: &str, mirror: &str) -> Option<()> {
    let lib_dir = path.to_string() + "/libraries";
    for item in node.as_array()? {
        if item["rules"].is_array() {
            if !check_rules(&item["rules"]) { continue; }
        }
        
        let local_path = lib_dir.clone() + "/" + node["downloads"]["artifact"]["path"].as_str()?;
        if !exists(&local_path) {
            let url = node["downloads"]["artifact"]["url"].as_str()?.replace("https://libraries.minecraft.net", mirror);
            let mut response = reqwest::blocking::get(&url);
            let mut c = 0; // retry times
            while response.is_err() {
                if c == 3 { return None; } // retry times: 3 TODO: support change this value
                response = reqwest::blocking::get(&url);
                c += 1;
            }
            fs::write(local_path, response.unwrap().bytes().ok()?).ok()?;
        }

        let os = if env::OS == "macOS" { "osx" } else { env::OS };
        // Add natives
        if item["natives"][os].is_string() {
            let key = item["natives"][os].as_str()?;
            let local_path = lib_dir.clone() + "/" + node["downloads"]["classifiers"][key]["path"].as_str()?;
            if !exists(&local_path) {
                let url = node["downloads"]["classifiers"][key]["url"].as_str()?.replace("https://libraries.minecraft.net", mirror);
                let mut response = reqwest::blocking::get(&url);
                let mut c = 0; // retry times
                while response.is_err() {
                    if c == 3 { return None; } // retry times: 3 TODO: support change this value
                    response = reqwest::blocking::get(&url);
                    c += 1;
                }
                fs::write(local_path, response.unwrap().bytes().ok()?).ok()?;
            }
        }
    }
    
    Some(())
}

/// 下载游戏本体jar
pub fn download_game(path: &str, version: &str) -> Option<()> {
    let game_path = path.to_string() + "/versions/" + version;
    let json = serde_json::from_str::<Value>(fs::read_to_string(&(game_path.clone() + "/" + version + ".json")).ok()?.as_str()).ok()?;
    

    
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
