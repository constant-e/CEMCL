//! Java Manager
//! 管理Java安装列表，储存到java.json配置文件
//! 使用 java crate 中的 JavaInstallation 来解析和管理 Java 版本

use frontend::UIUpdate;
use java::java::JavaInstallation;
use log::warn;
use serde_json::json;
use std::fs::{self, exists};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{LauncherError, ui::UiSender};

pub struct JavaManager {
    /// 默认使用的 Java 版本索引，None 表示未选择
    default: Option<u32>,
    /// Java 安装列表，按版本降序排列
    java_list: Vec<JavaInstallation>,
    /// 最近一次列表变更的索引映射：旧索引 -> 新索引。
    /// 启动时由 java.json 中记录的旧路径解析得到，供 VersionManager 修正各版本的 java_index。
    index_map: Vec<Option<u32>>,
    ui: UiSender,
}

impl JavaManager {
    pub fn new(ui: UiSender) -> Result<Self, LauncherError> {
        Self::i_new(ui, JavaManager::i_auto_detect())
    }

    /// 使用外部提供的检测结果构造管理器。
    /// 正常启动使用 `new`（自动寻找本机 Java）；测试可借此排除本机环境。
    pub fn new_with_detected(
        update_sender: tokio::sync::mpsc::UnboundedSender<UIUpdate>,
        detected: Vec<JavaInstallation>,
    ) -> Result<Self, LauncherError> {
        Self::i_new(UiSender::new(update_sender), detected)
    }

    fn i_new(ui: UiSender, detected: Vec<JavaInstallation>) -> Result<Self, LauncherError> {
        let (loaded_default, old_paths) = JavaManager::i_load()?;

        // 校验 java.json 中的路径：失效项直接删除
        let mut java_list: Vec<JavaInstallation> = Vec::new();
        for path in &old_paths {
            // 软链接等别名统一解析到实际安装位置，避免同一 Java 被重复添加
            let path = JavaManager::i_canonical_path(path);
            match JavaInstallation::new(path.clone()) {
                Ok(installation) => {
                    if !java_list
                        .iter()
                        .any(|j| j.get_path() == installation.get_path())
                    {
                        java_list.push(installation);
                    }
                }
                Err(e) => warn!("Invalid Java at {}: {e:?}, removing it.", path.display()),
            }
        }

        // 合并外部检测结果（启动时为系统 Java），按真实路径去重
        for installation in detected {
            let path = JavaManager::i_canonical_path(installation.get_path());
            if java_list.iter().any(|j| j.get_path() == path) {
                continue;
            }
            if path == installation.get_path() {
                java_list.push(installation);
            } else {
                match JavaInstallation::new(path.clone()) {
                    Ok(installation) => java_list.push(installation),
                    Err(e) => warn!("Invalid Java at {}: {e:?}.", path.display()),
                }
            }
        }

        JavaManager::sort(&mut java_list);

        let index_map = JavaManager::i_build_index_map(&old_paths, &java_list);
        let default = JavaManager::i_remap_index(loaded_default, &index_map);
        let manager = Self {
            default,
            java_list,
            index_map,
            ui,
        };

        // 删除失效项、合并检测结果或重新排序后写回
        if manager.default != loaded_default
            || manager
                .java_list
                .iter()
                .map(|j| j.get_path())
                .ne(old_paths.iter().map(|p| p.as_path()))
        {
            manager.save()?;
        }

        Ok(manager)
    }

    /// 取出最近一次列表变更的索引映射（旧索引 -> 新索引），用于同步各版本的 java_index
    pub fn take_index_map(&mut self) -> Vec<Option<u32>> {
        std::mem::take(&mut self.index_map)
    }

    /// 添加 Java 安装并刷新界面
    pub fn add_from_ui(&mut self, java_path: PathBuf) -> Result<(), LauncherError> {
        self.add(java_path)?;
        self.refresh_ui_list()?;
        self.refresh_ui_config()?;
        self.ui.send(UIUpdate::QuitAddJavaDialog)
    }

    /// 删除 Java 安装并刷新界面
    pub fn del_from_ui(&mut self, index: u32) -> Result<(), LauncherError> {
        self.del(index)?;
        self.refresh_ui_list()?;
        self.refresh_ui_config()
    }

    pub fn check(&self, java_path: PathBuf) -> Result<(), LauncherError> {
        use java::java::JavaInstallationError;
        let (result, version) = match JavaInstallation::new(java_path) {
            Ok(installation) => (
                frontend::ui::JavaCheckResult::Detected,
                installation.get_version().to_string(),
            ),
            Err(e) => {
                let result = match e {
                    JavaInstallationError::PathIsEmpty => {
                        frontend::ui::JavaCheckResult::PathIsEmpty
                    }
                    JavaInstallationError::JavaExecutableNotFound => {
                        frontend::ui::JavaCheckResult::JavaExecutableNotFound
                    }
                    JavaInstallationError::ReleaseFileInvalid => {
                        frontend::ui::JavaCheckResult::ReleaseFileInvalid
                    }
                    JavaInstallationError::IOError(_) => frontend::ui::JavaCheckResult::IOError,
                };
                (result, String::new())
            }
        };
        self.ui.send(UIUpdate::SetJavaCheckResult(result, version))
    }

    pub fn set_default_from_ui(&mut self, index: i32) -> Result<(), LauncherError> {
        self.set_default(if index >= 0 { Some(index as u32) } else { None })
    }

    /// 构造 Java 列表；给出 MC 版本时标记是否满足该版本的最低 Java 要求
    pub fn info_list(&self, game_path: &Path, mc_version: Option<&str>) -> Vec<frontend::JavaInfo> {
        let min_java = mc_version.and_then(|version| min_java_version_for_mc(version, game_path));
        self.java_list
            .iter()
            .map(|j| {
                let compatible = match &min_java {
                    Some(min) => j.get_version() >= min,
                    None => true,
                };
                frontend::JavaInfo {
                    version: j.get_version().to_string(),
                    path: j.get_path().to_string_lossy().into_owned(),
                    compatible,
                }
            })
            .collect()
    }

    pub fn refresh_ui_list(&self) -> Result<(), LauncherError> {
        self.ui
            .send(UIUpdate::SetJavaList(self.info_list(Path::new(""), None)))
    }

    fn refresh_ui_config(&self) -> Result<(), LauncherError> {
        self.ui
            .send(UIUpdate::SetJavaModel(self.info_list(Path::new(""), None)))?;
        self.ui.send(UIUpdate::SetJavaIndex(
            self.default.map(|i| i as i32).unwrap_or(-1),
        ))
    }

    /// 添加一个 Java 安装。如果路径（解析软链接后）已存在则忽略。
    pub fn add(&mut self, java_path: PathBuf) -> Result<(), LauncherError> {
        let java_path = JavaManager::i_canonical_path(&java_path);
        let old_paths = self.paths();
        if self.java_list.iter().any(|j| j.get_path() == java_path) {
            self.index_map = JavaManager::i_build_index_map(&old_paths, &self.java_list);
            return Ok(());
        }

        let installation = JavaInstallation::new(java_path)?;
        self.java_list.push(installation);
        JavaManager::sort(&mut self.java_list);
        self.i_remap(&old_paths);
        self.save()
    }

    /// 删除指定索引的 Java 安装
    pub fn del(&mut self, index: u32) -> Result<(), LauncherError> {
        if (index as usize) < self.java_list.len() {
            let old_paths = self.paths();
            self.java_list.remove(index as usize);
            self.i_remap(&old_paths);
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
        if let Some(index) = index {
            if index as usize >= self.java_list.len() {
                return Err(LauncherError::OutOfRange);
            }
        }
        self.default = index;
        self.save()
    }

    /// 根据索引获取 java 可执行文件路径
    pub fn get_java_path_by_index(&self, index: u32) -> Result<PathBuf, LauncherError> {
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

    /// 自动寻找系统 Java，返回检测结果。尝试常见路径、JAVA_HOME 和 PATH 中的 java。
    fn i_auto_detect() -> Vec<JavaInstallation> {
        let mut candidates: Vec<PathBuf> = vec![
            PathBuf::from("/usr/lib/jvm/default-java"),
            PathBuf::from("/usr/lib/jvm/java-21-openjdk"),
            PathBuf::from("/usr/lib/jvm/java-17-openjdk"),
            PathBuf::from("/usr/lib/jvm/java-11-openjdk"),
            PathBuf::from("/usr/lib/jvm/java-8-openjdk"),
            PathBuf::from("/usr/lib/jvm/java-21"),
            PathBuf::from("/usr/lib/jvm/java-17"),
            PathBuf::from("/usr/lib/jvm/java-11"),
            PathBuf::from("/usr/lib/jvm/java-8"),
        ];

        // /usr/lib/jvm 下的实际安装目录
        if let Ok(entries) = fs::read_dir("/usr/lib/jvm") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    candidates.push(path);
                }
            }
        }

        // Try to detect JAVA_HOME from environment
        if let Ok(java_home) = std::env::var("JAVA_HOME") {
            if !java_home.is_empty() {
                candidates.push(PathBuf::from(java_home));
            }
        }

        // Try to detect java from PATH
        if let Ok(output) = Command::new("java")
            .arg("-XshowSettings:properties")
            .arg("-version")
            .output()
        {
            let stderr = String::from_utf8_lossy(&output.stderr);
            for line in stderr.lines() {
                let line = line.trim();
                if let Some((key, value)) = line.split_once('=') {
                    if key.trim() == "java.home" {
                        let home = value.trim();
                        if !home.is_empty() {
                            candidates.push(PathBuf::from(home));
                        }
                        break;
                    }
                }
            }
        }

        let mut detected: Vec<JavaInstallation> = Vec::new();
        for path in candidates {
            let path = JavaManager::i_canonical_path(&path);
            if detected.iter().any(|j| j.get_path() == path)
                || !std::fs::exists(&path).unwrap_or(false)
            {
                continue;
            }
            match JavaInstallation::new(path.clone()) {
                Ok(installation) => detected.push(installation),
                Err(e) => warn!("Invalid Java at {}: {e:?}.", path.display()),
            }
        }
        detected
    }

    /// 按版本降序排列（新的在前）
    fn sort(java_list: &mut [JavaInstallation]) {
        java_list.sort_by(|a, b| b.get_version().cmp(a.get_version()));
    }

    fn paths(&self) -> Vec<PathBuf> {
        self.java_list
            .iter()
            .map(|j| j.get_path().to_path_buf())
            .collect()
    }

    /// 解析软链接等别名，返回 Java 的实际安装路径；无法解析时保留原路径
    fn i_canonical_path(path: &Path) -> PathBuf {
        fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    }

    /// 旧列表路径 -> 新列表索引；按解析软链接后的实际路径匹配，旧项不存在时为 None
    fn i_build_index_map(old_paths: &[PathBuf], new_list: &[JavaInstallation]) -> Vec<Option<u32>> {
        old_paths
            .iter()
            .map(|path| {
                let canonical = JavaManager::i_canonical_path(path);
                new_list
                    .iter()
                    .position(|j| j.get_path() == canonical.as_path())
                    .map(|index| index as u32)
            })
            .collect()
    }

    fn i_remap_index(index: Option<u32>, index_map: &[Option<u32>]) -> Option<u32> {
        index.and_then(|index| index_map.get(index as usize).copied().flatten())
    }

    /// 列表变更后按路径更新 default，并记录旧索引到新索引的映射
    fn i_remap(&mut self, old_paths: &[PathBuf]) {
        let index_map = JavaManager::i_build_index_map(old_paths, &self.java_list);
        self.default = JavaManager::i_remap_index(self.default, &index_map);
        self.index_map = index_map;
    }

    fn save(&self) -> Result<(), LauncherError> {
        let paths: Vec<String> = self
            .java_list
            .iter()
            .map(|j| j.get_path().to_string_lossy().into_owned())
            .collect();
        let json = json!({
            "default": self.default,
            "java_list": paths,
        });
        fs::write("java.json", json.to_string())?;
        Ok(())
    }

    /// 读取 java.json 中记录的默认索引与路径列表（路径尚未校验）
    fn i_load() -> Result<(Option<u32>, Vec<PathBuf>), LauncherError> {
        if !exists("java.json")? {
            return Ok((None, Vec::new()));
        }

        let json = serde_json::from_str::<serde_json::Value>(&fs::read_to_string("java.json")?)?;

        let default = json["default"].as_u64().map(|v| v as u32);

        let paths = if let Some(array) = json["java_list"].as_array() {
            array
                .iter()
                .filter_map(|item| item.as_str().map(PathBuf::from))
                .collect()
        } else if let Some(array) = json.as_array() {
            // 兼容旧格式：直接是路径数组
            array
                .iter()
                .filter_map(|item| item.as_str().map(PathBuf::from))
                .collect()
        } else {
            warn!("java.json has unexpected format, starting fresh.");
            Vec::new()
        };

        Ok((default, paths))
    }
}

/// Determine the minimum Java version required for a given Minecraft version.
/// Reads the MC version JSON file to get the `javaVersion.majorVersion` field.
/// Falls back to hardcoded rules if the JSON cannot be read.
fn min_java_version_for_mc(
    mc_version: &str,
    game_path: &Path,
) -> Option<::java::java_version::JavaVersion> {
    // Extract the base MC version from modded version strings like "fabric-loader-0.16.10-1.21.5"
    let base_version = extract_mc_version(mc_version);

    // Try to read from the MC version JSON file
    let json_path = game_path
        .join("versions")
        .join(base_version)
        .join(format!("{base_version}.json"));
    if let Ok(content) = std::fs::read_to_string(&json_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(major) = json["javaVersion"]["majorVersion"].as_u64() {
                return Some(::java::java_version::JavaVersion::from(
                    major.to_string().as_str(),
                ));
            }
        }
    }

    // Fallback: hardcoded rules based on MC version
    let parts: Vec<&str> = base_version.split(".").collect();
    if parts.len() < 2 {
        return None;
    }
    let major: u32 = parts[0].parse().ok()?;
    let minor: u32 = parts[1].parse().ok()?;
    let patch: u32 = parts.get(2).and_then(|p| p.parse().ok()).unwrap_or(0);

    if major > 1 || (major == 1 && minor >= 21) || (major == 1 && minor == 20 && patch >= 5) {
        Some(::java::java_version::JavaVersion::from("21"))
    } else if major == 1 && minor >= 18 {
        Some(::java::java_version::JavaVersion::from("17"))
    } else if major == 1 && minor == 17 {
        Some(::java::java_version::JavaVersion::from("16"))
    } else {
        Some(::java::java_version::JavaVersion::from("1.8"))
    }
}

/// Extract the base Minecraft version from a modded version string.
/// e.g. "fabric-loader-0.16.10-1.21.5" -> "1.21.5"
///      "1.21.5-forge-52.0.0" -> "1.21.5"
///      "1.21.5" -> "1.21.5"
fn extract_mc_version(version: &str) -> &str {
    // For fabric: "fabric-loader-X.Y.Z-MC_VERSION"
    if let Some(rest) = version.strip_prefix("fabric-loader-") {
        // Find the last '-' and take everything after it
        if let Some(pos) = rest.rfind('-') {
            return &rest[pos + 1..];
        }
        return rest;
    }
    // For forge: "MC_VERSION-forge-X.Y.Z"
    if let Some(pos) = version.find("-forge-") {
        return &version[..pos];
    }
    version
}
