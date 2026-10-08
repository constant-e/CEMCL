//! utils

use log::{info, warn};
use sha1::{Digest, Sha1};
use std::env::consts as env;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use tokio::io::AsyncReadExt;

/// 检查参数是否可以添加
pub fn check_rules(n: &serde_json::Value) -> bool {
    // 获取操作系统名称
    let os = if env::OS == "macOS" { "osx" } else { env::OS };

    if let Some(array) = n.as_array() {
        for r in array {
            if !r["features"].is_null() {
                // 暂时不支持
                return false;
            }
            if r["os"].is_null() {
                continue;
            } // 无意义rule
            if r["action"] == "allow" {
                if r["os"]["arch"].is_string() && r["os"]["arch"] != env::ARCH {
                    // debug!("ALLOW: {} not match {}", r["os"]["arch"], env::ARCH);
                    return false;
                }
                if r["os"]["name"].is_string() && r["os"]["name"] != os {
                    // debug!("ALLOW: {} not match {}", r["os"]["name"], os);
                    return false;
                }
            } else if r["action"] == "disallow" {
                if r["os"]["arch"].is_string() && r["os"]["arch"] == env::ARCH {
                    // debug!("DISALLOW: {} match {}", r["os"]["arch"], env::ARCH);
                    return false;
                }
                if r["os"]["name"].is_string() && r["os"]["name"] == os {
                    // debug!("DISALLOW: {} match {}", r["os"]["name"], os);
                    return false;
                }
            }
        }
    } else {
        warn!("Failed to get rules");
    }
    true
}

#[derive(Debug)]
pub enum DLError {
    IOError(tokio::io::Error),
    ReqwestError(reqwest::Error),
}

impl std::fmt::Display for DLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DLError::IOError(e) => write!(f, "{e}"),
            DLError::ReqwestError(e) => write!(f, "{e}"),
        }
    }
}

impl From<reqwest::Error> for DLError {
    fn from(value: reqwest::Error) -> Self {
        DLError::ReqwestError(value)
    }
}

impl From<tokio::io::Error> for DLError {
    fn from(value: tokio::io::Error) -> Self {
        DLError::IOError(value)
    }
}

/// 复用的http客户端，带连接/整体超时，避免网络卡死时无限等待
static CLIENT: std::sync::LazyLock<reqwest::Client> = std::sync::LazyLock::new(|| {
    reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .unwrap_or_default()
});

/// 计算文件的sha1（小写十六进制）
pub async fn sha1_file(path: &str) -> std::io::Result<String> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut hasher = Sha1::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(to_hex(&hasher.finalize()))
}

/// 字节序列转小写十六进制
pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// 下载单个文件，用于下载json
///
/// `sha1` 非空时校验下载内容，不一致则删除文件后重试
pub async fn download(
    url: String,
    path: String,
    max: usize,
    sha1: Option<&str>,
) -> Result<(), DLError> {
    info!("Start downloading {url}");
    let mut c = 0; // retry times
    loop {
        match download_once(&url, &path, sha1).await {
            Ok(()) => {
                info!("Finish downloading {url}");
                return Ok(());
            }
            Err(e) => {
                if c >= max {
                    return Err(e);
                }
                warn!("Failed to download {url}. Reason: {e}. Retrying.");
                tokio::time::sleep(std::time::Duration::from_millis(500 * (c as u64 + 1))).await;
                c += 1;
            }
        }
    }
}

/// 下载并校验一次，校验失败时删除文件（不删除的话重试就会跳过写入）
async fn download_once(url: &str, path: &str, sha1: Option<&str>) -> Result<(), DLError> {
    let bytes = CLIENT.get(url).send().await?.bytes().await?;
    tokio::fs::write(path, bytes).await?;
    let Some(sha1) = sha1 else {
        return Ok(());
    };

    let actual = match sha1_file(path).await {
        Ok(actual) => actual,
        Err(e) => {
            remove_invalid(path).await;
            return Err(e.into());
        }
    };
    if actual.eq_ignore_ascii_case(sha1) {
        return Ok(());
    }
    remove_invalid(path).await;
    Err(DLError::IOError(std::io::Error::other(format!(
        "sha1 mismatch for {path}: expected {sha1}, got {actual}"
    ))))
}

/// 删除校验失败的文件，删除失败只记录日志（重试时会覆盖写入）
async fn remove_invalid(path: &str) {
    if let Err(e) = tokio::fs::remove_file(path).await {
        warn!("Failed to remove the invalid file {path}. Reason: {e}");
    }
}

/// 获取文件所在文件夹
pub fn get_parent_dir(path: &str) -> String {
    Path::new(path)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .to_string()
}

/// 列出目录下所有文件夹
pub fn list_dir(path: &String) -> std::io::Result<Vec<String>> {
    let mut result: Vec<String> = Vec::new();
    for entry in fs::read_dir(&Path::new(path))? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        result.push(
            path.file_name()
                .ok_or(ErrorKind::InvalidData)?
                .to_str()
                .ok_or(ErrorKind::InvalidData)?
                .into(),
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

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
                    None => {
                        "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                            .to_string()
                    }
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
}
