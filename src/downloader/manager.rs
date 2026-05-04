use dashmap::DashMap;
use std::sync::Arc;
use tokio::{sync::Semaphore, task::JoinHandle};

use super::{task::TaskInfo, taskset::{self, TaskSet, TaskSetStatus}};

pub enum DownloadManagerError {
    TaskSetNotFound,
    DownloadFailed,
    Other(String),
}

impl From<taskset::DownloadError> for DownloadManagerError {
    fn from(value: taskset::DownloadError) -> Self {
        match value {
            taskset::DownloadError::Failed => DownloadManagerError::DownloadFailed,
            taskset::DownloadError::Other(s) => DownloadManagerError::Other(s),
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
        let task_set = TaskSet::new(self.client.clone(), tasks, self.semaphore.clone(), None, None, None, None);
        self.tasks.insert(id, task_set);
    }

    pub fn start_taskset(&self, id: String) -> JoinHandle<Result<(), DownloadManagerError>> {
        let tasks = self.tasks.clone();
        tokio::spawn(async move {
            if let Some(task_set) = tasks.get(id.as_str()) {
                task_set.start().await.map_err(DownloadManagerError::from)
            } else {
                Err(DownloadManagerError::TaskSetNotFound)
            }
        })
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
        let taskset = self.tasks.get(id.as_str()).ok_or(DownloadManagerError::TaskSetNotFound)?;
        Ok(taskset.get_status())
    }

    pub fn get_status_by_number(&self, id: String) -> Result<TaskSetStatus, DownloadManagerError> {
        let taskset = self.tasks.get(id.as_str()).ok_or(DownloadManagerError::TaskSetNotFound)?;
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
