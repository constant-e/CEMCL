use slint::ComponentHandle;

use crate::ui::{self, AskDialog, MsgBoxDialog};

/// User-facing error categories; the raw detail is passed separately
#[derive(Clone, Copy, Debug)]
pub enum MsgID {
    AccountConfig,
    AuthSession,
    Client,
    Connection,
    DirectoryNotEmpty,
    Download,
    FileAlreadyExists,
    FileBusy,
    FileNotFound,
    GameConfig,
    Interrupted,
    JavaConfig,
    JavaNotFound,
    JavaReleaseInvalid,
    LauncherConfig,
    LoginInvalid,
    Network,
    PermissionDenied,
    Request,
    InternalChannel,
    InternalSync,
    OutOfRange,
    TaskSetNotFound,
    Unknown,
}

fn ui_msg(id: MsgID) -> ui::MsgID {
    match id {
        MsgID::AccountConfig => ui::MsgID::AccountConfig,
        MsgID::AuthSession => ui::MsgID::AuthSession,
        MsgID::Client => ui::MsgID::Client,
        MsgID::Connection => ui::MsgID::Connection,
        MsgID::DirectoryNotEmpty => ui::MsgID::DirectoryNotEmpty,
        MsgID::Download => ui::MsgID::Download,
        MsgID::FileAlreadyExists => ui::MsgID::FileAlreadyExists,
        MsgID::FileBusy => ui::MsgID::FileBusy,
        MsgID::FileNotFound => ui::MsgID::FileNotFound,
        MsgID::GameConfig => ui::MsgID::GameConfig,
        MsgID::Interrupted => ui::MsgID::Interrupted,
        MsgID::JavaConfig => ui::MsgID::JavaConfig,
        MsgID::JavaNotFound => ui::MsgID::JavaNotFound,
        MsgID::JavaReleaseInvalid => ui::MsgID::JavaReleaseInvalid,
        MsgID::LauncherConfig => ui::MsgID::LauncherConfig,
        MsgID::LoginInvalid => ui::MsgID::LoginInvalid,
        MsgID::Network => ui::MsgID::Network,
        MsgID::PermissionDenied => ui::MsgID::PermissionDenied,
        MsgID::Request => ui::MsgID::Request,
        MsgID::InternalChannel => ui::MsgID::InternalChannel,
        MsgID::InternalSync => ui::MsgID::InternalSync,
        MsgID::OutOfRange => ui::MsgID::OutOfRange,
        MsgID::TaskSetNotFound => ui::MsgID::TaskSetNotFound,
        MsgID::Unknown => ui::MsgID::Unknown,
    }
}

pub(crate) fn set_msg(dialog: &MsgBoxDialog, id: MsgID, detail: impl Into<slint::SharedString>) {
    dialog.set_msgid(ui_msg(id));
    dialog.set_detail(detail.into());
}

pub enum AskID {
    DelAccConfirm,  // Confirm to delete an account
    DelGameConfirm, // Confirm to delete a game
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

    let ui_weak = dialog.as_weak();
    dialog.on_drag_window(move || {
        if let Some(dialog) = ui_weak.upgrade() {
            ui::drag_window(&dialog);
        }
    });

    dialog.show()
}

/// 显示本地化的错误对话框，返回其弱引用以便后续复用（避免连续错误叠出多个窗口）
pub fn msg_box(
    id: MsgID,
    detail: impl Into<slint::SharedString>,
) -> Result<slint::Weak<MsgBoxDialog>, slint::PlatformError> {
    let dialog = MsgBoxDialog::new()?;
    set_msg(&dialog, id, detail);

    let ui_weak = dialog.as_weak();
    dialog.on_drag_window(move || {
        if let Some(dialog) = ui_weak.upgrade() {
            ui::drag_window(&dialog);
        }
    });

    dialog.show()?;
    Ok(dialog.as_weak())
}
