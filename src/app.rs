//! CEMCL 主模块

use std::fs::{self, exists};
use std::io::ErrorKind;
use std::process::Command;
use std::rc::Rc;
use std::{sync, thread};

use log::{debug, error, info, warn};
use serde_json::json;
use slint::{ComponentHandle, ModelRc, StandardListViewItem, VecModel};

use crate::Config as UIConfig;
use crate::ConfigDL as UIConfigDL;
use crate::ConfigGeneral as UIConfigGeneral;
use crate::ConfigMC as UIConfigMC;
use crate::dialogs::msgbox::{self, MsgID, msg_dialog};
use crate::downloader::DownloadManager;
use crate::file_tools::list_dir;
use crate::mc::download::{Fabric, Forge, GameUrl};
use crate::mc::launch::DownloadError;
use crate::mc::{Account, Game, launch};
use crate::{AccountInner, AccountType, AppWindow, Messages, State};

#[derive(Debug)]
pub enum LauncherError {
    /// account.json invalid
    AccountConfigError,
    /// Connection error
    ConnectionError,
    /// Directory is not empty
    DirNotEmpty,
    /// File already exists
    FileAlreadyExists,
    /// File is busy
    FileBusy,
    /// File not found
    FileNotFound,
    /// game config.json invalid
    GameConfigError,
    /// Operation interrupted
    Interrupted,
    /// launcher config.json invalid
    LauncherConfigError,
    /// Network error
    NetworkError,
    /// Permission denied
    PermissionDenied,
    /// Index out of range
    OutOfRange,
    /// Weak pointer upgrade error
    WeakPtrError,
    /// Others
    Unknown,
}

impl From<std::io::Error> for LauncherError {
    fn from(value: std::io::Error) -> Self {
        match value.kind() {
            ErrorKind::AlreadyExists => LauncherError::FileAlreadyExists,
            ErrorKind::ConnectionAborted
            | ErrorKind::ConnectionRefused
            | ErrorKind::ConnectionReset
            | ErrorKind::NotConnected => LauncherError::ConnectionError,
            ErrorKind::DirectoryNotEmpty => LauncherError::DirNotEmpty,
            ErrorKind::ExecutableFileBusy => LauncherError::FileBusy,
            ErrorKind::NotFound => LauncherError::FileNotFound,
            ErrorKind::Interrupted => LauncherError::Interrupted,
            ErrorKind::NetworkDown | ErrorKind::NetworkUnreachable => LauncherError::NetworkError,
            ErrorKind::PermissionDenied | ErrorKind::ReadOnlyFilesystem => {
                LauncherError::PermissionDenied
            }
            _ => LauncherError::Unknown,
        }
    }
}

impl From<DownloadError> for LauncherError {
    fn from(value: DownloadError) -> Self {
        match value {
            DownloadError::Cancelled => LauncherError::Interrupted,
            DownloadError::Failed => LauncherError::Unknown,
            DownloadError::IOError(e) => e.into(),
            DownloadError::Other(_) => LauncherError::Unknown,
        }
    }
}

impl From<serde_json::Error> for LauncherError {
    fn from(value: serde_json::Error) -> Self {
        std::io::Error::from(value).into()
    }
}

impl std::fmt::Display for LauncherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LauncherError::AccountConfigError => write!(f, "Account config error"),
            LauncherError::ConnectionError => write!(f, "Connection error"),
            LauncherError::DirNotEmpty => write!(f, "Directory is not empty"),
            LauncherError::FileAlreadyExists => write!(f, "File already exists"),
            LauncherError::FileBusy => write!(f, "File is busy"),
            LauncherError::FileNotFound => write!(f, "File not found"),
            LauncherError::GameConfigError => write!(f, "Game config error"),
            LauncherError::Interrupted => write!(f, "Operation interrupted"),
            LauncherError::LauncherConfigError => write!(f, "Launcher config error"),
            LauncherError::NetworkError => write!(f, "Network error"),
            LauncherError::PermissionDenied => write!(f, "Permission denied"),
            LauncherError::OutOfRange => write!(f, "Index out of range"),
            LauncherError::WeakPtrError => write!(f, "Failed to upgrade a weak pointer"),
            LauncherError::Unknown => write!(f, "Unknown error"),
        }
    }
}

#[derive(Clone)]
pub struct ConfigGeneral {
    /// 启动后关闭启动器
    pub close_after_launch: bool,
    /// .minecraft路径
    pub game_path: String,
}

#[derive(Clone)]
pub struct ConfigDL {
    /// assets下载源
    pub assets_source: String,
    /// 下载时的最大并发数量
    pub concurrency: usize,
    /// Fabric下载源
    pub fabric_source: String,
    /// Forge下载源
    pub forge_source: String,
    /// MC本体下载源
    pub game_source: String,
    /// libraries下载源
    pub libraries_source: String,
}

#[derive(Clone)]
pub struct ConfigMC {
    /// 默认游戏窗口高度
    pub height: String,
    /// java可执行文件路径
    pub java_path: String,
    /// 默认游戏窗口宽度
    pub width: String,
    /// 默认JVM最小内存
    pub xms: String,
    /// 默认JVM最大内存
    pub xmx: String,
}

/// 启动器配置
#[derive(Clone)]
pub struct Config {
    /// 通用
    pub general: ConfigGeneral,
    /// 下载
    pub dl: ConfigDL,
    /// MC
    pub mc: ConfigMC,
}

impl Default for ConfigGeneral {
    fn default() -> Self {
        ConfigGeneral {
            close_after_launch: false,
            game_path: String::from(".minecraft"),
        }
    }
}

impl Default for ConfigDL {
    fn default() -> Self {
        ConfigDL {
            assets_source: String::from("https://resources.download.minecraft.net"),
            concurrency: 10,
            fabric_source: String::from("https://maven.fabricmc.net"),
            forge_source: String::from("https://files.minecraftforge.net"),
            game_source: String::from("https://piston-meta.mojang.com"),
            libraries_source: String::from("https://libraries.minecraft.net"),
        }
    }
}

impl Default for ConfigMC {
    fn default() -> Self {
        ConfigMC {
            height: String::from("600"),
            java_path: String::from("java"),
            width: String::from("800"),
            xms: String::from("1G"),
            xmx: String::from("2G"),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            general: ConfigGeneral::default(),
            dl: ConfigDL::default(),
            mc: ConfigMC::default(),
        }
    }
}

impl From<UIConfig> for Config {
    fn from(config: UIConfig) -> Self {
        Config {
            general: ConfigGeneral {
                close_after_launch: config.general.close_after_launch,
                game_path: config.general.game_path.to_string(),
            },
            dl: ConfigDL {
                assets_source: config.dl.assets_source.to_string(),
                concurrency: config.dl.concurrency as usize,
                fabric_source: config.dl.fabric_source.to_string(),
                forge_source: config.dl.forge_source.to_string(),
                game_source: config.dl.game_source.to_string(),
                libraries_source: config.dl.libraries_source.to_string(),
            },
            mc: ConfigMC {
                height: config.mc.height.to_string(),
                java_path: config.mc.java_path.to_string(),
                width: config.mc.width.to_string(),
                xms: config.mc.xms.to_string(),
                xmx: config.mc.xmx.to_string(),
            },
        }
    }
}

impl From<Config> for UIConfig {
    fn from(config: Config) -> Self {
        UIConfig {
            general: UIConfigGeneral {
                close_after_launch: config.general.close_after_launch,
                game_path: config.general.game_path.into(),
            },
            dl: UIConfigDL {
                assets_source: config.dl.assets_source.into(),
                concurrency: config.dl.concurrency as i32,
                fabric_source: config.dl.fabric_source.into(),
                forge_source: config.dl.forge_source.into(),
                game_source: config.dl.game_source.into(),
                libraries_source: config.dl.libraries_source.into(),
            },
            mc: UIConfigMC {
                height: config.mc.height.into(),
                java_path: config.mc.java_path.into(),
                width: config.mc.width.into(),
                xms: config.mc.xms.into(),
                xmx: config.mc.xmx.into(),
            },
        }
    }
}

impl From<AccountInner> for Account {
    fn from(account: AccountInner) -> Self {
        Account {
            access_token: String::new(),
            account_type: match account.account_type {
                AccountType::Legacy => "Legacy".to_string(),
                AccountType::MSA => "msa".to_string(),
                AccountType::Other => "".to_string(),
            },
            refresh_token: account.token.into(),
            uuid: account.uuid.into(),
            user_name: account.user_name.into(),
        }
    }
}

impl From<Account> for AccountInner {
    fn from(account: Account) -> Self {
        let account_type = if account.account_type == "Legacy" {
            AccountType::Legacy
        } else if account.account_type == "msa" {
            AccountType::MSA
        } else {
            AccountType::Other
        };
        AccountInner {
            account_type,
            token: account.refresh_token.into(),
            user_name: account.user_name.into(),
            uuid: account.uuid.into(),
        }
    }
}

pub struct App {
    pub acc_list: Vec<Account>,
    pub config: Config,
    pub current_acc_index: usize,
    pub device_code: String,
    pub download_fabric_list: Vec<Fabric>,
    pub download_forge_list: Vec<Forge>,
    pub download_game_list: Vec<GameUrl>,
    pub downloader: DownloadManager,
    pub game_list: Vec<Game>,
    pub ui_weak: slint::Weak<AppWindow>,
}

impl App {
    /// Create a new app with the weak pointer of ui provided
    pub fn new(ui_weak: slint::Weak<AppWindow>) -> Result<App, LauncherError> {
        let mut app = App::default();

        if let Err(e) = app.load_acc_list() {
            warn!("Failed to load account list. Reason: {e}.");
            msg_dialog(MsgID::LoadAccFailed(e.to_string()));
        }

        if let Err(e) = app.load_config() {
            warn!("Failed to load config. Reason: {e}.");
            msg_dialog(MsgID::LoadConfigFailed(e.to_string()));
        }

        if let Err(e) = app.load_game_list() {
            warn!("Failed to load game list. Reason: {e}.");
            msg_dialog(MsgID::LoadGameFailed(e.to_string()));
        }

        app.downloader = DownloadManager::new(app.config.dl.concurrency);

        app.ui_weak = ui_weak;
        app.refresh_ui_acc_list()?;
        app.refresh_ui_game_list()?;
        app.refresh_ui_settings()?;

        if let Some(ui) = app.ui_weak.upgrade() {
            ui.set_acc_index(app.current_acc_index as i32);
        }

        Ok(app)
    }

    /// Add an account to self.acc_list, also call self.save_acc_list() and self.refresh_ui_acc_list()
    pub fn add_account(&mut self, account: &Account) -> Result<(), LauncherError> {
        self.acc_list.push(account.clone());
        self.save_acc_list()?;
        self.refresh_ui_acc_list()
    }

    /// Add a game to self.game_list, also call game.save(), self.save_launcher_profiles() and self.refresh_ui_game_list()
    pub fn add_game(&mut self, game: &Game) -> Result<(), LauncherError> {
        self.game_list.push(game.clone());
        let dir = self.config.general.game_path.clone() + "/versions/" + &game.version;
        if !exists(&dir)? {
            fs::create_dir_all(&dir)?;
        }
        let path = dir + "/config.json";
        game.save(&path)?;
        self.save_launcher_profiles()?;
        self.refresh_ui_game_list()
    }

    /// Delete an account, also call self.save_acc_list() and self.refresh_ui_acc_list()
    pub fn del_account(&mut self, index: usize) -> Result<(), LauncherError> {
        // if index >= self.acc_list.len() {
        //     error!("Index out of bounds: the len is {} but the index is {index}.", self.acc_list.len());
        //     return None;
        // }
        self.acc_list.remove(index);
        self.save_acc_list()?;
        self.refresh_ui_acc_list()
    }

    /// Delete a game, also delete the game directory and call self.save_launcher_profiles() and self.refresh_ui_game_list()
    pub fn del_game(&mut self, index: usize) -> Result<(), LauncherError> {
        // if index >= self.game_list.len() {
        //     error!("Index out of bounds: the len is {} but the index is {index}.", self.game_list.len());
        //     return None;
        // }
        let path =
            self.config.general.game_path.clone() + "/versions/" + &self.game_list[index].version;
        self.game_list.remove(index);
        fs::remove_dir_all(path)?;
        self.save_launcher_profiles()?;
        self.refresh_ui_game_list()
    }

    /// Edit an account, also call self.save_acc_list() and self.refresh_ui_acc_list()
    pub fn edit_account(&mut self, index: usize, account: Account) -> Result<(), LauncherError> {
        self.acc_list[index] = account;
        self.save_acc_list()?;
        self.refresh_ui_acc_list()
    }

    /// Edit a game, also call Game::save, self.save_launcher_profiles() and self.refresh_ui_game_list()
    pub fn edit_game(&mut self, index: usize, game: Game) -> Result<(), LauncherError> {
        let path =
            self.config.general.game_path.clone() + "/versions/" + &game.version + "/config.json";
        game.save(&path)?;
        self.game_list[index] = game;
        self.save_launcher_profiles()?;
        self.refresh_ui_game_list()
    }

    /// Get the current index of account list in ui, return None when index is out of range
    pub fn get_acc_index(&self) -> Result<usize, LauncherError> {
        let ui = self.ui_weak.upgrade().ok_or(LauncherError::WeakPtrError)?;
        let index = ui.get_acc_index() as usize;
        if index >= self.acc_list.len() {
            warn!(
                "Index out of bounds: the len is {} but the index is {index}.",
                self.acc_list.len()
            );
            return Err(LauncherError::OutOfRange);
        }
        Ok(index)
    }

    /// Get the current index of game list in ui, return None when index is out of range
    pub fn get_game_index(&self) -> Result<usize, LauncherError> {
        let ui = self.ui_weak.upgrade().ok_or(LauncherError::WeakPtrError)?;
        let index = ui.get_game_index() as usize;
        if index >= self.game_list.len() {
            warn!(
                "Index out of bounds: the len is {} but the index is {index}.",
                self.game_list.len()
            );
            return Err(LauncherError::OutOfRange);
        }
        Ok(index)
    }

    // we should get acc index and game index in main thread
    /// Launch the game
    pub async fn launch(
        &mut self,
        acc_index: usize,
        game_index: usize,
    ) -> Result<(), LauncherError> {
        if self
            .ui_weak
            .upgrade_in_event_loop(|ui| {
                ui.set_progress(0.0);
            })
            .is_err()
        {
            return Err(LauncherError::Unknown);
        }

        if acc_index >= self.acc_list.len() || game_index >= self.game_list.len() {
            warn!(
                "Index out of bounds: the len is ({}, {}) but the index is ({acc_index}, {game_index}).",
                self.acc_list.len(),
                self.game_list.len()
            );
            msgbox::msg_dialog(MsgID::BothNotSelected);
            return Err(LauncherError::OutOfRange);
        }

        // refresh access_token
        self.ui_weak
            .upgrade_in_event_loop(|ui| ui.set_state(State::LoggingIn))
            .unwrap();
        if self.acc_list[acc_index]
            .refresh(self.ui_weak.clone())
            .await
            .is_none()
        {
            error!("Failed to login.");
            msgbox::msg_dialog(MsgID::LoginFailed("None".to_string()));
            return Err(LauncherError::NetworkError);
        }

        match launch::get_launch_command(
            &self.acc_list[acc_index],
            &self.game_list[game_index],
            &self.config.general.game_path,
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

                self.ui_weak
                    .upgrade_in_event_loop(|ui| ui.set_state(State::Downloading))
                    .unwrap();

                // UI进度条
                let ui_weak_clone = self.ui_weak.clone();
                let f = move |progress: (u64, u64)| {
                    info!("Download progress: {}/{}", progress.0, progress.1);
                    ui_weak_clone
                        .upgrade_in_event_loop(move |ui| {
                            ui.set_progress((progress.0 as f32) / (progress.1 as f32));
                        })
                        .unwrap();
                };

                if let Err(e) = launch::download_all(
                    &self.config.general.game_path,
                    &self.config.dl,
                    &game_download,
                    &self.downloader,
                    f,
                ) {
                    error!("Failed to download. Reason: {e}");
                    //msgdialog
                    return Err(e.into());
                }

                if let Err(e) = self
                    .ui_weak
                    .upgrade_in_event_loop(|ui| ui.set_state(State::Launching))
                {
                    error!("Failed to upgrade a weak pointer. Reason: {e}.");
                    msgbox::msg_dialog(MsgID::WeakPtrError);
                    return Err(LauncherError::WeakPtrError);
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
                        msgbox::msg_dialog(MsgID::LaunchFailed(format!("{e}")));
                    }
                });

                if r.recv().unwrap().is_some() {
                    if self.config.general.close_after_launch {
                        self.ui_weak
                            .upgrade_in_event_loop(|ui| ui.hide().unwrap())
                            .unwrap();
                    }
                } else {
                    slint::invoke_from_event_loop(|| {
                        msgbox::msg_dialog(MsgID::LaunchFailed(String::from(
                            "Failed to run command.",
                        )));
                    })
                    .unwrap();
                }
            }
            Err(e) => {
                error!("Failed to get launch command. Reason: {e}");
                msgbox::msg_dialog(MsgID::LaunchFailed(format!("{e}")));
            }
        }

        self.ui_weak
            .upgrade_in_event_loop(|ui| ui.set_state(State::Spare))
            .unwrap();
        Ok(())
    }

    /// Load the account list from account.json (won't refresh ui)
    pub fn load_acc_list(&mut self) -> Result<(), LauncherError> {
        self.acc_list.clear();

        if !exists("account.json")? {
            self.acc_list = vec![Account::default()];
            return self.save_acc_list();
        }

        let json = serde_json::from_str::<serde_json::Value>(&fs::read_to_string("account.json")?)?;
        if let Some(array) = json["accounts"].as_array() {
            for item in array {
                let account = Account {
                    access_token: String::new(),
                    account_type: String::from(
                        item["account_type"]
                            .as_str()
                            .ok_or(LauncherError::AccountConfigError)?,
                    ),
                    refresh_token: String::from(
                        item["token"]
                            .as_str()
                            .ok_or(LauncherError::AccountConfigError)?,
                    ),
                    uuid: String::from(
                        item["uuid"]
                            .as_str()
                            .ok_or(LauncherError::AccountConfigError)?,
                    ),
                    user_name: String::from(
                        item["user_name"]
                            .as_str()
                            .ok_or(LauncherError::AccountConfigError)?,
                    ),
                };

                self.acc_list.push(account);
            }
        } else {
            error!("Failed to convert account.json to an array.");
            return Err(LauncherError::AccountConfigError);
        }

        self.current_acc_index = json["current"]
            .as_i64()
            .ok_or(LauncherError::AccountConfigError)? as usize;

        Ok(())
    }

    /// Load the configs from config.json (won't refresh ui)
    fn load_config(&mut self) -> Result<(), LauncherError> {
        if exists(&"config.json")? {
            let json: serde_json::Value =
                serde_json::from_str(&fs::read_to_string("config.json")?.as_str())?;

            self.config.dl.assets_source = String::from(
                json["assets_source"]
                    .as_str()
                    .ok_or(LauncherError::LauncherConfigError)?,
            );
            self.config.general.close_after_launch = json["close_after_launch"]
                .as_bool()
                .ok_or(LauncherError::LauncherConfigError)?;
            self.config.dl.concurrency = json["concurrency"]
                .as_u64()
                .ok_or(LauncherError::LauncherConfigError)?
                as usize;
            self.config.dl.fabric_source = String::from(
                json["fabric_source"]
                    .as_str()
                    .ok_or(LauncherError::LauncherConfigError)?,
            );
            self.config.dl.forge_source = String::from(
                json["forge_source"]
                    .as_str()
                    .ok_or(LauncherError::LauncherConfigError)?,
            );
            self.config.general.game_path = String::from(
                json["game_path"]
                    .as_str()
                    .ok_or(LauncherError::LauncherConfigError)?,
            );
            self.config.dl.game_source = String::from(
                json["game_source"]
                    .as_str()
                    .ok_or(LauncherError::LauncherConfigError)?,
            );
            self.config.mc.height = String::from(
                json["height"]
                    .as_str()
                    .ok_or(LauncherError::LauncherConfigError)?,
            );
            self.config.mc.java_path = String::from(
                json["java_path"]
                    .as_str()
                    .ok_or(LauncherError::LauncherConfigError)?,
            );
            self.config.dl.libraries_source = String::from(
                json["libraries_source"]
                    .as_str()
                    .ok_or(LauncherError::LauncherConfigError)?,
            );
            self.config.mc.width = String::from(
                json["width"]
                    .as_str()
                    .ok_or(LauncherError::LauncherConfigError)?,
            );
            self.config.mc.xms = String::from(
                json["xms"]
                    .as_str()
                    .ok_or(LauncherError::LauncherConfigError)?,
            );
            self.config.mc.xmx = String::from(
                json["xmx"]
                    .as_str()
                    .ok_or(LauncherError::LauncherConfigError)?,
            );
        } else {
            self.save_config()?;
        }

        Ok(())
    }

    /// Load the game list (won't refresh ui)
    pub fn load_game_list(&mut self) -> Result<(), LauncherError> {
        self.game_list.clear();

        let dir = self.config.general.game_path.clone() + "/versions";

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
                    height: self.config.mc.height.clone(),
                    java_path: self.config.mc.java_path.clone(),
                    jvm_args: Vec::new(),
                    separated: false,
                    game_type: String::from(
                        json["type"]
                            .as_str()
                            .ok_or(LauncherError::GameConfigError)?,
                    ),
                    version: version,
                    width: self.config.mc.width.clone(),
                    xms: self.config.mc.xms.clone(),
                    xmx: self.config.mc.xmx.clone(),
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

                    for arg in json["game_args"]
                        .as_array()
                        .ok_or(LauncherError::GameConfigError)?
                    {
                        game_args.push(
                            arg.as_str()
                                .ok_or(LauncherError::GameConfigError)?
                                .to_string(),
                        );
                    }

                    for arg in json["jvm_args"]
                        .as_array()
                        .ok_or(LauncherError::GameConfigError)?
                    {
                        jvm_args.push(
                            arg.as_str()
                                .ok_or(LauncherError::GameConfigError)?
                                .to_string(),
                        );
                    }

                    game.description = String::from(
                        json["description"]
                            .as_str()
                            .ok_or(LauncherError::GameConfigError)?,
                    );
                    game.game_args = game_args;
                    game.height = String::from(
                        json["height"]
                            .as_str()
                            .ok_or(LauncherError::GameConfigError)?,
                    );
                    game.java_path = String::from(
                        json["java_path"]
                            .as_str()
                            .ok_or(LauncherError::GameConfigError)?,
                    );
                    game.jvm_args = jvm_args;
                    game.separated = json["separated"]
                        .as_bool()
                        .ok_or(LauncherError::GameConfigError)?;
                    game.width = String::from(
                        json["width"]
                            .as_str()
                            .ok_or(LauncherError::GameConfigError)?,
                    );
                    game.xms =
                        String::from(json["xms"].as_str().ok_or(LauncherError::GameConfigError)?);
                    game.xmx =
                        String::from(json["xmx"].as_str().ok_or(LauncherError::GameConfigError)?);
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
    pub fn save_acc_list(&self) -> Result<(), LauncherError> {
        let acc_index = match self.get_acc_index() {
            Ok(i) => i,
            Err(e) => {
                error!("{e}");
                0
            }
        };

        let mut json = json!(
            {
                "current": acc_index,
                "accounts": []
            }
        );
        for account in &self.acc_list {
            let node = serde_json::json!(
                {
                    "account_type": account.account_type,
                    "token": account.refresh_token,
                    "uuid": account.uuid,
                    "user_name": account.user_name,
                }
            );
            if let Some(array) = json["accounts"].as_array_mut() {
                array.push(node);
            } else {
                error!("");
                return Err(LauncherError::AccountConfigError);
            }
        }
        fs::write("account.json", json.to_string())?;
        Ok(())
    }

    /// Save the configs to config.json
    pub fn save_config(&self) -> Result<(), LauncherError> {
        let json = json!(
            {
                "assets_source": self.config.dl.assets_source,
                "close_after_launch": self.config.general.close_after_launch,
                "concurrency": self.config.dl.concurrency,
                "fabric_source": self.config.dl.fabric_source,
                "forge_source": self.config.dl.forge_source,
                "game_path": self.config.general.game_path,
                "game_source": self.config.dl.game_source,
                "height": self.config.mc.height,
                "java_path": self.config.mc.java_path,
                "libraries_source": self.config.dl.libraries_source,
                "width": self.config.mc.width,
                "xms": self.config.mc.xms,
                "xmx": self.config.mc.xmx,
            }
        );
        fs::write("config.json", json.to_string())?;
        Ok(())
    }

    /// 保存官方启动器格式的launcher_profiles.json，适配forge
    pub fn save_launcher_profiles(&self) -> Result<(), LauncherError> {
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
            self.config.general.game_path.to_string() + "/launcher_profiles.json",
            json.to_string(),
        )?;
        Ok(())
    }

    /// Set the config from ui, also save the config to config.json
    pub fn set_config(&mut self) -> Result<(), LauncherError> {
        let ui = self.ui_weak.upgrade().ok_or(LauncherError::WeakPtrError)?;
        self.config = ui.get_config().into();
        self.save_config()
    }

    /// Refresh account list in ui
    pub fn refresh_ui_acc_list(&self) -> Result<(), LauncherError> {
        let ui = self.ui_weak.upgrade().ok_or(LauncherError::WeakPtrError)?;
        ui.set_acc_list(ModelRc::from(Rc::from(VecModel::from(
            self.acc_list
                .iter()
                .map(|acc| acc.clone().into())
                .collect::<Vec<AccountInner>>(),
        ))));
        Ok(())
    }

    /// Refresh settings in ui
    pub fn refresh_ui_settings(&self) -> Result<(), LauncherError> {
        let ui = self.ui_weak.upgrade().ok_or(LauncherError::WeakPtrError)?;
        ui.set_config(self.config.clone().into());
        ui.set_authors(env!("CARGO_PKG_AUTHORS").into());
        ui.set_version(env!("CARGO_PKG_VERSION").into());
        Ok(())
    }

    /// Refresh game list in ui
    pub fn refresh_ui_game_list(&self) -> Result<(), LauncherError> {
        let ui = self.ui_weak.upgrade().ok_or(LauncherError::WeakPtrError)?;
        let mut combo_box_list: Vec<slint::SharedString> = Vec::new();
        let mut ui_game_list: Vec<ModelRc<StandardListViewItem>> = Vec::new();
        for game in &self.game_list {
            let version = StandardListViewItem::from(game.version.as_str());
            let game_type = StandardListViewItem::from(game.game_type.as_str());
            let description = StandardListViewItem::from(game.description.as_str());
            let model: Rc<VecModel<StandardListViewItem>> =
                Rc::from(VecModel::from(vec![version, game_type, description]));
            let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
            combo_box_list.push(game.version.clone().into());
            ui_game_list.push(row);
        }
        ui.set_combo_box_model(ModelRc::from(Rc::from(VecModel::from(combo_box_list))));
        ui.set_game_model(ModelRc::from(Rc::from(VecModel::from(ui_game_list))));
        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        App {
            acc_list: Vec::new(),
            config: Config::default(),
            current_acc_index: 0,
            device_code: String::new(),
            download_fabric_list: Vec::new(),
            download_forge_list: Vec::new(),
            download_game_list: Vec::new(),
            downloader: DownloadManager::default(),
            game_list: Vec::new(),
            ui_weak: slint::Weak::default(),
        }
    }
}
