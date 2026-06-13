//! 添加账号

use crate::LoginDialog;
use crate::app::App;
use crate::dialogs::msgbox::{self, MsgID};
use crate::mc::Account;
use crate::mc::account::init_oauth;
use clipboard::{ClipboardContext, ClipboardProvider};
use log::{error, warn};
use slint::ComponentHandle;
use std::sync::{self, Mutex};
use std::thread;

/// 添加账号Dialog
pub fn login_dialog(app_weak: sync::Weak<Mutex<App>>) -> Result<(), slint::PlatformError> {
    let ui = LoginDialog::new()?;
    let ui_weak = ui.as_weak();

    let mut account = Account::default();

    ui.set_user_name(slint::SharedString::from(&account.user_name));
    ui.set_uuid(slint::SharedString::from(&account.uuid));

    let app_weak_clone = app_weak.clone();
    ui.on_msa_clicked(move || {
        let app_weak_clone = app_weak_clone.clone();
        if let Err(e) = slint::invoke_from_event_loop(move || {
            if let Err(e) = slint::spawn_local(async move {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let _tokio = rt.enter();
                if let Some((_, device_code, user_code, url)) = init_oauth().await {
                    if let Some(app) = app_weak_clone.upgrade() {
                        if let Ok(mut app) = app.try_lock() {
                            app.device_code = device_code;
                        } else {
                            error!("Failed to lock a mutex.");
                        }
                    } else {
                        error!("Failed to upgrade a weak pointer.");
                    }
                    if let Ok(ctx) = ClipboardProvider::new() {
                        let mut ctx: ClipboardContext = ctx; // type announce is needed
                        if let Err(e) = ctx.set_contents(user_code) {
                            warn!("Failed to copy user code. Reason: {e}");
                        }

                        if let Err(e) = webbrowser::open(&url) {
                            warn!("Failed to open web browser. Reason: {e}");
                        }
                    } else {
                        warn!("Failed to copy user code.");
                        msgbox::msg_dialog(MsgID::CopyUserCodeFailed(user_code)).unwrap();
                    }
                } else {
                    msgbox::msg_dialog(MsgID::OAuthFailed).unwrap();
                }
            }) {
                error!("{e}");
            };
        }) {
            error!("{e}");
        }
    });

    let app_weak_clone = app_weak.clone();
    let ui_weak_clone = ui_weak.clone();
    ui.on_msa_ok_clicked(move || {
        if let (Some(app), Some(ui)) = (app_weak_clone.upgrade(), ui_weak_clone.upgrade()) {
            if let Ok(mut app) = app.try_lock() {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let _tokio = rt.enter();
                if let Some(acc) =
                    rt.block_on(Account::new(&app.device_code, app.ui_weak.clone()))
                {
                    if let Err(e) = app.add_account(&acc) {
                        error!("{e}");
                    }
                    ui.hide().unwrap();
                } else {
                    error!("Failed to login.");
                }
            }
        }
    });

    ui.on_offline_ok_clicked(move || {
        if let (Some(app), Some(ui)) = (app_weak.upgrade(), ui_weak.upgrade()) {
            if let Ok(mut app) = app.try_lock() {
                account.user_name = ui.get_user_name().into();
                account.uuid = ui.get_uuid().into();
                if let Err(e) = app.add_account(&account) {
                    error!("{e}");
                }
                ui.hide().unwrap();
            }
        }
    });

    ui.show()
}
