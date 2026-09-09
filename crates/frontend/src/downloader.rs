//! Downloader Page相关

use slint::{ModelRc, VecModel};
use std::rc::Rc;

use crate::ui;

/// 下载任务集状态
#[derive(Clone, PartialEq, Eq)]
pub enum TaskSetStatus {
    Pending,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl From<ui::TaskSetStatus> for TaskSetStatus {
    fn from(value: ui::TaskSetStatus) -> Self {
        match value {
            ui::TaskSetStatus::Pending => TaskSetStatus::Pending,
            ui::TaskSetStatus::Downloading => TaskSetStatus::Downloading,
            ui::TaskSetStatus::Paused => TaskSetStatus::Paused,
            ui::TaskSetStatus::Completed => TaskSetStatus::Completed,
            ui::TaskSetStatus::Failed => TaskSetStatus::Failed,
            ui::TaskSetStatus::Cancelled => TaskSetStatus::Cancelled,
        }
    }
}

impl From<TaskSetStatus> for ui::TaskSetStatus {
    fn from(value: TaskSetStatus) -> Self {
        match value {
            TaskSetStatus::Pending => ui::TaskSetStatus::Pending,
            TaskSetStatus::Downloading => ui::TaskSetStatus::Downloading,
            TaskSetStatus::Paused => ui::TaskSetStatus::Paused,
            TaskSetStatus::Completed => ui::TaskSetStatus::Completed,
            TaskSetStatus::Failed => ui::TaskSetStatus::Failed,
            TaskSetStatus::Cancelled => ui::TaskSetStatus::Cancelled,
        }
    }
}

/// 下载任务集信息，用于Downloader Page展示
#[derive(Clone)]
pub struct TaskSetInfo {
    pub id: String,
    pub status: TaskSetStatus,
    /// (downloaded_bytes, total_bytes)
    pub progress: (u64, u64),
    /// (completed_tasks, total_tasks)
    pub progress_by_number: (u64, u64),
}

impl From<ui::TaskSetInfo> for TaskSetInfo {
    fn from(value: ui::TaskSetInfo) -> Self {
        Self {
            id: value.id.into(),
            status: value.status.into(),
            progress: (value.downloaded as u64, value.total as u64),
            progress_by_number: (value.downloaded_num as u64, value.total_num as u64),
        }
    }
}

impl From<TaskSetInfo> for ui::TaskSetInfo {
    fn from(value: TaskSetInfo) -> Self {
        Self {
            id: value.id.into(),
            status: value.status.into(),
            downloaded: value.progress.0 as f32,
            total: value.progress.1 as f32,
            downloaded_num: value.progress_by_number.0 as i32,
            total_num: value.progress_by_number.1 as i32,
        }
    }
}

/// 获取ui用的任务集列表
pub fn ui_task_set_list(list: &Vec<TaskSetInfo>) -> ModelRc<ui::TaskSetInfo> {
    ModelRc::from(Rc::from(VecModel::from(
        list.iter()
            .map(|info| info.clone().into())
            .collect::<Vec<ui::TaskSetInfo>>(),
    )))
}