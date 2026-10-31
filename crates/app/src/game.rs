//! 游戏版本管理：版本列表、添加游戏（含 Fabric/Forge 安装）与相关界面状态

use downloader::{TaskSetStatusInfo, taskset::TaskSetStatus};
use frontend::{UIUpdate, game::ModType};
use log::error;
use mc::{
    MCInstallation,
    manifest::{Fabric, Forge, MCDL, download_fabric, download_forge, download_mc},
};
use std::{
    fs::{create_dir_all, exists, remove_dir_all},
    process::Command,
};
use utils::get_parent_dir;

use crate::{
    LauncherError,
    config::ConfigManager,
    java::JavaManager,
    tasks::TaskManager,
    ui::UiSender,
    version::{
        ConfigMC, VersionManager, frontend_fabric, frontend_forge, frontend_mc_config,
        frontend_mc_dl, frontend_mc_info, frontend_mc_type,
    },
};

struct CacheData {
    dl_mc_list: Option<Vec<MCDL>>,
    /// MC 版本 -> Fabric 列表
    dl_fabric_list: Option<(String, Vec<Fabric>)>,
    /// MC 版本 -> Forge 列表
    dl_forge_list: Option<(String, Vec<Forge>)>,
}

impl CacheData {
    fn new() -> Self {
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

/// 由界面配置构造要加入版本列表的安装项
fn i_installation(
    config: &frontend::game::MCConfig,
    version: String,
    game_type: mc::MCType,
) -> MCInstallation {
    MCInstallation {
        description: config.description.clone(),
        game_args: config.game_args.clone(),
        game_type,
        height: config.height,
        java_index: if config.java_index >= 0 {
            Some(config.java_index as u32)
        } else {
            None
        },
        jvm_args: config.jvm_args.clone(),
        separated: config.separated,
        version,
        width: config.width,
        wrapper: config.wrapper.clone(),
        xms: config.xms.clone(),
        xmx: config.xmx.clone(),
    }
}

pub struct GameManager {
    cache: CacheData,
    /// 等待 Forge 安装完成的待添加版本
    pending_forge: Option<PendingForge>,
    ui: UiSender,
    versions: VersionManager,
}

impl GameManager {
    pub fn new(config: ConfigMC, ui: UiSender) -> Result<Self, LauncherError> {
        Ok(Self {
            cache: CacheData::new(),
            pending_forge: None,
            ui,
            versions: VersionManager::new(config)?,
        })
    }

    pub fn resolve_java_indices(&mut self, java: &JavaManager) {
        self.versions.resolve_java_indices(java);
    }

    /// Java 列表变更后，按旧索引到新索引的映射修正所有版本引用的 Java
    pub fn remap_java_indices(&mut self, index_map: &[Option<u32>]) -> Result<(), LauncherError> {
        self.versions.remap_java_indices(index_map)
    }

    pub fn version(&self, index: u32) -> &MCInstallation {
        self.versions.get(index)
    }

    pub fn version_count(&self) -> usize {
        self.versions.get_version_list().len()
    }

    pub fn version_manager_mut(&mut self) -> &mut VersionManager {
        &mut self.versions
    }

    pub fn get_config(&self) -> &ConfigMC {
        self.versions.get_config()
    }

    pub fn refresh_ui_version_list(&self) -> Result<(), LauncherError> {
        self.ui.send(UIUpdate::SetGameList(
            self.versions
                .get_version_list()
                .iter()
                .map(|v| frontend_mc_info(v.clone()))
                .collect(),
        ))?;
        self.ui
            .send(UIUpdate::SetGameIndex(self.versions.get_current_index()))
    }

    pub fn switch_current(&mut self, index: u32) -> Result<(), LauncherError> {
        self.versions.set_current_index(index)
    }

    pub fn del_game(&mut self, index: u32) -> Result<(), LauncherError> {
        self.versions.del(index)?;
        self.refresh_ui_version_list()?;
        self.ui.send(UIUpdate::QuitEditGameDialog)
    }

    /// 添加版本；已存在同名版本时保持原配置，避免添加原版 + mod 版时产生重复项
    fn add_installation(&mut self, installation: MCInstallation) -> Result<(), LauncherError> {
        if self.versions.contains(&installation.version) {
            return Ok(());
        }
        self.versions.add(&installation)
    }

    pub fn edit_game(
        &mut self,
        index: u32,
        installation: frontend::game::MCConfig,
    ) -> Result<(), LauncherError> {
        let mut version = self.versions.get(index).clone();
        version.description = installation.description;
        version.game_args = installation.game_args;
        version.height = installation.height;
        version.java_index = if installation.java_index >= 0 {
            Some(installation.java_index as u32)
        } else {
            None
        };
        version.jvm_args = installation.jvm_args;
        version.separated = installation.separated;
        version.width = installation.width;
        version.wrapper = installation.wrapper;
        version.xms = installation.xms;
        version.xmx = installation.xmx;
        self.versions.edit(index, version)?;
        self.refresh_ui_version_list()?;
        self.ui.send(UIUpdate::QuitEditGameDialog)
    }

    pub fn get_edit_game_config(
        &self,
        index: u32,
        game_path: &str,
        java: &JavaManager,
    ) -> Result<(), LauncherError> {
        let version = self.versions.get(index).clone();
        let java_list = java.info_list(game_path, Some(&version.version));
        self.ui
            .send(UIUpdate::SetEditGameConfig(frontend::game::MCConfig {
                java_index: version.java_index.map(|i| i as i32).unwrap_or(-1),
                ..frontend_mc_config(version)
            }))?;
        self.ui
            .send(UIUpdate::SetEditGameJavaList(java_list.clone()))?;
        self.ui.send(UIUpdate::SetJavaList(java_list))
    }

    pub fn get_edit_game_version(&self, index: u32) -> Result<(), LauncherError> {
        self.ui.send(UIUpdate::SetEditGameVersion(
            self.versions.get(index).version.clone(),
        ))
    }

    pub fn add_game_default(
        &self,
        config: &ConfigManager,
        java: &JavaManager,
    ) -> Result<(), LauncherError> {
        let config_mc = self.versions.get_config();
        let java_list = java.info_list(&config.general().game_path, None);
        let java_index = java.get_default().map(|i| i as i32).unwrap_or(-1);
        self.ui
            .send(UIUpdate::SetAddGameDefault(frontend::game::MCConfig {
                description: String::new(),
                game_args: Vec::new(),
                height: config_mc.height,
                java_index,
                jvm_args: Vec::new(),
                separated: false,
                width: config_mc.width,
                wrapper: config_mc.wrapper.clone(),
                xms: config_mc.xms.clone(),
                xmx: config_mc.xmx.clone(),
            }))?;
        self.ui
            .send(UIUpdate::SetAddGameJavaList(java_list.clone()))?;
        // Also send the Java list for the Java page table
        self.ui.send(UIUpdate::SetJavaList(java_list))
    }

    pub async fn add_game_java_list(
        &mut self,
        mc_type: Option<frontend::game::MCType>,
        mc_index: u32,
        game_path: &str,
        java: &JavaManager,
    ) -> Result<(), LauncherError> {
        let mut list = if let Some(list) = &self.cache.dl_mc_list {
            list.clone()
        } else {
            let list = mc::manifest::list_game(game_path.to_string()).await?;
            self.cache.dl_mc_list = Some(list.clone());
            list
        };

        if let Some(mc_type) = mc_type {
            list.retain(|v| frontend_mc_type(v.game_type.clone()) == mc_type);
        }

        if (mc_index as usize) < list.len() {
            let mc = &list[mc_index as usize];
            let java_list = java.info_list(game_path, Some(&mc.version));
            self.ui.send(UIUpdate::SetAddGameJavaList(java_list))?;
        }
        Ok(())
    }

    pub async fn add_game_list(
        &mut self,
        filter: Option<frontend::game::MCType>,
        combo_index: i32,
        game_path: &str,
    ) -> Result<(), LauncherError> {
        let mut list = if let Some(list) = &self.cache.dl_mc_list {
            list.clone()
        } else {
            let list = mc::manifest::list_game(game_path.to_string()).await?;
            self.cache.dl_mc_list = Some(list.clone());
            list
        };

        if let Some(mc_type) = filter {
            list.retain(|v| frontend_mc_type(v.game_type.clone()) == mc_type);
        }

        let dl_list = list.into_iter().map(frontend_mc_dl).collect();

        self.ui.send(UIUpdate::SetAddGameList(dl_list, combo_index))
    }

    pub async fn add_mod_list_fabric(
        &mut self,
        mc_type: Option<frontend::game::MCType>,
        index: u32,
        mc_combo: i32,
        mod_combo: i32,
        game_path: &str,
    ) -> Result<(), LauncherError> {
        let mut list = if let Some(list) = &self.cache.dl_mc_list {
            list.clone()
        } else {
            let list = mc::manifest::list_game(game_path.to_string()).await?;
            self.cache.dl_mc_list = Some(list.clone());
            list
        };

        if let Some(mc_type) = mc_type {
            list.retain(|v| frontend_mc_type(v.game_type.clone()) == mc_type);
        }

        let mc = list[index as usize].clone();

        let fabric_list = match &self.cache.dl_fabric_list {
            Some((cached_version, list)) if cached_version == &mc.version => list.clone(),
            _ => {
                let list = mc::manifest::list_fabric(&mc.version).await?;
                self.cache.dl_fabric_list = Some((mc.version.clone(), list.clone()));
                list
            }
        }
        .into_iter()
        .map(frontend_fabric)
        .collect();
        self.ui.send(UIUpdate::SetAddModListFabric(
            fabric_list,
            mc_combo,
            index,
            mod_combo,
        ))
    }

    pub async fn add_mod_list_forge(
        &mut self,
        mc_type: Option<frontend::game::MCType>,
        index: u32,
        mc_combo: i32,
        mod_combo: i32,
        game_path: &str,
    ) -> Result<(), LauncherError> {
        let mut list = if let Some(list) = &self.cache.dl_mc_list {
            list.clone()
        } else {
            let list = mc::manifest::list_game(game_path.to_string()).await?;
            self.cache.dl_mc_list = Some(list.clone());
            list
        };

        if let Some(mc_type) = mc_type {
            list.retain(|v| frontend_mc_type(v.game_type.clone()) == mc_type);
        }

        let mc = list[index as usize].clone();

        let forge_list = match &self.cache.dl_forge_list {
            Some((cached_version, list)) if cached_version == &mc.version => list.clone(),
            _ => {
                let list = mc::manifest::list_forge(&mc.version).await?;
                self.cache.dl_forge_list = Some((mc.version.clone(), list.clone()));
                list
            }
        }
        .into_iter()
        .map(frontend_forge)
        .collect();
        self.ui.send(UIUpdate::SetAddModListForge(
            forge_list, mc_combo, index, mod_combo,
        ))
    }

    pub async fn add_game(
        &mut self,
        mc_type: Option<frontend::game::MCType>,
        mc_index: u32,
        mod_type: Option<ModType>,
        mod_index: u32,
        config: frontend::game::MCConfig,
        game_path: &str,
        java: &JavaManager,
        tasks: &TaskManager,
    ) -> Result<(), LauncherError> {
        let mut mc_list = if let Some(list) = self.cache.dl_mc_list.take() {
            list
        } else {
            mc::manifest::list_game(game_path.to_string()).await?
        };

        if let Some(filter) = mc_type {
            mc_list.retain(|v| frontend_mc_type(v.game_type.clone()) == filter);
        }

        let version = mc_list[mc_index as usize].clone();
        let ver_type = version.game_type.clone();

        // 判断原版是否已下载，未下载则先下载
        let mc_json_path = format!(
            "{}/versions/{}/{}.json",
            game_path, version.version, version.version
        );
        if !exists(&mc_json_path)? {
            download_mc(game_path, version.clone()).await?;
        }

        let vanilla = i_installation(&config, version.version.clone(), ver_type.clone());

        if let Some(filter) = mod_type {
            // 两步执行：先添加原版，再添加 mod 版
            self.add_installation(vanilla)?;
            self.refresh_ui_version_list()?;

            match filter {
                ModType::Fabric => {
                    let mod_list = match self.cache.dl_fabric_list.take() {
                        Some((cached_version, list)) if cached_version == version.version => list,
                        _ => mc::manifest::list_fabric(&version.version).await?,
                    };

                    let fabric = &mod_list[mod_index as usize];

                    download_fabric(game_path, &version.version, fabric.clone()).await?;

                    let mod_version = format!(
                        "fabric-loader-{fabric_version}-{mc_version}",
                        fabric_version = fabric.loader_version,
                        mc_version = version.version,
                    );

                    // Fabric 无独立安装环节，直接添加
                    let installation = i_installation(&config, mod_version, ver_type);
                    self.add_installation(installation)?;
                    self.refresh_ui_version_list()?;
                    self.ui.send(UIUpdate::QuitAddGameDialog)?;
                }
                ModType::Forge => {
                    let mod_list = match self.cache.dl_forge_list.take() {
                        Some((cached_version, list)) if cached_version == version.version => list,
                        _ => mc::manifest::list_forge(&version.version).await?,
                    };

                    let forge = &mod_list[mod_index as usize];

                    let task_info =
                        download_forge(&version.version, forge.clone(), "{forge_source}");

                    let dir = get_parent_dir(&task_info.save_path);
                    if !exists(&dir)? {
                        create_dir_all(&dir)?;
                    }

                    let java_path = if config.java_index >= 0 {
                        java.get_java_path_by_index(config.java_index as u32)?
                    } else {
                        "java".to_string()
                    };
                    let forge_path = task_info.save_path.clone();
                    let f = move || match Command::new(&java_path)
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
                    };

                    let mut task = downloader::task::TaskInfo::new(
                        task_info.url,
                        task_info.save_path,
                        None,
                        None,
                        None,
                        Some(Box::new(f)),
                        None,
                        None,
                    );
                    task.expected_size = task_info.size;

                    let id = format!("{0}-forge-{1}", version.version, forge.version);
                    tasks.downloader().add_taskset(id.clone(), vec![task]);
                    tasks.downloader().start_taskset(id.clone())?;

                    // 记录待添加的安装信息，Forge 安装成功后再加入版本列表
                    self.pending_forge = Some(PendingForge {
                        id,
                        installation: i_installation(
                            &config,
                            format!(
                                "{mc_version}-forge-{forge_version}",
                                mc_version = version.version,
                                forge_version = forge.version,
                            ),
                            ver_type,
                        ),
                    });

                    // 显示 Forge 下载弹窗
                    self.ui.send(UIUpdate::ShowForgeDownloadDialog(format!(
                        "Downloading Forge {forge_version} for Minecraft {mc_version}",
                        forge_version = forge.version,
                        mc_version = version.version,
                    )))?;
                    // 关闭添加游戏弹窗
                    self.ui.send(UIUpdate::QuitAddGameDialog)?;
                }
            }
        } else {
            // 无 mod loader，直接添加原版
            self.add_installation(vanilla)?;
            self.refresh_ui_version_list()?;
            self.ui.send(UIUpdate::QuitAddGameDialog)?;
        }

        Ok(())
    }

    /// 处理 Forge 安装任务集的进度与结果
    pub fn handle_download_status(
        &mut self,
        info: &TaskSetStatusInfo,
    ) -> Result<(), LauncherError> {
        let Some(pending) = &self.pending_forge else {
            return Ok(());
        };
        if pending.id != info.id {
            return Ok(());
        }

        match info.status {
            TaskSetStatus::Downloading(downloaded, total) => {
                let progress = if total != 0 {
                    downloaded as f32 / total as f32
                } else {
                    0.0
                };
                self.ui.send(UIUpdate::SetForgeDownloadProgress(progress))?;
                // 下载完成，正在运行 Forge 安装程序
                if total != 0 && downloaded >= total {
                    self.ui.send(UIUpdate::SetForgeDownloadInstalling(true))?;
                }
            }
            TaskSetStatus::Pending(_) => {
                self.ui.send(UIUpdate::SetForgeDownloadProgress(0.0))?;
            }
            TaskSetStatus::Completed(_) => {
                // Forge 安装成功，将版本加入列表
                let installation = pending.installation.clone();
                self.pending_forge = None;
                self.add_installation(installation)?;
                self.refresh_ui_version_list()?;
                self.ui.send(UIUpdate::QuitForgeDownloadDialog)?;
                self.ui.send(UIUpdate::QuitAddGameDialog)?;
            }
            TaskSetStatus::Failed | TaskSetStatus::Cancelled => {
                self.pending_forge = None;
                self.ui.send(UIUpdate::QuitForgeDownloadDialog)?;
            }
            TaskSetStatus::Paused(downloaded, total) => {
                let progress = if total != 0 {
                    downloaded as f32 / total as f32
                } else {
                    0.0
                };
                self.ui.send(UIUpdate::SetForgeDownloadProgress(progress))?;
            }
        }
        Ok(())
    }

    pub fn cancel_forge_download(&mut self, tasks: &TaskManager) -> Result<(), LauncherError> {
        if let Some(pending) = self.pending_forge.take() {
            tasks.cancel(pending.id);
        }
        self.ui.send(UIUpdate::QuitForgeDownloadDialog)
    }

    pub fn hide_forge_download_dialog(&self) -> Result<(), LauncherError> {
        self.ui.send(UIUpdate::QuitForgeDownloadDialog)
    }
}
