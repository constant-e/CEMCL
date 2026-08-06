//! Version Manager

use frontend::game::MCInfo;
use log::{error, warn};
use serde_json::json;
use std::{
    collections::HashSet, fs::{create_dir_all, exists, read_to_string, remove_dir_all, write},
};

use mc::{MCInstallation, manifest::MCDL};
use utils::list_dir;

use crate::LauncherError;

#[derive(Clone)]
pub struct ConfigMC {
    /// 默认游戏窗口高度
    pub height: u32,
    /// java可执行文件路径
    pub java_path: String,
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
        let (version_list, current_index) = VersionManager::i_load(config.clone())?;

        Ok(Self {
            config,
            version_list,
            current_index,
        })
    }

    pub fn add(&mut self, version: &MCInstallation) -> Result<(), LauncherError> {
        self.version_list.push(version.clone());

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

    pub fn get_mut(&mut self, index: u32) -> &mut MCInstallation {
        &mut self.version_list[index as usize]
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
        self.config = config
    }

    fn i_load(config: ConfigMC) -> Result<(Vec<MCInstallation>, u32), LauncherError> {
        let mut version_name_set = HashSet::new();
        let mut version_list = Vec::new();
        let mut index = 0;
        let dir = config.path + "/versions";

        if !exists(&dir)? {
            // 空目录
            warn!("{dir} is empty.");
            return Ok((Vec::new(), 0));
        }

        if exists("versions.json")? {
            let json =
                serde_json::from_str::<serde_json::Value>(&read_to_string("versions.json")?)?;

            index = json["current"]
                .as_i64()
                .ok_or(LauncherError::GameConfigError)? as u32;

            for (k, v) in json["versions"]
                .as_object()
                .ok_or(LauncherError::GameConfigError)?
            {
                if !exists(dir.clone() + "/" + k + "/" + k + ".json")? {
                    warn!("{k} is empty");
                    continue;
                }
                let node = v.as_object().ok_or(LauncherError::GameConfigError)?;
                let value = MCInstallation {
                    description: node["description"]
                        .as_str()
                        .ok_or(LauncherError::GameConfigError)?
                        .to_string(),
                    game_args: node["game_args"]
                        .as_array()
                        .ok_or(LauncherError::GameConfigError)?
                        .iter()
                        .map(|arg| {
                            arg.as_str()
                                .ok_or(LauncherError::GameConfigError)
                                .map(|s| s.to_string())
                        })
                        .collect::<Result<Vec<String>, LauncherError>>()?,
                    game_type: to_mc_type(
                        node["game_type"]
                            .as_str()
                            .ok_or(LauncherError::GameConfigError)?,
                    )?,
                    height: node["height"]
                        .as_i64()
                        .ok_or(LauncherError::GameConfigError)? as u32,
                    java_path: node["java_path"]
                        .as_str()
                        .ok_or(LauncherError::GameConfigError)?
                        .to_string(),
                    jvm_args: node["jvm_args"]
                        .as_array()
                        .ok_or(LauncherError::GameConfigError)?
                        .iter()
                        .map(|arg| {
                            arg.as_str()
                                .ok_or(LauncherError::GameConfigError)
                                .map(|s| s.to_string())
                        })
                        .collect::<Result<Vec<String>, LauncherError>>()?,
                    separated: node["separated"]
                        .as_bool()
                        .ok_or(LauncherError::GameConfigError)?,
                    version: k.clone(),
                    width: node["width"]
                        .as_i64()
                        .ok_or(LauncherError::GameConfigError)? as u32,
                    wrapper: node["wrapper"]
                        .as_str()
                        .ok_or(LauncherError::GameConfigError)?
                        .to_string(),
                    xms: node["xms"]
                        .as_str()
                        .ok_or(LauncherError::GameConfigError)?
                        .to_string(),
                    xmx: node["xmx"]
                        .as_str()
                        .ok_or(LauncherError::GameConfigError)?
                        .to_string(),
                };
                version_name_set.insert(k.clone());
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

            let json = serde_json::from_str::<serde_json::Value>(&read_to_string(&path)?.as_str())?;
            let value = MCInstallation {
                description: String::new(),
                game_args: Vec::new(),
                height: config.height,
                java_path: config.java_path.clone(),
                jvm_args: Vec::new(),
                separated: false,
                game_type: to_mc_type(
                    json["type"]
                        .as_str()
                        .ok_or(LauncherError::GameConfigError)?,
                )?,
                version: version,
                width: config.width,
                wrapper: String::new(),
                xms: config.xms.clone(),
                xmx: config.xmx.clone(),
            };

            version_list.push(value);
        }

        Ok((version_list, index))
    }

    pub fn reload(&mut self) -> Result<(), LauncherError> {
        let (version_list, current_index) = VersionManager::i_load(self.config.clone())?;
        self.version_list = version_list;
        self.current_index = current_index;
        Ok(())
    }

    /// 保存（CEMCL格式）
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
                to_json_value(version),
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

fn to_json_value(version: &MCInstallation) -> serde_json::Value {
    json!({
        "description": version.description,
        "game_args": version.game_args,
        "game_type": version.game_type.as_str(),
        "height": version.height,
        "java_path": version.java_path,
        "jvm_args": version.jvm_args,
        "separated": version.separated,
        "version": version.version,
        "width": version.width,
        "wrapper": version.wrapper,
        "xms": version.xms,
        "xmx": version.xmx,
    })
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
        java_path: config.java_path,
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
