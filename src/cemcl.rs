use log::{info};
use serde_json::Value;
use slint::{include_modules, Model, StandardListViewItem};
use std::default;
use std::process::Command;

use crate::file_tools::{exists, open_file, save_file};
use crate::mc_core::{
    Account,
    Game,
    get_launch_command,
    refresh_game_list
};

const default_config: &str = "{
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

include_modules!();

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
    let mut config: Config;
    
    if !exists(&"config.json".to_string()) {
        save_file(&"config.json".to_string(), &default_config.to_string());
    }

    let json: Value = serde_json::from_str(&open_file(&"config.json".to_string()).as_str())
        .expect("[Error] mc_core: failed to load config.json.");
    config = Config {
        close_after_launch: json["close_after_launch"].as_bool().expect(""),
        forge_source: json["forge_source"].as_str().expect("").to_string(),
        game_path: "~/.minecraft".to_string(),
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
pub fn load_game_list() -> Vec<Game> {
    let mut game_list: Vec<Game> = Vec::new();

    if !exists(&"index.json".to_string()) {
        game_list = refresh_game_list(&game_list);
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
            game_type: item["game_typr"].as_str().expect("").to_string(),
            version: item["version"].as_str().expect("").to_string(),
            width: item["width"].as_i64().expect("") as isize,
            xms: item["xms"].as_str().expect("").to_string(),
            xmx: item["xmx"].as_str().expect("").to_string()
        };
        game_list.push(game);
    }

    game_list
}

// init app
pub fn init() {
    info!(target: "cemcl", "Start.");
    let window = CEMCL::new().expect("Couldn't create window.");
    slint::init_translations!(std::env::current_exe().unwrap().parent().unwrap().join("translations"));
    window.run().expect("Could't start.");

    let mut acc_rows = window.get_acc_table_rows();
    let mut ver_rows = window.get_ver_table_rows();
    let acc_index = window.get_acc_table_index();
    let ver_index = window.get_acc_table_index();

    let mut acc_list = load_account();
    let mut config = load_config();
    let mut game_list = load_game_list();

    window.on_clicked_add_btn(|| {

    });
    window.on_clicked_edit_btn(|| {

    });
    window.on_clicked_settings_btn(|| {

    });
    window.on_clicked_start_btn(move || {
        Command::new(get_launch_command(&acc_list[acc_index as usize], &game_list[ver_index as usize], &config.java_path, &config.game_path));
    })
}
