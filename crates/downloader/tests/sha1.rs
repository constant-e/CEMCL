//! 下载任务的 sha1 校验：预期哈希 / 从 `<url>.sha1` 获取并缓存

use downloader::task::{DownloadTask, DownloadTaskStatus};
use reqwest::Client;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use tokio::sync::Semaphore;

const HELLO: &str = "hello world";
const HELLO_SHA1: &str = "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed";

/// 建一个空的临时目录，返回路径
fn temp_dir(name: &str) -> String {
    let dir = std::env::temp_dir().join(format!("cemcl-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.to_str().unwrap().to_string()
}

/// 起一个本地HTTP服务，按路径返回内容，返回地址
fn serve(responses: Vec<(&'static str, String)>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else {
                continue;
            };
            let mut buf = [0u8; 4096];
            let n = stream.read(&mut buf).unwrap_or(0);
            let request = String::from_utf8_lossy(&buf[..n]).into_owned();
            let path = request.split_whitespace().nth(1).unwrap_or("/").to_string();
            let body = responses
                .iter()
                .find(|(p, _)| *p == path)
                .map(|(_, b)| b.clone());
            let response = match body {
                Some(body) => format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                ),
                None => "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    .to_string(),
            };
            let _ = stream.write_all(response.as_bytes());
        }
    });
    format!("http://{addr}")
}

/// 建一个下载任务
fn new_task(url: String, save_path: String) -> DownloadTask {
    DownloadTask::new(url, save_path, Client::new(), Arc::new(Semaphore::new(1)))
}

/// 下载完成后按sha1校验：一致时保留文件，不一致时删除文件并失败
#[tokio::test]
async fn task_verifies_sha1() {
    let dir = temp_dir("sha1");
    let server = serve(vec![("/f", HELLO.to_string())]);

    let path = format!("{dir}/ok");
    let mut task = new_task(format!("{server}/f"), path.clone());
    task.sha1 = Some(HELLO_SHA1.to_string());
    task.start().await.unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), HELLO);
    assert!(matches!(
        *task.status.try_lock().unwrap(),
        DownloadTaskStatus::Completed
    ));

    let path = format!("{dir}/bad");
    let mut task = new_task(format!("{server}/f"), path.clone());
    task.sha1 = Some("0".repeat(40));
    let e = task.start().await.unwrap_err();
    assert!(e.to_string().contains("sha1 mismatch"), "{e}");
    assert!(!std::fs::exists(&path).unwrap());
    assert!(matches!(
        *task.status.try_lock().unwrap(),
        DownloadTaskStatus::Failed
    ));
}

/// 同时给出sha1与sha1_url时以sha1为准，不请求也不缓存sha1_url
#[tokio::test]
async fn task_prefers_sha1_over_sha1_url() {
    let dir = temp_dir("sha1-priority");
    let server = serve(vec![("/f.jar", HELLO.to_string())]);

    let path = format!("{dir}/f.jar");
    let mut task = new_task(format!("{server}/f.jar"), path.clone());
    task.sha1 = Some(HELLO_SHA1.to_string());
    task.sha1_url = Some(format!("{server}/f.jar.sha1"));
    task.start().await.unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), HELLO);
    assert!(!std::fs::exists(format!("{path}.sha1")).unwrap());
}

/// 只有sha1_url时从该地址获取哈希，校验后缓存到<save_path>.sha1（maven格式的文件名不影响解析）
#[tokio::test]
async fn task_verifies_sha1_from_url() {
    let dir = temp_dir("sha1-url");
    let server = serve(vec![
        ("/f.jar", HELLO.to_string()),
        ("/f.jar.sha1", format!("{HELLO_SHA1}  f.jar\n")),
    ]);

    let path = format!("{dir}/f.jar");
    let mut task = new_task(format!("{server}/f.jar"), path.clone());
    task.sha1_url = Some(format!("{server}/f.jar.sha1"));
    task.start().await.unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), HELLO);
    assert_eq!(
        std::fs::read_to_string(format!("{path}.sha1")).unwrap(),
        HELLO_SHA1
    );
}

/// sha1_url 上的哈希与文件不符时任务失败、文件删除，但缓存的哈希保留
#[tokio::test]
async fn task_fails_on_remote_sha1_mismatch() {
    let dir = temp_dir("sha1-url-bad");
    let bad_sha1 = "0".repeat(40);
    let server = serve(vec![
        ("/f.jar", HELLO.to_string()),
        ("/f.jar.sha1", bad_sha1.clone()),
    ]);

    let path = format!("{dir}/f.jar");
    let mut task = new_task(format!("{server}/f.jar"), path.clone());
    task.sha1_url = Some(format!("{server}/f.jar.sha1"));
    let e = task.start().await.unwrap_err();
    assert!(e.to_string().contains("sha1 mismatch"), "{e}");
    assert!(!std::fs::exists(&path).unwrap());
    assert_eq!(
        std::fs::read_to_string(format!("{path}.sha1")).unwrap(),
        bad_sha1
    );
}

/// sha1_url 不可用（404、内容不是sha1）时跳过校验，镜像可能不提供sha1文件
#[tokio::test]
async fn task_skips_verification_without_usable_sha1() {
    let dir = temp_dir("sha1-url-unusable");
    let server = serve(vec![
        ("/f.jar", HELLO.to_string()),
        ("/not-sha1", "not a sha1".to_string()),
    ]);

    for (name, sha1_path) in [("missing", "/missing.sha1"), ("garbage", "/not-sha1")] {
        let path = format!("{dir}/{name}.jar");
        let mut task = new_task(format!("{server}/f.jar"), path.clone());
        task.sha1_url = Some(format!("{server}{sha1_path}"));
        task.start().await.unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), HELLO, "{name}");
        assert!(!std::fs::exists(format!("{path}.sha1")).unwrap(), "{name}");
    }
}
