use log::error;
use serde_json::json;
use std::fs::{self, exists};

use crate::errors::LauncherError;
use mc::account::{
    Account, AccountType,
    auth::{AuthSession, request_oauth, request_refresh_account},
};

pub struct AccountManager {
    account_list: Vec<Account>,
    current_index: u32,
    current_session: Option<AuthSession>,
}

impl AccountManager {
    pub fn new() -> Result<Self, LauncherError> {
        let (account_list, current_index) = AccountManager::i_load()?;
        Ok(Self {
            account_list,
            current_index,
            current_session: None,
        })
    }

    pub fn add(&mut self, account: Account) -> Result<(), LauncherError> {
        self.account_list.push(account);
        self.save()
    }

    pub fn del(&mut self, index: u32) -> Result<(), LauncherError> {
        self.account_list.remove(index as usize);

        // if index = self.current_index, then switch to another account.
        if self.current_index != 0 && index >= self.current_index {
            self.current_index -= 1;
        }

        self.save()
    }

    pub fn edit(&mut self, index: u32, account: Account) -> Result<(), LauncherError> {
        self.account_list[index as usize] = account;

        self.save()
    }

    pub fn get(&self, index: u32) -> &Account {
        &self.account_list[index as usize]
    }

    pub fn get_mut(&mut self, index: u32) -> &mut Account {
        &mut self.account_list[index as usize]
    }

    pub fn get_account_list(&self) -> &Vec<Account> {
        &self.account_list
    }

    pub fn get_current_index(&self) -> u32 {
        self.current_index
    }

    fn i_load() -> Result<(Vec<Account>, u32), LauncherError> {
        if !exists("account.json")? {
            return Ok((vec![Account::default()], 0));
        }

        let json = serde_json::from_str::<serde_json::Value>(&fs::read_to_string("account.json")?)?;
        let mut account_list = Vec::new();
        if let Some(array) = json["accounts"].as_array() {
            for item in array {
                let account_type = match item["account_type"]
                    .as_str()
                    .ok_or(LauncherError::AccountConfigError)?
                {
                    "Legacy" => AccountType::Legacy,
                    "msa" => AccountType::MSA,
                    _ => AccountType::Other,
                };

                let account = Account {
                    access_token: String::new(),
                    account_type,
                    refresh_token: item["token"]
                        .as_str()
                        .ok_or(LauncherError::AccountConfigError)?
                        .to_string(),
                    uuid: item["uuid"]
                        .as_str()
                        .ok_or(LauncherError::AccountConfigError)?
                        .to_string(),
                    user_name: item["user_name"]
                        .as_str()
                        .ok_or(LauncherError::AccountConfigError)?
                        .to_string(),
                };

                account_list.push(account);
            }
        } else {
            return Err(LauncherError::AccountConfigError);
        }

        let current_index = json["current"]
            .as_i64()
            .ok_or(LauncherError::AccountConfigError)? as u32;

        Ok((account_list, current_index))
    }

    pub fn reload(&mut self) -> Result<(), LauncherError> {
        let (account_list, current_index) = AccountManager::i_load()?;
        self.account_list = account_list;
        self.current_index = current_index;
        Ok(())
    }

    /// return (uri, code)
    pub async fn request_login(&mut self) -> Result<(String, String), LauncherError> {
        if self.current_session.is_some() {
            drop(self.current_session.take().unwrap());
        }

        let (uri, code, session) = request_oauth().await?;
        self.current_session = Some(session);

        Ok((uri, code))
    }

    /// return Ok(false) if account isn't msa
    pub async fn request_refresh_account(&mut self, index: u32) -> Result<bool, LauncherError> {
        if self.current_session.is_some() {
            drop(self.current_session.take().unwrap());
        }

        let account = &mut self.account_list[index as usize];
        if account.account_type != AccountType::MSA {
            return Ok(false);
        }

        let session = request_refresh_account(&account.refresh_token).await?;
        self.current_session = Some(session);

        Ok(true)
    }

    pub fn save(&self) -> Result<(), LauncherError> {
        let mut json = json!(
            {
                "current": self.current_index,
                "accounts": []
            }
        );

        for account in &self.account_list {
            let node = serde_json::json!(
                {
                    "account_type": String::from(account.account_type.clone()),
                    "token": account.refresh_token,
                    "uuid": account.uuid,
                    "user_name": account.user_name,
                }
            );
            if let Some(array) = json["accounts"].as_array_mut() {
                array.push(node);
            } else {
                error!("");
            }
        }
        fs::write("account.json", json.to_string())?;
        Ok(())
    }

    pub fn set_current_index(&mut self, index: u32) -> Result<(), LauncherError> {
        self.current_index = index;
        self.save()
    }

    pub fn take_auth_session(&mut self) -> Option<AuthSession> {
        self.current_session.take()
    }
}

pub fn to_account_type(account_type: frontend::account::AccountType) -> AccountType {
    match account_type {
        frontend::account::AccountType::Legacy => AccountType::Legacy,
        frontend::account::AccountType::MSA => AccountType::MSA,
        frontend::account::AccountType::Other => AccountType::Other,
    }
}

pub fn frontend_account_type(account_type: AccountType) -> frontend::account::AccountType {
    match account_type {
        AccountType::Legacy => frontend::account::AccountType::Legacy,
        AccountType::MSA => frontend::account::AccountType::MSA,
        AccountType::Other => frontend::account::AccountType::Other,
    }
}

pub fn frontend_account(account: Account) -> frontend::account::Account {
    frontend::account::Account {
        account_type: frontend_account_type(account.account_type),
        token: account.refresh_token,
        user_name: account.user_name,
        uuid: account.uuid,
    }
}
