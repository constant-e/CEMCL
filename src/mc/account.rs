use std::cell::RefCell;
use std::fs;
use std::rc::Rc;
use serde_json::Value;
use crate::file_tools::exists;
use crate::ui_acc_list;
use super::Account;

slint::include_modules!();

pub fn add_dialog(acc_list: &Rc<RefCell<Vec<Account>>>, app: &crate::AppWindow) {
    let ui = AddAccDialog::new().unwrap();
    ui.set_offline_uuid(uuid::Uuid::new_v4().to_string().into());
    
    ui.on_ok_clicked({
        let app_handle = app.as_weak();
        let ui_handle = ui.as_weak();
        let acc_list_handle = Rc::downgrade(&acc_list);
        move || {
            let app = app_handle.unwrap();
            let ui = ui_handle.unwrap();
            let acc_list = acc_list_handle.upgrade().unwrap();
            let index = ui.get_account_type_index();
            if index == 0 {
                // TODO: Online login
            } else if index == 1 {
                // Offline Account
                acc_list.borrow_mut().push(Account {
                    account_type: RefCell::from("Legacy".to_string()),
                    token: RefCell::from("None".to_string()),
                    uuid: RefCell::from(ui.get_offline_uuid().to_string()),
                    user_name: RefCell::from(ui.get_offline_name().to_string()),
                });
            } else {
                // Costumized Account
                acc_list.borrow_mut().push(Account {
                    account_type: RefCell::from(ui.get_other_acc_type().to_string()),
                    token: RefCell::from(ui.get_other_token().to_string()),
                    uuid: RefCell::from(ui.get_other_uuid().to_string()),
                    user_name: RefCell::from(ui.get_other_name().to_string()),
                });
            }
            save(acc_list.borrow().as_ref());
            app.set_acc_list(ui_acc_list(acc_list.borrow().as_ref()));
            ui.hide();
        }
    });

    ui.on_cancel_clicked({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.hide();
        }
    });

    ui.show().unwrap();
}

pub fn edit_dialog(acc_list: &Rc<RefCell<Vec<Account>>>, index: usize, app: &crate::AppWindow) {
    let ui = EditAccDialog::new().unwrap();
    let account = &acc_list.borrow()[index.clone()];
    ui.set_acc_type(account.account_type.borrow().clone().into());
    ui.set_name(account.user_name.borrow().clone().into());
    ui.set_token(account.token.borrow().clone().into());
    ui.set_uuid(account.uuid.borrow().clone().into());

    ui.on_ok_clicked({
        let acc_list_handle = Rc::downgrade(acc_list);
        let app_handle = app.as_weak();
        let ui_handle = ui.as_weak();
        move || {
            let acc_list = acc_list_handle.upgrade().unwrap();
            let app = app_handle.unwrap();
            let ui = ui_handle.unwrap();
            let account = &acc_list.borrow()[index];
            *account.account_type.borrow_mut() = ui.get_acc_type().into();
            *account.token.borrow_mut() = ui.get_token().into();
            *account.user_name.borrow_mut() = ui.get_name().into();
            *account.uuid.borrow_mut() = ui.get_uuid().into();
            save(acc_list.borrow().as_ref());
            app.set_acc_list(ui_acc_list(acc_list.borrow().as_ref()));
            ui.hide();
        }
    });

    ui.on_cancel_clicked({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.hide();
        }
    });

    ui.on_click_del_btn({
        let acc_list_handle = Rc::downgrade(acc_list);
        let app_handle = app.as_weak();
        let ui_handle = ui.as_weak();
        move || {
            let acc_list = acc_list_handle.upgrade().unwrap();
            let app = app_handle.unwrap();
            let ui = ui_handle.unwrap();
            acc_list.borrow_mut().remove(index);
            save(acc_list.borrow().as_ref());
            app.set_acc_list(ui_acc_list(acc_list.borrow().as_ref()));
            ui.hide();
        }
    });

    ui.show().unwrap();
}

pub fn load() -> Option<Vec<Account>> {
    if exists(&"account.json".into()) {
        let mut acc_list: Vec<Account> = Vec::new();
        let json = serde_json::from_str::<Value>(&fs::read_to_string("account.json").ok()?).ok()?;
        for item in json.as_array()? {
            let account = Account {
                account_type: RefCell::from(String::from(item["account_type"].as_str()?)),
                token: RefCell::from(String::from(item["token"].as_str()?)), 
                uuid: RefCell::from(String::from(item["uuid"].as_str()?)),
                user_name: RefCell::from(String::from(item["user_name"].as_str()?)),
            };
            acc_list.push(account);
        }
        Some(acc_list)
    } else {
        let acc_list = vec![
            Account {
                account_type: RefCell::from(String::from("Legacy")),
                token: RefCell::from(String::from("None")),
                uuid: RefCell::from(String::from(uuid::Uuid::new_v4())),
                user_name: RefCell::from(String::from("Steve")),
            },
        ];
        save(&acc_list)?;
        Some(acc_list)
    }
}

fn save(acc_list: &Vec<Account>) -> Option<()> {
    let mut json = serde_json::json!([]);
    for account in acc_list {
        let node = serde_json::json!(
            {
                "account_type": *account.account_type.borrow(),
                "token": *account.token.borrow(),
                "uuid": *account.uuid.borrow(),
                "user_name": *account.user_name.borrow(),
            }
        );
        json.as_array_mut()?.push(node);
    }
    fs::write("account.json", json.to_string()).ok()?;
    Some(())
}
