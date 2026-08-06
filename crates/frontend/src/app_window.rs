//! AppWindow UI封装
use log::error;
use slint::ComponentHandle;
use std::sync::{Arc, Mutex};

use crate::game::{MCInfo, ui_combo_box_list, ui_game_dl_list, ui_game_list};
use crate::settings::Config;
use crate::ui::{self, AddGameDialog, EditGameDialog, LoginDialog};
use crate::{
    account::{self, Account},
    game::{self, Fabric, Forge, MCConfig, MCDL, MCType, ModType},
    home,
};

// UI -> App
pub enum UICommand {
    /// User name and UUID
    AddOfflineAccount(String, String),
    AddGame(Option<MCType>, u32, Option<ModType>, u32, MCConfig),
    DelAccount(u32),
    DelGame(u32),
    DelJava(u32),
    EditAccount(u32, Account),
    EditGame(u32, MCConfig),
    FinishLogin,
    GetAddGameDefault,
    GetAddGameList(Option<MCType>),
    GetAddModListFabric(Option<MCType>, u32),
    GetAddModListForge(Option<MCType>, u32),
    GetEditGameConfig(u32),
    GetEditGameVersion(u32),
    GetOfflineAccount,
    RequestLogin,
    SetConfig(Config),
    Start(u32, u32),
    SwitchAccount(u32),
    SwitchGame(u32),
}

// App -> UI
pub enum UIUpdate {
    SetAccountIndex(u32),
    SetAccountList(Vec<Account>),
    SetAddGameDefault(MCConfig),
    SetAddGameList(Vec<MCDL>),
    SetAddModListFabric(Vec<Fabric>),
    SetAddModListForge(Vec<Forge>),
    SetAuthors(String),
    SetConfig(Config),
    SetEditGameConfig(MCConfig),
    SetEditGameVersion(String),
    SetHomePageProgress(u32, u32),
    SetHomePageStatus(home::State),
    SetGameIndex(u32),
    SetGameList(Vec<MCInfo>),
    SetOfflineAccount(Account),
    SetVersion(String),
    Quit,
    QuitAddGameDialog,
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
    edit_game_dialog: Arc<Mutex<Option<slint::Weak<EditGameDialog>>>>,
    login_dialog: Arc<Mutex<Option<slint::Weak<LoginDialog>>>>,
}

impl AppWindow {
    pub fn new() -> Result<Self, AppWindowError> {
        let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::unbounded_channel();
        let (update_tx, mut update_rx) = tokio::sync::mpsc::unbounded_channel();
        let ui = ui::AppWindow::new()?;
        let add_game_dialog = Arc::new(Mutex::new(None));
        let edit_game_dialog = Arc::new(Mutex::new(None));
        let login_dialog = Arc::new(Mutex::new(None));

        let ui_weak = ui.as_weak();

        let tx = cmd_tx.clone();
        ui.on_del_acc(move |index| {
            if let Err(e) = tx.send(UICommand::DelAccount(index as u32)) {
                error!("{e}");
            }
        });

        let tx = cmd_tx.clone();
        ui.on_del_game(move |index| {
            if let Err(e) = tx.send(UICommand::DelGame(index as u32)) {
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
        ui.on_open_add_java_dialog(move || {});

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
        ui.on_open_edit_java_dialog(move |index| {});

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

        let ui_weak_clone = ui_weak.clone();
        let add_game_dialog_clone = add_game_dialog.clone();
        let edit_game_dialog_clone = edit_game_dialog.clone();
        let login_dialog_clone = login_dialog.clone();
        tokio::spawn(async move {
            while let Some(update) = update_rx.recv().await {
                AppWindow::handle(
                    update,
                    ui_weak_clone.clone(),
                    add_game_dialog_clone.clone(),
                    edit_game_dialog_clone.clone(),
                    login_dialog_clone.clone(),
                )
                .await;
            }
        });

        Ok(Self {
            ui,
            update_sender: update_tx,
            cmd_receiver: Some(cmd_rx),
            add_game_dialog,
            edit_game_dialog,
            login_dialog,
        })
    }

    async fn handle(
        update: UIUpdate,
        ui_weak: slint::Weak<ui::AppWindow>,
        add_game_dialog: Arc<Mutex<Option<slint::Weak<AddGameDialog>>>>,
        edit_game_dialog: Arc<Mutex<Option<slint::Weak<EditGameDialog>>>>,
        login_dialog: Arc<Mutex<Option<slint::Weak<LoginDialog>>>>,
    ) {
        match update {
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
            UIUpdate::SetVersion(version) => {
                if let Err(e) = ui_weak.upgrade_in_event_loop(move |ui| {
                    ui.set_version(version.into());
                }) {
                    error!("{e}")
                }
            }
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
