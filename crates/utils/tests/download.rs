//! 下载与 sha1 校验

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use utils::{DLError, download, sha1_file};

/// 建一个空的临时目录，返回路径
fn temp_dir(name: &str) -> String {
    let dir = std::env::temp_dir().join(format!("cemcl-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.to_str().unwrap().to_string()
}

/// 起一个本地HTTP服务，按路径返回内容，返回（地址，收到的请求数）
fn serve(responses: Vec<(&'static str, &'static str)>) -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let count = Arc::new(AtomicUsize::new(0));
    let counter = count.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else {
                continue;
            };
            counter.fetch_add(1, Ordering::Relaxed);
            let mut buf = [0u8; 4096];
            let n = stream.read(&mut buf).unwrap_or(0);
            let request = String::from_utf8_lossy(&buf[..n]).into_owned();
            let path = request.split_whitespace().nth(1).unwrap_or("/").to_string();
            let body = responses.iter().find(|(p, _)| *p == path).map(|(_, b)| *b);
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
    (format!("http://{addr}"), count)
}

/// sha1 与已知向量一致（大文件跨读取缓冲区）
#[tokio::test]
async fn sha1_file_matches_vectors() {
    let dir = temp_dir("sha1");
    let big = "a".repeat(100_000);
    let cases = [
        ("empty", "", "da39a3ee5e6b4b0d3255bfef95601890afd80709"),
        ("abc", "abc", "a9993e364706816aba3e25717850c26c9cd0d89d"),
        (
            "hello",
            "hello world",
            "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed",
        ),
        (
            "big",
            big.as_str(),
            "c4d4b30851182fc4eb8675494d42fd7f17e29c93",
        ),
    ];
    for (name, content, expect) in cases {
        let path = format!("{dir}/{name}");
        std::fs::write(&path, content).unwrap();
        assert_eq!(sha1_file(&path).await.unwrap(), expect, "{name}");
    }
}

/// 校验通过的下载保留文件，不一致的删除文件并重试
#[tokio::test]
async fn download_verifies_sha1() {
    let dir = temp_dir("download");
    let (server, count) = serve(vec![("/ok", "hello world"), ("/bad", "hello worlD")]);

    let path = format!("{dir}/ok");
    download(
        format!("{server}/ok"),
        path.clone(),
        0,
        Some("2aae6c35c94fcfb415dbe95f408b9ce91ee846ed"),
    )
    .await
    .unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello world");

    // 内容不一致：报错、删除文件，且按max参数重试
    let path = format!("{dir}/bad");
    let e = download(
        format!("{server}/bad"),
        path.clone(),
        1,
        Some("2aae6c35c94fcfb415dbe95f408b9ce91ee846ed"),
    )
    .await
    .unwrap_err();
    match e {
        DLError::IOError(e) => assert!(e.to_string().contains("sha1 mismatch"), "{e}"),
        _ => panic!("unexpected error kind"),
    }
    assert!(!std::fs::exists(&path).unwrap());
    assert_eq!(count.load(Ordering::Relaxed), 3); // 1 次成功 + 2 次失败（含重试）
}
