//! Java Page 相关

use log::error;
use slint::{ComponentHandle, ModelRc, StandardListViewItem, VecModel};
use std::rc;
use tokio::sync::mpsc::UnboundedSender;

use crate::app_window::UICommand;
use crate::ui::{self, AddJavaDialog};

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
/// The tr() function is used for the "(not compatible)" suffix.
pub fn ui_java_combo_box_list(list: &Vec<JavaInfo>) -> Vec<String> {
    list.iter()
        .map(|j| {
            if j.compatible {
                j.version.clone()
            } else {
                format!("{} ({})", j.version, tr_not_compatible())
            }
        })
        .collect()
}

/// Returns the localized "not compatible" string.
/// This is a function so it can be called from non-Slint Rust code.
fn tr_not_compatible() -> String {
    // Use Slint's translation mechanism via a temporary component or just hardcode English
    // Since we're in Rust, we use the English default. The Slint UI will handle translation
    // when the string is part of the model — but since we're building the model in Rust,
    // we need to handle it here. For now, use English as the model strings are displayed
    // directly in the ComboBox.
    "not compatible".to_string()
}

/// Convert a list of JavaInfo to a Slint ModelRc<SharedString> for the ComboBox.
pub fn ui_java_combo_box_model(list: &Vec<JavaInfo>) -> ModelRc<slint::SharedString> {
    let items: Vec<slint::SharedString> = ui_java_combo_box_list(list)
        .into_iter()
        .map(|s| s.into())
        .collect();
    ModelRc::from(rc::Rc::new(VecModel::from(items)))
}