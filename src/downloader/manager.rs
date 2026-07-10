use dashmap::DashMap;
use std::sync::Arc;
use tokio::{sync::Semaphore, task::JoinHandle};

use super::{
    task::{DownloadTaskError, TaskInfo},
    taskset::{TaskSet, TaskSetStatus},
};

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

pub struct DownloadManager {
    client: reqwest::Client,
    semaphore: Arc<Semaphore>,
    tasks: Arc<DashMap<String, TaskSet>>,
}

impl DownloadManager {
    pub fn new(concurrency: usize) -> Self {
        Self {
            client: reqwest::Client::new(),
            semaphore: Arc::new(Semaphore::new(concurrency)),
            tasks: Arc::new(DashMap::new()),
        }
    }

    pub fn add_taskset(&self, id: String, tasks: Vec<TaskInfo>) {
        let task_set = TaskSet::new(
            self.client.clone(),
            tasks,
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
                task_set.cancel().await.map_err(DownloadManagerError::from)
            } else {
                Err(DownloadManagerError::TaskSetNotFound)
            }
        })
    }

    pub fn resume_taskset(&self, id: String) -> JoinHandle<Result<(), DownloadManagerError>> {
        let tasks = self.tasks.clone();
        tokio::spawn(async move {
            if let Some(task_set) = tasks.get(id.as_str()) {
                task_set.start().await.map_err(DownloadManagerError::from)
            } else {
                Err(DownloadManagerError::TaskSetNotFound)
            }
        })
    }
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new(crate::app::ConfigDL::default().concurrency)
    }
}
