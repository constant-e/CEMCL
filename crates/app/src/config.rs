//! 启动器配置：config.json 的读取、应用与保存

use downloader::{Config as DownloaderConfig, DownloadManager};
use serde_json::json;
use std::{collections::HashMap, fs};

use crate::{
    LauncherError,
    version::{ConfigMC, VersionManager},
};

#[derive(Clone)]
pub struct ConfigGeneral {
    /// 启动后关闭启动器
    pub close_after_launch: bool,
    /// .minecraft路径
    pub game_path: String,
    /// 进度显示方式
    pub progress_mode: frontend::ProgressMode,
}

#[derive(Clone)]
pub struct ConfigDL {
    /// assets下载源
    pub assets_source: String,
    /// 下载时的最大并发数量
    pub concurrency: u32,
    /// Fabric下载源
    pub fabric_source: String,
    /// Forge下载源
    pub forge_source: String,
    /// MC本体下载源
    pub game_source: String,
    /// libraries下载源
    pub libraries_source: String,
}

impl From<ConfigDL> for DownloaderConfig {
    fn from(value: ConfigDL) -> Self {
        let mut map = HashMap::new();
        map.insert("assets_source".to_string(), value.assets_source);
        map.insert("fabric_source".to_string(), value.fabric_source);
        map.insert("forge_source".to_string(), value.forge_source);
        map.insert("game_source".to_string(), value.game_source);
        map.insert("libraries_source".to_string(), value.libraries_source);
        Self {
            concurrency: value.concurrency,
            mirrors: map,
        }
    }
}

impl From<DownloaderConfig> for ConfigDL {
    fn from(value: DownloaderConfig) -> Self {
        Self {
            concurrency: value.concurrency,
            assets_source: value.mirrors["assets_source"].clone(),
            fabric_source: value.mirrors["fabric_source"].clone(),
            forge_source: value.mirrors["forge_source"].clone(),
            game_source: value.mirrors["game_source"].clone(),
            libraries_source: value.mirrors["libraries_source"].clone(),
        }
    }
}

impl From<frontend::ConfigDL> for ConfigDL {
    fn from(value: frontend::ConfigDL) -> Self {
        Self {
            assets_source: value.assets_source,
            concurrency: value.concurrency,
            fabric_source: value.fabric_source,
            forge_source: value.forge_source,
            game_source: value.game_source,
            libraries_source: value.libraries_source,
        }
    }
}

impl From<frontend::ConfigGeneral> for ConfigGeneral {
    fn from(value: frontend::ConfigGeneral) -> Self {
        Self {
            close_after_launch: value.close_after_launch,
            game_path: value.game_path,
            progress_mode: value.progress_mode,
        }
    }
}

impl From<frontend::ConfigMC> for ConfigMC {
    fn from(value: frontend::ConfigMC) -> Self {
        Self {
            height: value.height,
            path: value.path,
            width: value.width,
            wrapper: value.wrapper,
            xms: value.xms,
            xmx: value.xmx,
        }
    }
}

impl From<ConfigDL> for frontend::ConfigDL {
    fn from(value: ConfigDL) -> Self {
        Self {
            assets_source: value.assets_source,
            concurrency: value.concurrency,
            fabric_source: value.fabric_source,
            forge_source: value.forge_source,
            game_source: value.game_source,
            libraries_source: value.libraries_source,
        }
    }
}

impl From<ConfigGeneral> for frontend::ConfigGeneral {
    fn from(value: ConfigGeneral) -> Self {
        Self {
            close_after_launch: value.close_after_launch,
            game_path: value.game_path,
            progress_mode: value.progress_mode,
        }
    }
}

impl From<ConfigMC> for frontend::ConfigMC {
    fn from(value: ConfigMC) -> Self {
        Self {
            height: value.height,
            path: value.path,
            width: value.width,
            wrapper: value.wrapper,
            xms: value.xms,
            xmx: value.xmx,
        }
    }
}

impl Default for ConfigGeneral {
    fn default() -> Self {
        ConfigGeneral {
            close_after_launch: false,
            game_path: String::from(".minecraft"),
            progress_mode: frontend::ProgressMode::BySize,
        }
    }
}

impl Default for ConfigDL {
    fn default() -> Self {
        ConfigDL {
            assets_source: String::from("https://resources.download.minecraft.net"),
            concurrency: 10,
            fabric_source: String::from("https://maven.fabricmc.net"),
            forge_source: String::from("https://files.minecraftforge.net"),
            game_source: String::from("https://piston-meta.mojang.com"),
            libraries_source: String::from("https://libraries.minecraft.net"),
        }
    }
}

/// 全局配置（config.json），并把修改应用到下载器/版本管理器
pub struct ConfigManager {
    general: ConfigGeneral,
}

impl ConfigManager {
    /// 读取 config.json，同时返回下载与版本配置
    pub fn load() -> Result<(Self, ConfigDL, ConfigMC), LauncherError> {
        if !fs::exists("config.json")? {
            ConfigManager::i_save_config(
                ConfigGeneral::default(),
                ConfigDL::default(),
                ConfigMC::default(),
            )?;
            return Ok((
                Self {
                    general: ConfigGeneral::default(),
                },
                ConfigDL::default(),
                ConfigMC::default(),
            ));
        }

        let mut config_dl = ConfigDL::default();
        let mut config_general = ConfigGeneral::default();
        let mut config_mc = ConfigMC::default();
        let json: serde_json::Value =
            serde_json::from_str(fs::read_to_string("config.json")?.as_str())?;

        config_dl.assets_source = String::from(
            json["assets_source"]
                .as_str()
                .ok_or(LauncherError::LauncherConfigError)?,
        );
        config_general.close_after_launch = json["close_after_launch"]
            .as_bool()
            .ok_or(LauncherError::LauncherConfigError)?;
        config_dl.concurrency = json["concurrency"]
            .as_u64()
            .ok_or(LauncherError::LauncherConfigError)? as u32;
        config_dl.fabric_source = String::from(
            json["fabric_source"]
                .as_str()
                .ok_or(LauncherError::LauncherConfigError)?,
        );
        config_dl.forge_source = String::from(
            json["forge_source"]
                .as_str()
                .ok_or(LauncherError::LauncherConfigError)?,
        );
        config_general.game_path = String::from(
            json["game_path"]
                .as_str()
                .ok_or(LauncherError::LauncherConfigError)?,
        );
        config_general.progress_mode = match json["progress_mode"].as_str() {
            Some("ByNumber") => frontend::ProgressMode::ByNumber,
            Some("Both") => frontend::ProgressMode::Both,
            _ => frontend::ProgressMode::BySize,
        };
        config_dl.game_source = String::from(
            json["game_source"]
                .as_str()
                .ok_or(LauncherError::LauncherConfigError)?,
        );
        config_mc.height = json["height"]
            .as_u64()
            .ok_or(LauncherError::LauncherConfigError)? as u32;
        config_dl.libraries_source = String::from(
            json["libraries_source"]
                .as_str()
                .ok_or(LauncherError::LauncherConfigError)?,
        );
        config_mc.width = json["width"]
            .as_u64()
            .ok_or(LauncherError::LauncherConfigError)? as u32;
        config_mc.wrapper = String::from(
            json["wrapper"]
                .as_str()
                .ok_or(LauncherError::LauncherConfigError)?,
        );
        config_mc.xms = String::from(
            json["xms"]
                .as_str()
                .ok_or(LauncherError::LauncherConfigError)?,
        );
        config_mc.xmx = String::from(
            json["xmx"]
                .as_str()
                .ok_or(LauncherError::LauncherConfigError)?,
        );

        Ok((
            Self {
                general: config_general,
            },
            config_dl,
            config_mc,
        ))
    }

    pub fn general(&self) -> &ConfigGeneral {
        &self.general
    }

    /// 应用设置页的修改，并写回 config.json
    pub fn apply(
        &mut self,
        config: frontend::Config,
        downloader: &mut DownloadManager,
        versions: &mut VersionManager,
    ) -> Result<(), LauncherError> {
        self.general = config.general.into();
        downloader.set_config(ConfigDL::from(config.dl).into());
        versions.set_config(config.mc.into());
        self.save(downloader, versions)
    }

    pub fn save(
        &self,
        downloader: &DownloadManager,
        versions: &VersionManager,
    ) -> Result<(), LauncherError> {
        ConfigManager::i_save_config(
            self.general.clone(),
            downloader.get_config().clone().into(),
            versions.get_config().clone(),
        )
    }

    fn i_save_config(
        config: ConfigGeneral,
        config_dl: ConfigDL,
        config_mc: ConfigMC,
    ) -> Result<(), LauncherError> {
        let json = json!(
            {
                "assets_source": config_dl.assets_source,
                "close_after_launch": config.close_after_launch,
                "concurrency": config_dl.concurrency,
                "fabric_source": config_dl.fabric_source,
                "forge_source": config_dl.forge_source,
                "game_path": config.game_path,
                "game_source": config_dl.game_source,
                "height": config_mc.height,
                "libraries_source": config_dl.libraries_source,
                "progress_mode": match config.progress_mode {
                    frontend::ProgressMode::BySize => "BySize",
                    frontend::ProgressMode::ByNumber => "ByNumber",
                    frontend::ProgressMode::Both => "Both",
                },
                "width": config_mc.width,
                "wrapper": config_mc.wrapper,
                "xms": config_mc.xms,
                "xmx": config_mc.xmx,
            }
        );
        fs::write("config.json", json.to_string())?;
        Ok(())
    }
}
