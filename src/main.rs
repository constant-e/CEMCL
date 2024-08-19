mod add_account;
mod file_tools;
mod mc_core;
mod settings;

use log::{debug, error, info};
use serde_json::Value;
use slint::{ModelRc, StandardListViewItem, VecModel};
use std::cell::RefCell;
use std::fs::{read_to_string, write};
use std::io::{self, Write};
use std::process::Command;
use std::rc::Rc;
use std::{sync, thread};
use file_tools::exists;
use mc_core::{get_launch_command, load_game_list, Account, Game};

slint::include_modules!();

const DEFAULT_CONFIG: &str = "{
    \"close_after_launch\": false,
    \"fabric_source\": \"https://maven.fabricmc.net\",
    \"forge_source\": \"https://maven.minecraftforge.net\",
    \"game_path\": \".minecraft\",
    \"height\": 600,
    \"java_path\": \"java\",
    \"game_source\": \"https://piston-meta.mojang.com\",
    \"optifine_source\": \"https://optifine.net\",
    \"width\": 800,
    \"xms\": \"1G\", 
    \"xmx\": \"2G\"
}";

// configs for cemcl
struct Config {
    pub close_after_launch: RefCell<bool>,
    pub fabric_source: RefCell<String>,
    pub forge_source: RefCell<String>,
    pub game_path: RefCell<String>,
    pub height: RefCell<isize>,
    pub java_path: RefCell<String>,
    pub game_source: RefCell<String>,
    pub optifine_source: RefCell<String>,
    pub width: RefCell<isize>,
    pub xms: RefCell<String>,
    pub xmx: RefCell<String>,
}

// load account from account.json
fn load_account() -> Option<Vec<Account>> {
    let mut acc_list: Vec<Account> = Vec::new();

    if !exists(&"account.json".into()) {
        acc_list.push(
            Account {
                account_type: "Legacy".into(),
                token: "None".into(),
                uuid: uuid::Uuid::new_v4().into(),
                user_name: "Steve".into()
            }
        );
        save_account(&acc_list)?;
        return Some(acc_list);
    }

    let json = serde_json::from_str::<Value>(&read_to_string("account.json").ok()?).ok()?;
    for item in json.as_array()? {
        let account = Account {
            account_type: item["account_type"].as_str()?.into(),
            token: item["token"].as_str()?.into(), 
            uuid: item["uuid"].as_str()?.into(),
            user_name: item["user_name"].as_str()?.into()
        };
        acc_list.push(account);
    }
    return Some(acc_list);
}

// load config from config.json
fn load_config() -> Option<Config> {
    let config: Config;

    if !exists(&"config.json".into()) {
        write("config.json", &DEFAULT_CONFIG).ok()?;
    }

    let json: Value = serde_json::from_str(&read_to_string("config.json").ok()?.as_str()).ok()?;
    config = Config {
        close_after_launch: RefCell::from(json["close_after_launch"].as_bool()?),
        fabric_source: RefCell::from(json["fabric_source"].as_str()?.to_string()),
        forge_source: RefCell::from(json["forge_source"].as_str()?.to_string()),
        game_path: RefCell::from(json["game_path"].as_str()?.to_string()),
        height: RefCell::from(json["height"].as_i64()? as isize),
        java_path: RefCell::from(json["java_path"].as_str()?.to_string()),
        game_source: RefCell::from(json["game_source"].as_str()?.to_string()),
        optifine_source: RefCell::from(json["optifine_source"].as_str()?.to_string()),
        width: RefCell::from(json["width"].as_i64()? as isize),
        xms: RefCell::from(json["xms"].as_str()?.to_string()),
        xmx: RefCell::from(json["xmx"].as_str()?.to_string()),
    };

    Some(config)
}

fn save_account(acc_list: &Vec<Account>) -> Option<()> {
    let mut json = serde_json::json!([]);
    for account in acc_list {
        let node = serde_json::json!(
            {
                "account_type": account.account_type.clone(),
                "token": account.token.clone(),
                "uuid": account.uuid.clone(),
                "user_name": account.user_name.clone()
            }
        );
        json.as_array_mut()?.push(node);
    }
    write("account.json", json.to_string()).ok()?;
    Some(())
}

fn main() -> Result<(), slint::PlatformError> {
    env_logger::init();
    info!("App start.");
    let ui = AppWindow::new()?;

    // load config
    let acc_list: Vec<Account>;
    let config: Rc<Config>;
    let game_list: Vec<Game>;

    if let Some(temp_config) = load_config() {
        config = Rc::new(temp_config);
    } else {
        error!("Failed to load config.json.");
        return Err(slint::PlatformError::from("Failed to load config.json."));
    }

    if let Some(temp_acc_list) = load_account() {
        acc_list = temp_acc_list;
    } else {
        error!("Failed to load account.json.");
        return Err(slint::PlatformError::from("Failed to load account.json."));
    }

    if let Some(temp_game_list) = load_game_list(&config) {
        game_list = temp_game_list;
    } else {
        error!("Failed to load game list.");
        return Err(slint::PlatformError::from("Failed to load game list."));
    }

    // load account list in ui
    let mut ui_acc_list: Vec<ModelRc<StandardListViewItem>> = Vec::new();
    for item in &acc_list {
        let account_name = StandardListViewItem::from(item.user_name.as_str());
        let account_type = StandardListViewItem::from(item.account_type.as_str());
        let model: Rc<VecModel<StandardListViewItem>> = Rc::new(VecModel::from(vec![account_name.into(), account_type.into()]));
        let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
        ui_acc_list.push(row);
    }

    ui.set_acc_list(ModelRc::from(Rc::new(VecModel::from(ui_acc_list))));

    // load game list in ui
    let mut ui_game_list: Vec<ModelRc<StandardListViewItem>> = Vec::new();
    for item in &game_list {
        let version = StandardListViewItem::from(item.version.as_str());
        let game_type = StandardListViewItem::from(item.game_type.as_str());
        let description = StandardListViewItem::from(item.description.as_str());
        let model: Rc<VecModel<StandardListViewItem>> = Rc::new(VecModel::from(vec![version.into(), game_type.into(), description.into()]));
        let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
        ui_game_list.push(row);
    }

    ui.set_game_list(ModelRc::from(Rc::new(VecModel::from(ui_game_list))));

    // callbacks
    ui.on_click_add_acc_btn({
        move || {
            add_account::init();
        }
    });

    ui.on_click_settings_btn({
        let config_handle = Rc::downgrade(&config);
        move || {
            let config = config_handle.upgrade().unwrap();
            settings::init(&config);
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
                if acc_index > acc_list.len() || game_index > game_list.len() {
                    let dialog = ErrorDialog::new().unwrap();
                    dialog.set_msg("Haven't select account or game yet.".into());
                    dialog.on_close({
                        let dialog_handle = dialog.as_weak();
                        move || {
                            let dialog = dialog_handle.unwrap();
                            dialog.hide().unwrap();
                        }
                    });
                    dialog.show().unwrap();
                    return;
                }
                if let Some(cmd) = get_launch_command(&acc_list[acc_index], &game_list[game_index], &config.game_path.borrow()) {
                    let mut str = String::new();
                    for i in &cmd {
                        str.push_str(i);
                        str.push_str(" ");
                    }
                    debug!("{str}");
                    
                    let java_path = config.java_path.borrow().clone();
                    let (s, r) = sync::mpsc::channel();

                    thread::spawn(move || {
                        if let Ok(child) = Command::new(java_path).args(cmd).spawn() {
                            s.send(Some(()));
                        } else {
                            s.send(None);
                            error!("Failed to run command.");
                        }
                    });

                    if r.recv().unwrap().is_some() {
                        if config.close_after_launch.borrow().clone() {
                            ui.hide();
                        }
                    } else {
                        let dialog = ErrorDialog::new().unwrap();
                        dialog.set_msg("Failed to run command.".into());
                        dialog.on_close({
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
