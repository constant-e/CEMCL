//! 添加账号

use std::cell::RefCell;
use std::rc;
use clipboard::{ClipboardContext, ClipboardProvider};
use log::{error, warn};
use slint::ComponentHandle;
use crate::app::App;
use crate::AddAccDialog;
use crate::mc::Account;
use crate::mc::account::init_oauth;
use crate::Messages;

/// 添加账号Dialog
pub async fn add_acc_dialog(app_weak: rc::Weak<RefCell<App>>) -> Result<(), slint::PlatformError> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _tokio = rt.enter();

    let ui = AddAccDialog::new()?;
    let ui_weak = ui.as_weak();

    let mut account = Account::default();
    ui.set_offline_name(slint::SharedString::from(&account.user_name));
    ui.set_offline_uuid(slint::SharedString::from(&account.uuid));

    if let (Some(app), Some(ui)) = (app_weak.upgrade(), ui_weak.upgrade()) {
        if let Some((message, device_code, user_code)) = init_oauth().await {
            if let Ok(ctx) = ClipboardProvider::new() {
                let mut ctx: ClipboardContext = ctx;  // type announce is needed
                if let Err(e) = ctx.set_contents(user_code) {
                    warn!("Failed to copy user code. Reason: {e}");
                }
            } else {
                warn!("Failed to copy user code.");
            }

            app.borrow_mut().device_code = device_code;

            let msg = app.borrow().ui_weak.upgrade()
                .ok_or(slint::PlatformError::Other(String::from("Failed to upgrade a weak pointer.")))?
                .global::<Messages>().get_acc_online_msg().to_string();
            let message = message + "\n" + &msg;
            ui.set_online_msg(message.into());
        } else {
            let msg = app.borrow().ui_weak.upgrade()
                .ok_or(slint::PlatformError::Other(String::from("Failed to upgrade a weak pointer.")))?
                .global::<Messages>().get_acc_online_failed().to_string();
            ui.set_online_msg(msg.into());
        }
    } else {
        error!("Failed to upgrade a weak pointer.");
        return Err(slint::PlatformError::Other(String::from("Failed to upgrade a weak pointer.")));
    }
    
    let ui_weak_clone = ui_weak.clone();
    ui.on_ok_clicked(move || {
        if let (Some(app), Some(ui)) = (app_weak.upgrade(), ui_weak_clone.upgrade()) {
            let index = ui.get_account_type_index();
            if index == 0 {
                // Online Account
                let rt = tokio::runtime::Runtime::new().unwrap();
                let _tokio = rt.enter();
                if let Some(acc) = rt.block_on(Account::new(&app.borrow().device_code)) {
                    account = acc;
                } else {
                    error!("Failed to login.");
                    return;
                }
            } else if index == 1 {
                // Offline Account
                account.user_name = ui.get_offline_name().to_string();
                account.uuid = ui.get_offline_uuid().to_string();
            } else {
                // Costumized Account
                account.account_type = ui.get_other_acc_type().to_string();
                account.refresh_token = ui.get_other_token().to_string();
                account.uuid = ui.get_other_uuid().to_string();
                account.user_name = ui.get_other_name().to_string();
            }
            app.borrow_mut().add_account(&account);
            ui.hide().unwrap();
        } else {
            error!("Failed to upgrade a weak pointer.");
        }
    });

    ui.on_cancel_clicked(move || {
        let ui = ui_weak.unwrap();
        ui.hide().unwrap();
    });

    ui.show()
}
