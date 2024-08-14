slint::include_modules!();

pub fn init() {
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