//! 多窗口用到的各种Dialog

mod login;
mod add_game;
mod edit_game;
pub mod msgbox;

pub use login::login_dialog;
pub use add_game::add_game_dialog;
pub use edit_game::edit_game_dialog;
