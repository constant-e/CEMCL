//! 账号管理：账号列表、登录/刷新、头像以及对应的 UI 状态

use clipboard::{ClipboardContext, ClipboardProvider};
use frontend::UIUpdate;
use log::info;
use mc::account::{
    Account, AccountType,
    auth::{AuthPollAction, AuthSession, request_oauth, request_refresh_account},
    skin,
};
use serde_json::json;
use std::fs::{self, exists};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{avatar::AvatarManager, errors::LauncherError, ui::UiSender};

pub struct AccountManager {
    account_list: Vec<Account>,
    avatars: AvatarManager,
    current_index: u32,
    current_session: Option<AuthSession>,
    ui: UiSender,
}

impl AccountManager {
    pub fn new(ui: UiSender) -> Result<Self, LauncherError> {
        let (account_list, current_index) = AccountManager::i_load()?;
        Ok(Self {
            account_list,
            avatars: AvatarManager::new(),
            current_index,
            current_session: None,
            ui,
        })
    }

    /// 启动时加载头像并刷新账号列表
    pub fn init(&mut self) -> Result<(), LauncherError> {
        // 先给出缓存或默认头像，正版账号的皮肤在后台更新
        self.avatars.load(&self.account_list);
        self.refresh_ui_acc_list()
    }

    pub fn take_avatar_receiver(&mut self) -> Option<UnboundedReceiver<(String, skin::Avatar)>> {
        self.avatars.take_update_receiver()
    }

    /// 后台头像更新完成
    pub fn handle_avatar_update(
        &mut self,
        uuid: String,
        avatar: skin::Avatar,
    ) -> Result<(), LauncherError> {
        self.avatars.insert(uuid, avatar);
        self.refresh_ui_acc_list()
    }

    pub fn add_offline(&mut self, user_name: String, uuid: String) -> Result<(), LauncherError> {
        let account = Account {
            access_token: String::new(),
            account_type: AccountType::Legacy,
            refresh_token: String::new(),
            uuid,
            user_name,
        };
        self.add(account.clone())?;
        self.avatars.load_account(&account);
        self.refresh_ui_acc_list()?;
        self.ui.send(UIUpdate::QuitLoginDialog)
    }

    pub fn del_from_ui(&mut self, index: u32) -> Result<(), LauncherError> {
        let uuid = self.get(index).uuid.clone();
        self.del(index)?;
        self.avatars.remove(&uuid);
        self.refresh_ui_acc_list()
    }

    pub fn edit_from_ui(
        &mut self,
        index: u32,
        account: frontend::account::Account,
    ) -> Result<(), LauncherError> {
        let mut i_account = self.get(index).clone();
        let old_uuid = i_account.uuid.clone();
        i_account.account_type = to_account_type(account.account_type);
        i_account.refresh_token = account.token;
        i_account.user_name = account.user_name;
        i_account.uuid = account.uuid;
        self.edit(index, i_account.clone())?;
        self.reload_avatar(&old_uuid, &i_account);
        self.refresh_ui_acc_list()
    }

    pub async fn finish_login(&mut self) -> Result<(), LauncherError> {
        let mut session = self
            .take_auth_session()
            .ok_or(LauncherError::AuthSessionNotFound)?;
        while let Ok(action) = session.poll().await {
            match action {
                AuthPollAction::Continue(step) => info!("Step {step} / 5"),
                AuthPollAction::Done(account) => {
                    info!("Step 5 / 5");
                    self.add(account.clone())?;
                    self.avatars.load_account(&account);
                    self.refresh_ui_acc_list()?;
                    self.ui.send(UIUpdate::QuitLoginDialog)?;
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn get_offline_account(&self) -> Result<(), LauncherError> {
        self.ui.send(UIUpdate::SetOfflineAccount(frontend_account(
            Account::default(),
            None,
        )))
    }

    pub async fn request_login(&mut self) -> Result<(), LauncherError> {
        if let Some(session) = self.current_session.take() {
            drop(session);
        }

        let (uri, code, session) = request_oauth().await?;
        self.current_session = Some(session);

        let mut ctx: ClipboardContext = ClipboardProvider::new()?;
        ctx.set_contents(code)?;
        webbrowser::open(&uri)?;
        Ok(())
    }

    /// 启动前刷新正版账号的令牌，并把登录进度发给主页
    pub async fn refresh_for_launch(&mut self, index: u32) -> Result<(), LauncherError> {
        if !self.request_refresh_account(index).await? {
            return Ok(());
        }

        self.ui
            .send(UIUpdate::SetHomePageProgress(1.0 / 5.0, 1, 5))?;
        let mut session = self
            .take_auth_session()
            .ok_or(LauncherError::AuthSessionNotFound)?;

        let mut action = session.poll().await?;
        loop {
            match action {
                AuthPollAction::Continue(step) => {
                    self.ui.send(UIUpdate::SetHomePageProgress(
                        step as f32 / 5.0,
                        step as u32,
                        5,
                    ))?;
                }
                AuthPollAction::Done(account) => {
                    self.ui.send(UIUpdate::SetHomePageProgress(1.0, 5, 5))?;
                    let old_uuid = self.get(index).uuid.clone();
                    self.reload_avatar(&old_uuid, &account);
                    self.edit(index, account)?;
                    self.refresh_ui_acc_list()?;
                    break;
                }
            }
            action = session.poll().await?;
        }
        Ok(())
    }

    pub fn switch_current(&mut self, index: u32) -> Result<(), LauncherError> {
        self.set_current_index(index)
    }

    pub fn get(&self, index: u32) -> &Account {
        &self.account_list[index as usize]
    }

    pub fn get_account_list(&self) -> &Vec<Account> {
        &self.account_list
    }

    fn add(&mut self, account: Account) -> Result<(), LauncherError> {
        self.account_list.push(account);
        self.save()
    }

    fn del(&mut self, index: u32) -> Result<(), LauncherError> {
        self.account_list.remove(index as usize);

        // if index = self.current_index, then switch to another account.
        if self.current_index != 0 && index >= self.current_index {
            self.current_index -= 1;
        }

        self.save()
    }

    fn edit(&mut self, index: u32, account: Account) -> Result<(), LauncherError> {
        self.account_list[index as usize] = account;

        self.save()
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

    /// return Ok(false) if account isn't msa
    async fn request_refresh_account(&mut self, index: u32) -> Result<bool, LauncherError> {
        if let Some(session) = self.current_session.take() {
            drop(session);
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

    /// 账号编辑（可能改了 uuid）后刷新头像
    fn reload_avatar(&mut self, old_uuid: &str, account: &Account) {
        if old_uuid != account.uuid {
            self.avatars.remove(old_uuid);
        }
        self.avatars.load_account(account);
    }

    fn refresh_ui_acc_list(&self) -> Result<(), LauncherError> {
        self.ui.send(UIUpdate::SetAccountList(
            self.account_list
                .iter()
                .map(|v| frontend_account(v.clone(), Some(self.avatars.get(v))))
                .collect(),
        ))?;
        self.ui.send(UIUpdate::SetAccountIndex(self.current_index))
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

pub fn frontend_account(
    account: Account,
    avatar: Option<skin::Avatar>,
) -> frontend::account::Account {
    frontend::account::Account {
        account_type: frontend_account_type(account.account_type),
        avatar: avatar.map(|avatar| frontend::account::Avatar {
            height: skin::Avatar::SIZE,
            rgba: avatar.into_rgba(),
            width: skin::Avatar::SIZE,
        }),
        token: account.refresh_token,
        user_name: account.user_name,
        uuid: account.uuid,
    }
}
