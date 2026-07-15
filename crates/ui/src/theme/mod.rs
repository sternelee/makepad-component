pub mod colors;

use makepad_widgets::*;

pub fn script_mod(vm: &mut ScriptVm) {
    crate::theme::colors::script_mod(vm);
}
