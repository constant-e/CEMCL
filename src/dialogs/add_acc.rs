//! 添加账号

use std::cell::RefCell;
use std::rc;
use log::error;
use slint::ComponentHandle;
use crate::app::App;
use crate::AddAccDialog;
use crate::mc::Account;

/// 添加账号Dialog
pub fn add_acc_dialog(app_weak: rc::Weak<RefCell<App>>) -> Result<(), slint::PlatformError> {
    let ui = AddAccDialog::new()?;
    let ui_weak = ui.as_weak();

    let mut account = Account::default();
    ui.set_offline_name(slint::SharedString::from(&account.user_name));
    ui.set_offline_uuid(slint::SharedString::from(&account.uuid));
    
    let ui_weak_clone = ui_weak.clone();
    ui.on_ok_clicked(move || {
        if let (Some(app), Some(ui)) = (app_weak.upgrade(), ui_weak_clone.upgrade()) {
            let index = ui.get_account_type_index();
            if index == 0 {
                // TODO: Online login
                return;
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

    return ui.show();
}
