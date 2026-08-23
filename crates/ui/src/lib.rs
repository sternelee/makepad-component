pub use makepad_motion;
pub use makepad_theme;

pub use makepad_widgets;

pub mod a2ui;
pub mod widgets;

use makepad_widgets::*;

pub fn script_mod(vm: &mut ScriptVm) {
    makepad_theme::script_mod(vm);
    crate::widgets::script_mod(vm);
    #[cfg(feature = "plot")]
    makepad_plot::script_mod(vm);
    crate::a2ui::script_mod(vm);
}
