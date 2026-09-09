use slint::ComponentHandle;

use crate::ui::{self, AskDialog};

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

    dialog.show()
}
