//! CEMCL Application Runtime

use clipboard::{ClipboardContext, ClipboardProvider};
use log::{error, info};
use mc::{
    MCInstallation,
    account::{Account, auth::AuthPollAction},
    manifest::{
        Fabric, Forge, MCDL, download_fabric, download_forge, download_mc, list_fabric, list_forge,
    },
};
use serde_json::json;
use std::{collections::HashMap, fs, process::Command};
use tokio::time::{Duration, sleep};

use crate::{
    account::{frontend_account, to_account_type},
    version::{
        ConfigMC, VersionManager, frontend_fabric, frontend_forge, frontend_mc_config,
        frontend_mc_dl, frontend_mc_info, frontend_mc_type,
    },
};
use downloader::{Config as DownloaderConfig, DownloadManager, task::TaskInfo};
use frontend::{
    UICommand,
    UIUpdate::{self, SetAccountIndex},
    game::{MCInfo, ModType},
};

use crate::{account::AccountManager, errors::LauncherError};

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
    pub concurrency: u32,
    /// Fabric下载源
    pub fabric_source: String,
    /// Forge下载源
    pub forge_source: String,
    /// MC本体下载源
    pub game_source: String,
    /// libraries下载源
    pub libraries_source: String,
}

impl From<ConfigDL> for DownloaderConfig {
    fn from(value: ConfigDL) -> Self {
        let mut map = HashMap::new();
        map.insert("assets_source".to_string(), value.assets_source);
        map.insert("fabric_source".to_string(), value.fabric_source);
        map.insert("forge_source".to_string(), value.forge_source);
        map.insert("game_source".to_string(), value.game_source);
        map.insert("libraries_source".to_string(), value.libraries_source);
        Self {
            concurrency: value.concurrency,
            mirrors: map,
        }
    }
}

impl From<DownloaderConfig> for ConfigDL {
    fn from(value: DownloaderConfig) -> Self {
        Self {
            concurrency: value.concurrency,
            assets_source: value.mirrors["assets_source"].clone(),
            fabric_source: value.mirrors["fabric_source"].clone(),
            forge_source: value.mirrors["forge_source"].clone(),
            game_source: value.mirrors["game_source"].clone(),
            libraries_source: value.mirrors["libraries_source"].clone(),
        }
    }
}

impl From<frontend::ConfigDL> for ConfigDL {
    fn from(value: frontend::ConfigDL) -> Self {
        Self {
            assets_source: value.assets_source,
            concurrency: value.concurrency,
            fabric_source: value.fabric_source,
            forge_source: value.forge_source,
            game_source: value.game_source,
            libraries_source: value.libraries_source,
        }
    }
}

impl From<frontend::ConfigGeneral> for ConfigGeneral {
    fn from(value: frontend::ConfigGeneral) -> Self {
        Self {
            close_after_launch: value.close_after_launch,
            game_path: value.game_path,
        }
    }
}

impl From<frontend::ConfigMC> for ConfigMC {
    fn from(value: frontend::ConfigMC) -> Self {
        Self {
            height: value.height,
            java_path: value.java_path,
            path: value.path,
            width: value.width,
            wrapper: value.wrapper,
            xms: value.xms,
            xmx: value.xmx,
        }
    }
}

impl From<ConfigDL> for frontend::ConfigDL {
    fn from(value: ConfigDL) -> Self {
        Self {
            assets_source: value.assets_source,
            concurrency: value.concurrency,
            fabric_source: value.fabric_source,
            forge_source: value.forge_source,
            game_source: value.game_source,
            libraries_source: value.libraries_source,
        }
    }
}

impl From<ConfigGeneral> for frontend::ConfigGeneral {
    fn from(value: ConfigGeneral) -> Self {
        Self {
            close_after_launch: value.close_after_launch,
            game_path: value.game_path,
        }
    }
}

impl From<ConfigMC> for frontend::ConfigMC {
    fn from(value: ConfigMC) -> Self {
        Self {
            height: value.height,
            java_path: value.java_path,
            path: value.path,
            width: value.width,
            wrapper: value.wrapper,
            xms: value.xms,
            xmx: value.xmx,
        }
    }
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
            height: 600,
            java_path: String::from("java"),
            path: ConfigGeneral::default().game_path,
            width: 800,
            wrapper: String::new(),
            xms: String::from("1G"),
            xmx: String::from("2G"),
        }
    }
}

struct CacheData {
    dl_mc_list: Option<Vec<MCDL>>,
    dl_fabric_list: Option<Vec<Fabric>>,
    dl_forge_list: Option<Vec<Forge>>,
}

impl CacheData {
    pub fn new() -> Self {
        Self {
            dl_fabric_list: None,
            dl_forge_list: None,
            dl_mc_list: None,
        }
    }
}

pub struct AppRuntime {
    account_manager: AccountManager,
    cache: CacheData,
    config: ConfigGeneral,
    cmd_receiver: Option<tokio::sync::mpsc::UnboundedReceiver<UICommand>>,
    downloader: DownloadManager,
    update_sender: tokio::sync::mpsc::UnboundedSender<UIUpdate>,
    version_manager: VersionManager,
}

impl AppRuntime {
    pub fn new(
        update_sender: tokio::sync::mpsc::UnboundedSender<UIUpdate>,
        cmd_receiver: tokio::sync::mpsc::UnboundedReceiver<UICommand>,
    ) -> Result<Self, LauncherError> {
        let account_manager = AccountManager::new()?;
        let (config_dl, config_general, config_mc) = AppRuntime::i_load_config()?;
        let downloader = DownloadManager::new(config_dl.into());
        let version_manager = VersionManager::new(config_mc.clone())?;

        Ok(Self {
            account_manager,
            cache: CacheData::new(),
            config: config_general,
            cmd_receiver: Some(cmd_receiver),
            downloader,
            update_sender,
            version_manager,
        })
    }

    fn init(&self) -> Result<(), LauncherError> {
        self.refresh_ui_info()?;
        self.refresh_ui_acc_list()?;
        self.refresh_ui_config()?;
        self.refresh_ui_version_list()?;
        Ok(())
    }

    async fn handle(&mut self, cmd: UICommand) -> Result<(), LauncherError> {
        match cmd {
            UICommand::AddOfflineAccount(user_name, uuid) => {
                self.account_manager.add(Account {
                    access_token: String::new(),
                    account_type: mc::account::AccountType::Legacy,
                    refresh_token: String::new(),
                    uuid,
                    user_name,
                })?;
                self.refresh_ui_acc_list()?;
                self.update_sender.send(UIUpdate::QuitLoginDialog)?;
            }
            UICommand::AddGame(mc_type, mc_index, mod_type, mod_index, config) => {
                let mut mc_list = if let Some(list) = self.cache.dl_mc_list.take() {
                    list
                } else {
                    mc::manifest::list_game(self.config.game_path.clone()).await?
                };

                if let Some(filter) = mc_type {
                    mc_list = mc_list
                        .into_iter()
                        .filter(|v| frontend_mc_type(v.game_type.clone()) == filter)
                        .collect();
                }

                let version = &mc_list[mc_index as usize];
                let mut ver = version.version.clone();
                let ver_type = version.game_type.clone();
                download_mc(&self.config.game_path, version.clone()).await?;

                if let Some(filter) = mod_type {
                    match filter {
                        ModType::Fabric => {
                            let mod_list = if let Some(list) = self.cache.dl_fabric_list.take() {
                                list
                            } else {
                                mc::manifest::list_fabric(&version.version).await?
                            };

                            let fabric = &mod_list[mod_index as usize];

                            download_fabric(
                                &self.config.game_path,
                                &version.version,
                                fabric.clone(),
                            )
                            .await?;

                            ver = format!(
                                "fabric-loader-{fabric_version}-{ver}",
                                fabric_version = fabric.loader_version,
                            );
                        }
                        ModType::Forge => {
                            let mod_list = if let Some(list) = self.cache.dl_forge_list.take() {
                                list
                            } else {
                                mc::manifest::list_forge(&version.version).await?
                            };

                            let forge = &mod_list[mod_index as usize];

                            let task_info =
                                download_forge(&version.version, forge.clone(), "{forge_source}");

                            let java_path = config.java_path.clone();
                            let forge_path = task_info.save_path.clone();
                            let f = move || {
                                if let Err(e) = Command::new(&java_path)
                                    .arg("-jar")
                                    .arg(&forge_path)
                                    .spawn()
                                {
                                    error!("Failed to run forge installer. Reason: {e}.");
                                }
                            };

                            let task = TaskInfo::new(
                                task_info.url,
                                task_info.save_path,
                                None,
                                Some(Box::new(f)),
                                None,
                                None,
                            );

                            let id = format!("{0}-forge-{1}", &version.version, &forge.version);
                            self.downloader.add_taskset(id.clone(), vec![task]);
                            self.downloader.start_taskset(id.clone())?;
                            ver = id;
                        }
                    }
                }
                let installation = MCInstallation {
                    description: config.description,
                    game_args: config.game_args,
                    game_type: ver_type,
                    height: config.height,
                    java_path: config.java_path,
                    jvm_args: config.jvm_args,
                    separated: config.separated,
                    version: ver,
                    width: config.width,
                    wrapper: config.wrapper,
                    xms: config.xms,
                    xmx: config.xmx,
                };
                self.version_manager.add(&installation)?;
                self.refresh_ui_version_list()?;
                self.update_sender.send(UIUpdate::QuitAddGameDialog)?;
            }
            UICommand::DelAccount(index) => {
                self.account_manager.del(index)?;
                self.refresh_ui_acc_list()?;
            }
            UICommand::DelGame(index) => {
                self.version_manager.del(index)?;
                self.refresh_ui_version_list()?;
                self.update_sender.send(UIUpdate::QuitEditGameDialog)?;
            }
            UICommand::DelJava(index) => {}
            UICommand::EditAccount(index, account) => {
                let mut i_account = self.account_manager.get(index).clone();
                i_account.account_type = to_account_type(account.account_type);
                i_account.refresh_token = account.token;
                i_account.user_name = account.user_name;
                i_account.uuid = account.uuid;
                self.account_manager.edit(index, i_account)?;
                self.refresh_ui_acc_list()?;
            }
            UICommand::EditGame(index, installation) => {
                let mut version = self.version_manager.get(index).clone();
                version.description = installation.description;
                version.game_args = installation.game_args;
                version.height = installation.height;
                version.java_path = installation.java_path;
                version.jvm_args = installation.jvm_args;
                version.separated = installation.separated;
                version.width = installation.width;
                version.wrapper = installation.wrapper;
                version.xms = installation.xms;
                version.xmx = installation.xmx;
                self.version_manager.edit(index, version)?;
                self.refresh_ui_version_list()?;
                self.update_sender.send(UIUpdate::QuitEditGameDialog)?;
            }
            UICommand::FinishLogin => {
                let mut session = self
                    .account_manager
                    .take_auth_session()
                    .ok_or(LauncherError::AuthSessionNotFound)?;
                while let Ok(action) = session.poll().await {
                    match action {
                        AuthPollAction::Continue(step) => info!("Step {step} / 5"),
                        AuthPollAction::Done(account) => {
                            info!("Step 5 / 5");
                            self.account_manager.add(account)?;
                            self.refresh_ui_acc_list()?;
                            self.update_sender.send(UIUpdate::QuitLoginDialog)?;
                            break;
                        }
                    }
                }
            }
            UICommand::GetAddGameDefault => {
                let config = self.version_manager.get_config();
                self.update_sender
                    .send(UIUpdate::SetAddGameDefault(frontend::game::MCConfig {
                        description: String::new(),
                        game_args: Vec::new(),
                        height: config.height,
                        java_path: config.java_path.clone(),
                        jvm_args: Vec::new(),
                        separated: false,
                        width: config.width,
                        wrapper: config.wrapper.clone(),
                        xms: config.xms.clone(),
                        xmx: config.xmx.clone(),
                    }))?;
            }
            UICommand::GetAddGameList(filter) => {
                let mut list = if let Some(list) = &self.cache.dl_mc_list {
                    list.clone()
                } else {
                    let list = mc::manifest::list_game(self.config.game_path.clone()).await?;
                    self.cache.dl_mc_list = Some(list.clone());
                    list
                };

                if let Some(mc_type) = filter {
                    list = list
                        .into_iter()
                        .filter(|v| frontend_mc_type(v.game_type.clone()) == mc_type)
                        .collect();
                }

                let dl_list = list.into_iter().map(|v| frontend_mc_dl(v)).collect();

                self.update_sender.send(UIUpdate::SetAddGameList(dl_list))?;
            }
            UICommand::GetAddModListFabric(mc_type, index) => {
                let mut list = if let Some(list) = &self.cache.dl_mc_list {
                    list.clone()
                } else {
                    let list = mc::manifest::list_game(self.config.game_path.clone()).await?;
                    self.cache.dl_mc_list = Some(list.clone());
                    list
                };

                if let Some(mc_type) = mc_type {
                    list = list
                        .into_iter()
                        .filter(|v| frontend_mc_type(v.game_type.clone()) == mc_type)
                        .collect();
                }

                let mc = list[index as usize].clone();

                let fabric_list = if let Some(list) = self.cache.dl_fabric_list.take() {
                    list
                } else {
                    let list = mc::manifest::list_fabric(&mc.version).await?;
                    self.cache.dl_fabric_list = Some(list.clone());
                    list
                }
                .into_iter()
                .map(|v| frontend_fabric(v))
                .collect();
                self.update_sender
                    .send(UIUpdate::SetAddModListFabric(fabric_list))?;
            }
            UICommand::GetAddModListForge(mc_type, index) => {
                let mut list = if let Some(list) = &self.cache.dl_mc_list {
                    list.clone()
                } else {
                    let list = mc::manifest::list_game(self.config.game_path.clone()).await?;
                    self.cache.dl_mc_list = Some(list.clone());
                    list
                };

                if let Some(mc_type) = mc_type {
                    list = list
                        .into_iter()
                        .filter(|v| frontend_mc_type(v.game_type.clone()) == mc_type)
                        .collect();
                }

                let mc = list[index as usize].clone();

                let forge_list = if let Some(list) = self.cache.dl_forge_list.take() {
                    list
                } else {
                    let list = mc::manifest::list_forge(&mc.version).await?;
                    self.cache.dl_forge_list = Some(list.clone());
                    list
                }
                .into_iter()
                .map(|v| frontend_forge(v))
                .collect();
                self.update_sender
                    .send(UIUpdate::SetAddModListForge(forge_list))?;
            }
            UICommand::GetEditGameConfig(index) => {
                self.update_sender
                    .send(UIUpdate::SetEditGameConfig(frontend_mc_config(
                        self.version_manager.get(index).clone(),
                    )))?;
            }
            UICommand::GetEditGameVersion(index) => {
                self.update_sender.send(UIUpdate::SetEditGameVersion(
                    self.version_manager.get(index).version.clone(),
                ))?;
            }
            UICommand::SetConfig(config) => {
                self.config = config.general.into();
                self.downloader.set_config(ConfigDL::from(config.dl).into());
                self.version_manager.set_config(config.mc.into());
                self.save_config()?;
            }
            UICommand::GetOfflineAccount => {
                self.update_sender
                    .send(UIUpdate::SetOfflineAccount(frontend_account(
                        Account::default(),
                    )))?;
            }
            UICommand::RequestLogin => {
                let (uri, code) = self.account_manager.request_login().await?;
                let mut ctx: ClipboardContext = ClipboardProvider::new()?;
                ctx.set_contents(code)?;
                webbrowser::open(&uri)?;
            }
            UICommand::Start(acc_index, ver_index) => {
                if acc_index >= self.account_manager.get_account_list().len() as u32
                    || ver_index >= self.version_manager.get_version_list().len() as u32
                {
                    return Err(LauncherError::OutOfRange);
                }

                self.update_sender.send(UIUpdate::SetHomePageStatus(
                    frontend::home::State::LoggingIn,
                ))?;

                if self
                    .account_manager
                    .request_refresh_account(acc_index)
                    .await?
                {
                    self.update_sender
                        .send(UIUpdate::SetHomePageProgress(1 as u32, 5))?;
                    let mut session = self
                        .account_manager
                        .take_auth_session()
                        .ok_or(LauncherError::AuthSessionNotFound)?;

                    let mut action = session.poll().await?;
                    loop {
                        match action {
                            AuthPollAction::Continue(s) => {
                                self.update_sender
                                    .send(UIUpdate::SetHomePageProgress(s as u32, 5))?;
                            }
                            AuthPollAction::Done(account) => {
                                self.update_sender
                                    .send(UIUpdate::SetHomePageProgress(5, 5))?;
                                self.account_manager.edit(acc_index, account)?;
                                break;
                            }
                        }
                        action = session.poll().await?;
                    }
                }

                self.update_sender.send(UIUpdate::SetHomePageStatus(
                    frontend::home::State::Launching,
                ))?;
                tokio::task::yield_now().await;

                let account = self.account_manager.get(acc_index);
                let version = self.version_manager.get(ver_index);
                let (cmd_list, dl_list) =
                    mc::launch::get_launch_command(account, version, &self.config.game_path)
                        .await?;

                if dl_list.len() != 0 {
                    self.update_sender.send(UIUpdate::SetHomePageStatus(
                        frontend::home::State::Downloading,
                    ))?;
                    tokio::task::yield_now().await;
                    let tasks: Vec<downloader::task::TaskInfo> = dl_list
                        .into_iter()
                        .map(|i| {
                            downloader::task::TaskInfo::new(
                                i.url,
                                i.save_path,
                                None,
                                i.on_finish,
                                None,
                                None,
                            )
                        })
                        .collect();

                    let id = version.version.clone();
                    self.downloader.add_taskset(id.clone(), tasks);
                    self.downloader.start_taskset(id.clone())?;

                    // progress by bytes may update total bytes, which looks strange
                    // TODO: make it a broadcast in downloader
                    let mut status = self.downloader.get_status_by_number(id.clone())?;
                    loop {
                        match status {
                            downloader::taskset::TaskSetStatus::Completed(total) => {
                                self.update_sender.send(UIUpdate::SetHomePageProgress(
                                    total as u32,
                                    total as u32,
                                ))?;
                                tokio::task::yield_now().await;
                                break;
                            }
                            downloader::taskset::TaskSetStatus::Failed => {
                                error!("Failed to download {0}.", &id);
                                return Err(LauncherError::DownloadFailed(id.clone()));
                            }
                            downloader::taskset::TaskSetStatus::Cancelled => {
                                return Err(LauncherError::Interrupted);
                            }
                            downloader::taskset::TaskSetStatus::Downloading(downloaded, total) => {
                                self.update_sender.send(UIUpdate::SetHomePageProgress(
                                    downloaded as u32,
                                    total as u32,
                                ))?;
                                tokio::task::yield_now().await;
                            }
                            downloader::taskset::TaskSetStatus::Paused(downloaded, total) => {
                                // This case shouldn't happen now. Pause hasn't been implemented
                                self.update_sender.send(UIUpdate::SetHomePageProgress(
                                    downloaded as u32,
                                    total as u32,
                                ))?;
                                tokio::task::yield_now().await;
                            }
                            downloader::taskset::TaskSetStatus::Pending(total) => {
                                // TODO: download this game first
                                self.update_sender
                                    .send(UIUpdate::SetHomePageProgress(0, total as u32))?;
                                tokio::task::yield_now().await;
                            }
                        }
                        drop(status);
                        sleep(Duration::from_millis(500)).await;
                        status = self.downloader.get_status_by_number(id.clone())?;
                    }
                }

                self.update_sender.send(UIUpdate::SetHomePageStatus(
                    frontend::home::State::Launching,
                ))?;
                tokio::task::yield_now().await;
                self.update_sender
                    .send(UIUpdate::SetHomePageProgress(1, 2))?;
                tokio::task::yield_now().await;

                let (s, r) = std::sync::mpsc::channel();
                let mut cmd = Command::new(if version.wrapper.is_empty() {
                    version.java_path.clone()
                } else {
                    version.wrapper.clone()
                });
                if !version.wrapper.is_empty() {
                    cmd.arg(version.java_path.clone());
                }
                cmd.args(cmd_list);

                std::thread::spawn(move || s.send(cmd.spawn()));

                match r.recv().unwrap() {
                    Ok(_) => {
                        self.update_sender
                            .send(UIUpdate::SetHomePageProgress(2, 2))?;
                        tokio::task::yield_now().await;
                        if self.config.close_after_launch {
                            self.update_sender.send(UIUpdate::Quit)?;
                        }
                    }
                    Err(e) => return Err(e.into()),
                }

                self.update_sender
                    .send(UIUpdate::SetHomePageStatus(frontend::home::State::Spare))?;

                self.update_sender
                    .send(UIUpdate::SetHomePageProgress(0, 0))?;
            }
            UICommand::SwitchAccount(index) => {
                self.account_manager.set_current_index(index)?;
            }
            UICommand::SwitchGame(index) => {
                self.version_manager.set_current_index(index)?;
            }
        }

        Ok(())
    }

    fn refresh_ui_acc_list(&self) -> Result<(), LauncherError> {
        let acc_index = self.account_manager.get_current_index();
        let acc_list = self.account_manager.get_account_list();
        self.update_sender.send(UIUpdate::SetAccountList(
            acc_list
                .iter()
                .map(|v| frontend_account(v.clone()))
                .collect(),
        ))?;
        self.update_sender
            .send(UIUpdate::SetAccountIndex(acc_index))?;
        Ok(())
    }

    fn refresh_ui_config(&self) -> Result<(), LauncherError> {
        let config_general = &self.config;
        let config_dl: ConfigDL = self.downloader.get_config().clone().into();
        let config_mc = self.version_manager.get_config();

        self.update_sender
            .send(UIUpdate::SetConfig(frontend::Config {
                dl: config_dl.into(),
                general: config_general.clone().into(),
                mc: config_mc.clone().into(),
            }))?;

        Ok(())
    }

    fn refresh_ui_info(&self) -> Result<(), LauncherError> {
        let authors = env!("CARGO_PKG_AUTHORS");
        let version = env!("CARGO_PKG_VERSION");

        self.update_sender
            .send(UIUpdate::SetAuthors(authors.into()))?;
        self.update_sender
            .send(UIUpdate::SetVersion(version.into()))?;

        Ok(())
    }

    fn refresh_ui_version_list(&self) -> Result<(), LauncherError> {
        let version_index = self.version_manager.get_current_index();
        let version_list = self.version_manager.get_version_list();
        self.update_sender.send(UIUpdate::SetGameList(
            version_list
                .iter()
                .map(|v| frontend_mc_info(v.clone()))
                .collect(),
        ))?;
        self.update_sender
            .send(UIUpdate::SetGameIndex(version_index))?;
        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), LauncherError> {
        self.init()?;

        let mut cmd_receiver = self
            .cmd_receiver
            .take()
            .ok_or(LauncherError::ChannelNotFound)?;
        loop {
            tokio::select! {
                Some(cmd) = cmd_receiver.recv() => {
                    if let Err(e) = self.handle(cmd).await {
                        error!("{e}");
                        self.update_sender.send(UIUpdate::SetHomePageStatus(frontend::home::State::Spare))?;
                        self.update_sender.send(UIUpdate::SetHomePageProgress(0, 0))?;
                    }
                },
            }
        }
    }

    fn i_load_config() -> Result<(ConfigDL, ConfigGeneral, ConfigMC), LauncherError> {
        if fs::exists(&"config.json")? {
            let mut config_dl = ConfigDL::default();
            let mut config_general = ConfigGeneral::default();
            let mut config_mc = ConfigMC::default();
            let json: serde_json::Value =
                serde_json::from_str(&fs::read_to_string("config.json")?.as_str())?;

            config_dl.assets_source = String::from(
                json["assets_source"]
                    .as_str()
                    .ok_or(LauncherError::LauncherConfigError)?,
            );
            config_general.close_after_launch = json["close_after_launch"]
                .as_bool()
                .ok_or(LauncherError::LauncherConfigError)?;
            config_dl.concurrency = json["concurrency"]
                .as_u64()
                .ok_or(LauncherError::LauncherConfigError)?
                as u32;
            config_dl.fabric_source = String::from(
                json["fabric_source"]
                    .as_str()
                    .ok_or(LauncherError::LauncherConfigError)?,
            );
            config_dl.forge_source = String::from(
                json["forge_source"]
                    .as_str()
                    .ok_or(LauncherError::LauncherConfigError)?,
            );
            config_general.game_path = String::from(
                json["game_path"]
                    .as_str()
                    .ok_or(LauncherError::LauncherConfigError)?,
            );
            config_dl.game_source = String::from(
                json["game_source"]
                    .as_str()
                    .ok_or(LauncherError::LauncherConfigError)?,
            );
            config_mc.height = json["height"]
                .as_u64()
                .ok_or(LauncherError::LauncherConfigError)? as u32;
            config_mc.java_path = String::from(
                json["java_path"]
                    .as_str()
                    .ok_or(LauncherError::LauncherConfigError)?,
            );
            config_dl.libraries_source = String::from(
                json["libraries_source"]
                    .as_str()
                    .ok_or(LauncherError::LauncherConfigError)?,
            );
            config_mc.width = json["width"]
                .as_u64()
                .ok_or(LauncherError::LauncherConfigError)? as u32;
            config_mc.wrapper = String::from(
                json["wrapper"]
                    .as_str()
                    .ok_or(LauncherError::LauncherConfigError)?,
            );
            config_mc.xms = String::from(
                json["xms"]
                    .as_str()
                    .ok_or(LauncherError::LauncherConfigError)?,
            );
            config_mc.xmx = String::from(
                json["xmx"]
                    .as_str()
                    .ok_or(LauncherError::LauncherConfigError)?,
            );

            Ok((config_dl, config_general, config_mc))
        } else {
            Self::i_save_config(
                ConfigGeneral::default(),
                ConfigDL::default(),
                ConfigMC::default(),
            )?;
            Ok((
                ConfigDL::default(),
                ConfigGeneral::default(),
                ConfigMC::default(),
            ))
        }
    }

    fn i_save_config(
        config: ConfigGeneral,
        config_dl: ConfigDL,
        config_mc: ConfigMC,
    ) -> Result<(), LauncherError> {
        let json = json!(
            {
                "assets_source": config_dl.assets_source,
                "close_after_launch": config.close_after_launch,
                "concurrency": config_dl.concurrency,
                "fabric_source": config_dl.fabric_source,
                "forge_source": config_dl.forge_source,
                "game_path": config.game_path,
                "game_source": config_dl.game_source,
                "height": config_mc.height,
                "java_path": config_mc.java_path,
                "libraries_source": config_dl.libraries_source,
                "width": config_mc.width,
                "wrapper": config_mc.wrapper,
                "xms": config_mc.xms,
                "xmx": config_mc.xmx,
            }
        );
        fs::write("config.json", json.to_string())?;
        Ok(())
    }

    pub fn save_config(&self) -> Result<(), LauncherError> {
        Self::i_save_config(
            self.config.clone(),
            self.downloader.get_config().clone().into(),
            self.version_manager.get_config().clone(),
        )
    }
}
