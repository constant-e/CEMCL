use crate::download::DownloadError::{DeserializeError, IOError, ReqwestError};

mod assets;
mod libraries;
pub mod manifest;

pub use assets::download_assets;
pub use libraries::download_libraries;

pub struct TaskInfo {
    pub url: String,
    pub save_path: String,
}

pub struct DownloadTask {
    pub url: String,
    pub save_path: String,
    pub on_finish: Option<Box<dyn Fn() + Send + Sync>>,
}

impl DownloadTask {
    pub fn new(
        url: String,
        save_path: String,
        on_finish: Option<Box<dyn Fn() + Send + Sync>>,
    ) -> Self {
        Self {
            url,
            save_path,
            on_finish,
        }
    }
}

pub enum DownloadError {
    DataInvalid,
    DeserializeError(serde_json::Error),
    IOError(std::io::Error),
    ReqwestError(reqwest::Error),
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
