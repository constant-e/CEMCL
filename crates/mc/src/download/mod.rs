use log::warn;
use std::fs::exists;
use std::io::ErrorKind;
use utils::sha1_file;

use crate::download::DownloadError::{DeserializeError, IOError, ReqwestError};

mod assets;
mod libraries;
pub mod manifest;

pub use assets::download_assets;
pub use libraries::download_libraries;

pub struct TaskInfo {
    pub url: String,
    pub save_path: String,
    /// 下载后的预期sha1
    pub sha1: Option<String>,
}

pub struct DownloadTask {
    pub url: String,
    pub save_path: String,
    /// 下载后的预期sha1；与`sha1_url`二选一，同时给出时以`sha1`为准
    pub sha1: Option<String>,
    /// 获取预期sha1的地址（maven风格的`<url>.sha1`），下载成功后获取、校验，
    /// 并缓存到`<save_path>.sha1`供离线校验
    pub sha1_url: Option<String>,
    pub on_finish: Option<Box<dyn Fn() + Send + Sync>>,
}

impl DownloadTask {
    pub fn new(
        url: String,
        save_path: String,
        sha1: Option<String>,
        sha1_url: Option<String>,
        on_finish: Option<Box<dyn Fn() + Send + Sync>>,
    ) -> Self {
        Self {
            url,
            save_path,
            sha1,
            sha1_url,
            on_finish,
        }
    }
}

/// 已存在的文件是否需要重新下载：文件缺失，或与预期sha1不一致时为`true`
///
/// 下载器以追加方式写入文件，返回`true`时调用方必须先删除旧文件
pub(crate) async fn needs_redownload(
    path: &str,
    sha1: Option<&str>,
) -> Result<bool, DownloadError> {
    if !exists(path)? {
        return Ok(true);
    }
    let Some(sha1) = sha1 else {
        return Ok(false);
    };

    match sha1_file(path).await {
        Ok(actual) if actual.eq_ignore_ascii_case(sha1) => Ok(false),
        Ok(actual) => {
            warn!("sha1 mismatch for {path}: expected {sha1}, got {actual}, redownloading");
            Ok(true)
        }
        Err(e) => {
            warn!("Failed to hash {path}. Reason: {e}, redownloading");
            Ok(true)
        }
    }
}

/// 删除失效的文件；文件不存在视为已删除
pub(crate) fn remove_file_if_exists(path: &str) -> Result<(), DownloadError> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

/// 读取下载器缓存的`<文件>.sha1`（maven格式，可能附带文件名），无效时返回`None`
pub(crate) fn read_cached_sha1(path: &str) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let token = text.split_whitespace().next()?;
    (token.len() == 40 && token.bytes().all(|b| b.is_ascii_hexdigit()))
        .then(|| token.to_lowercase())
}

/// 建的临时测试目录（<temp>/cemcl-test-<pid>-<name>）
#[cfg(test)]
pub(crate) fn temp_dir(name: &str) -> String {
    let dir = std::env::temp_dir().join(format!("cemcl-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.to_str().unwrap().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) const HELLO: &str = "hello world";
    pub(crate) const HELLO_SHA1: &str = "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed";

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

    /// 本地缓存的sha1按maven格式读取（允许大写、附带文件名，其它内容忽略）
    #[test]
    fn cached_sha1_is_read() {
        let dir = temp_dir("cached-sha1");
        let path = format!("{dir}/f.jar.sha1");
        assert_eq!(read_cached_sha1(&path), None);

        std::fs::write(&path, HELLO_SHA1).unwrap();
        assert_eq!(read_cached_sha1(&path).as_deref(), Some(HELLO_SHA1));

        std::fs::write(&path, format!("{HELLO_SHA1}  f.jar\n")).unwrap();
        assert_eq!(read_cached_sha1(&path).as_deref(), Some(HELLO_SHA1));

        std::fs::write(&path, &HELLO_SHA1.to_uppercase()).unwrap();
        assert_eq!(read_cached_sha1(&path).as_deref(), Some(HELLO_SHA1));

        std::fs::write(&path, "invalid").unwrap();
        assert_eq!(read_cached_sha1(&path), None);
    }

    /// 删除失效文件：存在时删除，不存在时视为已删除
    #[test]
    fn invalid_files_are_removed() {
        let dir = temp_dir("remove-file");
        let path = format!("{dir}/f.jar");
        remove_file_if_exists(&path).unwrap();
        std::fs::write(&path, HELLO).unwrap();
        remove_file_if_exists(&path).unwrap();
        assert!(!exists(&path).unwrap());
    }
}

#[derive(Debug)]
pub enum DownloadError {
    DataInvalid,
    DeserializeError(serde_json::Error),
    IOError(std::io::Error),
    ReqwestError(reqwest::Error),
}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadError::DataInvalid => write!(f, "Invalid data"),
            DownloadError::DeserializeError(e) => write!(f, "{e}"),
            DownloadError::IOError(e) => write!(f, "{e}"),
            DownloadError::ReqwestError(e) => write!(f, "{e}"),
        }
    }
}

impl From<std::io::Error> for DownloadError {
    fn from(value: std::io::Error) -> Self {
        IOError(value)
    }
}

impl From<serde_json::Error> for DownloadError {
    fn from(value: serde_json::Error) -> Self {
        DeserializeError(value)
    }
}

impl From<reqwest::Error> for DownloadError {
    fn from(value: reqwest::Error) -> Self {
        ReqwestError(value)
    }
}

impl From<utils::DLError> for DownloadError {
    fn from(value: utils::DLError) -> Self {
        match value {
            utils::DLError::IOError(err) => IOError(err),
            utils::DLError::ReqwestError(err) => ReqwestError(err),
        }
    }
}
