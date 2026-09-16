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
pub mod avatar;
pub mod bars;
pub mod button;
pub mod canvas;
pub mod checkbox;
pub mod code;
pub mod color_picker;
pub mod combobox;
pub mod control;
pub mod date;
pub mod description_list;
pub mod editor;
pub mod feedback;
pub mod floating;
pub mod focus;
pub mod hover_card;
pub mod icon;
pub mod icons;
pub mod input;
pub mod keys;
pub mod layout;
pub mod list;
pub mod markdown;
pub mod loaders;
pub mod number_input;
pub mod pagination;
pub mod palette;
pub mod popover;
pub mod radio;
pub mod scaffolding;
pub mod scroll;
pub mod search;
pub mod searchable_list;
pub mod segmented;
pub mod slider;
pub mod status;
pub mod table;
pub mod text;
pub mod tree;
pub mod step_indicator;
pub mod surface;
pub mod tooltip;
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
    // **After `surface`, which is what declares `mod.mp = {}`.** A block that writes `mod.mp.X = ...` or `use mod.mp.*`
    // needs the module to exist, and registering before the declaration is the failure the ordering test exists for —
    // except that the test watched `mod.mp.<Name> =` and this block's first line was `use mod.mp.*`, which it did not
    // look at. Both forms are checked now.
    crate::mp::step_indicator::script_mod(vm);
    crate::mp::keys::script_mod(vm);
    crate::mp::layout::script_mod(vm);
    crate::mp::floating::script_mod(vm);
    crate::mp::date::script_mod(vm);
    // The shared animator prototype every control inherits. Before the
    // controls, because their DSL blocks name it.
    crate::mp::control::script_mod(vm);
    crate::mp::bars::script_mod(vm);
    crate::mp::button::script_mod(vm);
    crate::mp::checkbox::script_mod(vm);
    crate::mp::switch::script_mod(vm);
    crate::mp::radio::script_mod(vm);
    crate::mp::slider::script_mod(vm);
    crate::mp::loaders::script_mod(vm);
    crate::mp::input::script_mod(vm);
    crate::mp::icon::script_mod(vm);
    crate::mp::feedback::script_mod(vm);
    crate::mp::status::script_mod(vm);
    crate::mp::scaffolding::script_mod(vm);
    crate::mp::number_input::script_mod(vm);
    crate::mp::pagination::script_mod(vm);
    crate::mp::scroll::script_mod(vm);
    crate::mp::canvas::script_mod(vm);
    crate::mp::searchable_list::script_mod(vm);
    crate::mp::segmented::script_mod(vm);
    crate::mp::search::script_mod(vm);
    crate::mp::table::script_mod(vm);
    // After `status`: the avatar names `mod.mp.StatusTone` for its presence dot,
    // and a type that is not registered yet is 3 runtime errors per use site.
    crate::mp::avatar::script_mod(vm);
    crate::mp::tree::script_mod(vm);
    crate::mp::list::script_mod(vm);
    crate::mp::markdown::script_mod(vm);
    crate::mp::description_list::script_mod(vm);
    crate::mp::editor::script_mod(vm);
    // **After `list`.** `MpPalette` composes an `MpMenu`, which `list.rs` defines,
    // and this crate has now hit that ordering rule twice: a widget that references
    // another widget's prototype in its own `script_mod!` must register later. The
    // first version of this line sat with the other compositions near the top and
    // failed at runtime with "property MpMenu not found in prototype chain" —
    // a message that points at the *user*, not at the line that is in the wrong
    // place.
    crate::mp::palette::script_mod(vm);
    // The overlay family, whose z-order comes from a DrawList2d rather than
    // from where the author put it in the tree.
    crate::mp::popover::script_mod(vm);
    crate::mp::tooltip::script_mod(vm);
    crate::mp::hover_card::script_mod(vm);
    // **Last, because it composes two prototypes that register earlier** — an
    // `MpTextInput` (from `input`) and an `MpPopover` (from `popover`). This is the third
    // time this port has paid for this rule: `palette` before `list` failed on `MpMenu`, and
    // this one failed on both of its dependencies at once, with "property MpTextInput not
    // found in prototype chain" pointing at the *user* of the name rather than at the line
    // that is in the wrong place.
    //
    // The rule, stated as the order it implies: **a widget that names another widget's
    // prototype in its own `script_mod!` registers after it.** The three failures are what
    // makes the ordering a rule rather than a preference, and `tests/registration_order.rs`
    // now checks it statically so a fourth one cannot happen.
    crate::mp::color_picker::script_mod(vm);
    crate::mp::combobox::script_mod(vm);
    crate::mp::code::script_mod(vm);
}
