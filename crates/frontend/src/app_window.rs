//! AppWindow UI封装
use log::error;
use slint::ComponentHandle;
use std::sync::{Arc, Mutex};

use crate::game::{MCInfo, ui_combo_box_list, ui_game_dl_list, ui_game_list};
use crate::java::{self, JavaInfo};
use crate::settings::Config;
use crate::ui::{self, AddGameDialog, AddJavaDialog, EditGameDialog, ForgeDownloadDialog, LoginDialog};
use crate::{
    account::{self, Account},
    game::{self, Fabric, Forge, MCConfig, MCDL, MCType, ModType},
    home, msg_box,
};

// UI -> App
pub enum UICommand {
    /// User name and UUID
    AddOfflineAccount(String, String),
    AddGame(Option<MCType>, u32, Option<ModType>, u32, MCConfig),
    AddJava(String),
    CheckJava(String),
    DelAccount(u32),
    DelGame(u32),
    DelJava(u32),
    EditAccount(u32, Account),
    EditGame(u32, MCConfig),
    FinishLogin,
    GetAddGameDefault,
    GetAddGameJavaList(Option<MCType>, u32),
    GetAddGameList(Option<MCType>),
    GetAddModListFabric(Option<MCType>, u32),
    GetAddModListForge(Option<MCType>, u32),
    GetEditGameConfig(u32),
    GetEditGameVersion(u32),
    GetJavaList,
    GetOfflineAccount,
    HideForgeDownloadDialog,
    CancelForgeDownload,
    PauseTaskSet(String),
    RequestLogin,
    ResumeTaskSet(String),
    CancelTaskSet(String),
    SetConfig(Config),
    SetDefaultJava(i32),
    Start(u32, u32),
    SwitchAccount(u32),
    SwitchGame(u32),
}

// App -> UI
pub enum UIUpdate {
    AskBox(msg_box::AskID, Box<dyn Fn() + Send + 'static>),
    MsgBox(msg_box::MsgID),
    SetAccountIndex(u32),
    SetAccountList(Vec<Account>),
    SetAddGameDefault(MCConfig),
    SetAddGameJavaList(Vec<String>),
    SetAddGameList(Vec<MCDL>),
    SetAddModListFabric(Vec<Fabric>),
    SetAddModListForge(Vec<Forge>),
    SetAuthors(String),
    SetConfig(Config),
    SetEditGameConfig(MCConfig),
    SetEditGameJavaList(Vec<String>),
    SetEditGameVersion(String),
    SetHomePageProgress(u32, u32),
    SetHomePageStatus(home::State),
    SetGameIndex(u32),
    SetGameList(Vec<MCInfo>),
    SetJavaIndex(i32),
    SetJavaList(Vec<JavaInfo>),
    SetJavaModel(Vec<String>),
    SetJavaCheckResult(ui::JavaCheckResult, String),
    SetOfflineAccount(Account),
    SetTaskSetList(Vec<crate::downloader::TaskSetInfo>),
    SetVersion(String),
    ShowForgeDownloadDialog(String),
    SetForgeDownloadMessage(String),
    SetForgeDownloadProgress(f32),
    SetForgeDownloadInstalling(bool),
    QuitForgeDownloadDialog,
    Quit,
    QuitAddGameDialog,
    QuitAddJavaDialog,
    QuitEditGameDialog,
    QuitLoginDialog,
}

#[derive(Debug)]
pub enum AppWindowError {
    SlintPlatformError(slint::PlatformError),
    UpgradeWeakPtrError,
}

impl From<slint::PlatformError> for AppWindowError {
    fn from(err: slint::PlatformError) -> Self {
        AppWindowError::SlintPlatformError(err)
    }
}

impl std::fmt::Display for AppWindowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppWindowError::SlintPlatformError(e) => write!(f, "{e}"),
            AppWindowError::UpgradeWeakPtrError => write!(f, "Failed to upgrade a weak pointer."),
        }
    }
}

fn get<T>(value: Arc<Mutex<Option<slint::Weak<T>>>>) -> Result<slint::Weak<T>, AppWindowError>
where
    T: slint::StrongHandle,
{
    match value.lock() {
        Ok(v) => {
            if let Some(w) = &*v {
                Ok(w.clone())
            } else {
                Err(AppWindowError::UpgradeWeakPtrError)
            }
        }
        Err(e) => {
            error!("{e}");
            Err(AppWindowError::UpgradeWeakPtrError)
        }
    }
}

pub struct AppWindow {
    ui: crate::ui::AppWindow,
    update_sender: tokio::sync::mpsc::UnboundedSender<UIUpdate>,
    cmd_receiver: Option<tokio::sync::mpsc::UnboundedReceiver<UICommand>>,
    // dialogs
    add_game_dialog: Arc<Mutex<Option<slint::Weak<AddGameDialog>>>>,
    add_java_dialog: Arc<Mutex<Option<slint::Weak<AddJavaDialog>>>>,
    edit_game_dialog: Arc<Mutex<Option<slint::Weak<EditGameDialog>>>>,
    forge_download_dialog: Arc<Mutex<Option<slint::Weak<ForgeDownloadDialog>>>>,
    login_dialog: Arc<Mutex<Option<slint::Weak<LoginDialog>>>>,
}

impl AppWindow {
    pub fn new() -> Result<Self, AppWindowError> {
        let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::unbounded_channel();
        let (update_tx, mut update_rx) = tokio::sync::mpsc::unbounded_channel();
        let ui = ui::AppWindow::new()?;
        let add_game_dialog = Arc::new(Mutex::new(None));
        let add_java_dialog = Arc::new(Mutex::new(None));
        let edit_game_dialog = Arc::new(Mutex::new(None));
        let forge_download_dialog = Arc::new(Mutex::new(None));
        let login_dialog = Arc::new(Mutex::new(None));

        let ui_weak = ui.as_weak();

        let tx = cmd_tx.clone();
        ui.on_del_acc(move |index| {
            let tx = tx.clone();
            if let Err(e) = msg_box::ask_box(msg_box::AskID::DelAccConfirm, move || {
                if let Err(e) = tx.send(UICommand::DelAccount(index as u32)) {
                    error!("{e}");
                }
            }) {
                error!("{e}")
            }
        });

        let tx = cmd_tx.clone();
        ui.on_del_game(move |index| {
            let tx = tx.clone();
            if let Err(e) = msg_box::ask_box(msg_box::AskID::DelGameConfirm, move || {
                if let Err(e) = tx.send(UICommand::DelGame(index as u32)) {
                    error!("{e}");
                }
            }) {
                error!("{e}")
            }
        });

        let tx = cmd_tx.clone();
        ui.on_del_java(move |index| {
            if let Err(e) = tx.send(UICommand::DelJava(index as u32)) {
                error!("{e}")
            }
        });

        let tx = cmd_tx.clone();
        ui.on_edit_acc(move |index, account| {
            if let Err(e) = tx.send(UICommand::EditAccount(index as u32, account.into())) {
                error!("{e}")
            }
        });

        let tx = cmd_tx.clone();
        let dialog = add_game_dialog.clone();
        ui.on_open_add_game_dialog(move || match dialog.lock() {
            Ok(mut dialog) => {
                let tx = tx.clone();
                match game::add_game_dialog(tx) {
                    Ok(w) => {
                        *dialog = Some(w);
                    }
                    Err(e) => {
                        error!("{e}");
                    }
                }
            }
            Err(e) => {
                error!("{e}");
            }
        });

        let tx = cmd_tx.clone();
        let dialog = add_java_dialog.clone();
        ui.on_open_add_java_dialog(move || match dialog.lock() {
            Ok(mut dialog) => {
                let tx = tx.clone();
                match java::add_java_dialog(tx) {
                    Ok(w) => {
                        *dialog = Some(w);
                    }
                    Err(e) => {
                        error!("{e}");
                    }
                }
            }
            Err(e) => {
                error!("{e}");
            }
        });

        let tx = cmd_tx.clone();
        let dialog = edit_game_dialog.clone();
        ui.on_open_edit_game_dialog(move |index| match dialog.lock() {
            Ok(mut dialog) => {
                let tx = tx.clone();
                match game::edit_game_dialog(tx, index as u32) {
                    Ok(w) => {
                        *dialog = Some(w);
                    }
                    Err(e) => {
                        error!("{e}");
                    }
                }
            }
            Err(e) => {
                error!("{e}");
            }
        });

        let tx = cmd_tx.clone();
        ui.on_open_edit_java_dialog(move |index| {
            // TODO: implement edit java dialog
        });

        let tx = cmd_tx.clone();
        let dialog = login_dialog.clone();
        ui.on_open_login_dialog(move || match dialog.lock() {
            Ok(mut dialog) => {
                let tx = tx.clone();
                match account::login_dialog(tx) {
                    Ok(w) => {
                        *dialog = Some(w);
                    }
                    Err(e) => {
                        error!("{e}");
                    }
                }
            }
            Err(e) => {
                error!("{e}");
            }
        });

        let tx = cmd_tx.clone();
        ui.on_set_config(move |config| {
            if let Err(e) = tx.send(UICommand::SetConfig(config.into())) {
                error!("{e}");
            }
        });

        let tx = cmd_tx.clone();
        ui.on_java_selected(move |index| {
            if let Err(e) = tx.send(UICommand::SetDefaultJava(index)) {
                error!("{e}");
            }
        });

        let tx = cmd_tx.clone();
        ui.on_start(move |acc_index, ver_index| {
            if let Err(e) = tx.send(UICommand::Start(acc_index as u32, ver_index as u32)) {
                error!("{e}")
            }
        });

        let tx = cmd_tx.clone();
        ui.on_switch_acc(move |index| {
            if let Err(e) = tx.send(UICommand::SwitchAccount(index as u32)) {
                error!("{e}");
            }
        });

        let tx = cmd_tx.clone();
        ui.on_switch_ver(move |index| {
            if let Err(e) = tx.send(UICommand::SwitchGame(index as u32)) {
                error!("{e}");
            }
        });

        let tx = cmd_tx.clone();
        ui.on_pause_taskset(move |id| {
            if let Err(e) = tx.send(UICommand::PauseTaskSet(id.into())) {
                error!("{e}");
            }
        });

        let tx = cmd_tx.clone();
        ui.on_resume_taskset(move |id| {
            if let Err(e) = tx.send(UICommand::ResumeTaskSet(id.into())) {
                error!("{e}");
            }
        });

        let tx = cmd_tx.clone();
        ui.on_cancel_taskset(move |id| {
            if let Err(e) = tx.send(UICommand::CancelTaskSet(id.into())) {
                error!("{e}");
            }
        });

        let ui_weak_clone = ui_weak.clone();
        let add_game_dialog_clone = add_game_dialog.clone();
        let add_java_dialog_clone = add_java_dialog.clone();
        let edit_game_dialog_clone = edit_game_dialog.clone();
        let forge_download_dialog_clone = forge_download_dialog.clone();
        let login_dialog_clone = login_dialog.clone();
        let cmd_tx_clone = cmd_tx.clone();
        tokio::spawn(async move {
            while let Some(update) = update_rx.recv().await {
                AppWindow::handle(
                    update,
                    ui_weak_clone.clone(),
                    add_game_dialog_clone.clone(),
                    add_java_dialog_clone.clone(),
                    edit_game_dialog_clone.clone(),
                    forge_download_dialog_clone.clone(),
                    login_dialog_clone.clone(),
                    cmd_tx_clone.clone(),
                )
                .await;
            }
        });

        Ok(Self {
            ui,
            update_sender: update_tx,
            cmd_receiver: Some(cmd_rx),
            add_game_dialog,
            add_java_dialog,
            edit_game_dialog,
            forge_download_dialog,
            login_dialog,
        })
    }

    async fn handle(
        update: UIUpdate,
        ui_weak: slint::Weak<ui::AppWindow>,
        add_game_dialog: Arc<Mutex<Option<slint::Weak<AddGameDialog>>>>,
        add_java_dialog: Arc<Mutex<Option<slint::Weak<AddJavaDialog>>>>,
        edit_game_dialog: Arc<Mutex<Option<slint::Weak<EditGameDialog>>>>,
        forge_download_dialog: Arc<Mutex<Option<slint::Weak<ForgeDownloadDialog>>>>,
        login_dialog: Arc<Mutex<Option<slint::Weak<LoginDialog>>>>,
        cmd_sender: tokio::sync::mpsc::UnboundedSender<UICommand>,
    ) {
        match update {
            UIUpdate::AskBox(id, f) => {
                if let Err(e) = msg_box::ask_box(id, f) {
                    error!("{e}");
                }
            }
            UIUpdate::MsgBox(id) => {
                if let Err(e) = msg_box::msg_box(id) {
                    error!("{e}");
                }
            }
            UIUpdate::SetAccountIndex(index) => {
                if let Err(e) = ui_weak.upgrade_in_event_loop(move |ui| {
                    ui.set_acc_index(index as i32);
                }) {
                    error!("{e}")
                }
            }
            UIUpdate::SetAccountList(list) => {
                if let Err(e) = ui_weak.upgrade_in_event_loop(move |ui| {
                    ui.set_acc_list(account::ui_acc_list(&list));
                }) {
                    error!("{e}")
                }
            }
            UIUpdate::SetAddGameDefault(config) => match get(add_game_dialog) {
                Ok(w) => {
                    if let Err(e) = w.upgrade_in_event_loop(|dialog| {
                        dialog.set_game_config(config.into());
                    }) {
                        error!("{e}");
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            },
            UIUpdate::SetAddGameJavaList(list) => match get(add_game_dialog) {
                Ok(w) => {
                    if let Err(e) = w.upgrade_in_event_loop(move |dialog| {
                        let items: Vec<slint::SharedString> = list.iter().map(|s| s.as_str().into()).collect();
                        dialog.set_java_combo_model(slint::ModelRc::from(std::rc::Rc::new(slint::VecModel::from(items))));
                    }) {
                        error!("{e}");
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            },
            UIUpdate::SetAddGameList(list) => match get(add_game_dialog) {
                Ok(w) => {
                    if let Err(e) = w.upgrade_in_event_loop(move |dialog| {
                        dialog.set_game_list(game::ui_game_dl_list(&list));
                    }) {
                        error!("{e}");
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            },
            UIUpdate::SetAddModListFabric(list) => match get(add_game_dialog) {
                Ok(w) => {
                    if let Err(e) = w.upgrade_in_event_loop(move |dialog| {
                        dialog.set_mod_list(game::ui_fabric_list(&list));
                    }) {
                        error!("{e}");
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            },
            UIUpdate::SetAddModListForge(list) => match get(add_game_dialog) {
                Ok(w) => {
                    if let Err(e) = w.upgrade_in_event_loop(move |dialog| {
                        dialog.set_mod_list(game::ui_forge_list(&list));
                    }) {
                        error!("{e}");
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            },
            UIUpdate::SetAuthors(authors) => {
                if let Err(e) = ui_weak.upgrade_in_event_loop(move |ui| {
                    ui.set_authors(authors.into());
                }) {
                    error!("{e}")
                }
            }
            UIUpdate::SetConfig(config) => {
                if let Err(e) = ui_weak.upgrade_in_event_loop(move |ui| {
                    ui.set_config(config.into());
                }) {
                    error!("{e}")
                }
            }
            UIUpdate::SetEditGameConfig(config) => match get(edit_game_dialog) {
                Ok(w) => {
                    if let Err(e) = w.upgrade_in_event_loop(move |dialog| {
                        dialog.set_game_config(config.into());
                    }) {
                        error!("{e}");
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            },
            UIUpdate::SetEditGameJavaList(list) => match get(edit_game_dialog) {
                Ok(w) => {
                    if let Err(e) = w.upgrade_in_event_loop(move |dialog| {
                        let items: Vec<slint::SharedString> = list.iter().map(|s| s.as_str().into()).collect();
                        dialog.set_java_combo_model(slint::ModelRc::from(std::rc::Rc::new(slint::VecModel::from(items))));
                    }) {
                        error!("{e}");
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            },
            UIUpdate::SetEditGameVersion(version) => match get(edit_game_dialog) {
                Ok(w) => {
                    if let Err(e) = w.upgrade_in_event_loop(move |dialog| {
                        dialog.set_version(version.into());
                    }) {
                        error!("{e}");
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            },
            UIUpdate::SetHomePageProgress(current, total) => {
                if let Err(e) = ui_weak.upgrade_in_event_loop(move |ui| {
                    ui.set_progress(current as f32 / total as f32);
                }) {
                    error!("{e}")
                }
            }
            UIUpdate::SetHomePageStatus(state) => {
                if let Err(e) = ui_weak.upgrade_in_event_loop(move |ui| {
                    ui.set_state(state.into());
                }) {
                    error!("{e}")
                }
            }
            UIUpdate::SetGameIndex(index) => {
                if let Err(e) = ui_weak.upgrade_in_event_loop(move |ui| {
                    ui.set_game_index(index as i32);
                }) {
                    error!("{e}")
                }
            }
            UIUpdate::SetGameList(list) => {
                if let Err(e) = ui_weak.upgrade_in_event_loop(move |ui| {
                    ui.set_game_model(ui_game_list(&list));
                    ui.set_combo_box_model(ui_combo_box_list(&list));
                }) {
                    error!("{e}")
                }
            }
            UIUpdate::SetJavaList(list) => {
                if let Err(e) = ui_weak.upgrade_in_event_loop(move |ui| {
                    ui.set_java_model(java::ui_java_list(&list));
                }) {
                    error!("{e}")
                }
            }
            UIUpdate::SetJavaModel(list) => {
                if let Err(e) = ui_weak.upgrade_in_event_loop(move |ui| {
                    let items: Vec<slint::SharedString> = list.iter().map(|s| s.as_str().into()).collect();
                    ui.set_java_combo_model(slint::ModelRc::from(std::rc::Rc::new(slint::VecModel::from(items))));
                }) {
                    error!("{e}")
                }
            }
            UIUpdate::SetJavaIndex(index) => {
                if let Err(e) = ui_weak.upgrade_in_event_loop(move |ui| {
                    ui.set_java_index(index);
                }) {
                    error!("{e}")
                }
            }
            UIUpdate::SetJavaCheckResult(result, version) => match get(add_java_dialog) {
                Ok(w) => {
                    if let Err(e) = w.upgrade_in_event_loop(move |dialog| {
                        dialog.set_check_result(result);
                        dialog.set_check_version(version.into());
                    }) {
                        error!("{e}");
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            },
            UIUpdate::SetOfflineAccount(account) => match get(login_dialog) {
                Ok(w) => {
                    if let Err(e) = w.upgrade_in_event_loop(move |dialog| {
                        dialog.set_user_name(account.user_name.into());
                        dialog.set_uuid(account.uuid.into());
                    }) {
                        error!("{e}");
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            },
            UIUpdate::SetTaskSetList(list) => {
                let unfinished: Vec<crate::downloader::TaskSetInfo> = list
                    .iter()
                    .filter(|info| {
                        info.status != crate::downloader::TaskSetStatus::Completed
                            && info.status != crate::downloader::TaskSetStatus::Failed
                            && info.status != crate::downloader::TaskSetStatus::Cancelled
                    })
                    .cloned()
                    .collect();
                let finished: Vec<crate::downloader::TaskSetInfo> = list
                    .iter()
                    .filter(|info| {
                        info.status == crate::downloader::TaskSetStatus::Completed
                            || info.status == crate::downloader::TaskSetStatus::Failed
                            || info.status == crate::downloader::TaskSetStatus::Cancelled
                    })
                    .cloned()
                    .collect();
                if let Err(e) = ui_weak.upgrade_in_event_loop(move |ui| {
                    ui.set_unfinished_list(crate::downloader::ui_unfinished_list(&unfinished));
                    ui.set_finished_list(crate::downloader::ui_finished_list(&finished));
                }) {
                    error!("{e}")
                }
            }
            UIUpdate::SetVersion(version) => {
                if let Err(e) = ui_weak.upgrade_in_event_loop(move |ui| {
                    ui.set_version(version.into());
                }) {
                    error!("{e}")
                }
            }
            UIUpdate::ShowForgeDownloadDialog(message) => {
                let dialog = forge_download_dialog.clone();
                let cmd_sender = cmd_sender.clone();
                if let Err(e) = slint::invoke_from_event_loop(move || {
                    match game::forge_download_dialog(cmd_sender) {
                        Ok(w) => match dialog.lock() {
                            Ok(mut d) => {
                                if let Some(dialog) = w.upgrade() {
                                    dialog.set_message(message.into());
                                }
                                *d = Some(w);
                            }
                            Err(e) => {
                                error!("{e}");
                            }
                        },
                        Err(e) => {
                            error!("{e}");
                        }
                    }
                }) {
                    error!("{e}");
                }
            }
            UIUpdate::SetForgeDownloadMessage(message) => match get(forge_download_dialog) {
                Ok(w) => {
                    if let Err(e) = w.upgrade_in_event_loop(move |dialog| {
                        dialog.set_message(message.into());
                    }) {
                        error!("{e}");
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            },
            UIUpdate::SetForgeDownloadProgress(progress) => match get(forge_download_dialog) {
                Ok(w) => {
                    if let Err(e) = w.upgrade_in_event_loop(move |dialog| {
                        dialog.set_progress(progress);
                    }) {
                        error!("{e}");
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            },
            UIUpdate::SetForgeDownloadInstalling(installing) => match get(forge_download_dialog) {
                Ok(w) => {
                    if let Err(e) = w.upgrade_in_event_loop(move |dialog| {
                        dialog.set_installing(installing);
                    }) {
                        error!("{e}");
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            },
            UIUpdate::QuitForgeDownloadDialog => match get(forge_download_dialog) {
                Ok(w) => {
                    if let Err(e) = w.upgrade_in_event_loop(move |dialog| {
                        dialog.hide().unwrap();
                    }) {
                        error!("{e}");
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            },
            UIUpdate::Quit => {
                if let Err(e) = ui_weak.upgrade_in_event_loop(|ui| {
                    ui.hide().unwrap();
                }) {
                    error!("{e}")
                }
            }
            UIUpdate::QuitAddGameDialog => match get(add_game_dialog) {
                Ok(w) => {
                    if let Err(e) = w.upgrade_in_event_loop(move |dialog| {
                        dialog.hide().unwrap();
                    }) {
                        error!("{e}");
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            },
            UIUpdate::QuitAddJavaDialog => match get(add_java_dialog) {
                Ok(w) => {
                    if let Err(e) = w.upgrade_in_event_loop(move |dialog| {
                        dialog.hide().unwrap();
                    }) {
                        error!("{e}");
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            },
            UIUpdate::QuitEditGameDialog => match get(edit_game_dialog) {
                Ok(w) => {
                    if let Err(e) = w.upgrade_in_event_loop(move |dialog| {
                        dialog.hide().unwrap();
                    }) {
                        error!("{e}");
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            },
            UIUpdate::QuitLoginDialog => match get(login_dialog) {
                Ok(w) => {
                    if let Err(e) = w.upgrade_in_event_loop(move |dialog| {
                        dialog.hide().unwrap();
                    }) {
                        error!("{e}");
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            },
        }
    }

    pub fn get_update_sender(&self) -> tokio::sync::mpsc::UnboundedSender<UIUpdate> {
        self.update_sender.clone()
    }

    pub fn take_cmd_receiver(&mut self) -> Option<tokio::sync::mpsc::UnboundedReceiver<UICommand>> {
        self.cmd_receiver.take()
    }

    pub fn run(&self) -> Result<(), slint::PlatformError> {
        self.ui.run()
    }
}
