//! CEMCL Application Runtime
//!
//! `AppRuntime` 只负责组合各 manager 并分发 UI 命令；
//! 账号、Java、游戏、下载与启动的具体逻辑分别位于对应的 manager。

use frontend::{UICommand, UIUpdate};
use log::error;

use crate::{
    LauncherError,
    account::AccountManager,
    config::{ConfigDL, ConfigManager},
    game::GameManager,
    java::JavaManager,
    launch::LaunchManager,
    tasks::TaskManager,
    ui::UiSender,
};

pub struct AppRuntime {
    account_manager: AccountManager,
    cmd_receiver: Option<tokio::sync::mpsc::UnboundedReceiver<UICommand>>,
    config: ConfigManager,
    games: GameManager,
    java_manager: JavaManager,
    launch: LaunchManager,
    tasks: TaskManager,
    ui: UiSender,
}

impl AppRuntime {
    pub fn new(
        update_sender: tokio::sync::mpsc::UnboundedSender<UIUpdate>,
        cmd_receiver: tokio::sync::mpsc::UnboundedReceiver<UICommand>,
    ) -> Result<Self, LauncherError> {
        let ui = UiSender::new(update_sender);
        let (config, config_dl, config_mc) = ConfigManager::load()?;
        let tasks = TaskManager::new(config_dl.into(), ui.clone());
        let account_manager = AccountManager::new(ui.clone())?;
        let mut java_manager = JavaManager::new(ui.clone())?;
        let mut games = GameManager::new(config_mc, ui.clone())?;
        // java.json 中的失效项已删除、系统 Java 已合并并按版本排序；
        // 先按路径修正版本中记录的 java_index，再为未选择 Java 的版本自动匹配
        games.remap_java_indices(&java_manager.take_index_map())?;
        games.resolve_java_indices(&java_manager);

        Ok(Self {
            account_manager,
            cmd_receiver: Some(cmd_receiver),
            config,
            games,
            java_manager,
            launch: LaunchManager::new(ui.clone()),
            tasks,
            ui,
        })
    }

    fn init(&mut self) -> Result<(), LauncherError> {
        self.account_manager.init()?;
        self.refresh_ui_info()?;
        self.refresh_ui_config()?;
        self.java_manager.refresh_ui_list()?;
        self.games.refresh_ui_version_list()
    }

    /// 分发 UI 命令；具体业务由各 manager 处理
    async fn handle(&mut self, cmd: UICommand) -> Result<(), LauncherError> {
        match cmd {
            UICommand::AddOfflineAccount(user_name, uuid) => {
                self.account_manager.add_offline(user_name, uuid)
            }
            UICommand::AddGame(mc_type, mc_index, mod_type, mod_index, config) => {
                self.games
                    .add_game(
                        mc_type,
                        mc_index,
                        mod_type,
                        mod_index,
                        config,
                        &self.config.general().game_path,
                        &self.java_manager,
                        &self.tasks,
                    )
                    .await
            }
            UICommand::AddJava(java_path) => {
                self.java_manager
                    .add_from_ui(std::path::PathBuf::from(java_path))?;
                let index_map = self.java_manager.take_index_map();
                self.games.remap_java_indices(&index_map)
            }
            UICommand::CheckJava(java_path) => {
                self.java_manager.check(std::path::PathBuf::from(java_path))
            }
            UICommand::DelAccount(index) => self.account_manager.del_from_ui(index),
            UICommand::DelGame(index) => self.games.del_game(index),
            UICommand::DelJava(index) => {
                self.java_manager.del_from_ui(index)?;
                let index_map = self.java_manager.take_index_map();
                self.games.remap_java_indices(&index_map)
            }
            UICommand::EditAccount(index, account) => {
                self.account_manager.edit_from_ui(index, account)
            }
            UICommand::EditGame(index, installation) => self.games.edit_game(index, installation),
            UICommand::FinishLogin => self.account_manager.finish_login().await,
            UICommand::GetAddGameDefault => self
                .games
                .add_game_default(&self.config, &self.java_manager),
            UICommand::GetAddGameJavaList(mc_type, mc_index) => {
                self.games
                    .add_game_java_list(
                        mc_type,
                        mc_index,
                        &self.config.general().game_path,
                        &self.java_manager,
                    )
                    .await
            }
            UICommand::GetAddGameList(filter, combo_index) => {
                self.games
                    .add_game_list(filter, combo_index, &self.config.general().game_path)
                    .await
            }
            UICommand::GetAddModListFabric(mc_type, index, mc_combo, mod_combo) => {
                self.games
                    .add_mod_list_fabric(
                        mc_type,
                        index,
                        mc_combo,
                        mod_combo,
                        &self.config.general().game_path,
                    )
                    .await
            }
            UICommand::GetAddModListForge(mc_type, index, mc_combo, mod_combo) => {
                self.games
                    .add_mod_list_forge(
                        mc_type,
                        index,
                        mc_combo,
                        mod_combo,
                        &self.config.general().game_path,
                    )
                    .await
            }
            UICommand::GetEditGameConfig(index) => self.games.get_edit_game_config(
                index,
                &self.config.general().game_path,
                &self.java_manager,
            ),
            UICommand::GetEditGameVersion(index) => self.games.get_edit_game_version(index),
            UICommand::GetOfflineAccount => self.account_manager.get_offline_account(),
            UICommand::HideForgeDownloadDialog => self.games.hide_forge_download_dialog(),
            UICommand::CancelForgeDownload => self.games.cancel_forge_download(&self.tasks),
            UICommand::PauseTaskSet(id) => {
                self.tasks.pause(id);
                Ok(())
            }
            UICommand::RequestLogin => self.account_manager.request_login().await,
            UICommand::ResumeTaskSet(id) => {
                self.tasks.resume(id);
                Ok(())
            }
            UICommand::CancelTaskSet(id) => {
                self.tasks.cancel(id);
                Ok(())
            }
            UICommand::SetConfig(config) => self.config.apply(
                config,
                self.tasks.downloader_mut(),
                self.games.version_manager_mut(),
            ),
            UICommand::SetDefaultJava(index) => self.java_manager.set_default_from_ui(index),
            UICommand::Start(acc_index, ver_index) => {
                self.launch
                    .start(
                        acc_index,
                        ver_index,
                        &mut self.account_manager,
                        &mut self.games,
                        &self.java_manager,
                        &self.config,
                        &mut self.tasks,
                    )
                    .await
            }
            UICommand::SwitchAccount(index) => self.account_manager.switch_current(index),
            UICommand::SwitchGame(index) => self.games.switch_current(index),
        }
    }

    fn refresh_ui_info(&self) -> Result<(), LauncherError> {
        self.ui
            .send(UIUpdate::SetAuthors(env!("CARGO_PKG_AUTHORS").into()))?;
        self.ui
            .send(UIUpdate::SetVersion(env!("CARGO_PKG_VERSION").into()))
    }

    fn refresh_ui_config(&self) -> Result<(), LauncherError> {
        let config_dl: ConfigDL = self.tasks.downloader().get_config().clone().into();
        let config_mc = self.games.get_config();
        self.ui.send(UIUpdate::SetConfig(frontend::Config {
            dl: config_dl.into(),
            general: self.config.general().clone().into(),
            mc: config_mc.clone().into(),
        }))?;

        let java_list = self
            .java_manager
            .info_list(&self.config.general().game_path, None);
        self.ui.send(UIUpdate::SetJavaModel(java_list))?;
        self.ui.send(UIUpdate::SetJavaIndex(
            self.java_manager
                .get_default()
                .map(|i| i as i32)
                .unwrap_or(-1),
        ))
    }

    pub async fn run(&mut self) -> Result<(), LauncherError> {
        self.init()?;

        let mut cmd_receiver = self
            .cmd_receiver
            .take()
            .ok_or(LauncherError::ChannelNotFound)?;
        let mut avatar_receiver = self
            .account_manager
            .take_avatar_receiver()
            .ok_or(LauncherError::ChannelNotFound)?;
        loop {
            tokio::select! {
                Some(cmd) = cmd_receiver.recv() => {
                    if let Err(e) = self.handle(cmd).await {
                        error!("{e}");
                        let (msgid, detail) = e.ui_msg();
                        self.ui.send(UIUpdate::ShowMsgBox(msgid, detail))?;
                        self.ui.send(UIUpdate::SetHomePageStatus(frontend::home::State::Spare))?;
                        self.ui.send(UIUpdate::SetHomePageProgress(0.0, 0, 0))?;
                    }
                },
                Some((uuid, avatar)) = avatar_receiver.recv() => {
                    self.account_manager.handle_avatar_update(uuid, avatar)?;
                },
                info = self.tasks.recv() => {
                    let info = info?;
                    self.tasks.handle_broadcast(info.clone())?;
                    self.games.handle_download_status(&info)?;
                },
            }
        }
    }
}
