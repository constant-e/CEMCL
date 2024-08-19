use log::{info, warn, error};
use serde_json::Value;
use std::{env::consts as env, fs::read_to_string};

use crate::{file_tools::{exists, list_dir}, Config};

// 游戏账号
pub struct Account {
    // 登录类型，直接填入
    pub account_type: String,
    // access_token,直接填入
    pub token: String,
    // uuid，直接填入
    pub uuid: String,
    // user_name，直接填入
    pub user_name: String,
}

// 游戏配置
pub struct Game {
    // 自定义参数，留空则使用默认
    pub args: String,
    // 描述，由用户输入
    pub description: String,
    // 窗口高度
    pub height: isize,
    // java可执行文件路径
    pub java_path: String,
    // 启用版本隔离
    pub seperated: bool,
    // 游戏类型，直接填入
    pub game_type: String,
    // 游戏版本，直接填入
    pub version: String,
    // 窗口宽度
    pub width: isize,
    // xms参数
    pub xms: String,
    // xmx参数
    pub xmx: String,
}

// 获取单条参数
fn add_arg(n: &Value) -> Option<Vec<String>> {
    let mut result: Vec<String> = Vec::new();

    for item in n.as_array()? {
        if item.is_string() {
            // 无限制，可直接添加
            result.push(item.as_str().unwrap().into());
            continue;
        }
        
        // 判断是否满足限制条件
        if !check_rules(&item["rules"]) {
            continue;
        }

        if item["value"].is_string() {
            // 单条参数
            result.push(item["value"].as_str().unwrap().into());
        } else {
            // 数组
            for arg in item["value"].as_array()? {
                result.push(arg.as_str()?.into());
            }
        }
    }

    Some(result)
}

// 检查参数是否可以添加
fn check_rules(n: &Value) -> bool {
    // 获取操作系统名称
    let os: &str;
    if env::OS == "macOS" {
        os = "osx";
    } else {
        os = env::OS;
    }

    if let Some(array) = n.as_array() {
        for r in array {
            if !r["features"].is_null() {
                // 暂时不支持
                return false;
            }
            if r["action"] == "allow" {
                if r["os"]["arch"] != env::ARCH {
                    return false;
                }
                if r["os"]["name"] != os {
                    return false;
                }
            } else if r["action"] == "disallow" {
                if r["os"]["arch"] == env::ARCH {
                    return false;
                }
                if r["os"]["name"] == os {
                    return false;
                }
            }
        }
    } else {
        warn!("Failed to get rules");
    }
    true
}

// 获取MC和JVM参数
fn get_args(n: &Value) -> Option<(Vec<String>, Vec<String>)> {
    let mut game_args:Vec<String> = Vec::new();
    let mut jvm_args:Vec<String> = vec![
        "-XX:+UseG1GC".to_string(),
        "-XX:-UseAdaptiveSizePolicy".to_string(),
        "-XX:-OmitStackTraceInFastThrow".to_string(),
        "-Dfml.ignoreInvalidMinecraftCertificates=True".to_string(),
        "-Dfml.ignorePatchDiscrepancies=True".to_string(),
        "-Dlog4j2.formatMsgNoLookups=true".to_string()
    ];

    if !n["arguments"].is_null() {
        // MC版本 >= 1.13
        game_args.append(&mut add_arg(&n["arguments"]["game"])?);
        jvm_args.append(&mut add_arg(&n["arguments"]["jvm"])?);
    } else {
        // MC版本 < 1.13
        let args: Vec<&str> = n["minecraftArguments"].as_str()?.split(" ").collect();
        for arg in args {
            game_args.push(arg.into());
        }
        jvm_args.append(&mut vec![
            "-Djava.library.path=${natives_directory}".into(), 
            "-cp".into(),
            "${classpath}".into()
        ]);
    }

    Some((game_args, jvm_args))
}

// 获取-cp参数 
fn get_classpaths(n: &Value, game_path: &String) -> Option<Vec<String>> {
    let mut result: Vec<String> = Vec::new();
    for item in n.as_array()? {
        let mut temp: String = game_path.clone() + "/libraries/";

        if !item["rules"].is_null() &&
            check_rules(&item["rules"]) {
            continue;
        }

        let name = String::from(item["name"].as_str()?);
        let name_split: Vec<&str> = name.split(":").collect();
        temp.push_str((
            name_split[0].replace(".", "/") + "/" +
            name_split[1] + "/" +
            name_split[2] + "/" +
            name_split[1] + "-" + name_split[2]
        ).as_str());
        if name_split.len() == 4 {
            // 添加后缀
            temp.push_str(("-".to_string() + name_split[3]).as_str());
        }
        temp.push_str(".jar");

        result.push(temp);
    }

    Some(result)
}

// 获取启动总命令
pub fn get_launch_command(account: &Account, game: &Game, game_path: &String) -> Option<Vec<String>> {
    info!("Start.");
    // 使用自定义参数
    // if !game.args.is_empty() {
    //     return game.args.clone();
    // }
    let mut result: Vec<String> = Vec::new();
    let dir = game_path.clone() + "/versions/" + game.version.as_str();  // 游戏目录
    
    // TODO: 或可与启动时的load_game_list合并
    // 读取json
    let cfg_path = dir.clone() + "/" + game.version.as_str() + ".json";
    if let Ok(config) = serde_json::from_str::<Value>(read_to_string(&cfg_path).ok()?.as_str()) {
        // assetIndex
        let asset_index: String;
        // forge需要提前写入的参数
        let mut classpaths: Vec<String> = Vec::new();
        let mut game_args:Vec<String> = Vec::new();
        let mut jvm_args:Vec<String> = Vec::new();
        // 判断inheritsFrom（forge需要）
        if config["inheritsFrom"].is_null() {
            // 无forge
            if let Some((temp_game_args, temp_jvm_args)) = get_args(&config) {
                game_args = temp_game_args;
                jvm_args = temp_jvm_args;
                if let Some(index) = config["assetIndex"]["id"].as_str() {
                    asset_index = index.into();
                } else {
                    error!("Failed to get assetIndex.");
                    return None;
                }
            } else {
                error!("Failed to get game arguments and jvm arguments.");
                return None;
            }
        } else {
            // 有forge
            if let Some(parent_version) = config["inheritsFrom"].as_str() {
                let parent_path = game_path.clone() + "/versions/" + &parent_version;
                if exists(&parent_path) {
                    if let Ok(parent) = serde_json::from_str::<Value>(&read_to_string(&parent_path).ok()?.as_str()) {
                        if let Some(index) = parent["assetIndex"]["id"].as_str() {
                            asset_index = index.into();
                        } else {
                            error!("Failed to get assetIndex.");
                            return None;
                        }
                        // MC和JVM的参数
                        if let (
                            Some((mut parent_game_args, mut parent_jvm_args)),
                            Some((mut self_game_args, mut self_jvm_args))
                        ) = (get_args(&parent), get_args(&config)) {
                            game_args.append(&mut parent_game_args);
                            game_args.append(&mut self_game_args);
                            jvm_args.append(&mut parent_jvm_args);
                            jvm_args.append(&mut self_jvm_args);
                        } else {
                            error!("Failed to get arguments from {cfg_path}.");
                            return None;
                        }
                        // classpaths列表
                        if let Some(vector) = get_classpaths(&parent["libraries"], game_path) {
                            classpaths = vector;
                        } else {
                            error!("Failed to load classpaths.");
                            return None;
                        }
                    } else {
                        error!("Failed to load {parent_path}.");
                        return None;
                    }
                } else {
                    // TODO: 下载原版
                    return None;
                }
            } else {
                error!("Failed to get inheritsFrom.");
                return None;
            }
        }

        // classpaths列表
        if let Some(mut vector) = get_classpaths(&config["libraries"], game_path) {
            classpaths.append(&mut vector);
        } else {
            error!("Failed to load classpaths.");
            return None;
        }
        classpaths.push(dir.clone() + "/" + game.version.as_str() + ".jar");  // 游戏本身
        
        // classpaths列表去重，获得最终字符串
        let mut i = 0;
        let mut cp = String::new();
        let l = classpaths.len();
        while i < l {
            if !classpaths[i+1..l].contains(&classpaths[i]) {
                cp.push_str((classpaths[i].clone() + ":").as_str());
            }
            i += 1;
        }

        // 设置额外参数 TODO: 更多自定义
        jvm_args.append(&mut vec![
            /*"${authlib_injector_param}".into(), */
            "-Xms".to_string() + &game.xms,
            "-Xmx".to_string() + &game.xmx
        ]);
        game_args.append(&mut vec![
            "--height".into(),
            game.height.to_string(),
            "--width".into(),
            game.width.to_string()
        ]);

        // 参数添加至result
        result.append(&mut jvm_args);
        // 主类
        if let Some(main_class) = config["mainClass"].as_str() {
            result.push(main_class.to_string());
        } else {
            error!("Failed to get mainClass.");
            return None;
        }
        result.append(&mut game_args);

        // 替换模板
        for item in result.iter_mut() {
            // TODO: 优化替换
            *item = item
                .replace("${assets_index_name}", &asset_index)
                .replace("${assets_root}", &(game_path.clone() + "/assets"))
                .replace("${auth_access_token}", &account.token)
                .replace("${auth_player_name}", &account.user_name)
                .replace("${auth_uuid}", &account.uuid)
                // .replace("${authlib_injector_param}", "") // 暂不支持
                .replace("${classpath}", &cp)
                .replace("${classpath_separator}", ":")
                .replace("${game_directory}", &game_path)
                .replace("${launcher_name}", "\"CE Minecraft Launcher\"")
                .replace("${launcher_version}", env!("CARGO_PKG_VERSION"))
                .replace("${library_directory}", &(game_path.clone() + "/libraries"))
                .replace("${natives_directory}", &(dir.clone() + "/natives"))
                .replace("${user_type}", &account.account_type)
                .replace("${version_name}", &game.version)
                .replace("${version_type}", &game.game_type);
        }
        return Some(result)
    } else {
        error!("Failed to load {cfg_path}.");
        return None;
    }
}

pub fn load_game_list(config: &Config) -> Option<Vec<Game>> {
    let mut game_list: Vec<Game> = Vec::new();
    let dir = config.game_path.borrow().clone() + "/versions";

    if !exists(&dir) {
        // 空目录
        warn!("{dir} is empty.");
        return Some(game_list);
    }

    for version in list_dir(&dir)? {
        let mut game: Game;
        let path = dir.clone() + "/" + version.as_str();
        
        // 先加载原版json
        if let Ok(json) = serde_json::from_str::<Value>(&read_to_string(&(path.clone() + "/" + &version.as_str() + ".json")).ok()?.as_str()) {
            game = Game {
                args: "".into(),
                description: "".into(),
                height: config.height.borrow().clone(),
                java_path: config.java_path.borrow().clone(),
                seperated: false,
                game_type: json["type"].as_str()?.into(),
                version: version.into(),
                width: config.width.borrow().clone(),
                xms: config.xms.borrow().clone(),
                xmx: config.xmx.borrow().clone()
            };
        } else {
            // 异常，跳过此次加载
            warn!("Failed to load {version}.json.");
            continue;
        }
        
        // 若config.json存在，覆盖原版json
        let cfg_path = path.clone() + "/" + "config.json";
        if exists(&cfg_path) {
            if let Ok(json) = serde_json::from_str::<Value>(&read_to_string(&cfg_path).ok()?.as_str()) {
                for item in json.as_array()? {
                    game.args = item["args"].as_str()?.into();
                    game.description = item["description"].as_str()?.into();
                    game.height = item["height"].as_i64()? as isize;
                    game.java_path = item["java_path"].as_str()?.into();
                    game.seperated = item["seperated"].as_bool()?;
                    game.width = item["width"].as_i64()? as isize;
                    game.xms = item["xms"].as_str()?.into();
                    game.xmx = item["xmx"].as_str()?.into();
                }
            } else {
                warn!("Failed to load {cfg_path}.");
                continue;
            }
        }
        game_list.push(game);
    };
    Some(game_list)
}
