//! 添加新MC版本

use std::fs;
use std::{cell::RefCell, rc};

use log::error;
use slint::{ComponentHandle, ModelRc, StandardListViewItem, VecModel};

use crate::app::App;
use crate::mc::download::{self, GameUrl};
use crate::mc::Game;
use crate::AddGameDialog;

/// 获取ui用的download_game_url
fn ui_game_url_list(game_url_list: &Vec<GameUrl>) -> ModelRc<ModelRc<StandardListViewItem>> {
    let mut ui_game_url_list: Vec<ModelRc<StandardListViewItem>> = Vec::new();
    for game in game_url_list {
        let game_type = StandardListViewItem::from(game.game_type.as_str());
        let version = StandardListViewItem::from(game.version.as_str());
        let model: rc::Rc<VecModel<StandardListViewItem>> = rc::Rc::new(VecModel::from(vec![version.into(), game_type.into()]));
        let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
        ui_game_url_list.push(row);
    }
    ModelRc::from(rc::Rc::new(VecModel::from(ui_game_url_list)))
}

pub async fn add_game_dialog(app_weak: rc::Weak<RefCell<App>>) -> Result<(), slint::PlatformError> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _tokio = rt.enter();

    let ui = AddGameDialog::new()?;
    let ui_weak = ui.as_weak();


    let game_url_list = if let Ok(Some(result)) = rt.spawn(download::list_game()).await {
        result
    } else {
        Vec::new()
    };

    if let Some(app) = app_weak.upgrade() {
        // 筛选版本类型后的列表
        app.borrow_mut().download_game_list = game_url_list.clone();
        ui.set_args(slint::SharedString::new());
        ui.set_config_height(app.borrow().config.height.clone().into());
        ui.set_config_width(app.borrow().config.width.clone().into());
        ui.set_description(slint::SharedString::new());
        ui.set_java_path(app.borrow().config.java_path.clone().into());
        ui.set_separated(false);
        ui.set_xms(app.borrow().config.xms.clone().into());
        ui.set_xmx(app.borrow().config.xmx.clone().into());
        ui.set_game_list(ui_game_url_list(&game_url_list));
    }
    
    let app_weak_clone = app_weak.clone();
    let ui_weak_clone = ui_weak.clone();
    ui.on_game_combo_box_changed(move |_| {
        if let (Some(app), Some(ui)) = (app_weak_clone.upgrade(), ui_weak_clone.upgrade()) {
            let require = if ui.get_game_type() == 1 {
                "release"
            } else if ui.get_game_type() == 2 {
                "snapshot"
            } else {
                ""
            };
            app.borrow_mut().download_game_list.clear();
            for game in &game_url_list {
                if !game.game_type.contains(require) {
                    continue;
                }
                app.borrow_mut().download_game_list.push(game.clone());
            }
            ui.set_game_list(ui_game_url_list(&app.borrow().download_game_list));
        } else {
            error!("Failed to upgrade a weak pointer.");
        }
    });

    let ui_weak_clone = ui_weak.clone();
    ui.on_ok_clicked(move || {
        if let (Some(app), Some(ui)) = (app_weak.upgrade(), ui_weak_clone.upgrade())
        {
            let index = ui.get_game_index() as usize;
            let len = app.borrow().download_game_list.len();
            if index >= len {
                error!("{index} is out of range (max: {})", len);
                return;
            }

            let game_url = app.borrow().download_game_list[index].clone();
            let dir = app.borrow().config.game_path.to_string() + "/versions/" + &game_url.version;
            if fs::create_dir_all(&dir).is_err() {
                error!("Failed to create {dir}.");
                return;
            };
            slint::spawn_local(async move {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let _tokio = rt.enter();
                rt.spawn(download::download(game_url.url.clone(), dir.clone() + "/" + &game_url.version + ".json", 3)).await.unwrap();
                let game = Game {
                    description: ui.get_description().to_string(),
                    game_args: Vec::new(),
                    height: ui.get_config_height().to_string(),
                    java_path: ui.get_java_path().to_string(),
                    jvm_args: Vec::new(),
                    separated: ui.get_separated(),
                    game_type: game_url.game_type,
                    version: game_url.version,
                    width: ui.get_config_width().to_string(),
                    xms: ui.get_xms().to_string(),
                    xmx: ui.get_xmx().to_string(),
                };
                app.borrow_mut().add_game(&game);
                ui.hide().unwrap();
            }).unwrap();
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
