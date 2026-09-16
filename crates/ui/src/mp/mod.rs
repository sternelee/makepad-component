//! The v3 component set.
//!
//! Everything here is built to the five laws in
//! `docs/superpowers/specs/2026-09-16-bezel-mp-architecture.md`:
//!
//! 1. **Style flows through the environment.** A widget reads
//!    [`Theme::of(cx)`](makepad_theme::Theme::of) in `draw_walk` and writes the
//!    values it needs into its own shader instances. No colour, font or size
//!    travels as a parameter. A caller overrides by setting a DSL field.
//! 2. **SwiftUI vocabulary.** A closed enum selects between shipped looks;
//!    free-form radius, colour and padding never become API.
//! 3. **Motion is named.** Durations and curves come from
//!    [`makepad_motion`], never from a literal at the call site.
//! 4. **Numbers drive layout, colours are paint.**
//! 5. **Measured, and dated.** A number reaches the theme when the platform
//!    named it.
//!
//! ## The shape of a widget here
//!
//! ```text
//! mp/button.rs
//!   script_mod! { ... }            the DSL: the shader, the base type, the
//!                                  named variants a caller writes
//!   enum MpButtonStyle             the closed set of shipped looks (Script)
//!   struct DrawMpButton            the shader: state mixes + resolved paint
//!   struct MpButton                the widget: handle_event, draw_walk
//!   impl MpButtonRef               the typed handle an app holds
//! ```
//!
//! Two rules the v2 set broke and this one does not:
//!
//! - **No palette in the shader.** Sixteen instance fields existed only so Rust
//!   could read tokens back out of the draw call, because the palette was baked
//!   at script-apply time. Reading `Theme::of(cx)` costs nothing and means an
//!   appearance change needs a redraw rather than a whole-heap script
//!   re-apply.
//! - **Never `find_widget_action(uid).cast()`.** See [`action`].

pub mod action;
pub mod button;
pub mod checkbox;
pub mod control;
pub mod radio;
pub mod surface;
pub mod switch;

use makepad_widgets::*;

/// Register every v3 widget, dependencies first.
///
/// Called by [`crate::script_mod`] after the theme and motion namespaces are
/// up, because a widget's DSL block names them.
pub fn script_mod(vm: &mut ScriptVm) {
    // Surfaces first: every container and several leaf widgets paint one,
    // and a widget's DSL block names the prototype it inherits.
    crate::mp::surface::script_mod(vm);
    // The shared animator prototype every control inherits. Before the
    // controls, because their DSL blocks name it.
    crate::mp::control::script_mod(vm);
    crate::mp::button::script_mod(vm);
    crate::mp::checkbox::script_mod(vm);
    crate::mp::switch::script_mod(vm);
    crate::mp::radio::script_mod(vm);
}
