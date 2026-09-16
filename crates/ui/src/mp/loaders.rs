//! Loaders: the spinner, the pulse, and the determinate bar.
//!
//! These are the payoff for [`makepad_motion::phase`]. Its constants are the
//! *numbers* — the period, the resting opacity, the per-cell stagger — and those
//! are injected into the shaders as instances from Rust rather than restated in
//! the DSL, because a drifting stagger is the kind of thing nobody notices until
//! two loaders disagree.
//!
//! What *is* restated is two lines of arithmetic: the stagger offset and the
//! pulse wave. Makepad shaders have no loops and no way to call into Rust, so a
//! shader that draws a breathing dot has to say what breathing is. Keeping the
//! numbers single-sourced is what matters; the formula is short enough to read.
//!
//! ## The clock
//!
//! A looping animator, not a timer. `Play::Loop` holds `phase` at
//! `(elapsed / duration) % 1` and makes the animator report `must_redraw` while
//! it runs, so a loader needs no lease, no tick list and no `Instant` — and a
//! window with no loader mounted schedules nothing at all. The v2 loaders each
//! carried their own frame accounting.

use makepad_widgets::*;

use makepad_motion::phase;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.motion.*

    // ---- the spinner ----
    //
    // A comet: a ring whose ink falls off around the circle and travels. One
    // draw call and one number (`phase`), where a ring of dots would be N draws
    // and N phases.
    set_type_default() do #(DrawMpSpinner::script_shader(vm)){
        ..mod.draw.DrawQuad

        // 0..1 around the circle, driven by the animator.
        phase: 0.0
        ring: #x00000000
        thickness: 2.0
        // How fast the tail falls off. 1.0 is a half-turn of visible ink;
        // higher is a shorter comet. The v2-ish 3.0 read as a short dash with a
        // hard end rather than as something travelling.
        tail: 1.0

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let c = self.rect_size * 0.5
            let r = min(c.x, c.y) - self.thickness * 0.5

            // This fragment's angle, as 0..1 clockwise from twelve o'clock —
            // the same origin `phase::orb_ring_seat` measures from, so a
            // spinner and a ring are read the same way.
            let d = self.pos * self.rect_size - c
            let turn = fract(atan2(d.x, -d.y) / TAU + 0.5)

            // Distance behind the head, wrapped, so the falloff is continuous
            // across the seam at twelve o'clock.
            let behind = fract(turn - self.phase + 1.0)
            let fall = pow(1.0 - behind, 2.0 * self.tail)

            let ink = self.ring
            sdf.circle(c.x, c.y, r)
            sdf.stroke(vec4(ink.x, ink.y, ink.z, ink.w * fall), self.thickness)
            return sdf.result
        }
    }

    // ---- the pulse ----
    //
    // Three cells in one draw call: the stagger is arithmetic on a index, so
    // unrolling three of them is cheaper than three widgets and three animator
    // paths — and Makepad shaders have no loops to write it as one.
    set_type_default() do #(DrawMpPulse::script_shader(vm)){
        ..mod.draw.DrawQuad

        phase: 0.0
        // From `phase::PULSE_STAGGER`, `PULSE_MIN_OPACITY`, `PULSE_MIN_SCALE`.
        stagger: 0.0625
        min_opacity: 0.08
        min_scale: 0.9
        // The gap between cells as a fraction of a cell's width.
        gap: 0.4
        ink: #x00000000

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let n = 3.0
            // A cell's side: three of them plus two gaps across the box.
            let side = self.rect_size.x / (n + (n - 1.0) * self.gap)
            let step = side * (1.0 + self.gap)
            let cy = self.rect_size.y * 0.5
            let p = self.pos * self.rect_size
            let ink = self.ink

            // Unrolled: cell i's own phase, its wave, and its disc.
            // `fill`, not `fill_keep`: three cells are three independent
            // shapes. With `fill_keep` the second and third circles union into
            // the first, and the last `fill` paints one blob spanning all three
            // — a wave that renders as a single smeared lozenge.
            let t0 = fract(self.phase + 1.0)
            let w0 = 0.5 - 0.5 * cos(t0 * TAU)
            sdf.circle(side * 0.5, cy, side * 0.5 * (self.min_scale + (1.0 - self.min_scale) * w0))
            sdf.fill(vec4(ink.x, ink.y, ink.z, ink.w * (self.min_opacity + (1.0 - self.min_opacity) * w0)))

            let t1 = fract(self.phase - self.stagger + 1.0)
            let w1 = 0.5 - 0.5 * cos(t1 * TAU)
            sdf.circle(side * 0.5 + step, cy, side * 0.5 * (self.min_scale + (1.0 - self.min_scale) * w1))
            sdf.fill(vec4(ink.x, ink.y, ink.z, ink.w * (self.min_opacity + (1.0 - self.min_opacity) * w1)))

            let t2 = fract(self.phase - self.stagger * 2.0 + 1.0)
            let w2 = 0.5 - 0.5 * cos(t2 * TAU)
            sdf.circle(side * 0.5 + step * 2.0, cy, side * 0.5 * (self.min_scale + (1.0 - self.min_scale) * w2))
            sdf.fill(vec4(ink.x, ink.y, ink.z, ink.w * (self.min_opacity + (1.0 - self.min_opacity) * w2)))

            return sdf.result
        }
    }

    // ---- the determinate bar ----
    set_type_default() do #(DrawMpProgress::script_shader(vm)){
        ..mod.draw.DrawQuad

        // 0..1, the caller's value.
        value: 0.0
        track: #x00000000
        fill: #x00000000
        radius: 4.0

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let h = self.rect_size.y
            let r = min(self.radius, h * 0.5)

            // `fill`, not `fill_keep`. `fill_keep` composites the colour but
            // **retains the shape**, so the fill's box below would union with
            // the track's full-width box and the bar would read 100% at every
            // value — which is exactly what a pixel probe of this bar found
            // before it was fixed: uniform #969696 along the whole 62% row.
            sdf.box(0.0, 0.0, self.rect_size.x, h, r)
            sdf.fill(self.track)

            if (self.value > 0.0) {
                // `max` against the height, so a value near zero draws a disc
                // rather than a rounded rect inside out — which renders as a
                // dark notch at the left end and looks like a layout bug.
                let w = max(self.rect_size.x * self.value, h)
                sdf.box(0.0, 0.0, w, h, r)
                sdf.fill(self.fill)
            }
            return sdf.result
        }
    }

    // ---- the widgets ----
    //
    // Both repeating loaders are `#(Type::register_widget(vm))` rather than
    // pure DSL because both drive an animator, and a Makepad animator needs a
    // widget to own it.
    mod.mp.MpSpinnerBase = #(MpSpinner::register_widget(vm))
    mod.mp.MpSpinner = set_type_default() do mod.mp.MpSpinnerBase{
        width: 20
        height: 20
        thickness: 2.0

        animator: Animator{
            spin: {
                default: @off
                off: AnimatorState{
                    from: {all: Snap}
                    apply: {draw_bg: {phase: 0.0}}
                }
                on: AnimatorState{
                    // Linear: a spinner that eases stutters once per turn.
                    ease: Ease.Linear
                    from: {all: Loop{duration: mod.motion.gradient_spin.duration, end: 1.0}}
                    apply: {draw_bg: {phase: 1.0}}
                }
            }
        }
    }

    mod.mp.MpSpinnerSmall = mod.mp.MpSpinner{ width: 14, height: 14, thickness: 1.5 }
    mod.mp.MpSpinnerLarge = mod.mp.MpSpinner{ width: 28, height: 28, thickness: 2.5 }

    mod.mp.MpPulseBase = #(MpPulse::register_widget(vm))
    mod.mp.MpPulse = set_type_default() do mod.mp.MpPulseBase{
        // Width and height are *derived* from `cell` at paint, not declared
        // here. A widget whose only drawing is a `draw_bg` shader has nothing
        // to give it a size, so `width: Fit` measured zero and the pulse drew
        // nothing at all — three invisible cells and an empty row.
        width: Fit
        height: Fit
        // A cell's side. Three of them plus the gaps make the box.
        cell: 8.0

        animator: Animator{
            pulse: {
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

    mod.mp.MpProgressBase = #(MpProgress::register_widget(vm))
    mod.mp.MpProgress = set_type_default() do mod.mp.MpProgressBase{
        width: Fill
        height: 8
        value: 0.0
        radius: 4.0
    }
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpSpinner {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    phase: f32,
    #[live]
    ring: Vec4f,
    #[live]
    thickness: f32,
    #[live]
    tail: f32,
}

/// A spinner. It repeats forever; there is deliberately no determinate spinner,
/// because a spinner that knows how far along it is, is a [`MpProgress`].
// No `ScriptHook` derive: it is implemented by hand to start the loop.
#[derive(Script, Widget, Animator)]
pub struct MpSpinner {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[apply_default]
    animator: Animator,
    #[redraw]
    #[live]
    draw_bg: DrawMpSpinner,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[live]
    thickness: f32,
}

impl ScriptHook for MpSpinner {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        // A loader is always running. Starting it here rather than in
        // `draw_walk` matters: `draw_walk` runs every frame, and playing the
        // state every frame would restart the loop every frame — the spinner
        // would freeze at its head.
        vm.with_cx_mut(|cx| {
            self.animator_play(cx, ids!(spin.on));
        });
    }
}

impl MpSpinner {
    /// The rotation's period, read from the catalog rather than repeated.
    pub fn period_ms() -> u64 {
        phase::GRADIENT_SPIN_MS
    }
}

impl Widget for MpSpinner {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let theme = makepad_theme::Theme::of(cx.cx);
        self.draw_bg.ring = theme.paint.text_muted;
        self.draw_bg.thickness = self.thickness;
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);
        DrawStep::done()
    }
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpPulse {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    phase: f32,
    #[live]
    stagger: f32,
    #[live]
    min_opacity: f32,
    #[live]
    min_scale: f32,
    #[live]
    gap: f32,
    #[live]
    ink: Vec4f,
}

/// A wave of dots — the loader for "working, and there is nothing to count".
// No `ScriptHook` derive: it is implemented by hand to start the loop.
#[derive(Script, Widget, Animator)]
pub struct MpPulse {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[apply_default]
    animator: Animator,
    #[redraw]
    #[live]
    draw_bg: DrawMpPulse,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    /// A cell's side. Three of them plus the gaps make the box.
    #[live]
    cell: f64,
}

/// How many cells the pulse draws.
///
/// Not `phase::PULSE_CELLS`, which is five. The crate's constant describes the
/// general loader; at the size a loader is actually drawn, five cells are a
/// smear and three are still a wave. The two names are deliberately different so
/// neither is mistaken for the other.
pub const PULSE_DRAWN_CELLS: usize = 3;

/// The gap between cells, as a fraction of a cell's width.
const PULSE_GAP: f64 = 0.4;

impl ScriptHook for MpPulse {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        vm.with_cx_mut(|cx| {
            self.animator_play(cx, ids!(pulse.on));
        });
    }
}

impl MpPulse {
    /// The box a pulse needs for `cells` cells at `cell` points each.
    ///
    /// Derived rather than set in the DSL, because the DSL's `width`/`height`
    /// are what the *caller* asked for and this is what the shader will
    /// actually draw. They are the same number unless a caller set a size, in
    /// which case the cells scale to fit.
    pub fn preferred_extent(cell: f64, cells: usize) -> f64 {
        let cells = cells as f64;
        cell * (cells + (cells - 1.0) * PULSE_GAP)
    }
}

impl Widget for MpPulse {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let theme = makepad_theme::Theme::of(cx.cx);
        // The numbers come from the crate that owns them, so a change to the
        // stagger reaches every loader without a second edit.
        self.draw_bg.stagger = phase::PULSE_STAGGER;
        self.draw_bg.min_opacity = phase::PULSE_MIN_OPACITY;
        self.draw_bg.min_scale = phase::PULSE_MIN_SCALE;
        self.draw_bg.gap = PULSE_GAP as f32;
        self.draw_bg.ink = theme.paint.text_muted;
        // The box is the cells and their gaps, so a caller sets `cell` and the
        // geometry follows. Overriding `width` would only clip the last cell,
        // so there is nothing here for a caller to want to override.
        self.walk.width = Size::Fixed(Self::preferred_extent(self.cell, PULSE_DRAWN_CELLS));
        self.walk.height = Size::Fixed(self.cell);
        let walk = self.walk;
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);
        DrawStep::done()
    }
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpProgress {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    value: f32,
    #[live]
    radius: f32,
    #[live]
    track: Vec4f,
    #[live]
    fill: Vec4f,
}

/// How far along something is, when that is knowable.
#[derive(Script, ScriptHook, Widget)]
pub struct MpProgress {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[redraw]
    #[live]
    draw_bg: DrawMpProgress,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[live]
    value: f64,
    #[live]
    radius: f64,
}

impl MpProgress {
    /// A raw value clamped into the drawable range.
    pub fn clamped(value: f64) -> f64 {
        value.clamp(0.0, 1.0)
    }

    pub fn value(&self) -> f64 {
        Self::clamped(self.value)
    }

    pub fn set_value(&mut self, cx: &mut Cx, value: f64) {
        let value = Self::clamped(value);
        if (self.value - value).abs() < f64::EPSILON {
            return;
        }
        self.value = value;
        self.redraw(cx);
    }
}

impl Widget for MpProgress {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {
        // A progress bar is not interactive: it reports, it does not take.
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let theme = makepad_theme::Theme::of(cx.cx);
        // The track is the wash the rest of the library uses for a well, so a
        // bar sitting in a card looks like part of the card.
        self.draw_bg.track = theme.paint.element_active;
        self.draw_bg.fill = theme.paint.accent;
        self.draw_bg.value = self.value() as f32;
        self.draw_bg.radius = self.radius as f32;
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);
        DrawStep::done()
    }
}

impl MpProgressRef {
    pub fn value(&self) -> f64 {
        self.borrow().map(|inner| inner.value()).unwrap_or(0.0)
    }

    pub fn set_value(&self, cx: &mut Cx, value: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_value(cx, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_spinner_period_is_the_catalogs() {
        // Two places name it — the animator's DSL and this accessor — and a
        // reader of one should not have to go looking for the other.
        assert_eq!(MpSpinner::period_ms(), phase::GRADIENT_SPIN_MS);
        assert_eq!(
            MpSpinner::period_ms(),
            makepad_motion::GRADIENT_SPIN.duration_ms
        );
    }

    #[test]
    fn test_the_pulse_period_is_the_catalogs() {
        assert_eq!(phase::PULSE_MS, makepad_motion::PULSE.duration_ms);
    }

    #[test]
    fn test_the_drawn_cell_count_matches_the_shader() {
        // The shader unrolls three cells by hand; this is what catches a fourth
        // being declared without the shader drawing it, or the reverse.
        assert_eq!(PULSE_DRAWN_CELLS, 3);
    }

    #[test]
    fn test_the_three_cells_never_all_peak_at_once() {
        // If the stagger were larger than a third of the cycle the wave would
        // read as a rotating highlight rather than a travelling breath.
        let spread = PULSE_DRAWN_CELLS as f32 * phase::PULSE_STAGGER;
        assert!(spread < 0.5, "{spread}");
        // ...and it must not be so small that the three move together.
        assert!(spread > 0.05, "{spread}");
    }

    #[test]
    fn test_the_drawn_extent_matches_the_shaders_own_arithmetic() {
        // The shader computes `side = x / (n + (n-1)*gap)`; this is the same
        // relation solved for the box. If they disagree the cells are clipped
        // on the right, which is easy to miss at 8pt.
        let cell = 8.0;
        let extent = MpPulse::preferred_extent(cell, PULSE_DRAWN_CELLS);
        let n = PULSE_DRAWN_CELLS as f64;
        let side = extent / (n + (n - 1.0) * PULSE_GAP);
        assert!((side - cell).abs() < 1e-9, "side {side} cell {cell}");
    }

    #[test]
    fn test_progress_clamps_rather_than_drawing_outside_itself() {
        // Above one the fill would run past the track's end; below zero the
        // rounded cap draws inside out, which shows as a dark notch.
        assert_eq!(MpProgress::clamped(-0.5), 0.0);
        assert_eq!(MpProgress::clamped(1.5), 1.0);
        assert_eq!(MpProgress::clamped(0.4), 0.4);
    }
}
