use crate::{AskDialog, WarnDialog, ErrorDialog};
use slint::ComponentHandle;

pub fn ask_dialog(title: &str, msg: &str, mut on_yes: impl FnMut() -> () + 'static) {
    let dialog = AskDialog::new().unwrap();
    dialog.set_title_text(title.into());
    dialog.set_msg(msg.into());

    dialog.on_yes_clicked({
        let dialog_handle = dialog.as_weak();
        move || {
            let dialog = dialog_handle.unwrap();
            on_yes();
            dialog.hide().unwrap();
        }
    });

    dialog.on_no_clicked({
        let dialog_handle = dialog.as_weak();
        move || {
            let dialog = dialog_handle.unwrap();
            dialog.hide().unwrap();
        }
    });

    dialog.show().unwrap();
}

pub fn warn_dialog(msg: &str) {
    let dialog = WarnDialog::new().unwrap();
    dialog.set_msg(msg.into());
    dialog.on_ok_clicked({
        let dialog_handle = dialog.as_weak();
        move || {
            let dialog = dialog_handle.unwrap();
            dialog.hide().unwrap();
        }
    });
    dialog.show().unwrap();
}

pub fn err_dialog(msg: &str) {
    let dialog = ErrorDialog::new().unwrap();
    dialog.set_msg(msg.into());
    dialog.on_ok_clicked({
        let dialog_handle = dialog.as_weak();
        move || {
            let dialog = dialog_handle.unwrap();
            dialog.hide().unwrap();
        }
    });
    dialog.show().unwrap();
}
