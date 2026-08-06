//! Account Page相关

use slint::{ComponentHandle, ModelRc, VecModel};
use std::rc::Rc;
use tokio::sync::mpsc::UnboundedSender;

use crate::app_window::UICommand;
use crate::ui::{self, LoginDialog};

#[derive(Clone)]
pub enum AccountType {
    Legacy,
    MSA,
    Other,
}

impl From<ui::AccountType> for AccountType {
    fn from(value: ui::AccountType) -> Self {
        match value {
            ui::AccountType::Legacy => AccountType::Legacy,
            ui::AccountType::MSA => AccountType::MSA,
            ui::AccountType::Other => AccountType::Other,
        }
    }
}

impl From<AccountType> for ui::AccountType {
    fn from(value: AccountType) -> Self {
        match value {
            AccountType::Legacy => ui::AccountType::Legacy,
            AccountType::MSA => ui::AccountType::MSA,
            AccountType::Other => ui::AccountType::Other,
        }
    }
}

#[derive(Clone)]
pub struct Account {
    pub account_type: AccountType,
    pub token: String,
    pub user_name: String,
    pub uuid: String,
}

impl From<ui::AccountInner> for Account {
    fn from(value: ui::AccountInner) -> Self {
        Self {
            account_type: value.account_type.into(),
            token: value.token.into(),
            user_name: value.user_name.into(),
            uuid: value.uuid.into(),
        }
    }
}

impl From<Account> for ui::AccountInner {
    fn from(value: Account) -> Self {
        Self {
            account_type: value.account_type.into(),
            token: value.token.into(),
            user_name: value.user_name.into(),
            uuid: value.uuid.into(),
        }
    }
}

pub fn ui_acc_list(list: &Vec<Account>) -> ModelRc<ui::AccountInner> {
    ModelRc::from(Rc::from(VecModel::from(
        list.iter()
            .map(|acc| acc.clone().into())
            .collect::<Vec<ui::AccountInner>>(),
    )))
}

pub fn login_dialog(
    tx: UnboundedSender<UICommand>,
) -> Result<slint::Weak<LoginDialog>, slint::PlatformError> {
    let ui = LoginDialog::new()?;
    let ui_weak = ui.as_weak();

    let tx_clone = tx.clone();
    ui.on_msa_clicked(move || {
        tx_clone.send(UICommand::RequestLogin);
    });

    let tx_clone = tx.clone();
    ui.on_msa_ok_clicked(move || {
        tx_clone.send(UICommand::FinishLogin);
    });

    let tx_clone = tx.clone();
    ui.on_offline_ok_clicked(move |user_name, uuid| {
        tx_clone.send(UICommand::AddOfflineAccount(user_name.into(), uuid.into()));
    });

    ui.show()?;
    tx.send(UICommand::GetOfflineAccount).unwrap();
    Ok(ui_weak)
}
