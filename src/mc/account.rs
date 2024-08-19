use std::cell::RefCell;
use std::fs;
use serde_json::Value;
use crate::file_tools::exists;
use super::Account;

slint::include_modules!();

pub fn add_dialog() {
    let ui = AddAccDialog::new().unwrap();
    
    ui.on_click_ok_btn({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            // TODO: Save changes
            ui.hide();
        }
    });

    ui.on_click_cancel_btn({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
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
