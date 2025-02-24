//! 多窗口用到的各种Dialog

mod add_acc;
mod add_game;
mod edit_acc;
mod edit_game;
pub mod msg_box;

pub use add_acc::add_acc_dialog;
pub use add_game::add_game_dialog;
pub use edit_acc::edit_acc_dialog;
pub use edit_game::edit_game_dialog;
