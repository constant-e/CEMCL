//! MC相关

use log::error;
use slint::{ComponentHandle, ModelRc, StandardListViewItem, VecModel};
use std::rc;
use tokio::sync::mpsc::UnboundedSender;

use crate::app_window::UICommand;
use crate::msg_box;
use crate::ui::{self, AddGameDialog, EditGameDialog};

pub enum ModType {
    Fabric,
    Forge,
}

#[derive(Clone, PartialEq, Eq)]
pub enum MCType {
    Release,
    Snapshot,
    OldAlpha,
    OldBeta,
}

/// Fabric信息
#[derive(Clone)]
pub struct Fabric {
    pub loader_version: String,
    pub intermediary_version: String,
}

/// Forge信息
#[derive(Clone)]
pub struct Forge {
    pub version: String,
    pub branch: String,
    pub modified: String,
}

/// MC信息，下载用
pub struct MCDL {
    pub game_type: MCType,
    pub url: String,
    pub version: String,
}

/// MC配置
pub struct MCConfig {
    pub description: String,
    pub game_args: Vec<String>,
    pub height: u32,
    pub java_path: String,
    pub jvm_args: Vec<String>,
    pub separated: bool,
    pub width: u32,
    pub wrapper: String,
    pub xms: String,
    pub xmx: String,
}

/// MC信息
pub struct MCInfo {
    pub description: String,
    pub game_type: MCType,
    pub version: String,
}

impl MCType {
    fn as_str(&self) -> &str {
        match self {
            MCType::OldAlpha => "old_alpha",
            MCType::OldBeta => "old_beta",
            MCType::Release => "release",
            MCType::Snapshot => "snapshot",
        }
    }
}

impl From<ui::MCConfig> for MCConfig {
    fn from(value: ui::MCConfig) -> Self {
        let game_args = if value.game_args.is_empty() {
            Vec::new()
        } else {
            value.game_args.split(" ").map(|s| s.to_string()).collect()
        };

        let jvm_args = if value.jvm_args.is_empty() {
            Vec::new()
        } else {
            value.jvm_args.split(" ").map(|s| s.to_string()).collect()
        };

        Self {
            description: value.description.into(),
            game_args,
            height: value.height as u32,
            java_path: value.java_path.into(),
            jvm_args,
            separated: value.separated,
            width: value.width as u32,
            wrapper: value.wrapper.into(),
            xms: value.xms.into(),
            xmx: value.xmx.into(),
        }
    }
}

impl From<MCConfig> for ui::MCConfig {
    fn from(value: MCConfig) -> Self {
        let mut game_args = String::new();
        for arg in value.game_args {
            game_args += &arg;
            game_args += " ";
        }
        game_args.pop();

        let mut jvm_args = String::new();
        for arg in value.jvm_args {
            jvm_args += &arg;
            jvm_args += " ";
        }
        jvm_args.pop();

        Self {
            description: value.description.into(),
            game_args: game_args.into(),
            height: value.height as i32,
            java_path: value.java_path.into(),
            jvm_args: jvm_args.into(),
            separated: value.separated,
            width: value.width as i32,
            wrapper: value.wrapper.into(),
            xms: value.xms.into(),
            xmx: value.xmx.into(),
        }
    }
}

/// 获取ui用的download_fabric_list
pub fn ui_fabric_list(fabric_list: &Vec<Fabric>) -> ModelRc<ModelRc<StandardListViewItem>> {
    let mut ui_fabric_list: Vec<ModelRc<StandardListViewItem>> = Vec::new();
    for fabric in fabric_list {
        let version = StandardListViewItem::from(fabric.loader_version.as_str());
        let model: rc::Rc<VecModel<StandardListViewItem>> = rc::Rc::new(VecModel::from(vec![
            version.into(),
            StandardListViewItem::default(),
        ]));
        let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
        ui_fabric_list.push(row);
    }
    ModelRc::from(rc::Rc::new(VecModel::from(ui_fabric_list)))
}

/// 获取ui用的download_forge_list
pub fn ui_forge_list(forge_list: &Vec<Forge>) -> ModelRc<ModelRc<StandardListViewItem>> {
    let mut ui_forge_list: Vec<ModelRc<StandardListViewItem>> = Vec::new();
    for forge in forge_list {
        let version = StandardListViewItem::from(forge.version.as_str());
        let modified =
            StandardListViewItem::from(forge.modified.split('T').collect::<Vec<&str>>()[0]);
        let model: rc::Rc<VecModel<StandardListViewItem>> =
            rc::Rc::new(VecModel::from(vec![version.into(), modified.into()]));
        let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
        ui_forge_list.push(row);
    }
    ModelRc::from(rc::Rc::new(VecModel::from(ui_forge_list)))
}

/// 获取ui用的game_list
pub fn ui_game_list(game_list: &Vec<MCInfo>) -> ModelRc<ModelRc<StandardListViewItem>> {
    let mut ui_game_list: Vec<ModelRc<StandardListViewItem>> = Vec::new();
    for game in game_list {
        let version = StandardListViewItem::from(game.version.as_str());
        let game_type = StandardListViewItem::from(game.game_type.as_str());
        let description = StandardListViewItem::from(game.description.as_str());
        let model: rc::Rc<VecModel<StandardListViewItem>> =
            rc::Rc::from(VecModel::from(vec![version, game_type, description]));
        let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
        ui_game_list.push(row);
    }
    ModelRc::from(rc::Rc::new(VecModel::from(ui_game_list)))
}

pub fn ui_combo_box_list(game_list: &Vec<MCInfo>) -> ModelRc<slint::SharedString> {
    let list: Vec<slint::SharedString> =
        game_list.iter().map(|v| v.version.clone().into()).collect();
    ModelRc::from(rc::Rc::new(VecModel::from(list)))
}

/// 获取ui用的download_game_list
pub fn ui_game_dl_list(game_list: &Vec<MCDL>) -> ModelRc<ModelRc<StandardListViewItem>> {
    let mut ui_game_list: Vec<ModelRc<StandardListViewItem>> = Vec::new();
    for game in game_list {
        let game_type = StandardListViewItem::from(game.game_type.as_str());
        let version = StandardListViewItem::from(game.version.as_str());
        let model: rc::Rc<VecModel<StandardListViewItem>> =
            rc::Rc::new(VecModel::from(vec![version.into(), game_type.into()]));
        let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
        ui_game_list.push(row);
    }
    ModelRc::from(rc::Rc::new(VecModel::from(ui_game_list)))
}

pub fn add_game_dialog(
    tx: UnboundedSender<UICommand>,
) -> Result<slint::Weak<AddGameDialog>, slint::PlatformError> {
    let ui = AddGameDialog::new()?;
    let ui_weak = ui.as_weak();

    let tx_clone = tx.clone();
    ui.on_get_default_game_config(move || {
        if let Err(e) = tx_clone.send(UICommand::GetAddGameDefault) {
            error!("{e}");
        }
    });

    let tx_clone = tx.clone();
    ui.on_get_game_list(move |index| {
        let filter = match index {
            1 => Some(MCType::Release),
            2 => Some(MCType::Snapshot),
            3 => Some(MCType::OldAlpha),
            4 => Some(MCType::OldBeta),
            _ => None,
        };

        if let Err(e) = tx_clone.send(UICommand::GetAddGameList(filter)) {
            error!("{e}");
        }
    });

    let tx_clone = tx.clone();
    ui.on_get_mod_list(move |mc_type, index, filter| {
        let t = match mc_type {
            1 => Some(MCType::Release),
            2 => Some(MCType::Snapshot),
            3 => Some(MCType::OldAlpha),
            4 => Some(MCType::OldBeta),
            _ => None,
        };
        let msg = match filter {
            1 => UICommand::GetAddModListForge(t, index as u32),
            2 => UICommand::GetAddModListFabric(t, index as u32),
            _ => {
                error!("Unexpected mod type {filter}");
                return;
            }
        };

        if let Err(e) = tx_clone.send(msg) {
            error!("{e}");
        }
    });

    let tx_clone = tx.clone();
    ui.on_add_game(move |mc_type, mc_index, mod_type, mod_index, config| {
        let mc_filter = match mc_type {
            1 => Some(MCType::Release),
            2 => Some(MCType::Snapshot),
            3 => Some(MCType::OldAlpha),
            4 => Some(MCType::OldBeta),
            _ => None,
        };
        let mod_filter = match mod_type {
            1 => Some(ModType::Forge),
            2 => Some(ModType::Fabric),
            _ => None,
        };
        tx_clone
            .send(UICommand::AddGame(
                mc_filter,
                mc_index as u32,
                mod_filter,
                mod_index as u32,
                config.into(),
            ))
            .unwrap();
    });

    let ui_weak_clone = ui_weak.clone();
    ui.on_cancel_clicked(move || {
        if let Some(ui) = ui_weak_clone.upgrade() {
            ui.hide().unwrap();
        } else {
            error!("Failed to upgrade a weak pointer.");
        }
    });

    ui.show()?;
    tx.send(UICommand::GetAddGameDefault).unwrap();
    tx.send(UICommand::GetAddGameList(None)).unwrap();
    Ok(ui_weak)
}

pub fn edit_game_dialog(
    tx: UnboundedSender<UICommand>,
    index: u32,
) -> Result<slint::Weak<EditGameDialog>, slint::PlatformError> {
    let ui = EditGameDialog::new()?;
    let ui_weak = ui.as_weak();

    let tx_clone = tx.clone();
    ui.on_del_game(move || {
        let tx = tx_clone.clone();
        if let Err(e) = msg_box::ask_box(msg_box::AskID::DelGameConfirm, move || {
            if let Err(e) = tx.send(UICommand::DelGame(index as u32)) {
                error!("{e}");
            }
        }) {
            error!("{e}")
        }
    });

    let tx_clone = tx.clone();
    ui.on_edit_game(move |config| {
        if let Err(e) = tx_clone.send(UICommand::EditGame(index, config.into())) {
            error!("{e}");
        }
    });

    let ui_weak_clone = ui_weak.clone();
    ui.on_cancel_clicked(move || {
        if let Some(ui) = ui_weak_clone.upgrade() {
            ui.hide().unwrap();
        } else {
            error!("Failed to upgrade a weak pointer.");
        }
    });

    ui.show()?;
    tx.send(UICommand::GetEditGameConfig(index)).unwrap();
    tx.send(UICommand::GetEditGameVersion(index)).unwrap();
    Ok(ui_weak)
}
