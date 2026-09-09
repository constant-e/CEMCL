use futures::future::join_all;
use log::error;
use std::sync::{Arc, atomic::Ordering};
use tokio::sync::Semaphore;

use super::task::{DownloadTask, DownloadTaskError, DownloadTaskStatus, TaskInfo};

#[derive(Clone, Debug)]
pub enum TaskSetStatus {
    Pending(u64),
    Downloading(u64, u64),
    Paused(u64, u64),
    Completed(u64),
    Cancelled,
    Failed,
}

pub struct TaskSet {
    tasks: Vec<DownloadTask>,
}

impl TaskSet {
    pub fn new(
        client: reqwest::Client,
        tasks_info: Vec<TaskInfo>,
        semaphore: Arc<Semaphore>,
    ) -> Self {
        let tasks = tasks_info
            .into_iter()
            .map(|info| {
                let mut task =
                    DownloadTask::new(info.url, info.save_path, client.clone(), semaphore.clone());
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
            })
            .collect();
        Self { tasks }
    }

    pub fn get_status(&self) -> TaskSetStatus {
        let mut pending = false;
        let mut downloading = false;
        let mut paused = false;
        let mut downloaded = 0;
        let mut total = 0;
        for task in &self.tasks {
            let (d, t) = (
                task.progress.0.load(Ordering::Relaxed),
                task.progress.1.load(Ordering::Relaxed),
            );
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
                }
                DownloadTaskStatus::Paused => paused = true,
                DownloadTaskStatus::Failed => return TaskSetStatus::Failed,
                DownloadTaskStatus::Completed => {
                    if t == 0 {
                        downloaded += 1;
                    }
                }
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

    /// Get the progress of the task set based on the total file size.
    /// Returns (downloaded_bytes, total_bytes).
    pub fn get_progress(&self) -> (u64, u64) {
        let mut downloaded = 0;
        let mut total = 0;
        for task in &self.tasks {
            let (d, t) = (
                task.progress.0.load(Ordering::Relaxed),
                task.progress.1.load(Ordering::Relaxed),
            );
            downloaded += d;
            total += t;
        }
        (downloaded, total)
    }

    pub async fn start(&self) -> Result<(), DownloadTaskError> {
        let mut handles = Vec::new();

        for task in &self.tasks {
            let status = *task.status.try_lock()?;
            if status != DownloadTaskStatus::Pending {
                // Only start pending tasks. Paused tasks should be resumed
                // via the resume method.
                continue;
            }
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
        for task in &self.tasks {
            task.try_pause()?;
        }
        Ok(())
    }

    pub async fn cancel(&self) -> Result<(), DownloadTaskError> {
        for task in &self.tasks {
            task.try_cancel()?;
        }
        Ok(())
    }

    pub async fn resume(&self) -> Result<(), DownloadTaskError> {
        let mut handles = Vec::new();

        for task in &self.tasks {
            let status = *task.status.try_lock()?;
            match status {
                DownloadTaskStatus::Pending => {
                    // The task has never been started
                    let handle = task.start();
                    handles.push(handle);
                }
                DownloadTaskStatus::Paused => {
                    if task.is_started() {
                        // The task is waiting for the resume command
                        task.try_resume()?;
                    } else {
                        // The task was paused before it started
                        let handle = task.start();
                        handles.push(handle);
                    }
                }
                DownloadTaskStatus::Downloading => {
                    // The task is already downloading
                }
                DownloadTaskStatus::Completed
                | DownloadTaskStatus::Failed
                | DownloadTaskStatus::Cancelled => {
                    // The task has finished, nothing to do
                }
            }
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
}
