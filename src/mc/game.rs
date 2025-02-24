//! MC游戏相关

use std::fs;

use serde_json::json;

/// 游戏
#[derive(Clone)]
pub struct Game {
    /// 备注
    pub description: String,

    /// MC自定义参数
    pub game_args: Vec<String>,

    /// 游戏类型，直接填入启动参数
    pub game_type: String,

    /// 窗口高度
    pub height: String,

    /// java可执行文件路径
    pub java_path: String,

    /// JVM自定义参数
    pub jvm_args: Vec<String>,

    /// 版本隔离
    pub separated: bool,

    // 游戏版本，直接填入启动参数
    pub version: String,

    // 窗口宽度
    pub width: String,

    // xms参数
    pub xms: String,

    // xmx参数
    pub xmx: String,
}

impl Game {
    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        let json = json!(
            {
                "description": self.description,
                "game_args": self.game_args,
                "height": self.height,
                "java_path": self.java_path,
                "jvm_args": self.jvm_args,
                "separated": self.separated,
                "width": self.width,
                "xms": self.xms,
                "xmx": self.xmx,
            }
        );
        return fs::write(path, json.to_string());
    }
}
