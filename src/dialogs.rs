slint::include_modules!();

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
