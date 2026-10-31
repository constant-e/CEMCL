//! 启动相关
//! mc::launch 获取MC的启动参数

use log::error;
use serde_json::Value;
use std::env::consts as env;
use std::fs::{self, exists, read_to_string};
use std::path::{Path, PathBuf};

use utils::{check_rules, download};

use crate::MCInstallation;
use crate::account::Account;
use crate::download::{
    DownloadError, DownloadTask, download_assets, download_libraries, maven_relative_path,
    needs_redownload, remove_file_if_exists,
};
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

impl From<utils::DLError> for LaunchError {
    fn from(value: utils::DLError) -> Self {
        match value {
            utils::DLError::IOError(err) => LaunchError::IOError(err),
            utils::DLError::ReqwestError(err) => LaunchError::ReqwestError(err),
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
fn get_classpaths(n: &Value, game_path: &Path) -> Result<Vec<PathBuf>, LaunchError> {
    let mut result: Vec<PathBuf> = Vec::new();
    for item in n.as_array().ok_or(LaunchError::DataInvalid)? {
        if !item["rules"].is_null() && !check_rules(&item["rules"]) {
            continue;
        }

        let mut temp = game_path.join("libraries");

        if let Some(p) = item["downloads"]["artifact"]["path"].as_str() {
            temp.push(p);
        } else if item["downloads"]["classifiers"].is_object() {
            // classifers for old versions
            let os = if env::OS == "macOS" { "osx" } else { env::OS };
            let arch = if env::ARCH.contains("64") { "64" } else { "32" };
            let key = item["natives"][os]
                .as_str()
                .ok_or(LaunchError::DataInvalid)?
                .replace("${arch}", arch);
            temp.push(
                item["downloads"]["classifiers"][&key]["path"]
                    .as_str()
                    .ok_or(LaunchError::DataInvalid)?,
            );
        } else {
            // fabric
            let name = item["name"].as_str().ok_or(LaunchError::DataInvalid)?;
            temp.push(maven_relative_path(name).ok_or(LaunchError::DataInvalid)?);
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
    game_path: &Path,
) -> Result<(Vec<String>, Vec<DownloadTask>), LaunchError> {
    let mut result: Vec<String> = Vec::new();
    let dir = game_path.join("versions").join(game.version.as_str()); // 游戏目录

    // 读取json
    let cfg_path = dir.join(format!("{}.json", game.version));
    let json = serde_json::from_str::<Value>(read_to_string(&cfg_path)?.as_str())?;

    // mod继承的参数
    let asset_index: String;
    let asset_index_sha1: Option<String>;
    let asset_index_url: String;
    let mc_url: String;
    let mc_sha1: Option<String>;
    let mc_size: Option<u64>;

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
        let parent_path = game_path.join("versions").join(parent_version);
        let parent_json_path = parent_path.join(format!("{parent_version}.json"));
        if exists(&parent_json_path)? {
            let mut parent =
                serde_json::from_str::<Value>(&read_to_string(&parent_json_path)?.as_str())?;
            asset_index_url = parent["assetIndex"]["url"]
                .as_str()
                .ok_or(std::io::Error::other("Failed to get asset url."))?
                .to_string();
            asset_index_sha1 = parent["assetIndex"]["sha1"].as_str().map(String::from);
            mc_url = parent["downloads"]["client"]["url"]
                .as_str()
                .ok_or(std::io::Error::other("Failed to get mc url."))?
                .to_string();
            mc_sha1 = parent["downloads"]["client"]["sha1"]
                .as_str()
                .map(String::from);
            mc_size = parent["downloads"]["client"]["size"].as_u64();
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
            error!("Failed to find {}.", parent_path.display());
            return Err(LaunchError::NotFound);
        }
    } else {
        // 无mod loader
        asset_index_url = json["assetIndex"]["url"]
            .as_str()
            .ok_or(std::io::Error::other("Failed to get asset url."))?
            .to_string();
        asset_index_sha1 = json["assetIndex"]["sha1"].as_str().map(String::from);
        mc_url = json["downloads"]["client"]["url"]
            .as_str()
            .ok_or(std::io::Error::other("Failed to get mc url."))?
            .to_string();
        mc_sha1 = json["downloads"]["client"]["sha1"]
            .as_str()
            .map(String::from);
        mc_size = json["downloads"]["client"]["size"].as_u64();
        let (mut temp_game_args, mut temp_jvm_args) = get_args(&json)?;
        asset_index = json["assetIndex"]["id"]
            .as_str()
            .ok_or(LaunchError::DataInvalid)?
            .to_string();
        game_args.append(&mut temp_game_args);
        jvm_args.append(&mut temp_jvm_args);
    }

    // classpaths列表
    let mut classpaths: Vec<PathBuf> = Vec::new();
    classpaths.append(&mut get_classpaths(&libraries_json, game_path)?);
    classpaths.push(dir.join(format!("{}.jar", game.version))); // 游戏本身

    // classpaths列表去重，获得最终字符串（保留重复项的最后一次出现）
    let sep = if env::OS == "windows" { ';' } else { ':' };
    let mut cp = String::new();
    let l = classpaths.len();
    for i in 0..l {
        if !classpaths[i + 1..l].contains(&classpaths[i]) {
            cp.push_str(&classpaths[i].to_string_lossy());
            cp.push(sep);
        }
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
    let assets_root = game_path.join("assets").to_string_lossy().into_owned();
    let library_dir = game_path.join("libraries").to_string_lossy().into_owned();
    let natives_dir = dir
        .join(format!("natives-{os}-{}", env::ARCH))
        .to_string_lossy()
        .into_owned();
    let game_dir = game_dir.to_string_lossy().into_owned();
    // 替换模板
    for item in result.iter_mut() {
        *item = item
            .replace("${assets_index_name}", &asset_index)
            .replace("${assets_root}", &assets_root)
            .replace("${auth_access_token}", &account.access_token)
            .replace("${auth_player_name}", &account.user_name)
            .replace("${auth_uuid}", &account.uuid)
            // .replace("${authlib_injector_param}", "") // 暂不支持
            .replace("${classpath}", &cp)
            .replace("${classpath_separator}", ":")
            .replace("${game_assets}", &assets_root) // support old versions
            .replace("${game_directory}", &game_dir)
            .replace("${launcher_name}", "\"CE Minecraft Launcher\"")
            .replace("${launcher_version}", env!("CARGO_PKG_VERSION"))
            .replace("${library_directory}", &library_dir)
            .replace("${natives_directory}", &natives_dir)
            .replace("${user_properties}", "{}")
            .replace("${user_type}", &String::from(account.account_type.clone()))
            .replace("${version_name}", &game.version)
            .replace("${version_type}", game.game_type.as_str());
    }

    // 处理依赖
    let mut tasks = Vec::new();
    let jar_path = dir.join(format!("{}.jar", game.version));
    if needs_redownload(&jar_path, mc_sha1.as_deref()).await? {
        remove_file_if_exists(&jar_path)?;
        // 本体
        let url = mc_url
            .clone()
            .replace("https://piston-meta.mojang.com", "{game_source}");
        let mut task = DownloadTask::new(url, jar_path, mc_sha1.clone(), None, None);
        task.size = mc_size;
        tasks.push(task);
    }

    // 处理依赖

    // json first
    let index_dir = game_path.join("assets").join("indexes");
    let index_path = index_dir.join(format!("{asset_index}.json"));
    if needs_redownload(&index_path, asset_index_sha1.as_deref()).await? {
        remove_file_if_exists(&index_path)?;
        fs::create_dir_all(&index_dir)?;
        download(
            asset_index_url.clone(),
            &index_path,
            3,
            asset_index_sha1.as_deref(),
        )
        .await?;
    }

    // assets
    tasks.append(&mut download_assets(game_path, &asset_index, "{assets_source}").await?);

    // download libraries
    tasks.append(
        &mut download_libraries(
            &libraries_json,
            game_path,
            &dir,
            "{libraries_source}",
            "{fabric_source}",
        )
        .await?,
    );

    Ok((result, tasks))
}
