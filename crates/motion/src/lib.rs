//! makepad-motion: the animation vocabulary.
//!
//! Every animation in the library comes from the [`MotionSpec`] catalog below;
//! there are no inline durations or curves in a widget (law 3). Numeric curves
//! live in [`CubicBezier`], which evaluates CSS `cubic-bezier()` exactly, and
//! the pure loader math lives in [`phase`].
//!
//! The catalog is deliberately free of any Makepad type so the arithmetic is
//! testable in milliseconds and reusable by any renderer. A widget applies a
//! spec through [`makepad_component::motion`], which turns it into an
//! [`Animator`] `Play` — one adapter, one place where the two vocabularies
//! meet.
//!
//! [`Animator`]: https://docs.rs/makepad-widgets

pub mod phase;
pub mod script;

use std::sync::atomic::{AtomicU32, Ordering};

/// Redraw rate for the pulse and spinner loaders.
///
/// 2026-08, M-series laptop: one spinner drawn at 120Hz cost 36% of a core —
/// the window redraw — with the animation math itself at 0.3%. 30fps is
/// visually equivalent for these chunky cell waves at a quarter of the draws,
/// and a window with no spinner mounted schedules nothing at all.
pub const PULSE_FPS: f32 = 30.0;

/// Redraw rate for hover fades. [`HOVER_FADE`] is 150ms, so this paints it in
/// nine steps.
pub const HOVER_FPS: f32 = 60.0;

/// The CSS `cubic-bezier()` timing function, evaluated exactly.
///
/// Two control points at `(0,0)` and `(1,1)`, the arguments being the other
/// two. Evaluating means solving `x(t) = progress` for `t` and reading `y(t)`:
/// the curve is parametric, so the progress axis is not the parameter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubicBezier {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl CubicBezier {
    pub const LINEAR: Self = Self::new(0.0, 0.0, 1.0, 1.0);

    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// The four control points, in the order Makepad's `Ease::Bezier` takes
    /// them.
    pub const fn control_points(self) -> (f32, f32, f32, f32) {
        (self.x1, self.y1, self.x2, self.y2)
    }

    /// Whether this curve is the identity, which needs no solving.
    pub fn is_linear(self) -> bool {
        (self.x1 - self.y1).abs() < 1e-6 && (self.x2 - self.y2).abs() < 1e-6
    }

    /// `y` at progress `x`, both in `0..1`.
    ///
    /// Newton-Raphson first because it converges in a handful of steps away from
    /// the flat ends, then bisection because Newton stalls exactly where a
    /// curve is nearly vertical there — the case `EASE_OUT_EXPO` is built on.
    pub fn eval(&self, x: f32) -> f32 {
        let x = x.clamp(0.0, 1.0);
        if x <= 0.0 {
            return 0.0;
        }
        if x >= 1.0 {
            return 1.0;
        }
        if self.is_linear() {
            return x;
        }

        let mut t = x;
        for _ in 0..8 {
            let error = sample(t, self.x1, self.x2) - x;
            if error.abs() < 1e-6 {
                return sample(t, self.y1, self.y2).clamp(0.0, 1.0);
            }
            let slope = sample_derivative(t, self.x1, self.x2);
            if slope.abs() < 1e-6 {
                break;
            }
            t -= error / slope;
        }

        let (mut lo, mut hi) = (0.0f32, 1.0f32);
        let mut t = x.clamp(lo, hi);
        for _ in 0..32 {
            let value = sample(t, self.x1, self.x2);
            if (value - x).abs() < 1e-6 {
                break;
            }
            if value < x {
                lo = t;
            } else {
                hi = t;
            }
            t = 0.5 * (lo + hi);
        }
        sample(t, self.y1, self.y2).clamp(0.0, 1.0)
    }
}

/// One coordinate of the cubic through `(0,0)`, `(a1,a2)`-derived control
/// points and `(1,1)`, at parameter `t`.
fn sample(t: f32, a1: f32, a2: f32) -> f32 {
    let a = 1.0 - 3.0 * a2 + 3.0 * a1;
    let b = 3.0 * a2 - 6.0 * a1;
    let c = 3.0 * a1;
    ((a * t + b) * t + c) * t
}

/// `d/dt` of [`sample`].
fn sample_derivative(t: f32, a1: f32, a2: f32) -> f32 {
    let a = 1.0 - 3.0 * a2 + 3.0 * a1;
    let b = 3.0 * a2 - 6.0 * a1;
    let c = 3.0 * a1;
    (3.0 * a * t + 2.0 * b) * t + c
}

/// Expo-out: the entrance curve. Nearly all the travel happens early and the
/// tail is a long settle, which is what makes a fade-in read as an arrival
/// rather than a fade.
pub const EASE_OUT_EXPO: CubicBezier = CubicBezier::new(0.16, 1.0, 0.3, 1.0);
/// The general-purpose out curve.
pub const EASE_OUT: CubicBezier = CubicBezier::new(0.0, 0.0, 0.58, 1.0);
/// The symmetric default.
pub const EASE: CubicBezier = CubicBezier::new(0.25, 0.1, 0.25, 1.0);
/// Tailwind's `ease-out` — the curve the interaction work standardised on.
pub const EASE_TAILWIND: CubicBezier = CubicBezier::new(0.4, 0.0, 0.2, 1.0);
/// A long, soft settle for a list re-sorting under the pointer.
pub const EASE_RESORT: CubicBezier = CubicBezier::new(0.22, 1.0, 0.36, 1.0);
/// The symmetric in-out, for motion with no preferred direction.
pub const EASE_IN_OUT: CubicBezier = CubicBezier::new(0.42, 0.0, 0.58, 1.0);

/// One named animation: how long, how late, and on what curve.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionSpec {
    /// How long the animation runs.
    pub duration_ms: u64,
    /// How long it waits first.
    pub delay_ms: u64,
    /// The curve.
    pub curve: CubicBezier,
}

impl MotionSpec {
    pub const fn new(duration_ms: u64, curve: CubicBezier) -> Self {
        Self {
            duration_ms,
            delay_ms: 0,
            curve,
        }
    }

    pub const fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    /// Duration in the seconds a Makepad animator takes.
    pub fn duration_secs(self) -> f64 {
        self.duration_ms as f64 / 1000.0
    }

    /// Delay in seconds, scaled by [`speed_scale`].
    pub fn delay_secs(self) -> f64 {
        self.delay_ms as f64 / 1000.0 * speed_scale() as f64
    }

    /// Duration in seconds, scaled by [`speed_scale`] so a global "reduce
    /// motion" or a test harness can slow or speed the whole app from one
    /// place.
    pub fn scaled_duration_secs(self) -> f64 {
        self.duration_secs() * speed_scale() as f64
    }

    /// Total time to settle, delay included, in seconds.
    pub fn total_secs(self) -> f64 {
        self.scaled_duration_secs() + self.delay_secs()
    }

    /// The spec's curve as Makepad's four `Ease::Bezier` control points.
    pub fn bezier(self) -> (f32, f32, f32, f32) {
        self.curve.control_points()
    }

    /// Map a linear `0..1` progress through `curve`.
    pub fn progress(self, raw: f32) -> f32 {
        self.curve.eval(raw)
    }

    /// The animations that run as they are written, without a caller having to
    /// know a number.
    pub const NAMED: &'static [(&'static str, MotionSpec)] = &[
        ("fade_in", FADE_IN),
        ("fade_quick", FADE_QUICK),
        ("menu_in", MENU_IN),
        ("menu_out", MENU_OUT),
        ("dialog_in", DIALOG_IN),
        ("splash_out", SPLASH_OUT),
        ("resize", RESIZE),
        ("tab_slide", TAB_SLIDE),
        ("collapse", COLLAPSE),
        ("layout", LAYOUT),
        ("chevron", CHEVRON),
        ("scroll_glide", SCROLL_GLIDE),
        ("hover_fade", HOVER_FADE),
        ("press", PRESS),
        ("pulse", PULSE),
        ("gradient_spin", GRADIENT_SPIN),
        ("orb", ORB),
    ];
}

// ---------------------------------------------------------------------------
// The catalog
// ---------------------------------------------------------------------------

/// An entrance: the long expo settle.
pub const FADE_IN: MotionSpec = MotionSpec::new(500, EASE_OUT_EXPO);
/// A state change that must not be watched: hover ink, a muted label.
pub const FADE_QUICK: MotionSpec = MotionSpec::new(150, EASE);
/// A popover arriving — short enough to feel attached to the gesture.
pub const MENU_IN: MotionSpec = MotionSpec::new(140, EASE);
/// A popover leaving, faster than it arrived: dismissal should feel immediate.
pub const MENU_OUT: MotionSpec = MotionSpec::new(100, EASE);
/// A dialog arriving. Slower than a menu because it takes over the window.
pub const DIALOG_IN: MotionSpec = MotionSpec::new(180, EASE);
/// A splash leaving.
pub const SPLASH_OUT: MotionSpec = MotionSpec::new(500, EASE).with_delay(150);
/// A pane resizing under a drag that has ended.
pub const RESIZE: MotionSpec = MotionSpec::new(200, EASE_OUT);
/// The selection indicator sliding between tabs.
pub const TAB_SLIDE: MotionSpec = MotionSpec::new(150, EASE_OUT);
/// A disclosure closing. Slightly longer than opening, so the content is not
/// visibly cut off.
pub const COLLAPSE: MotionSpec = MotionSpec::new(180, EASE_OUT);
/// A layout change that moves several things at once.
pub const LAYOUT: MotionSpec = MotionSpec::new(200, EASE_OUT);
/// A chevron rotating.
pub const CHEVRON: MotionSpec = MotionSpec::new(200, EASE);
/// A scroll that has to travel a long way — an easing long enough to read as
/// a glide rather than a jump.
pub const SCROLL_GLIDE: MotionSpec = MotionSpec::new(500, EASE_IN_OUT);
/// Hover ink. 150ms in both directions: faster reads as a flicker, slower as
/// lag.
pub const HOVER_FADE: MotionSpec = MotionSpec::new(150, EASE_TAILWIND);
/// A press. Deliberately shorter than a hover so the plate answers the finger
/// before the eye has finished moving.
pub const PRESS: MotionSpec = MotionSpec::new(90, EASE_TAILWIND);
/// The pulse loader's period.
pub const PULSE: MotionSpec = MotionSpec::new(phase::PULSE_MS, EASE);
/// The gradient matrix spinner's wave period.
pub const GRADIENT_SPIN: MotionSpec = MotionSpec::new(phase::GRADIENT_SPIN_MS, EASE);
/// The orb cluster's breath.
pub const ORB: MotionSpec = MotionSpec::new(phase::ORB_MS, EASE_IN_OUT);

/// The spec a named catalog entry refers to.
pub fn named(name: &str) -> Option<MotionSpec> {
    MotionSpec::NAMED
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, spec)| *spec)
}

// ---------------------------------------------------------------------------
// The global speed knob
// ---------------------------------------------------------------------------

/// A multiplier over every duration, as raw `f32` bits. 1.0 is the catalog as
/// written.
static SPEED: AtomicU32 = AtomicU32::new(1.0f32.to_bits());

/// The current multiplier. One value for the whole app: a "reduce motion"
/// switch and a test harness both want to move everything at once, and nothing
/// wants to move two things at different rates.
pub fn speed_scale() -> f32 {
    f32::from_bits(SPEED.load(Ordering::Relaxed))
}

/// Set the multiplier. Clamped to a floor rather than allowed to reach zero, so
/// "instant" keeps a frame to settle on rather than producing a duration an
/// animator divides by.
pub fn set_speed(scale: f32) {
    SPEED.store(scale.max(0.01).to_bits(), Ordering::Relaxed);
}

/// Print and restore around a region that changes the speed — the shape a test
/// needs, since the value is process-wide.
pub fn restore_speed(previous: f32) -> SpeedGuard {
    let now = speed_scale();
    set_speed(previous);
    SpeedGuard { restore: now }
}

/// Puts [`speed_scale`] back when dropped, including on a panic.
#[must_use = "the speed is restored when the guard is dropped"]
pub struct SpeedGuard {
    restore: f32,
}

impl Drop for SpeedGuard {
    fn drop(&mut self) {
        set_speed(self.restore);
    }
}

/// The phase of a repeating animation at a wall-clock time.
///
/// `elapsed_secs` divided by the period, wrapped to `0..1`. Kept here rather
/// than in each widget so two spinners started at different moments still agree
/// about what phase 0.4 looks like.
pub fn loop_phase(elapsed_secs: f64, period_ms: u64) -> f32 {
    let period = period_ms as f64 / 1000.0;
    if period <= 0.0 {
        return 0.0;
    }
    (elapsed_secs / period).rem_euclid(1.0) as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase;

    #[test]
    fn test_bezier_endpoints_are_exact() {
        for curve in [
            EASE,
            EASE_OUT,
            EASE_OUT_EXPO,
            EASE_TAILWIND,
            EASE_RESORT,
            EASE_IN_OUT,
        ] {
            assert_eq!(curve.eval(0.0), 0.0, "{curve:?}");
            assert_eq!(curve.eval(1.0), 1.0, "{curve:?}");
        }
    }

    #[test]
    fn test_bezier_clamps_outside_the_unit_interval() {
        assert_eq!(EASE.eval(-4.0), 0.0);
        assert_eq!(EASE.eval(9.0), 1.0);
    }

    #[test]
    fn test_linear_curve_is_the_identity() {
        let c = CubicBezier::LINEAR;
        assert!(c.is_linear());
        for i in 0..=100 {
            let x = i as f32 / 100.0;
            assert!((c.eval(x) - x).abs() < 1e-5, "{x}");
        }
    }

    #[test]
    fn test_a_linear_curve_written_as_control_points_is_also_the_identity() {
        // `cubic-bezier(0,0,1,1)` is linear without being `LINEAR`'s
        // coefficients: x1==y1 and x2==y2 is the real test.
        let c = CubicBezier::new(0.0, 0.0, 1.0, 1.0);
        assert!(c.is_linear());
        for i in 0..=10 {
            let x = i as f32 / 10.0;
            assert!((c.eval(x) - x).abs() < 1e-5, "{x}");
        }
    }

    #[test]
    fn test_curves_are_monotonic() {
        // A timing function that goes backwards would make an animation
        // visibly stutter in the middle.
        for curve in [EASE, EASE_OUT, EASE_OUT_EXPO, EASE_RESORT, EASE_IN_OUT] {
            let mut previous = 0.0;
            for i in 0..=200 {
                let v = curve.eval(i as f32 / 200.0);
                assert!(v >= previous - 1e-5, "{curve:?} at {i}: {v} < {previous}");
                previous = v;
            }
        }
    }

    #[test]
    fn test_ease_out_is_ahead_of_linear_and_ease_in_out_is_symmetric() {
        // The point of an "out" curve: more than half the travel is done by
        // the halfway point.
        for i in 1..100 {
            let x = i as f32 / 100.0;
            assert!(EASE_OUT.eval(x) >= x - 1e-5, "out at {x}");
        }
        for i in 1..100 {
            let x = i as f32 / 100.0;
            let sum = EASE_IN_OUT.eval(x) + EASE_IN_OUT.eval(1.0 - x);
            assert!((sum - 1.0).abs() < 1e-3, "in-out at {x}: {sum}");
        }
    }

    #[test]
    fn test_expo_out_settles_early_instead_of_creeping() {
        // 90% of the travel inside the first 40% of the time is what makes an
        // entrance read as an arrival.
        assert!(EASE_OUT_EXPO.eval(0.4) > 0.9, "{}", EASE_OUT_EXPO.eval(0.4));
    }

    #[test]
    fn test_near_vertical_curve_still_solves() {
        // Newton stalls where a curve is nearly vertical; the bisection
        // fallback is what keeps this finite.
        let steep = CubicBezier::new(0.0, 0.9, 0.1, 1.0);
        for i in 0..=100 {
            let v = steep.eval(i as f32 / 100.0);
            assert!(v.is_finite() && (0.0..=1.0).contains(&v), "{v}");
        }
    }

    #[test]
    fn test_control_points_match_the_constructor_arguments() {
        assert_eq!(EASE_TAILWIND.control_points(), (0.4, 0.0, 0.2, 1.0));
        assert_eq!(MotionSpec::new(1, EASE_TAILWIND).bezier(), (0.4, 0.0, 0.2, 1.0));
    }

    #[test]
    fn test_spec_durations_are_seconds() {
        assert!((HOVER_FADE.duration_secs() - 0.15).abs() < 1e-9);
        assert!((FADE_IN.duration_secs() - 0.5).abs() < 1e-9);
        assert!((PULSE.duration_secs() - 2.4).abs() < 1e-9);
        assert!((GRADIENT_SPIN.duration_secs() - 0.75).abs() < 1e-9);
    }

    #[test]
    fn test_delay_is_carried_and_counted_in_the_total() {
        // 500ms of travel after a 150ms wait.
        assert!((SPLASH_OUT.delay_secs() - 0.15).abs() < 1e-9);
        assert!((SPLASH_OUT.total_secs() - 0.65).abs() < 1e-9);
        assert!((HOVER_FADE.total_secs() - 0.15).abs() < 1e-9);
    }

    #[test]
    fn test_speed_scale_moves_every_duration_and_nothing_else() {
        let guard = restore_speed(1.0);
        set_speed(2.0);
        assert!((HOVER_FADE.scaled_duration_secs() - 0.30).abs() < 1e-9);
        // The catalog itself is a constant and must not be mutated by the knob.
        assert_eq!(HOVER_FADE.duration_ms, 150);
        drop(guard);
        assert!((speed_scale() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_speed_scale_is_floored_above_zero() {
        let guard = restore_speed(1.0);
        set_speed(0.0);
        assert!(speed_scale() > 0.0, "{}", speed_scale());
        assert!(HOVER_FADE.scaled_duration_secs() > 0.0);
        drop(guard);
    }

    #[test]
    fn test_speed_guard_restores() {
        let original = speed_scale();
        {
            let _guard = restore_speed(3.0);
            assert!((speed_scale() - 3.0).abs() < 1e-9);
        }
        assert!((speed_scale() - original).abs() < 1e-9);
    }

    #[test]
    fn test_every_named_entry_is_unique_and_resolvable() {
        let mut names: Vec<&str> = MotionSpec::NAMED.iter().map(|(n, _)| *n).collect();
        let count = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), count, "duplicate catalog name");
        for (name, spec) in MotionSpec::NAMED {
            assert_eq!(named(name), Some(*spec), "{name}");
        }
        assert_eq!(named("nope"), None);
    }

    #[test]
    fn test_no_catalog_entry_has_a_zero_duration() {
        // A zero-duration animation is a cut, and a cut belongs in the widget's
        // `Snap` state rather than in the catalog where it would be invisible
        // to a reader looking for "how fast".
        for (name, spec) in MotionSpec::NAMED {
            assert!(spec.duration_ms > 0, "{name}");
        }
    }

    #[test]
    fn test_state_changes_are_faster_than_entrances() {
        // A catalogue whose state changes outlast its entrances feels broken:
        // the ink is still settling while the panel is already gone.
        for (name, spec) in [
            ("hover", HOVER_FADE),
            ("press", PRESS),
            ("menu_in", MENU_IN),
            ("menu_out", MENU_OUT),
            ("dialog_in", DIALOG_IN),
        ] {
            assert!(
                spec.duration_ms < FADE_IN.duration_ms,
                "{name} outlasts the entrance"
            );
        }
        // A press answers the finger before a hover finishes answering the
        // pointer, and dismissal is faster than arrival.
        assert!(PRESS.duration_ms < HOVER_FADE.duration_ms);
        assert!(MENU_OUT.duration_ms < MENU_IN.duration_ms);
        // Loaders are the slowest things in the catalogue by an order of
        // magnitude — they repeat, so they must not read as urgent.
        for (name, spec) in MotionSpec::NAMED {
            if *name == "pulse" || *name == "gradient_spin" || *name == "orb" {
                assert!(spec.duration_ms >= 700, "{name}");
            } else {
                assert!(spec.duration_ms <= 600, "{name}");
            }
        }
    }

    #[test]
    fn test_progress_applies_the_curve() {
        assert!((HOVER_FADE.progress(0.0)).abs() < 1e-6);
        assert!((HOVER_FADE.progress(1.0) - 1.0).abs() < 1e-6);
        // Tailwind ease-out is ahead of linear at the midpoint.
        assert!(HOVER_FADE.progress(0.5) > 0.5);
    }

    #[test]
    fn test_loop_phase_wraps_and_survives_a_zero_period() {
        assert!((loop_phase(0.0, 1000) - 0.0).abs() < 1e-9);
        assert!((loop_phase(0.5, 1000) - 0.5).abs() < 1e-9);
        assert!((loop_phase(1.25, 1000) - 0.25).abs() < 1e-9);
        // A clock that stepped backwards must not produce a negative phase.
        assert!((0.0..1.0).contains(&loop_phase(-0.25, 1000)));
        assert_eq!(loop_phase(3.0, 0), 0.0);
    }

    #[test]
    fn test_loader_specs_agree_with_the_phase_module() {
        // Two places name these periods; they must not drift.
        assert_eq!(PULSE.duration_ms, phase::PULSE_MS);
        assert_eq!(GRADIENT_SPIN.duration_ms, phase::GRADIENT_SPIN_MS);
        assert_eq!(ORB.duration_ms, phase::ORB_MS);
    }

    #[test]
    fn test_redraw_rates_are_bounded() {
        // A zero or absurd rate turns the redraw lease into a spin.
        for fps in [PULSE_FPS, HOVER_FPS] {
            assert!((1.0..=240.0).contains(&fps), "{fps}");
        }
    }
}
