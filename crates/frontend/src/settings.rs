//! Settings Page

use crate::ui;

#[derive(Clone)]
pub struct ConfigGeneral {
    /// 启动后关闭启动器
    pub close_after_launch: bool,
    /// .minecraft路径
    pub game_path: String,
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

#[derive(Clone)]
pub struct ConfigMC {
    /// 默认游戏窗口高度
    pub height: u32,
    /// java可执行文件路径
    pub java_path: String,
    /// 默认游戏窗口宽度
    pub width: u32,
    /// 封装器
    pub wrapper: String,
    /// 默认JVM最小内存
    pub xms: String,
    /// 默认JVM最大内存
    pub xmx: String,
    /// MC Path
    pub path: String,
}

/// 启动器配置
#[derive(Clone)]
pub struct Config {
    /// 通用
    pub general: ConfigGeneral,
    /// 下载
    pub dl: ConfigDL,
    /// MC
    pub mc: ConfigMC,
}

impl From<ui::ConfigDL> for ConfigDL {
    fn from(value: ui::ConfigDL) -> Self {
        Self {
            assets_source: value.assets_source.into(),
            concurrency: value.concurrency as u32,
            fabric_source: value.fabric_source.into(),
            forge_source: value.forge_source.into(),
            game_source: value.game_source.into(),
            libraries_source: value.libraries_source.into(),
        }
    }
}

impl From<ui::ConfigGeneral> for ConfigGeneral {
    fn from(value: ui::ConfigGeneral) -> Self {
        Self {
            close_after_launch: value.close_after_launch,
            game_path: value.game_path.into(),
        }
    }
}

impl From<ui::ConfigMC> for ConfigMC {
    fn from(value: ui::ConfigMC) -> Self {
        Self {
            height: value.height as u32,
            java_path: value.java_path.into(),
            width: value.width as u32,
            wrapper: value.wrapper.into(),
            xms: value.xms.into(),
            xmx: value.xmx.into(),
            path: value.path.into(),
        }
    }
}

impl From<ui::Config> for Config {
    fn from(value: ui::Config) -> Self {
        Self {
            dl: value.dl.into(),
            general: value.general.into(),
            mc: value.mc.into(),
        }
    }
}

impl From<ConfigDL> for ui::ConfigDL {
    fn from(value: ConfigDL) -> Self {
        Self {
            assets_source: value.assets_source.into(),
            concurrency: value.concurrency as i32,
            fabric_source: value.fabric_source.into(),
            forge_source: value.forge_source.into(),
            game_source: value.game_source.into(),
            libraries_source: value.libraries_source.into(),
        }
    }
}

impl From<ConfigGeneral> for ui::ConfigGeneral {
    fn from(value: ConfigGeneral) -> Self {
        Self {
            close_after_launch: value.close_after_launch,
            game_path: value.game_path.into(),
        }
    }
}

impl From<ConfigMC> for ui::ConfigMC {
    fn from(value: ConfigMC) -> Self {
        Self {
            height: value.height as i32,
            java_path: value.java_path.into(),
            width: value.width as i32,
            wrapper: value.wrapper.into(),
            xms: value.xms.into(),
            xmx: value.xmx.into(),
            path: value.path.into(),
        }
    }
}

impl From<Config> for ui::Config {
    fn from(value: Config) -> Self {
        Self {
            dl: value.dl.into(),
            general: value.general.into(),
            mc: value.mc.into(),
        }
    }
}
