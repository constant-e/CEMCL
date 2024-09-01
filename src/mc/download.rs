//! mc::download 下载文件

use std::fs;
use std::sync::mpsc;
use std::thread;
use log::info;
use serde_json::Value;

use crate::file_tools::list_all;
use crate::file_tools::exists;
use super::{GameUrl, check_rules, env};

/// 下载
pub fn download(url: &str, path: &str, max: usize) -> Option<()> {
    let mut response = reqwest::blocking::get(url);
    let mut c = 0; // retry times
    while response.is_err() {
        if c == max { return None; } // retry times: 3 TODO: support change this value
        response = reqwest::blocking::get(url);
        c += 1;
    }
    fs::write(path, response.unwrap().bytes().ok()?).ok()?;
    info!("Downloaded {url}");
    Some(())
}

/// 下载assets
pub fn download_assets(path: &str, id: &str, mirror: &str) -> Option<()> {
    let assets_dir = path.to_string() + "/assets";
    let index_path = assets_dir.clone() + "/indexes/" + &id + ".json";
    let json = serde_json::from_str::<Value>(&fs::read_to_string(&index_path).ok()?).ok()?;
    let (tx, rx) = mpsc::channel();
    let mut c = 0; // thread counts
    for (_, node) in json["objects"].as_object()? {
        let hash = node["hash"].as_str()?;
        let dl_path = hash[0..2].to_string() + "/" + hash;
        let obj_path = assets_dir.clone() + "/objects";
        let local_path = obj_path.clone() + "/" + &dl_path;
        if !exists(&local_path) {
            let dir = obj_path.clone() + "/" + &hash[0..2];
            if !exists(&dir) { fs::create_dir_all(&dir).ok()?; }
            let url = mirror.to_string() + "/" + &dl_path;
            let tx = tx.clone();
            thread::spawn(move || {
                if download(&url, &local_path, 3).is_some() {
                    tx.send("ok").unwrap();
                } else {
                    tx.send("err").unwrap();
                }
            });
            c += 1;
        }
    }
    
    if c == 0 { return Some(()); }

    for received in rx {
        if received == "err" { return None; }
        c -= 1;
        if c == 0 { break; }
    }

    Some(())
}

fn download_lib(node: &Value, path: &str, game_dir: &str, mirror: &str, id: usize) -> Option<()> {
    let lib_dir = path.to_string() + "/libraries";

    if node["rules"].is_array() {
        if !check_rules(&node["rules"]) {
            println!("{}", node["name"].as_str().unwrap());
            return Some(());
        }
    }
    
    if node["downloads"]["artifact"].is_object() {
        let local_path = lib_dir.clone() + "/" + node["downloads"]["artifact"]["path"].as_str()?;
        if !exists(&local_path) {
            let vec: Vec<&str> = local_path.split("/").collect();
            let mut dir = String::new();
            for (index, item) in vec.iter().enumerate() {
                if index == vec.len() - 1 { break; }
                dir.push_str(item);
                if index != vec.len() - 2 { dir.push('/'); }
            }
            if !exists(&dir) { fs::create_dir_all(&dir).ok()?; }
            let url = node["downloads"]["artifact"]["url"].as_str()?.replace("https://libraries.minecraft.net", mirror);
            download(&url, &local_path, 3)?;
        }
    }

    let os = if env::OS == "macOS" { "osx" } else { env::OS };
    // Add natives
    if node["natives"][os].is_string() {
        let key = node["natives"][os].as_str()?;
        if node["downloads"]["classifiers"].is_object() {
            let local_path = lib_dir.clone() + "/" + node["downloads"]["classifiers"][key]["path"].as_str()?;
            let vec: Vec<&str> = local_path.split("/").collect();
            let mut dir = String::new();
            for (index, item) in vec.iter().enumerate() {
                if index == vec.len() - 1 { break; }
                dir.push_str(item);
                if index != vec.len() - 2 { dir.push('/'); }
            }
            if !exists(&local_path) {
                if !exists(&dir) { fs::create_dir_all(&dir).ok()?; }
                let url = node["downloads"]["classifiers"][key]["url"].as_str()?.replace("https://libraries.minecraft.net", mirror);
                download(&url, &local_path, 3)?;
            }

            // Extract to game dir
            if node["extract"].is_object() {
                let excludes = if node["extract"]["exclude"].is_array() { node["extract"]["exclude"].as_array()? } else { &Vec::new() };
                let natives_dir = game_dir.to_string() + "/natives-" + os + "-" + env::ARCH;
                if exists(&("temp".to_string() + &id.to_string())) {
                    fs::remove_dir_all("temp".to_string() + &id.to_string()).ok()?;
                }
                if !exists(&natives_dir) { fs::create_dir(&natives_dir).ok()?; }
                fs::create_dir("temp".to_string() + &id.to_string()).ok()?; // 临时文件夹
                let mut zip = zip::ZipArchive::new(fs::File::open(local_path).ok()?).ok()?;
                zip.extract("temp".to_string() + &id.to_string()).ok()?;
                let dir = list_all(&("temp".to_string() + &id.to_string()))?;
                for name in dir {
                    let mut allow = true;
                    for n in excludes {
                        let e = n.as_str()?;
                        if e.replace("/", "") == name {
                            allow = false;
                            break;
                        }
                    }
                    if !allow { continue; }
                    let target_path = natives_dir.clone() + "/" + &name;
                    if !exists(&target_path) { fs::copy("temp".to_string() + &id.to_string() + "/" + &name, &target_path).ok()?; }
                }
                fs::remove_dir_all("temp".to_string() + &id.to_string()).ok()?;
            }
        }
    }
    Some(())
}

/// 下载libraries node: mc json["libraries"]
pub fn download_libraries(node: &Value, path: &str, game_dir: &str, mirror: &str) -> Option<()> {
    let (tx, rx) = mpsc::channel();
    let mut c = 0; // thread counts
    for item in node.as_array()? {
        let tx = tx.clone();
        let (i, p, g, m, id) = (item.clone(), path.to_string(), game_dir.to_string(), mirror.to_string(), c.clone());
        thread::spawn(move || {
            if download_lib(&i, &p, &g, &m, id).is_some() {
                tx.send("ok").unwrap();
            } else {
                tx.send("err").unwrap();
            }
        });
        c += 1;
    }
    
    if c == 0 { return Some(()); } // should be unused

    for received in rx {
        if received == "err" { return None; }
        c -= 1;
        if c == 0 { break; }
    }

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
