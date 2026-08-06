//! Download libraries

use serde_json::Value;
use std::env::consts as env;
use std::fs::{copy, create_dir_all, exists, remove_dir_all};

use utils::{check_rules, get_parent_dir, list_file};

use super::{DownloadError, DownloadTask, TaskInfo};

/// 下载library
fn download_lib(save_path: &str, node: &Value, mirror: &str) -> Result<TaskInfo, DownloadError> {
    let dir = get_parent_dir(&save_path);
    if !exists(&dir)? {
        create_dir_all(&dir)?;
    }
    let mut url = node["url"]
        .as_str()
        .ok_or(DownloadError::DataInvalid)?
        .to_string();
    url = url.replace("https://libraries.minecraft.net", &mirror);
    Ok(TaskInfo {
        url,
        save_path: save_path.to_string(),
    })
}

/// 下载libraries，node: mc json["libraries"]，返回Tasks
pub fn download_libraries(
    node: &Value,
    path: &str,
    game_dir: &str,
    mirror: &str,
    fabric_mirror: &str,
) -> Result<Vec<DownloadTask>, DownloadError> {
    let mut c = 0;
    let mut tasks = Vec::new();
    for item in node.as_array().ok_or(DownloadError::DataInvalid)? {
        let (node, path, game_dir, mirror, id) = (
            item.clone(),
            path.to_string(),
            game_dir.to_string(),
            mirror.to_string(),
            c.clone(),
        );
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
            let key = node["natives"][os]
                .as_str()
                .ok_or(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invaild data",
                ))?
                .replace("${arch}", arch);
            let node = &node["downloads"]["classifiers"][&key];
            let save_path =
                lib_dir.clone() + "/" + node["path"].as_str().ok_or(DownloadError::DataInvalid)?; // 储存位置
            if !exists(&save_path)? {
                let task_info = download_lib(&save_path, node, &mirror)?;
                let natives_dir_clone = natives_dir.clone();
                tasks.push(DownloadTask {
                    url: task_info.url,
                    save_path: task_info.save_path,
                    on_finish: Some(Box::new(move || {
                        extract_lib(&natives_dir_clone, &save_path, &id.to_string());
                    })),
                });
            } else {
                // TODO: check hash
                let natives_dir_clone = natives_dir.clone();
                extract_lib(&natives_dir_clone, &save_path, &id.to_string());
            }
        }
        if node["downloads"]["artifact"].is_object() {
            let save_path = lib_dir.clone()
                + "/"
                + node["downloads"]["artifact"]["path"]
                    .as_str()
                    .ok_or(DownloadError::DataInvalid)?;
            if !exists(&save_path)? {
                let task_info = download_lib(&save_path, &node["downloads"]["artifact"], &mirror)?;
                // Add natives
                let name: Vec<&str> = node["name"]
                    .as_str()
                    .ok_or(DownloadError::DataInvalid)?
                    .split(":")
                    .collect();
                let name = name.last().ok_or(DownloadError::DataInvalid)?;
                if name.contains("natives") {
                    let natives_dir_clone = natives_dir.clone();
                    tasks.push(DownloadTask {
                        url: task_info.url,
                        save_path: task_info.save_path,
                        on_finish: Some(Box::new(move || {
                            extract_lib(&natives_dir_clone, &save_path, &id.to_string());
                        })),
                    });
                } else {
                    tasks.push(DownloadTask {
                        url: task_info.url,
                        save_path: task_info.save_path,
                        on_finish: None,
                    });
                }
            } else {
                let natives_dir_clone = natives_dir.clone();
                extract_lib(&natives_dir_clone, &save_path, &id.to_string());
            }
        } else {
            if let Some(url) = node["url"].as_str() {
                if url == "https://maven.fabricmc.net/" {
                    // Fabric
                    let mut path = String::new();
                    let name = node["name"].as_str().ok_or(DownloadError::DataInvalid)?;
                    let split_1: Vec<&str> = name.split(":").collect();
                    let split_2: Vec<&str> = split_1[0].split(".").collect();
                    for name in split_2 {
                        path = path + name + "/";
                    }
                    for i in 1..split_1.len() {
                        let name = split_1[i];
                        path = path + name + "/";
                    }
                    path = path + split_1[1] + "-" + split_1[2] + ".jar";
                    if !exists(&path)? {
                        let url = fabric_mirror.to_string() + "/" + &path;
                        let local_path = lib_dir.clone() + "/" + &path;
                        tasks.push(DownloadTask::new(url, local_path, None));
                    } else {
                        // TODO: check hash
                    }
                }
            }
        }
        c += 1;
    }

    Ok(tasks)
}

/// 解压出natives
pub fn extract_lib(natives_dir: &str, local_path: &str, id: &str) -> Result<(), DownloadError> {
    // 目标natives文件夹
    if !exists(&natives_dir)? {
        std::fs::create_dir(&natives_dir)?;
    }

    // 解压用的临时文件夹
    if exists(&("temp".to_string() + id))? {
        std::fs::remove_dir_all("temp".to_string() + id)?;
    }
    std::fs::create_dir("temp".to_string() + id)?;

    let mut zip = zip::ZipArchive::new(std::fs::File::open(local_path)?)
        .map_err(|err| std::io::Error::from(err))?;
    zip.extract("temp".to_string() + &id.to_string())
        .map_err(|err| std::io::Error::from(err))?;
    let files = list_file(&("temp".to_string() + &id.to_string()))?;
    for name in files {
        let format: Vec<&str> = name.split(".").collect();
        let format = *format.last().ok_or(DownloadError::DataInvalid)?;
        if !(format == "dll" || format == "dylib" || format == "so") {
            // windows || macOS || linux
            continue;
        }
        let split: Vec<&str> = name.split("/").collect();
        let file_name = split.last().ok_or(DownloadError::DataInvalid)?;
        let target_path = natives_dir.to_string() + "/" + &file_name;
        if !exists(&target_path)? {
            copy(name, &target_path)?;
        }
    }
    remove_dir_all("temp".to_string() + &id.to_string())?;
    Ok(())
}
