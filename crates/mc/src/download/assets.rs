//! Download assets

use serde_json::Value;
use std::collections::HashSet;
use std::fs::{create_dir_all, read_to_string};
use std::path::Path;

use super::{DownloadError, DownloadTask, needs_redownload, remove_file_if_exists};

/// 下载assets
pub async fn download_assets(
    path: &Path,
    id: &str,
    mirror: &str,
) -> Result<Vec<DownloadTask>, DownloadError> {
    let assets_dir = path.join("assets");
    let index_path = assets_dir.join("indexes").join(format!("{id}.json"));
    let json = serde_json::from_str::<Value>(&read_to_string(&index_path)?)?;
    let mut tasks = Vec::new();
    // 不同资产名可以引用同一个对象，按 hash 去重，避免多个任务写同一路径
    let mut scheduled = HashSet::new();
    for (_, node) in json["objects"]
        .as_object()
        .ok_or(DownloadError::DataInvalid)?
    {
        let hash = node["hash"].as_str().ok_or(DownloadError::DataInvalid)?;
        if !scheduled.insert(hash) {
            continue;
        }
        let obj_dir = assets_dir.join("objects");
        let save_path = obj_dir.join(&hash[0..2]).join(hash);
        if needs_redownload(&save_path, Some(hash)).await? {
            remove_file_if_exists(&save_path)?;
            create_dir_all(obj_dir.join(&hash[0..2]))?;
            let url = format!("{mirror}/{}/{}", &hash[0..2], hash);
            let mut task = DownloadTask::new(url, save_path, Some(hash.to_string()), None, None);
            task.size = node["size"].as_u64();
            tasks.push(task);
        }
    }

    Ok(tasks)
}
