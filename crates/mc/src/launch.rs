//! 启动相关
//! mc::launch 获取MC的启动参数

use log::error;
use serde_json::Value;
use std::env::consts as env;
use std::fs::{self, exists, read_to_string};

use utils::{check_rules, download};

use crate::MCInstallation;
use crate::account::Account;
use crate::download::{DownloadError, DownloadTask, download_assets, download_libraries};
use crate::launch::LaunchError::{DeserializeError, IOError};

pub enum LaunchError {
    DataInvalid,
    DeserializeError(serde_json::Error),
    IOError(std::io::Error),
    NotFound,
    ReqwestError(reqwest::Error),
}

impl From<std::io::Error> for LaunchError {
    fn from(value: std::io::Error) -> Self {
        IOError(value)
    }
}

impl From<serde_json::Error> for LaunchError {
    fn from(value: serde_json::Error) -> Self {
        DeserializeError(value)
    }
}

impl From<DownloadError> for LaunchError {
    fn from(value: DownloadError) -> Self {
        match value {
            DownloadError::DataInvalid => LaunchError::DataInvalid,
            DownloadError::DeserializeError(err) => LaunchError::DeserializeError(err),
            DownloadError::IOError(err) => LaunchError::IOError(err),
            DownloadError::ReqwestError(err) => LaunchError::ReqwestError(err),
        }
    }
}

/// 从json对象单次获取参数
fn add_arg(n: &Value) -> Result<Vec<String>, LaunchError> {
    let mut result: Vec<String> = Vec::new();

    for item in n.as_array().ok_or(LaunchError::DataInvalid)? {
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
            for arg in item["value"].as_array().ok_or(LaunchError::DataInvalid)? {
                result.push(arg.as_str().ok_or(LaunchError::DataInvalid)?.into());
            }
        }
    }

    Ok(result)
}

/// 获取MC和JVM参数（1.13+）
fn get_args_new(n: &Value) -> Result<(Vec<String>, Vec<String>), LaunchError> {
    let mut game_args: Vec<String> = Vec::new();
    let mut jvm_args: Vec<String> = vec![
        "-XX:+UseG1GC".to_string(),
        "-XX:-UseAdaptiveSizePolicy".to_string(),
        "-XX:-OmitStackTraceInFastThrow".to_string(),
        "-Dfml.ignoreInvalidMinecraftCertificates=True".to_string(),
        "-Dfml.ignorePatchDiscrepancies=True".to_string(),
        "-Dlog4j2.formatMsgNoLookups=true".to_string(),
    ];

    game_args.append(&mut add_arg(&n["arguments"]["game"])?);
    if !n["arguments"]["jvm"].is_null() {
        // forge无此项
        jvm_args.append(&mut add_arg(&n["arguments"]["jvm"])?);
    }

    Ok((game_args, jvm_args))
}

/// 获取MC和JVM参数（1.13-）
fn get_args_old(n: &Value) -> Result<(Vec<String>, Vec<String>), LaunchError> {
    let mut game_args: Vec<String> = Vec::new();
    let mut jvm_args: Vec<String> = vec![
        "-XX:+UseG1GC".to_string(),
        "-XX:-UseAdaptiveSizePolicy".to_string(),
        "-XX:-OmitStackTraceInFastThrow".to_string(),
        "-Dfml.ignoreInvalidMinecraftCertificates=True".to_string(),
        "-Dfml.ignorePatchDiscrepancies=True".to_string(),
        "-Dlog4j2.formatMsgNoLookups=true".to_string(),
    ];

    let args: Vec<&str> = n["minecraftArguments"]
        .as_str()
        .ok_or(LaunchError::DataInvalid)?
        .split(" ")
        .collect();
    for arg in args {
        game_args.push(arg.into());
    }
    jvm_args.append(&mut vec![
        "-Djava.library.path=${natives_directory}".into(),
        "-cp".into(),
        "${classpath}".into(),
    ]);

    Ok((game_args, jvm_args))
}

/// 获取MC和JVM参数（原版）
fn get_args(n: &Value) -> Result<(Vec<String>, Vec<String>), LaunchError> {
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
        let args: Vec<&str> = n["minecraftArguments"]
            .as_str()
            .ok_or(LaunchError::DataInvalid)?
            .split(" ")
            .collect();
        for arg in args {
            game_args.push(arg.into());
        }
        jvm_args.append(&mut vec![
            "-Djava.library.path=${natives_directory}".into(),
            "-cp".into(),
            "${classpath}".into(),
        ]);
    }

    Ok((game_args, jvm_args))
}

/// 获取-cp参数
fn get_classpaths(n: &Value, game_path: &str) -> Result<Vec<String>, LaunchError> {
    let mut result: Vec<String> = Vec::new();
    for item in n.as_array().ok_or(LaunchError::DataInvalid)? {
        if !item["rules"].is_null() && !check_rules(&item["rules"]) {
            continue;
        }

        let mut temp = game_path.to_string() + "/libraries/";

        if let Some(p) = item["downloads"]["artifact"]["path"].as_str() {
            temp += p;
        } else if item["downloads"]["classifiers"].is_object() {
            // classifers for old versions
            let os = if env::OS == "macOS" { "osx" } else { env::OS };
            let arch = if env::ARCH.contains("64") { "64" } else { "32" };
            let key = item["natives"][os]
                .as_str()
                .ok_or(LaunchError::DataInvalid)?
                .replace("${arch}", arch);
            temp += item["downloads"]["classifiers"][&key]["path"]
                .as_str()
                .ok_or(LaunchError::DataInvalid)?;
        } else {
            // fabric
            let mut path = String::new();
            let name = item["name"].as_str().ok_or(LaunchError::DataInvalid)?;
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

    Ok(result)
}

/// 获取启动总命令，返回参数和下载列表
/// Note that all the download sources should be replaced
/// {assets_source}, {fabric_source}, {game_source}, {libraries_source}
pub async fn get_launch_command(
    account: &Account,
    game: &MCInstallation,
    game_path: &str,
) -> Result<(Vec<String>, Vec<DownloadTask>), LaunchError> {
    let mut result: Vec<String> = Vec::new();
    let dir = game_path.to_string() + "/versions/" + game.version.as_str(); // 游戏目录

    // 读取json
    let cfg_path = dir.clone() + "/" + game.version.as_str() + ".json";
    let json = serde_json::from_str::<Value>(read_to_string(&cfg_path)?.as_str())?;

    // mod继承的参数
    let asset_index: String;
    let asset_index_url: String;
    let mc_url: String;

    // mod需要额外写入的参数
    let mut game_args: Vec<String> = game.game_args.clone();
    let mut jvm_args: Vec<String> = game.jvm_args.clone();
    let mut libraries_json = json["libraries"].clone();

    // 判断inheritsFrom（mod需要）
    if json["inheritsFrom"].is_string() {
        // 有mod loader
        let parent_version = json["inheritsFrom"]
            .as_str()
            .ok_or(LaunchError::DataInvalid)?;
        let parent_path = game_path.to_string() + "/versions/" + &parent_version;
        let parent_json_path = parent_path.clone() + "/" + parent_version + ".json";
        if exists(&parent_json_path)? {
            let mut parent =
                serde_json::from_str::<Value>(&read_to_string(&parent_json_path)?.as_str())?;
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
            asset_index = parent["assetIndex"]["id"]
                .as_str()
                .ok_or(LaunchError::DataInvalid)?
                .to_string();

            // MC和JVM的参数
            if !parent["arguments"].is_null() {
                // 1.13以上，jvm参数、game参数追加
                let (mut parent_game_args, mut parent_jvm_args) = get_args_new(&parent)?;
                let (mut self_game_args, mut self_jvm_args) = get_args_new(&json)?;
                game_args.append(&mut parent_game_args);
                game_args.append(&mut self_game_args);
                jvm_args.append(&mut parent_jvm_args);
                jvm_args.append(&mut self_jvm_args);
            } else {
                // 1.13以下，json中无jvm参数，minecraftArguments应覆盖原版的
                let (mut self_game_args, mut self_jvm_args) = get_args(&json)?;
                game_args.append(&mut self_game_args);
                jvm_args.append(&mut self_jvm_args);
            }
        } else {
            error!("Failed to find {parent_path}.");
            return Err(LaunchError::NotFound);
        }
    } else {
        // 无mod loader
        asset_index_url = json["assetIndex"]["url"]
            .as_str()
            .ok_or(std::io::Error::other("Failed to get asset url."))?
            .to_string();
        mc_url = json["downloads"]["client"]["url"]
            .as_str()
            .ok_or(std::io::Error::other("Failed to get mc url."))?
            .to_string();
        let (mut temp_game_args, mut temp_jvm_args) = get_args(&json)?;
        asset_index = json["assetIndex"]["id"]
            .as_str()
            .ok_or(LaunchError::DataInvalid)?
            .to_string();
        game_args.append(&mut temp_game_args);
        jvm_args.append(&mut temp_jvm_args);
    }

    // classpaths列表
    let mut classpaths: Vec<String> = Vec::new();
    classpaths.append(&mut get_classpaths(&libraries_json, game_path)?);
    classpaths.push(dir.clone() + "/" + game.version.as_str() + ".jar"); // 游戏本身

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
        game.height.to_string(),
        "--width".into(),
        game.width.to_string(),
    ]);

    // 参数添加至result
    result.append(&mut jvm_args);
    // 主类
    result.push(
        json["mainClass"]
            .as_str()
            .ok_or(LaunchError::DataInvalid)?
            .to_string(),
    );
    result.append(&mut game_args);

    // 版本隔离
    let game_dir = if game.separated { &dir } else { game_path };

    let os = if env::OS == "macOS" { "osx" } else { env::OS };
    // 替换模板
    for item in result.iter_mut() {
        *item = item
            .replace("${assets_index_name}", &asset_index)
            .replace("${assets_root}", &(game_path.to_string() + "/assets"))
            .replace("${auth_access_token}", &account.access_token)
            .replace("${auth_player_name}", &account.user_name)
            .replace("${auth_uuid}", &account.uuid)
            // .replace("${authlib_injector_param}", "") // 暂不支持
            .replace("${classpath}", &cp)
            .replace("${classpath_separator}", ":")
            .replace("${game_assets}", &(game_path.to_string() + "/assets")) // support old versions
            .replace("${game_directory}", &game_dir)
            .replace("${launcher_name}", "\"CE Minecraft Launcher\"")
            .replace("${launcher_version}", env!("CARGO_PKG_VERSION"))
            .replace(
                "${library_directory}",
                &(game_path.to_string() + "/libraries"),
            )
            .replace(
                "${natives_directory}",
                &(dir.clone() + "/natives-" + os + "-" + env::ARCH),
            )
            .replace("${user_properties}", "{}")
            .replace("${user_type}", &String::from(account.account_type.clone()))
            .replace("${version_name}", &game.version)
            .replace("${version_type}", game.game_type.as_str());
    }

    // 处理依赖
    let mut tasks = Vec::new();
    let jar_path = dir.clone() + "/" + game.version.as_str() + ".jar";
    if !exists(&jar_path)? {
        // 本体
        let url = mc_url
            .clone()
            .replace("https://piston-meta.mojang.com", "{game_source}");
        tasks.push(DownloadTask::new(url, jar_path, None));
    }

    // 处理依赖

    // json first
    let index_dir = game_path.to_string() + "/assets/indexes/";
    let index_path = index_dir.clone() + &asset_index + ".json";
    if !exists(&index_path)? {
        if !exists(&index_dir)? {
            fs::create_dir_all(&index_dir)?;
        }
        futures::executor::block_on(download(asset_index_url.clone(), index_path, 3));
    }

    // assets
    tasks.append(&mut download_assets(
        game_path,
        &asset_index,
        "{assets_source}",
    )?);

    // download libraries
    tasks.append(&mut download_libraries(
        &libraries_json,
        game_path,
        &dir,
        "{libraries_source}",
        "{fabric_source}",
    )?);

    Ok((result, tasks))
}
