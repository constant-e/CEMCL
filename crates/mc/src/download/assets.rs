//! Download assets

use serde_json::Value;
use std::fs::{create_dir_all, exists, read_to_string};

use super::{DownloadError, DownloadTask};

/// 下载assets
pub fn download_assets(
    path: &str,
    id: &str,
    mirror: &str,
) -> Result<Vec<DownloadTask>, DownloadError> {
    let assets_dir = path.to_string() + "/assets";
    let index_path = assets_dir.clone() + "/indexes/" + &id + ".json";
    let json = serde_json::from_str::<Value>(&read_to_string(&index_path)?)?;
    let mut tasks = Vec::new();
    for (_, node) in json["objects"]
        .as_object()
        .ok_or(DownloadError::DataInvalid)?
    {
        let hash = node["hash"].as_str().ok_or(DownloadError::DataInvalid)?;
        let dl_path = hash[0..2].to_string() + "/" + hash;
        let obj_path = assets_dir.clone() + "/objects";
        let save_path = obj_path.clone() + "/" + &dl_path;
        if !exists(&save_path)? {
            let dir = obj_path.clone() + "/" + &hash[0..2];
            if !exists(&dir)? {
                create_dir_all(&dir)?;
            }
            let url = mirror.to_string() + "/" + &dl_path;
            tasks.push(DownloadTask::new(url, save_path, None));
        } else {
            // TODO: check hash
        }
    }

    Ok(tasks)
}
