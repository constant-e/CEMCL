use serde_json::json;

/// Minecraft Installation
#[derive(Clone)]
pub struct MCInstallation {
    /// 备注
    pub description: String,

    /// MC自定义参数
    pub game_args: Vec<String>,

    /// 游戏类型，直接填入启动参数
    pub game_type: MCType,

    /// 窗口高度
    pub height: u32,

    /// java可执行文件路径
    pub java_path: String,

    /// JVM自定义参数
    pub jvm_args: Vec<String>,

    /// 版本隔离
    pub separated: bool,

    /// 游戏版本，直接填入启动参数
    pub version: String,

    /// 窗口宽度
    pub width: u32,

    /// 封装器
    pub wrapper: String,

    /// xms参数
    pub xms: String,

    /// xmx参数
    pub xmx: String,
}

#[derive(Clone, PartialEq, Eq)]
pub enum MCType {
    Release,
    Snapshot,
    OldAlpha,
    OldBeta,
}

impl MCType {
    pub fn as_str(&self) -> &str {
        match self {
            MCType::OldAlpha => "old_alpha",
            MCType::OldBeta => "old_beta",
            MCType::Release => "release",
            MCType::Snapshot => "snapshot",
        }
    }
}
