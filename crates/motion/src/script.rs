//! The catalog on the script heap, and the one adapter between this crate's
//! vocabulary and Makepad's animator.
//!
//! A widget's animator block cannot multiply a duration or convert a curve, so
//! the catalog has to exist as script values it can name directly:
//!
//! ```text
//! from: {all: motion.hover_fade.play}
//! ease: motion.hover_fade.ease
//! ```
//!
//! This is the only place the two vocabularies meet. Nothing upstream of it
//! knows what a `Play` is, which is why [`MotionSpec`] stays arithmetic.

use makepad_widgets::{
    animator::{Ease, Play},
    makepad_script::{trap::NoTrap, ScriptApply},
    LiveId, ScriptValue, ScriptVm,
};

use crate::MotionSpec;

impl MotionSpec {
    /// The catalog curve as a Makepad ease.
    ///
    /// Makepad's `Ease::Bezier` takes the same four control points CSS does, so
    /// this is the curve itself rather than an approximation of it — the reason
    /// [`CubicBezier`](crate::CubicBezier) is evaluated in this crate and not
    /// left to a nearest-match enum variant.
    pub fn ease(self) -> Ease {
        let (cp0, cp1, cp2, cp3) = self.curve.control_points();
        Ease::Bezier {
            cp0: cp0 as f64,
            cp1: cp1 as f64,
            cp2: cp2 as f64,
            cp3: cp3 as f64,
        }
    }

    /// The catalog entry as a Makepad play.
    ///
    /// `Play` has no delay field — a Makepad animator expresses a wait as a
    /// keyframe at a later `time` — so [`MotionSpec::delay_ms`] is published
    /// separately as `delay` and a spec that carries one has to be applied as
    /// two keyframes. That is the one place the two models do not line up, and
    /// it is the caller's to resolve rather than something to paper over with a
    /// zero-length first keyframe here.
    pub fn play(self) -> Play {
        Play::Forward {
            duration: self.scaled_duration_secs(),
        }
    }

    /// The spec as a looping play, for a loader.
    pub fn loop_play(self) -> Play {
        Play::Loop {
            duration: self.scaled_duration_secs(),
            end: 1.0,
        }
    }
}

/// Publish the catalog as `mod.motion.<name>`.
///
/// Each entry carries the raw numbers (`duration`, `delay`), the four control
/// points, the ready `ease` and a ready `play`, so a widget can take whichever
/// shape its animator needs without this module having to guess.
///
/// **Call this after `makepad_widgets::script_mod`**: `Play` and `Ease` are
/// script *types* registered by the widget layer, and converting one before its
/// type exists panics inside the heap rather than reporting a missing module.
pub fn script_mod(vm: &mut ScriptVm) {
    let module = vm.bx.heap.new_module(LiveId::from_str("motion"));
    for (name, spec) in MotionSpec::NAMED {
        let entry = vm.bx.heap.new_object();
        let put = |vm: &mut ScriptVm, key: &str, value: ScriptValue| {
            vm.bx
                .heap
                .set_value_def(entry, LiveId::from_str(key).into(), value);
        };
        put(vm, "duration", (spec.duration_secs()).into());
        put(vm, "delay", (spec.delay_ms as f64 / 1000.0).into());
        let (cp0, cp1, cp2, cp3) = spec.curve.control_points();
        for (key, value) in [
            ("cp0", cp0),
            ("cp1", cp1),
            ("cp2", cp2),
            ("cp3", cp3),
        ] {
            put(vm, key, (value as f64).into());
        }
        let ease = spec.ease().script_to_value(vm);
        put(vm, "ease", ease);
        let play = spec.play().script_to_value(vm);
        put(vm, "play", play);
        let looping = spec.loop_play().script_to_value(vm);
        put(vm, "loop_play", looping);
        vm.bx
            .heap
            .set_value_def(module, LiveId::from_str(name).into(), entry.into());
    }
    // The redraw rates, for a widget that drives its own clock.
    vm.bx.heap.set_value_def(
        module,
        LiveId::from_str("pulse_fps").into(),
        (crate::PULSE_FPS as f64).into(),
    );
    vm.bx.heap.set_value_def(
        module,
        LiveId::from_str("hover_fps").into(),
        (crate::HOVER_FPS as f64).into(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CubicBezier, EASE_TAILWIND, HOVER_FADE};

    #[test]
    fn test_ease_is_the_curve_not_a_nearest_match() {
        // The whole reason this crate evaluates bezier itself: the control
        // points survive into the animator exactly.
        match HOVER_FADE.ease() {
            Ease::Bezier { cp0, cp1, cp2, cp3 } => {
                // The curve crosses into f64 for the animator, so the control
                // points land one f32 ulp away from the decimal literal.
                assert!((cp0 - 0.4).abs() < 1e-6, "{cp0}");
                assert_eq!(cp1, 0.0);
                assert!((cp2 - 0.2).abs() < 1e-6, "{cp2}");
                assert_eq!(cp3, 1.0);
            }
            other => panic!("expected Bezier, got {other:?}"),
        }
    }

    #[test]
    fn test_every_catalog_entry_maps_to_a_bezier() {
        for (name, spec) in MotionSpec::NAMED {
            assert!(
                matches!(spec.ease(), Ease::Bezier { .. }),
                "{name} did not map to Bezier"
            );
        }
    }

    #[test]
    fn test_a_linear_curve_still_maps_to_bezier() {
        // `cubic-bezier(0,0,1,1)` is linear but is not `Ease::Linear` — an
        // identity curve that turned into a different enum variant would
        // silently change how it interpolates near the ends.
        let spec = MotionSpec::new(100, CubicBezier::LINEAR);
        assert!(matches!(spec.ease(), Ease::Bezier { .. }));
    }

    #[test]
    fn test_play_is_forward_with_the_catalog_duration() {
        match HOVER_FADE.play() {
            Play::Forward { duration } => assert!((duration - 0.15).abs() < 1e-9),
            other => panic!("expected Forward, got {other:?}"),
        }
    }

    #[test]
    fn test_play_follows_the_speed_scale() {
        let guard = crate::restore_speed(1.0);
        crate::set_speed(2.0);
        match HOVER_FADE.play() {
            Play::Forward { duration } => assert!((duration - 0.30).abs() < 1e-9, "{duration}"),
            other => panic!("expected Forward, got {other:?}"),
        }
        drop(guard);
    }

    #[test]
    fn test_a_loader_maps_to_a_looping_play() {
        match crate::PULSE.loop_play() {
            Play::Loop { duration, end } => {
                assert!((duration - 2.4).abs() < 1e-9);
                assert!((end - 1.0).abs() < 1e-9);
            }
            other => panic!("expected Loop, got {other:?}"),
        }
    }

    #[test]
    fn test_delay_is_not_folded_into_the_play() {
        // `Play` has no delay field, so a spec carrying one must not silently
        // lose it: the duration stays the duration and the delay stays a
        // separate number the caller has to place.
        match crate::SPLASH_OUT.play() {
            Play::Forward { duration } => assert!((duration - 0.5).abs() < 1e-9, "{duration}"),
            other => panic!("expected Forward, got {other:?}"),
        }
        assert_eq!(crate::SPLASH_OUT.delay_ms, 150);
    }

    #[test]
    fn test_tailwind_curve_survives_the_round_trip() {
        match MotionSpec::new(150, EASE_TAILWIND).ease() {
            Ease::Bezier { cp0, cp1, cp2, cp3 } => {
                assert!((cp0 - 0.4).abs() < 1e-6, "{cp0}");
                assert_eq!(cp1, 0.0);
                assert!((cp2 - 0.2).abs() < 1e-6, "{cp2}");
                assert_eq!(cp3, 1.0);
            }
            other => panic!("expected Bezier, got {other:?}"),
        }
    }

    /// A VM with the widget script layer up, which registers the `Play` and
    /// `Ease` types this module converts.
    fn vm_cx() -> makepad_widgets::makepad_platform::Cx {
        let mut cx = makepad_widgets::makepad_platform::Cx::new(Box::new(|_, _| {}));
        cx.with_vm(makepad_widgets::script_mod);
        cx
    }

    #[test]
    fn test_the_catalog_is_published_under_the_name_a_widget_writes() {
        // A rename in `NAMED` that skipped this module would leave a widget
        // naming an entry that is not there.
        let mut cx = vm_cx();
        cx.with_vm(script_mod);
        cx.with_vm(|vm| {
            let module = vm.bx.heap.module(LiveId::from_str("motion"));
            for (name, spec) in MotionSpec::NAMED {
                let entry = vm
                    .bx
                    .heap
                    .value(module, LiveId::from_str(name).into(), NoTrap);
                let entry = entry
                    .as_object()
                    .unwrap_or_else(|| panic!("mod.motion.{name} is missing"));
                let duration = vm
                    .bx
                    .heap
                    .value(entry, LiveId::from_str("duration").into(), NoTrap)
                    .as_f64();
                assert_eq!(duration, Some(spec.duration_secs()), "motion.{name}.duration");
                let ease = vm
                    .bx
                    .heap
                    .value(entry, LiveId::from_str("ease").into(), NoTrap);
                assert!(!ease.is_nil(), "motion.{name}.ease");
                let play = vm
                    .bx
                    .heap
                    .value(entry, LiveId::from_str("play").into(), NoTrap);
                assert!(!play.is_nil(), "motion.{name}.play");
            }
        });
    }

    #[test]
    fn test_the_control_points_are_published_for_a_hand_written_ease() {
        let mut cx = vm_cx();
        cx.with_vm(script_mod);
        cx.with_vm(|vm| {
            let module = vm.bx.heap.module(LiveId::from_str("motion"));
            let entry = vm
                .bx
                .heap
                .value(module, LiveId::from_str("hover_fade").into(), NoTrap)
                .as_object()
                .expect("hover_fade");
            for (key, want) in [("cp0", 0.4f64), ("cp1", 0.0), ("cp2", 0.2), ("cp3", 1.0)] {
                let got = vm
                    .bx
                    .heap
                    .value(entry, LiveId::from_str(key).into(), NoTrap)
                    .as_f64()
                    .unwrap_or_else(|| panic!("hover_fade.{key} is not a number"));
                assert!((got - want).abs() < 1e-6, "hover_fade.{key}: {got} want {want}");
            }
        });
    }
}
