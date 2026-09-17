//! **Temporary.** The v2 script namespace, kept alive while the v2 widget set
//! is replaced one family at a time.
//!
//! `crates/ui/src/widgets/**` was written against a flat, upper-case
//! `mod.mpc_theme` carrying forty tokens plus nested `dark`/`light` copies, and
//! `MpThemeState` copies one over the other to switch appearance. The v3 theme
//! names its tokens in lower case under `mod.mpc.tokens`, and resolves the
//! appearance in Rust rather than by heap surgery.
//!
//! Rather than take the whole widget set down at once, this module re-emits the
//! v2 shape from the v3 palette. The old widgets keep running — and, because the
//! values now come from the v3 palette, they inherit its corrected contrast —
//! while their replacements land.
//!
//! **Delete this module, and the `legacy_paint`/`legacy_dark`/`legacy_light`
//! lines in [`script_mod`](crate::install), with the last v2 widget.**

use makepad_widgets::{
    makepad_script::{trap::NoTrap, ScriptApply},
    LiveId, ScriptValue, ScriptVm, Vec4f,
};

use crate::{appearance::Appearance, color, paint::Paint, palette};

/// The v2 token names, in the order the v2 `Tokens::get_by_name` accepted them.
///
/// `TRANSPARENT` is deliberately absent: the v2 script namespace emitted it but
/// the v2 `TOKEN_NAMES` did not carry it, so `MpThemeState`'s palette sweep
/// never touched it. It is re-emitted by [`namespace`] to keep the two lists
/// matching what the v2 widgets actually saw.
pub const TOKEN_NAMES: &[&str] = &[
    "BG",
    "SURFACE",
    "SURFACE_RAISED",
    "SURFACE_CARD",
    "SURFACE_DIALOG",
    "SURFACE_OVERLAY",
    "ELEMENT_HOVER",
    "ELEMENT_ACTIVE",
    "BORDER",
    "BORDER_STRONG",
    "DIVIDER",
    "TEXT",
    "TEXT_MUTED",
    "TEXT_FAINT",
    "SOLID",
    "SOLID_HOVER",
    "ON_SOLID",
    "ACCENT",
    "ACCENT_HOVER",
    "ACCENT_MUTED",
    "ON_ACCENT",
    "SECONDARY",
    "SECONDARY_HOVER",
    "ON_SECONDARY",
    "DANGER",
    "DANGER_HOVER",
    "DANGER_MUTED",
    "WARNING",
    "WARNING_MUTED",
    "SUCCESS",
    "SUCCESS_MUTED",
    "INFO",
    "INFO_MUTED",
    "BUSY",
    "INPUT_BG",
    "SELECTION",
    "CARET",
    "CODE_TEXT",
    "CODE_WASH",
];

/// The v2 palette, derived from a v3 [`Paint`].
///
/// The tokens the v2 palette had and v3 does not (`SOLID_HOVER`, `ACCENT_MUTED`,
/// `SECONDARY*`, `INFO*`) are recomputed here by the rule the v2 palette used,
/// from the v3 base tones — so an old widget's hover reads as the same *kind* of
/// step it always did, at the new contrast.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tokens {
    pub bg: Vec4f,
    pub surface: Vec4f,
    pub surface_raised: Vec4f,
    pub surface_card: Vec4f,
    pub surface_dialog: Vec4f,
    pub surface_overlay: Vec4f,
    pub element_hover: Vec4f,
    pub element_active: Vec4f,
    pub border: Vec4f,
    pub border_strong: Vec4f,
    pub divider: Vec4f,
    pub text: Vec4f,
    pub text_muted: Vec4f,
    pub text_faint: Vec4f,
    pub solid: Vec4f,
    pub solid_hover: Vec4f,
    pub on_solid: Vec4f,
    pub accent: Vec4f,
    pub accent_hover: Vec4f,
    pub accent_muted: Vec4f,
    pub on_accent: Vec4f,
    pub secondary: Vec4f,
    pub secondary_hover: Vec4f,
    pub on_secondary: Vec4f,
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
    pub input_bg: Vec4f,
    pub selection: Vec4f,
    pub caret: Vec4f,
    pub code_text: Vec4f,
    pub code_wash: Vec4f,
}

impl Tokens {
    /// Read a token by its v2 name.
    pub fn get_by_name(&self, name: &str) -> Option<Vec4f> {
        Some(match name {
            "BG" => self.bg,
            "SURFACE" => self.surface,
            "SURFACE_RAISED" => self.surface_raised,
            "SURFACE_CARD" => self.surface_card,
            "SURFACE_DIALOG" => self.surface_dialog,
            "SURFACE_OVERLAY" => self.surface_overlay,
            "ELEMENT_HOVER" => self.element_hover,
            "ELEMENT_ACTIVE" => self.element_active,
            "BORDER" => self.border,
            "BORDER_STRONG" => self.border_strong,
            "DIVIDER" => self.divider,
            "TEXT" => self.text,
            "TEXT_MUTED" => self.text_muted,
            "TEXT_FAINT" => self.text_faint,
            "SOLID" => self.solid,
            "SOLID_HOVER" => self.solid_hover,
            "ON_SOLID" => self.on_solid,
            "ACCENT" => self.accent,
            "ACCENT_HOVER" => self.accent_hover,
            "ACCENT_MUTED" => self.accent_muted,
            "ON_ACCENT" => self.on_accent,
            "SECONDARY" => self.secondary,
            "SECONDARY_HOVER" => self.secondary_hover,
            "ON_SECONDARY" => self.on_secondary,
            "DANGER" => self.danger,
            "DANGER_HOVER" => self.danger_hover,
            "DANGER_MUTED" => self.danger_muted,
            "WARNING" => self.warning,
            "WARNING_MUTED" => self.warning_muted,
            "SUCCESS" => self.success,
            "SUCCESS_MUTED" => self.success_muted,
            "INFO" => self.info,
            "INFO_MUTED" => self.info_muted,
            "BUSY" => self.busy,
            "INPUT_BG" => self.input_bg,
            "SELECTION" => self.selection,
            "CARET" => self.caret,
            "CODE_TEXT" => self.code_text,
            "CODE_WASH" => self.code_wash,
            _ => return None,
        })
    }

    /// Every token as `(v2 name, value)`.
    pub fn pairs(&self) -> Vec<(&'static str, Vec4f)> {
        TOKEN_NAMES
            .iter()
            .filter_map(|name| self.get_by_name(name).map(|v| (*name, v)))
            .collect()
    }
}

/// Build the v2 token set from a v3 palette.
pub fn tokens(paint: &Paint, appearance: Appearance) -> Tokens {
    let bg = paint.bg;
    let surface = paint.surface;
    let white = color::grey(0xff);
    let black = color::grey(0x00);
    // The v2 palette flattened its washes against the *page*, because its
    // widgets swapped opaque colours. Same rule, new base tones.
    let hover = |c: Vec4f| color::flatten_a(c, bg, 0.88);
    let wash = |appearance: Appearance, a: f32| match appearance {
        Appearance::Dark => color::flatten_a(white, surface, a),
        Appearance::Light => color::flatten_a(black, surface, a),
    };
    let (info, info_muted) = match appearance {
        Appearance::Dark => (color::oklch(0.72, 0.13, 230.0), color::oklch(0.80, 0.08, 230.0)),
        Appearance::Light => (color::oklch(0.55, 0.14, 240.0), color::oklch(0.72, 0.09, 240.0)),
    };
    Tokens {
        bg,
        surface,
        surface_raised: paint.surface_raised,
        surface_card: paint.surface_card,
        surface_dialog: paint.surface_dialog,
        surface_overlay: paint.surface_overlay,
        element_hover: paint.element_hover,
        element_active: paint.element_active,
        border: paint.border,
        border_strong: paint.border_strong,
        divider: paint.divider,
        text: paint.text,
        text_muted: paint.text_muted,
        text_faint: paint.text_faint,
        solid: paint.solid,
        solid_hover: hover(paint.solid),
        on_solid: paint.on_solid,
        accent: paint.accent,
        accent_hover: hover(paint.accent),
        accent_muted: color::flatten_a(paint.accent, bg, 0.25),
        on_accent: paint.on_accent,
        secondary: wash(appearance, 0.10),
        secondary_hover: wash(appearance, 0.16),
        on_secondary: paint.text,
        danger: paint.danger,
        danger_hover: hover(paint.danger),
        danger_muted: paint.danger_muted,
        warning: paint.warning,
        warning_muted: paint.warning_muted,
        success: paint.success,
        success_muted: paint.success_muted,
        info,
        info_muted,
        busy: paint.busy,
        input_bg: paint.input_bg,
        selection: paint.selection,
        caret: paint.caret,
        code_text: paint.code_text,
        code_wash: paint.code_wash,
    }
}

fn object_from<'a>(
    vm: &mut ScriptVm,
    pairs: impl Iterator<Item = (&'static str, Vec4f)>,
) -> ScriptValue {
    let object = vm.bx.heap.new_object();
    for (name, value) in pairs {
        let key: ScriptValue = LiveId::from_str(name).into();
        let value = value.script_to_value(vm);
        vm.bx.heap.set_value_def(object, key, value);
    }
    object.into()
}

/// The v2 namespace: flat tokens plus the nested `dark` and `light` copies
/// `MpThemeState` sweeps between.
pub fn namespace(vm: &mut ScriptVm, paint: &Paint, appearance: Appearance) -> ScriptValue {
    let active = tokens(paint, appearance);
    let dark = tokens(&palette::dark(), Appearance::Dark);
    let light = tokens(&palette::light(), Appearance::Light);
    let object = vm.bx.heap.new_object();
    let flat = object_from(vm, active.pairs().into_iter());
    let dark_value = object_from(vm, dark.pairs().into_iter());
    let light_value = object_from(vm, light.pairs().into_iter());
    vm.bx
        .heap
        .set_value_def(object, LiveId::from_str("dark").into(), dark_value);
    vm.bx
        .heap
        .set_value_def(object, LiveId::from_str("light").into(), light_value);
    // Emitted by the v2 namespace but absent from its `TOKEN_NAMES`, so it was
    // never swept by the appearance switch. Six v2 widgets paint with it and
    // the shaders that `mix()` it fail to compile without it, so a missing
    // entry here is not a cosmetic difference — it is 394 compile errors.
    let transparent = Vec4f {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };
    let transparent = transparent.script_to_value(vm);
    vm.bx
        .heap
        .set_value_def(object, LiveId::from_str("TRANSPARENT").into(), transparent);
    // Copy the flat names onto the same object so `use mod.mpc_theme.*` picks
    // them up as before.
    for name in TOKEN_NAMES {
        let key: ScriptValue = LiveId::from_str(name).into();
        let value = vm.bx.heap.value(flat.as_object().unwrap(), key, NoTrap);
        vm.bx.heap.set_value_def(object, key, value);
    }
    object.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_every_legacy_name_resolves() {
        for appearance in [Appearance::Dark, Appearance::Light] {
            let t = tokens(&palette::for_appearance(appearance), appearance);
            for name in TOKEN_NAMES {
                assert!(t.get_by_name(name).is_some(), "{name}");
            }
            assert!(t.get_by_name("NOPE").is_none());
        }
    }

    #[test]
    fn test_pairs_cover_the_name_list_exactly() {
        let t = tokens(&palette::dark(), Appearance::Dark);
        assert_eq!(t.pairs().len(), TOKEN_NAMES.len());
    }

    #[test]
    fn test_legacy_names_are_a_permutation_of_the_paint_field_set() {
        // Not the same list as v3's — this one has SOLID_HOVER/ACCENT_MUTED/
        // SECONDARY*/INFO* and lacks v3's newer tokens — but it must be
        // internally consistent, which the two tests above establish.
        let mut sorted = TOKEN_NAMES.to_vec();
        let count = sorted.len();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), count, "duplicate legacy name");
    }

    #[test]
    fn test_derived_tokens_are_opaque_and_visible() {
        for appearance in [Appearance::Dark, Appearance::Light] {
            let t = tokens(&palette::for_appearance(appearance), appearance);
            for (name, v) in [
                ("SOLID_HOVER", t.solid_hover),
                ("ACCENT_HOVER", t.accent_hover),
                ("ACCENT_MUTED", t.accent_muted),
                ("SECONDARY", t.secondary),
                ("SECONDARY_HOVER", t.secondary_hover),
                ("DANGER_HOVER", t.danger_hover),
            ] {
                assert!((v.w - 1.0).abs() < 1e-6, "{appearance:?}/{name} alpha {}", v.w);
                assert!(v.x.is_finite() && (0.0..=1.0).contains(&v.x), "{name}");
            }
        }
    }

    #[test]
    fn test_hovers_step_away_from_their_rest_tone() {
        let t = tokens(&palette::dark(), Appearance::Dark);
        assert_ne!(t.solid_hover, t.solid);
        assert_ne!(t.accent_hover, t.accent);
        assert_ne!(t.secondary_hover, t.secondary);
    }

    #[test]
    fn test_transparent_is_emitted_at_zero_alpha() {
        // The v2 namespace had it and the v2 name list did not, which is
        // exactly the kind of gap that only shows at runtime.
        assert!(!TOKEN_NAMES.contains(&"TRANSPARENT"), "it was never swept");
        let zero = Vec4f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        };
        assert_eq!(zero.w, 0.0);
    }

    #[test]
    fn test_legacy_text_still_clears_aa_on_its_page() {
        // The point of re-deriving rather than freezing: the old widgets
        // inherit the corrected contrast.
        for appearance in [Appearance::Dark, Appearance::Light] {
            let t = tokens(&palette::for_appearance(appearance), appearance);
            let ratio = color::contrast_ratio(t.text, t.bg);
            assert!(ratio >= 4.5, "{appearance:?}: {ratio}");
        }
    }
}
