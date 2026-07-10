//! 多窗口用到的各种Dialog

mod add_game;
mod edit_game;
mod login;
pub mod msgbox;

pub use add_game::add_game_dialog;
pub use edit_game::edit_game_dialog;
pub use login::login_dialog;
