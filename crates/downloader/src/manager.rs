use dashmap::DashMap;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    sync::{Semaphore, broadcast},
    task::JoinHandle,
};

use super::{
    task::{DownloadTaskError, TaskInfo},
    taskset::{TaskSet, TaskSetStatus},
};

/// Status of a task set, broadcast to the upper layer
#[derive(Clone, Debug)]
pub struct TaskSetStatusInfo {
    pub id: String,
    /// Status based on the total file size
    pub status: TaskSetStatus,
    /// Status based on the number of completed tasks
    pub status_by_number: TaskSetStatus,
    /// (downloaded_bytes, total_bytes)
    pub progress: (u64, u64),
}

impl TaskSetStatusInfo {
    pub fn new(
        id: String,
        status: TaskSetStatus,
        status_by_number: TaskSetStatus,
        progress: (u64, u64),
    ) -> Self {
        Self {
            id,
            status,
            status_by_number,
            progress,
        }
    }
}

#[derive(Debug)]
pub enum DownloadManagerError {
    TaskSetNotFound,
    // same as task error
    Cancelled,
    ClientError(Option<String>),
    Disconnected,
    Failed(Option<String>),
    LockError(Option<String>),
    SemaphoreError(Option<String>),
    SendError,
    RecvError,
}

impl From<DownloadTaskError> for DownloadManagerError {
    fn from(value: DownloadTaskError) -> Self {
        match value {
            DownloadTaskError::Cancelled => DownloadManagerError::Cancelled,
            DownloadTaskError::ClientError(s) => DownloadManagerError::ClientError(s),
            DownloadTaskError::Disconnected => DownloadManagerError::Disconnected,
            DownloadTaskError::Failed(s) => DownloadManagerError::Failed(s),
            DownloadTaskError::LockError(s) => DownloadManagerError::LockError(s),
            DownloadTaskError::SemaphoreError(s) => DownloadManagerError::SemaphoreError(s),
            DownloadTaskError::SendError => DownloadManagerError::SendError,
            DownloadTaskError::RecvError => DownloadManagerError::RecvError,
        }
    }
}

impl std::fmt::Display for DownloadManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadManagerError::TaskSetNotFound => write!(f, "Task set not found"),
            DownloadManagerError::Cancelled => write!(f, "Download was cancelled"),
            DownloadManagerError::ClientError(reason) => {
                if let Some(reason) = reason {
                    write!(f, "Client error: {reason}")
                } else {
                    write!(f, "Client error")
                }
            }
            DownloadManagerError::Disconnected => write!(f, "Download task disconnected"),
            DownloadManagerError::Failed(reason) => {
                if let Some(reason) = reason {
                    write!(f, "Download failed: {reason}")
                } else {
                    write!(f, "Download failed")
                }
            }
            DownloadManagerError::LockError(e) => {
                if let Some(reason) = e {
                    write!(f, "Lock error: {reason}")
                } else {
                    write!(f, "Lock error")
                }
            }
            DownloadManagerError::SemaphoreError(e) => {
                if let Some(reason) = e {
                    write!(f, "Semaphore error: {reason}")
                } else {
                    write!(f, "Semaphore error")
                }
            }
            DownloadManagerError::SendError => write!(f, "Failed to send command to download task"),
            DownloadManagerError::RecvError => write!(f, "Failed to receive command"),
        }
    }
}

#[derive(Clone)]
pub struct Config {
    pub concurrency: u32,
    pub mirrors: HashMap<String, String>,
}

pub struct DownloadManager {
    client: reqwest::Client,
    semaphore: Arc<Semaphore>,
    tasks: Arc<DashMap<String, TaskSet>>,
    config: Config,
    broadcast_sender: broadcast::Sender<TaskSetStatusInfo>,
}

impl DownloadManager {
    pub fn new(config: Config) -> Self {
        let (broadcast_sender, _) = broadcast::channel(64);
        let manager = Self {
            client: reqwest::Client::new(),
            semaphore: Arc::new(Semaphore::new(config.concurrency as usize)),
            tasks: Arc::new(DashMap::new()),
            config,
            broadcast_sender,
        };
        manager.start_broadcast();
        manager
    }

    /// Subscribe to the task set status broadcast
    pub fn subscribe(&self) -> broadcast::Receiver<TaskSetStatusInfo> {
        self.broadcast_sender.subscribe()
    }

    /// Broadcast the status of all task sets periodically
    fn start_broadcast(&self) {
        let tasks = self.tasks.clone();
        let sender = self.broadcast_sender.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));
            loop {
                interval.tick().await;
                for entry in tasks.iter() {
                    let id = entry.key().clone();
                    let task_set = entry.value();
                    let status = task_set.get_status();
                    let status_by_number = task_set.get_status_by_number();
                    let progress = task_set.get_progress();
                    if sender
                        .send(TaskSetStatusInfo::new(
                            id,
                            status,
                            status_by_number,
                            progress,
                        ))
                        .is_err()
                    {
                        // No receiver, stop broadcasting
                        return;
                    }
                }
            }
        });
    }

    pub fn add_taskset(&self, id: String, tasks: Vec<TaskInfo>) {
        let task_set = TaskSet::new(
            self.client.clone(),
            tasks
                .into_iter()
                .map(|mut v| {
                    for (k, mirror) in self.config.mirrors.clone() {
                        let k = format!("{{{k}}}");
                        v.url = v.url.replace(&k, &mirror);
                    }
                    v
                })
                .collect(),
            self.semaphore.clone(),
            None,
            None,
            None,
            None,
        );
        self.tasks.insert(id, task_set);
    }

    pub fn start_taskset(
        &self,
        id: String,
    ) -> Result<JoinHandle<Result<(), DownloadTaskError>>, DownloadManagerError> {
        let tasks = self.tasks.clone();

        if !tasks.contains_key(&id) {
            return Err(DownloadManagerError::TaskSetNotFound);
        }

        Ok(tokio::spawn(async move {
            tasks.get(id.as_str()).unwrap().start().await
        }))
    }

    pub fn cancel_taskset(&self, id: String) -> JoinHandle<Result<(), DownloadManagerError>> {
        let tasks = self.tasks.clone();
        tokio::spawn(async move {
            if let Some(task_set) = tasks.get(id.as_str()) {
                task_set.cancel().await.map_err(DownloadManagerError::from)
            } else {
                Err(DownloadManagerError::TaskSetNotFound)
            }
        })
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }

    pub fn get_status(&self, id: String) -> Result<TaskSetStatus, DownloadManagerError> {
        let taskset = self
            .tasks
            .get(id.as_str())
            .ok_or(DownloadManagerError::TaskSetNotFound)?;
        Ok(taskset.get_status())
    }

    pub fn get_status_by_number(&self, id: String) -> Result<TaskSetStatus, DownloadManagerError> {
        let taskset = self
            .tasks
            .get(id.as_str())
            .ok_or(DownloadManagerError::TaskSetNotFound)?;
        Ok(taskset.get_status_by_number())
    }

    pub fn get_progress(&self, id: String) -> Result<(u64, u64), DownloadManagerError> {
        let taskset = self
            .tasks
            .get(id.as_str())
            .ok_or(DownloadManagerError::TaskSetNotFound)?;
        Ok(taskset.get_progress())
    }

    pub fn set_config(&mut self, config: Config) {
        self.config = config
    }

    pub fn pause_taskset(&self, id: String) -> JoinHandle<Result<(), DownloadManagerError>> {
        let tasks = self.tasks.clone();
        tokio::spawn(async move {
            if let Some(task_set) = tasks.get(id.as_str()) {
                task_set.pause().await.map_err(DownloadManagerError::from)
            } else {
                Err(DownloadManagerError::TaskSetNotFound)
            }
        })
    }

    pub fn remove_taskset(&self, id: String) -> JoinHandle<Result<(), DownloadManagerError>> {
        let tasks = self.tasks.clone();
        tokio::spawn(async move {
            if let Some(task_set) = tasks.get(id.as_str()) {
                task_set.cancel().await.map_err(DownloadManagerError::from)?;
                tasks.remove(id.as_str());
                Ok(())
            } else {
                Err(DownloadManagerError::TaskSetNotFound)
            }
        })
    }

    pub fn resume_taskset(&self, id: String) -> JoinHandle<Result<(), DownloadManagerError>> {
        let tasks = self.tasks.clone();
        tokio::spawn(async move {
            if let Some(task_set) = tasks.get(id.as_str()) {
                task_set.resume().await.map_err(DownloadManagerError::from)
            } else {
                Err(DownloadManagerError::TaskSetNotFound)
            }
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            concurrency: 10,
            mirrors: HashMap::new(),
        }
    }
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
