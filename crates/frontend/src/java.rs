//! Java Page 相关

use log::error;
use slint::{ComponentHandle, ModelRc, StandardListViewItem, VecModel};
use std::rc;
use tokio::sync::mpsc::UnboundedSender;

use crate::app_window::UICommand;
use crate::ui::AddJavaDialog;

/// Java 安装信息，用于前端展示
#[derive(Clone)]
pub struct JavaInfo {
    pub version: String,
    pub path: String,
    /// 是否与当前选中的 MC 版本兼容
    pub compatible: bool,
}

/// Create the Add Java dialog and wire up callbacks
pub fn add_java_dialog(
    tx: UnboundedSender<UICommand>,
) -> Result<slint::Weak<AddJavaDialog>, slint::PlatformError> {
    let ui = AddJavaDialog::new()?;
    let ui_weak = ui.as_weak();

    let tx_clone = tx.clone();
    ui.on_add_java(move |path| {
        if let Err(e) = tx_clone.send(UICommand::AddJava(path.into())) {
            error!("{e}");
        }
    });

    let tx_clone = tx.clone();
    ui.on_check_java(move |path| {
        if let Err(e) = tx_clone.send(UICommand::CheckJava(path.into())) {
            error!("{e}");
        }
    });

    ui.show()?;
    Ok(ui_weak)
}

/// Convert a list of JavaInfo to a Slint model for the Java table (Java Page)
pub fn ui_java_list(list: &Vec<JavaInfo>) -> ModelRc<ModelRc<StandardListViewItem>> {
    let mut rows: Vec<ModelRc<StandardListViewItem>> = Vec::new();
    for java in list {
        let version = StandardListViewItem::from(java.version.as_str());
        let path = StandardListViewItem::from(java.path.as_str());
        let model: rc::Rc<VecModel<StandardListViewItem>> =
            rc::Rc::new(VecModel::from(vec![version.into(), path.into()]));
        let row: ModelRc<StandardListViewItem> = ModelRc::from(model);
        rows.push(row);
    }
    ModelRc::from(rc::Rc::new(VecModel::from(rows)))
}

/// Build a formatted combo box model for JavaVersionSelector.
/// Compatible versions show just the version string (e.g. "26.0.1").
/// Incompatible versions show "version (not compatible)" (e.g. "1.8.0_500 (not compatible)").
/// The localized suffix is read from the UI via AppWindow's `not-compatible-text` property.
pub fn ui_java_combo_box_list(list: &Vec<JavaInfo>, not_compatible: &str) -> Vec<String> {
    list.iter()
        .map(|j| {
            if j.compatible {
                j.version.clone()
            } else {
                format!("{} ({})", j.version, not_compatible)
            }
        })
        .collect()
}