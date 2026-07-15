pub use makepad_widgets;
#[cfg(feature = "plot")]
pub use makepad_plot;

pub mod a2ui;
pub mod theme;
pub mod widgets;

use makepad_widgets::*;

pub fn script_mod(vm: &mut ScriptVm) {
    crate::theme::script_mod(vm);
    crate::widgets::script_mod(vm);
    #[cfg(feature = "plot")]
    makepad_plot::script_mod(vm);
    crate::a2ui::script_mod(vm);
}
