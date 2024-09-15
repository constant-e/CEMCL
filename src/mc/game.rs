use log::{error, warn};
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;
use serde_json::{json, Value};
use slint::{ComponentHandle, ModelRc, StandardListViewItem, VecModel};
use crate::{AddGameDialog, AppWindow, Config, EditGameDialog, ui_game_list};
use crate::dialogs::ask_dialog;
use crate::file_tools::{exists, list_dir};
use super::{download, Game, GameUrl};

fn ui_game_url_list(game_url_list: &Vec<GameUrl>) -> ModelRc<ModelRc<StandardListViewItem>> {
    let mut ui_game_url_list: Vec<ModelRc<StandardListViewItem>> = Vec::new();
    for game in game_url_list {
        let game_type = StandardListViewItem::from(game.game_type.as_str());
        let version = StandardListViewItem::from(game.version.as_str());
        let model: Rc<VecModel<StandardListViewItem>> = Rc::new(VecModel::from(vec![version.into(), game_type.into()]));
        let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
        ui_game_url_list.push(row);
    }
    ModelRc::from(Rc::new(VecModel::from(ui_game_url_list)))
}

pub fn add_dialog(download_game_list: &Rc<RefCell<Vec<GameUrl>>>, game_list: &Rc<RefCell<Vec<Game>>>, app: &AppWindow, config: &Rc<Config>) {
    let ui = AddGameDialog::new().unwrap();

    let game_url_list = if let Some(result) = download::list_game() {
        result
    } else {
        Vec::new()
    };

    // 筛选版本类型后的列表
    *download_game_list.borrow_mut() = game_url_list.clone();
 
    ui.set_game_list(ui_game_url_list(&game_url_list));

    ui.on_game_combo_box_changed({
        let ui_handle = ui.as_weak();
        let real_list_handle = Rc::downgrade(&download_game_list);
        move |game_type| {
            if let (Some(ui), Some(real_list)) = (ui_handle.upgrade(), real_list_handle.upgrade()) {
                let require = if game_type.contains("R") {
                    "release"
                } else if game_type.contains("S") {
                    "snapshot"
                } else {
                    ""
                };

                real_list.borrow_mut().clear();
                for game in &game_url_list {
                    if !game.game_type.contains(require) {
                        continue;
                    }
                    real_list.borrow_mut().push(game.clone());
                }
                ui.set_game_list(ui_game_url_list(real_list.borrow().as_ref()));
            } else {
                error!("Failed to update game list.");
            }
        }
    });

    ui.on_ok_clicked({
        let app_handle = app.as_weak();
        let config_handle = Rc::downgrade(config);
        let game_list_handle = Rc::downgrade(game_list);
        let ui_handle = ui.as_weak();
        let real_list_handle = Rc::downgrade(download_game_list);
        move || {
            if let (Some(app), Some(config), Some(game_list), Some(real_list), Some(ui)) =
                (app_handle.upgrade(), config_handle.upgrade(), game_list_handle.upgrade(), real_list_handle.upgrade(), ui_handle.upgrade())
            {
                let index = ui.get_game_index() as usize;
                if index >= real_list.borrow().len() {
                    error!("{index} is out of range (max: {})", real_list.borrow().len());
                    return;
                }
                let game = real_list.borrow()[index].clone();
                let dir = config.game_path.borrow().to_string() + "/versions/" + &game.version;
                if fs::create_dir_all(&dir).is_err() {
                    error!("Failed to create {dir}.");
                    return;
                };
                download::download(&game.url, &(dir + "/" + &game.version + ".json"), 3);
                if let Some(list) = load(&config) {
                    *game_list.borrow_mut() = list;
                    app.set_game_list(ui_game_list(game_list.borrow().as_ref()));
                } else {
                    error!("Failed to add game.");
                }
                ui.hide().unwrap();
            } else {
                error!("Failed to add game.");
            }
        }
    });

    ui.on_cancel_clicked({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.hide().unwrap();
        }
    });

    ui.show().unwrap();
}

pub fn edit_dialog(game_list: &Rc<RefCell<Vec<Game>>>, index: usize, game_path: &String, app: &AppWindow) {
    let ui = EditGameDialog::new().unwrap();
    let game = &game_list.borrow()[index.clone()];
    ui.set_args(game.args.borrow().clone().into());
    ui.set_config_height(game.height.borrow().clone().into());
    ui.set_config_width(game.width.borrow().clone().into());
    ui.set_description(game.description.borrow().clone().into());
    ui.set_java_path(game.java_path.borrow().clone().into());
    ui.set_separated(game.separated.borrow().clone());
    ui.set_xms(game.xms.borrow().clone().into());
    ui.set_xmx(game.xmx.borrow().clone().into());
    
    ui.on_ok_clicked({
        let app_handle = app.as_weak();
        let game_list_handle = Rc::downgrade(game_list);
        let game_path = game_path.clone();
        let ui_handle = ui.as_weak();
        move || {
            let game_list = game_list_handle.upgrade().unwrap();
            let app = app_handle.unwrap();
            let ui = ui_handle.unwrap();
            let game = &game_list.borrow()[index];
            *game.args.borrow_mut() = ui.get_args().into();
            *game.description.borrow_mut() = ui.get_description().into();
            *game.height.borrow_mut() = ui.get_config_height().into();
            *game.java_path.borrow_mut() = ui.get_java_path().into();
            *game.separated.borrow_mut() = ui.get_separated();
            *game.width.borrow_mut() = ui.get_config_width().into();
            *game.xms.borrow_mut() = ui.get_xms().into();
            *game.xmx.borrow_mut() = ui.get_xmx().into();
            let path = game_path.clone() + "/versions/" + game.version.borrow().as_ref() + "/config.json";
            save(&path, game).unwrap();
            app.set_game_list(ui_game_list(game_list.borrow().as_ref()));
            ui.hide().unwrap();
        }
    });

    ui.on_cancel_clicked({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.hide().unwrap();
        }
    });

    ui.on_click_del_btn({
        let app_handle = app.as_weak();
        let game_list_handle = Rc::downgrade(game_list);
        let game_path = game_path.clone();
        let ui_handle = ui.as_weak();
        move || {
            let app = app_handle.unwrap();
            let game_list = game_list_handle.upgrade().unwrap();
            let ui = ui_handle.unwrap();
            ask_dialog("Warning", "All the files under this game's dir will be deleted. Continue?", {
                let app_handle = app.as_weak();
                let game_list_handle = Rc::downgrade(&game_list);
                let ui_handle = ui.as_weak();
                let path = game_path.clone() + "/versions/" + &game_list.borrow()[index].version.borrow().as_ref();
                move || {
                    let app = app_handle.unwrap();
                    let game_list = game_list_handle.upgrade().unwrap();
                    let ui = ui_handle.unwrap();
                    fs::remove_dir_all(path.clone()).unwrap();
                    game_list.borrow_mut().remove(index);
                    app.set_game_list(ui_game_list(game_list.borrow().as_ref()));
                    ui.hide().unwrap();
                }
            });
        }
    });

    ui.show().unwrap();
}

pub fn load(config: &Config) -> Option<Vec<Game>> {
    let mut game_list: Vec<Game> = Vec::new();
    let dir = config.game_path.borrow().clone() + "/versions";

    if !exists(&dir) {
        // 空目录
        warn!("{dir} is empty.");
        return Some(game_list);
    }

    for version in list_dir(&dir)? {
        let mut game: Game;
        let path = dir.clone() + "/" + version.as_str();
        
        // 先加载原版json
        if let Ok(json) = serde_json::from_str::<Value>(&fs::read_to_string(&(path.clone() + "/" + &version.as_str() + ".json")).ok()?.as_str()) {
            game = Game {
                args: RefCell::from(String::from("")),
                description: RefCell::from(String::from("")),
                height: config.height.clone(),
                java_path: config.java_path.clone(),
                separated: RefCell::from(false),
                game_type: RefCell::from(String::from(json["type"].as_str()?)),
                version: RefCell::from(version),
                width: config.width.clone(),
                xms: config.xms.clone(),
                xmx: config.xmx.clone(),
            };
        } else {
            // 异常，跳过此次加载
            warn!("Failed to load {version}.json.");
            continue;
        }
        
        // 若config.json存在，覆盖原版json
        let cfg_path = path.clone() + "/config.json";
        if exists(&cfg_path) {
            if let Ok(json) = serde_json::from_str::<Value>(&fs::read_to_string(&cfg_path).ok()?.as_str()) {
                game.args = RefCell::from(String::from(json["args"].as_str()?));
                game.description = RefCell::from(String::from(json["description"].as_str()?));
                game.height = RefCell::from(String::from(json["height"].as_str()?));
                game.java_path = RefCell::from(String::from(json["java_path"].as_str()?));
                game.separated = RefCell::from(json["separated"].as_bool()?);
                game.width = RefCell::from(String::from(json["width"].as_str()?));
                game.xms = RefCell::from(String::from(json["xms"].as_str()?));
                game.xmx = RefCell::from(String::from(json["xmx"].as_str()?));
            } else {
                warn!("Failed to load {cfg_path}.");
                continue;
            }
        }
        game_list.push(game);
    };
    Some(game_list)
}

fn save(path: &str, game: &Game) -> Option<()> {
    let json = json!(
        {
            "args": *game.args.borrow(),
            "description": *game.description.borrow(),
            "height": *game.height.borrow(),
            "java_path": *game.java_path.borrow(),
            "separated": *game.separated.borrow(),
            "width": *game.width.borrow(),
            "xms": *game.xms.borrow(),
            "xmx": *game.xmx.borrow(),
        }
    );
    fs::write(path, json.to_string()).ok()
}
