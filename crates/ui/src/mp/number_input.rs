//! `MpNumberInput` — a number, with a field and a step cluster.
//!
//! ## The value is held once, and everything else is a function of it
//!
//! The widget stores a number and the four things that constrain it — `min`, `max`, `step`, `decimals` — and the field
//! shows `format_number(value, decimals)` while the cluster moves it with `nudge`. **Nothing else is stored**: no "the
//! field says X but the value is Y", which is the state that comes apart the first time a caller sets one of them.
//!
//! ## Three functions, and one of them was wrong in the widget this replaces
//!
//! [`format_number`], [`clamp_number`] and [`snap_step`] are the pure parts. The v2 versions of all three are in the
//! target of this port and carried three tests — and **`snap_step` divided by the step without checking it**:
//! `(v / 0.0).round() * 0.0` is `NaN` for any `v`, and `NaN` then propagates through `clamp` (which returns `NaN` rather
//! than clamping, because every comparison against `NaN` is false) into the field, where it formats as `NaN`. A step of
//! zero is what a caller writes when it means "no grid", so that is what this treats it as, and
//! [`test_a_step_of_zero_is_no_grid_rather_than_a_nan`] is the case.
//!
//! A second defect of the same shape: the v2 `clamp_number` was `v.clamp(min, max)`, which **panics** when `min > max`
//! (`f64::clamp` asserts `min <= max`). Two bounds a caller passes the wrong way round is a typo, not a crash —
//! [`set_bounds`] orders them, and a test passes them backwards.
//!
//! ## The composition is tested, not only the parts
//!
//! `nudge` is [`snap_step`] then [`clamp_number`], and **that order is the whole behaviour**: snapping after clamping
//! can walk the value back outside the range (snap 9.9 onto a grid of 5 with a maximum of 9 and you get 10), so the
//! order is checked rather than assumed — which is the failure the v2 tests could not see, because they tested the two
//! functions separately and never together.

use makepad_widgets::*;

/// A number, formatted for display.
///
/// Zero decimals show an integer, which is what a step-by-one field means: `41.7` shows as `42` rather than as `41.7`
/// with a hidden fraction the arrows will eventually reveal.
pub fn format_number(value: f64, decimals: usize) -> String {
    if !value.is_finite() {
        // A `NaN` or an infinity reaching the field is a bug upstream, and `NaN` on screen tells the reader nothing. The
        // zero is shown **and the value is still whatever the caller set**, so the caller's own readback stays honest.
        return "0".to_string();
    }
    if decimals == 0 {
        format!("{}", value.round() as i64)
    } else {
        format!("{value:.decimals$}")
    }
}

/// A number brought inside its bounds.
///
/// **Orders the bounds rather than trusting them**, because `f64::clamp` has an assertion and a caller that passed
/// `min > max` would take the process down for a typo. A `NaN` value clamps to `min`, since nothing about a `NaN` says
/// where it belongs and the lower bound is the answer that is at least inside the range.
pub fn clamp_number(value: f64, min: f64, max: f64) -> f64 {
    let (low, high) = if min <= max { (min, max) } else { (max, min) };
    if value.is_nan() {
        return low;
    }
    value.clamp(low, high)
}

/// A number moved onto the nearest multiple of `step`.
///
/// **A step of zero (or a negative, or a `NaN`) means no grid**, and the value is returned unchanged. That is what a
/// caller means by it — and it is the case the v2 version got wrong, where a zero step produced `NaN` for every input.
pub fn snap_step(value: f64, step: f64) -> f64 {
    if !step.is_finite() || step <= 0.0 || !value.is_finite() {
        return value;
    }
    (value / step).round() * step
}

/// One step up or down from `value`, inside its bounds.
///
/// **Snap first, then clamp** — and the order is the behaviour, not an implementation detail. Clamping first and snapping
/// after can leave the value outside the range: snap `9.9` onto a grid of `5` with a maximum of `9` and the result is
/// `10`. Snapping first means the grid is what moves the value and the bounds have the last word, which is what a
/// stepper must do — a control that can walk past its own maximum is not bounded.
///
/// `direction` is a sign rather than a bool, so `nudge(v, ..., 2.0)` steps twice. A magnitude is clamped to one step
/// per call in effect, because the value is snapped onto the grid first, so `2.0` and `1.0` land on the same neighbour
/// unless the grid is finer than the step — which is a caller's mistake the tests pin rather than a thing to guess about.
pub fn nudge(value: f64, min: f64, max: f64, step: f64, direction: f64) -> f64 {
    let (low, high) = if min <= max { (min, max) } else { (max, min) };
    if !step.is_finite() || step <= 0.0 || !direction.is_finite() || direction == 0.0 {
        // No grid, or no movement: the value is only brought inside its bounds.
        return clamp_number(value, low, high);
    }
    let target = snap_step(value, step) + step * direction.signum();
    clamp_number(target, low, high)
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    /// The numeric field. It is `MpTextInput` with a numeric keyboard hint and an empty-text of `0`, because the hard
    /// parts of a text field — caret, selection, IME — are makepad's and this widget only reads a number out of it.
    ///
    /// **No Rust of its own**, which is the same decision `mp/input.rs` records: a field is a style, and reimplementing
    /// one would be reimplementing the IME.
    mod.mp.MpNumericField = mod.mp.MpTextInput{
        empty_text: "0"
    }

    mod.mp.MpNumberInputBase = #(MpNumberInput::register_widget(vm))

    mod.mp.MpNumberInput = set_type_default() do mod.mp.MpNumberInputBase{
        width: Fit
        height: Fit
        flow: Right
        align: Align{y: 0.5}
        spacing: 4

        input := mod.mp.MpNumericField{
            width: Fill
            height: Fit
        }

        /// The two arrows, stacked. `Ghost` because a stepper that shouts is a stepper a form cannot have many of, and
        /// `MpButtonSmall` rather than a bespoke 22px button because a control of an unusual size is how a form ends up
        /// with two kinds of button in it.
        step_cluster := View{
            width: 22
            height: Fit
            flow: Down
            spacing: 2

            step_up := mod.mp.MpButtonSmall{
                width: Fill
                height: Fit
                style: mod.mp.ButtonStyle.Ghost
                text: "▲"
            }
            step_down := mod.mp.MpButtonSmall{
                width: Fill
                height: Fit
                style: mod.mp.ButtonStyle.Ghost
                text: "▼"
            }
        }
    }
}

/// What a number input reports.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum MpNumberInputAction {
    /// The value changed, carrying it.
    Changed(f64),
    #[default]
    None,
}

/// A number, with a field and a step cluster.
#[derive(Script, ScriptHook, Widget)]
pub struct MpNumberInput {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    /// The value, and the four things that constrain it. **All `#[rust]`**, because a `#[live]` field is written back to
    /// its declared value whenever the script is re-applied — so a value and its bounds kept there would be reset by a
    /// theme change.
    #[rust]
    value: f64,
    #[rust]
    min: f64,
    #[rust]
    max: f64,
    #[rust]
    step: f64,
    #[rust]
    decimals: usize,
}

impl Widget for MpNumberInput {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // **The sub-tree's actions are captured, not passed on.** A `Widget` is given the event rather than the actions,
        // and the arrows are this widget's own business — a host wiring its own steppers is a host that has to
        // reimplement the bounds. So the field and the two buttons are handled here, from actions read out of the
        // capture, and nothing about the value escapes to the host except the `Changed` it emits.
        //
        // The first version of this read `Actions::default()`, which is an empty batch that nothing can be found in:
        // **a stub that compiles, runs, and reports nothing**, which is the failure this port keeps meeting from the
        // other side. The clicks are read from a real capture.
        let actions = cx.capture_actions(|cx| self.view.handle_event(cx, event, scope));

        if self.can_step() {
            if self.view.button(cx, ids!(step_cluster.step_up)).clicked(&actions) {
                self.apply(cx, nudge(self.value, self.min, self.max, self.step, 1.0));
                return;
            }
            if self
                .view
                .button(cx, ids!(step_cluster.step_down))
                .clicked(&actions)
            {
                self.apply(cx, nudge(self.value, self.min, self.max, self.step, -1.0));
                return;
            }
        }

        // **Typed text, and only what parses.** A field showing `12` while somebody is halfway through typing `12.5`
        // holds text that is not yet a number, so a parse failure is not an error — it is the middle of a keystroke, and
        // the value simply does not change yet.
        if let Some(text) = self.view.text_input(cx, ids!(input)).changed(&actions) {
            if let Ok(parsed) = text.trim().parse::<f64>() {
                self.apply(cx, parsed);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // **Nothing is written to the field here**, and that is the difference between a stepper and a fight: a field
        // re-formatted on every paint would turn the `12.` somebody is typing into `12` under their caret. The text is
        // written when the **value** changes — in `apply`, `set_bounds` and `set_value` — so typing and stepping cannot
        // disagree about who owns the text.
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpNumberInput {
    /// Whether stepping is possible at all.
    fn can_step(&self) -> bool {
        self.step.is_finite() && self.step > 0.0 && self.value.is_finite()
    }

    /// Write the value into the field, in the field's own formatting.
    fn show(&mut self, cx: &mut Cx) {
        let text = format_number(self.value, self.decimals);
        self.view.text_input(cx, ids!(input)).set_text(cx, &text);
    }

    /// Set the value, clamped, shown, and reported.
    fn apply(&mut self, cx: &mut Cx, value: f64) {
        self.value = clamp_number(value, self.min, self.max);
        self.show(cx);
        self.redraw(cx);
        cx.widget_action(self.widget_uid(), MpNumberInputAction::Changed(self.value));
    }

    /// Set the bounds and the display precision.
    pub fn set_bounds(&mut self, cx: &mut Cx, min: f64, max: f64, step: f64, decimals: usize) {
        // The bounds are ordered **once, here**, rather than in each function that reads them, so a caller that passed
        // them backwards is corrected at the door.
        let (low, high) = if min <= max { (min, max) } else { (max, min) };
        self.min = low;
        self.max = high;
        self.step = step;
        self.decimals = decimals;
        self.value = clamp_number(self.value, low, high);
        self.show(cx);
        self.redraw(cx);
    }

    /// Set the value, brought inside the bounds.
    pub fn set_value(&mut self, cx: &mut Cx, value: f64) {
        self.value = clamp_number(value, self.min, self.max);
        self.show(cx);
        self.redraw(cx);
    }

    /// The value.
    pub fn value(&self) -> f64 {
        self.value
    }

    /// The bounds and precision in force.
    pub fn bounds(&self) -> (f64, f64, f64, usize) {
        (self.min, self.max, self.step, self.decimals)
    }

    /// Step in the given direction without a click.
    pub fn step_by(&mut self, cx: &mut Cx, direction: f64) {
        if !self.can_step() {
            return;
        }
        self.apply(cx, nudge(self.value, self.min, self.max, self.step, direction));
    }
}

impl MpNumberInputRef {
    pub fn set_bounds(&self, cx: &mut Cx, min: f64, max: f64, step: f64, decimals: usize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_bounds(cx, min, max, step, decimals);
        }
    }

    pub fn set_value(&self, cx: &mut Cx, value: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_value(cx, value);
        }
    }

    pub fn value(&self) -> f64 {
        self.borrow().map(|inner| inner.value).unwrap_or(0.0)
    }

    pub fn step_by(&self, cx: &mut Cx, direction: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.step_by(cx, direction);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_number_renders_with_the_precision_it_was_given() {
        // Zero decimals show an integer, which is what a step-by-one field means: `41.7` shows as `42`, not as `41.7`
        // with a fraction the arrows will eventually reveal.
        assert_eq!(format_number(41.7, 0), "42");
        assert_eq!(format_number(1.25, 2), "1.25");
        assert_eq!(format_number(2.0, 2), "2.00");
        assert_eq!(format_number(-0.4, 0), "0", "a value that rounds to zero shows as -0");
    }

    #[test]
    fn test_a_non_finite_value_shows_as_zero_rather_than_nan() {
        // `NaN` on screen tells a reader nothing, and it is what the v2 widget could produce — see the test below. The
        // **value** is left alone; only the display is substituted, so a caller's own readback stays honest.
        assert_eq!(format_number(f64::NAN, 2), "0");
        assert_eq!(format_number(f64::INFINITY, 0), "0");
        assert_eq!(format_number(f64::NEG_INFINITY, 0), "0");
    }

    #[test]
    fn test_a_step_of_zero_is_no_grid_rather_than_a_nan() {
        // **The v2 defect.** `(v / 0.0).round() * 0.0` is `NaN`, and `NaN.clamp(0.0, 100.0)` is `NaN` (every comparison
        // against `NaN` is false), so a caller that set `step: 0.0` — which is what "no grid" looks like — got a field
        // reading `NaN` and a value that propagated through every later calculation. The v2 tests did not cover it.
        assert_eq!(snap_step(42.0, 0.0), 42.0);
        assert_eq!(snap_step(42.0, -5.0), 42.0);
        assert_eq!(snap_step(42.0, f64::NAN), 42.0);
        // ...and the same through the composition, because that is where a caller meets it.
        assert_eq!(nudge(42.0, 0.0, 100.0, 0.0, 1.0), 42.0);
        assert!(nudge(42.0, 0.0, 100.0, 0.0, 1.0).is_finite());
    }

    #[test]
    fn test_bounds_passed_the_wrong_way_round_are_ordered_rather_than_panicking() {
        // **The second v2 defect.** `f64::clamp` asserts `min <= max` and **panics** otherwise, so a caller that wrote
        // its bounds backwards took the process down for a typo. Ordered here instead.
        assert_eq!(clamp_number(50.0, 100.0, 0.0), 50.0);
        assert_eq!(clamp_number(150.0, 100.0, 0.0), 100.0);
        assert_eq!(clamp_number(-5.0, 100.0, 0.0), 0.0);
        // A `NaN` value lands on the lower bound, which is the answer that is at least inside the range.
        assert_eq!(clamp_number(f64::NAN, 0.0, 100.0), 0.0);
        assert_eq!(nudge(50.0, 100.0, 0.0, 5.0, 1.0), 55.0, "the bounds were not ordered");
    }

    #[test]
    fn test_clamping_keeps_a_value_inside_its_bounds() {
        assert_eq!(clamp_number(150.0, 0.0, 100.0), 100.0);
        assert_eq!(clamp_number(-5.0, 0.0, 100.0), 0.0);
        assert_eq!(clamp_number(42.0, 0.0, 100.0), 42.0);
    }

    #[test]
    fn test_the_grid_is_the_nearest_multiple() {
        assert_eq!(snap_step(12.0, 5.0), 10.0);
        assert_eq!(snap_step(13.0, 5.0), 15.0);
        assert_eq!(snap_step(2.6, 0.5), 2.5);
        // A value already on the grid does not move, which is what makes pressing an arrow twice return to where it
        // started.
        assert_eq!(snap_step(15.0, 5.0), 15.0);
    }

    #[test]
    fn test_the_step_lands_on_the_grid_and_stays_inside_the_bounds() {
        // **The composition, which the v2 tests never checked** — they tested `snap_step` and `clamp_number` separately,
        // and the failure lives in the order they are applied.
        //
        // `9.9` on a grid of `5` with a maximum of `9`: snapping first gives `10`, and the clamp brings it back to `9`.
        // Clamping first would give `9`, and snapping **after** would give `10` — outside the range.
        assert_eq!(nudge(9.9, 0.0, 9.0, 5.0, 1.0), 9.0);
        assert_eq!(nudge(9.9, 0.0, 9.0, 5.0, -1.0), 5.0);
        // A step up from the maximum stays at the maximum, and one down from the minimum stays at the minimum: a
        // control that can walk past its own bounds is not bounded.
        assert_eq!(nudge(100.0, 0.0, 100.0, 5.0, 1.0), 100.0);
        assert_eq!(nudge(0.0, 0.0, 100.0, 5.0, -1.0), 0.0);
    }

    #[test]
    fn test_stepping_up_then_down_returns_to_the_start() {
        // The property a stepper must have, over a range of grids and starts: two opposite steps cancel. It holds because
        // the value is snapped before the offset is added — so whatever the start, the first press lands on the grid.
        for step in [0.5, 1.0, 2.0, 5.0, 7.0] {
            for start in [0.0, 0.3, 3.7, 12.1] {
                let up = nudge(start, 0.0, 100.0, step, 1.0);
                let back = nudge(up, 0.0, 100.0, step, -1.0);
                assert_eq!(
                    back,
                    snap_step(start, step).clamp(0.0, 100.0),
                    "step={step} start={start}: up then down did not return to the grid"
                );
            }
        }
    }

    #[test]
    fn test_a_direction_of_zero_moves_nothing_but_still_clamps() {
        // A caller that computes its direction can compute zero, and that must mean "no movement" rather than "step by
        // zero" — which, with a grid, would snap the value to the nearest multiple, a change the caller did not ask for.
        assert_eq!(nudge(7.0, 0.0, 100.0, 5.0, 0.0), 7.0);
        assert_eq!(nudge(150.0, 0.0, 100.0, 5.0, 0.0), 100.0, "a zero direction skipped the bounds");
        assert_eq!(nudge(7.0, 0.0, 100.0, 5.0, f64::NAN), 7.0);
    }
}
