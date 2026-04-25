//! A2UI Surface Widget
//!
//! The A2uiSurface widget is the root container for rendering A2UI component trees.
//! It manages the A2uiMessageProcessor and dynamically renders components.

mod draw_types;
mod helpers;
mod widget;

pub use draw_types::*;
pub use widget::*;

use makepad_widgets::*;

pub fn live_design(cx: &mut Cx) {
    draw_types::live_design(cx);
    helpers::live_design(cx);
    widget::live_design(cx);
}
