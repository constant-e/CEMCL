use slint::ComponentHandle;

use crate::ui::{self, AskDialog, MsgDialog};

pub enum MsgID {
    AccNotSelected,             // On launch, not select account
    BothNotSelected,            // On launch, not select account and game
    CopyUserCodeFailed(String), // On login, copy usercode failed, with user code provided
    DLFailed(String),           // Download failed, with reason
    GameNotSelected,            // On launch, not select game
    LoadAccFailed(String),      // On init, failed to load account list, with reason
    LoadConfigFailed(String),   // On init, failed to load config, with reason
    LoadGameFailed(String),     // On init, failed to load game list, with reason
    LoginFailed,                // On launch, failed to login, with reason
    LaunchFailed,               // On launch, failed to launch game, with reason
    OAuthFailed,                // On add account, OAuth Error, with reason
    VersionExists,              // On add game, version already exists
    WeakPtrError,               // Failed to upgrade a weak pointer
}

pub enum AskID {
    DelAccConfirm,  // Confirm to delete an account
    DelGameConfirm, // Confirm to delete a game
}

fn ui_msg(id: MsgID) -> (ui::MsgID, Option<String>) {
    match id {
        MsgID::AccNotSelected => (ui::MsgID::AccNotSelected, None),
        MsgID::BothNotSelected => (ui::MsgID::BothNotSelected, None),
        MsgID::CopyUserCodeFailed(s) => (ui::MsgID::CopyUserCodeFailed, Some(s)),
        MsgID::DLFailed(s) => (ui::MsgID::DLFailed, Some(s)),
        MsgID::GameNotSelected => (ui::MsgID::GameNotSelected, None),
        MsgID::LoadAccFailed(s) => (ui::MsgID::LoadAccFailed, Some(s)),
        MsgID::LoadConfigFailed(s) => (ui::MsgID::LoadConfigFailed, Some(s)),
        MsgID::LoadGameFailed(s) => (ui::MsgID::LoadGameFailed, Some(s)),
        MsgID::LoginFailed => (ui::MsgID::LoginFailed, None),
        MsgID::LaunchFailed => (ui::MsgID::LaunchFailed, None),
        MsgID::OAuthFailed => (ui::MsgID::OAuthFailed, None),
        MsgID::VersionExists => (ui::MsgID::VersionExists, None),
        MsgID::WeakPtrError => (ui::MsgID::WeakPtrError, None),
    }
}

fn ui_ask(id: AskID) -> (ui::AskID, Option<String>) {
    match id {
        AskID::DelAccConfirm => (ui::AskID::DelAccConfirm, None),
        AskID::DelGameConfirm => (ui::AskID::DelGameConfirm, None),
    }
}

pub fn ask_box<F>(id: AskID, on_yes: F) -> Result<(), slint::PlatformError>
where
    F: Fn() + 'static,
{
    let dialog = AskDialog::new()?;
    let (ui_id, extra_str) = ui_ask(id);
    dialog.set_msgid(ui_id);
    if let Some(s) = extra_str {
        dialog.set_extra_str(s.into());
    }
    dialog.on_yes_clicked(on_yes);

    dialog.show()
}

pub fn msg_box(id: MsgID) -> Result<(), slint::PlatformError> {
    let dialog = MsgDialog::new()?;
    let (ui_id, extra_str) = ui_msg(id);
    dialog.set_msgid(ui_id);
    if let Some(s) = extra_str {
        dialog.set_extra_str(s.into());
    }

    dialog.show()
}
