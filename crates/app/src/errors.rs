/// App层统一使用LauncherError
use std::io::ErrorKind;

use downloader::DownloadManagerError;
use mc::account::auth::AuthError;
use mc::{DownloadError, launch::LaunchError};

#[derive(Debug)]
pub enum LauncherError {
    /// account.json invalid
    AccountConfigError,
    /// Auth session is none
    AuthSessionNotFound,
    /// Channel Closed
    ChannelClosed,
    /// Channel Not Fount
    ChannelNotFound,
    /// Client Error
    ClientError(String),
    /// Connection error
    ConnectionError,
    /// Directory is not empty
    DirNotEmpty,
    /// Download Failed
    DownloadFailed(String),
    /// File already exists
    FileAlreadyExists,
    /// File is busy
    FileBusy,
    /// File not found
    FileNotFound,
    /// game config.json invalid
    GameConfigError,
    /// Operation interrupted
    Interrupted,
    /// launcher config.json invalid
    LauncherConfigError,
    /// Login data invalid
    LoginInvalid(&'static str),
    /// Mutex error
    MutexError(String),
    /// Network error
    NetworkError,
    /// Index out of range
    OutOfRange,
    /// Permission denied
    PermissionDenied,
    /// Receive Error
    RecvError,
    /// Errors from reqwest
    ReqwestError(reqwest::Error),
    /// Send Error
    SendError,
    /// Semaphore Error
    SemaphoreError(String),
    /// Task Set Not Found
    TaskSetNotFound,
    /// Weak pointer upgrade error
    WeakPtrError,
    /// Others
    Unknown,
}

impl From<AuthError> for LauncherError {
    fn from(value: AuthError) -> Self {
        match value {
            AuthError::AccessTokenNotFound => LauncherError::LoginInvalid("access_token"),
            AuthError::DeserializationError(err) => err.into(),
            AuthError::DeviceCodeNotFound => LauncherError::LoginInvalid("device_code"),
            AuthError::MSAccessTokenNotFound => {
                LauncherError::LoginInvalid("microsoft access_token")
            }
            AuthError::RefreshTokenNotFound => LauncherError::LoginInvalid("refresh_token"),
            AuthError::ReqwestError(err) => err.into(),
            AuthError::UUIDNotFound => LauncherError::LoginInvalid("uuid"),
            AuthError::UserCodeNotFound => LauncherError::LoginInvalid("user_code"),
            AuthError::UserNameNotFound => LauncherError::LoginInvalid("user_name"),
            AuthError::VerificationUriNotFound => LauncherError::LoginInvalid("verification_uri"),
            AuthError::XSTSTokenNotFound => LauncherError::LoginInvalid("xsts_token"),
            AuthError::XSTSUserHashNotFound => LauncherError::LoginInvalid("user_hash"),
            AuthError::XboxTokenNotFound => LauncherError::LoginInvalid("xbox_token"),
        }
    }
}

impl From<DownloadManagerError> for LauncherError {
    fn from(value: DownloadManagerError) -> Self {
        match value {
            DownloadManagerError::Cancelled => LauncherError::Interrupted,
            DownloadManagerError::ClientError(str) => {
                LauncherError::ClientError(str.unwrap_or("".into()))
            }
            DownloadManagerError::Disconnected => LauncherError::ConnectionError,
            DownloadManagerError::Failed(str) => {
                LauncherError::DownloadFailed(str.unwrap_or("".into()))
            }
            DownloadManagerError::RecvError => LauncherError::RecvError,
            DownloadManagerError::LockError(str) => {
                LauncherError::MutexError(str.unwrap_or("".into()))
            }
            DownloadManagerError::SemaphoreError(str) => {
                LauncherError::SemaphoreError(str.unwrap_or("".into()))
            }
            DownloadManagerError::SendError => LauncherError::SendError,
            DownloadManagerError::TaskSetNotFound => LauncherError::TaskSetNotFound,
        }
    }
}

impl From<DownloadError> for LauncherError {
    fn from(value: DownloadError) -> Self {
        match value {
            DownloadError::DataInvalid => LauncherError::GameConfigError,
            DownloadError::DeserializeError(err) => err.into(),
            DownloadError::IOError(err) => err.into(),
            DownloadError::ReqwestError(err) => err.into(),
        }
    }
}

impl From<LaunchError> for LauncherError {
    fn from(value: LaunchError) -> Self {
        match value {
            LaunchError::DataInvalid => LauncherError::GameConfigError,
            LaunchError::DeserializeError(err) => err.into(),
            LaunchError::IOError(err) => err.into(),
            LaunchError::NotFound => LauncherError::FileNotFound,
            LaunchError::ReqwestError(err) => err.into(),
        }
    }
}

impl From<reqwest::Error> for LauncherError {
    fn from(value: reqwest::Error) -> Self {
        LauncherError::ReqwestError(value.without_url())
    }
}

impl From<std::io::Error> for LauncherError {
    fn from(value: std::io::Error) -> Self {
        match value.kind() {
            ErrorKind::AlreadyExists => LauncherError::FileAlreadyExists,
            ErrorKind::ConnectionAborted
            | ErrorKind::ConnectionRefused
            | ErrorKind::ConnectionReset
            | ErrorKind::NotConnected => LauncherError::ConnectionError,
            ErrorKind::DirectoryNotEmpty => LauncherError::DirNotEmpty,
            ErrorKind::ExecutableFileBusy => LauncherError::FileBusy,
            ErrorKind::NotFound => LauncherError::FileNotFound,
            ErrorKind::Interrupted => LauncherError::Interrupted,
            ErrorKind::NetworkDown | ErrorKind::NetworkUnreachable => LauncherError::NetworkError,
            ErrorKind::PermissionDenied | ErrorKind::ReadOnlyFilesystem => {
                LauncherError::PermissionDenied
            }
            _ => LauncherError::Unknown,
        }
    }
}

impl From<Box<dyn std::error::Error>> for LauncherError {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        value.into()
    }
}

impl From<serde_json::Error> for LauncherError {
    fn from(value: serde_json::Error) -> Self {
        std::io::Error::from(value).into()
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for LauncherError {
    fn from(value: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::ChannelClosed
    }
}

impl std::fmt::Display for LauncherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LauncherError::AccountConfigError => write!(f, "Account config error"),
            LauncherError::AuthSessionNotFound => write!(f, "AuthSession is None"),
            LauncherError::ChannelClosed => write!(f, "Channel closed"),
            LauncherError::ChannelNotFound => write!(f, "Failed to take channel receiver"),
            LauncherError::ClientError(s) => write!(f, "Client error. {s}"),
            LauncherError::ConnectionError => write!(f, "Connection error"),
            LauncherError::DirNotEmpty => write!(f, "Directory is not empty"),
            LauncherError::DownloadFailed(s) => write!(f, "Download failed. {s}"),
            LauncherError::FileAlreadyExists => write!(f, "File already exists"),
            LauncherError::FileBusy => write!(f, "File is busy"),
            LauncherError::FileNotFound => write!(f, "File not found"),
            LauncherError::GameConfigError => write!(f, "Game config error"),
            LauncherError::Interrupted => write!(f, "Operation interrupted"),
            LauncherError::LauncherConfigError => write!(f, "Launcher config error"),
            LauncherError::LoginInvalid(s) => write!(f, "Login data invalid. Failed to find {s}."),
            LauncherError::MutexError(s) => write!(f, "Mutex Lock Error. {s}"),
            LauncherError::NetworkError => write!(f, "Network error"),
            LauncherError::OutOfRange => write!(f, "Index out of range"),
            LauncherError::PermissionDenied => write!(f, "Permission denied"),
            LauncherError::RecvError => write!(f, "Receive error"),
            LauncherError::ReqwestError(err) => write!(f, "{err}"),
            LauncherError::SemaphoreError(s) => write!(f, "Semaphore Error. {s}"),
            LauncherError::SendError => write!(f, "Send error"),
            LauncherError::TaskSetNotFound => write!(f, "Task set not found."),
            LauncherError::WeakPtrError => write!(f, "Failed to upgrade a weak pointer"),
            LauncherError::Unknown => write!(f, "Unknown error"),
        }
    }
}
