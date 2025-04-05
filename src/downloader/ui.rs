use std::sync;
use std::sync::Mutex;
use std::{rc::Rc, thread};

use log::error;
use slint::{ComponentHandle, ModelRc, StandardListViewItem, ToSharedString, VecModel};

use crate::app::App;

use super::downloader::DownloadState;

pub fn downloader(app_weak: sync::Weak<Mutex<App>>) -> Result<(), slint::PlatformError> {
    let ui = crate::Downloader::new()?;
    let ui_weak = ui.as_weak();

    thread::spawn(move || {
        // TODO: Update the UI
        if let Some(app) = app_weak.upgrade() {
            if let Ok(app) = app.lock() {
                if let (Some(ui), Some(tasks)) = (ui_weak.upgrade(), app.downloader.get_tasks()) {
                    let mut ui_tasks_in_process: Vec<ModelRc<StandardListViewItem>> = Vec::new();
                    let mut ui_tasks_finished: Vec<ModelRc<StandardListViewItem>> = Vec::new();
                    let mut downloaded_sum = 0;
                    let mut total_sum = 0;
                    for (id, url, state) in tasks {
                        match state {
                            DownloadState::Queued(_) => {
                                let model: Rc<VecModel<StandardListViewItem>> =
                                    Rc::from(VecModel::from(vec![
                                        StandardListViewItem::from(id.to_shared_string()),
                                        StandardListViewItem::from(url.to_shared_string()),
                                        StandardListViewItem::from(""),
                                        StandardListViewItem::from(""),
                                        StandardListViewItem::from(""),
                                    ]));
                                let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
                                ui_tasks_in_process.push(row);
                            }
                            DownloadState::Downloading(downloaded, total) => {
                                downloaded_sum += downloaded;
                                total_sum += total;
                                let model: Rc<VecModel<StandardListViewItem>> =
                                    Rc::from(VecModel::from(vec![
                                        StandardListViewItem::from(id.to_shared_string()),
                                        StandardListViewItem::from(url.to_shared_string()),
                                        StandardListViewItem::from(
                                            (downloaded / total * 100).to_shared_string(),
                                        ),
                                        StandardListViewItem::from(""),
                                        StandardListViewItem::from(""),
                                    ]));
                                let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
                                ui_tasks_in_process.push(row);
                            }
                            DownloadState::Paused => todo!(),
                            DownloadState::Completed(total) => {
                                downloaded_sum += total;
                                total_sum += total;
                                let model: Rc<VecModel<StandardListViewItem>> =
                                    Rc::from(VecModel::from(vec![
                                        StandardListViewItem::from(id.to_shared_string()),
                                        StandardListViewItem::from(url.to_shared_string()),
                                        StandardListViewItem::from("Completed"),
                                    ]));
                                let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
                                ui_tasks_finished.push(row);
                            }
                            DownloadState::Error(msg) => {
                                let model: Rc<VecModel<StandardListViewItem>> =
                                    Rc::from(VecModel::from(vec![
                                        StandardListViewItem::from(id.to_shared_string()),
                                        StandardListViewItem::from(url.to_shared_string()),
                                        StandardListViewItem::from(msg.to_shared_string()),
                                    ]));
                                let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
                                ui_tasks_finished.push(row);
                            }
                            DownloadState::Cancelled => {
                                let model: Rc<VecModel<StandardListViewItem>> =
                                    Rc::from(VecModel::from(vec![
                                        StandardListViewItem::from(id.to_shared_string()),
                                        StandardListViewItem::from(url.to_shared_string()),
                                        StandardListViewItem::from("Cancelled"),
                                    ]));
                                let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
                                ui_tasks_finished.push(row);
                            }
                        };
                    }
                    ui.set_finished_list(ModelRc::from(Rc::from(VecModel::from(
                        ui_tasks_finished,
                    ))));
                    ui.set_in_progress_list(ModelRc::from(Rc::from(VecModel::from(
                        ui_tasks_in_process,
                    ))));
                }
            } else {
                error!("Failed to lock a mutex");
            }
        } else {
            error!("Failed to upgrade a weak pointer");
        }
    });

    ui.show()
}
