//! 一些共用工具

use std::env::consts as env;
use log::{debug, warn};

/// 检查参数是否可以添加
pub fn check_rules(n: &serde_json::Value) -> bool {
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
                    debug!("ALLOW: {} not match {}", r["os"]["arch"], env::ARCH);
                    return false;
                }
                if r["os"]["name"].is_string() && r["os"]["name"] != os {
                    debug!("ALLOW: {} not match {}", r["os"]["name"], os);
                    return false;
                }
            } else if r["action"] == "disallow" {
                if r["os"]["arch"].is_string() && r["os"]["arch"] == env::ARCH {
                    debug!("DISALLOW: {} match {}", r["os"]["arch"], env::ARCH);
                    return false;
                }
                if r["os"]["name"].is_string() && r["os"]["name"] == os {
                    debug!("DISALLOW: {} match {}", r["os"]["name"], os);
                    return false;
                }
            }
        }
    } else {
        warn!("Failed to get rules");
    }
    true
}
