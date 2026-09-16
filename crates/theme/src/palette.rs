//! The two concrete palettes: `dark` (the default) and `light`.
//!
//! Light is designed, not inverted: the content plane stays pure white, chrome
//! greys recede downward, elevation is carried by border + wash instead of
//! lightness steps, and status accents drop to the darker step of the same hue
//! so they clear WCAG AA on white.
//!
//! Values are ported from `gpui-bezel/crates/theme/src/theme/palettes.rs`,
//! including that file's provenance notes, because the readings are the reason
//! the numbers are what they are.

use crate::{appearance::Appearance, color, paint::Paint};

/// Hairline ink scaled for the appearance.
///
/// A 1px edge has to hold its own against a bright surround, so light scales
/// *up*: the dark palette's white hairlines are deliberately faint, and the
/// same alpha in black on white dissolves into the panel.
pub(crate) fn hairline_for(appearance: Appearance, alpha: f32) -> makepad_widgets::Vec4f {
    match appearance {
        Appearance::Dark => color::ink(1.0, alpha),
        Appearance::Light => color::ink(0.0, (alpha * 1.35).min(0.5)),
    }
}

/// Recessed band behind a palette / picker header or footer strip.
///
/// A recessed strip on white needs far less ink than on near-black; the dark
/// 16% would read as a bruise.
pub(crate) fn band_for(appearance: Appearance) -> makepad_widgets::Vec4f {
    match appearance {
        Appearance::Dark => color::ink(0.0, 0.16),
        Appearance::Light => color::ink(0.0, 0.045),
    }
}

/// Dark appearance — the default look.
///
/// The surface tones are sampled from reference screenshots: main panel
/// `#060606`, shell / sidebar `#0d0d0d`.
pub fn dark() -> Paint {
    let dark = Appearance::Dark;
    Paint {
        bg: color::grey(0x06),
        surface: color::grey(0x0d),
        surface_raised: color::neutral(0.235),
        surface_card: color::grey(0x0e),
        surface_dialog: color::grey(0x10),
        surface_overlay: color::grey(0x16),
        surface_raised_hover: color::neutral(0.29),
        // Pure ink: white at 8% and 12%.
        element_hover: color::ink(1.0, 0.08),
        element_active: color::ink(1.0, 0.12),
        border: color::ink(1.0, 0.08),
        border_strong: color::ink(1.0, 0.14),
        divider: color::ink(1.0, 0.08),

        text: color::neutral(0.922),
        text_muted: color::neutral(0.708),
        text_faint: color::neutral(0.556),
        text_dim: color::grey(0x98),

        solid: color::neutral(0.922),
        on_solid: color::grey(0x0e),
        // Indigo-400's lightness with no chroma: the shipped accent is
        // neutral, and a brand is what puts a hue on it.
        accent: color::neutral(0.673),
        accent_strong: color::neutral(0.922),
        on_accent: color::grey(0x0e),

        danger: color::oklch(0.704, 0.191, 22.216),
        danger_strong: color::oklch(0.58, 0.16, 25.0),
        danger_muted: color::oklch(0.808, 0.114, 19.571),
        warning: color::oklch(0.828, 0.189, 84.429),
        warning_muted: color::oklch(0.924, 0.12, 95.746),
        success: color::oklch(0.765, 0.177, 163.223),
        success_muted: color::oklch(0.845, 0.143, 164.978),
        busy: color::oklch(0.718, 0.202, 349.761),

        input_bg: color::ink(1.0, 0.03),
        band: band_for(dark),
        selection: color::hsl(0.66, 0.6, 0.55, 0.35),
        cursor: color::ink(1.0, 0.35),
        caret: color::neutral(0.673),
        // ~2.5x border_strong: a ring has to be seen, an edge must not shout.
        ring: hairline_for(dark, 0.35),
        code_text: color::neutral(0.94),
        code_wash: color::ink(1.0, 0.08),

        diff_add: color::oklch(0.765, 0.177, 163.223),
        diff_del: color::oklch(0.704, 0.191, 22.216),
        diff_hunk_bg: color::hsl(0.6, 0.35, 0.6, 0.05),
    }
}

/// Light appearance.
///
/// Neutrals are the same OKLCH scale read from the other end, but the *roles*
/// are reassigned rather than mirrored: content plane white, chrome grey,
/// raised surfaces white-plus-shadow. Text tones reproduce the dark theme's
/// contrast ratios, and accents drop from the 400 to the 600 step at identical
/// hue so they clear WCAG AA on white instead of glowing.
pub fn light() -> Paint {
    let light = Appearance::Light;
    Paint {
        // Main panel — clean white.
        bg: color::grey(0xff),
        // Deeper than a paper-reading of neutral-100: the content card is pure
        // white and sits *inside* this surface, so too small a step leaves the
        // whole window one flat sheet with a hairline drawn on it.
        surface: color::neutral(0.968),
        // A real grey, NOT white. This is the opaque-plate tone — user message
        // bubbles, the jump-to-bottom pill — and those sit directly on the
        // white content plane with no border or shadow to save them. White
        // here made the user's own messages vanish into the page.
        surface_raised: color::neutral(0.940),
        surface_card: color::grey(0xff),
        surface_dialog: color::grey(0xff),
        surface_overlay: color::grey(0xff),
        // Opaque pills darken on hover here rather than brighten — the same
        // "brighten the plate, don't wash it out" rule read the other way.
        surface_raised_hover: color::neutral(0.900),
        // The same tokens in light: black at 4% and 6%.
        element_hover: color::ink(0.0, 0.04),
        element_active: color::ink(0.0, 0.06),
        border: color::ink(0.0, 0.10),
        border_strong: color::ink(0.0, 0.17),
        divider: color::ink(0.0, 0.10),

        // ~neutral-850. Pure neutral-900 measures 17.9:1 on white — *more*
        // contrast than dark mode's 16.1:1, which reads as harsh rather than
        // crisp. Backing off to 0.25 lands at ~16:1: the same perceived weight
        // as the dark theme, not the maximum available.
        text: color::neutral(0.25),
        text_muted: color::neutral(0.439),
        // A touch darker than dark mode's counterpart: the light sidebar is a
        // real grey, and faint text has to clear its floor there too.
        text_faint: color::neutral(0.535),
        text_dim: color::neutral(0.50),

        solid: color::neutral(0.205),
        on_solid: color::neutral(0.985),
        accent: color::neutral(0.511),
        accent_strong: color::neutral(0.205),
        on_accent: color::neutral(0.985),

        danger: color::oklch(0.577, 0.245, 27.325),
        danger_strong: color::oklch(0.51, 0.20, 25.0),
        danger_muted: color::oklch(0.505, 0.213, 27.518),
        // Amber-700 carries 12px text, which amber-500 was never going to.
        warning: color::oklch(0.555, 0.163, 48.998),
        warning_muted: color::oklch(0.473, 0.137, 46.201),
        success: color::oklch(0.596, 0.145, 163.225),
        success_muted: color::oklch(0.508, 0.118, 165.612),
        busy: color::oklch(0.592, 0.249, 0.584),

        input_bg: color::grey(0xff),
        band: band_for(light),
        selection: color::hsl(0.66, 0.75, 0.62, 0.28),
        cursor: color::ink(0.0, 0.55),
        caret: color::neutral(0.511),
        ring: hairline_for(light, 0.35),
        code_text: color::neutral(0.18),
        code_wash: color::ink(0.0, 0.06),

        diff_add: color::oklch(0.596, 0.145, 163.225),
        diff_del: color::oklch(0.577, 0.245, 27.325),
        diff_hunk_bg: color::hsl(0.6, 0.35, 0.35, 0.07),
    }
}

/// The shipped palette for an appearance.
pub fn for_appearance(appearance: Appearance) -> Paint {
    match appearance {
        Appearance::Dark => dark(),
        Appearance::Light => light(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Contrast floors the palette is held to, per pairing.
    const AA_TEXT: f32 = 4.5;
    /// A control edge only has to be perceivable, not readable.
    const EDGE: f32 = 1.2;

    fn both() -> [(&'static str, Appearance); 2] {
        [("dark", Appearance::Dark), ("light", Appearance::Light)]
    }

    #[test]
    fn test_body_text_clears_wcag_aa_on_every_surface() {
        for (name, appearance) in both() {
            let p = for_appearance(appearance);
            for (plane, tone) in [
                ("bg", p.bg),
                ("surface", p.surface),
                ("card", p.surface_card),
                ("dialog", p.surface_dialog),
                ("overlay", p.surface_overlay),
            ] {
                let ratio = color::contrast_ratio(p.text, tone);
                assert!(ratio >= AA_TEXT, "{name}/{plane}: {ratio}");
            }
        }
    }

    #[test]
    fn test_muted_and_faint_text_are_readable_but_quieter_than_body() {
        for (name, appearance) in both() {
            let p = for_appearance(appearance);
            let body = color::contrast_ratio(p.text, p.bg);
            let muted = color::contrast_ratio(p.text_muted, p.bg);
            let faint = color::contrast_ratio(p.text_faint, p.bg);
            assert!(muted >= AA_TEXT, "{name} muted: {muted}");
            // Faint is deliberately below AA: it carries placeholders and
            // timestamps, never the only copy of anything.
            assert!(faint >= 3.0, "{name} faint: {faint}");
            assert!(body > muted && muted > faint, "{name}: {body}/{muted}/{faint}");
        }
    }

    #[test]
    fn test_light_reproduces_dark_contrast_rather_than_maximising_it() {
        // The reason light's text is neutral(0.25) and not neutral(0.15): it
        // should read with the same weight, not more.
        let d = color::contrast_ratio(dark().text, dark().bg);
        let l = color::contrast_ratio(light().text, light().bg);
        assert!((d - l).abs() < 1.5, "dark {d} vs light {l}");
    }

    #[test]
    fn test_plate_labels_clear_aa_on_their_plate() {
        for (name, appearance) in both() {
            let p = for_appearance(appearance);
            let solid = color::contrast_ratio(p.on_solid, p.solid);
            let accent = color::contrast_ratio(p.on_accent, p.accent_strong);
            assert!(solid >= AA_TEXT, "{name} on_solid: {solid}");
            assert!(accent >= AA_TEXT, "{name} on_accent: {accent}");
        }
    }

    #[test]
    fn test_status_colours_are_readable_on_the_main_panel() {
        for (name, appearance) in both() {
            let p = for_appearance(appearance);
            for (token, tone) in [
                ("danger", p.danger),
                ("warning", p.warning),
                ("success", p.success),
                ("busy", p.busy),
            ] {
                let ratio = color::contrast_ratio(tone, p.bg);
                assert!(ratio >= 3.0, "{name}/{token}: {ratio}");
            }
        }
    }

    #[test]
    fn test_dark_separates_surfaces_by_lightness() {
        // Dark has headroom above its page, so every plane is its own tone.
        let p = for_appearance(Appearance::Dark);
        assert_ne!(p.surface, p.surface_card);
        assert_ne!(p.surface_card, p.surface_dialog);
        assert_ne!(p.surface_dialog, p.surface_overlay);
        assert!(p.surface.x < p.surface_card.x, "dark elevates upward");
    }

    #[test]
    fn test_light_separates_surfaces_by_edge_where_it_has_no_lightness_to_spend() {
        // Light's card, dialog and overlay are all the page's own white: it has
        // run out of lightness, so elevation is carried by border and shadow.
        // The shell is the grey one, which is what stops the window being one
        // flat sheet with a hairline drawn on it.
        let p = for_appearance(Appearance::Light);
        assert!(p.surface_card.x > p.surface.x, "the card is brighter than the shell");
        assert_eq!(p.surface_card, p.surface_dialog);
        assert_eq!(p.surface_dialog, p.surface_overlay);
        assert!(p.border.w > 0.0);
    }

    #[test]
    fn test_borders_are_visible_against_their_ground() {
        for (name, appearance) in both() {
            let p = for_appearance(appearance);
            let strong = color::contrast_ratio(
                color::flatten(p.border_strong, p.surface),
                p.surface,
            );
            assert!(strong >= EDGE, "{name} border_strong: {strong}");
            // border_strong must out-read border, or a focused field looks
            // exactly like an idle one.
            let plain =
                color::contrast_ratio(color::flatten(p.border, p.surface), p.surface);
            assert!(strong > plain, "{name}: {strong} vs {plain}");
        }
    }

    #[test]
    fn test_caret_and_ring_are_visible() {
        for (name, appearance) in both() {
            let p = for_appearance(appearance);
            let caret = color::contrast_ratio(p.caret, p.bg);
            let ring = color::contrast_ratio(color::flatten(p.ring, p.bg), p.bg);
            assert!(caret > 3.0, "{name} caret: {caret}");
            assert!(ring >= EDGE, "{name} ring: {ring}");
        }
    }

    #[test]
    fn test_selection_composites_to_a_readable_pair() {
        for (name, appearance) in both() {
            let p = for_appearance(appearance);
            // Both tokens are translucent and both sit on the page, in order:
            // the field's own ground first, then the selection over it. Testing
            // the selection against the *unflattened* input ground compares two
            // see-through colours and reports a meaningless 1.38.
            let field = color::flatten(p.input_bg, p.bg);
            let selected = color::flatten(p.selection, field);
            let ratio = color::contrast_ratio(p.text, selected);
            assert!(ratio >= AA_TEXT, "{name} selected text: {ratio}");
            // ...and the selection must actually be visible against the field.
            let bare = color::contrast_ratio(selected, field);
            assert!(bare > 1.1, "{name} selection is invisible: {bare}");
        }
    }

    #[test]
    fn test_translucent_tokens_are_actually_translucent() {
        // These composite over glass and scrims; opaque values here would bake
        // in an assumed background and break over anything else.
        for (name, appearance) in both() {
            let p = for_appearance(appearance);
            for (token, tone) in [
                ("element_hover", p.element_hover),
                ("element_active", p.element_active),
                ("border", p.border),
                ("band", p.band),
                ("code_wash", p.code_wash),
            ] {
                assert!(tone.w < 1.0, "{name}/{token} alpha {}", tone.w);
            }
        }
    }

    #[test]
    fn test_hunk_and_code_washes_stay_subtle() {
        for (name, appearance) in both() {
            let p = for_appearance(appearance);
            assert!(p.diff_hunk_bg.w < 0.15, "{name}");
            assert!(p.code_wash.w < 0.15, "{name}");
        }
    }

    #[test]
    fn test_diffs_are_opposite_hues() {
        for (name, appearance) in both() {
            let p = for_appearance(appearance);
            // Addition is green-dominant, deletion red-dominant, in both.
            assert!(p.diff_add.y > p.diff_add.x, "{name} add");
            assert!(p.diff_del.x > p.diff_del.y, "{name} del");
        }
    }

    #[test]
    fn test_light_is_not_an_inversion_of_dark() {
        // Neutral-role reassignment, not a mirror: the card plane is white in
        // light and near-black in dark, and the *shell* is the grey one.
        let d = dark();
        let l = light();
        assert!(l.bg.x > d.bg.x);
        assert!(l.surface.x < l.bg.x, "light's shell must recede below its page");
        assert!(d.surface.x > d.bg.x, "dark's shell must rise above its page");
    }
}
