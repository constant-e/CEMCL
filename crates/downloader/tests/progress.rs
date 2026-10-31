//! 任务集按大小的进度：未开始的任务也要计入预期大小，
//! 否则下载过程中分母不断增大，进度会先到 100% 再倒退。

use downloader::task::TaskInfo;
use downloader::taskset::TaskSet;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;

fn info(path: &str, expected_size: Option<u64>) -> TaskInfo {
    let mut info = TaskInfo::new(
        "http://127.0.0.1:1/obj".to_string(),
        PathBuf::from(path),
        Some("0".repeat(40)),
        None,
        None,
        None,
        None,
        None,
    );
    info.expected_size = expected_size;
    info
}

/// 任务开始前，预期大小已经计入总量
#[tokio::test]
async fn progress_includes_pending_expected_sizes() {
    let set = TaskSet::new(
        reqwest::Client::new(),
        vec![info("pending-a", Some(100)), info("pending-b", Some(300))],
        Arc::new(Semaphore::new(2)),
    );

    assert_eq!(set.get_progress(), (0, 400));
}

/// 没有元数据大小的任务也给出非零估算，避免进度提前到 100%
#[tokio::test]
async fn progress_estimates_unknown_pending_sizes() {
    let set = TaskSet::new(
        reqwest::Client::new(),
        vec![info("pending-unknown", None)],
        Arc::new(Semaphore::new(1)),
    );

    assert!(set.get_progress().1 > 0);
}
