//! mc MC相关

use log::{info, warn};
use serde_json::Value;
use std::env::consts as env;

/// 游戏账号相关
pub mod account;
/// 下载相关
pub mod download;
/// 游戏配置相关
pub mod game;
/// 启动参数相关
pub mod launch;

// 游戏账号
pub struct Account {
    // 登录类型，直接填入
    pub account_type: std::cell::RefCell<String>,
    // access_token,直接填入
    pub token: std::cell::RefCell<String>,
    // uuid，直接填入
    pub uuid: std::cell::RefCell<String>,
    // user_name，直接填入
    pub user_name: std::cell::RefCell<String>,
}

// 游戏配置
pub struct Game {
    // 自定义参数，留空则使用默认
    pub args: std::cell::RefCell<String>,
    // 描述，由用户输入
    pub description: std::cell::RefCell<String>,
    // 窗口高度
    pub height: std::cell::RefCell<String>,
    // java可执行文件路径
    pub java_path: std::cell::RefCell<String>,
    // 启用版本隔离
    pub separated: std::cell::RefCell<bool>,
    // 游戏类型，直接填入
    pub game_type: std::cell::RefCell<String>,
    // 游戏版本，直接填入
    pub version: std::cell::RefCell<String>,
    // 窗口宽度
    pub width: std::cell::RefCell<String>,
    // xms参数
    pub xms: std::cell::RefCell<String>,
    // xmx参数
    pub xmx: std::cell::RefCell<String>,
}

/// 在下载游戏时使用的游戏信息
#[derive(Clone)]
pub struct GameUrl {
    /// 游戏类型
    pub game_type: String,
    /// 本体下载链接
    pub url: String,
    /// 游戏版本
    pub version: String,
}

// 检查参数是否可以添加
fn check_rules(n: &Value) -> bool {
    // 获取操作系统名称
    let os = if env::OS == "macOS" { "osx" } else { env::OS };

    if let Some(array) = n.as_array() {
        for r in array {
            if !r["features"].is_null() {
                // 暂时不支持
                return false;
            }
            if r["os"].is_null() { continue; } // 无意义rule
            if r["action"] == "allow" {
                if r["os"]["arch"].is_string() && r["os"]["arch"] != env::ARCH {
                    info!("ALLOW: {} not match {}", r["os"]["arch"], env::ARCH);
                    return false;
                }
                if r["os"]["name"].is_string() && r["os"]["name"] != os {
                    info!("ALLOW: {} not match {}", r["os"]["name"], os);
                    return false;
                }
            } else if r["action"] == "disallow" {
                if r["os"]["arch"].is_string() && r["os"]["arch"] == env::ARCH {
                    info!("DISALLOW: {} match {}", r["os"]["arch"], env::ARCH);
                    return false;
                }
                if r["os"]["name"].is_string() && r["os"]["name"] == os {
                    info!("DISALLOW: {} match {}", r["os"]["name"], os);
                    return false;
                }
            }
        }
    } else {
        warn!("Failed to get rules");
    }
    true
}
