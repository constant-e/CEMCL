//! mc::launch 获取MC的启动参数

use futures::join;
use log::error;
use std::{env::consts as env, fs};
use serde_json::Value;
use crate::file_tools::exists;
use super::{check_rules, download, Account, Game, Mirrors};

/// 从json对象单次获取参数
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
        if !item["rules"].is_null() &&
            !check_rules(&item["rules"]) {
            continue;
        }

        let mut temp = game_path.clone() + "/libraries/";

        if let Some(p) = item["downloads"]["artifact"]["path"].as_str() {
            temp += p;
        } else {
            // classifers for old versions
            let os = if env::OS == "macOS" { "osx" } else { env::OS };
            let arch = if env::ARCH.contains("64") { "64" } else { "32" };
            let key = item["natives"][os].as_str()?.replace("${arch}", arch);
            temp += item["downloads"]["classifiers"][&key]["path"].as_str()?;
        }

        result.push(temp);
    }

    Some(result)
}

/// 获取启动总命令
pub async fn get_launch_command(account: &Account, game: &Game, game_path: &String, mirrors: &Mirrors) -> Option<Vec<String>> {
    // 使用自定义参数
    // if !game.args.is_empty() {
    //     return game.args.clone();
    // }
    let mut result: Vec<String> = Vec::new();
    let dir = game_path.clone() + "/versions/" + game.version.borrow().as_str();  // 游戏目录
    
    // 读取json
    let cfg_path = dir.clone() + "/" + game.version.borrow().as_str() + ".json";
    if let Ok(config) = serde_json::from_str::<Value>(fs::read_to_string(&cfg_path).ok()?.as_str()) {
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
                    if let Ok(parent) = serde_json::from_str::<Value>(&fs::read_to_string(&parent_path).ok()?.as_str()) {
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
        classpaths.push(dir.clone() + "/" + game.version.borrow().as_str() + ".jar"); // 游戏本身
        
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
            "-Xms".to_string() + game.xms.borrow().as_str(),
            "-Xmx".to_string() + game.xmx.borrow().as_str(),
        ]);
        game_args.append(&mut vec![
            "--height".into(),
            game.height.borrow().clone(),
            "--width".into(),
            game.width.borrow().clone()
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

        // 版本隔离
        let game_dir = if *game.separated.borrow() { &dir } else { game_path };

        let os = if env::OS == "macOS" { "osx" } else { env::OS };
        // 替换模板
        for item in result.iter_mut() {
            
            // TODO: 优化替换
            *item = item
                .replace("${assets_index_name}", &asset_index)
                .replace("${assets_root}", &(game_path.clone() + "/assets"))
                .replace("${auth_access_token}", &account.token.borrow().as_ref())
                .replace("${auth_player_name}", &account.user_name.borrow().as_ref())
                .replace("${auth_uuid}", &account.uuid.borrow().as_ref())
                // .replace("${authlib_injector_param}", "") // 暂不支持
                .replace("${classpath}", &cp)
                .replace("${classpath_separator}", ":")
                .replace("${game_assets}", &(game_path.clone() + "/assets")) // support old versions
                .replace("${game_directory}", &game_dir)
                .replace("${launcher_name}", "\"CE Minecraft Launcher\"")
                .replace("${launcher_version}", env!("CARGO_PKG_VERSION"))
                .replace("${library_directory}", &(game_path.clone() + "/libraries"))
                .replace("${natives_directory}", &(dir.clone() + "/natives-" + os + "-" + env::ARCH))
                .replace("${user_properties}", "{}")
                .replace("${user_type}", &account.account_type.borrow().as_ref())
                .replace("${version_name}", &game.version.borrow().as_ref())
                .replace("${version_type}", &game.game_type.borrow().as_ref());
        }

        // 处理依赖
        // json first
        let index_dir = game_path.clone() + "/assets/indexes/";
        let index_path = index_dir.clone() + &asset_index + ".json";
        if !exists(&index_path) {
            if !exists(&index_dir) { fs::create_dir_all(&index_dir).ok()?; }
            download::download(config["assetIndex"]["url"].as_str()?.to_string(), index_path, 3).await;
        }

        // TODO: support using mirrors

        // assets
        let ass_future = download::download_assets(&game_path, &asset_index, &mirrors.assets_source);
        
        // libraries
        let lib_future = download::download_libraries(&config["libraries"], &game_path, &dir, &mirrors.libraries_source);

        let jar_path = dir.clone() + "/" + game.version.borrow().as_ref() + ".jar";
        if !exists(&jar_path) {
            // 本体
            let url = config["downloads"]["client"]["url"].as_str()?.to_string()
                .replace("https://piston-meta.mojang.com", &mirrors.game_source);
            let future = download::download(url, jar_path, 3);
            join!(future, ass_future, lib_future);
        } else {
            join!(ass_future, lib_future);
        }

        return Some(result)
    } else {
        error!("Failed to load {cfg_path}.");
        return None;
    }
}
