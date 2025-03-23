//! 添加新MC版本

use std::fs::exists;
use std::process::Command;
use std::rc;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use std::{fs, sync, thread};

use log::{error, warn};
use slint::{ComponentHandle, ModelRc, StandardListViewItem, VecModel};

use crate::app::App;
use crate::dialogs::msg_box;
use crate::mc::Game;
use crate::mc::download::{self, Fabric, Forge, GameUrl, list_forge};
use crate::{AddGameDialog, Messages};

/// 获取ui用的download_fabric_list
fn ui_fabric_list(fabric_list: &Vec<Fabric>) -> ModelRc<ModelRc<StandardListViewItem>> {
    let mut ui_fabric_list: Vec<ModelRc<StandardListViewItem>> = Vec::new();
    for fabric in fabric_list {
        let version = StandardListViewItem::from(fabric.loader_version.as_str());
        let model: rc::Rc<VecModel<StandardListViewItem>> = rc::Rc::new(VecModel::from(vec![
            version.into(),
            StandardListViewItem::default(),
        ]));
        let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
        ui_fabric_list.push(row);
    }
    ModelRc::from(rc::Rc::new(VecModel::from(ui_fabric_list)))
}

/// 获取ui用的download_forge_list
fn ui_forge_list(forge_list: &Vec<Forge>) -> ModelRc<ModelRc<StandardListViewItem>> {
    let mut ui_forge_list: Vec<ModelRc<StandardListViewItem>> = Vec::new();
    for forge in forge_list {
        let version = StandardListViewItem::from(forge.version.as_str());
        let modified =
            StandardListViewItem::from(forge.modified.split('T').collect::<Vec<&str>>()[0]);
        let model: rc::Rc<VecModel<StandardListViewItem>> =
            rc::Rc::new(VecModel::from(vec![version.into(), modified.into()]));
        let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
        ui_forge_list.push(row);
    }
    ModelRc::from(rc::Rc::new(VecModel::from(ui_forge_list)))
}

/// 获取ui用的download_game_list
fn ui_game_url_list(game_url_list: &Vec<GameUrl>) -> ModelRc<ModelRc<StandardListViewItem>> {
    let mut ui_game_url_list: Vec<ModelRc<StandardListViewItem>> = Vec::new();
    for game in game_url_list {
        let game_type = StandardListViewItem::from(game.game_type.as_str());
        let version = StandardListViewItem::from(game.version.as_str());
        let model: rc::Rc<VecModel<StandardListViewItem>> =
            rc::Rc::new(VecModel::from(vec![version.into(), game_type.into()]));
        let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
        ui_game_url_list.push(row);
    }
    ModelRc::from(rc::Rc::new(VecModel::from(ui_game_url_list)))
}

async fn load_mod(
    app_weak: sync::Weak<Mutex<App>>,
    ui_weak: slint::Weak<AddGameDialog>,
    index: usize,
) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _tokio = rt.enter();

    if let (Some(app), Some(ui)) = (app_weak.upgrade(), ui_weak.upgrade()) {
        if let Ok(mut app) = app.try_lock() {
            let game_index = ui.get_game_index() as usize;
            if game_index >= app.download_game_list.len() {
                warn!("Minecraft not selected.");
                return;
            }

            ui.set_mod_list(ModelRc::default());
            if index == 1 {
                // forge
                let version = &app.download_game_list[game_index].version;
                app.download_forge_list = list_forge(&version).await.unwrap();
                ui.set_mod_list(ui_forge_list(&app.download_forge_list));
            } else if index == 2 {
                // fabric
                let version = &app.download_game_list[game_index].version;
                app.download_fabric_list = download::list_fabric(&version).await.unwrap();
                ui.set_mod_list(ui_fabric_list(&app.download_fabric_list));
            } else {
                app.download_forge_list.clear();
            }
        } else {
            error!("Failed to lock a mutex.");
        }
    } else {
        error!("Failed to upgrade a weak pointer.");
    }
}

pub async fn add_game_dialog(app_weak: sync::Weak<Mutex<App>>) -> Result<(), slint::PlatformError> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _tokio = rt.enter();

    let ui = AddGameDialog::new()?;
    let ui_weak = ui.as_weak();

    let game_url_list = if let Some(app) = app_weak.upgrade() {
        if let Ok(app) = app.try_lock() {
            if let Ok(Some(result)) = rt
                .spawn(download::list_game(app.config.game_path.clone()))
                .await
            {
                result
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    if let Some(app) = app_weak.upgrade() {
        if let Ok(mut app) = app.try_lock() {
            // 筛选版本类型后的列表
            app.download_game_list = game_url_list.clone();
            ui.set_config_height(app.config.height.clone().into());
            ui.set_config_width(app.config.width.clone().into());
            ui.set_description(slint::SharedString::new());
            ui.set_game_args(slint::SharedString::new());
            ui.set_java_path(app.config.java_path.clone().into());
            ui.set_jvm_args(slint::SharedString::new());
            ui.set_separated(false);
            ui.set_xms(app.config.xms.clone().into());
            ui.set_xmx(app.config.xmx.clone().into());
            ui.set_game_list(ui_game_url_list(&game_url_list));
        } else {
            error!("Failed to lock a mutex.");
            return Err(slint::PlatformError::Other(String::from(
                "Failed to lock a mutex",
            )));
        }
    } else {
        error!("Failed to upgrade a weak pointer.");
        return Err(slint::PlatformError::Other(String::from(
            "Failed to upgrade a weak pointer",
        )));
    }

    let app_weak_clone = app_weak.clone();
    let ui_weak_clone = ui_weak.clone();
    ui.on_game_combo_box_changed(move |index| {
        if let (Some(app), Some(ui)) = (app_weak_clone.upgrade(), ui_weak_clone.upgrade()) {
            if let Ok(mut app) = app.try_lock() {
                let require = match index {
                    0 => "",
                    1 => "release",
                    2 => "snapshot",
                    3 => "old_", // old_beta, old_alpha
                    _ => "",
                };
                app.download_game_list.clear();
                for game in &game_url_list {
                    if !game.game_type.contains(require) {
                        continue;
                    }
                    app.download_game_list.push(game.clone());
                }
                ui.set_game_list(ui_game_url_list(&app.download_game_list));
            } else {
                error!("Failed to lock a mutex.");
            }
        } else {
            error!("Failed to upgrade a weak pointer.");
        }
    });

    let app_weak_clone = app_weak.clone();
    let ui_weak_clone = ui_weak.clone();
    ui.on_game_list_changed(move |_| {
        if let Some(ui) = ui_weak_clone.upgrade() {
            let index = ui.get_mod_type();
            let app_weak_clone = app_weak_clone.clone();
            let ui_weak_clone = ui_weak_clone.clone();
            slint::spawn_local(load_mod(app_weak_clone, ui_weak_clone, index as usize)).unwrap();
        } else {
            error!("Failed to upgrade a weak pointer.");
        }
    });

    let app_weak_clone = app_weak.clone();
    let ui_weak_clone = ui_weak.clone();
    ui.on_mod_combo_box_changed(move |index| {
        let app_weak_clone = app_weak_clone.clone();
        let ui_weak_clone = ui_weak_clone.clone();
        slint::spawn_local(load_mod(app_weak_clone, ui_weak_clone, index as usize)).unwrap();
    });

    let ui_weak_clone = ui_weak.clone();
    ui.on_ok_clicked(move || {
        if let (Some(app), Some(ui)) = (app_weak.upgrade(), ui_weak_clone.upgrade()) {
            if let Ok(mut app) = app.lock() {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let _tokio = rt.enter();

                let index = ui.get_game_index() as usize;
                let len = app.download_game_list.len();
                if index >= len {
                    error!("{index} is out of range (max: {})", len);
                    return;
                }

                let game_url = app.download_game_list[index].clone();
                let dir = app.config.game_path.to_string() + "/versions/" + &game_url.version;
                let mod_type = ui.get_mod_type();

                let result = match exists(&dir) {
                    Ok(result) => result,
                    Err(e) => {
                        error!("Failed to check if {dir} exists. Reason: {e}.");
                        return;
                    }
                };

                let mut add_orig = true;
                if !result {
                    if fs::create_dir_all(&dir).is_err() {
                        error!("Failed to create {dir}.");
                        return;
                    };

                    rt.block_on(download::download(game_url.url.clone(), dir.clone() + "/" + &game_url.version + ".json", 3));
                } else {
                    if mod_type == 0 {
                        warn!("The version already exists.");
                        if let Some(app_ui) = app.ui_weak.upgrade() {
                            msg_box::warn_dialog(&app_ui.global::<Messages>().get_version_exists());
                        } else {
                            error!("Failed to upgrade a weak pointer.");
                        }
                        ui.hide().unwrap();
                        return;
                    } else {
                        add_orig = false;
                    }
                }

                let mut game_args = Vec::new();
                let mut jvm_args = Vec::new();

                let game_args_str = ui.get_game_args();
                // make sure the vec is empty when nothing entered
                if !game_args_str.is_empty() {
                    for arg in game_args_str.split(' ') {
                        game_args.push(arg.to_string());
                    }
                }

                let jvm_args_str = ui.get_jvm_args();
                if !jvm_args_str.is_empty() {
                    for arg in jvm_args_str.split(' ') {
                        jvm_args.push(arg.to_string());
                    }
                }

                let game = Game {
                    description: ui.get_description().to_string(),
                    game_args: game_args,
                    height: ui.get_config_height().to_string(),
                    java_path: ui.get_java_path().to_string(),
                    jvm_args: jvm_args,
                    separated: ui.get_separated(),
                    game_type: game_url.game_type,
                    version: game_url.version.clone(),
                    width: ui.get_config_width().to_string(),
                    xms: ui.get_xms().to_string(),
                    xmx: ui.get_xmx().to_string(),
                };

                // mod loader
                match mod_type {
                    0 => {
                        // original
                        if app.add_game(&game).is_none() {
                            error!("Failed to add a game.");
                        }
                    },
                    1 => {
                        // forge
                        let forge_index = ui.get_mod_index() as usize;
                        let forge = app.download_forge_list[forge_index].clone();
                        let forge_url = format!(
                            "{mirror}/maven/net/minecraftforge/forge/{mcversion}-{version}/forge-{mcversion}-{version}-installer.jar",
                            mirror = app.config.forge_source,
                            mcversion = game_url.version,
                            version = forge.version
                        );
                        let forge_path = format!("temp/forge-{mcversion}-{version}-installer.jar", mcversion = game_url.version, version = forge.version);

                        if add_orig && app.add_game(&game).is_none() {
                            error!("Failed to add a game.");
                        }

                        let forge_game = Game {
                            version: game.version.clone() + "-forge-" + &forge.version,
                            ..game
                        };

                        if app.add_game(&forge_game).is_none() {
                            error!("Failed to add a game.");
                        }

                        let app_weak = app_weak.clone();
                        let java_path = app.config.java_path.clone();
                        let game_path = app.config.game_path.clone();
                        thread::spawn(move || {
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            let _tokio = rt.enter();

                            if !fs::exists("temp").unwrap() {
                                if let Err(e) = fs::create_dir("temp") {
                                    error!("Failed to create temp directory. Reason: {e}.");
                                }
                            }
                            if let Some(app) = app_weak.upgrade() {
                                if let Ok(app) = app.lock() {
                                    app.downloader.clear();

                                    if let Err(e) = app.ui_weak.upgrade_in_event_loop(|ui| {
                                        ui.set_progress(0.0);
                                        ui.invoke_set_loading();
                                        ui.invoke_state_set_downloading();
                                    }) {
                                        error!("Failed to upgrade a weak pointer. Reason: {e}.");
                                        return;
                                    }

                                    if let Err(e) = app.downloader.add(forge_url, forge_path.clone()) {
                                        error!("Failed to add a download task. Reason: {e}.");
                                        return;
                                    }

                                    let app_ui_weak = app.ui_weak.clone();
                                    let stop = Arc::new(AtomicBool::new(false));
                                    app.downloader.update_progress_size(stop.clone(), move |progress| {
                                        app_ui_weak
                                            .upgrade_in_event_loop(move |ui| {
                                                ui.set_progress(progress as f32);
                                            })
                                            .unwrap();
                                    });

                                    while app.downloader.in_progress().unwrap() {
                                        sleep(Duration::from_millis(10));
                                        if app.downloader.has_error() {
                                            error!("Failed to download forge.");
                                            stop.store(true, sync::atomic::Ordering::Relaxed);
                                            return;
                                        }
                                    }

                                    stop.store(true, sync::atomic::Ordering::Relaxed);

                                    // 让用户手动安装
                                    match Command::new(java_path)
                                        .arg("-jar")
                                        .arg(forge_path)
                                        // .arg("--installClient")
                                        // .arg(game_path)
                                        .spawn() {
                                        Ok(mut child) => {
                                            if let Err(e) = app.ui_weak.upgrade_in_event_loop(|ui| {
                                                ui.invoke_state_set_launching();
                                            }) {
                                                error!("Failed to upgrade a weak pointer. Reason: {e}.");
                                                return;
                                            }
                                            if let Err(e) = child.wait() {
                                                error!("Failed to run forge installer. Reason: {e}.");
                                                app.ui_weak.upgrade_in_event_loop(move |ui| {
                                                    let msg = ui.global::<Messages>().get_start_failed() + &format!("\n{e}");
                                                    msg_box::err_dialog(&msg);
                                                }).unwrap();
                                            }
                                            app.ui_weak.upgrade_in_event_loop(|ui| ui.invoke_unset_loading()).unwrap();
                                        },
                                        Err(e) => {
                                            error!("Failed to run forge installer. Reason: {e}.");
                                            app.ui_weak.upgrade_in_event_loop(move |ui| {
                                                let msg = ui.global::<Messages>().get_start_failed() + &format!("\n{e}");
                                                msg_box::err_dialog(&msg);
                                            }).unwrap();
                                            return;
                                        },
                                    }

                                    if let Err(e) = fs::remove_dir_all("temp") {
                                        error!("Failed to remove temp directory. Reason: {e}.");
                                    }
                                } else {
                                    error!("Failed to lock a mutex.");
                                }
                            } else {
                                error!("Failed to upgrade a weak pointer.");
                            }
                        });
                    },
                    2 => {
                        // fabric
                        if add_orig && app.add_game(&game).is_none() {
                            error!("Failed to add a game.");
                        }

                        let fabric_index = ui.get_mod_index() as usize;
                        let fabric = app.download_fabric_list[fabric_index].clone();

                        let name = format!("fabric-loader-{fabric_version}-{mc_version}",
                            fabric_version = fabric.loader_version,
                            mc_version = game.version,
                        );

                        let dir = format!("{mc_path}/versions/{name}",
                            mc_path = app.config.game_path,
                        );

                        let result = match exists(&dir) {
                            Ok(result) => result,
                            Err(e) => {
                                error!("Failed to check if {dir} exists. Reason: {e}.");
                                return;
                            }
                        };

                        if !result {
                            if fs::create_dir_all(&dir).is_err() {
                                error!("Failed to create {dir}.");
                                return;
                            };
                        }

                        let url = format!("https://meta.fabricmc.net/v2/versions/loader/{mc_version}/{fabric_version}/profile/json",
                            fabric_version = fabric.loader_version,
                            mc_version = game.version,
                        );
                        let path = format!("{dir}/{name}.json");

                        rt.block_on(download::download(url, path, 3));

                        let fabric_game = Game {
                            version: name,
                            ..game
                        };

                        if app.add_game(&fabric_game).is_none() {
                            error!("Failed to add a game.");
                        }
                    },
                    _ => {},
                }
            } else {
                error!("Failed to lock a mutex.");
            }

            ui.hide().unwrap();
        } else {
            error!("Failed to upgrade a weak pointer.");
        }
    });

    ui.on_cancel_clicked(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.hide().unwrap();
        } else {
            error!("Failed to upgrade a weak pointer.");
        }
    });

    ui.show()
}
