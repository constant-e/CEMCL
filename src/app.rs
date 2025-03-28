//! CEMCL 主模块

use std::fs::{self, exists};
use std::io::ErrorKind;
use std::process::Command;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::{sync, thread};

use log::{debug, error, warn};
use serde_json::json;
use slint::{ComponentHandle, ModelRc, StandardListViewItem, VecModel};

use crate::dialogs::msg_box::{err_dialog, warn_dialog};
use crate::downloader::downloader::Downloader;
use crate::file_tools::list_dir;
use crate::mc::download::{Fabric, Forge, GameUrl};
use crate::mc::{Account, Game, launch};
use crate::{AppWindow, Messages};

/// 启动器配置
#[derive(Clone)]
pub struct Config {
    /// assets下载源
    pub assets_source: String,

    /// 启动后关闭启动器
    pub close_after_launch: bool,

    /// 下载时的最大并发数量
    pub concurrency: usize,

    /// Fabric下载源
    pub fabric_source: String,

    /// Forge下载源
    pub forge_source: String,

    /// .minecraft路径
    pub game_path: String,

    /// MC本体下载源
    pub game_source: String,

    /// 默认游戏窗口高度
    pub height: String,

    /// java可执行文件路径
    pub java_path: String,

    /// libraries下载源
    pub libraries_source: String,

    /// 默认游戏窗口宽度
    pub width: String,

    /// 默认JVM最小内存
    pub xms: String,

    /// 默认JVM最大内存
    pub xmx: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            assets_source: String::from("https://resources.download.minecraft.net"),
            close_after_launch: false,
            concurrency: 10,
            fabric_source: String::from("https://maven.fabricmc.net"),
            forge_source: String::from("https://files.minecraftforge.net"),
            game_path: String::from(".minecraft"),
            game_source: String::from("https://piston-meta.mojang.com"),
            height: String::from("600"),
            java_path: String::from("java"),
            libraries_source: String::from("https://libraries.minecraft.net"),
            width: String::from("800"),
            xms: String::from("1G"),
            xmx: String::from("2G"),
        }
    }
}

pub struct App {
    pub acc_list: Vec<Account>,
    pub config: Config,
    pub device_code: String,
    pub download_fabric_list: Vec<Fabric>,
    pub download_forge_list: Vec<Forge>,
    pub download_game_list: Vec<GameUrl>,
    pub downloader: Downloader,
    pub game_list: Vec<Game>,
    pub ui_weak: slint::Weak<AppWindow>,
}

impl App {
    /// Create a new app with the weak pointer of ui provided
    pub fn new(ui_weak: slint::Weak<AppWindow>) -> Result<App, std::io::Error> {
        let mut app = App::default();

        if let Err(e) = app.load_acc_list() {
            warn!("Failed to load account list. Reason: {e}.");
            let msg = ui_weak
                .upgrade()
                .ok_or(ErrorKind::Other)?
                .global::<Messages>()
                .get_load_acc_failed()
                .to_string()
                + &format!("{e}");
            warn_dialog(&msg);
        }

        if let Err(e) = app.load_config() {
            warn!("Failed to load config. Reason: {e}.");
            let msg = ui_weak
                .upgrade()
                .ok_or(ErrorKind::Other)?
                .global::<Messages>()
                .get_load_config_failed()
                .to_string()
                + &format!("{e}");
            warn_dialog(&msg);
        }

        if let Err(e) = app.load_game_list() {
            warn!("Failed to load game list. Reason: {e}.");
            let msg = ui_weak
                .upgrade()
                .ok_or(ErrorKind::Other)?
                .global::<Messages>()
                .get_load_game_failed()
                .to_string()
                + &format!("{e}");
            warn_dialog(&msg);
        }

        // todo: set concurrency
        app.downloader = Downloader::new(app.config.concurrency);

        app.ui_weak = ui_weak;
        app.refresh_ui_acc_list();
        app.refresh_ui_game_list();

        Ok(app)
    }

    /// Add an account to self.acc_list, also call self.save_acc_list() and self.refresh_ui_acc_list()
    pub fn add_account(&mut self, account: &Account) -> Option<()> {
        self.acc_list.push(account.clone());
        self.save_acc_list().ok()?;
        self.refresh_ui_acc_list()
    }

    /// Add a game to self.game_list, also call game.save(), self.save_launcher_profiles() and self.refresh_ui_game_list()
    pub fn add_game(&mut self, game: &Game) -> Option<()> {
        self.game_list.push(game.clone());
        let dir = self.config.game_path.clone() + "/versions/" + &game.version;
        if !exists(&dir).ok()? {
            fs::create_dir_all(&dir).ok()?;
        }
        let path = dir + "/config.json";
        game.save(&path).ok()?;
        self.save_launcher_profiles().ok()?;
        self.refresh_ui_game_list()
    }

    /// Delete an account, also call self.save_acc_list() and self.refresh_ui_acc_list()
    pub fn del_account(&mut self, index: usize) -> Option<()> {
        // if index >= self.acc_list.len() {
        //     error!("Index out of bounds: the len is {} but the index is {index}.", self.acc_list.len());
        //     return None;
        // }
        self.acc_list.remove(index);
        self.save_acc_list().ok()?;
        self.refresh_ui_acc_list()
    }

    /// Delete a game, also delete the game directory and call self.save_launcher_profiles() and self.refresh_ui_game_list()
    pub fn del_game(&mut self, index: usize) -> Option<()> {
        // if index >= self.game_list.len() {
        //     error!("Index out of bounds: the len is {} but the index is {index}.", self.game_list.len());
        //     return None;
        // }
        let path = self.config.game_path.clone() + "/versions/" + &self.game_list[index].version;
        self.game_list.remove(index);
        fs::remove_dir_all(path).ok()?;
        self.save_launcher_profiles().ok()?;
        self.refresh_ui_game_list()
    }

    /// Edit an account, also call self.save_acc_list() and self.refresh_ui_acc_list()
    pub fn edit_account(&mut self, index: usize, account: Account) -> Option<()> {
        self.acc_list[index] = account;
        self.save_acc_list().ok()?;
        self.refresh_ui_acc_list()
    }

    /// Edit a game, also call Game::save, self.save_launcher_profiles() and self.refresh_ui_game_list()
    pub fn edit_game(&mut self, index: usize, game: Game) -> Option<()> {
        let path = self.config.game_path.clone() + "/versions/" + &game.version + "/config.json";
        game.save(&path).ok()?;
        self.game_list[index] = game;
        self.save_launcher_profiles().ok()?;
        self.refresh_ui_game_list()
    }

    /// Get the current index of account list in ui, return None when index is out of range
    pub fn get_acc_index(&self) -> Option<usize> {
        let ui = self.ui_weak.upgrade()?;
        let index = ui.get_acc_index() as usize;
        if index >= self.acc_list.len() {
            warn!(
                "Index out of bounds: the len is {} but the index is {index}.",
                self.acc_list.len()
            );
            return None;
        }
        Some(index)
    }

    /// Get the current index of game list in ui, return None when index is out of range
    pub fn get_game_index(&self) -> Option<usize> {
        let ui = self.ui_weak.upgrade()?;
        let index = ui.get_game_index() as usize;
        if index >= self.game_list.len() {
            warn!(
                "Index out of bounds: the len is {} but the index is {index}.",
                self.game_list.len()
            );
            return None;
        }
        Some(index)
    }

    // we should get acc index and game index in main thread
    /// Launch the game
    pub async fn launch(&mut self, acc_index: usize, game_index: usize) -> Option<()> {
        self.ui_weak
            .upgrade_in_event_loop(|ui| {
                ui.set_progress(0.0);
            })
            .ok()?;

        if let Err(e) = self.downloader.clear() {
            error!("Failed to clear downloader. Reason: {e}");
            self.ui_weak
                .upgrade_in_event_loop(move |ui| {
                    err_dialog(&format!("{e}"));
                    ui.invoke_unset_loading();
                })
                .unwrap();
            return None;
        }

        if acc_index >= self.acc_list.len() || game_index >= self.game_list.len() {
            warn!(
                "Index out of bounds: the len is ({}, {}) but the index is ({acc_index}, {game_index}).",
                self.acc_list.len(),
                self.game_list.len()
            );
            self.ui_weak
                .upgrade_in_event_loop(|ui| {
                    err_dialog(&ui.global::<Messages>().get_acc_or_game_not_selected())
                })
                .unwrap();
            return None;
        }

        self.ui_weak
            .upgrade_in_event_loop(|ui| ui.invoke_set_loading())
            .ok()?;

        // refresh access_token
        self.ui_weak
            .upgrade_in_event_loop(|ui| ui.invoke_state_set_logging_in())
            .ok()?;
        if self.acc_list[acc_index]
            .refresh(self.ui_weak.clone())
            .await
            .is_none()
        {
            error!("Failed to login.");
            self.ui_weak
                .upgrade_in_event_loop(|ui| {
                    err_dialog(&ui.global::<Messages>().get_login_failed());
                    ui.invoke_unset_loading();
                })
                .unwrap();
            return None;
        }

        match launch::get_launch_command(
            &self.acc_list[acc_index],
            &self.game_list[game_index],
            &self.config,
        )
        .await
        {
            Ok((cmd, game_download)) => {
                if cfg!(debug_assertions) {
                    let mut str = self.game_list[game_index].java_path.clone() + " ";
                    for i in &cmd {
                        str.push_str(i);
                        str.push_str(" ");
                    }
                    debug!("{str}");
                }

                if let Err(e) = self
                    .ui_weak
                    .upgrade_in_event_loop(|ui| ui.invoke_state_set_downloading())
                {
                    error!("Failed to upgrade a weak pointer. Reason: {e}.");
                    self.ui_weak
                        .upgrade_in_event_loop(move |ui| {
                            err_dialog(&format!("{e}"));
                            ui.invoke_unset_loading();
                        })
                        .unwrap();
                    return None;
                }

                // UI进度条
                let ui_weak_clone = self.ui_weak.clone();
                let stop = Arc::new(AtomicBool::new(false));
                self.downloader
                    .update_progress(stop.clone(), move |progress| {
                        ui_weak_clone
                            .upgrade_in_event_loop(move |ui| {
                                ui.set_progress(progress as f32);
                            })
                            .unwrap();
                    });

                if let Err(e) = launch::download_all(&self.config, &game_download, &self.downloader)
                {
                    error!("Failed to download. Reason: {e}");
                    stop.store(true, sync::atomic::Ordering::Relaxed);
                    self.ui_weak
                        .upgrade_in_event_loop(move |ui| {
                            let msg =
                                ui.global::<Messages>().get_download_failed() + &format!("{e}");
                            err_dialog(&msg);
                            ui.invoke_unset_loading();
                        })
                        .unwrap();
                    return None;
                }
                stop.store(true, sync::atomic::Ordering::Relaxed);

                if let Err(e) = self
                    .ui_weak
                    .upgrade_in_event_loop(|ui| ui.invoke_state_set_launching())
                {
                    error!("Failed to upgrade a weak pointer. Reason: {e}.");
                    self.ui_weak
                        .upgrade_in_event_loop(move |ui| {
                            err_dialog(&format!("{e}"));
                            ui.invoke_unset_loading();
                        })
                        .unwrap();
                    return None;
                }

                let java_path = self.game_list[game_index].java_path.clone();

                let (s, r) = sync::mpsc::channel();
                let ui_weak = self.ui_weak.clone();
                thread::spawn(move || match Command::new(java_path).args(cmd).spawn() {
                    Ok(_) => {
                        s.send(Some(())).unwrap();
                    }
                    Err(e) => {
                        error!("Failed to run command. Reason: {e}");
                        s.send(None).unwrap();
                        ui_weak
                            .upgrade_in_event_loop(move |ui| {
                                let msg =
                                    ui.global::<Messages>().get_start_failed() + &format!("\n{e}");
                                err_dialog(&msg);
                            })
                            .unwrap();
                    }
                });

                if r.recv().unwrap().is_some() {
                    if self.config.close_after_launch {
                        self.ui_weak
                            .upgrade_in_event_loop(|ui| ui.hide().unwrap())
                            .ok()?;
                    }
                } else {
                    slint::invoke_from_event_loop(|| {
                        err_dialog("Failed to run command.");
                    })
                    .ok()?;
                }
            }
            Err(e) => {
                error!("Failed to get launch command. Reason: {e}");
                self.ui_weak
                    .upgrade_in_event_loop(move |ui| {
                        let msg = ui.global::<Messages>().get_start_failed() + &format!("{e}");
                        err_dialog(&msg);
                    })
                    .unwrap();
            }
        }

        self.ui_weak
            .upgrade_in_event_loop(|ui| ui.invoke_unset_loading())
            .unwrap();
        Some(())
    }

    /// Load the account list from account.json (won't refresh ui)
    pub fn load_acc_list(&mut self) -> Result<(), std::io::Error> {
        self.acc_list.clear();

        if !exists("account.json")? {
            self.acc_list = vec![Account::default()];
            return self.save_acc_list();
        }

        let json = serde_json::from_str::<serde_json::Value>(&fs::read_to_string("account.json")?)?;
        if let Some(array) = json.as_array() {
            for item in array {
                let account = Account {
                    access_token: String::new(),
                    account_type: String::from(
                        item["account_type"]
                            .as_str()
                            .ok_or(ErrorKind::InvalidData)?,
                    ),
                    refresh_token: String::from(
                        item["token"].as_str().ok_or(ErrorKind::InvalidData)?,
                    ),
                    uuid: String::from(item["uuid"].as_str().ok_or(ErrorKind::InvalidData)?),
                    user_name: String::from(
                        item["user_name"].as_str().ok_or(ErrorKind::InvalidData)?,
                    ),
                };

                self.acc_list.push(account);
            }
        } else {
            error!("Failed to convert account.json to an array.");
            return Err(ErrorKind::InvalidData.into());
        }

        Ok(())
    }

    /// Load the configs from config.json (won't refresh ui)
    fn load_config(&mut self) -> Result<(), std::io::Error> {
        if exists(&"config.json")? {
            let json: serde_json::Value =
                serde_json::from_str(&fs::read_to_string("config.json")?.as_str())?;

            self.config.assets_source = String::from(
                json["assets_source"]
                    .as_str()
                    .ok_or(ErrorKind::InvalidData)?,
            );
            self.config.close_after_launch = json["close_after_launch"]
                .as_bool()
                .ok_or(ErrorKind::InvalidData)?;
            self.config.concurrency =
                json["concurrency"].as_u64().ok_or(ErrorKind::InvalidData)? as usize;
            self.config.fabric_source = String::from(
                json["fabric_source"]
                    .as_str()
                    .ok_or(ErrorKind::InvalidData)?,
            );
            self.config.forge_source = String::from(
                json["forge_source"]
                    .as_str()
                    .ok_or(ErrorKind::InvalidData)?,
            );
            self.config.game_path =
                String::from(json["game_path"].as_str().ok_or(ErrorKind::InvalidData)?);
            self.config.game_source =
                String::from(json["game_source"].as_str().ok_or(ErrorKind::InvalidData)?);
            self.config.height =
                String::from(json["height"].as_str().ok_or(ErrorKind::InvalidData)?);
            self.config.java_path =
                String::from(json["java_path"].as_str().ok_or(ErrorKind::InvalidData)?);
            self.config.libraries_source = String::from(
                json["libraries_source"]
                    .as_str()
                    .ok_or(ErrorKind::InvalidData)?,
            );
            self.config.width = String::from(json["width"].as_str().ok_or(ErrorKind::InvalidData)?);
            self.config.xms = String::from(json["xms"].as_str().ok_or(ErrorKind::InvalidData)?);
            self.config.xmx = String::from(json["xmx"].as_str().ok_or(ErrorKind::InvalidData)?);
        } else {
            self.save_config()?;
        }

        Ok(())
    }

    /// Load the game list (won't refresh ui)
    pub fn load_game_list(&mut self) -> Result<(), std::io::Error> {
        self.game_list.clear();

        let dir = self.config.game_path.clone() + "/versions";

        if !exists(&dir)? {
            // 空目录
            warn!("{dir} is empty.");
            return Ok(());
        }

        for version in list_dir(&dir)? {
            let mut game: Game;
            let path = dir.clone() + "/" + version.as_str();

            // 先加载原版json
            let json_path = path.clone() + "/" + &version.as_str() + ".json";
            if !exists(&json_path)? {
                warn!("{json_path} not exists.");
                continue;
            }
            if let Ok(json) =
                serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&json_path)?.as_str())
            {
                game = Game {
                    description: String::new(),
                    game_args: Vec::new(),
                    height: self.config.height.clone(),
                    java_path: self.config.java_path.clone(),
                    jvm_args: Vec::new(),
                    separated: false,
                    game_type: String::from(json["type"].as_str().ok_or(ErrorKind::InvalidData)?),
                    version: version,
                    width: self.config.width.clone(),
                    xms: self.config.xms.clone(),
                    xmx: self.config.xmx.clone(),
                };
            } else {
                // 异常，跳过此次加载
                warn!("Failed to load {version}.json.");
                continue;
            }

            // 若config.json存在，覆盖原版json
            let cfg_path = path.clone() + "/config.json";
            if exists(&cfg_path)? {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(
                    &fs::read_to_string(&cfg_path)?.as_str(),
                ) {
                    let mut game_args = Vec::new();
                    let mut jvm_args = Vec::new();

                    for arg in json["game_args"].as_array().ok_or(ErrorKind::InvalidData)? {
                        game_args.push(arg.as_str().ok_or(ErrorKind::InvalidData)?.to_string());
                    }

                    for arg in json["jvm_args"].as_array().ok_or(ErrorKind::InvalidData)? {
                        jvm_args.push(arg.as_str().ok_or(ErrorKind::InvalidData)?.to_string());
                    }

                    game.description =
                        String::from(json["description"].as_str().ok_or(ErrorKind::InvalidData)?);
                    game.game_args = game_args;
                    game.height =
                        String::from(json["height"].as_str().ok_or(ErrorKind::InvalidData)?);
                    game.java_path =
                        String::from(json["java_path"].as_str().ok_or(ErrorKind::InvalidData)?);
                    game.jvm_args = jvm_args;
                    game.separated = json["separated"].as_bool().ok_or(ErrorKind::InvalidData)?;
                    game.width =
                        String::from(json["width"].as_str().ok_or(ErrorKind::InvalidData)?);
                    game.xms = String::from(json["xms"].as_str().ok_or(ErrorKind::InvalidData)?);
                    game.xmx = String::from(json["xmx"].as_str().ok_or(ErrorKind::InvalidData)?);
                } else {
                    warn!("Failed to load {cfg_path}.");
                    continue;
                }
            }
            self.game_list.push(game);
        }
        Ok(())
    }

    /// Save the account list to account.json
    pub fn save_acc_list(&self) -> Result<(), std::io::Error> {
        let mut json = json!([]);
        for account in &self.acc_list {
            let node = serde_json::json!(
                {
                    "account_type": account.account_type,
                    "token": account.refresh_token,
                    "uuid": account.uuid,
                    "user_name": account.user_name,
                }
            );
            if let Some(array) = json.as_array_mut() {
                array.push(node);
            } else {
                error!("");
                return Err(ErrorKind::InvalidData.into());
            }
        }
        fs::write("account.json", json.to_string())?;
        Ok(())
    }

    /// Save the configs to config.json
    pub fn save_config(&self) -> Result<(), std::io::Error> {
        let json = json!(
            {
                "assets_source": self.config.assets_source,
                "close_after_launch": self.config.close_after_launch,
                "concurrency": self.config.concurrency,
                "fabric_source": self.config.fabric_source,
                "forge_source": self.config.forge_source,
                "game_path": self.config.game_path,
                "game_source": self.config.game_source,
                "height": self.config.height,
                "java_path": self.config.java_path,
                "libraries_source": self.config.libraries_source,
                "width": self.config.width,
                "xms": self.config.xms,
                "xmx": self.config.xmx,
            }
        );
        fs::write("config.json", json.to_string())
    }

    /// 保存管启格式的launcher_profiles.json，适配forge
    pub fn save_launcher_profiles(&self) -> Result<(), std::io::Error> {
        let mut json = json!({"profiles": {}});
        for game in &self.game_list {
            let node = serde_json::json!(
                {
                    "name": game.version,
                    "type": "custom",
                    "lastVersionId": game.version,
                }
            );
            json["profiles"][&game.version] = node;
        }

        fs::write(
            self.config.game_path.to_string() + "/launcher_profiles.json",
            json.to_string(),
        )
    }

    /// Refresh account list in ui
    pub fn refresh_ui_acc_list(&self) -> Option<()> {
        let ui = self.ui_weak.upgrade()?;
        let mut ui_acc_list: Vec<ModelRc<StandardListViewItem>> = Vec::new();
        for account in &self.acc_list {
            let account_name = StandardListViewItem::from(account.user_name.as_str());
            let account_type = StandardListViewItem::from(account.account_type.as_str());
            let model: Rc<VecModel<StandardListViewItem>> =
                Rc::from(VecModel::from(vec![account_name, account_type]));
            let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
            ui_acc_list.push(row);
        }
        ui.set_acc_list(ModelRc::from(Rc::from(VecModel::from(ui_acc_list))));
        Some(())
    }

    /// Refresh game list in ui
    pub fn refresh_ui_game_list(&self) -> Option<()> {
        let ui = self.ui_weak.upgrade()?;
        let mut ui_game_list: Vec<ModelRc<StandardListViewItem>> = Vec::new();
        for game in &self.game_list {
            let version = StandardListViewItem::from(game.version.as_str());
            let game_type = StandardListViewItem::from(game.game_type.as_str());
            let description = StandardListViewItem::from(game.description.as_str());
            let model: Rc<VecModel<StandardListViewItem>> =
                Rc::from(VecModel::from(vec![version, game_type, description]));
            let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
            ui_game_list.push(row);
        }
        ui.set_game_list(ModelRc::from(Rc::from(VecModel::from(ui_game_list))));
        Some(())
    }
}

impl Default for App {
    fn default() -> Self {
        App {
            acc_list: Vec::new(),
            config: Config::default(),
            device_code: String::new(),
            download_fabric_list: Vec::new(),
            download_forge_list: Vec::new(),
            download_game_list: Vec::new(),
            downloader: Downloader::default(),
            game_list: Vec::new(),
            ui_weak: slint::Weak::default(),
        }
    }
}
