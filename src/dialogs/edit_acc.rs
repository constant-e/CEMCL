//! 修改账号

use std::sync::{self, Mutex};

use log::error;
use slint::ComponentHandle;
use crate::{app::App, dialogs::msg_box::err_dialog, mc::Account, EditAccDialog, Messages};

pub fn edit_acc_dialog(app_weak: sync::Weak<Mutex<App>>) -> Result<(), slint::PlatformError> {
    let ui = EditAccDialog::new()?;
    let ui_weak = ui.as_weak();

    let index = if let Some(app) = app_weak.upgrade() {
        if let Ok(app) = app.try_lock() {
            if let Some(index) = app.get_acc_index() {
                let account = &app.acc_list[index];
                ui.set_acc_type(slint::SharedString::from(&account.account_type));
                ui.set_name(slint::SharedString::from(&account.user_name));
                ui.set_token(slint::SharedString::from(&account.refresh_token));
                ui.set_uuid(slint::SharedString::from(&account.uuid));
                index
            } else {
                err_dialog(&app.ui_weak.upgrade()
                    .ok_or(slint::PlatformError::Other(String::from("Failed to upgrade a weak pointer")))?
                    .global::<Messages>().get_acc_not_selected());
                return Err(slint::PlatformError::Other(String::from("Failed to get the index of acc_list")));
            }
        } else {
            error!("Failed to lock a mutex.");
            return Err(slint::PlatformError::Other(String::from("Failed to lock a mutex")));
        }
    } else {
        error!("Failed to upgrade a weak pointer.");
        return Err(slint::PlatformError::Other(String::from("Failed to upgrade a weak pointer")));
    };

    let app_weak_clone = app_weak.clone();
    let ui_weak_clone = ui_weak.clone();
    ui.on_ok_clicked(move || {
        if let (Some(app), Some(ui)) = (app_weak_clone.upgrade(), ui_weak_clone.upgrade()) {
            if let Ok(mut app) = app.try_lock() {
                let mut account = Account {
                    access_token: String::new(),
                    account_type: ui.get_acc_type().into(),
                    refresh_token: ui.get_token().into(),
                    user_name: ui.get_name().into(),
                    uuid: ui.get_uuid().into(),
                };
    
                let rt = tokio::runtime::Runtime::new().unwrap();
                let _tokio = rt.enter();
                rt.block_on(account.refresh());
    
                app.edit_account(index, account);
            } else {
                error!("Failed to lock a mutex.");
            }
            
            ui.hide().unwrap();
        } else {
            error!("Failed to upgrade a weak pointer.");
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

    ui.on_del_clicked(move || {
        if let (Some(app), Some(ui)) = (app_weak.upgrade(), ui_weak.upgrade()) {
            if let Ok(mut app) = app.try_lock() {
                app.del_account(index);
                ui.hide().unwrap();
            } else {
                error!("Failed to lock a mutex.");
            }
        } else {
            error!("Failed to upgrade a weak pointer.");
        }
    });

    ui.show()
}
