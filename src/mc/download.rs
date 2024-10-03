//! 下载文件

use std::fs::{self, exists};
use std::env::consts as env;
use std::sync::Arc;
use log::info;
use serde_json::Value;
use tokio::sync::Semaphore;

use crate::file_tools::list_file;
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
    return Some(());
}

/// 下载assets
pub fn download_assets(path: &str, id: &str, mirror: &str, semaphore: &Arc<Semaphore>) -> Option<Vec<tokio::task::JoinHandle<()>>> {
    let assets_dir = path.to_string() + "/assets";
    let index_path = assets_dir.clone() + "/indexes/" + &id + ".json";
    let json = serde_json::from_str::<Value>(&fs::read_to_string(&index_path).ok()?).ok()?;
    let mut futures = Vec::new();
    for (_, node) in json["objects"].as_object()? {
        let hash = node["hash"].as_str()?;
        let dl_path = hash[0..2].to_string() + "/" + hash;
        let obj_path = assets_dir.clone() + "/objects";
        let local_path = obj_path.clone() + "/" + &dl_path;
        if !exists(&local_path).ok()? {
            let dir = obj_path.clone() + "/" + &hash[0..2];
            if !exists(&dir).ok()? { fs::create_dir_all(&dir).ok()?; }
            let url = mirror.to_string() + "/" + &dl_path;
            let semaphore = semaphore.clone();
            let future = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                download(url.clone(), local_path.clone(), 3).await;
            });
            futures.push(future);
        }
    }

    return Some(futures);
}

/// 下载单个library
async fn download_lib(node: Value, path: String, game_dir: String, mirror: String, id: usize) -> Option<()> {
    let lib_dir = path.to_string() + "/libraries";

    if node["rules"].is_array() {
        if !check_rules(&node["rules"]) {
            return Some(());
        }
    }

    let os = if env::OS == "macOS" { "osx" } else { env::OS };
    // Add natives for old versions
    if node["natives"][os].is_string() {
        let arch = if env::ARCH.contains("64") { "64" } else { "32" };
        let key = node["natives"][os].as_str()?.replace("${arch}", arch);
        if node["downloads"]["classifiers"].is_object() {
            let local_path = lib_dir.clone() + "/" + node["downloads"]["classifiers"][&key]["path"].as_str()?;
            let vec: Vec<&str> = local_path.split("/").collect();
            let mut dir = String::new();
            for (index, item) in vec.iter().enumerate() {
                if index == vec.len() - 1 { break; }
                dir.push_str(item);
                if index != vec.len() - 2 { dir.push('/'); }
            }
            if !exists(&local_path).ok()? {
                if !exists(&dir).ok()? { fs::create_dir_all(&dir).ok()?; }
                let url = node["downloads"]["classifiers"][&key]["url"].as_str()?.replace("https://libraries.minecraft.net", &mirror);
                download(url.clone(), local_path.clone(), 3).await?;
            }

            // Add natives
            let natives_dir = game_dir.to_string() + "/natives-" + os + "-" + env::ARCH;
            if exists(&("temp".to_string() + &id.to_string())).ok()? {
                tokio::fs::remove_dir_all("temp".to_string() + &id.to_string()).await.ok()?;
            }
            if !exists(&natives_dir).ok()? { tokio::fs::create_dir(&natives_dir).await.ok()?; }
            tokio::fs::create_dir("temp".to_string() + &id.to_string()).await.ok()?; // 临时文件夹
            let mut zip = zip::ZipArchive::new(fs::File::open(&local_path).ok()?).ok()?;
            zip.extract("temp".to_string() + &id.to_string()).ok()?;
            let files = list_file(&("temp".to_string() + &id.to_string())).ok()?;
            for name in files {
                let format: Vec<&str> = name.split(".").collect();
                let format = *format.last()?;
                if !(format == "dll" || format == "dylib" || format == "so") { // windows || macOS || linux
                    continue;
                }
                let split: Vec<&str> = name.split("/").collect();
                let file_name = split.last()?;
                let target_path = natives_dir.clone() + "/" + &file_name;
                if !exists(&target_path).ok()? { tokio::fs::copy(name, &target_path).await.ok()?; }
            }
            tokio::fs::remove_dir_all("temp".to_string() + &id.to_string()).await.ok()?;
        }
    }
    
    if node["downloads"]["artifact"].is_object() {
        let local_path = lib_dir.clone() + "/" + node["downloads"]["artifact"]["path"].as_str()?;
        if !exists(&local_path).ok()? {
            let vec: Vec<&str> = local_path.split("/").collect();
            let mut dir = String::new();
            for (index, item) in vec.iter().enumerate() {
                if index == vec.len() - 1 { break; }
                dir.push_str(item);
                if index != vec.len() - 2 { dir.push('/'); }
            }
            if !exists(&dir).ok()? { tokio::fs::create_dir_all(&dir).await.ok()?; }
            let url = node["downloads"]["artifact"]["url"].as_str()?.replace("https://libraries.minecraft.net", &mirror);
            download(url.clone(), local_path.clone(), 3).await?;
        }
        // Add natives
        let name: Vec<&str> = node["name"].as_str()?.split(":").collect();
        let name = name.last()?;
        if name.contains("natives") {
            let natives_dir = game_dir.to_string() + "/natives-" + os + "-" + env::ARCH;
            if exists(&("temp".to_string() + &id.to_string())).ok()? {
                tokio::fs::remove_dir_all("temp".to_string() + &id.to_string()).await.ok()?;
            }
            if !exists(&natives_dir).ok()? { fs::create_dir(&natives_dir).ok()?; }
            tokio::fs::create_dir("temp".to_string() + &id.to_string()).await.ok()?; // 临时文件夹
            let mut zip = zip::ZipArchive::new(fs::File::open(&local_path).ok()?).ok()?;
            zip.extract("temp".to_string() + &id.to_string()).ok()?;
            let files = list_file(&("temp".to_string() + &id.to_string())).ok()?;
            for name in files {
                let format: Vec<&str> = name.split(".").collect();
                let format = *format.last()?;
                if !(format == "dll" || format == "dylib" || format == "so") { // windows || macOS || linux
                    continue;
                }
                let split: Vec<&str> = name.split("/").collect();
                let file_name = split.last()?;
                let target_path = natives_dir.clone() + "/" + &file_name;
                if !exists(&target_path).ok()? { tokio::fs::copy(name, &target_path).await.ok()?; }
            }
            tokio::fs::remove_dir_all("temp".to_string() + &id.to_string()).await.ok()?;
        }
    }
    
    return Some(());
}

/// 下载libraries，node: mc json["libraries"]
pub fn download_libraries(node: &Value, path: &str, game_dir: &str, mirror: &str, semaphore: &Arc<Semaphore>) -> Option<Vec<tokio::task::JoinHandle<()>>> {
    let mut futures = Vec::new();
    let mut c = 0;
    for item in node.as_array()? {
        let (i, p, g, m, id) = (item.clone(), path.to_string(), game_dir.to_string(), mirror.to_string(), c.clone());
        let semaphore = semaphore.clone();
        let future = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            download_lib(i, p, g, m, id).await;
        });
        futures.push(future);
        c += 1;
    }

    return Some(futures);
}

/// 获取下载列表
pub async fn list_game() -> Option<Vec<GameUrl>> {
    let mut game_list = Vec::new();

    // 下载列表
    let text = reqwest::get("http://launchermeta.mojang.com/mc/game/version_manifest_v2.json").await.ok()?.text().await.ok()?;
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

    return Some(game_list);
}
