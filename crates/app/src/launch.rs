//! 启动游戏：登录刷新、文件下载、进度汇总与进程启动

use frontend::{UIUpdate, home};
use mc::launch::get_launch_command;
use std::process::Command;

use crate::{
    LauncherError, account::AccountManager, config::ConfigManager, game::GameManager,
    java::JavaManager, tasks::TaskManager, ui::UiSender,
};

pub struct LaunchManager {
    ui: UiSender,
}

impl LaunchManager {
    pub fn new(ui: UiSender) -> Self {
        Self { ui }
    }

    pub async fn start(
        &self,
        acc_index: u32,
        ver_index: u32,
        accounts: &mut AccountManager,
        games: &mut GameManager,
        java: &JavaManager,
        config: &ConfigManager,
        tasks: &mut TaskManager,
    ) -> Result<(), LauncherError> {
        if acc_index >= accounts.get_account_list().len() as u32
            || ver_index >= games.version_count() as u32
        {
            return Err(LauncherError::OutOfRange);
        }

        self.ui
            .send(UIUpdate::SetHomePageStatus(home::State::LoggingIn))?;
        accounts.refresh_for_launch(acc_index).await?;

        self.ui
            .send(UIUpdate::SetHomePageStatus(home::State::Launching))?;
        tokio::task::yield_now().await;

        let account = accounts.get(acc_index).clone();
        let version = games.version(ver_index).clone();
        let game_path = &config.general().game_path;
        let (cmd_list, dl_list) = get_launch_command(&account, &version, game_path).await?;

        if !dl_list.is_empty() {
            self.ui
                .send(UIUpdate::SetHomePageStatus(home::State::Downloading))?;
            tokio::task::yield_now().await;

            let tasks_list: Vec<downloader::task::TaskInfo> = dl_list
                .into_iter()
                .map(|i| {
                    let mut task = downloader::task::TaskInfo::new(
                        i.url,
                        i.save_path,
                        i.sha1,
                        i.sha1_url,
                        None,
                        i.on_finish,
                        None,
                        None,
                    );
                    task.expected_size = i.size;
                    task
                })
                .collect();

            let id = version.version.clone();
            tasks.downloader().add_taskset(id.clone(), tasks_list);
            tasks.downloader().start_taskset(id.clone())?;

            // Wait for the download to complete via broadcast. The progress
            // display depends on the configured progress mode. Size-based
            // progress is clamped to never go backwards while the downloader
            // learns more precise sizes.
            let mut last_progress = 0.0;
            loop {
                let info = tasks.recv().await?;
                tasks.handle_broadcast(info.clone())?;
                games.handle_download_status(&info)?;
                if info.id != id {
                    continue;
                }
                match info.status_by_number {
                    downloader::taskset::TaskSetStatus::Completed(total) => {
                        self.send_home_progress(
                            info.progress,
                            (total, total),
                            config.general().progress_mode,
                            &mut last_progress,
                        )?;
                        tokio::task::yield_now().await;
                        break;
                    }
                    downloader::taskset::TaskSetStatus::Failed => {
                        log::error!("Failed to download {0}.", id);
                        return Err(LauncherError::DownloadFailed(id.clone()));
                    }
                    downloader::taskset::TaskSetStatus::Cancelled => {
                        return Err(LauncherError::Interrupted);
                    }
                    downloader::taskset::TaskSetStatus::Downloading(downloaded, total) => {
                        self.send_home_progress(
                            info.progress,
                            (downloaded, total),
                            config.general().progress_mode,
                            &mut last_progress,
                        )?;
                        tokio::task::yield_now().await;
                    }
                    downloader::taskset::TaskSetStatus::Paused(downloaded, total) => {
                        self.send_home_progress(
                            info.progress,
                            (downloaded, total),
                            config.general().progress_mode,
                            &mut last_progress,
                        )?;
                        tokio::task::yield_now().await;
                    }
                    downloader::taskset::TaskSetStatus::Pending(total) => {
                        self.send_home_progress(
                            info.progress,
                            (0, total),
                            config.general().progress_mode,
                            &mut last_progress,
                        )?;
                        tokio::task::yield_now().await;
                    }
                }
            }
        }

        self.ui
            .send(UIUpdate::SetHomePageStatus(home::State::Launching))?;
        tokio::task::yield_now().await;
        self.ui.send(UIUpdate::SetHomePageProgress(0.5, 1, 2))?;
        tokio::task::yield_now().await;

        let (s, r) = std::sync::mpsc::channel();
        // 如果未选择 Java，直接使用系统 PATH 中的 java
        let java_path = match version.java_index {
            Some(idx) => java.get_java_path_by_index(idx)?,
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
                self.ui.send(UIUpdate::SetHomePageProgress(1.0, 2, 2))?;
                tokio::task::yield_now().await;
                if config.general().close_after_launch {
                    self.ui.send(UIUpdate::Quit)?;
                }
            }
            Err(e) => return Err(e.into()),
        }

        self.ui
            .send(UIUpdate::SetHomePageStatus(home::State::Spare))?;
        self.ui.send(UIUpdate::SetHomePageProgress(0.0, 0, 0))?;
        Ok(())
    }

    /// 根据配置的进度显示方式，向主页发送进度更新。
    /// `size_progress` 为按字节计算的 (downloaded, total)，
    /// `number_progress` 为按任务数量计算的 (completed, total)。
    /// 按大小时钳制为单调递增，避免任务开始后分母变化导致进度倒退。
    fn send_home_progress(
        &self,
        size_progress: (u64, u64),
        number_progress: (u64, u64),
        mode: frontend::ProgressMode,
        last_progress: &mut f32,
    ) -> Result<(), LauncherError> {
        let (size_downloaded, size_total) = size_progress;
        let (number_done, number_total) = number_progress;
        let size_progress = if size_total != 0 {
            size_downloaded as f32 / size_total as f32
        } else {
            0.0
        };
        let number_progress = if number_total != 0 {
            number_done as f32 / number_total as f32
        } else {
            0.0
        };

        match mode {
            frontend::ProgressMode::BySize => {
                let progress = size_progress.max(*last_progress).min(1.0);
                *last_progress = progress;
                self.ui.send(UIUpdate::SetHomePageProgress(
                    progress,
                    size_downloaded as u32,
                    size_total as u32,
                ))
            }
            frontend::ProgressMode::ByNumber => self.ui.send(UIUpdate::SetHomePageProgress(
                number_progress,
                number_done as u32,
                number_total as u32,
            )),
            frontend::ProgressMode::Both => {
                let progress = size_progress.max(*last_progress).min(1.0);
                *last_progress = progress;
                self.ui.send(UIUpdate::SetHomePageProgress(
                    progress,
                    number_done as u32,
                    number_total as u32,
                ))
            }
        }
    }
}
