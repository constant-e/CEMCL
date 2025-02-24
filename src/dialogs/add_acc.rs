//! 添加账号

use crate::AddAccDialog;
use crate::Messages;
use crate::app::App;
use crate::mc::Account;
use crate::mc::account::init_oauth;
use clipboard::{ClipboardContext, ClipboardProvider};
use log::{error, warn};
use slint::ComponentHandle;
use std::sync::{self, Mutex};
use std::thread;

/// 添加账号Dialog
pub async fn add_acc_dialog(app_weak: sync::Weak<Mutex<App>>) -> Result<(), slint::PlatformError> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _tokio = rt.enter();

    let ui = AddAccDialog::new()?;
    let ui_weak = ui.as_weak();

    let mut account = Account::default();
    ui.set_offline_name(slint::SharedString::from(&account.user_name));
    ui.set_offline_uuid(slint::SharedString::from(&account.uuid));

    if let (Some(app), Some(ui)) = (app_weak.upgrade(), ui_weak.upgrade()) {
        if let Some((message, device_code, user_code, url)) = init_oauth().await {
            if let Ok(ctx) = ClipboardProvider::new() {
                let mut ctx: ClipboardContext = ctx; // type announce is needed
                if let Err(e) = ctx.set_contents(user_code) {
                    warn!("Failed to copy user code. Reason: {e}");
                }
            } else {
                warn!("Failed to copy user code.");
            }

            if let Err(e) = webbrowser::open(&url) {
                warn!("Failed to open web browser. Reason: {e}");
            }

            if let Ok(mut app) = app.try_lock() {
                app.device_code = device_code;
                let msg = app
                    .ui_weak
                    .upgrade()
                    .ok_or(slint::PlatformError::Other(String::from(
                        "Failed to upgrade a weak pointer",
                    )))?
                    .global::<Messages>()
                    .get_acc_online_msg()
                    .to_string();
                let message = message + "\n" + &msg;
                ui.set_online_msg(message.into());
            } else {
                error!("Failed to lock a mutex.");
                return Err(slint::PlatformError::Other(String::from(
                    "Failed to lock a mutex",
                )));
            }
        } else {
            if let Ok(app) = app.try_lock() {
                let msg = app
                    .ui_weak
                    .upgrade()
                    .ok_or(slint::PlatformError::Other(String::from(
                        "Failed to upgrade a weak pointer",
                    )))?
                    .global::<Messages>()
                    .get_acc_online_failed()
                    .to_string();
                ui.set_online_msg(msg.into());
            } else {
                error!("Failed to lock a mutex.");
                return Err(slint::PlatformError::Other(String::from(
                    "Failed to lock a mutex",
                )));
            }
        }
    } else {
        error!("Failed to upgrade a weak pointer.");
        return Err(slint::PlatformError::Other(String::from(
            "Failed to upgrade a weak pointer",
        )));
    }

    let ui_weak_clone = ui_weak.clone();
    ui.on_ok_clicked(move || {
        if let (Some(app), Some(ui)) = (app_weak.upgrade(), ui_weak_clone.upgrade()) {
            let index = ui.get_account_type_index();
            if index == 0 {
                // Online Account
                thread::spawn(move || {
                    if let Ok(mut app) = app.try_lock() {
                        app.ui_weak
                            .upgrade_in_event_loop(|ui| ui.invoke_set_loading())
                            .unwrap();
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        let _tokio = rt.enter();
                        if let Some(acc) =
                            rt.block_on(Account::new(&app.device_code, app.ui_weak.clone()))
                        {
                            app.add_account(&acc);
                        } else {
                            error!("Failed to login.");
                        }
                        app.ui_weak
                            .upgrade_in_event_loop(|ui| ui.invoke_unset_loading())
                            .unwrap();
                    }
                });
            } else {
                if let Ok(mut app) = app.try_lock() {
                    if index == 1 {
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
                    app.add_account(&account);
                } else {
                    error!("Failed to lock a mutex.");
                }
            }

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
