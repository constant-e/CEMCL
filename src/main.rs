//! main 程序入口

mod dialogs;
mod file_tools;
mod mc;
mod settings;

use log::{debug, error};
use std::cell::RefCell;
use std::{fs, sync};
use std::process::Command;
use std::rc::Rc;
use std::thread;
use serde_json::{json, Value};
use slint::{ModelRc, VecModel, StandardListViewItem};
use dialogs::err_dialog;
use file_tools::exists;
use mc::{Account, account, Game, game, launch};

slint::include_modules!();

/// 启动器配置
struct Config {
    /// 启动后关闭启动器
    pub close_after_launch: RefCell<bool>,
    /// Fabric下载源
    pub fabric_source: RefCell<String>,
    /// Forge下载源
    pub forge_source: RefCell<String>,
    /// .minecraft路径
    pub game_path: RefCell<String>,
    /// MC下载源
    pub game_source: RefCell<String>,
    /// 默认游戏窗口高度
    pub height: RefCell<String>,
    /// java可执行文件路径
    pub java_path: RefCell<String>,
    /// OptiFine下载源
    pub optifine_source: RefCell<String>,
    /// 默认游戏窗口宽度
    pub width: RefCell<String>,
    /// 默认JVM最小内存
    pub xms: RefCell<String>,
    /// 默认JVM最大内存
    pub xmx: RefCell<String>,
}

/// 从config.json加载配置文件
fn load_config() -> Option<Config> {
    if exists(&"config.json".into()) {
        let json: Value = serde_json::from_str(&fs::read_to_string("config.json").ok()?.as_str()).ok()?;
        let config = Config {
            close_after_launch: RefCell::from(json["close_after_launch"].as_bool()?),
            fabric_source: RefCell::from(String::from(json["fabric_source"].as_str()?)),
            forge_source: RefCell::from(String::from(json["forge_source"].as_str()?)),
            game_path: RefCell::from(String::from(json["game_path"].as_str()?)),
            game_source: RefCell::from(String::from(json["game_source"].as_str()?)),
            height: RefCell::from(String::from(json["height"].as_str()?)),
            java_path: RefCell::from(String::from(json["java_path"].as_str()?)),
            optifine_source: RefCell::from(String::from(json["optifine_source"].as_str()?)),
            width: RefCell::from(String::from(json["width"].as_str()?)),
            xms: RefCell::from(String::from(json["xms"].as_str()?)),
            xmx: RefCell::from(String::from(json["xmx"].as_str()?)),
        };

        Some(config)
    } else {
        let config = Config {
            close_after_launch: RefCell::from(false),
            fabric_source: RefCell::from(String::from("https://maven.fabricmc.net")),
            forge_source: RefCell::from(String::from("https://maven.minecraftforge.net")),
            game_path: RefCell::from(String::from(".minecraft")),
            game_source: RefCell::from(String::from("https://piston-meta.mojang.com")),
            height: RefCell::from(String::from("600")),
            java_path: RefCell::from(String::from("java")),
            optifine_source: RefCell::from(String::from("https://optifine.net")),
            width: RefCell::from(String::from("800")),
            xms: RefCell::from(String::from("1G")),
            xmx: RefCell::from(String::from("2G")),
        };
        save_config(&config);

        Some(config)
    }
}

/// 保存配置文件
fn save_config(config: &Config) -> Option<()> {
    let json = json!(
        {
            "close_after_launch": *config.close_after_launch.borrow(),
            "fabric_source": *config.fabric_source.borrow(),
            "forge_source": *config.forge_source.borrow(),
            "game_path": *config.game_path.borrow(),
            "game_source": *config.game_source.borrow(),
            "height": *config.height.borrow(),
            "java_path": *config.java_path.borrow(),
            "optifine_source": *config.optifine_source.borrow(),
            "width": *config.width.borrow(),
            "xms": *config.xms.borrow(),
            "xmx": *config.xmx.borrow(),
        }
    );
    fs::write("config.json", json.to_string()).ok()
}

pub fn ui_acc_list(acc_list: &Vec<Account>) -> ModelRc<ModelRc<StandardListViewItem>> {
    let mut ui_acc_list: Vec<ModelRc<StandardListViewItem>> = Vec::new();
    for item in acc_list {
        let account_name = StandardListViewItem::from(item.user_name.borrow().as_str());
        let account_type = StandardListViewItem::from(item.account_type.borrow().as_str());
        let model: Rc<VecModel<StandardListViewItem>> = Rc::new(VecModel::from(vec![account_name.into(), account_type.into()]));
        let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
        ui_acc_list.push(row);
    }
    ModelRc::from(Rc::new(VecModel::from(ui_acc_list)))
}

pub fn ui_game_list(game_list: &Vec<Game>) -> ModelRc<ModelRc<StandardListViewItem>> {
    let mut ui_game_list: Vec<ModelRc<StandardListViewItem>> = Vec::new();
    for item in game_list {
        let version = StandardListViewItem::from(item.version.borrow().as_str());
        let game_type = StandardListViewItem::from(item.game_type.borrow().as_str());
        let description = StandardListViewItem::from(item.description.borrow().as_str());
        let model: Rc<VecModel<StandardListViewItem>> = Rc::new(VecModel::from(vec![version.into(), game_type.into(), description.into()]));
        let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
        ui_game_list.push(row);
    }
    ModelRc::from(Rc::new(VecModel::from(ui_game_list)))
}

fn main() -> Result<(), slint::PlatformError> {
    env_logger::init();
    let ui = AppWindow::new()?;

    // load config
    let acc_list: Rc<RefCell<Vec<Account>>>;
    let config: Rc<Config>;
    let game_list: Rc<RefCell<Vec<Game>>>;

    if let Some(temp_config) = load_config() {
        config = Rc::new(temp_config);
    } else {
        error!("Failed to load config.json.");
        return Err(slint::PlatformError::from("Failed to load config.json."));
    }

    if let Some(temp_acc_list) = account::load() {
        acc_list = Rc::new(RefCell::from(temp_acc_list));
    } else {
        error!("Failed to load account.json.");
        return Err(slint::PlatformError::from("Failed to load account.json."));
    }

    if let Some(temp_game_list) = game::load(&config) {
        game_list = Rc::new(RefCell::from(temp_game_list));
    } else {
        error!("Failed to load game list.");
        return Err(slint::PlatformError::from("Failed to load game list."));
    }

    // load account list in ui
    ui.set_acc_list(ui_acc_list(acc_list.borrow().as_ref()));

    // load game list in ui
    ui.set_game_list(ui_game_list(game_list.borrow().as_ref()));

    // callbacks
    ui.on_click_add_acc_btn({
        let acc_list_handle = Rc::downgrade(&acc_list);
        let ui_handle = ui.as_weak();
        move || {
            if let (Some(acc_list), Some(ui)) = (acc_list_handle.upgrade(), ui_handle.upgrade()) {
                account::add_dialog(&acc_list, &ui);
            } else {
                error!("Failed to get acc_list.");
            }
        }
    });

    ui.on_click_add_game_btn({
        let game_list_handle = Rc::downgrade(&game_list);
        let ui_handle = ui.as_weak();
        move || {
            if let (Some(game_list), Some(ui)) = (game_list_handle.upgrade(), ui_handle.upgrade()) {
                game::add_dialog(&game_list, &ui);
            } else {
                error!("Failed to get game_list.");
            }
        }
    });

    ui.on_click_edit_acc_btn({
        let acc_list_handle = Rc::downgrade(&acc_list);
        let ui_handle = ui.as_weak();
        move || {
            if let (Some(acc_list), Some(ui)) = (acc_list_handle.upgrade(), ui_handle.upgrade()) {
                let index = ui.get_acc_index() as usize;
                if index > acc_list.borrow().len() {
                    err_dialog("Please select a account first.");
                    return;
                }
                account::edit_dialog(&acc_list, index, &ui);
            } else {
                error!("Failed to get acc_list.");
            }
        }
    });

    ui.on_click_edit_game_btn({
        let config_handle = Rc::downgrade(&config);
        let game_list_handle = Rc::downgrade(&game_list);
        let ui_handle = ui.as_weak();
        move || {
            if let (Some(config), Some(game_list), Some(ui)) =
                (config_handle.upgrade(), game_list_handle.upgrade(), ui_handle.upgrade())
            {
                let index = ui.get_game_index() as usize;
                if index > game_list.borrow().len() {
                    err_dialog("Please select a game first.");
                    return;
                }
                game::edit_dialog(&game_list, index, &config.game_path.borrow().clone(), &ui);
            } else {
                error!("Failed to get game_list.");
            }
        }
    });

    ui.on_click_settings_btn({
        let config_handle = Rc::downgrade(&config);
        let ui_handle = ui.as_weak();
        move || {
            let config = config_handle.upgrade().unwrap();
            let ui = ui_handle.unwrap();
            settings::init(&config, &ui);
        }
    });

    ui.on_click_start_btn({
        let ui_handle = ui.as_weak();
        let config_handle = Rc::downgrade(&config);
        move || {
            let ui = ui_handle.unwrap();
            if let Some(config) = config_handle.upgrade() {
                let acc_index = ui.get_acc_index() as usize;
                let game_index = ui.get_game_index() as usize;
                if acc_index >= acc_list.borrow().len() || game_index >= game_list.borrow().len() {
                    err_dialog("Please select a account and a game first.");
                    return;
                }
                if let Some(cmd) = launch::get_launch_command(&acc_list.borrow()[acc_index], &game_list.borrow()[game_index], &config.game_path.borrow()) {
                    let mut str = String::new();
                    for i in &cmd {
                        str.push_str(i);
                        str.push_str(" ");
                    }
                    debug!("{str}");
                    
                    let java_path = config.java_path.borrow().clone();
                    let (s, r) = sync::mpsc::channel();

                    thread::spawn(move || {
                        if let Ok(_child) = Command::new(java_path).args(cmd).spawn() {
                            s.send(Some(())).unwrap();
                        } else {
                            s.send(None).unwrap();
                            error!("Failed to run command.");
                        }
                    });

                    if r.recv().unwrap().is_some() {
                        if *config.close_after_launch.borrow() {
                            ui.hide().unwrap();
                        }
                    } else {
                        let dialog = ErrorDialog::new().unwrap();
                        dialog.set_msg("Failed to run command.".into());
                        dialog.on_ok_clicked({
                            let dialog_handle = dialog.as_weak();
                            move || {
                                let dialog = dialog_handle.unwrap();
                                dialog.hide().unwrap();
                            }
                        });
                        dialog.show().unwrap();
                    }
                } else {
                    error!("Failed to get launch command.")
                }
            }
        }
    });

    ui.run()
}
