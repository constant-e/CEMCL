use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use log::info;
use slint::{ComponentHandle, ModelRc, SharedString, StandardListViewItem, ToSharedString, VecModel};

use super::downloader::{self, DownloadState};

pub fn downloader() -> Result<(), slint::PlatformError> {
    let ui = crate::Downloader::new()?;
    let ui_weak = ui.as_weak();

    let downloader = Rc::new(RefCell::new(downloader::Downloader::new()));

    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(500),
        move || {
            if let (Some(ui), Some(tasks)) =
                (ui_weak.upgrade(), downloader.borrow().get_tasks())
            {
                info!("Running");
                let mut ui_tasks_in_process: Vec<ModelRc<StandardListViewItem>> = Vec::new();
                let mut ui_tasks_finished: Vec<ModelRc<StandardListViewItem>> = Vec::new();
                for (id, url, state) in tasks {
                    match state {
                        DownloadState::Queued => {
                            let model: Rc<VecModel<StandardListViewItem>> = Rc::from(VecModel::from(vec![StandardListViewItem::from(id.to_shared_string()), StandardListViewItem::from(url.to_shared_string()), StandardListViewItem::from(""), StandardListViewItem::from(""), StandardListViewItem::from("")]));
                            let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
                            ui_tasks_in_process.push(row);
                        },
                        DownloadState::Downloading(process) => {
                            let model: Rc<VecModel<StandardListViewItem>> = Rc::from(VecModel::from(vec![StandardListViewItem::from(id.to_shared_string()), StandardListViewItem::from(url.to_shared_string()), StandardListViewItem::from(process.to_shared_string()), StandardListViewItem::from(""), StandardListViewItem::from("")]));
                            let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
                            ui_tasks_in_process.push(row);
                        },
                        DownloadState::Paused => todo!(),
                        DownloadState::Completed => {
                            let model: Rc<VecModel<StandardListViewItem>> = Rc::from(VecModel::from(vec![StandardListViewItem::from(id.to_shared_string()), StandardListViewItem::from(url.to_shared_string()), StandardListViewItem::from("Completed")]));
                            let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
                            ui_tasks_finished.push(row);
                        },
                        DownloadState::Error(msg) => {
                            let model: Rc<VecModel<StandardListViewItem>> = Rc::from(VecModel::from(vec![StandardListViewItem::from(id.to_shared_string()), StandardListViewItem::from(url.to_shared_string()), StandardListViewItem::from(msg.to_shared_string())]));
                            let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
                            ui_tasks_finished.push(row);
                        },
                        DownloadState::Cancelled => {
                            let model: Rc<VecModel<StandardListViewItem>> = Rc::from(VecModel::from(vec![StandardListViewItem::from(id.to_shared_string()), StandardListViewItem::from(url.to_shared_string()), StandardListViewItem::from("Cancelled")]));
                            let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
                            ui_tasks_finished.push(row);
                        },
                    };
                }
                ui.set_finished_list(ModelRc::from(Rc::from(VecModel::from(ui_tasks_finished))));
                ui.set_in_progress_list(ModelRc::from(Rc::from(VecModel::from(ui_tasks_in_process))));
            }
            
        },
    );

    ui.show()
}