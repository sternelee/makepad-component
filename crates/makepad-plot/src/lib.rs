// Makepad Plot - Matplotlib-style plotting library for Makepad

pub mod elements;
pub mod plot;
pub mod text;

pub use elements::*;
pub use plot::*;
pub use text::*;

use makepad_widgets::*;

pub fn script_mod(vm: &mut ScriptVm) {
    crate::elements::script_mod(vm);
    crate::text::script_mod(vm);
    crate::plot::script_mod(vm);
}
