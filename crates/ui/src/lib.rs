pub use makepad_widgets;
#[cfg(feature = "plot")]
pub use makepad_plot;

pub mod theme;
pub mod widgets;
// a2ui 待 Wave 5 迁移完成后恢复
// pub mod a2ui;

use makepad_widgets::*;

pub fn script_mod(vm: &mut ScriptVm) {
    crate::theme::script_mod(vm);
    crate::widgets::script_mod(vm);
}
