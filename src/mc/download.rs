//! 下载文件

use std::fs::{self, exists};
use std::env::consts as env;
use std::sync::Arc;
use std::time::Duration;
use log::info;
use serde_json::Value;
use tokio::sync::Semaphore;
use std::thread::sleep;
use crate::downloader::downloader::Downloader;
use crate::file_tools::{get_parent_dir, list_file};
use super::check_rules;

/// 在下载游戏时使用的游戏信息
#[derive(Clone)]
pub struct GameUrl {
    /// 游戏类型
    pub game_type: String,

    /// 本体下载链接
    pub url: String,
    
    /// 游戏版本
    pub version: String,
}

/// Forge信息
pub struct Forge {
    pub version: String,
    pub branch: String,
    pub modified: String,
}

/// 下载
pub async fn download(url: String, path: String, max: usize) -> Option<()> {
    info!("Start downloading {url}");
    let mut response = reqwest::get(&url).await;
    let mut c = 0; // retry times
    while response.is_err() {
        if c == max { return None; } // retry times: 3 TODO: support change this value
        response = reqwest::get(&url).await;
        c += 1;
    }
    tokio::fs::write(path, response.unwrap().bytes().await.ok()?).await.ok()?;
    info!("Finish downloading {url}");
    Some(())
}

/// 下载assets
pub fn download_assets(path: &str, id: &str, mirror: &str, downloader: &Downloader) -> Option<()> {
    let assets_dir = path.to_string() + "/assets";
    let index_path = assets_dir.clone() + "/indexes/" + &id + ".json";
    let json = serde_json::from_str::<Value>(&fs::read_to_string(&index_path).ok()?).ok()?;
    for (_, node) in json["objects"].as_object()? {
        let hash = node["hash"].as_str()?;
        let dl_path = hash[0..2].to_string() + "/" + hash;
        let obj_path = assets_dir.clone() + "/objects";
        let local_path = obj_path.clone() + "/" + &dl_path;
        if !exists(&local_path).ok()? {
            let dir = obj_path.clone() + "/" + &hash[0..2];
            if !exists(&dir).ok()? { fs::create_dir_all(&dir).ok()?; }
            let url = mirror.to_string() + "/" + &dl_path;
            downloader.add(url.clone(), local_path.clone()).ok()?;
        }
    }

    Some(())
}

/// 下载library
fn download_lib(local_path: &String, node: &Value, mirror: &String, downloader: &Downloader) -> Option<()> {
    if !exists(&local_path).ok()? {
        let dir = get_parent_dir(&local_path);
        if !exists(&dir).ok()? { fs::create_dir_all(&dir).ok()?; }
        let url = node["url"].as_str()?.replace("https://libraries.minecraft.net", &mirror);
        downloader.add(url.clone(), local_path.clone()).ok()?;
    }

    Some(())
}

/// 下载libraries，node: mc json["libraries"]
pub fn download_libraries(node: &Value, path: &str, game_dir: &str, mirror: &str, downloader: &Downloader) -> Option<()> {
    let mut c = 0;
    for item in node.as_array()? {
        let (node, path, game_dir, mirror, id) = (item.clone(), path.to_string(), game_dir.to_string(), mirror.to_string(), c.clone());
        let lib_dir = path.to_string() + "/libraries";
        let os = if env::OS == "macOS" { "osx" } else { env::OS };
        let natives_dir = game_dir.to_string() + "/natives-" + os + "-" + env::ARCH;
        if node["rules"].is_array() {
            if !check_rules(&node["rules"]) {
                continue;
            }
        }
        // Add natives for old versions
        if node["natives"][os].is_string() && node["downloads"]["classifiers"].is_object() {
            let arch = if env::ARCH.contains("64") { "64" } else { "32" };
            let key = node["natives"][os].as_str()?.replace("${arch}", arch);
            let node = &node["downloads"]["classifiers"][&key];
            let local_path = lib_dir.clone() + "/" + node["path"].as_str()?;  // 储存位置
            download_lib(&local_path, node, &mirror, downloader)?;
            extract_lib(&natives_dir, &local_path, &id.to_string())?;
        }
        if node["downloads"]["artifact"].is_object() {
            let local_path = lib_dir.clone() + "/" + node["downloads"]["artifact"]["path"].as_str()?;
            download_lib(&local_path, &node["downloads"]["artifact"], &mirror, downloader)?;
            // Add natives
            let name: Vec<&str> = node["name"].as_str()?.split(":").collect();
            let name = name.last()?;
            if name.contains("natives") {
                extract_lib(&natives_dir, &local_path, &id.to_string())?;
            }
        }
        c += 1;
    }

    Some(())
}

/// 解压出natives
fn extract_lib(natives_dir: &String, local_path: &String, id: &String) -> Option<()> {
    // 目标natives文件夹
    if !exists(&natives_dir).ok()? { fs::create_dir(&natives_dir).ok()?; }

    // 解压用的临时文件夹
    if exists(&("temp".to_string() + id)).ok()? {
        fs::remove_dir_all("temp".to_string() + &id.to_string()).ok()?;
    }
    fs::create_dir("temp".to_string() + id).ok()?;

    let mut zip = zip::ZipArchive::new(fs::File::open(local_path).ok()?).ok()?;
    zip.extract("temp".to_string() + &id.to_string()).ok()?;
    let files = list_file(&("temp".to_string() + &id.to_string())).ok()?;
    for name in files {
        let format: Vec<&str> = name.split(".").collect();
        let format = *format.last()?;
        if !(format == "dll" || format == "dylib" || format == "so") {  // windows || macOS || linux
            continue;
        }
        let split: Vec<&str> = name.split("/").collect();
        let file_name = split.last()?;
        let target_path = natives_dir.clone() + "/" + &file_name;
        if !exists(&target_path).ok()? { fs::copy(name, &target_path).ok()?; }
    }
    fs::remove_dir_all("temp".to_string() + &id.to_string()).ok()
}

/// 获取Forge列表 官方没有json，使用BMCLAPI2
pub async fn list_forge(mcversion: &String) -> Option<Vec<Forge>> {
    let mut forge_list = Vec::new();

    let url = String::from("https://bmclapi2.bangbang93.com/forge/minecraft/") + mcversion;
    let text = reqwest::get(url).await.ok()?.text().await.ok()?;
    let json = serde_json::from_str::<Value>(&text).ok()?;

    for version in json.as_array()? {
        let branch = if let Some(branch) = version["branch"].as_str() {
            branch.to_string()
        } else {
            String::new()
        };
        
        let modified = version["modified"].as_str()?.to_string();

        let forge = Forge {
            version: version["version"].as_str()?.to_string(),
            branch: branch,
            modified: modified,
        };

        forge_list.push(forge);
    }

    forge_list.sort_by(|a, b| b.modified.cmp(&a.modified));

    Some(forge_list)
}

/// 获取下载列表
pub async fn list_game() -> Option<Vec<GameUrl>> {
    let mut game_list = Vec::new();

    // 下载列表
    let text = reqwest::get("https://launchermeta.mojang.com/mc/game/version_manifest_v2.json").await.ok()?.text().await.ok()?;
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
