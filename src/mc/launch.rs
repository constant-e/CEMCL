//! 启动相关

//! mc::launch 获取MC的启动参数

use crate::app::Config;
use crate::downloader::downloader::Downloader;
use futures::executor::block_on;
use log::error;
use serde_json::Value;
use std::env::consts as env;
use std::fs::{self, exists};
use std::thread::sleep;
use std::time::Duration;

use super::{Account, Game, check_rules, download};

/// 完整下载游戏使用的信息
pub struct GameDownload {
    /// assetIndex
    pub asset_index: String,

    /// asset index json的下载url
    pub asset_index_url: String,

    /// 游戏所在路径
    pub dir: String,

    /// 下载libraries所用json
    pub libraries_json: serde_json::Value,

    /// 本体url
    pub mc_url: String,

    /// 版本
    pub version: String,
}

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

/// 获取MC和JVM参数
fn get_args(n: &Value) -> Option<(Vec<String>, Vec<String>)> {
    let mut game_args: Vec<String> = Vec::new();
    let mut jvm_args: Vec<String> = vec![
        "-XX:+UseG1GC".to_string(),
        "-XX:-UseAdaptiveSizePolicy".to_string(),
        "-XX:-OmitStackTraceInFastThrow".to_string(),
        "-Dfml.ignoreInvalidMinecraftCertificates=True".to_string(),
        "-Dfml.ignorePatchDiscrepancies=True".to_string(),
        "-Dlog4j2.formatMsgNoLookups=true".to_string(),
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
            "${classpath}".into(),
        ]);
    }

    Some((game_args, jvm_args))
}

/// 获取-cp参数
fn get_classpaths(n: &Value, game_path: &String) -> Option<Vec<String>> {
    let mut result: Vec<String> = Vec::new();
    for item in n.as_array()? {
        if !item["rules"].is_null() && !check_rules(&item["rules"]) {
            continue;
        }

        let mut temp = game_path.clone() + "/libraries/";

        if let Some(p) = item["downloads"]["artifact"]["path"].as_str() {
            temp += p;
        } else if item["downloads"]["classifiers"].is_object() {
            // classifers for old versions
            let os = if env::OS == "macOS" { "osx" } else { env::OS };
            let arch = if env::ARCH.contains("64") { "64" } else { "32" };
            let key = item["natives"][os].as_str()?.replace("${arch}", arch);
            temp += item["downloads"]["classifiers"][&key]["path"].as_str()?;
        } else {
            // fabric
            let mut path = String::new();
            let name = item["name"].as_str()?;
            let split_1: Vec<&str> = name.split(":").collect();
            let split_2: Vec<&str> = split_1[0].split(".").collect();
            for name in split_2 {
                path = path + name + "/";
            }
            for i in 1..split_1.len() {
                let name = split_1[i];
                path = path + name + "/";
            }
            path = path + split_1[1] + "-" + split_1[2] + ".jar";
            temp += &path;
        }

        result.push(temp);
    }

    Some(result)
}

/// 获取启动总命令
pub async fn get_launch_command(
    account: &Account,
    game: &Game,
    config: &Config,
) -> Result<(Vec<String>, GameDownload), std::io::Error> {
    let mut result: Vec<String> = Vec::new();
    let game_path = &config.game_path;
    let dir = game_path.clone() + "/versions/" + game.version.as_str(); // 游戏目录

    // 读取json
    let cfg_path = dir.clone() + "/" + game.version.as_str() + ".json";
    if let Ok(json) = serde_json::from_str::<Value>(fs::read_to_string(&cfg_path)?.as_str()) {
        // mod继承的参数
        let asset_index: String;
        let asset_index_url: String;
        let mc_url: String;

        // mod需要额外写入的参数
        let mut classpaths: Vec<String> = Vec::new();
        let mut game_args: Vec<String> = game.game_args.clone();
        let mut jvm_args: Vec<String> = game.jvm_args.clone();
        let mut libraries_json = json["libraries"].clone();
        // 判断inheritsFrom（mod需要）
        if json["inheritsFrom"].is_null() {
            // 无mod loader
            asset_index_url = json["assetIndex"]["url"]
                .as_str()
                .ok_or(std::io::Error::other("Failed to get asset url."))?
                .to_string();
            mc_url = json["downloads"]["client"]["url"]
                .as_str()
                .ok_or(std::io::Error::other("Failed to get mc url."))?
                .to_string();
            classpaths.push(dir.clone() + "/" + game.version.as_str() + ".jar"); // 游戏本身
            if let Some((mut temp_game_args, mut temp_jvm_args)) = get_args(&json) {
                game_args.append(&mut temp_game_args);
                jvm_args.append(&mut temp_jvm_args);
                if let Some(index) = json["assetIndex"]["id"].as_str() {
                    asset_index = index.into();
                } else {
                    error!("Failed to get assetIndex.");
                    return Err(std::io::Error::other("Failed to get assetIndex."));
                }
            } else {
                error!("Failed to get game arguments and jvm arguments.");
                return Err(std::io::Error::other(
                    "Failed to get game arguments and jvm arguments.",
                ));
            }
        } else {
            // 有mod loader
            if let Some(parent_version) = json["inheritsFrom"].as_str() {
                let parent_path = game_path.clone() + "/versions/" + &parent_version;
                let parent_json_path = parent_path.clone() + "/" + parent_version + ".json";
                if exists(&parent_json_path)? {
                    if let Ok(mut parent) = serde_json::from_str::<Value>(
                        &fs::read_to_string(&parent_json_path)?.as_str(),
                    ) {
                        asset_index_url = parent["assetIndex"]["url"]
                            .as_str()
                            .ok_or(std::io::Error::other("Failed to get asset url."))?
                            .to_string();
                        mc_url = parent["downloads"]["client"]["url"]
                            .as_str()
                            .ok_or(std::io::Error::other("Failed to get mc url."))?
                            .to_string();
                        libraries_json
                            .as_array_mut()
                            .ok_or(std::io::Error::other("Failed to library list."))?
                            .append(
                                parent["libraries"]
                                    .as_array_mut()
                                    .ok_or(std::io::Error::other("Failed to library list."))?,
                            );
                        if let Some(index) = parent["assetIndex"]["id"].as_str() {
                            asset_index = index.into();
                        } else {
                            error!("Failed to get assetIndex.");
                            return Err(std::io::Error::other("Failed to get assetIndex."));
                        }
                        // MC和JVM的参数
                        if let (
                            Some((mut parent_game_args, mut parent_jvm_args)),
                            Some((mut self_game_args, mut self_jvm_args)),
                        ) = (get_args(&parent), get_args(&json))
                        {
                            game_args.append(&mut parent_game_args);
                            game_args.append(&mut self_game_args);
                            jvm_args.append(&mut parent_jvm_args);
                            jvm_args.append(&mut self_jvm_args);
                        } else {
                            error!("Failed to get arguments from {cfg_path}.");
                            return Err(std::io::Error::other(format!(
                                "Failed to get arguments from {cfg_path}."
                            )));
                        }
                        // classpaths列表
                        classpaths.push(parent_path.clone() + "/" + parent_version + ".jar"); // 游戏本身
                        if let Some(vector) = get_classpaths(&parent["libraries"], game_path) {
                            classpaths = vector;
                        } else {
                            error!("Failed to load classpaths.");
                            return Err(std::io::Error::other("Failed to load classpaths."));
                        }
                    } else {
                        error!("Failed to load {parent_path}.");
                        return Err(std::io::Error::other(format!(
                            "Failed to load {parent_path}."
                        )));
                    }
                } else {
                    // TODO: 下载原版
                    error!("Failed to find {parent_path}.");
                    return Err(std::io::Error::other(format!(
                        "Failed to find {parent_path}."
                    )));
                }
            } else {
                error!("Failed to get inheritsFrom.");
                return Err(std::io::Error::other("Failed to get inheritsFrom."));
            }
        }

        // classpaths列表
        if let Some(mut vector) = get_classpaths(&json["libraries"], game_path) {
            classpaths.append(&mut vector);
        } else {
            error!("Failed to load classpaths.");
            return Err(std::io::Error::other("Failed to load classpaths."));
        }

        // classpaths列表去重，获得最终字符串
        let sep = if env::OS == "windows" { ";" } else { ":" };
        let mut i = 0;
        let mut cp = String::new();
        let l = classpaths.len();
        while i < l {
            if !classpaths[i + 1..l].contains(&classpaths[i]) {
                cp.push_str((classpaths[i].clone() + sep).as_str());
            }
            i += 1;
        }

        // 设置额外参数
        jvm_args.append(&mut vec![
            /*"${authlib_injector_param}".into(), */
            "-Xms".to_string() + game.xms.as_str(),
            "-Xmx".to_string() + game.xmx.as_str(),
        ]);
        game_args.append(&mut vec![
            "--height".into(),
            game.height.clone(),
            "--width".into(),
            game.width.clone(),
        ]);

        // 参数添加至result
        result.append(&mut jvm_args);
        // 主类
        if let Some(main_class) = json["mainClass"].as_str() {
            result.push(main_class.to_string());
        } else {
            error!("Failed to get mainClass.");
            return Err(std::io::Error::other("Failed to get mainClass."));
        }
        result.append(&mut game_args);

        // 版本隔离
        let game_dir = if game.separated { &dir } else { game_path };

        let os = if env::OS == "macOS" { "osx" } else { env::OS };
        // 替换模板
        for item in result.iter_mut() {
            *item = item
                .replace("${assets_index_name}", &asset_index)
                .replace("${assets_root}", &(game_path.clone() + "/assets"))
                .replace("${auth_access_token}", &account.access_token)
                .replace("${auth_player_name}", &account.user_name)
                .replace("${auth_uuid}", &account.uuid)
                // .replace("${authlib_injector_param}", "") // 暂不支持
                .replace("${classpath}", &cp)
                .replace("${classpath_separator}", ":")
                .replace("${game_assets}", &(game_path.clone() + "/assets")) // support old versions
                .replace("${game_directory}", &game_dir)
                .replace("${launcher_name}", "\"CE Minecraft Launcher\"")
                .replace("${launcher_version}", env!("CARGO_PKG_VERSION"))
                .replace("${library_directory}", &(game_path.clone() + "/libraries"))
                .replace(
                    "${natives_directory}",
                    &(dir.clone() + "/natives-" + os + "-" + env::ARCH),
                )
                .replace("${user_properties}", "{}")
                .replace("${user_type}", &account.account_type)
                .replace("${version_name}", &game.version)
                .replace("${version_type}", &game.game_type);
        }

        // 处理依赖
        let game_download = GameDownload {
            asset_index,
            asset_index_url,
            dir,
            libraries_json,
            mc_url,
            version: game.version.clone(),
        };

        Ok((result, game_download))
    } else {
        error!("Failed to load {cfg_path}.");
        Err(std::io::Error::other(format!("Failed to load {cfg_path}.")))
    }
}

pub fn download_all(
    config: &Config,
    game: &GameDownload,
    downloader: &Downloader,
) -> Result<(), std::io::Error> {
    let jar_path = game.dir.clone() + "/" + game.version.as_ref() + ".jar";
    if !exists(&jar_path)? {
        // 本体
        let url = game
            .mc_url
            .clone()
            .replace("https://piston-meta.mojang.com", &config.game_source);
        if let Err(e) = downloader.add(url, jar_path) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("{e}"),
            ));
        }
    }

    // 处理依赖

    // json first
    let index_dir = config.game_path.clone() + "/assets/indexes/";
    let index_path = index_dir.clone() + &game.asset_index + ".json";
    if !exists(&index_path)? {
        if !exists(&index_dir)? {
            fs::create_dir_all(&index_dir)?;
        }
        block_on(download::download(
            game.asset_index_url.clone(),
            index_path,
            3,
        ));
    }

    // assets
    download::download_assets(
        &config.game_path,
        &game.asset_index,
        &config.assets_source,
        downloader,
    )?;

    // download libraries
    let natives = download::download_libraries(
        &game.libraries_json,
        &config.game_path,
        &game.dir,
        &config.libraries_source,
        &config.fabric_source,
        downloader,
    )?;

    while downloader.in_progress().ok_or(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Downloader Error",
    ))? {
        sleep(Duration::from_millis(10));
        if downloader.has_error() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Downloader Error",
            ));
        }
    }

    if downloader.has_error() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Downloader Error",
        ));
    }

    // extract natives
    for (natives_dir, local_path, id) in natives {
        download::extract_lib(&natives_dir, &local_path, &id)?;
    }

    Ok(())
}
