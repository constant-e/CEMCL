mod add_account;
mod file_tools;
mod mc_core;
mod settings;

use log::{debug, error, info};
use serde_json::Value;
use slint::{ModelRc, StandardListViewItem, VecModel};
use std::fs::{read_to_string, write};
use std::io::{self, Write};
use std::process::Command;
use std::rc::Rc;
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
#[derive(Clone)]
struct Config {
    pub close_after_launch: bool,
    pub fabric_source: String,
    pub forge_source: String,
    pub game_path: String,
    pub height: isize,
    pub java_path: String,
    pub game_source: String,
    pub optifine_source: String,
    pub width: isize,
    pub xms: String,
    pub xmx: String,
}

// load account from account.json
fn load_account() -> Option<Vec<Account>> {
    let mut acc_list: Vec<Account> = Vec::new();

    if !exists(&"account.json".into()) {
        acc_list.push(
            Account {
                account_type: "Legacy".into(),
                token: "None".into(),
                uuid: "Enter uuid".into(),
                user_name: "Steve".into()
            }
        );
        return Some(acc_list);
    }

    if let Ok(json) = serde_json::from_str::<Value>(&read_to_string("account.json").ok()?) {
        for item in json.as_object() {
            let account = Account {
                account_type: item["account_type"].as_str()?.into(),
                token: item["token"].as_str()?.into(), 
                uuid: item["uuid"].as_str()?.into(),
                user_name: item["user_name"].as_str()?.into()
            };
            acc_list.push(account);
        }
        return Some(acc_list);
    } else {
        return None;
    }
}

// load config from config.json
fn load_config() -> Option<Config> {
    let config: Config;

    if !exists(&"config.json".into()) {
        write("config.json", &DEFAULT_CONFIG).ok()?;
    }

    let json: Value = serde_json::from_str(&read_to_string("config.json").ok()?.as_str()).ok()?;
    config = Config {
        close_after_launch: json["close_after_launch"].as_bool()?,
        fabric_source: json["fabric_source"].as_str()?.into(),
        forge_source: json["forge_source"].as_str()?.into(),
        game_path: json["game_path"].as_str()?.into(),
        height: json["height"].as_i64()? as isize,
        java_path: json["java_path"].as_str()?.into(),
        game_source: json["game_source"].as_str()?.into(),
        optifine_source: json["optifine_source"].as_str()?.into(),
        width: json["width"].as_i64()? as isize,
        xms: json["xms"].as_str()?.into(),
        xmx: json["xmx"].as_str()?.into(),
    };

    Some(config)
}

fn main() -> Result<(), slint::PlatformError> {
    env_logger::init();
    info!("App start.");
    let ui = AppWindow::new()?;

    // load config
    let acc_list: Vec<Account>;
    let mut config: Config;
    let game_list: Vec<Game>;

    if let Some(temp_config) = load_config() {
        config = temp_config;
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
        let temp_config = config.clone();
        move || {
            settings::init(&temp_config);
        }
    });

    ui.on_click_start_btn({
        let ui_handle = ui.as_weak();
        let temp_config = config.clone();
        move || {
            let ui = ui_handle.unwrap();
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
            if let Some(cmd) = get_launch_command(&acc_list[acc_index], &game_list[game_index], &temp_config.game_path) {
                let mut str = String::new();
                for i in &cmd {
                    str.push_str(i);
                    str.push_str(" ");
                }
                debug!("{str}");
                if let Ok(out) = Command::new(temp_config.java_path.clone()).args(cmd).output() {
                    io::stdout().write_all(&out.stdout);
                } else {
                    error!("Failed to run command.")
                }
            } else {
                error!("Failed to get launch command.")
            }
        }
    });

    ui.run()
}
