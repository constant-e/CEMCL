//! 下载任务管理：任务集的增删启停、广播状态与下载页数据

use downloader::{
    Config as DownloaderConfig, DownloadManager, TaskSetStatusInfo, taskset::TaskSetStatus,
};
use frontend::UIUpdate;
use std::collections::HashMap;
use tokio::sync::broadcast::error::RecvError;

use crate::{LauncherError, ui::UiSender};

pub struct TaskManager {
    downloader: DownloadManager,
    broadcast: tokio::sync::broadcast::Receiver<TaskSetStatusInfo>,
    /// Latest status of each task set, updated by the broadcast
    statuses: HashMap<String, frontend::downloader::TaskSetInfo>,
    ui: UiSender,
}

impl TaskManager {
    pub fn new(config: DownloaderConfig, ui: UiSender) -> Self {
        let downloader = DownloadManager::new(config);
        let broadcast = downloader.subscribe();
        Self {
            downloader,
            broadcast,
            statuses: HashMap::new(),
            ui,
        }
    }

    pub fn downloader(&self) -> &DownloadManager {
        &self.downloader
    }

    pub fn downloader_mut(&mut self) -> &mut DownloadManager {
        &mut self.downloader
    }

    /// 等待下一条任务集状态；broadcast 落后时跳过旧状态
    pub async fn recv(&mut self) -> Result<TaskSetStatusInfo, LauncherError> {
        loop {
            match self.broadcast.recv().await {
                Ok(info) => return Ok(info),
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => return Err(LauncherError::ChannelClosed),
            }
        }
    }

    /// 更新下载页的任务集列表
    pub fn handle_broadcast(&mut self, info: TaskSetStatusInfo) -> Result<(), LauncherError> {
        let progress_by_number = match &info.status_by_number {
            TaskSetStatus::Pending(total) => (0, *total),
            TaskSetStatus::Downloading(downloaded, total) => (*downloaded, *total),
            TaskSetStatus::Paused(downloaded, total) => (*downloaded, *total),
            TaskSetStatus::Completed(total) => (*total, *total),
            TaskSetStatus::Cancelled | TaskSetStatus::Failed => (0, 0),
        };

        self.statuses.insert(
            info.id.clone(),
            frontend::downloader::TaskSetInfo {
                id: info.id,
                status: match info.status {
                    TaskSetStatus::Pending(_) => frontend::downloader::TaskSetStatus::Pending,
                    TaskSetStatus::Downloading(_, _) => {
                        frontend::downloader::TaskSetStatus::Downloading
                    }
                    TaskSetStatus::Paused(_, _) => frontend::downloader::TaskSetStatus::Paused,
                    TaskSetStatus::Completed(_) => frontend::downloader::TaskSetStatus::Completed,
                    TaskSetStatus::Cancelled => frontend::downloader::TaskSetStatus::Cancelled,
                    TaskSetStatus::Failed => frontend::downloader::TaskSetStatus::Failed,
                },
                progress: info.progress,
                progress_by_number,
            },
        );
        let list: Vec<frontend::downloader::TaskSetInfo> =
            self.statuses.values().cloned().collect();
        self.ui.send(UIUpdate::SetTaskSetList(list))
    }

    pub fn pause(&self, id: String) {
        self.downloader.pause_taskset(id);
    }

    pub fn resume(&self, id: String) {
        self.downloader.resume_taskset(id);
    }

    pub fn cancel(&self, id: String) {
        self.downloader.cancel_taskset(id);
    }
}
