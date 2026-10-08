//! Download assets

use serde_json::Value;
use std::fs::{create_dir_all, exists, read_to_string};

use super::{DownloadError, DownloadTask, needs_redownload, remove_file_if_exists};

/// 下载assets
pub async fn download_assets(
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
        if needs_redownload(&save_path, Some(hash)).await? {
            remove_file_if_exists(&save_path)?;
            let dir = obj_path.clone() + "/" + &hash[0..2];
            if !exists(&dir)? {
                create_dir_all(&dir)?;
            }
            let url = mirror.to_string() + "/" + &dl_path;
            tasks.push(DownloadTask::new(
                url,
                save_path,
                Some(hash.to_string()),
                None,
                None,
            ));
        }
    }

    Ok(tasks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::download::temp_dir;
    use crate::download::tests::{HELLO, HELLO_SHA1};
    use serde_json::json;

    /// 资源对象按索引中的hash校验：缺失、不一致时下载，一致时跳过
    #[tokio::test]
    async fn existing_asset_is_checked_against_hash() {
        let dir = temp_dir("assets");
        std::fs::create_dir_all(format!("{dir}/assets/indexes")).unwrap();
        std::fs::write(
            format!("{dir}/assets/indexes/test.json"),
            json!({"objects": {"minecraft/test": {"hash": HELLO_SHA1, "size": 11}}}).to_string(),
        )
        .unwrap();
        let save_path = format!("{dir}/assets/objects/{}/{HELLO_SHA1}", &HELLO_SHA1[0..2]);

        // 缺失：下载并带上预期hash
        let tasks = download_assets(&dir, "test", "{assets_source}")
            .await
            .unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].sha1.as_deref(), Some(HELLO_SHA1));

        // 一致：跳过
        std::fs::write(&save_path, HELLO).unwrap();
        let tasks = download_assets(&dir, "test", "{assets_source}")
            .await
            .unwrap();
        assert!(tasks.is_empty());

        // 不一致：删除文件并重新下载
        std::fs::write(&save_path, "hello").unwrap();
        let tasks = download_assets(&dir, "test", "{assets_source}")
            .await
            .unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].save_path, save_path);
        assert!(!exists(&save_path).unwrap());
    }
}
