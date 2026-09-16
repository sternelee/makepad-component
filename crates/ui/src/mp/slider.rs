//! `MpSlider` — a value you drag, and the pure arithmetic that turns a pointer
//! into one.
//!
//! The v2 slider was 847 lines, the largest control in the set, and most of it
//! was a private copy of the same hit block the other four carried plus the
//! mapping from pixels to values. The mapping is the interesting part and it was
//! untestable where it sat, because it needed a live `Area`.
//!
//! So the arithmetic is four free functions with tests, and the widget is the
//! part that has to know about `Area`:
//!
//! | function | what it answers |
//! |---|---|
//! | [`clamp`] | a value inside the range, or the range's own bound if that is empty |
//! | [`snap`] | a value on the step grid, without drifting off either end |
//! | [`fraction`] | where a value sits on the track, `0..=1` |
//! | [`value_at`] | what the pointer at `x` is asking for |
//!
//! ## Grab anywhere, and where the knob sits
//!
//! A press anywhere on the track jumps the value to that point and the drag
//! continues from there — bezel's `slider_fraction`, and the reason the whole
//! widget is the drag source rather than the knob.
//!
//! The knob's *centre* is inset by its own radius, so its travel runs from
//! `radius` to `width - radius` and it never overhangs the track. Bezel lets the
//! knob hang half-off at both ends; a clipped circle reads as a rendering bug
//! rather than as a value, so this differs deliberately.

use makepad_widgets::*;

use makepad_theme::ControlSize;

use crate::mp::control;

/// A value constrained to `min..=max`.
///
/// A reversed range (`min > max`) answers `min` rather than picking a bound at
/// random or producing `NaN`: an inverted range is a caller's bug, and a
/// deterministic answer makes it show up as a stuck control instead of as a
/// value that jumps about.
pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
    if !value.is_finite() {
        return min;
    }
    if min > max {
        return min;
    }
    value.clamp(min, max)
}

/// `value` moved to the nearest multiple of `step` above `min`, then clamped.
///
/// The grid is anchored at `min`, not at zero: a slider from 5 to 10 stepping by
/// 2 offers 5, 7, 9 — not 6, 8, 10, which is what a zero-anchored grid gives and
/// which puts the first and last steps out of reach.
///
/// **`min` and `max` are members of the grid.** A range whose extent is not a
/// whole number of steps still has reachable ends: 0 to 1 by 0.3 offers
/// 0, 0.3, 0.6, 0.9 **and 1**. Without that, dragging to the far end of the
/// track stops at 0.9 and the maximum is unreachable by any gesture — a slider
/// that cannot reach its own maximum is a bug the user finds and the author does
/// not. The trade is that the two end values may be off the interior grid, which
/// is exactly where a slider wants them.
///
/// A `step` of zero or less means continuous, and is returned clamped rather
/// than divided by.
pub fn snap(value: f64, min: f64, max: f64, step: f64) -> f64 {
    let clamped = clamp(value, min, max);
    if !(step > 0.0) || !step.is_finite() {
        return clamped;
    }
    // A value at or past an end *is* that end, before the grid is consulted.
    // Reachability of the extremes is not something a step may take away, and
    // the grid cannot be trusted to give it back: a step larger than the whole
    // range has exactly one member, so both ends are equidistant from it and a
    // tie-break sends one of them to the other.
    if clamped <= min {
        return min;
    }
    if clamped >= max {
        return max;
    }
    let ticks = ((clamped - min) / step).round();
    let on_grid = clamp(min + ticks * step, min, max);
    // The ends join the grid when they are at least as near as its nearest
    // member, so a range whose extent is not a whole number of steps still has
    // touchable extremes — 0 to 1 by 0.3 offers 0, 0.3, 0.6, 0.9 and 1.
    let to_grid = (clamped - on_grid).abs();
    if (clamped - min).abs() <= to_grid {
        return min;
    }
    if (max - clamped).abs() < to_grid {
        return max;
    }
    on_grid
}

/// Where `value` sits on a track running `min..=max`, as `0..=1`.
///
/// An empty range answers `0.0`: a track with no travel has one position, and
/// the left end is the one that cannot be mistaken for movement.
pub fn fraction(value: f64, min: f64, max: f64) -> f64 {
    let span = max - min;
    if !span.is_finite() || span <= 0.0 {
        return 0.0;
    }
    ((value - min) / span).clamp(0.0, 1.0)
}

/// What a pointer at `fraction` along the track is asking for.
///
/// The inverse of [`fraction`], then snapped — so a pointer anywhere between two
/// ticks asks for the nearer one, and a drag *feels* like it clicks into place
/// rather than sliding past.
pub fn value_at(fraction: f64, min: f64, max: f64, step: f64) -> f64 {
    let fraction = if fraction.is_finite() {
        fraction.clamp(0.0, 1.0)
    } else {
        0.0
    };
    snap(min + (max - min) * fraction, min, max, step)
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.motion.*

    set_type_default() do #(DrawMpSlider::script_shader(vm)){
        ..mod.draw.DrawQuad

        hover: 0.0
        press: 0.0
        focus: 0.0
        disabled: 0.0

        // 0..1, where the knob sits. Already snapped and clamped by Rust.
        value: 0.0
        track: #x00000000
        fill: #x00000000
        knob: #x00000000
        ring: #x00000000
        // The knob's rest radius and the halo it grows when pressed.
        knob_radius: 7.0
        halo: 0.0
        // The bar's thickness, centred in the row.
        bar: 4.0

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let h = self.rect_size.y
            let bw = self.bar
            let kr = self.knob_radius
            let y = (h - bw) * 0.5
            // The knob's centre is inset by its own radius on both sides, so
            // its travel is `kr .. w - kr` and it never overhangs the track.
            let usable = max(self.rect_size.x - kr * 2.0, 1.0)
            let cx = kr + usable * self.value
            let cy = h * 0.5
            let cap = bw * 0.5
            // Disabled cuts the whole control's coverage rather than swapping
            // its tones, so a state that is unavailable still reads as *this*
            // slider. The v2 control tracked the state in an animator field and
            // never read it, so a disabled slider was pixel-identical to an
            // enabled one.
            let fade = 1.0 - self.disabled * 0.55

            // Track, then fill. `fill` and not `fill_keep` on the track: the
            // fill below is an independent shape, and `fill_keep` would retain
            // the full-width track so the two unions and the bar reads full at
            // every value.
            let track = self.track
            sdf.box(0.0, y, self.rect_size.x, bw, cap)
            sdf.fill(vec4(track.x, track.y, track.z, track.w * fade))

            // The fill runs to the knob's centre, which is where the value is.
            let fill = self.fill
            sdf.box(0.0, y, max(cx, bw), bw, cap)
            sdf.fill(vec4(fill.x, fill.y, fill.z, fill.w * fade))

            // A halo at press, so the knob answers the finger before the value
            // has moved far enough to see.
            if (self.halo > 0.0) {
                sdf.circle(cx, cy, kr + 3.0 * self.halo)
                let ring = self.ring
                sdf.fill(vec4(ring.x, ring.y, ring.z, ring.w * 0.18 * self.halo))
            }

            // The knob. `fill_keep` here is the right call: the stroke on the
            // next line is the *same* circle, which is what keeping the shape
            // is for.
            let knob = self.knob
            sdf.circle(cx, cy, kr)
            sdf.fill_keep(vec4(knob.x, knob.y, knob.z, knob.w * fade))
            let edge = mix(self.track, self.ring, self.focus)
            let edge_w = mix(0.0, 2.0, self.focus)
            if (edge_w > 0.0) {
                sdf.stroke(edge, edge_w)
            }
            return sdf.result
        }
    }

    mod.mp.MpSliderBase = #(MpSlider::register_widget(vm))

    mod.mp.MpSlider = set_type_default() do mod.mp.MpSliderBase{
        width: Fill
        // 24pt, the form row: the track is 4pt of it and the rest is the target
        // the finger gets. A slider drawn 4pt tall is a slider you cannot hit.
        height: mod.mpc.layout.control.regular.height

        value: 0.0
        min: 0.0
        max: 1.0
        // Continuous by default. A caller who wants detents sets `step`, and the
        // value is then on the grid anchored at `min`.
        step: 0.0
        disabled: false
        control: mod.mpc.ControlSize.Regular

        animator: mod.mp.ControlAnimator{}
    }

    mod.mp.MpSliderSmall = mod.mp.MpSlider{
        control: mod.mpc.ControlSize.Small
        height: mod.mpc.layout.control.small.height
    }
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpSlider {
    #[deref]
    draw_super: DrawQuad,

    #[live]
    hover: f32,
    #[live]
    press: f32,
    #[live]
    focus: f32,
    #[live]
    disabled: f32,

    #[live]
    value: f32,
    #[live]
    track: Vec4f,
    #[live]
    fill: Vec4f,
    #[live]
    knob: Vec4f,
    #[live]
    ring: Vec4f,
    #[live]
    knob_radius: f32,
    #[live]
    halo: f32,
    #[live]
    bar: f32,
}

/// What a slider reports.
#[derive(Clone, Debug, Default)]
pub enum MpSliderAction {
    /// The value changed, carrying it. Emitted on every step of a drag, which is
    /// what a live readout wants; a caller that only wants the resting value
    /// listens for [`MpSliderAction::Released`].
    Changed(f64),
    /// The gesture ended. Carries the value it ended on.
    Released(f64),
    #[default]
    None,
}

// No `ScriptHook` derive: it is implemented by hand to seat `disabled`.
#[derive(Script, Widget, Animator)]
pub struct MpSlider {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[apply_default]
    animator: Animator,

    #[live]
    value: f64,
    #[live]
    min: f64,
    #[live]
    max: f64,
    #[live]
    step: f64,
    #[live]
    disabled: bool,
    #[live]
    control: ControlSize,

    #[redraw]
    #[live]
    draw_bg: DrawMpSlider,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[rust]
    area: Area,
    /// Whether a press is currently being dragged, so a `FingerMove` that
    /// arrives without one is ignored.
    #[rust]
    dragging: bool,
}

impl MpSlider {
    /// The value, put on the step grid and inside the range.
    pub fn value(&self) -> f64 {
        snap(self.value, self.min, self.max, self.step)
    }

    pub fn min(&self) -> f64 {
        self.min
    }

    pub fn max(&self) -> f64 {
        self.max
    }

    pub fn step(&self) -> f64 {
        self.step
    }

    /// The value from a change action, if this slider emitted one.
    pub fn changed(&self, actions: &Actions) -> Option<f64> {
        crate::mp::action::first::<MpSliderAction>(self.widget_uid(), actions).and_then(|a| {
            match a {
                MpSliderAction::Changed(v) => Some(*v),
                _ => None,
            }
        })
    }

    /// The value from a release action.
    pub fn released(&self, actions: &Actions) -> Option<f64> {
        crate::mp::action::first::<MpSliderAction>(self.widget_uid(), actions).and_then(|a| {
            match a {
                MpSliderAction::Released(v) => Some(*v),
                _ => None,
            }
        })
    }

    /// Set the value, snapped and clamped.
    ///
    /// The single mutation path, so a drag, an arrow key and a programmatic
    /// `set_value` cannot disagree about what the current value is.
    pub fn set_value(&mut self, cx: &mut Cx, value: f64) {
        let value = snap(value, self.min, self.max, self.step);
        if (self.value - value).abs() < f64::EPSILON {
            return;
        }
        self.value = value;
        cx.widget_action(self.widget_uid(), MpSliderAction::Changed(value));
        self.redraw(cx);
    }

    /// The value a pointer at `x` screen pixels is asking for.
    ///
    /// Public and taking an `x` rather than reading the event, so the mapping
    /// can be exercised without a window — which is what made the v2 version's
    /// untestable.
    pub fn value_at_x(&self, x: f64, track: Rect) -> f64 {
        let knob = self.draw_bg.knob_radius as f64;
        let usable = (track.size.x - knob * 2.0).max(1.0);
        // The pointer is mapped through the *usable* span, so a press on the
        // extreme left asks for the minimum rather than for a value half a knob
        // to the left of it.
        let f = (x - track.pos.x - knob) / usable;
        let f = if f.is_finite() { f.clamp(0.0, 1.0) } else { 0.0 };
        value_at(f, self.min, self.max, self.step)
    }

    /// Move by `direction` steps — what an arrow key does.
    pub fn nudge(&mut self, cx: &mut Cx, direction: f64) {
        // A continuous slider has no step to move by, so it moves by a
        // hundredth of its range. Without this an arrow key on a continuous
        // slider either does nothing or jumps the whole track.
        let step = if self.step > 0.0 {
            self.step
        } else {
            (self.max - self.min) / 100.0
        };
        // Nudging walks the grid rather than adding `step` to a raw value, so
        // repeated presses cannot accumulate a drift that leaves the value off
        // the ticks.
        let ticks = ((self.value() - self.min) / step).round() + direction;
        self.set_value(cx, self.min + ticks * step);
    }

    pub fn set_disabled(&mut self, cx: &mut Cx, disabled: bool) {
        if self.disabled == disabled {
            return;
        }
        self.disabled = disabled;
        control::set_disabled(&mut self.animator, cx, disabled);
        self.redraw(cx);
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

impl ScriptHook for MpSlider {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        // A slider built `disabled: true` has to *paint* disabled: the field is
        // true from construction but the animator is in its default `off` state,
        // and the paint reads the animator.
        let disabled = self.disabled;
        vm.with_cx_mut(|cx| {
            control::init_disabled(&mut self.animator, cx, disabled);
        });
    }
}

impl Widget for MpSlider {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let signals = control::handle(&mut self.animator, cx, event, self.area);
        if signals.redraw {
            self.redraw(cx);
        }
        if self.disabled {
            return;
        }

        if signals.down {
            // Grab anywhere: the press itself sets the value, so a slider
            // responds to a click on its track rather than demanding the knob.
            self.dragging = true;
            let rect = self.area.rect(cx);
            let value = self.value_at_x(signals.pointer.x, rect);
            self.set_value(cx, value);
        }
        if signals.moved && self.dragging {
            let rect = self.area.rect(cx);
            let value = self.value_at_x(signals.pointer.x, rect);
            self.set_value(cx, value);
        }
        if signals.up {
            self.dragging = false;
            cx.widget_action(self.widget_uid(), MpSliderAction::Released(self.value()));
        }

        // Arrow keys. `activate` is deliberately ignored: Enter and Space do
        // nothing to a value, and treating them as "set to the middle" would be
        // an invention rather than a convention.
        if cx.has_key_focus(self.area) {
            if let Event::KeyDown(ke) = event {
                if !ke.is_repeat {
                    match ke.key_code {
                        KeyCode::ArrowRight | KeyCode::ArrowUp => self.nudge(cx, 1.0),
                        KeyCode::ArrowLeft | KeyCode::ArrowDown => self.nudge(cx, -1.0),
                        KeyCode::Home => self.set_value(cx, self.min),
                        KeyCode::End => self.set_value(cx, self.max),
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let theme = makepad_theme::Theme::of(cx.cx);
        let p = &theme.paint;

        // The track is the wash the library uses for a well; the fill is body
        // ink, which is what makes a slider read as a filled quantity rather
        // than as a coloured control.
        self.draw_bg.track = p.element_active;
        self.draw_bg.fill = p.text_muted;
        self.draw_bg.knob = p.solid;
        self.draw_bg.ring = p.caret;

        self.draw_bg.value = fraction(self.value(), self.min, self.max) as f32;
        // The halo appears with the press and stays while the finger is down, so
        // the slider shows that it is still following the pointer even when the
        // pointer has left the track.
        self.draw_bg.halo = if self.dragging { 1.0 } else { 0.0 };

        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        control::register(cx, self.widget_uid(), self.area, self.disabled);
        DrawStep::done()
    }
}

impl MpSliderRef {
    pub fn value(&self) -> f64 {
        self.borrow().map(|inner| inner.value()).unwrap_or(0.0)
    }

    pub fn changed(&self, actions: &Actions) -> Option<f64> {
        self.borrow().and_then(|inner| inner.changed(actions))
    }

    pub fn released(&self, actions: &Actions) -> Option<f64> {
        self.borrow().and_then(|inner| inner.released(actions))
    }

    pub fn set_value(&self, cx: &mut Cx, value: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_value(cx, value);
        }
    }

    pub fn set_disabled(&self, cx: &mut Cx, disabled: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_disabled(cx, disabled);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-9;

    #[test]
    fn test_clamp_holds_the_range() {
        assert!((clamp(5.0, 0.0, 10.0) - 5.0).abs() < EPS);
        assert!((clamp(-1.0, 0.0, 10.0) - 0.0).abs() < EPS);
        assert!((clamp(11.0, 0.0, 10.0) - 10.0).abs() < EPS);
    }

    #[test]
    fn test_clamp_answers_a_reversed_range_deterministically() {
        // An inverted range is a caller's bug. A deterministic answer makes it
        // show up as a stuck control rather than as a value that jumps about.
        assert!((clamp(5.0, 10.0, 0.0) - 10.0).abs() < EPS);
        assert!((clamp(5.0, 10.0, 0.0) - 10.0).abs() < EPS, "not random");
    }

    #[test]
    fn test_clamp_rejects_a_non_finite_value() {
        // A NaN reaching the shader draws nothing at all, on every frame, with
        // no error — the harshest failure mode in the set.
        assert!((clamp(f64::NAN, 2.0, 8.0) - 2.0).abs() < EPS);
        assert!((clamp(f64::INFINITY, 2.0, 8.0) - 2.0).abs() < EPS);
        assert!((clamp(f64::NEG_INFINITY, 2.0, 8.0) - 2.0).abs() < EPS);
    }

    #[test]
    fn test_the_grid_is_anchored_at_min_not_zero() {
        // 5..10 by 2 offers 5, 7, 9. A zero-anchored grid gives 6, 8, 10, which
        // puts the minimum out of reach — the slider can never be at its own
        // left end.
        assert!((snap(5.0, 5.0, 10.0, 2.0) - 5.0).abs() < EPS, "min");
        assert!((snap(6.4, 5.0, 10.0, 2.0) - 7.0).abs() < EPS, "on the grid");
        assert!((snap(9.4, 5.0, 10.0, 2.0) - 9.0).abs() < EPS, "the last tick");
        // 10 is not a tick — the next is 11 — but both ends are touchable, and
        // the ends join the grid.
        assert!((snap(10.0, 5.0, 10.0, 2.0) - 10.0).abs() < EPS, "max");
    }

    #[test]
    fn test_both_ends_are_always_reachable_through_the_pointer() {
        // The property that matters more than a uniform grid: dragging to
        // either end of the track lands exactly on min and max, for any step —
        // including a step larger than the whole range, where the grid has
        // exactly one member and both ends are equidistant from it.
        for step in [0.0, 0.1, 0.25, 0.3, 0.7, 3.0, 100.0] {
            for (min, max) in [(0.0, 1.0), (5.0, 10.0), (-3.0, 3.0), (0.0, 7.0)] {
                assert!(
                    (value_at(0.0, min, max, step) - min).abs() < EPS,
                    "step {step} range {min}..{max}: left end"
                );
                assert!(
                    (value_at(1.0, min, max, step) - max).abs() < EPS,
                    "step {step} range {min}..{max}: right end"
                );
            }
        }
    }

    #[test]
    fn test_a_tick_beyond_the_maximum_is_not_reachable_by_snapping() {
        // The grid is not extended past the range: no value *inside* the range
        // is beyond the maximum's tick, so a stepped slider never offers 11 on a
        // 5..10 track.
        for value in [5.0, 5.5, 6.0, 7.5, 8.0, 9.5, 10.0] {
            let got = snap(value, 5.0, 10.0, 2.0);
            assert!((5.0..=10.0).contains(&got), "{value} -> {got}");
        }
    }

    #[test]
    fn test_snap_rounds_to_the_nearer_tick() {
        // Just under halfway rounds down, just over rounds up. `0.55` exactly is
        // a tie and `round()` takes it upward; the test states that rather than
        // avoiding it, so a change to the tie rule is a visible edit.
        assert!((snap(0.54, 0.0, 1.0, 0.1) - 0.5).abs() < 1e-9);
        assert!((snap(0.56, 0.0, 1.0, 0.1) - 0.6).abs() < 1e-9);
        assert!((snap(0.55, 0.0, 1.0, 0.1) - 0.6).abs() < 1e-9);
    }

    #[test]
    fn test_snap_never_returns_a_value_off_the_grid_or_outside_the_range() {
        for value in [-5.0, 0.0, 0.13, 0.5, 0.87, 1.0, 3.0] {
            let got = snap(value, 0.0, 1.0, 0.25);
            assert!((0.0..=1.0).contains(&got), "{value} -> {got}");
            let ticks = got / 0.25;
            assert!((ticks - ticks.round()).abs() < 1e-9, "{value} -> {got}");
        }
    }

    #[test]
    fn test_the_ends_are_members_of_the_grid() {
        // 0..1 by 0.3: the interior ticks are 0, 0.3, 0.6, 0.9 and 1 joins them.
        // Without that, dragging to the far end stops at 0.9 and the maximum is
        // unreachable by any gesture.
        for (value, want) in [(0.0, 0.0), (0.3, 0.3), (0.55, 0.6), (0.9, 0.9), (1.0, 1.0)] {
            let got = snap(value, 0.0, 1.0, 0.3);
            assert!((got - want).abs() < 1e-9, "{value} -> {got}, want {want}");
        }
        // ...and a value nearer the end than the interior tick takes the end.
        assert!((snap(0.97, 0.0, 1.0, 0.3) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_snap_clamps_a_final_tick_that_overshoots_the_maximum() {
        // 5..10 by 2 has no tick at 10 — the next is 11. The maximum is still
        // reachable, because the ends join the grid.
        assert!((snap(1.0, 0.0, 1.0, 0.3) - 1.0).abs() < EPS);
        assert!((snap(10.0, 5.0, 10.0, 2.0) - 10.0).abs() < EPS);
    }

    #[test]
    fn test_an_interior_value_is_never_off_the_grid() {
        // Only the two ends may leave the grid, and only when the range does not
        // divide evenly. Everything between them is a tick, so a reader can
        // still reason about the values a stepped slider offers.
        for range in [(0.0, 1.0, 0.25), (5.0, 10.0, 2.0), (0.0, 1.0, 0.3)] {
            let (min, max, step) = range;
            for i in 0..=100 {
                let value = min + (max - min) * i as f64 / 100.0;
                let got = snap(value, min, max, step);
                if (got - min).abs() < EPS || (got - max).abs() < EPS {
                    continue;
                }
                let ticks = (got - min) / step;
                assert!(
                    (ticks - ticks.round()).abs() < 1e-9,
                    "{range:?}: {value} -> {got} is off-grid in the interior"
                );
            }
        }
    }

    #[test]
    fn test_a_zero_or_negative_step_means_continuous() {
        // Not a division by zero: a slider is continuous when it has no step,
        // and continuous is the default.
        assert!((snap(0.37, 0.0, 1.0, 0.0) - 0.37).abs() < EPS);
        assert!((snap(0.37, 0.0, 1.0, -1.0) - 0.37).abs() < EPS);
    }

    #[test]
    fn test_fraction_maps_the_range_onto_zero_to_one() {
        assert!((fraction(0.0, 0.0, 10.0) - 0.0).abs() < EPS);
        assert!((fraction(5.0, 0.0, 10.0) - 0.5).abs() < EPS);
        assert!((fraction(10.0, 0.0, 10.0) - 1.0).abs() < EPS);
        assert!((fraction(-4.0, 0.0, 10.0) - 0.0).abs() < EPS);
        assert!((fraction(40.0, 0.0, 10.0) - 1.0).abs() < EPS);
    }

    #[test]
    fn test_fraction_of_an_empty_range_is_zero_not_a_division() {
        assert!((fraction(5.0, 3.0, 3.0) - 0.0).abs() < EPS);
        assert!((fraction(5.0, 3.0, 2.0) - 0.0).abs() < EPS);
        assert!((fraction(5.0, 0.0, f64::INFINITY) - 0.0).abs() < EPS);
    }

    #[test]
    fn test_value_at_is_the_inverse_of_fraction_on_the_ticks() {
        // Round-tripping is the property that matters: a slider that has been
        // dragged to a position and then asked for its position again must not
        // move.
        for ticks in 0..=10 {
            let value = 5.0 + ticks as f64 * 0.5;
            let f = fraction(value, 5.0, 10.0);
            let back = value_at(f, 5.0, 10.0, 0.5);
            assert!((back - value).abs() < 1e-9, "{value} -> {f} -> {back}");
        }
    }

    #[test]
    fn test_value_at_clamps_a_pointer_off_the_end() {
        assert!((value_at(-2.0, 0.0, 1.0, 0.0) - 0.0).abs() < EPS);
        assert!((value_at(3.0, 0.0, 1.0, 0.0) - 1.0).abs() < EPS);
        assert!((value_at(f64::NAN, 0.0, 1.0, 0.0) - 0.0).abs() < EPS);
    }

    #[test]
    fn test_value_at_lands_on_the_grid() {
        // A drag anywhere asks for the nearer tick, which is what makes a
        // stepped slider click into place rather than slide past.
        assert!((value_at(0.31, 0.0, 1.0, 0.25) - 0.25).abs() < EPS);
        assert!((value_at(0.40, 0.0, 1.0, 0.25) - 0.5).abs() < EPS);
    }
}
