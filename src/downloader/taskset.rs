use log::error;
use futures::future::join_all;
use std::sync::{Arc, atomic::Ordering};
use tokio::sync::Semaphore;

use super::task::{DownloadTask, DownloadTaskStatus, TaskInfo, DownloadTaskError};

pub enum TaskSetStatus {
    Pending(u64),
    Downloading(u64, u64),
    Paused(u64, u64),
    Completed(u64),
    Cancelled,
    Failed,
}

pub struct TaskSet {
    client: reqwest::Client,
    tasks: Vec<DownloadTask>,
    semaphore: Arc<Semaphore>,
    on_failed: Option<Box<dyn Fn() + Send + Sync>>,
    on_finish: Option<Box<dyn Fn() + Send + Sync>>,
    on_pause: Option<Box<dyn Fn() + Send + Sync>>,
    on_cancel: Option<Box<dyn Fn() + Send + Sync>>,
}

impl TaskSet {
    pub fn new(client: reqwest::Client, tasks_info: Vec<TaskInfo>, semaphore: Arc<Semaphore>, on_cancel: Option<Box<dyn Fn() + Send + Sync>>, on_failed: Option<Box<dyn Fn() + Send + Sync>>, on_finish: Option<Box<dyn Fn() + Send + Sync>>, on_pause: Option<Box<dyn Fn() + Send + Sync>>) -> Self {
        let tasks = tasks_info.into_iter().map(|info| {
            let mut task = DownloadTask::new(info.url, info.save_path, client.clone(), semaphore.clone());
            if let Some(f) = info.on_cancel {
                task.set_on_cancel(f);
            }
            if let Some(f) = info.on_finish {
                task.set_on_finish(f);
            }
            if let Some(f) = info.on_pause {
                task.set_on_pause(f);
            }
            if let Some(f) = info.on_failed {
                task.set_on_failed(f);
            }
            task
        }).collect();
        Self { client, tasks, semaphore, on_cancel, on_failed, on_finish, on_pause }
    }

    pub fn get_status(&self) -> TaskSetStatus {
        let mut pending = false;
        let mut downloading = false;
        let mut paused = false;
        let mut downloaded = 0;
        let mut total = 0;
        for task in &self.tasks {
            let (d, t) = (task.progress.0.load(Ordering::Relaxed), task.progress.1.load(Ordering::Relaxed));
            if t != 0 {
                downloaded += d;
                total += t;
            } else {
                total += 1;
            }
            
            match *task.status.try_lock().unwrap() {
                DownloadTaskStatus::Pending => pending = true,
                DownloadTaskStatus::Downloading => {
                    downloading = true;
                },
                DownloadTaskStatus::Paused => paused = true,
                DownloadTaskStatus::Failed => return TaskSetStatus::Failed,
                DownloadTaskStatus::Completed => {
                    if t == 0 {
                        downloaded += 1;
                    }
                },
                DownloadTaskStatus::Cancelled => return TaskSetStatus::Cancelled,
            }
        }
        if paused {
            // Although tasks in the same taskset are designed to be paused together, we still need to consider unexpected cases.
            TaskSetStatus::Paused(downloaded, total)
        } else if downloading {
            // If some are downloading while others are pending, it may be due to semaphore limits.
            TaskSetStatus::Downloading(downloaded, total)
        } else if pending {
            // It means all the tasks are pending
            TaskSetStatus::Pending(total)
        } else {
            // If failed or cancelled, the function should have returned.
            TaskSetStatus::Completed(downloaded)
        }
    }

    pub fn get_status_by_number(&self) -> TaskSetStatus {
        let mut pending = false;
        let mut downloading = false;
        let mut paused = false;
        let mut downloaded = 0;
        let mut total = 0;
        for task in &self.tasks {
            total += 1;
            match *task.status.try_lock().unwrap() {
                DownloadTaskStatus::Pending => pending = true,
                DownloadTaskStatus::Downloading => downloading = true,
                DownloadTaskStatus::Paused => paused = true,
                DownloadTaskStatus::Failed => return TaskSetStatus::Failed,
                DownloadTaskStatus::Completed => downloaded += 1,
                DownloadTaskStatus::Cancelled => return TaskSetStatus::Cancelled,
            }
        }
        if paused {
            // Although tasks in the same taskset are designed to be paused together, we still need to consider unexpected cases.
            TaskSetStatus::Paused(downloaded, total)
        } else if downloading {
            // If some are downloading while others are pending, it may be due to semaphore limits.
            TaskSetStatus::Downloading(downloaded, total)
        } else if pending {
            // It means all the tasks are pending
            TaskSetStatus::Pending(total)
        } else {
            // If failed or cancelled, the function should have returned.
            TaskSetStatus::Completed(downloaded)
        }
    }

    pub async fn start(&self) -> Result<(), DownloadTaskError> {
        let mut handles = Vec::new();

        for task in &self.tasks {
            let handle = task.start();
            handles.push(handle);
        }
        
        let results = join_all(handles).await;
        for result in results {
            if let Err(e) = result {
                error!("Failed to complete download task: {e}");
                return Err(e);
            }
        }

        Ok(())
    }

    pub async fn pause(&self) -> Result<(), DownloadTaskError> {
        Ok(())
    }

    pub async fn cancel(&self) -> Result<(), DownloadTaskError> {
        Ok(())
    }
}
