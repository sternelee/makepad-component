pub mod colors;
pub mod dark;

use makepad_widgets::*;

pub fn script_mod(vm: &mut ScriptVm) {
    crate::theme::colors::script_mod(vm);
    crate::theme::dark::script_mod(vm);
}
