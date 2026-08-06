use futures::StreamExt;
use log::{debug, error, warn};
use reqwest::Client;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use tokio::{
    io::AsyncWriteExt,
    sync::{Mutex, RwLock, Semaphore, mpsc::error::TryRecvError},
    time::Duration,
};

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
    pub on_failed: Option<Box<dyn Fn() + Send + Sync>>,
    pub on_finish: Option<Box<dyn Fn() + Send + Sync>>,
    pub on_pause: Option<Box<dyn Fn() + Send + Sync>>,
    pub on_cancel: Option<Box<dyn Fn() + Send + Sync>>,
}

impl TaskInfo {
    pub fn new(
        url: String,
        save_path: String,
        on_failed: Option<Box<dyn Fn() + Send + Sync>>,
        on_finish: Option<Box<dyn Fn() + Send + Sync>>,
        on_pause: Option<Box<dyn Fn() + Send + Sync>>,
        on_cancel: Option<Box<dyn Fn() + Send + Sync>>,
    ) -> Self {
        Self {
            url,
            save_path,
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
    pub status: Mutex<DownloadTaskStatus>,
    /// (downloaded_bytes, total_bytes), (0, 0) if never started, (downloaded, 0) if length is unknown
    pub progress: (AtomicU64, AtomicU64),
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
        debug!("created url={0} path={1}", self.url, self.save_path);
        let semaphore = self.semaphore.clone();

        let permit = match semaphore.acquire().await {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to acquire semaphore for {0}. Reason: {e}", self.url);

                *self.status.try_lock()? = DownloadTaskStatus::Failed;
                return Err(e.into());
            }
        };

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

        debug!("Start downloading {0}", self.url);

        let mut attempts: u8 = 0;
        // update progress every 256KB
        let mut c = 0;

        while let Some(chunk) = stream.next().await {
            match self.receiver.try_write()?.try_recv() {
                Ok(DownloadTaskCommand::Pause) => {
                    *self.status.try_lock()? = DownloadTaskStatus::Paused;
                    debug!("Paused {0}", self.url);
                    if let Some(on_pause) = &self.on_pause {
                        on_pause();
                    }
                }
                Ok(DownloadTaskCommand::Cancel) => {
                    *self.status.try_lock()? = DownloadTaskStatus::Cancelled;
                    debug!("Cancelled downloading {0}", self.url);
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

        if let Some(on_finish) = &self.on_finish {
            on_finish();
        }

        *self.status.try_lock()? = DownloadTaskStatus::Completed;
        debug!("Finish downloading {0}", self.url);
        Ok(())
    }

    pub fn try_cancel(&self) -> Result<(), DownloadTaskError> {
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
        if let Err(e) = self.sender.try_send(DownloadTaskCommand::Pause) {
            error!(
                "Failed to send pause command for {0}. Reason: {e}",
                self.url
            );
            return Err(e.into());
        }
        Ok(())
    }

    pub async fn resume(&self) -> Result<(), DownloadTaskError> {
        let semaphore = self.semaphore.clone();
        let permit = match semaphore.acquire().await {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to acquire semaphore for {0}. Reason: {e}", self.url);
                *self.status.try_lock()? = DownloadTaskStatus::Failed;
                return Err(e.into());
            }
        };

        *self.status.try_lock()? = DownloadTaskStatus::Downloading;

        self.download(permit).await
    }
}
