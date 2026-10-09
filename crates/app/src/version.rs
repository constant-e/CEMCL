//! Version Manager

use frontend::game::MCInfo;
use log::{error, warn};
use serde_json::json;
use std::{
    collections::HashSet,
    fs::{exists, read_to_string, remove_dir_all, write},
};

use mc::{MCInstallation, manifest::MCDL};
use utils::list_dir;

use crate::LauncherError;

#[derive(Clone)]
pub struct ConfigMC {
    /// 默认游戏窗口高度
    pub height: u32,
    /// .minecraft位置
    pub path: String,
    /// 默认游戏窗口宽度
    pub width: u32,
    /// 默认封装器
    pub wrapper: String,
    /// 默认JVM最小内存
    pub xms: String,
    /// 默认JVM最大内存
    pub xmx: String,
}

pub struct VersionManager {
    version_list: Vec<MCInstallation>,
    current_index: u32,
    config: ConfigMC,
}

impl VersionManager {
    pub fn new(config: ConfigMC) -> Result<Self, LauncherError> {
        let (version_list, current_index, cleaned) = VersionManager::i_load(config.clone())?;

        let manager = Self {
            config,
            version_list,
            current_index,
        };

        if cleaned {
            // 启动时删除了 versions.json 中的无效项，写回文件
            if let Err(e) = manager.save() {
                error!("{e}");
            }
            if let Err(e) = manager.save_launcher_profiles() {
                error!("{e}");
            }
        }

        Ok(manager)
    }

    pub fn add(&mut self, version: &MCInstallation) -> Result<(), LauncherError> {
        self.version_list.push(version.clone());
        self.version_list.sort_by(|a, b| a.version.cmp(&b.version));

        self.save()?;
        self.save_launcher_profiles()?;
        Ok(())
    }

    pub fn del(&mut self, index: u32) -> Result<(), LauncherError> {
        let version = &self.get(index).version;
        let path = self.config.path.clone() + "/versions/" + version;
        remove_dir_all(path)?;

        self.version_list.remove(index as usize);

        // if index = self.current_index, then switch to another version.
        if self.current_index != 0 && index >= self.current_index {
            self.current_index -= 1;
        }

        self.save()?;
        self.save_launcher_profiles()?;
        Ok(())
    }

    pub fn edit(&mut self, index: u32, version: MCInstallation) -> Result<(), LauncherError> {
        self.version_list[index as usize] = version;

        self.save()?;
        self.save_launcher_profiles()?;
        Ok(())
    }

    pub fn get(&self, index: u32) -> &MCInstallation {
        &self.version_list[index as usize]
    }

    pub fn get_config(&self) -> &ConfigMC {
        &self.config
    }

    pub fn get_version_list(&self) -> &Vec<MCInstallation> {
        &self.version_list
    }

    pub fn get_current_index(&self) -> u32 {
        self.current_index
    }

    pub fn set_config(&mut self, config: ConfigMC) {
        // 未自定义的配置项跟随 config.json 的默认值（与保存时的省略规则一致）
        let old = &self.config;
        for version in &mut self.version_list {
            if version.height == old.height {
                version.height = config.height;
            }
            if version.width == old.width {
                version.width = config.width;
            }
            if version.xms == old.xms {
                version.xms = config.xms.clone();
            }
            if version.xmx == old.xmx {
                version.xmx = config.xmx.clone();
            }
            if version.wrapper == old.wrapper {
                version.wrapper = config.wrapper.clone();
            }
        }
        self.config = config
    }

    /// 为所有版本解析最佳 Java 索引。对每个版本，根据其 MC 版本 JSON 中的 javaVersion 要求，
    /// 在 Java 列表中查找最佳兼容版本。仅在首次加载或 Java 列表变更时调用。
    /// 如果找不到兼容版本，java_index 设为 None（未选择）。
    pub fn resolve_java_indices(&mut self, java_manager: &crate::java::JavaManager) {
        for installation in &mut self.version_list {
            // 如果已经设置了 java_index，跳过
            if installation.java_index.is_some() {
                continue;
            }

            // Read the MC version JSON to get javaVersion requirement
            let json_path = format!(
                "{}/versions/{}/{}.json",
                self.config.path, installation.version, installation.version
            );
            let min_java = if let Ok(content) = std::fs::read_to_string(&json_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    json["javaVersion"]["majorVersion"]
                        .as_u64()
                        .map(|v| ::java::java_version::JavaVersion::from(v.to_string().as_str()))
                } else {
                    None
                }
            } else {
                None
            };

            installation.java_index = java_manager.find_best_java_index(min_java.as_ref());
        }
    }

    /// 加载版本列表：读取 versions.json 并与 .minecraft 核对（缺版本 json 的项会被删除，第三个
    /// 返回值表示是否有过删除），再加入 .minecraft 中未列出的版本。
    ///
    /// 配置项缺省、为 null 或留空（0/空串/空列表）时使用 config 中的默认值。
    fn i_load(config: ConfigMC) -> Result<(Vec<MCInstallation>, u32, bool), LauncherError> {
        let mut version_name_set = HashSet::new();
        let mut version_list = Vec::new();
        let mut current_name = None;
        let mut index = 0usize;
        let mut cleaned = false;
        let dir = config.path + "/versions";

        if !exists(&dir)? {
            // 空目录
            warn!("{dir} is empty.");
            return Ok((Vec::new(), 0, false));
        }

        if exists("versions.json")? {
            let json =
                serde_json::from_str::<serde_json::Value>(&read_to_string("versions.json")?)?;

            index = json["current"].as_u64().unwrap_or(0) as usize;

            let empty = serde_json::Map::new();
            let versions = json["versions"].as_object().unwrap_or(&empty);
            // 用版本名锚定当前版本，以便在删除无效项后重新定位
            current_name = versions.keys().nth(index).cloned();

            for (name, node) in versions {
                let path = dir.clone() + "/" + name + "/" + name + ".json";
                if !exists(&path)? {
                    warn!("Invalid version {name}: {path} not exists, removing it.");
                    cleaned = true;
                    continue;
                }

                let empty_node = serde_json::Map::new();
                // null 等非对象值视为留空
                let node = node.as_object().unwrap_or(&empty_node);

                let value = MCInstallation {
                    description: i_get_str(node, "description")?.unwrap_or_default(),
                    game_args: i_get_list(node, "game_args")?,
                    game_type: match i_get_str(node, "game_type")? {
                        Some(mc_type) => to_mc_type(&mc_type)?,
                        None => i_type_from_json(&path)?,
                    },
                    height: i_get_u32(node, "height")?.unwrap_or(config.height),
                    java_index: i_get_index(node)?,
                    jvm_args: i_get_list(node, "jvm_args")?,
                    separated: i_get_bool(node, "separated")?,
                    version: name.clone(),
                    width: i_get_u32(node, "width")?.unwrap_or(config.width),
                    wrapper: i_get_str(node, "wrapper")?.unwrap_or_else(|| config.wrapper.clone()),
                    xms: i_get_str(node, "xms")?.unwrap_or_else(|| config.xms.clone()),
                    xmx: i_get_str(node, "xmx")?.unwrap_or_else(|| config.xmx.clone()),
                };
                version_name_set.insert(name.clone());
                version_list.push(value);
            }
        }

        // Check .minecraft/versions to add other versions
        for version in list_dir(&dir)? {
            if version_name_set.contains(&version) {
                continue;
            }

            let path = dir.clone() + "/" + &version + "/" + &version + ".json";
            if !exists(&path)? {
                error!("{path} not exists.");
                continue;
            }

            let value = MCInstallation {
                description: String::new(),
                game_args: Vec::new(),
                height: config.height,
                java_index: None, // 未选择，等待用户配置
                jvm_args: Vec::new(),
                separated: false,
                game_type: i_type_from_json(&path)?,
                version,
                width: config.width,
                wrapper: config.wrapper.clone(),
                xms: config.xms.clone(),
                xmx: config.xmx.clone(),
            };

            version_list.push(value);
        }

        // 保持有序（add 也按此排序，current 索引依赖该顺序）
        version_list.sort_by(|a, b| a.version.cmp(&b.version));

        let index = match current_name
            .and_then(|name| version_list.iter().position(|v| v.version == name))
        {
            Some(index) => index as u32,
            // 当前版本不存在时钳制到列表内
            None => index.min(version_list.len().saturating_sub(1)) as u32,
        };

        Ok((version_list, index, cleaned))
    }

    /// 保存（CEMCL格式）。与 config 默认值相同的配置项会省略，以继续跟随 config.json
    pub fn save(&self) -> Result<(), LauncherError> {
        let mut json = json!(
            {
                "current": self.current_index,
                "versions": {}
            }
        );

        for version in &self.version_list {
            serde_json::Map::insert(
                json["versions"]
                    .as_object_mut()
                    .ok_or(LauncherError::GameConfigError)?,
                version.version.clone(),
                to_json_value(version, &self.config),
            );
        }

        write("versions.json", json.to_string())?;

        Ok(())
    }

    /// 保存（官方格式）
    pub fn save_launcher_profiles(&self) -> Result<(), LauncherError> {
        let mut json = json!({"profiles": {}});
        for version in &self.version_list {
            let node = serde_json::json!(
                {
                    "name": version.version,
                    "type": "custom",
                    "lastVersionId": version.version,
                }
            );
            json["profiles"][&version.version] = node;
        }

        write(
            self.config.path.clone() + "/launcher_profiles.json",
            json.to_string(),
        )?;
        Ok(())
    }

    pub fn set_current_index(&mut self, index: u32) -> Result<(), LauncherError> {
        self.current_index = index;
        self.save()
    }
}

/// 将版本转换为 versions.json 中的节点
///
/// 与默认值相同的配置项会省略（缺省即跟随 config.json 的默认值）
fn to_json_value(version: &MCInstallation, config: &ConfigMC) -> serde_json::Value {
    let mut node = serde_json::Map::new();

    node.insert("version".to_string(), version.version.clone().into());
    node.insert("game_type".to_string(), version.game_type.as_str().into());

    if !version.description.is_empty() {
        node.insert(
            "description".to_string(),
            version.description.clone().into(),
        );
    }
    if !version.game_args.is_empty() {
        node.insert("game_args".to_string(), version.game_args.clone().into());
    }
    if version.height != 0 && version.height != config.height {
        node.insert("height".to_string(), version.height.into());
    }
    if let Some(index) = version.java_index {
        node.insert("java_index".to_string(), index.into());
    }
    if !version.jvm_args.is_empty() {
        node.insert("jvm_args".to_string(), version.jvm_args.clone().into());
    }
    if version.separated {
        node.insert("separated".to_string(), true.into());
    }
    if version.width != 0 && version.width != config.width {
        node.insert("width".to_string(), version.width.into());
    }
    if !version.wrapper.is_empty() && version.wrapper != config.wrapper {
        node.insert("wrapper".to_string(), version.wrapper.clone().into());
    }
    if !version.xms.is_empty() && version.xms != config.xms {
        node.insert("xms".to_string(), version.xms.clone().into());
    }
    if !version.xmx.is_empty() && version.xmx != config.xmx {
        node.insert("xmx".to_string(), version.xmx.clone().into());
    }

    node.into()
}

/// 读取可留空的字符串配置项：缺省、null、空串返回 None
fn i_get_str(
    node: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<Option<String>, LauncherError> {
    match node.get(key) {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::String(s)) if !s.is_empty() => Ok(Some(s.clone())),
        Some(serde_json::Value::String(_)) => Ok(None),
        Some(_) => Err(LauncherError::GameConfigError),
    }
}

/// 读取可留空的整数配置项：缺省、null、0 返回 None
fn i_get_u32(
    node: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<Option<u32>, LauncherError> {
    match node.get(key) {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(v) => {
            let value = v.as_u64().ok_or(LauncherError::GameConfigError)?;
            Ok(if value == 0 { None } else { Some(value as u32) })
        }
    }
}

/// 读取可留空的列表配置项：缺省、null、空列表返回空列表
fn i_get_list(
    node: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<Vec<String>, LauncherError> {
    match node.get(key) {
        None | Some(serde_json::Value::Null) => Ok(Vec::new()),
        Some(v) => v
            .as_array()
            .ok_or(LauncherError::GameConfigError)?
            .iter()
            .map(|arg| {
                arg.as_str()
                    .ok_or(LauncherError::GameConfigError)
                    .map(|s| s.to_string())
            })
            .collect(),
    }
}

/// 读取可留空的布尔配置项：缺省、null 返回 false
fn i_get_bool(
    node: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<bool, LauncherError> {
    match node.get(key) {
        None | Some(serde_json::Value::Null) => Ok(false),
        Some(v) => v.as_bool().ok_or(LauncherError::GameConfigError),
    }
}

/// 读取 java 索引：缺省、null 返回 None（0 是合法索引）
fn i_get_index(
    node: &serde_json::Map<String, serde_json::Value>,
) -> Result<Option<u32>, LauncherError> {
    match node.get("java_index") {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(v) => Ok(Some(
            v.as_u64().ok_or(LauncherError::GameConfigError)? as u32
        )),
    }
}

/// 读取 .minecraft 版本 json 中的游戏类型
fn i_type_from_json(path: &str) -> Result<mc::MCType, LauncherError> {
    let json = serde_json::from_str::<serde_json::Value>(&read_to_string(path)?)?;
    to_mc_type(
        json["type"]
            .as_str()
            .ok_or(LauncherError::GameConfigError)?,
    )
}

fn to_mc_type(s: &str) -> Result<mc::MCType, LauncherError> {
    match s {
        "release" => Ok(mc::MCType::Release),
        "snapshot" => Ok(mc::MCType::Snapshot),
        "old_alpha" => Ok(mc::MCType::OldAlpha),
        "old_beta" => Ok(mc::MCType::OldBeta),
        _ => Err(LauncherError::GameConfigError),
    }
}

pub fn frontend_mc_type(mc_type: mc::MCType) -> frontend::game::MCType {
    match mc_type {
        mc::MCType::OldAlpha => frontend::game::MCType::OldAlpha,
        mc::MCType::OldBeta => frontend::game::MCType::OldBeta,
        mc::MCType::Release => frontend::game::MCType::Release,
        mc::MCType::Snapshot => frontend::game::MCType::Snapshot,
    }
}

pub fn frontend_mc_info(version: MCInstallation) -> MCInfo {
    MCInfo {
        description: version.description,
        game_type: frontend_mc_type(version.game_type),
        version: version.version,
    }
}

pub fn frontend_mc_config(config: MCInstallation) -> frontend::game::MCConfig {
    frontend::game::MCConfig {
        description: config.description,
        game_args: config.game_args,
        height: config.height,
        java_index: config.java_index.map(|i| i as i32).unwrap_or(-1),
        jvm_args: config.jvm_args,
        separated: config.separated,
        width: config.width,
        wrapper: config.wrapper,
        xms: config.xms,
        xmx: config.xmx,
    }
}

pub fn frontend_mc_dl(version: MCDL) -> frontend::game::MCDL {
    frontend::game::MCDL {
        game_type: frontend_mc_type(version.game_type),
        url: version.url,
        version: version.version,
    }
}

pub fn frontend_fabric(fabric: mc::manifest::Fabric) -> frontend::game::Fabric {
    frontend::game::Fabric {
        loader_version: fabric.loader_version,
        intermediary_version: fabric.intermediary_version,
    }
}

pub fn frontend_forge(forge: mc::manifest::Forge) -> frontend::game::Forge {
    frontend::game::Forge {
        version: forge.version,
        branch: forge.branch,
        modified: forge.modified,
    }
}
