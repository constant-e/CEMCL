//! utils

use log::{info, warn};
use std::env::consts as env;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

/// 检查参数是否可以添加
pub fn check_rules(n: &serde_json::Value) -> bool {
    // 获取操作系统名称
    let os = if env::OS == "macOS" { "osx" } else { env::OS };

    if let Some(array) = n.as_array() {
        for r in array {
            if !r["features"].is_null() {
                // 暂时不支持
                return false;
            }
            if r["os"].is_null() {
                continue;
            } // 无意义rule
            if r["action"] == "allow" {
                if r["os"]["arch"].is_string() && r["os"]["arch"] != env::ARCH {
                    // debug!("ALLOW: {} not match {}", r["os"]["arch"], env::ARCH);
                    return false;
                }
                if r["os"]["name"].is_string() && r["os"]["name"] != os {
                    // debug!("ALLOW: {} not match {}", r["os"]["name"], os);
                    return false;
                }
            } else if r["action"] == "disallow" {
                if r["os"]["arch"].is_string() && r["os"]["arch"] == env::ARCH {
                    // debug!("DISALLOW: {} match {}", r["os"]["arch"], env::ARCH);
                    return false;
                }
                if r["os"]["name"].is_string() && r["os"]["name"] == os {
                    // debug!("DISALLOW: {} match {}", r["os"]["name"], os);
                    return false;
                }
            }
        }
    } else {
        warn!("Failed to get rules");
    }
    true
}

pub enum DLError {
    IOError(tokio::io::Error),
    ReqwestError(reqwest::Error),
}

impl From<reqwest::Error> for DLError {
    fn from(value: reqwest::Error) -> Self {
        DLError::ReqwestError(value)
    }
}

impl From<tokio::io::Error> for DLError {
    fn from(value: tokio::io::Error) -> Self {
        DLError::IOError(value)
    }
}

/// 下载单个文件，用于下载json
pub async fn download(url: String, path: String, max: usize) -> Result<(), DLError> {
    info!("Start downloading {url}");
    let mut response = reqwest::get(&url).await;
    let mut c = 0; // retry times
    while let Err(e) = response {
        if c >= max {
            return Err(e.into());
        }
        response = reqwest::get(&url).await;
        c += 1;
    }
    tokio::fs::write(path, response?.bytes().await?).await?;
    info!("Finish downloading {url}");
    Ok(())
}

/// 获取文件所在文件夹
pub fn get_parent_dir(path: &str) -> String {
    let mut vec: Vec<&str> = path.split("/").collect();
    if vec.len() == 1 {
        return String::new();
    }
    vec.pop().unwrap();
    let mut dir = String::new();
    for item in vec {
        dir.push_str(item);
        dir.push('/');
    }
    dir.pop();
    dir
}

/// 列出目录下所有文件和文件夹
pub fn list_all(path: &String) -> std::io::Result<Vec<String>> {
    let mut result: Vec<String> = Vec::new();
    for entry in fs::read_dir(&Path::new(path))? {
        let entry = entry?;
        let path = entry.path();
        result.push(
            path.file_name()
                .ok_or(ErrorKind::InvalidData)?
                .to_str()
                .ok_or(ErrorKind::InvalidData)?
                .into(),
        );
    }
    Ok(result)
}

/// 列出目录下所有文件夹
pub fn list_dir(path: &String) -> std::io::Result<Vec<String>> {
    let mut result: Vec<String> = Vec::new();
    for entry in fs::read_dir(&Path::new(path))? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        result.push(
            path.file_name()
                .ok_or(ErrorKind::InvalidData)?
                .to_str()
                .ok_or(ErrorKind::InvalidData)?
                .into(),
        );
    }
    Ok(result)
}

/// 递归列出目录下所有文件
pub fn list_file(path: &String) -> std::io::Result<Vec<String>> {
    let mut result = Vec::new();
    for entry in fs::read_dir(&Path::new(path))? {
        let entry = entry?;
        let entry_path = entry.path();
        let path = path.clone()
            + "/"
            + entry_path
                .file_name()
                .ok_or(ErrorKind::InvalidData)?
                .to_str()
                .ok_or(ErrorKind::InvalidData)?;
        if entry_path.is_dir() {
            result.append(&mut list_file(&path)?)
        } else {
            result.push(path);
        }
    }
    Ok(result)
}

pub async fn list_file_async(path: &String) -> tokio::io::Result<Vec<String>> {
    let mut result = Vec::new();
    let mut entries = tokio::fs::read_dir(&Path::new(path)).await?;
    while let Some(entry) = entries.next_entry().await? {
        let entry_path = entry.path();
        let path = path.clone()
            + "/"
            + entry_path
                .file_name()
                .ok_or(ErrorKind::InvalidData)?
                .to_str()
                .ok_or(ErrorKind::InvalidData)?;
        if entry_path.is_dir() {
            result.append(&mut list_file(&path)?)
        } else {
            result.push(path);
        }
    }
    Ok(result)
}
