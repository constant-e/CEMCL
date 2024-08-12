mod file_tools;
mod mc_core;

use log::info;
use serde_json::Value;
use slint::{ModelRc, StandardListViewItem, VecModel};
use std::io::{self, Write};
use std::process::Command;
use std::rc::Rc;
use file_tools::{exists, open_file, save_file};
use mc_core::{Account, Game, get_launch_command, refresh_game_list};

slint::include_modules!();

const DEFAULT_CONFIG: &str = "{
    \"close_after_launch\": false,
    \"forge_source\": \"https://maven.minecraftforge.net\",
    \"game_path\": \".minecraft\",
    \"height\": 600,
    \"java_path\": \"java\",
    \"mc_source\": \"https://piston-meta.mojang.com\",
    \"width\": 800,
    \"xms\": \"1G\", 
    \"xmx\": \"2G\"
}";

// configs for cemcl
struct Config {
    pub close_after_launch: bool,
    pub forge_source: String,
    pub game_path: String,
    pub height: isize,
    pub java_path: String,
    pub mc_source: String,
    pub width: isize,
    pub xms: String,
    pub xmx: String,
}

// load account from account.json
fn load_account() -> Vec<Account> {
    let mut acc_list: Vec<Account> = Vec::new();

    if !exists(&"account.json".to_string()) {
        acc_list.push(
            Account {
                account_type: "Legacy".into(),
                token: "None".into(),
                uuid: "Enter uuid".into(),
                user_name: "Steve".into()
            }
        );
        return acc_list;
    }

    let json: Value = serde_json::from_str(&open_file(&"account.json".to_string()).as_str())
        .expect("[Error] mc_core: failed to load account.json.");

    for item in json.as_array().expect("") {
        let account = Account {
            account_type: item["account_type"].as_str().expect("").to_string(),
            token: item["token"].as_str().expect("").to_string(), 
            uuid: item["uuid"].as_str().expect("").to_string(),
            user_name: item["user_name"].as_str().expect("").to_string()
        };
        acc_list.push(account);
    }

    acc_list
}

// load config from config.json
fn load_config() -> Config {
    let config: Config;

    if !exists(&"config.json".to_string()) {
        save_file(&"config.json".to_string(), &DEFAULT_CONFIG.to_string());
    }

    let json: Value = serde_json::from_str(&open_file(&"config.json".to_string()).as_str())
        .expect("[Error] mc_core: failed to load config.json.");
    config = Config {
        close_after_launch: json["close_after_launch"].as_bool().expect(""),
        forge_source: json["forge_source"].as_str().expect("").to_string(),
        game_path: json["game_path"].as_str().expect("").to_string(),
        height: json["height"].as_i64().expect("") as isize,
        java_path: json["java_path"].as_str().expect("").to_string(),
        mc_source: json["mc_source"].as_str().expect("").to_string(),
        width: json["width"].as_i64().expect("") as isize,
        xms: json["xms"].as_str().expect("").to_string(),
        xmx: json["xmx"].as_str().expect("").to_string(),
    };
    config
}

// load game list from index.json
pub fn load_game_list(config: &Config) -> Vec<Game> {
    let mut game_list: Vec<Game> = Vec::new();

    if !exists(&"index.json".to_string()) {
        game_list = refresh_game_list(&game_list, &config);
        return game_list;
    }

    let json: Value = serde_json::from_str(&open_file(&"index.json".to_string()).as_str())
        .expect("[Error] mc_core: failed to load index.json.");

    for item in json.as_array().expect("") {
        let game = Game {
            args: item["args"].as_str().expect("").to_string(),
            description: item["description"].as_str().expect("").to_string(),
            height: item["height"].as_i64().expect("") as isize,
            java_path: item["java_path"].as_str().expect("").to_string(),
            seperated: item["seperated"].as_bool().expect(""),
            game_type: item["game_type"].as_str().expect("").to_string(),
            version: item["version"].as_str().expect("").to_string(),
            width: item["width"].as_i64().expect("") as isize,
            xms: item["xms"].as_str().expect("").to_string(),
            xmx: item["xmx"].as_str().expect("").to_string()
        };
        game_list.push(game);
    }

    game_list
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    // load config
    let mut acc_list = load_account();
    let mut config = load_config();
    let mut game_list = load_game_list(&config); 

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
    ui.on_click_start_btn({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            let acc_index = ui.get_acc_index() as usize;
            let game_index = ui.get_game_index() as usize;
            println!("{acc_index} {game_index}");
            if acc_index > acc_list.len() || game_index > game_list.len() {
                println!("Haven't select account or game yet.");
                return;
            }
            let cmd = mc_core::get_launch_command(&acc_list[acc_index], &game_list[game_index], &config.game_path);
            for i in &cmd {
                println!("{i}");
            }
            let out = Command::new(config.java_path.clone()).args(cmd).output().expect("Error");
            println!("status: {}", out.status);
            io::stdout().write_all(&out.stdout).unwrap();
            io::stderr().write_all(&out.stderr).unwrap();
        }
    });

    ui.run()
}
