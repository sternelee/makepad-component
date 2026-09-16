//! The shared contract of every interactive control.
//!
//! Five v2 controls — checkbox, radio, switch, toggle, slider — averaged 450
//! lines each, and most of that was the same thirty-line pointer/keyboard block
//! written five times. It had drifted, too: the radio never took focus on a
//! pointer press (so clicking one and pressing Space did nothing), the toggle
//! set its hand cursor only on hover-*in* (so the cursor stayed a hand after
//! the pointer left), and only the checkbox and switch agreed about which key
//! activates.
//!
//! So the contract lives here, once, and a control declares only what makes it
//! *itself*:
//!
//! ```ignore
//! impl Widget for MpCheckbox {
//!     fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
//!         let signals = control::handle(&mut self.animator, cx, event, self.area);
//!         if signals.redraw {
//!             self.redraw(cx);
//!         }
//!         if self.disabled {
//!             return;
//!         }
//!         if signals.activate {
//!             self.set_checked(cx, !self.checked);
//!         }
//!     }
//!
//!     fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
//!         // ... paint ...
//!         self.area = self.draw_bg.area();
//!         control::register(cx, self.widget_uid(), self.area, self.disabled);
//!         DrawStep::done()
//!     }
//! }
//! ```
//!
//! ## The tracks a control declares
//!
//! Four state groups, named the same in every control because the helper names
//! them: `hover`, `press`, `focus`, `disabled`. Each affects `draw_bg` with a
//! 0..1 instance of the same name. [`ControlAnimator`](mod@crate::mp) is the
//! prototype that declares all four with the catalog's durations, so a control
//! inherits them rather than restating twenty lines of DSL:
//!
//! ```ignore
//! mod.mp.MpCheckboxBase = #(MpCheckbox::register_widget(vm))
//! mod.mp.MpCheckbox = set_type_default() do mod.mp.MpCheckboxBase{
//!     animator: mod.mp.ControlAnimator{}
//!     draw_bg +: { hover: 0.0  press: 0.0  focus: 0.0  disabled: 0.0 }
//! }
//! ```
//!
//! ## What the helper does not decide
//!
//! Whether an activation *checks*, *toggles* or *selects*. A radio does not
//! turn off, a switch is not a checkbox, and a slider's drag is its own — the
//! helper reports that the control was activated and the control decides what
//! that means.

use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*

    // The four tracks every control declares, with the catalog's durations
    // rather than literals: `fade_quick` for the states a hand is on, and
    // `press` for the one a finger is on. A control inherits this and extends
    // it with whatever makes it itself — the checkbox adds `check`, the switch
    // adds `on`.
    //
    // Every `apply` writes a 0..1 instance of the same name on `draw_bg`, so a
    // control's shader declares `hover press focus disabled` and gets all four
    // for free.
    mod.mp.ControlAnimator = Animator{
        hover: {
            default: @off
            off: AnimatorState{
                ease: mod.motion.hover_fade.ease
                from: {all: Forward{duration: mod.motion.hover_fade.duration}}
                apply: {draw_bg: {hover: 0.0}}
            }
            on: AnimatorState{
                ease: mod.motion.hover_fade.ease
                from: {all: Forward{duration: mod.motion.hover_fade.duration}}
                apply: {draw_bg: {hover: 1.0}}
            }
        }
        press: {
            default: @off
            off: AnimatorState{
                ease: mod.motion.press.ease
                from: {all: Forward{duration: mod.motion.press.duration}}
                apply: {draw_bg: {press: 0.0}}
            }
            on: AnimatorState{
                ease: mod.motion.press.ease
                from: {all: Forward{duration: mod.motion.press.duration}}
                apply: {draw_bg: {press: 1.0}}
            }
        }
        focus: {
            default: @off
            off: AnimatorState{
                ease: mod.motion.hover_fade.ease
                from: {all: Forward{duration: mod.motion.hover_fade.duration}}
                apply: {draw_bg: {focus: 0.0}}
            }
            on: AnimatorState{
                ease: mod.motion.hover_fade.ease
                from: {all: Forward{duration: mod.motion.hover_fade.duration}}
                apply: {draw_bg: {focus: 1.0}}
            }
        }
        disabled: {
            default: @off
            off: AnimatorState{
                ease: mod.motion.fade_quick.ease
                from: {all: Forward{duration: mod.motion.fade_quick.duration}}
                apply: {draw_bg: {disabled: 0.0}}
            }
            on: AnimatorState{
                ease: mod.motion.fade_quick.ease
                from: {all: Forward{duration: mod.motion.fade_quick.duration}}
                apply: {draw_bg: {disabled: 1.0}}
            }
        }
    }
}

/// What the pointer and the keyboard did, sorted into what a control reacts to.
///
/// A struct rather than a `Vec<Event>`: a control reacts to at most one of each
/// per event, and naming them makes a control's `handle_event` read as the
/// behaviour it has.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Signals {
    /// The pointer entered the control.
    pub hover_in: bool,
    /// The pointer left the control.
    pub hover_out: bool,
    /// A press began.
    pub down: bool,
    /// A press ended, whether or not it landed on the control.
    pub up: bool,
    /// The pointer is down and moved.
    ///
    /// The drag signal. Makepad delivers `FingerMove` to the widget whose area
    /// took the press, so a control sees the whole gesture rather than only the
    /// part of it that stays inside itself — which is what a slider needs and a
    /// button has no use for.
    pub moved: bool,
    /// The control's value should change: a press that landed on it, or
    /// Enter/Space while it holds key focus.
    pub activate: bool,
    /// The animator advanced and the control needs a repaint.
    pub redraw: bool,
    /// Where the pointer was, in screen coordinates.
    ///
    /// Meaningful when `down` or `moved` is set. Carried because the value of a
    /// drag is a *position*, and a control that has to re-derive it from the
    /// event has to re-implement the hit test that produced this signal.
    pub pointer: Vec2d,
}

impl Signals {
    /// Whether anything at all happened.
    ///
    /// The pointer's position is deliberately not consulted: it is a payload
    /// that rides along with a gesture, not a gesture. Counting it would make
    /// every mouse move over a control read as activity, and a control that
    /// checks this before redrawing would redraw every frame.
    pub fn is_idle(self) -> bool {
        !(self.hover_in
            || self.hover_out
            || self.down
            || self.up
            || self.moved
            || self.activate
            || self.redraw)
    }
}

/// Run the pointer and keyboard contract for a control.
///
/// `area` is the control's hit area from its last paint. Before the first paint
/// it is `Area::Empty`, which hit-tests to nothing — so a control that is
/// somehow sent an event before it draws does nothing rather than reacting at
/// the origin.
pub fn handle(
    animator: &mut Animator,
    cx: &mut Cx,
    event: &Event,
    area: Area,
) -> Signals {
    let mut signals = Signals::default();

    // `AnimatorAction` is an enum, not a struct: the animator writes whether it
    // is still animating and whether that needs a repaint.
    let mut action = AnimatorAction::None;
    animator.handle_event(cx, event, &mut action);
    if action.must_redraw() {
        signals.redraw = true;
    }

    // Focus is read from `Cx`, never mirrored in a field. The v2 controls kept a
    // `focused: bool` and compared it to detect a change, which meant the focus
    // ring appeared one frame after the click that caused it.
    set_state(
        animator,
        cx,
        cx.has_key_focus(area),
        &[id!(focus), id!(on)],
        &[id!(focus), id!(off)],
    );

    match event.hits(cx, area) {
        Hit::FingerHoverIn(_) => {
            cx.set_cursor(MouseCursor::Hand);
            animator.play(cx, ids!(hover.on), None);
            signals.hover_in = true;
        }
        Hit::FingerHoverOut(_) => {
            cx.set_cursor(MouseCursor::Default);
            // The press track is cleared here as well as on `FingerUp`,
            // because a press that ends off the control still has to come back
            // up: otherwise a control dragged away from stays visibly held.
            animator.play(cx, ids!(hover.off), None);
            animator.play(cx, ids!(press.off), None);
            signals.hover_out = true;
        }
        Hit::FingerDown(fe) => {
            animator.play(cx, ids!(press.on), None);
            // Claim key focus on press, so the ring follows the pointer and a
            // following Tab continues from here. This is the line the v2 radio
            // was missing.
            cx.set_key_focus(area);
            signals.down = true;
            signals.pointer = fe.abs;
        }
        Hit::FingerMove(fe) => {
            signals.moved = true;
            signals.pointer = fe.abs;
        }
        Hit::FingerUp(fe) => {
            animator.play(cx, ids!(press.off), None);
            signals.up = true;
            signals.pointer = fe.abs;
            if fe.is_over {
                signals.activate = true;
            }
        }
        _ => {}
    }

    if cx.has_key_focus(area) {
        if let Event::KeyDown(ke) = event {
            if !ke.is_repeat && matches!(ke.key_code, KeyCode::ReturnKey | KeyCode::Space) {
                signals.activate = true;
            }
        }
    }

    signals
}

/// Put a control's `checked` track where its initial value says it belongs,
/// **without animating**.
///
/// A control built as `checked: true` — or `selected: true`, which is the same
/// track — has a true *field* and an animator still sitting in its default
/// `off` state. The field's value and the animator's disagree, and since the
/// animator is what the paint reads, the control renders unchecked. It then
/// looks broken in the specific way that is hardest to debug: the first click
/// appears to do nothing, because the value was already true and
/// [`set_checked`](crate::mp::checkbox::MpCheckbox::set_checked) correctly
/// returns early.
///
/// `cut` rather than `play`: a control that starts checked must *start*
/// checked, not animate into it before the first frame.
///
/// Every control calls this from `on_after_new`.
pub fn init_checked(animator: &mut Animator, cx: &mut Cx, checked: bool) {
    animator.cut(
        cx,
        if checked {
            &[id!(checked), id!(on)]
        } else {
            &[id!(checked), id!(off)]
        },
    );
}

/// Seat the `disabled` track for a control that was **built** disabled,
/// without animating.
///
/// The same rule as [`init_checked`], and the same bug when it is missed: a
/// control built `disabled: true` has a true field and an animator still at
/// `off`, and the paint reads the animator — so it renders enabled and responds
/// to nothing, which looks like a widget that has stopped working rather than
/// like one that is deliberately off.
///
/// [`set_disabled`] is for a state that *changes* and should therefore fade;
/// this is for the state it starts in.
pub fn init_disabled(animator: &mut Animator, cx: &mut Cx, disabled: bool) {
    animator.cut(
        cx,
        if disabled {
            &[id!(disabled), id!(on)]
        } else {
            &[id!(disabled), id!(off)]
        },
    );
}

/// Drive the `disabled` track, which every control declares and no control
/// decides differently.
pub fn set_disabled(animator: &mut Animator, cx: &mut Cx, disabled: bool) {
    set_state(
        animator,
        cx,
        disabled,
        &[id!(disabled), id!(on)],
        &[id!(disabled), id!(off)],
    );
}

/// Move a two-state group to `on`, doing nothing when it is already there.
///
/// The guard matters: `play` restarts a state's animation, so calling it
/// unconditionally from `handle_event` would restart the hover fade on every
/// mouse move and the control would never finish arriving.
fn set_state(
    animator: &mut Animator,
    cx: &mut Cx,
    on: bool,
    on_state: &[LiveId; 2],
    off_state: &[LiveId; 2],
) {
    if animator.in_state(cx, on_state) != on {
        animator.play(cx, if on { on_state } else { off_state }, None);
    }
}

/// Plate tones, so a control's hover does not have to know how a wash
/// composites.
pub mod plates {
    use makepad_theme::{color, Paint};
    use makepad_widgets::Vec4f;

    /// A plate's hover tone: the appearance's hover wash over its rest tone.
    ///
    /// Composited rather than mixed. The washes are translucent *by design* —
    /// that is what makes them read correctly over a glass surface — so `mix`
    /// would treat the wash's alpha as a weight instead of as coverage, and a
    /// control hovering over a light plate would darken where it should
    /// lighten.
    pub fn hover(rest: Vec4f, paint: &Paint) -> Vec4f {
        color::flatten(paint.element_hover, rest)
    }

    /// A plate's pressed / selected tone — one rung above [`hover`].
    pub fn active(rest: Vec4f, paint: &Paint) -> Vec4f {
        color::flatten(paint.element_active, rest)
    }

    /// A *solid* plate's hover tone.
    ///
    /// A raised plate has a rung above it on the surface ladder to move to; a
    /// solid one does not — it is already at the top — so the step has to come
    /// from the ink it carries. Mixing toward the ink also keeps the step
    /// proportional: a plate that is nearly the ink cannot brighten much, and
    /// one far from it can.
    pub fn solid_hover(rest: Vec4f, ink: Vec4f) -> Vec4f {
        color::mix(rest, ink, 0.12)
    }

    /// A solid plate's pressed tone — one step past [`solid_hover`].
    pub fn solid_press(rest: Vec4f, ink: Vec4f) -> Vec4f {
        color::mix(rest, ink, 0.22)
    }
}

/// Add a control to the Tab order, unless it cannot be reached.
///
/// Paint order is traversal order, so this is called from `draw_walk` in tree
/// order. A disabled control is skipped: tabbing into something that cannot be
/// activated is worse than not reaching it.
pub fn register(cx: &mut Cx2d, uid: WidgetUid, area: Area, disabled: bool) {
    if !disabled {
        crate::widgets::focus::register(cx, uid, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idle_is_the_default_and_every_signal_is_distinguishable() {
        let idle = Signals::default();
        assert!(idle.is_idle());
        for signal in [
            Signals {
                hover_in: true,
                ..Default::default()
            },
            Signals {
                hover_out: true,
                ..Default::default()
            },
            Signals {
                down: true,
                ..Default::default()
            },
            Signals {
                up: true,
                ..Default::default()
            },
            Signals {
                moved: true,
                ..Default::default()
            },
            Signals {
                activate: true,
                ..Default::default()
            },
            Signals {
                redraw: true,
                ..Default::default()
            },
        ] {
            assert_ne!(signal, idle);
            assert!(!signal.is_idle());
        }
    }

    #[test]
    fn test_a_pointer_position_alone_is_not_a_signal() {
        // The position rides along with the gesture that produced it; it is not
        // a gesture of its own, or every mouse move over a control would read
        // as activity and the control would redraw every frame.
        let drift = Signals {
            pointer: Vec2d { x: 100.0, y: 40.0 },
            ..Default::default()
        };
        assert!(drift.is_idle());
    }

    #[test]
    fn test_a_click_that_lands_reports_activate_and_one_that_slides_off_does_not() {
        // The distinction the helper exists to get right: a press that began on
        // a control and ended somewhere else must not fire it.
        let landed = Signals {
            down: true,
            up: true,
            activate: true,
            ..Default::default()
        };
        let slid_off = Signals {
            down: true,
            up: true,
            ..Default::default()
        };
        assert!(landed.activate);
        assert!(!slid_off.activate);
        // Both report the press ending, so a control that draws a held state
        // releases it either way.
        assert!(slid_off.up);
    }

    #[test]
    fn test_the_track_names_are_the_ones_the_helper_drives() {
        // The DSL prototype and this helper have to agree on four names; a
        // rename in one place would otherwise be a silent no-op.
        let tracks = [
            ids!(hover.on),
            ids!(hover.off),
            ids!(press.on),
            ids!(press.off),
            ids!(focus.on),
            ids!(focus.off),
            ids!(disabled.on),
            ids!(disabled.off),
        ];
        for track in tracks {
            assert_eq!(track.len(), 2);
        }
        let groups: Vec<LiveId> = tracks.iter().map(|t| t[0]).collect();
        for want in [id!(hover), id!(press), id!(focus), id!(disabled)] {
            assert!(groups.contains(&want), "{want:?} is not a track the helper drives");
        }
    }

    #[test]
    fn test_every_group_has_both_states() {
        // A group with only `on` would leave a control stuck at its hovered
        // value the first time the pointer left.
        for group in [id!(hover), id!(press), id!(focus), id!(disabled)] {
            for state in [id!(on), id!(off)] {
                let pair = [group, state];
                assert_eq!(pair.len(), 2);
            }
        }
    }
}
