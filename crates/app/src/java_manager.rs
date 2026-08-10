//! Java Manager
//! 管理Java安装列表，储存到java.json配置文件
//! 使用 java crate 中的 JavaInstallation 来解析和管理 Java 版本

use java::java::JavaInstallation;
use log::{error, warn};
use serde_json::json;
use std::fs::{self, exists};

use crate::LauncherError;

pub struct JavaManager {
    java_list: Vec<JavaInstallation>,
}

impl JavaManager {
    pub fn new() -> Result<Self, LauncherError> {
        let java_list = JavaManager::i_load()?;
        Ok(Self { java_list })
    }

    /// 添加一个 Java 安装。传入 Java Home 路径，会自动解析版本。
    /// 如果路径已存在则跳过。
    pub fn add(&mut self, java_path: String) -> Result<(), LauncherError> {
        // 检查是否已存在相同路径
        if self.java_list.iter().any(|j| j.get_path() == java_path) {
            return Ok(());
        }

        match JavaInstallation::new(java_path) {
            Ok(installation) => {
                self.java_list.push(installation);
                // 按版本排序
                self.java_list.sort_by(|a, b| a.get_version().cmp(b.get_version()));
                self.save()
            }
            Err(e) => {
                error!("Failed to create JavaInstallation: {:?}", e);
                Err(LauncherError::JavaInstallationError)
            }
        }
    }

    pub fn del(&mut self, index: u32) -> Result<(), LauncherError> {
        if (index as usize) < self.java_list.len() {
            self.java_list.remove(index as usize);
            self.save()
        } else {
            Err(LauncherError::OutOfRange)
        }
    }

    pub fn get_java_list(&self) -> &Vec<JavaInstallation> {
        &self.java_list
    }

    fn save(&self) -> Result<(), LauncherError> {
        let paths: Vec<&str> = self.java_list.iter().map(|j| j.get_path()).collect();
        let json = json!(paths);
        fs::write("java.json", json.to_string())?;
        Ok(())
    }

    fn i_load() -> Result<Vec<JavaInstallation>, LauncherError> {
        if !exists("java.json")? {
            return Ok(Vec::new());
        }

        let json = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string("java.json")?,
        )?;

        let mut java_list = Vec::new();
        if let Some(array) = json.as_array() {
            for item in array {
                if let Some(path) = item.as_str() {
                    match JavaInstallation::new(path.to_string()) {
                        Ok(installation) => {
                            java_list.push(installation);
                        }
                        Err(e) => {
                            warn!("Failed to load Java at {}: {:?}", path, e);
                        }
                    }
                }
            }
        } else {
            warn!("java.json is not an array, starting fresh.");
        }

        // 按版本排序
        java_list.sort_by(|a, b| a.get_version().cmp(b.get_version()));

        Ok(java_list)
    }
}