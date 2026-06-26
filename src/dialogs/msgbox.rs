use slint::ComponentHandle;

use crate::{AskDialog, MsgDialog};
use crate::MsgID as UIMsgID;

#[derive(Clone)]
pub enum MsgID {
    AccNotSelected,             // On launch, not select account
    BothNotSelected,            // On launch, not select account and game
    CopyUserCodeFailed(String), // On login, copy usercode failed, with user code provided
    DelAccConfirm,              // Confirm to delete an account
    DelGameConfirm,             // Confirm to delete a game
    DLFailed(String),           // Download failed, with reason
    GameNotSelected,            // On launch, not select game
    LoadAccFailed(String),      // On init, failed to load account list, with reason
    LoadConfigFailed(String),   // On init, failed to load config, with reason
    LoadGameFailed(String),     // On init, failed to load game list, with reason
    LoginFailed(String),        // On launch, failed to login, with reason
    LaunchFailed(String),       // On launch, failed to launch game, with reason
    OAuthFailed,                // On add account, OAuth Error
    VersionExists,              // On add game, version already exists
    WeakPtrError,               // Failed to upgrade a weak pointer
}

impl From<MsgID> for UIMsgID {
    fn from(value: MsgID) -> Self {
        match value {
            MsgID::AccNotSelected => UIMsgID::AccNotSelected,
            MsgID::BothNotSelected => UIMsgID::BothNotSelected,
            MsgID::CopyUserCodeFailed(_) => UIMsgID::CopyUserCodeFailed, 
            MsgID::DelAccConfirm => UIMsgID::DelAccConfirm,
            MsgID::DelGameConfirm => UIMsgID::DelGameConfirm,
            MsgID::DLFailed(_) => UIMsgID::DLFailed,
            MsgID::GameNotSelected => UIMsgID::GameNotSelected,
            MsgID::LoadAccFailed(_) => UIMsgID::LoadAccFailed,
            MsgID::LoadConfigFailed(_) => UIMsgID::LoadConfigFailed,
            MsgID::LoadGameFailed(_) => UIMsgID::LoadGameFailed,
            MsgID::LoginFailed(_) => UIMsgID::LoginFailed,
            MsgID::LaunchFailed(_) => UIMsgID::LaunchFailed,
            MsgID::OAuthFailed => UIMsgID::OAuthFailed,
            MsgID::VersionExists => UIMsgID::VersionExists,
            MsgID::WeakPtrError => UIMsgID::WeakPtrError,
        }
    }
}

pub fn ask_dialog(id: MsgID, mut on_click_yes: impl FnMut() + 'static) -> Result<(), slint::PlatformError> {
    let dialog = AskDialog::new()?;
    let weak = dialog.as_weak();

    dialog.set_msgid(id.into());
    dialog.on_yes_clicked(move || {
        on_click_yes();
        weak.upgrade().unwrap().hide().unwrap();
    });

    dialog.show()
}

pub fn msg_dialog(id: MsgID) -> Result<(), slint::PlatformError> {
    let dialog = MsgDialog::new()?;

    dialog.set_msgid(id.clone().into());

    match id {
        MsgID::CopyUserCodeFailed(str) |
        MsgID::DLFailed(str) |
        MsgID::LoadAccFailed(str) |
        MsgID::LoadConfigFailed(str) |
        MsgID::LoadGameFailed(str) |
        MsgID::LoginFailed(str) |
        MsgID::LaunchFailed(str) => {
            dialog.set_extra_str(str.into());
        }
        _ => {}
    }

    dialog.show()
}