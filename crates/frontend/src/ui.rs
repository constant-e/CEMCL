//! Slint UI
//! Slint生成的UI数据类型会被储存在frontend::ui下
slint::include_modules!();

use slint::platform::{PointerEventButton, WindowEvent};
use slint::winit_030::WinitWindowAccessor;
use slint::{ComponentHandle, LogicalPosition};

/// 拖动无边框窗口：把移动交给窗口管理器处理（Wayland 下只能这样移动窗口）。
/// 非 winit 后端时为空操作。
pub(crate) fn drag_window(ui: &(impl ComponentHandle + 'static)) {
    let dragged = ui
        .window()
        .with_winit_window(|winit_window| match winit_window.drag_window() {
            Ok(()) => true,
            Err(e) => {
                log::error!("{e}");
                false
            }
        });
    if dragged != Some(true) {
        return;
    }

    // 交互移动期间指针被窗口管理器接管，抬起事件不会送达应用：Slint 会一直认为指针
    // 处于按下并把后续点击都路由给标题栏的拖动区（表现为"点什么都在拖窗口"）。
    // 因此在事件循环里补发一次窗口外的抬起，复位 Slint 的输入状态。
    let weak = ui.as_weak();
    if let Err(e) = slint::invoke_from_event_loop(move || {
        if let Some(ui) = weak.upgrade() {
            ui.window().dispatch_event(WindowEvent::PointerReleased {
                position: LogicalPosition::new(-1.0, -1.0),
                button: PointerEventButton::Left,
            });
        }
    }) {
        log::debug!("{e}");
    }
}
