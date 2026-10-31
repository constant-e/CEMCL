//! UI 更新通道的薄封装，供各 manager 向上层发送状态

use frontend::UIUpdate;
use tokio::sync::mpsc::UnboundedSender;

use crate::LauncherError;

#[derive(Clone)]
pub struct UiSender {
    sender: UnboundedSender<UIUpdate>,
}

impl UiSender {
    pub fn new(sender: UnboundedSender<UIUpdate>) -> Self {
        Self { sender }
    }

    pub fn send(&self, update: UIUpdate) -> Result<(), LauncherError> {
        self.sender.send(update)?;
        Ok(())
    }
}
