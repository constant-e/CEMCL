use crate::ui;

pub enum State {
    Spare,
    Downloading,
    Launching,
    LoggingIn,
}

impl From<State> for ui::State {
    fn from(value: State) -> Self {
        match value {
            State::Downloading => ui::State::Downloading,
            State::Launching => ui::State::Launching,
            State::LoggingIn => ui::State::LoggingIn,
            State::Spare => ui::State::Spare,
        }
    }
}
