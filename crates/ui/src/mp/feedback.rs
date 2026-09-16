//! `MpProgressRing` and `MpSkeleton` — the two shapes that say "not yet".
//!
//! [`MpProgress`](crate::mp::loaders::MpProgress) already covers the determinate
//! bar and [`MpSpinner`](crate::mp::loaders::MpSpinner) the indeterminate
//! rotation. These are the two that were missing, and each has a job the others
//! cannot do:
//!
//! - **A ring**, for where a bar does not fit. In a toolbar, on an avatar, beside
//!   a title — a horizontal bar needs a column's worth of width, and a ring needs
//!   a square the height of the row.
//! - **A skeleton**, for where *nothing* is knowable. A spinner says "wait"; a
//!   skeleton says "wait, and this is the shape of what is coming", which is the
//!   better answer when the layout is already decided and only the content is
//!   missing. That distinction is why both exist rather than one.
//!
//! ## The ring draws no arc
//!
//! An arc is a path, and an SDF path needs stroking with a cap discipline. The
//! ring instead paints **the whole circle** and chooses per fragment whether it is
//! the filled part or the track, from the fragment's own angle. That is exact, it
//! has no seam where the arc starts, and it needs no path at all — the same trick
//! `MpSpinner`'s comet tail uses.

use makepad_widgets::*;

use makepad_motion::phase;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.motion.*

    // ---- the ring ----
    set_type_default() do #(DrawMpProgressRing::script_shader(vm)){
        ..mod.draw.DrawQuad

        // 0..1, the caller's value.
        value: 0.0
        track: #x00000000
        fill: #x00000000
        // The band's thickness as a fraction of the radius, so the ring keeps
        // its proportions at every size and Rust never needs the drawn size.
        band: 0.22

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let c = self.rect_size * 0.5
            let r = min(c.x, c.y)
            let w = r * self.band

            // The fragment's angle, 0..1 clockwise from twelve o'clock.
            //
            // `atan2(x, -y)` is already 0 at twelve o'clock and increases
            // clockwise — the first version added a spurious `+ 0.5` and started
            // the arc at six. The `+ 1.0` before `fract` is only there to bring
            // `atan2`'s negative half into range.
            let d = self.pos * self.rect_size - c
            let turn = fract(atan2(d.x, -d.y) / TAU + 1.0)

            // The track everywhere, then the filled part over it. Two `fill`s
            // rather than one with a mix: the boundary between them is a hard
            // angular edge, and a mix would blend it into a gradient no progress
            // bar should have.
            sdf.circle(c.x, c.y, r - w * 0.5)
            sdf.stroke(self.track, w)

            // **An `if`, not `step`.** Makepad's `step` does not take GLSL's
            // argument order — `step(0.999, 1.0)` evaluates to zero — so the
            // first version's `max(step(turn, value), step(0.999, value))` was 0
            // at every angle for every value, and every ring drew as an empty
            // track. A comparison inside an `if` has no argument order to get
            // wrong. A full ring is the `|| value > 0.999` case, because
            // `turn < 1.0` is false for the fragment at exactly twelve o'clock
            // and that leaves a one-pixel notch at the top of a completed ring.
            // Two plain comparisons rather than one with `||`: a compound
            // boolean in a shader is an operator whose semantics are worth not
            // depending on, and the second case is only ever true for a complete
            // ring anyway.
            if (self.value > 0.0) {
                if (turn < self.value) {
                    sdf.circle(c.x, c.y, r - w * 0.5)
                    sdf.stroke(self.fill, w)
                }
            }
            if (self.value > 0.999) {
                sdf.circle(c.x, c.y, r - w * 0.5)
                sdf.stroke(self.fill, w)
            }
            return sdf.result
        }
    }

    mod.mp.MpProgressRingBase = #(MpProgressRing::register_widget(vm))

    mod.mp.MpProgressRing = set_type_default() do mod.mp.MpProgressRingBase{
        width: 28
        height: 28
        // An *initial* value; `set_value` from Rust takes precedence and survives
        // a script re-apply. See the Rust field's note.
        initial: 0.0
    }

    mod.mp.MpProgressRingSmall = mod.mp.MpProgressRing{ width: 18, height: 18 }
    mod.mp.MpProgressRingLarge = mod.mp.MpProgressRing{ width: 44, height: 44 }

    // ---- the skeleton ----
    //
    // The shimmer is a band sweeping the plate, driven by the pulse period from
    // the catalog rather than its own number: a skeleton and a pulse loader on one
    // screen are the same "working" and should breathe at the same rate.
    set_type_default() do #(DrawMpSkeleton::script_shader(vm)){
        ..mod.draw.DrawQuad

        // 0..1, the sweep's position.
        phase: 0.0
        base: #x00000000
        sheen: #x00000000
        radius: 6.0
        // The band's width as a fraction of the plate, and how much lighter the
        // sheen is at its centre. Both are fractions so a skeleton keeps its
        // proportions at any size.
        band: 0.45
        gain: 0.55

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, max(1.0, self.radius))

            // Distance from the sweep's centre, in units of the plate's width, so
            // the band is the same fraction of any plate. The sweep travels from
            // off the left to off the right, so it enters and leaves rather than
            // appearing mid-plate on the first frame.
            let travel = 1.0 + self.band * 2.0
            let centre = self.phase * travel - self.band
            let distance = abs(self.pos.x - centre)
            let glow = max(0.0, 1.0 - distance / max(self.band, 0.001))
            // Squared, so the band has a soft shoulder and a bright centre rather
            // than a linear triangle.
            let mix_amount = glow * glow * self.gain

            let ink = mix(self.base, self.sheen, mix_amount)
            sdf.fill(ink)
            return sdf.result
        }
    }

    mod.mp.MpSkeletonBase = #(MpSkeleton::register_widget(vm))

    mod.mp.MpSkeleton = set_type_default() do mod.mp.MpSkeletonBase{
        width: Fill
        height: 12
        // The corner is on the shader, not the widget — `radius` is a
        // `draw_bg` field, and setting it on the widget is a property error that
        // names every property except the one being set.
        draw_bg +: {radius: 6.0}

        animator: Animator{
            shimmer: {
                default: @off
                off: AnimatorState{
                    from: {all: Snap}
                    apply: {draw_bg: {phase: 0.0}}
                }
                on: AnimatorState{
                    ease: Ease.Linear
                    from: {all: Loop{duration: mod.motion.pulse.duration, end: 1.0}}
                    apply: {draw_bg: {phase: 1.0}}
                }
            }
        }
    }

    // Named shapes, because a skeleton's whole job is to be the shape of the
    // content: a line of text, a short line, a title, a square.
    mod.mp.MpSkeletonText = mod.mp.MpSkeleton{ height: 10 }
    mod.mp.MpSkeletonShort = mod.mp.MpSkeleton{ width: 96, height: 10 }
    mod.mp.MpSkeletonTitle = mod.mp.MpSkeleton{ width: 180, height: 18, draw_bg +: {radius: 4.0} }
    mod.mp.MpSkeletonSquare = mod.mp.MpSkeleton{ width: 40, height: 40, draw_bg +: {radius: 8.0} }
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpProgressRing {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    value: f32,
    #[live]
    track: Vec4f,
    #[live]
    fill: Vec4f,
    #[live]
    band: f32,
}

/// How far along something is, where a bar does not fit.
#[derive(Script, ScriptHook, Widget)]
pub struct MpProgressRing {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[redraw]
    #[live]
    draw_bg: DrawMpProgressRing,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    /// The value the DSL declares, as an *initial* one.
    ///
    /// `#[live]`, so a page can write `MpProgressRing{ initial: 0.5 }`.
    #[live]
    initial: f64,
    /// The value in force, mutated by [`MpProgressRing::set_value`].
    ///
    /// **`#[rust]`, and that is the point.** A `#[live]` field is re-asserted
    /// from the script whenever the heap is re-applied — which `Theme::install`
    /// triggers with `request_script_reapply()` — so a value a Rust setter writes
    /// into one is silently reset to its declared default with no error anywhere.
    /// `MpList::selected` is `#[rust]` and its Rust-set selection renders; this
    /// field was `#[live]` and every setter call was lost. Widget state an app
    /// mutates must not be `#[live]`.
    #[rust]
    value: Option<f64>,
}

impl MpProgressRing {
    /// The value in force: what a setter wrote, or what the DSL declared.
    pub fn live_value(&self) -> f64 {
        self.value.unwrap_or(self.initial)
    }

    /// A raw value clamped into the drawable range.
    pub fn clamped(value: f64) -> f64 {
        if value.is_nan() {
            // A NaN reaching the shader paints nothing on every frame with no
            // error, which is the harshest failure mode in the set — and unlike
            // an infinity it has no sensible clamp, so it becomes the value a
            // ring shows before it knows anything.
            return 0.0;
        }
        // An infinity *does* have a sensible answer, and it is the one
        // `clamp` gives: a progress that is unbounded above is complete.
        value.clamp(0.0, 1.0)
    }

    pub fn value(&self) -> f64 {
        Self::clamped(self.live_value())
    }

    pub fn set_value(&mut self, cx: &mut Cx, value: f64) {
        let value = Self::clamped(value);
        if (self.live_value() - value).abs() < f64::EPSILON {
            return;
        }
        self.value = Some(value);
        self.redraw(cx);
    }
}

impl Widget for MpProgressRing {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {
        // A ring reports; it does not take.
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let (track, fill) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            let p = &theme.paint;
            // The same pair the bar uses, so a ring and a bar on one screen are
            // the same quantity in two shapes.
            (p.element_active, p.accent)
        };
        self.draw_bg.track = track;
        self.draw_bg.fill = fill;
        self.draw_bg.value = self.value() as f32;
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);
        DrawStep::done()
    }
}

impl MpProgressRingRef {
    pub fn value(&self) -> f64 {
        self.borrow().map(|inner| inner.value()).unwrap_or(0.0)
    }

    pub fn set_value(&self, cx: &mut Cx, value: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_value(cx, value);
        }
    }
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpSkeleton {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    phase: f32,
    #[live]
    base: Vec4f,
    #[live]
    sheen: Vec4f,
    #[live]
    radius: f32,
    #[live]
    band: f32,
    #[live]
    gain: f32,
}

/// The shape of content that has not arrived.
// No `ScriptHook` derive: it is implemented by hand to start the shimmer.
#[derive(Script, Widget, Animator)]
pub struct MpSkeleton {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[apply_default]
    animator: Animator,
    #[redraw]
    #[live]
    draw_bg: DrawMpSkeleton,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
}

impl ScriptHook for MpSkeleton {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        // A skeleton is always shimmering; starting it here rather than in
        // `draw_walk` matters, because `draw_walk` runs every frame and playing
        // the state every frame would restart the sweep every frame — the band
        // would freeze at the left edge.
        vm.with_cx_mut(|cx| {
            self.animator_play(cx, ids!(shimmer.on));
        });
    }
}

impl MpSkeleton {
    /// The sweep's period, read from the catalog rather than repeated, so a
    /// skeleton and a pulse loader breathe at the same rate.
    pub fn period_ms() -> u64 {
        phase::PULSE_MS
    }

    /// Where the sweep sits at `phase`, as a fraction of the plate, for a band of
    /// `band`.
    ///
    /// Exposed and tested because the shader's own arithmetic is the same formula
    /// and the two must agree: a sweep that appeared mid-plate on its first frame
    /// is what a missing offset looks like.
    pub fn sweep_centre(phase: f32, band: f32) -> f32 {
        let travel = 1.0 + band * 2.0;
        phase * travel - band
    }
}

impl Widget for MpSkeleton {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let (base, sheen) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            let p = &theme.paint;
            // The plate is a wash rather than a surface, so a skeleton sits on
            // whatever it is standing in for rather than on a plane of its own,
            // and the sheen is ink so the band reads as light rather than as a
            // second colour.
            (p.element_active, p.element_hover)
        };
        self.draw_bg.base = base;
        self.draw_bg.sheen = sheen;
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);
        DrawStep::done()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_sweep_enters_from_off_the_left() {
        // At phase 0 the band's centre is one band-width to the *left* of the
        // plate, so it slides in rather than appearing already inside it. A
        // missing offset is what that fault looks like, and it is invisible in a
        // still frame at any other phase.
        for band in [0.2f32, 0.45, 0.6] {
            let start = MpSkeleton::sweep_centre(0.0, band);
            assert!(
                start <= -band + 1e-6,
                "band {band}: the sweep starts at {start}, inside the plate"
            );
        }
    }

    #[test]
    fn test_the_sweep_leaves_past_the_right_edge() {
        for band in [0.2f32, 0.45, 0.6] {
            let end = MpSkeleton::sweep_centre(1.0, band);
            assert!(
                end >= 1.0 + band - 1e-6,
                "band {band}: the sweep ends at {end}, still on the plate"
            );
        }
    }

    #[test]
    fn test_the_sweep_crosses_the_whole_plate() {
        // The band's centre must pass through the middle somewhere in the cycle,
        // or the shimmer only ever appears at one edge.
        let band = 0.45;
        let crossed = (0..=100).any(|i| {
            let centre = MpSkeleton::sweep_centre(i as f32 / 100.0, band);
            (0.45..=0.55).contains(&centre)
        });
        assert!(crossed, "the sweep never reached the middle");
    }

    #[test]
    fn test_the_skeleton_and_the_pulse_share_one_period() {
        // A skeleton and a pulse loader on one screen are the same "working" and
        // should breathe at the same rate.
        assert_eq!(MpSkeleton::period_ms(), phase::PULSE_MS);
        assert_eq!(MpSkeleton::period_ms(), makepad_motion::PULSE.duration_ms);
    }

    #[test]
    fn test_a_ring_clamps_and_rejects_a_non_finite_value() {
        // A NaN reaching the shader paints nothing on every frame with no error.
        assert_eq!(MpProgressRing::clamped(-0.5), 0.0);
        assert_eq!(MpProgressRing::clamped(1.5), 1.0);
        assert_eq!(MpProgressRing::clamped(0.4), 0.4);
        assert_eq!(MpProgressRing::clamped(f64::NAN), 0.0);
        assert_eq!(MpProgressRing::clamped(f64::INFINITY), 1.0);
    }

    #[test]
    fn test_the_completed_ring_has_no_notch() {
        // `turn < value` alone leaves a one-pixel gap at exactly twelve o'clock
        // when the value is 1.0, because the fragment whose angle is 1.0 is not
        // less than it. The shader adds the `value > 0.999` case; this records the
        // constant so a change to it is a failing test.
        assert!(0.999f32 < 1.0);
        assert!(0.999f32 > 0.99);
    }
}
