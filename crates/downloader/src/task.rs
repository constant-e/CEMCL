use futures::StreamExt;
use log::{error, info, warn};
use reqwest::Client;
use sha1::{Digest, Sha1};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{Mutex, RwLock, Semaphore, mpsc::error::TryRecvError},
    time::Duration,
};

#[derive(Debug)]
pub enum DownloadTaskError {
    Cancelled,
    ClientError(Option<String>),
    Disconnected,
    Failed(Option<String>),
    LockError(Option<String>),
    SemaphoreError(Option<String>),
    SendError,
    RecvError,
}

impl std::fmt::Display for DownloadTaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadTaskError::Cancelled => write!(f, "Download cancelled"),
            DownloadTaskError::ClientError(reason) => {
                if let Some(reason) = reason {
                    write!(f, "Client error: {reason}")
                } else {
                    write!(f, "Client error")
                }
            }
            DownloadTaskError::Disconnected => write!(f, "Download task disconnected"),
            DownloadTaskError::Failed(reason) => {
                if let Some(reason) = reason {
                    write!(f, "Download failed: {reason}")
                } else {
                    write!(f, "Download failed")
                }
            }
            DownloadTaskError::LockError(e) => {
                if let Some(reason) = e {
                    write!(f, "Lock error: {reason}")
                } else {
                    write!(f, "Lock error")
                }
            }
            DownloadTaskError::SemaphoreError(e) => {
                if let Some(reason) = e {
                    write!(f, "Semaphore error: {reason}")
                } else {
                    write!(f, "Semaphore error")
                }
            }
            DownloadTaskError::SendError => write!(f, "Failed to send command to download task"),
            DownloadTaskError::RecvError => write!(f, "Failed to receive command"),
        }
    }
}

impl From<reqwest::Error> for DownloadTaskError {
    fn from(err: reqwest::Error) -> Self {
        DownloadTaskError::Failed(Some(err.to_string()))
    }
}

impl From<std::io::Error> for DownloadTaskError {
    fn from(err: std::io::Error) -> Self {
        DownloadTaskError::Failed(Some(err.to_string()))
    }
}

impl From<tokio::sync::AcquireError> for DownloadTaskError {
    fn from(err: tokio::sync::AcquireError) -> Self {
        DownloadTaskError::SemaphoreError(Some(err.to_string()))
    }
}

impl From<tokio::sync::mpsc::error::TrySendError<DownloadTaskCommand>> for DownloadTaskError {
    fn from(err: tokio::sync::mpsc::error::TrySendError<DownloadTaskCommand>) -> Self {
        match err {
            tokio::sync::mpsc::error::TrySendError::Full(_) => DownloadTaskError::SendError,
            tokio::sync::mpsc::error::TrySendError::Closed(_) => DownloadTaskError::Disconnected,
        }
    }
}

impl From<tokio::sync::mpsc::error::TryRecvError> for DownloadTaskError {
    fn from(err: tokio::sync::mpsc::error::TryRecvError) -> Self {
        match err {
            tokio::sync::mpsc::error::TryRecvError::Disconnected => DownloadTaskError::Disconnected,
            tokio::sync::mpsc::error::TryRecvError::Empty => DownloadTaskError::RecvError,
        }
    }
}

impl From<tokio::sync::TryLockError> for DownloadTaskError {
    fn from(err: tokio::sync::TryLockError) -> Self {
        DownloadTaskError::LockError(Some(err.to_string()))
    }
}

/// 下载状态
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DownloadTaskStatus {
    Pending,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone)]
pub enum DownloadTaskCommand {
    Pause,
    Cancel,
    Resume,
}

/// Download task info used for creating a task
pub struct TaskInfo {
    pub url: String,
    pub save_path: String,
    /// 下载后的预期sha1；与`sha1_url`二选一，同时给出时以`sha1`为准
    pub sha1: Option<String>,
    /// 获取预期sha1的地址（maven风格的`<url>.sha1`），下载成功后获取并校验，
    /// 同时缓存到`<save_path>.sha1`，供之后的离线校验
    pub sha1_url: Option<String>,
    pub on_failed: Option<Box<dyn Fn() + Send + Sync>>,
    pub on_finish: Option<Box<dyn Fn() + Send + Sync>>,
    pub on_pause: Option<Box<dyn Fn() + Send + Sync>>,
    pub on_cancel: Option<Box<dyn Fn() + Send + Sync>>,
}

impl TaskInfo {
    pub fn new(
        url: String,
        save_path: String,
        sha1: Option<String>,
        sha1_url: Option<String>,
        on_failed: Option<Box<dyn Fn() + Send + Sync>>,
        on_finish: Option<Box<dyn Fn() + Send + Sync>>,
        on_pause: Option<Box<dyn Fn() + Send + Sync>>,
        on_cancel: Option<Box<dyn Fn() + Send + Sync>>,
    ) -> Self {
        Self {
            url,
            save_path,
            sha1,
            sha1_url,
            on_failed,
            on_finish,
            on_pause,
            on_cancel,
        }
    }
}

/// 下载任务
pub struct DownloadTask {
    client: Client,
    semaphore: Arc<Semaphore>,
    pub url: String,
    pub save_path: String,
    /// 下载后的预期sha1；与`sha1_url`二选一，同时给出时以`sha1`为准
    pub sha1: Option<String>,
    /// 获取预期sha1的地址（maven风格的`<url>.sha1`），下载成功后获取并校验
    pub sha1_url: Option<String>,
    pub status: Mutex<DownloadTaskStatus>,
    /// (downloaded_bytes, total_bytes), (0, 0) if never started, (downloaded, 0) if length is unknown
    pub progress: (AtomicU64, AtomicU64),
    /// Whether the task has ever started downloading
    started: AtomicBool,
    on_failed: Option<Box<dyn Fn() + Send + Sync>>,
    on_finish: Option<Box<dyn Fn() + Send + Sync>>,
    on_pause: Option<Box<dyn Fn() + Send + Sync>>,
    on_cancel: Option<Box<dyn Fn() + Send + Sync>>,
    sender: tokio::sync::mpsc::Sender<DownloadTaskCommand>,
    receiver: RwLock<tokio::sync::mpsc::Receiver<DownloadTaskCommand>>,
}

impl DownloadTask {
    pub fn new(url: String, save_path: String, client: Client, semaphore: Arc<Semaphore>) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel::<DownloadTaskCommand>(10);
        DownloadTask {
            client,
            semaphore,
            url,
            save_path,
            sha1: None,
            sha1_url: None,
            started: AtomicBool::new(false),
            status: Mutex::new(DownloadTaskStatus::Pending),
            progress: (AtomicU64::new(0), AtomicU64::new(0)),
            on_failed: None,
            on_finish: None,
            on_pause: None,
            on_cancel: None,
            sender,
            receiver: RwLock::new(receiver),
        }
    }

    pub fn set_on_finish<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_finish = Some(Box::new(callback));
    }

    pub fn set_on_pause<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_pause = Some(Box::new(callback));
    }

    pub fn set_on_cancel<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_cancel = Some(Box::new(callback));
    }

    pub fn set_on_failed<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_failed = Some(Box::new(callback));
    }

    /// start the task
    /// the task may not start immediately due to the concurrency limit
    pub async fn start(&self) -> Result<(), DownloadTaskError> {
        info!("created url={0} path={1}", self.url, self.save_path);
        let semaphore = self.semaphore.clone();

        let permit = match semaphore.acquire().await {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to acquire semaphore for {0}. Reason: {e}", self.url);

                *self.status.try_lock()? = DownloadTaskStatus::Failed;
                return Err(e.into());
            }
        };

        self.started.store(true, Ordering::Relaxed);
        *self.status.try_lock()? = DownloadTaskStatus::Downloading;

        self.download(permit).await
    }

    async fn download(
        &self,
        _permit: tokio::sync::SemaphorePermit<'_>,
    ) -> Result<(), DownloadTaskError> {
        let client = self.client.clone();
        let downloaded = self.progress.0.load(Ordering::Relaxed);
        let range = if downloaded != 0 {
            format!("bytes={}-", downloaded)
        } else {
            "bytes=0-".to_string()
        };
        let response = match client
            .get(&self.url)
            .header(reqwest::header::RANGE, range)
            .send()
            .await
        {
            Ok(res) => res,
            Err(e) => {
                error!("Failed to get response for {0}. Reason: {e}", self.url);
                *self.status.try_lock()? = DownloadTaskStatus::Failed;
                return Err(e.into());
            }
        };

        let mut file = match tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.save_path)
            .await
        {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to open {0}. Reason: {e}", self.save_path);
                *self.status.try_lock()? = DownloadTaskStatus::Failed;
                return Err(e.into());
            }
        };

        let (d, t) = (
            self.progress.0.load(Ordering::Relaxed),
            self.progress.1.load(Ordering::Relaxed),
        );
        if t == 0 && d == 0 {
            // initialize download
            if let Some(total_bytes) = response.content_length() {
                self.progress.0.store(0, Ordering::Relaxed);
                self.progress.1.store(total_bytes, Ordering::Relaxed);
            } else {
                warn!("Failed to get content length for {0}", self.url);
                // calculate the total size while downloading, but keep the total bytes as 0 to indicate that it's still downloading and the progress is unknown.
            }
        }

        let mut stream = response.bytes_stream();

        info!("Start downloading {0}", self.url);

        let mut attempts: u8 = 0;
        // update progress every 256KB
        let mut c = 0;

        while let Some(chunk) = stream.next().await {
            match self.receiver.try_write()?.try_recv() {
                Ok(DownloadTaskCommand::Pause) => {
                    *self.status.try_lock()? = DownloadTaskStatus::Paused;
                    info!("Paused {0}", self.url);
                    if let Some(on_pause) = &self.on_pause {
                        on_pause();
                    }
                    // Wait until the task is resumed or cancelled
                    loop {
                        match self.receiver.try_write()?.recv().await {
                            Some(DownloadTaskCommand::Resume) => {
                                *self.status.try_lock()? = DownloadTaskStatus::Downloading;
                                info!("Resumed {0}", self.url);
                                break;
                            }
                            Some(DownloadTaskCommand::Cancel) => {
                                *self.status.try_lock()? = DownloadTaskStatus::Cancelled;
                                info!("Cancelled downloading {0}", self.url);
                                drop(file);
                                if let Err(e) = tokio::fs::remove_file(&self.save_path).await {
                                    error!(
                                        "Failed to remove incompleted file {0}. Reason: {e}",
                                        self.save_path
                                    );
                                }
                                if let Some(on_cancel) = &self.on_cancel {
                                    on_cancel();
                                }
                                return Ok(());
                            }
                            Some(DownloadTaskCommand::Pause) => {}
                            None => {
                                error!("Command channel closed for {0}", self.url);
                                *self.status.try_lock()? = DownloadTaskStatus::Failed;
                                drop(file);
                                if let Err(e) = tokio::fs::remove_file(&self.save_path).await {
                                    error!(
                                        "Failed to remove incompleted file {0}. Reason: {e}",
                                        self.save_path
                                    );
                                }
                                if let Some(on_failed) = &self.on_failed {
                                    on_failed();
                                }
                                return Err(DownloadTaskError::Disconnected);
                            }
                        }
                    }
                }
                Ok(DownloadTaskCommand::Cancel) => {
                    *self.status.try_lock()? = DownloadTaskStatus::Cancelled;
                    info!("Cancelled downloading {0}", self.url);
                    drop(file);
                    if let Err(e) = tokio::fs::remove_file(&self.save_path).await {
                        error!(
                            "Failed to remove incompleted file {0}. Reason: {e}",
                            self.save_path
                        );
                    }
                    if let Some(on_cancel) = &self.on_cancel {
                        on_cancel();
                    }
                    return Ok(());
                }
                Err(e) => {
                    if e != TryRecvError::Empty {
                        error!("Failed to receive command for {0}. Reason: {e}", self.url);
                        *self.status.try_lock()? = DownloadTaskStatus::Failed;
                        drop(file);
                        if let Err(e) = tokio::fs::remove_file(&self.save_path).await {
                            error!(
                                "Failed to remove incompleted file {0}. Reason: {e}",
                                self.save_path
                            );
                        }
                        if let Some(on_failed) = &self.on_failed {
                            on_failed();
                        }
                        return Err(e.into());
                    }
                }
                _ => {
                    // resume, ignored
                }
            }

            match chunk {
                Ok(chunk) => {
                    attempts = 0;

                    if let Err(e) = file.write_all(&chunk).await {
                        error!("Failed to write chunk to {0}. Reason: {e}", self.save_path);
                        continue;
                    }

                    c += chunk.len() as u64;
                    if c >= 256 * 1024 {
                        self.progress.0.fetch_add(c, Ordering::Relaxed);
                        c = 0;
                    }
                }
                Err(e) => {
                    if attempts < 3 {
                        attempts += 1;
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        let downloaded = self.progress.0.load(Ordering::Relaxed);
                        let range = if downloaded != 0 {
                            format!("bytes={}-", downloaded)
                        } else {
                            "bytes=0-".to_string()
                        };
                        match client
                            .get(&self.url)
                            .header(reqwest::header::RANGE, range)
                            .send()
                            .await
                        {
                            Ok(res) => {
                                stream = res.bytes_stream();
                            }
                            Err(e) => {
                                error!(
                                    "Failed to download get response for {0}. Reason: {e}",
                                    self.url
                                );
                            }
                        }
                    } else {
                        drop(file);
                        if let Err(e) = tokio::fs::remove_file(&self.save_path).await {
                            error!(
                                "Failed to remove incompleted file {0}. Reason: {e}",
                                self.save_path
                            );
                        }
                        *self.status.try_lock()? = DownloadTaskStatus::Failed;
                        return Err(e.into());
                    }
                }
            }
        }
        self.progress.0.fetch_add(c, Ordering::Relaxed);
        if self.progress.1.load(Ordering::Relaxed) != 0 {
            // This may happen when the total size is unknown at the beginning and the server sends more data than expected, or when the content length is wrong. In this case we just set the total size to the downloaded size to avoid confusion.
            self.progress
                .1
                .store(self.progress.0.load(Ordering::Relaxed), Ordering::Relaxed);
        }

        drop(file);
        if let Err(e) = self.verify().await {
            *self.status.try_lock()? = DownloadTaskStatus::Failed;
            return Err(e);
        }

        if let Some(on_finish) = &self.on_finish {
            on_finish();
        }

        *self.status.try_lock()? = DownloadTaskStatus::Completed;
        info!("Finish downloading {0}", self.url);
        Ok(())
    }

    /// 校验下载完成的文件内容，不一致时删除文件并报错（磁盘上不留下坏文件）
    ///
    /// 预期sha1优先取`sha1`；只有`sha1_url`时从该地址获取（获取失败则跳过校验），
    /// 并把获取到的sha1缓存到`<save_path>.sha1`，供之后的离线校验。
    async fn verify(&self) -> Result<(), DownloadTaskError> {
        let expected = if let Some(sha1) = &self.sha1 {
            sha1.clone()
        } else if let Some(url) = &self.sha1_url {
            match fetch_sha1(&self.client, url).await {
                Some(sha1) => {
                    if let Err(e) =
                        tokio::fs::write(format!("{}.sha1", &self.save_path), &sha1).await
                    {
                        warn!("Failed to cache sha1 of {0}. Reason: {e}", self.save_path);
                    }
                    sha1
                }
                None => {
                    warn!(
                        "Failed to get sha1 of {0} from {1}, skip verification",
                        self.save_path, url
                    );
                    return Ok(());
                }
            }
        } else {
            return Ok(());
        };

        let actual = match sha1_file(&self.save_path).await {
            Ok(actual) => actual,
            Err(e) => {
                error!("Failed to hash {0}. Reason: {e}", self.save_path);
                self.remove_invalid().await;
                return Err(DownloadTaskError::Failed(Some(format!(
                    "Failed to hash {}: {e}",
                    self.save_path
                ))));
            }
        };
        if actual.eq_ignore_ascii_case(&expected) {
            return Ok(());
        }

        error!(
            "sha1 mismatch for {0}: expected {expected}, got {actual}",
            self.save_path
        );
        self.remove_invalid().await;
        Err(DownloadTaskError::Failed(Some(format!(
            "sha1 mismatch for {}: expected {expected}, got {actual}",
            self.save_path
        ))))
    }

    /// 删除校验失败的文件，删除失败只记录日志
    async fn remove_invalid(&self) {
        if let Err(e) = tokio::fs::remove_file(&self.save_path).await {
            warn!(
                "Failed to remove the invalid file {0}. Reason: {e}",
                self.save_path
            );
        }
    }

    pub fn try_cancel(&self) -> Result<(), DownloadTaskError> {
        if !self.started.load(Ordering::Relaxed) {
            // The task has never started, cancel it directly
            *self.status.try_lock()? = DownloadTaskStatus::Cancelled;
            if let Some(on_cancel) = &self.on_cancel {
                on_cancel();
            }
            return Ok(());
        }
        if let Err(e) = self.sender.try_send(DownloadTaskCommand::Cancel) {
            error!(
                "Failed to send cancel command for {0}. Reason: {e}",
                self.url
            );
            return Err(e.into());
        }
        Ok(())
    }

    pub fn try_pause(&self) -> Result<(), DownloadTaskError> {
        if !self.started.load(Ordering::Relaxed) {
            // The task has never started, pause it directly
            *self.status.try_lock()? = DownloadTaskStatus::Paused;
            if let Some(on_pause) = &self.on_pause {
                on_pause();
            }
            return Ok(());
        }
        if let Err(e) = self.sender.try_send(DownloadTaskCommand::Pause) {
            error!(
                "Failed to send pause command for {0}. Reason: {e}",
                self.url
            );
            return Err(e.into());
        }
        Ok(())
    }

    pub fn try_resume(&self) -> Result<(), DownloadTaskError> {
        if let Err(e) = self.sender.try_send(DownloadTaskCommand::Resume) {
            error!(
                "Failed to send resume command for {0}. Reason: {e}",
                self.url
            );
            return Err(e.into());
        }
        Ok(())
    }

    /// Whether the task has ever started downloading
    pub fn is_started(&self) -> bool {
        self.started.load(Ordering::Relaxed)
    }
}

/// 从maven风格的`<url>.sha1`获取哈希，网络错误时重试
async fn fetch_sha1(client: &Client, url: &str) -> Option<String> {
    let mut attempts = 0;
    loop {
        match client.get(url).send().await {
            Ok(res) => match res.text().await {
                Ok(text) => return parse_sha1(&text),
                Err(e) => {
                    warn!("Failed to read sha1 file {url}. Reason: {e}");
                    return None;
                }
            },
            Err(e) => {
                if attempts >= 3 {
                    warn!("Failed to get sha1 file {url}. Reason: {e}");
                    return None;
                }
                attempts += 1;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

/// 解析sha1文件内容，取第一个合法的40位十六进制串（maven的sha1文件有时带文件名）
fn parse_sha1(text: &str) -> Option<String> {
    text.split_whitespace()
        .find(|t| t.len() == 40 && t.bytes().all(|b| b.is_ascii_hexdigit()))
        .map(str::to_lowercase)
}

/// 计算文件的sha1（小写十六进制）
async fn sha1_file(path: &str) -> std::io::Result<String> {
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
fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
