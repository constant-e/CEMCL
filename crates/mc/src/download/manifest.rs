//! Get Manifests

use serde_json::Value;
use std::fs::{create_dir_all, exists, write};
use utils::download;

use crate::{
    MCType,
    download::{DownloadError, TaskInfo},
};

#[derive(Clone)]
pub struct MCDL {
    pub game_type: MCType,
    pub url: String,
    pub version: String,
}

#[derive(Clone)]
pub struct Fabric {
    pub loader_version: String,
    pub intermediary_version: String,
}

#[derive(Clone)]
pub struct Forge {
    pub version: String,
    pub branch: String,
    pub modified: String,
}

fn mc_type(s: &str) -> Option<MCType> {
    match s {
        "release" => Some(MCType::Release),
        "snapshot" => Some(MCType::Snapshot),
        "old_alpha" => Some(MCType::OldAlpha),
        "old_beta" => Some(MCType::OldBeta),
        _ => None,
    }
}

/// 获取Fabric列表
pub async fn list_fabric(mcversion: &str) -> Result<Vec<Fabric>, DownloadError> {
    let mut fabric_list = Vec::new();

    let url = String::from("https://meta.fabricmc.net/v2/versions/loader/") + mcversion;
    let text = reqwest::get(url).await?.text().await?;
    let json = serde_json::from_str::<Value>(&text)?;

    for version in json.as_array().ok_or(DownloadError::DataInvalid)? {
        let loader_version = version["loader"]["version"]
            .as_str()
            .ok_or(DownloadError::DataInvalid)?
            .to_string();
        let intermediary_version = version["intermediary"]["version"]
            .as_str()
            .ok_or(DownloadError::DataInvalid)?
            .to_string();

        let fabric = Fabric {
            loader_version: loader_version,
            intermediary_version: intermediary_version,
        };

        fabric_list.push(fabric);
    }

    Ok(fabric_list)
}

/// 获取Forge列表 官方没有json，使用BMCLAPI2
pub async fn list_forge(mcversion: &String) -> Result<Vec<Forge>, DownloadError> {
    let mut forge_list = Vec::new();

    let url = String::from("https://bmclapi2.bangbang93.com/forge/minecraft/") + mcversion;
    let text = reqwest::get(url).await?.text().await?;
    let json = serde_json::from_str::<Value>(&text)?;

    for version in json.as_array().ok_or(DownloadError::DataInvalid)? {
        let branch = if let Some(branch) = version["branch"].as_str() {
            branch.to_string()
        } else {
            String::new()
        };

        let modified = version["modified"]
            .as_str()
            .ok_or(DownloadError::DataInvalid)?
            .to_string();

        let forge = Forge {
            version: version["version"]
                .as_str()
                .ok_or(DownloadError::DataInvalid)?
                .to_string(),
            branch: branch,
            modified: modified,
        };

        forge_list.push(forge);
    }

    forge_list.sort_by(|a, b| b.modified.cmp(&a.modified));

    Ok(forge_list)
}

/// 获取下载列表
pub async fn list_game(path: String) -> Result<Vec<MCDL>, DownloadError> {
    let mut game_list = Vec::new();

    // 下载列表
    let text = reqwest::get("https://launchermeta.mojang.com/mc/game/version_manifest_v2.json")
        .await?
        .text()
        .await?;

    // 储存json，与官启一致
    let path = path + "/versions";
    if !exists(&path)? {
        create_dir_all(&path)?;
    }
    write(String::from(path) + "/version_manifest_v2.json", &text)?;

    // 开始解析
    let json = serde_json::from_str::<Value>(&text)?;

    for version in json["versions"]
        .as_array()
        .ok_or(DownloadError::DataInvalid)?
    {
        let game = MCDL {
            game_type: mc_type(version["type"].as_str().ok_or(DownloadError::DataInvalid)?)
                .ok_or(DownloadError::DataInvalid)?,
            url: version["url"]
                .as_str()
                .ok_or(DownloadError::DataInvalid)?
                .to_string(),
            version: version["id"]
                .as_str()
                .ok_or(DownloadError::DataInvalid)?
                .to_string(),
        };
        game_list.push(game);
    }

    Ok(game_list)
}

pub async fn download_fabric(
    mc_path: &str,
    mc_version: &str,
    fabric: Fabric,
) -> Result<(), DownloadError> {
    let name = format!(
        "fabric-loader-{fabric_version}-{mc_version}",
        fabric_version = fabric.loader_version,
    );

    let dir = format!("{mc_path}/versions/{name}");

    if !exists(&dir)? {
        create_dir_all(&dir)?;
    }

    let url = format!(
        "https://meta.fabricmc.net/v2/versions/loader/{mc_version}/{fabric_version}/profile/json",
        fabric_version = fabric.loader_version,
    );
    let save_path = format!("{dir}/{name}.json");

    download(url, save_path, 3).await?;
    Ok(())
}

pub fn download_forge(mcversion: &str, forge: Forge, mirror: &str) -> TaskInfo {
    let forge_url = format!(
        "{mirror}/maven/net/minecraftforge/forge/{mcversion}-{version}/forge-{mcversion}-{version}-installer.jar",
        version = forge.version
    );

    let forge_path = format!(
        "temp-forge-{mcversion}-{version}/forge-{mcversion}-{version}-installer.jar",
        version = forge.version
    );

    TaskInfo {
        url: forge_url,
        save_path: forge_path,
    }
}

pub async fn download_mc(mc_path: &str, mcdl: MCDL) -> Result<(), DownloadError> {
    let dir = mc_path.to_string() + "/versions/" + &mcdl.version;
    let path = dir.clone() + "/" + &mcdl.version + ".json";
    if exists(&path)? {
        return Ok(());
    }
    if !exists(&dir)? {
        create_dir_all(&dir)?;
    }

    download(mcdl.url, path, 3).await?;

    Ok(())
}
