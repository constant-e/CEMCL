//! 下载清单构建与本地文件的 sha1 校验决策

use mc::download::{download_assets, download_libraries, needs_redownload};
use serde_json::json;
use utils::get_parent_dir;

const HELLO: &str = "hello world";
const HELLO_SHA1: &str = "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed";

/// 建一个空的临时目录，返回路径
fn temp_dir(name: &str) -> String {
    let dir = std::env::temp_dir().join(format!("cemcl-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.to_str().unwrap().to_string()
}

/// 缺失的文件、哈希不一致的文件需要重新下载；一致的跳过，无哈希信息时只按存在性判断
#[tokio::test]
async fn files_are_rechecked_against_sha1() {
    let dir = temp_dir("needs-redownload");
    let path = format!("{dir}/f.jar");
    assert!(needs_redownload(&path, Some(HELLO_SHA1)).await.unwrap());
    assert!(needs_redownload(&path, None).await.unwrap());

    std::fs::write(&path, HELLO).unwrap();
    assert!(!needs_redownload(&path, Some(HELLO_SHA1)).await.unwrap());
    assert!(!needs_redownload(&path, None).await.unwrap());
    // 大小写不同的sha1是同一个
    assert!(
        !needs_redownload(&path, Some(&HELLO_SHA1.to_uppercase()))
            .await
            .unwrap()
    );
    assert!(
        needs_redownload(&path, Some(&"0".repeat(40)))
            .await
            .unwrap()
    );
}

/// 已存在的库文件按json中的sha1校验：一致时跳过，不一致时删除并重新下载
#[tokio::test]
async fn existing_artifact_is_checked_against_sha1() {
    let dir = temp_dir("libraries");
    let save_path = format!("{dir}/libraries/com/example/lib/1.0/lib-1.0.jar");
    let node = json!([{
        "downloads": {"artifact": {
            "path": "com/example/lib/1.0/lib-1.0.jar",
            "sha1": HELLO_SHA1,
            "url": "https://libraries.minecraft.net/com/example/lib/1.0/lib-1.0.jar",
        }},
    }]);

    // 缺失时下载，并带上预期sha1
    let tasks = download_libraries(&node, &dir, &dir, "{libraries_source}", "{fabric_source}")
        .await
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].sha1.as_deref(), Some(HELLO_SHA1));
    assert_eq!(
        tasks[0].url,
        "{libraries_source}/com/example/lib/1.0/lib-1.0.jar"
    );

    // 一致：跳过
    std::fs::create_dir_all(get_parent_dir(&save_path)).unwrap();
    std::fs::write(&save_path, HELLO).unwrap();
    let tasks = download_libraries(&node, &dir, &dir, "{libraries_source}", "{fabric_source}")
        .await
        .unwrap();
    assert!(tasks.is_empty());

    // 不一致：删除文件并重新下载
    std::fs::write(&save_path, "hello").unwrap();
    let tasks = download_libraries(&node, &dir, &dir, "{libraries_source}", "{fabric_source}")
        .await
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].save_path, save_path);
    assert!(!std::fs::exists(&save_path).unwrap());
}

/// fabric库没有json哈希：缺失时下载并从maven的.sha1校验，之后按本地缓存离线校验
#[tokio::test]
async fn fabric_library_uses_cached_sha1() {
    let dir = temp_dir("fabric");
    let node = json!([{
        "name": "net.fabricmc:intermediary:1.21.5",
        "url": "https://maven.fabricmc.net/",
    }]);
    let save_path =
        format!("{dir}/libraries/net/fabricmc/intermediary/1.21.5/intermediary-1.21.5.jar");
    let url =
        "{fabric_source}/net/fabricmc/intermediary/1.21.5/intermediary-1.21.5.jar".to_string();

    // 缺失：下载库文件，并从同路径的.sha1获取哈希校验
    let tasks = download_libraries(&node, &dir, &dir, "{libraries_source}", "{fabric_source}")
        .await
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].url, url);
    assert_eq!(
        tasks[0].sha1_url.as_deref(),
        Some(&format!("{url}.sha1")[..])
    );
    assert_eq!(tasks[0].sha1, None);
    // 下载目录已建好
    assert!(std::fs::exists(get_parent_dir(&save_path)).unwrap());

    // 有缓存且一致：跳过（缓存的sha1允许maven格式的附加文件名与大小写差异）
    std::fs::write(&save_path, HELLO).unwrap();
    std::fs::write(
        format!("{save_path}.sha1"),
        format!("{HELLO_SHA1}  intermediary-1.21.5.jar\n"),
    )
    .unwrap();
    let tasks = download_libraries(&node, &dir, &dir, "{libraries_source}", "{fabric_source}")
        .await
        .unwrap();
    assert!(tasks.is_empty());

    std::fs::write(format!("{save_path}.sha1"), HELLO_SHA1.to_uppercase()).unwrap();
    let tasks = download_libraries(&node, &dir, &dir, "{libraries_source}", "{fabric_source}")
        .await
        .unwrap();
    assert!(tasks.is_empty());

    // 有缓存但不一致：删除并重新下载，缓存保留
    std::fs::write(&save_path, "hello").unwrap();
    let tasks = download_libraries(&node, &dir, &dir, "{libraries_source}", "{fabric_source}")
        .await
        .unwrap();
    assert_eq!(tasks.len(), 1);
    assert!(!std::fs::exists(&save_path).unwrap());
    assert!(std::fs::exists(format!("{save_path}.sha1")).unwrap());

    // 无缓存：没有可用的哈希，保留文件
    std::fs::write(&save_path, "hello").unwrap();
    std::fs::remove_file(format!("{save_path}.sha1")).unwrap();
    let tasks = download_libraries(&node, &dir, &dir, "{libraries_source}", "{fabric_source}")
        .await
        .unwrap();
    assert!(tasks.is_empty());
}

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
    assert!(!std::fs::exists(&save_path).unwrap());
}
