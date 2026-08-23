//! The two shipped palettes: `dark` (default) and `light`.
//!
//! Light is designed, not inverted: the content plane stays pure white,
//! chrome greys recede downward, elevation is carried by border + wash
//! instead of lightness steps, and status accents drop to the darker
//! 600-level step of the same hue for WCAG AA on white.

use crate::color::{flatten, grey, neutral, oklch};
use makepad_widgets::Vec4f;

pub struct Tokens {
    // Surface ladder (dark: rises away from bg; light: recedes downward)
    pub bg: Vec4f,
    pub surface: Vec4f,
    pub surface_raised: Vec4f,
    pub surface_card: Vec4f,
    pub surface_dialog: Vec4f,
    pub surface_overlay: Vec4f,
    pub element_hover: Vec4f,
    pub element_active: Vec4f,

    // Hairlines
    pub border: Vec4f,
    pub border_strong: Vec4f,
    pub divider: Vec4f,

    // Text, three levels
    pub text: Vec4f,
    pub text_muted: Vec4f,
    pub text_faint: Vec4f,

    // Inverted solid plate (Prominent buttons)
    pub solid: Vec4f,
    pub solid_hover: Vec4f,
    pub on_solid: Vec4f,

    // Accent, neutral by default (brandable)
    pub accent: Vec4f,
    pub accent_hover: Vec4f,
    pub on_accent: Vec4f,

    // Status colors (+ muted companions for large fills / badges)
    pub danger: Vec4f,
    pub danger_hover: Vec4f,
    pub danger_muted: Vec4f,
    pub warning: Vec4f,
    pub warning_muted: Vec4f,
    pub success: Vec4f,
    pub success_muted: Vec4f,
    pub info: Vec4f,
    pub info_muted: Vec4f,
    pub busy: Vec4f,

    // Component-specific
    pub input_bg: Vec4f,
    pub selection: Vec4f,
    pub caret: Vec4f,
    pub code_text: Vec4f,
    pub code_wash: Vec4f,
}

/// Dark appearance (the default look).
pub fn dark() -> Tokens {
    let bg = grey(0.045);
    let surface = grey(0.065);
    let raised = grey(0.09);
    let white = grey(1.0);

    Tokens {
        bg,
        surface,
        surface_raised: raised,
        surface_card: grey(0.075),
        surface_dialog: grey(0.10),
        surface_overlay: grey(0.12),
        element_hover: flatten(white, surface, 0.11),
        element_active: flatten(white, surface, 0.16),

        border: flatten(white, bg, 0.08),
        border_strong: flatten(white, bg, 0.14),
        divider: flatten(white, bg, 0.08),

        text: neutral(0.92),
        text_muted: neutral(0.71),
        text_faint: neutral(0.56),

        solid: neutral(0.92),
        solid_hover: flatten(neutral(0.92), bg, 0.88),
        on_solid: neutral(0.15),

        accent: neutral(0.80),
        accent_hover: flatten(neutral(0.80), bg, 0.88),
        on_accent: neutral(0.14),

        danger: oklch(0.70, 0.19, 25.0),
        danger_hover: flatten(oklch(0.70, 0.19, 25.0), bg, 0.88),
        danger_muted: oklch(0.80, 0.10, 25.0),
        warning: oklch(0.80, 0.16, 80.0),
        warning_muted: oklch(0.86, 0.09, 80.0),
        success: oklch(0.74, 0.17, 155.0),
        success_muted: oklch(0.82, 0.10, 155.0),
        info: oklch(0.72, 0.13, 230.0),
        info_muted: oklch(0.80, 0.08, 230.0),
        busy: oklch(0.70, 0.19, 330.0),

        input_bg: flatten(white, bg, 0.03),
        selection: neutral(0.80),
        caret: neutral(0.92),
        code_text: neutral(0.90),
        code_wash: flatten(white, bg, 0.04),
    }
}

/// Light appearance — designed, not inverted.
pub fn light() -> Tokens {
    let bg = grey(1.0);
    let black = grey(0.0);

    Tokens {
        bg,
        surface: grey(0.965),
        surface_raised: grey(0.93),
        surface_card: grey(1.0),
        surface_dialog: grey(1.0),
        surface_overlay: grey(0.985),
        element_hover: flatten(black, bg, 0.05),
        element_active: flatten(black, bg, 0.09),

        border: flatten(black, bg, 0.10),
        border_strong: flatten(black, bg, 0.17),
        divider: flatten(black, bg, 0.10),

        text: neutral(0.26),
        text_muted: neutral(0.46),
        text_faint: neutral(0.58),

        solid: neutral(0.27),
        solid_hover: flatten(neutral(0.27), bg, 0.88),
        on_solid: grey(1.0),

        accent: neutral(0.38),
        accent_hover: flatten(neutral(0.38), bg, 0.88),
        on_accent: grey(1.0),

        // 600-level step of the same hue as dark's 400s, for AA on white.
        danger: oklch(0.56, 0.21, 25.0),
        danger_hover: flatten(oklch(0.56, 0.21, 25.0), bg, 0.88),
        danger_muted: oklch(0.72, 0.13, 25.0),
        warning: oklch(0.54, 0.14, 75.0),
        warning_muted: oklch(0.72, 0.10, 75.0),
        success: oklch(0.52, 0.14, 155.0),
        success_muted: oklch(0.70, 0.10, 155.0),
        info: oklch(0.55, 0.14, 240.0),
        info_muted: oklch(0.72, 0.09, 240.0),
        busy: oklch(0.56, 0.21, 330.0),

        input_bg: grey(1.0),
        selection: neutral(0.38),
        caret: neutral(0.26),
        code_text: neutral(0.28),
        code_wash: flatten(black, bg, 0.04),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::contrast_ratio;

    fn assert_aa(fg: Vec4f, bg: Vec4f, label: &str) {
        let r = contrast_ratio(fg, bg);
        assert!(r >= 4.5, "{} contrast {:.2}:1 (< 4.5)", label, r);
    }

    #[test]
    fn test_dark_pairings_meet_aa() {
        let t = dark();
        assert_aa(t.text, t.bg, "text/bg");
        assert_aa(t.text, t.surface_card, "text/card");
        assert_aa(t.on_solid, t.solid, "on_solid/solid");
        assert_aa(t.on_accent, t.accent, "on_accent/accent");
        assert_aa(t.danger, t.bg, "danger/bg");
        assert_aa(t.success, t.bg, "success/bg");
        assert_aa(t.warning, t.bg, "warning/bg");
    }

    #[test]
    fn test_light_pairings_meet_aa() {
        let t = light();
        assert_aa(t.text, t.bg, "text/bg");
        assert_aa(t.text, t.surface, "text/surface");
        assert_aa(t.on_solid, t.solid, "on_solid/solid");
        assert_aa(t.on_accent, t.accent, "on_accent/accent");
        assert_aa(t.danger, t.bg, "danger/bg");
        assert_aa(t.success, t.bg, "success/bg");
        assert_aa(t.warning, t.bg, "warning/bg");
        assert_aa(t.info, t.bg, "info/bg");
    }

    #[test]
    fn test_light_is_designed_not_inverted() {
        let d = dark();
        let l = light();
        // Content plane stays pure white in light mode.
        assert_eq!((l.bg.x * 255.0).round(), 255.0);
        // Chrome recedes downward: surfaces are darker than the content plane.
        assert!(l.surface.x < l.bg.x);
        assert!(l.surface_raised.x < l.surface.x);
        // Dark mode rises the other way.
        assert!(d.surface_raised.x > d.surface.x);
    }
}
