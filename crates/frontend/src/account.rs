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

/// 账号头像：皮肤头部正面的位图（RGBA8）
#[derive(Clone)]
pub struct Avatar {
    pub height: u32,
    pub rgba: Vec<u8>,
    pub width: u32,
}

impl Avatar {
    /// 最近邻缩放到边长 `size`（设备像素）
    ///
    /// 头像位图只有几个像素，直接交给 UI 缩放会糊掉，所以按显示尺寸放大后再交给 UI。
    fn to_image(&self, size: u32) -> slint::Image {
        if self.width == 0
            || self.height == 0
            || self.rgba.len() < (self.width * self.height * 4) as usize
        {
            return slint::Image::default();
        }

        let mut buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(size, size);
        let pixels = buffer.make_mut_bytes();
        for y in 0..size {
            let src_y = (y as u64 * self.height as u64 / size as u64) as usize;
            for x in 0..size {
                let src_x = (x as u64 * self.width as u64 / size as u64) as usize;
                let src = (src_y * self.width as usize + src_x) * 4;
                let dst = ((y * size + x) * 4) as usize;
                pixels[dst..dst + 4].copy_from_slice(&self.rgba[src..src + 4]);
            }
        }

        slint::Image::from_rgba8(buffer)
    }
}

#[derive(Clone)]
pub struct Account {
    pub account_type: AccountType,
    pub avatar: Option<Avatar>,
    pub token: String,
    pub user_name: String,
    pub uuid: String,
}

impl From<ui::AccountInner> for Account {
    fn from(value: ui::AccountInner) -> Self {
        Self {
            account_type: value.account_type.into(),
            // 头像由 App 侧维护，UI 传来的账号不带头像
            avatar: None,
            token: value.token.into(),
            user_name: value.user_name.into(),
            uuid: value.uuid.into(),
        }
    }
}

/// 头像位图的目标边长（设备像素）
pub fn ui_avatar_size(icon_size: f32, scale_factor: f32) -> u32 {
    (icon_size * scale_factor).round().clamp(1.0, 512.0) as u32
}

pub fn ui_acc_list(list: &Vec<Account>, icon_size: u32) -> ModelRc<ui::AccountInner> {
    ModelRc::from(Rc::from(VecModel::from(
        list.iter()
            .map(|acc| ui_account(acc, icon_size))
            .collect::<Vec<ui::AccountInner>>(),
    )))
}

fn ui_account(account: &Account, icon_size: u32) -> ui::AccountInner {
    ui::AccountInner {
        account_type: account.account_type.clone().into(),
        avatar: account
            .avatar
            .as_ref()
            .map(|avatar| avatar.to_image(icon_size))
            .unwrap_or_default(),
        token: account.token.clone().into(),
        user_name: account.user_name.clone().into(),
        uuid: account.uuid.clone().into(),
    }
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
