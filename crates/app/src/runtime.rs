//! CEMCL Application Runtime

use clipboard::{ClipboardContext, ClipboardProvider};
use log::{error, info};
use mc::{
    MCInstallation,
    account::{Account, auth::AuthPollAction},
    manifest::{
        Fabric, Forge, MCDL, download_fabric, download_forge, download_mc,
    },
};
use serde_json::json;
use utils::get_parent_dir;
use std::{collections::HashMap, fs::{self, create_dir_all, exists, remove_dir_all}, process::Command};

use crate::{
    account::{frontend_account, to_account_type},
    java::JavaManager,
    version::{
        ConfigMC, VersionManager, frontend_fabric, frontend_forge, frontend_mc_config,
        frontend_mc_dl, frontend_mc_info, frontend_mc_type,
    },
};
use downloader::{Config as DownloaderConfig, DownloadManager, task::TaskInfo};
use frontend::{
    UICommand,
    UIUpdate,
    game::ModType,
    java,
};

use crate::{account::AccountManager, errors::LauncherError};

#[derive(Clone)]
pub struct ConfigGeneral {
    /// 启动后关闭启动器
    pub close_after_launch: bool,
    /// .minecraft路径
    pub game_path: String,
    /// 进度显示方式
    pub progress_mode: frontend::ProgressMode,
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
            progress_mode: value.progress_mode,
        }
    }
}

impl From<frontend::ConfigMC> for ConfigMC {
    fn from(value: frontend::ConfigMC) -> Self {
        Self {
            height: value.height,
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
            progress_mode: value.progress_mode,
        }
    }
}

impl From<ConfigMC> for frontend::ConfigMC {
    fn from(value: ConfigMC) -> Self {
        Self {
            height: value.height,
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
            progress_mode: frontend::ProgressMode::BySize,
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

/// 等待 Forge 安装完成的待添加版本
struct PendingForge {
    /// 下载任务集 id
    id: String,
    /// Forge 安装成功后要加入版本列表的安装信息
    installation: MCInstallation,
}

pub struct AppRuntime {
    account_manager: AccountManager,
    cache: CacheData,
    config: ConfigGeneral,
    cmd_receiver: Option<tokio::sync::mpsc::UnboundedReceiver<UICommand>>,
    downloader: DownloadManager,
    downloader_broadcast: tokio::sync::broadcast::Receiver<downloader::TaskSetStatusInfo>,
    /// Latest status of each task set, updated by the broadcast
    task_set_status: HashMap<String, frontend::downloader::TaskSetInfo>,
    java_manager: JavaManager,
    /// 等待 Forge 安装完成的待添加版本
    pending_forge: Option<PendingForge>,
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
        let downloader_broadcast = downloader.subscribe();
        let mut java_manager = JavaManager::new()?;
        java_manager.auto_detect_system_java();
        let mut version_manager = VersionManager::new(config_mc.clone())?;
        version_manager.resolve_java_indices(&java_manager);

        Ok(Self {
            account_manager,
            cache: CacheData::new(),
            config: config_general,
            cmd_receiver: Some(cmd_receiver),
            downloader,
            downloader_broadcast,
            task_set_status: HashMap::new(),
            java_manager,
            pending_forge: None,
            update_sender,
            version_manager,
        })
    }

    fn init(&self) -> Result<(), LauncherError> {
        self.refresh_ui_info()?;
        self.refresh_ui_acc_list()?;
        self.refresh_ui_config()?;
        self.refresh_ui_java_list()?;
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

                // 判断原版是否已下载，未下载则先下载
                let mc_json_path = format!(
                    "{}/versions/{}/{}.json",
                    self.config.game_path, version.version, version.version
                );
                if !exists(&mc_json_path)? {
                    download_mc(&self.config.game_path, version.clone()).await?;
                }

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

                            // Fabric 无独立安装环节，直接添加
                            let installation = MCInstallation {
                                description: config.description,
                                game_args: config.game_args,
                                game_type: ver_type,
                                height: config.height,
                                java_index: if config.java_index >= 0 {
                                    Some(config.java_index as u32)
                                } else {
                                    None
                                },
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
                        ModType::Forge => {
                            let mod_list = if let Some(list) = self.cache.dl_forge_list.take() {
                                list
                            } else {
                                mc::manifest::list_forge(&version.version).await?
                            };

                            let forge = &mod_list[mod_index as usize];

                            let task_info =
                                download_forge(&version.version, forge.clone(), "{forge_source}");

                            let dir = get_parent_dir(&task_info.save_path);
                            if !exists(&dir)? {
                                create_dir_all(&dir)?;
                            }

                            let java_path = if config.java_index >= 0 {
                                self.java_manager.get_java_path_by_index(config.java_index as u32)?
                            } else {
                                "java".to_string()
                            };
                            let forge_path = task_info.save_path.clone();
                            let f = move || {
                                match Command::new(&java_path)
                                    .arg("-jar")
                                    .arg(&forge_path)
                                    .spawn()
                                {
                                    Ok(mut child) => {
                                        if let Err(e) = child.wait() {
                                            error!("{e}");
                                            if exists(&dir).unwrap() {
                                                remove_dir_all(&dir).unwrap();
                                            }
                                        }
                                    }
                                    Err(e) => error!("Failed to run forge installer. Reason: {e}."),
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

                            // 记录待添加的安装信息，Forge 安装成功后再加入版本列表
                            self.pending_forge = Some(PendingForge {
                                id,
                                installation: MCInstallation {
                                    description: config.description,
                                    game_args: config.game_args,
                                    game_type: ver_type,
                                    height: config.height,
                                    java_index: if config.java_index >= 0 {
                                        Some(config.java_index as u32)
                                    } else {
                                        None
                                    },
                                    jvm_args: config.jvm_args,
                                    separated: config.separated,
                                    version: format!(
                                        "{mc_version}-forge-{forge_version}",
                                        mc_version = version.version,
                                        forge_version = forge.version,
                                    ),
                                    width: config.width,
                                    wrapper: config.wrapper,
                                    xms: config.xms,
                                    xmx: config.xmx,
                                },
                            });

                            // 显示 Forge 下载弹窗
                            self.update_sender.send(UIUpdate::ShowForgeDownloadDialog(
                                format!(
                                    "Downloading Forge {forge_version} for Minecraft {mc_version}",
                                    forge_version = forge.version,
                                    mc_version = version.version,
                                ),
                            ))?;
                            // 关闭添加游戏弹窗
                            self.update_sender.send(UIUpdate::QuitAddGameDialog)?;
                        }
                    }
                } else {
                    // 无 mod loader，直接添加
                    let installation = MCInstallation {
                        description: config.description,
                        game_args: config.game_args,
                        game_type: ver_type,
                        height: config.height,
                        java_index: if config.java_index >= 0 {
                            Some(config.java_index as u32)
                        } else {
                            None
                        },
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
            }
            UICommand::AddJava(java_path) => {
                self.java_manager.add(java_path)?;
                self.refresh_ui_java_list()?;
                self.refresh_ui_config()?;
                self.update_sender.send(UIUpdate::QuitAddJavaDialog)?;
            }
            UICommand::CheckJava(java_path) => {
                use ::java::java::JavaInstallationError;
                let (result, version) = match ::java::java::JavaInstallation::new(java_path) {
                    Ok(installation) => {
                        (frontend::ui::JavaCheckResult::Detected, installation.get_version().to_string())
                    }
                    Err(e) => {
                        let result = match e {
                            JavaInstallationError::PathIsEmpty => frontend::ui::JavaCheckResult::PathIsEmpty,
                            JavaInstallationError::JavaExecutableNotFound => frontend::ui::JavaCheckResult::JavaExecutableNotFound,
                            JavaInstallationError::ReleaseFileInvalid => frontend::ui::JavaCheckResult::ReleaseFileInvalid,
                            JavaInstallationError::IOError(_) => frontend::ui::JavaCheckResult::IOError,
                        };
                        (result, String::new())
                    }
                };
                self.update_sender
                    .send(UIUpdate::SetJavaCheckResult(result, version))?;
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
            UICommand::DelJava(index) => {
                self.java_manager.del(index)?;
                self.refresh_ui_java_list()?;
                self.refresh_ui_config()?;
            }
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
                version.java_index = if installation.java_index >= 0 { Some(installation.java_index as u32) } else { None };
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
                let java_list = self.build_java_info_list(None);
                let java_model: Vec<String> = java::ui_java_combo_box_list(&java_list);
                let java_index = self.java_manager.get_default().map(|i| i as i32).unwrap_or(-1);
                self.update_sender
                    .send(UIUpdate::SetAddGameDefault(frontend::game::MCConfig {
                        description: String::new(),
                        game_args: Vec::new(),
                        height: config.height,
                        java_index,
                        jvm_args: Vec::new(),
                        separated: false,
                        width: config.width,
                        wrapper: config.wrapper.clone(),
                        xms: config.xms.clone(),
                        xmx: config.xmx.clone(),
                    }))?;
                self.update_sender
                    .send(UIUpdate::SetAddGameJavaList(java_model))?;
                // Also send the Java list for the Java page table
                self.update_sender.send(UIUpdate::SetJavaList(java_list))?;
            }
            UICommand::GetAddGameJavaList(mc_type, mc_index) => {
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

                if (mc_index as usize) < list.len() {
                    let mc = &list[mc_index as usize];
                    let java_list = self.build_java_info_list(Some(&mc.version));
                    let java_model: Vec<String> = java::ui_java_combo_box_list(&java_list);
                    self.update_sender
                        .send(UIUpdate::SetAddGameJavaList(java_model))?;
                }
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
                let version = self.version_manager.get(index).clone();
                let java_list = self.build_java_info_list(Some(&version.version));
                let java_model: Vec<String> = java::ui_java_combo_box_list(&java_list);
                self.update_sender
                    .send(UIUpdate::SetEditGameConfig(frontend::game::MCConfig {
                        java_index: version.java_index.map(|i| i as i32).unwrap_or(-1),
                        ..frontend_mc_config(version.clone())
                    }))?;
                self.update_sender
                    .send(UIUpdate::SetEditGameJavaList(java_model))?;
                self.update_sender.send(UIUpdate::SetJavaList(java_list))?;
            }
            UICommand::GetEditGameVersion(index) => {
                self.update_sender.send(UIUpdate::SetEditGameVersion(
                    self.version_manager.get(index).version.clone(),
                ))?;
            }
            UICommand::GetJavaList => {
                self.refresh_ui_java_list()?;
            }
            UICommand::SetConfig(config) => {
                self.config = config.general.into();
                self.downloader.set_config(ConfigDL::from(config.dl).into());
                self.version_manager.set_config(config.mc.into());
                self.save_config()?;
            }
            UICommand::SetDefaultJava(index) => {
                let idx = if index >= 0 { Some(index as u32) } else { None };
                self.java_manager.set_default(idx)?;
            }
            UICommand::GetOfflineAccount => {
                self.update_sender
                    .send(UIUpdate::SetOfflineAccount(frontend_account(
                        Account::default(),
                    )))?;
            }
            UICommand::HideForgeDownloadDialog => {
                self.update_sender.send(UIUpdate::QuitForgeDownloadDialog)?;
            }
            UICommand::CancelForgeDownload => {
                if let Some(pending) = self.pending_forge.take() {
                    self.downloader.cancel_taskset(pending.id);
                }
                self.update_sender.send(UIUpdate::QuitForgeDownloadDialog)?;
            }
            UICommand::PauseTaskSet(id) => {
                self.downloader.pause_taskset(id);
            }
            UICommand::ResumeTaskSet(id) => {
                self.downloader.resume_taskset(id);
            }
            UICommand::CancelTaskSet(id) => {
                self.downloader.cancel_taskset(id);
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
                        .send(UIUpdate::SetHomePageProgress(1.0 / 5.0, 1, 5))?;
                    let mut session = self
                        .account_manager
                        .take_auth_session()
                        .ok_or(LauncherError::AuthSessionNotFound)?;

                    let mut action = session.poll().await?;
                    loop {
                        match action {
                            AuthPollAction::Continue(s) => {
                                self.update_sender
                                    .send(UIUpdate::SetHomePageProgress(
                                        s as f32 / 5.0,
                                        s as u32,
                                        5,
                                    ))?;
                            }
                            AuthPollAction::Done(account) => {
                                self.update_sender
                                    .send(UIUpdate::SetHomePageProgress(1.0, 5, 5))?;
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
                let version = self.version_manager.get(ver_index).clone();
                let (cmd_list, dl_list) =
                    mc::launch::get_launch_command(account, &version, &self.config.game_path)
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

                    // Wait for the download to complete via broadcast.
                    // The progress display depends on the configured progress mode.
                    loop {
                        let info = match self.downloader_broadcast.recv().await {
                            Ok(info) => info,
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                                continue;
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                                return Err(LauncherError::ChannelClosed);
                            }
                        };
                        self.handle_broadcast(info.clone())?;
                        if info.id != id {
                            continue;
                        }
                        match info.status_by_number {
                            downloader::taskset::TaskSetStatus::Completed(total) => {
                                self.send_home_progress(info.progress, (total, total))?;
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
                                self.send_home_progress(info.progress, (downloaded, total))?;
                                tokio::task::yield_now().await;
                            }
                            downloader::taskset::TaskSetStatus::Paused(downloaded, total) => {
                                self.send_home_progress(info.progress, (downloaded, total))?;
                                tokio::task::yield_now().await;
                            }
                            downloader::taskset::TaskSetStatus::Pending(total) => {
                                self.send_home_progress(info.progress, (0, total))?;
                                tokio::task::yield_now().await;
                            }
                        }
                    }
                }

                self.update_sender.send(UIUpdate::SetHomePageStatus(
                    frontend::home::State::Launching,
                ))?;
                tokio::task::yield_now().await;
                self.update_sender
                    .send(UIUpdate::SetHomePageProgress(0.5, 1, 2))?;
                tokio::task::yield_now().await;

                let (s, r) = std::sync::mpsc::channel();
                // 如果未选择 Java，直接使用系统 PATH 中的 java
                let java_path = match version.java_index {
                    Some(idx) => self.java_manager.get_java_path_by_index(idx)?,
                    None => "java".to_string(),
                };
                let mut cmd = Command::new(if version.wrapper.is_empty() {
                    java_path.clone()
                } else {
                    version.wrapper.clone()
                });
                if !version.wrapper.is_empty() {
                    cmd.arg(java_path);
                }
                cmd.args(cmd_list);

                std::thread::spawn(move || s.send(cmd.spawn()));

                match r.recv().unwrap() {
                    Ok(_) => {
                        self.update_sender
                            .send(UIUpdate::SetHomePageProgress(1.0, 2, 2))?;
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
                    .send(UIUpdate::SetHomePageProgress(0.0, 0, 0))?;
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

    /// 根据配置的进度显示方式，向主页发送进度更新。
    /// `size_progress` 为按字节计算的 (downloaded, total)，
    /// `number_progress` 为按任务数量计算的 (completed, total)。
    fn send_home_progress(
        &self,
        size_progress: (u64, u64),
        number_progress: (u64, u64),
    ) -> Result<(), LauncherError> {
        let (size_downloaded, size_total) = size_progress;
        let (number_done, number_total) = number_progress;
        match self.config.progress_mode {
            frontend::ProgressMode::BySize => {
                let progress = if size_total != 0 {
                    size_downloaded as f32 / size_total as f32
                } else {
                    0.0
                };
                self.update_sender.send(UIUpdate::SetHomePageProgress(
                    progress,
                    size_downloaded as u32,
                    size_total as u32,
                ))?;
            }
            frontend::ProgressMode::ByNumber => {
                let progress = if number_total != 0 {
                    number_done as f32 / number_total as f32
                } else {
                    0.0
                };
                self.update_sender.send(UIUpdate::SetHomePageProgress(
                    progress,
                    number_done as u32,
                    number_total as u32,
                ))?;
            }
            frontend::ProgressMode::Both => {
                let progress = if size_total != 0 {
                    size_downloaded as f32 / size_total as f32
                } else {
                    0.0
                };
                self.update_sender.send(UIUpdate::SetHomePageProgress(
                    progress,
                    number_done as u32,
                    number_total as u32,
                ))?;
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
        let java_list = self.build_java_info_list(None);
        let java_model: Vec<String> = java::ui_java_combo_box_list(&java_list);
        let java_index = self.java_manager.get_default().map(|i| i as i32).unwrap_or(-1);

        self.update_sender
            .send(UIUpdate::SetConfig(frontend::Config {
                dl: config_dl.into(),
                general: config_general.clone().into(),
                mc: config_mc.clone().into(),
            }))?;
        self.update_sender
            .send(UIUpdate::SetJavaModel(java_model))?;
        self.update_sender
            .send(UIUpdate::SetJavaIndex(java_index))?;

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

    fn refresh_ui_java_list(&self) -> Result<(), LauncherError> {
        let java_list = self.build_java_info_list(None);
        self.update_sender.send(UIUpdate::SetJavaList(java_list))?;
        Ok(())
    }

    fn build_java_info_list(&self, mc_version: Option<&str>) -> Vec<frontend::JavaInfo> {
        let min_java = mc_version.and_then(|v| min_java_version_for_mc(v, &self.config.game_path));
        self.java_manager
            .get_java_list()
            .iter()
            .map(|j| {
                let compatible = match &min_java {
                    Some(min) => j.get_version() >= min,
                    None => true,
                };
                frontend::JavaInfo {
                    version: j.get_version().to_string(),
                    path: j.get_path().to_string(),
                    compatible,
                }
            })
            .collect()
    }
}

/// Determine the minimum Java version required for a given Minecraft version.
/// Reads the MC version JSON file to get the `javaVersion.majorVersion` field.
/// Falls back to hardcoded rules if the JSON cannot be read.
fn min_java_version_for_mc(mc_version: &str, game_path: &str) -> Option<::java::java_version::JavaVersion> {
    // Extract the base MC version from modded version strings like "fabric-loader-0.16.10-1.21.5"
    let base_version = extract_mc_version(mc_version);

    // Try to read from the MC version JSON file
    let json_path = format!("{}/versions/{}/{}.json", game_path, base_version, base_version);
    if let Ok(content) = std::fs::read_to_string(&json_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(major) = json["javaVersion"]["majorVersion"].as_u64() {
                return Some(::java::java_version::JavaVersion::from(major.to_string().as_str()));
            }
        }
    }

    // Fallback: hardcoded rules based on MC version
    let parts: Vec<&str> = base_version.split(".").collect();
    if parts.len() < 2 {
        return None;
    }
    let major: u32 = parts[0].parse().ok()?;
    let minor: u32 = parts[1].parse().ok()?;
    let patch: u32 = parts.get(2).and_then(|p| p.parse().ok()).unwrap_or(0);

    if major > 1 || (major == 1 && minor >= 21) || (major == 1 && minor == 20 && patch >= 5) {
        Some(::java::java_version::JavaVersion::from("21"))
    } else if major == 1 && minor >= 18 {
        Some(::java::java_version::JavaVersion::from("17"))
    } else if major == 1 && minor == 17 {
        Some(::java::java_version::JavaVersion::from("16"))
    } else {
        Some(::java::java_version::JavaVersion::from("1.8"))
    }
}

/// Extract the base Minecraft version from a modded version string.
/// e.g. "fabric-loader-0.16.10-1.21.5" -> "1.21.5"
///      "1.21.5-forge-52.0.0" -> "1.21.5"
///      "1.21.5" -> "1.21.5"
fn extract_mc_version(version: &str) -> &str {
    // For fabric: "fabric-loader-X.Y.Z-MC_VERSION"
    if let Some(rest) = version.strip_prefix("fabric-loader-") {
        // Find the last '-' and take everything after it
        if let Some(pos) = rest.rfind('-') {
            return &rest[pos + 1..];
        }
        return rest;
    }
    // For forge: "MC_VERSION-forge-X.Y.Z"
    if let Some(pos) = version.find("-forge-") {
        return &version[..pos];
    }
    version
}

impl AppRuntime {
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
                        self.update_sender.send(UIUpdate::SetHomePageProgress(0.0, 0, 0))?;
                    }
                },
                info = self.downloader_broadcast.recv() => {
                    match info {
                        Ok(info) => self.handle_broadcast(info)?,
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            return Err(LauncherError::ChannelClosed);
                        }
                    }
                },
            }
        }
    }

    /// Handle a broadcast message from the downloader
    fn handle_broadcast(
        &mut self,
        info: downloader::TaskSetStatusInfo,
    ) -> Result<(), LauncherError> {
        // 处理 Forge 安装任务集
        if let Some(pending) = &self.pending_forge {
            if pending.id == info.id {
                match info.status.clone() {
                    downloader::taskset::TaskSetStatus::Downloading(downloaded, total) => {
                        let progress = if total != 0 {
                            downloaded as f32 / total as f32
                        } else {
                            0.0
                        };
                        self.update_sender
                            .send(UIUpdate::SetForgeDownloadProgress(progress))?;
                        // 下载完成，正在运行 Forge 安装程序
                        if total != 0 && downloaded >= total {
                            self.update_sender
                                .send(UIUpdate::SetForgeDownloadInstalling(true))?;
                        }
                    }
                    downloader::taskset::TaskSetStatus::Pending(_) => {
                        self.update_sender
                            .send(UIUpdate::SetForgeDownloadProgress(0.0))?;
                    }
                    downloader::taskset::TaskSetStatus::Completed(_) => {
                        // Forge 安装成功，将版本加入列表
                        let installation = pending.installation.clone();
                        self.pending_forge = None;
                        self.version_manager.add(&installation)?;
                        self.refresh_ui_version_list()?;
                        self.update_sender.send(UIUpdate::QuitForgeDownloadDialog)?;
                        self.update_sender.send(UIUpdate::QuitAddGameDialog)?;
                    }
                    downloader::taskset::TaskSetStatus::Failed => {
                        self.pending_forge = None;
                        self.update_sender.send(UIUpdate::QuitForgeDownloadDialog)?;
                    }
                    downloader::taskset::TaskSetStatus::Cancelled => {
                        self.pending_forge = None;
                        self.update_sender.send(UIUpdate::QuitForgeDownloadDialog)?;
                    }
                    downloader::taskset::TaskSetStatus::Paused(downloaded, total) => {
                        let progress = if total != 0 {
                            downloaded as f32 / total as f32
                        } else {
                            0.0
                        };
                        self.update_sender
                            .send(UIUpdate::SetForgeDownloadProgress(progress))?;
                    }
                }
            }
        }

        // 从 status_by_number 提取按数量的进度
        let progress_by_number = match &info.status_by_number {
            downloader::taskset::TaskSetStatus::Pending(total) => (0, *total),
            downloader::taskset::TaskSetStatus::Downloading(downloaded, total) => {
                (*downloaded, *total)
            }
            downloader::taskset::TaskSetStatus::Paused(downloaded, total) => {
                (*downloaded, *total)
            }
            downloader::taskset::TaskSetStatus::Completed(total) => (*total, *total),
            downloader::taskset::TaskSetStatus::Cancelled => (0, 0),
            downloader::taskset::TaskSetStatus::Failed => (0, 0),
        };

        self.task_set_status.insert(
            info.id.clone(),
            frontend::downloader::TaskSetInfo {
                id: info.id,
                status: match info.status {
                    downloader::taskset::TaskSetStatus::Pending(_) => {
                        frontend::downloader::TaskSetStatus::Pending
                    }
                    downloader::taskset::TaskSetStatus::Downloading(_, _) => {
                        frontend::downloader::TaskSetStatus::Downloading
                    }
                    downloader::taskset::TaskSetStatus::Paused(_, _) => {
                        frontend::downloader::TaskSetStatus::Paused
                    }
                    downloader::taskset::TaskSetStatus::Completed(_) => {
                        frontend::downloader::TaskSetStatus::Completed
                    }
                    downloader::taskset::TaskSetStatus::Cancelled => {
                        frontend::downloader::TaskSetStatus::Cancelled
                    }
                    downloader::taskset::TaskSetStatus::Failed => {
                        frontend::downloader::TaskSetStatus::Failed
                    }
                },
                progress: info.progress,
                progress_by_number,
            },
        );
        let list: Vec<frontend::downloader::TaskSetInfo> =
            self.task_set_status.values().cloned().collect();
        self.update_sender.send(UIUpdate::SetTaskSetList(list))?;
        Ok(())
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
            config_general.progress_mode = match json["progress_mode"].as_str() {
                Some("ByNumber") => frontend::ProgressMode::ByNumber,
                Some("Both") => frontend::ProgressMode::Both,
                _ => frontend::ProgressMode::BySize,
            };
            config_dl.game_source = String::from(
                json["game_source"]
                    .as_str()
                    .ok_or(LauncherError::LauncherConfigError)?,
            );
            config_mc.height = json["height"]
                .as_u64()
                .ok_or(LauncherError::LauncherConfigError)? as u32;
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
                "libraries_source": config_dl.libraries_source,
                "progress_mode": match config.progress_mode {
                    frontend::ProgressMode::BySize => "BySize",
                    frontend::ProgressMode::ByNumber => "ByNumber",
                    frontend::ProgressMode::Both => "Both",
                },
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
