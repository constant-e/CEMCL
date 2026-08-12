//! Java Manager
//! 管理Java安装列表，储存到java.json配置文件
//! 使用 java crate 中的 JavaInstallation 来解析和管理 Java 版本

use java::java::JavaInstallation;
use log::warn;
use serde_json::json;
use std::fs::{self, exists};
use std::process::Command;

use crate::LauncherError;

pub struct JavaManager {
    /// 默认使用的 Java 版本索引，None 表示未选择
    default: Option<u32>,
    /// Java 安装列表，按版本降序排列
    java_list: Vec<JavaInstallation>,
}

impl JavaManager {
    pub fn new() -> Result<Self, LauncherError> {
        let (default, java_list) = JavaManager::i_load()?;
        Ok(Self { default, java_list })
    }

    /// 添加一个 Java 安装。如果路径已存在则忽略。
    pub fn add(&mut self, java_path: String) -> Result<(), LauncherError> {
        if self.java_list.iter().any(|j| j.get_path() == java_path) {
            return Ok(());
        }

        let installation = JavaInstallation::new(java_path)?;
        self.java_list.push(installation);
        // 按版本降序排列（新的在前）
        self.java_list
            .sort_by(|a, b| b.get_version().cmp(a.get_version()));
        self.save()
    }

    /// 删除指定索引的 Java 安装
    pub fn del(&mut self, index: u32) -> Result<(), LauncherError> {
        if (index as usize) < self.java_list.len() {
            self.java_list.remove(index as usize);
            // 调整 default 索引
            self.default = match self.default {
                Some(d) if d == index => None,
                Some(d) if d > index => Some(d - 1),
                other => other,
            };
            self.save()
        } else {
            Err(LauncherError::OutOfRange)
        }
    }

    pub fn get_java_list(&self) -> &Vec<JavaInstallation> {
        &self.java_list
    }

    pub fn get_default(&self) -> Option<u32> {
        self.default
    }

    pub fn set_default(&mut self, index: Option<u32>) -> Result<(), LauncherError> {
        self.default = index;
        self.save()
    }

    /// 根据索引获取 java 可执行文件路径
    pub fn get_java_path_by_index(&self, index: u32) -> Result<String, LauncherError> {
        self.java_list
            .get(index as usize)
            .ok_or(LauncherError::OutOfRange)?
            .get_java_path()
            .map_err(|e| e.into())
    }

    /// 查找与指定最低版本兼容的最佳 Java 索引（优先选择最新的兼容版本）。
    /// 返回 None 表示没有兼容版本或列表为空。
    pub fn find_best_java_index(
        &self,
        min_version: Option<&java::java_version::JavaVersion>,
    ) -> Option<u32> {
        let min = min_version?;
        for (i, j) in self.java_list.iter().enumerate() {
            if j.get_version() >= min {
                return Some(i as u32);
            }
        }
        None // 没有兼容版本
    }

    /// 首次启动时自动检测系统 Java。尝试常见路径、JAVA_HOME 和 PATH 中的 java。
    pub fn auto_detect_system_java(&mut self) {
        if !self.java_list.is_empty() {
            return; // already have Java installations
        }

        // Common Java home paths
        let candidates = vec![
            "/usr/lib/jvm/default-java",
            "/usr/lib/jvm/java-21-openjdk",
            "/usr/lib/jvm/java-17-openjdk",
            "/usr/lib/jvm/java-11-openjdk",
            "/usr/lib/jvm/java-8-openjdk",
            "/usr/lib/jvm/java-21",
            "/usr/lib/jvm/java-17",
            "/usr/lib/jvm/java-11",
            "/usr/lib/jvm/java-8",
        ];

        for path in candidates {
            if std::fs::exists(path).unwrap_or(false) {
                if let Ok(installation) = JavaInstallation::new(path.to_string()) {
                    self.java_list.push(installation);
                }
            }
        }

        // Try to detect JAVA_HOME from environment
        if let Ok(java_home) = std::env::var("JAVA_HOME") {
            if !java_home.is_empty()
                && !self.java_list.iter().any(|j| j.get_path() == java_home)
            {
                if let Ok(installation) = JavaInstallation::new(java_home) {
                    self.java_list.push(installation);
                }
            }
        }

        // Try to detect java from PATH
        if let Ok(output) = Command::new("java").arg("-XshowSettings:properties").arg("-version").output() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            for line in stderr.lines() {
                if line.trim().starts_with("java.home") {
                    let home = line.trim().trim_start_matches("java.home = ");
                    if !home.is_empty()
                        && !self.java_list.iter().any(|j| j.get_path() == home)
                    {
                        if let Ok(installation) = JavaInstallation::new(home.to_string()) {
                            self.java_list.push(installation);
                        }
                    }
                    break;
                }
            }
        }

        if !self.java_list.is_empty() {
            self.java_list
                .sort_by(|a, b| b.get_version().cmp(a.get_version()));
            let _ = self.save();
        }
    }

    fn save(&self) -> Result<(), LauncherError> {
        let paths: Vec<&str> = self.java_list.iter().map(|j| j.get_path()).collect();
        let json = json!({
            "default": self.default,
            "java_list": paths,
        });
        fs::write("java.json", json.to_string())?;
        Ok(())
    }

    fn i_load() -> Result<(Option<u32>, Vec<JavaInstallation>), LauncherError> {
        if !exists("java.json")? {
            return Ok((None, Vec::new()));
        }

        let json = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string("java.json")?,
        )?;

        let default = json["default"].as_u64().map(|v| v as u32);

        let mut java_list = Vec::new();
        if let Some(array) = json["java_list"].as_array() {
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
            // 兼容旧格式：直接是路径数组
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
                warn!("java.json has unexpected format, starting fresh.");
            }
        }

        // 按版本降序排列（新的在前）
        java_list.sort_by(|a, b| b.get_version().cmp(a.get_version()));

        Ok((default, java_list))
    }
}