//! 任务集对相同目标路径的任务去重

use downloader::task::TaskInfo;
use downloader::taskset::{TaskSet, TaskSetStatus};
use std::sync::Arc;
use tokio::sync::Semaphore;

fn info(path: String) -> TaskInfo {
    TaskInfo::new(
        "http://127.0.0.1:1/obj".to_string(),
        path,
        Some("0".repeat(40)),
        None,
        None,
        None,
        None,
        None,
    )
}

/// 相同 save_path 的重复任务在进入任务集时被折叠为一个
#[tokio::test]
async fn taskset_deduplicates_same_save_path() {
    let dir = std::env::temp_dir().join(format!("cemcl-test-dedup-{}", std::process::id()));
    let path = dir.join("obj").to_str().unwrap().to_string();
    let set = TaskSet::new(
        reqwest::Client::new(),
        vec![info(path.clone()), info(path)],
        Arc::new(Semaphore::new(2)),
    );

    assert!(matches!(set.get_status(), TaskSetStatus::Pending(1)));
}
