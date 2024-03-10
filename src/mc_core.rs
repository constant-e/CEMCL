use serde_json::{Result, Value};
use std::env::consts as env;

use crate::file_tools;

// 游戏账号
struct Account {
    // 登录类型，直接填入
    account_type: String,
    // access_token,直接填入
    token: String,
    // uuid，直接填入
    uuid: String,
    // user_name，直接填入
    user_name: String,
}

struct Game {
    // 自定义参数，留空则使用默认
    args: String,
    // 描述，由用户输入
    description: String,
    // 窗口高度，-1默认
    height: i32,
    // java可执行文件路径
    java_path: String,
    // 启用版本隔离
    seperated: bool,
    // 游戏类型，直接填入
    game_type: String,
    // 游戏版本，直接填入
    version: String,
    // 窗口宽度，-1默认
    width: i32,
    // xms参数，留空默认
    xms: String,
    // xmx参数，留空默认
    xmx: String,
}

fn add_args(n: &Value) -> String {
    let mut result = String::new();
    for item in n.as_array().expect("[Error] mc_core: Failed to get arguments.") {
        if item.is_string() {
            // 无限制，可直接添加
            result.push_str(item.as_str().expect("[Error] mc_core: Failed to get arguments."));
            continue;
        }
        
        // 判断是否满足限制条件

        let rules = item.get("rules").expect("[Error] mc_core: Failed to get rules");
        let mut allow = true;

        // 获取操作系统名称
        let mut os = "";
        if env::OS == "windows" || env::OS == "linux" {
            os = env::OS;
        } else if env::OS == "macOS" {
            os = "osx";
        } else {
            os = "";
            println!("[Warning] mc_core: Your OS may not be supported.");
        }

        for r in item.as_array().expect("[Error] mc_core: Failed to get arguments.") {
            if !r["features"].is_null() {
                // 暂时不支持
                allow = false;
                break;
            }
            if r["action"] == "allow" {
                if r["os"]["arch"] != env::ARCH {
                    allow = false;
                    break;
                }
                if r["os"]["name"] != os {
                    allow = false;
                    break;
                }
            } else if r["action"] == "disallow" {
                if r["os"]["arch"] == env::ARCH {
                    allow = false;
                    break;
                }
                if r["os"]["name"] == os {
                    allow = false;
                    break;
                }
            }
        }

        if !allow {
            continue;
        }

        if item["value"].is_string() {
            // 单条参数
            result.push_str(&item["value"].as_str().expect("[Error] mc_core: Failed to get argument."));
            result.push_str(" ");
        } else {
            // 数组
            for arg in item["value"].as_array().expect("[Error] mc_core: Failed to get arguments.") {
                result.push_str(&arg.as_str().expect("[Error] mc_core: Failed to get arguments."));
                result.push_str(" ");
            }
        }
    }
    result

}

fn add_classpaths(n: &Value, game_dir: &String) {
    
}

fn get_launch_args() {

}

fn get_launch_command(account: &Account, game: &Game, java_path: &String, game_dir: &String) -> String {
    println!("[Info] mc_core: Start.");
    let mut result: String = String::new();

    if !game.args.is_empty() {
        result.push_str(java_path.as_str());
        result.push_str(" ");
        result.push_str(game.args.as_str());
        return result;
    }

    let mut jvm_args: String = String::from("-XX:+UseG1GC 
                                             -XX:-UseAdaptiveSizePolicy 
                                             -XX:-OmitStackTraceInFastThrow 
                                             -Dfml.ignoreInvalidMinecraftCertificates=True 
                                             -Dfml.ignorePatchDiscrepancies=True 
                                             -Dlog4j2.formatMsgNoLookups=true ");
    let mut game_args: String = String::new();
    let mut dir = game_dir.clone();
    dir.push_str(game_dir.as_str());
    dir.push_str("/versions/");
    dir.push_str(game.version.as_str());
    
    let mut cfg_path = dir.clone();
    cfg_path.push_str("/");
    cfg_path.push_str(game.version.as_str());
    cfg_path.push_str(".json");
    let config: Value = serde_json::from_str(&file_tools::open_file(&cfg_path).as_str()).expect(&format!("[Error] mc_core: failed to load {cfg_path}."));

    // 判断inheritsFrom(forge需要)
    if config.get("inheritsFrom").is_none() {
        // 无forge


    }

    result
}

