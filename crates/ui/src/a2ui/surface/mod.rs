//! A2UI Surface Widget
//!
//! The A2uiSurface widget is the root container for rendering A2UI component trees.
//! It manages the A2uiMessageProcessor and dynamically renders components.

mod draw_types;
mod widget;

pub use draw_types::*;
pub use widget::*;

use makepad_widgets::*;

pub fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
    draw_types::script_mod(vm);
    widget::script_mod(vm)
}
