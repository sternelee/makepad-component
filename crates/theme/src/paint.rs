//! The paint layer: the colour tokens and the elevation that goes with them.
//!
//! Colours here are *paint only* — nothing in this module is ever read by a
//! layout pass (law 4). Several are deliberately translucent (`element_hover`,
//! `border`, `band`, `code_wash`) so they composite correctly over a glass
//! surface or a scrim instead of being baked against one assumed background.

use makepad_widgets::Vec4f;

/// Every colour a component may paint with.
///
/// Names are the vocabulary; the values come from [`palette`](crate::palette),
/// which is the only module that decides a lightness.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Paint {
    // ---- surface ladder ----
    /// The main panel behind everything.
    pub bg: Vec4f,
    /// The shell / sidebar plane.
    pub surface: Vec4f,
    /// Opaque plate tone — user bubbles, the jump-to-bottom pill.
    pub surface_raised: Vec4f,
    /// Content card sitting inside `surface`.
    pub surface_card: Vec4f,
    /// A dialog's plane.
    pub surface_dialog: Vec4f,
    /// A popover / menu plane, one rung above a dialog.
    pub surface_overlay: Vec4f,
    /// `surface_raised`, one step into hover.
    pub surface_raised_hover: Vec4f,
    /// Translucent fill for interactive states, one rung up.
    pub element_hover: Vec4f,
    /// Translucent fill for the pressed / selected rung above hover.
    pub element_active: Vec4f,

    // ---- hairlines ----
    /// The everyday 1px edge.
    pub border: Vec4f,
    /// An edge that has to hold against its surround — a focused input.
    pub border_strong: Vec4f,
    /// The same weight as [`Paint::border`], named for the seams between rows
    /// so a component reads as a list rather than a box.
    pub divider: Vec4f,

    // ---- ink ladder ----
    /// Body text.
    pub text: Vec4f,
    /// Secondary text.
    pub text_muted: Vec4f,
    /// Tertiary text — placeholders, timestamps, disabled labels.
    pub text_faint: Vec4f,
    /// Between `text` and `text_muted`, for chrome labels.
    pub text_dim: Vec4f,

    // ---- plates ----
    /// The inverted plate: a prominent button's fill.
    pub solid: Vec4f,
    /// The label that plate carries.
    pub on_solid: Vec4f,
    /// The emphasis colour.
    pub accent: Vec4f,
    /// The accent as a *plate* carrying [`Paint::on_accent`].
    pub accent_strong: Vec4f,
    /// The label [`Paint::accent_strong`] carries.
    pub on_accent: Vec4f,

    // ---- status ----
    pub danger: Vec4f,
    /// Deeper danger, for a plate carrying a label.
    pub danger_strong: Vec4f,
    /// A large danger fill: badge background, alert wash.
    pub danger_muted: Vec4f,
    pub warning: Vec4f,
    pub warning_muted: Vec4f,
    pub success: Vec4f,
    pub success_muted: Vec4f,
    /// Work in progress — in-flight, streaming, thinking. Not an error, and
    /// not a success: a third state the status set would otherwise be missing.
    pub busy: Vec4f,

    // ---- component-specific ----
    /// A text field's ground.
    pub input_bg: Vec4f,
    /// A recessed strip behind a palette or picker header.
    pub band: Vec4f,
    /// Selected-text background.
    pub selection: Vec4f,
    /// The I-beam over text.
    pub cursor: Vec4f,
    /// The caret, and the focus ring's colour.
    pub caret: Vec4f,
    /// The focus ring itself.
    pub ring: Vec4f,
    /// Inline code ink.
    pub code_text: Vec4f,
    /// The wash behind inline code and fenced blocks.
    pub code_wash: Vec4f,

    // ---- diffs ----
    pub diff_add: Vec4f,
    pub diff_del: Vec4f,
    pub diff_hunk_bg: Vec4f,
}

/// A drop shadow, as the values a Makepad bg shader reads.
///
/// Bezel expresses elevation as a pair of `BoxShadow`s; Makepad paints one
/// Gaussian through `GaussShadow.rounded_box_shadow`, so a surface carries one
/// [`Shadow`] and the draw rect grows by `radius + offset` to hold it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    pub color: Vec4f,
    /// Gaussian sigma in logical pixels.
    pub sigma: f32,
    /// How far the shadow's field reaches, in logical pixels — the number the
    /// caller expands its draw rect by.
    pub radius: f32,
    /// Downward displacement of the shadow beneath its shape.
    pub offset_y: f32,
}

impl Shadow {
    /// No shadow at all — for a surface that is flush with its ground.
    pub const NONE: Self = Self {
        color: Vec4f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        },
        sigma: 0.0,
        radius: 0.0,
        offset_y: 0.0,
    };

    /// How much a draw rect must grow on every side to contain this shadow.
    ///
    /// Derived from the two numbers that move rather than stored beside them,
    /// so a shadow can never be drawn clipped by its own rect.
    pub fn grow(self) -> f32 {
        self.radius + self.offset_y.abs()
    }

    /// Whether this shadow paints anything.
    pub fn is_visible(self) -> bool {
        self.color.w > 0.0 && self.radius > 0.0
    }
}

impl Paint {
    /// The colour a reader actually sees behind code: the wash composited over the page.
    ///
    /// **Use this, not [`Paint::code_wash`], for anything that measures contrast.** The wash is a
    /// translucent ink — `ink(1.0, 0.08)` in dark — and [`crate::color::contrast_ratio`] treats its
    /// argument as opaque, so measuring against the raw wash measures against **pure white** in
    /// dark and **pure black** in light. That is not a close approximation of the truth; it is a
    /// different colour, and a palette tuned against it is tuned against nothing.
    ///
    /// Found while building `syntax.rs`, where the first contrast test reported 2.38:1 for every
    /// kind in dark mode and looked like a palette fault.
    pub fn code_ground(self) -> Vec4f {
        crate::color::flatten(self.code_wash, self.bg)
    }

    /// Every colour token, mutably, in a fixed order.
    ///
    /// The one place the token list is written down. [`Brand::apply`] walks it
    /// to rotate hues, and `install` walks it to stamp the script heap, so a
    /// token added here appears in both without a second edit.
    ///
    /// [`Brand::apply`]: crate::brand::Brand::apply
    pub fn mutable_tokens(&mut self) -> Vec<&mut Vec4f> {
        vec![
            &mut self.bg,
            &mut self.surface,
            &mut self.surface_raised,
            &mut self.surface_card,
            &mut self.surface_dialog,
            &mut self.surface_overlay,
            &mut self.surface_raised_hover,
            &mut self.element_hover,
            &mut self.element_active,
            &mut self.border,
            &mut self.border_strong,
            &mut self.divider,
            &mut self.text,
            &mut self.text_muted,
            &mut self.text_faint,
            &mut self.text_dim,
            &mut self.solid,
            &mut self.on_solid,
            &mut self.accent,
            &mut self.accent_strong,
            &mut self.on_accent,
            &mut self.danger,
            &mut self.danger_strong,
            &mut self.danger_muted,
            &mut self.warning,
            &mut self.warning_muted,
            &mut self.success,
            &mut self.success_muted,
            &mut self.busy,
            &mut self.input_bg,
            &mut self.band,
            &mut self.selection,
            &mut self.cursor,
            &mut self.caret,
            &mut self.ring,
            &mut self.code_text,
            &mut self.code_wash,
            &mut self.diff_add,
            &mut self.diff_del,
            &mut self.diff_hunk_bg,
        ]
    }

    /// The script name of every token, in [`Paint::mutable_tokens`] order.
    ///
    /// Paired with the walk above so the script heap export and the brand
    /// rotation can never disagree about what the tokens are or what order
    /// they come in.
    pub const TOKEN_NAMES: [&'static str; 40] = [
        "bg",
        "surface",
        "surface_raised",
        "surface_card",
        "surface_dialog",
        "surface_overlay",
        "surface_raised_hover",
        "element_hover",
        "element_active",
        "border",
        "border_strong",
        "divider",
        "text",
        "text_muted",
        "text_faint",
        "text_dim",
        "solid",
        "on_solid",
        "accent",
        "accent_strong",
        "on_accent",
        "danger",
        "danger_strong",
        "danger_muted",
        "warning",
        "warning_muted",
        "success",
        "success_muted",
        "busy",
        "input_bg",
        "band",
        "selection",
        "cursor",
        "caret",
        "ring",
        "code_text",
        "code_wash",
        "diff_add",
        "diff_del",
        "diff_hunk_bg",
    ];

    /// Look a token up by its script name. Used by the appearance switcher,
    /// which copies one palette over another on the script heap.
    pub fn get(&self, name: &str) -> Option<Vec4f> {
        let index = Self::TOKEN_NAMES.iter().position(|n| *n == name)?;
        let mut i = 0;
        for slot in self.read_tokens() {
            if i == index {
                return Some(slot);
            }
            i += 1;
        }
        None
    }

    /// The same order as [`Paint::mutable_tokens`], read-only.
    pub fn read_tokens(&self) -> impl Iterator<Item = Vec4f> + '_ {
        let own = *self;
        (0..Self::TOKEN_NAMES.len()).map(move |i| match i {
            0 => own.bg,
            1 => own.surface,
            2 => own.surface_raised,
            3 => own.surface_card,
            4 => own.surface_dialog,
            5 => own.surface_overlay,
            6 => own.surface_raised_hover,
            7 => own.element_hover,
            8 => own.element_active,
            9 => own.border,
            10 => own.border_strong,
            11 => own.divider,
            12 => own.text,
            13 => own.text_muted,
            14 => own.text_faint,
            15 => own.text_dim,
            16 => own.solid,
            17 => own.on_solid,
            18 => own.accent,
            19 => own.accent_strong,
            20 => own.on_accent,
            21 => own.danger,
            22 => own.danger_strong,
            23 => own.danger_muted,
            24 => own.warning,
            25 => own.warning_muted,
            26 => own.success,
            27 => own.success_muted,
            28 => own.busy,
            29 => own.input_bg,
            30 => own.band,
            31 => own.selection,
            32 => own.cursor,
            33 => own.caret,
            34 => own.ring,
            35 => own.code_text,
            36 => own.code_wash,
            37 => own.diff_add,
            38 => own.diff_del,
            _ => own.diff_hunk_bg,
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_the_code_ground_is_the_wash_composited_and_not_the_wash() {
        // The accessor exists because measuring against the raw wash measures against a different
        // colour — white in dark, black in light. This is the assertion that it does what its doc
        // says, and that the difference is large enough to matter rather than a rounding step.
        for appearance in [crate::appearance::Appearance::Dark, crate::appearance::Appearance::Light]
        {
            let paint = crate::palette::for_appearance(appearance);
            let ground = paint.code_ground();
            let raw = paint.code_wash;
            assert_ne!(ground, raw, "the ground is the raw wash in {appearance:?}");
            assert!(
                (ground.w - 1.0).abs() < 1e-6,
                "the composited ground is still translucent ({})",
                ground.w
            );
            // The raw wash's luminance is 1.0 in dark and 0.0 in light, because those are the
            // colours it is made of — so the gap between the two is the whole page.
            let raw_lum = crate::color::relative_luminance(raw);
            let ground_lum = crate::color::relative_luminance(ground);
            assert!(
                (raw_lum - ground_lum).abs() > 0.3,
                "{appearance:?}: the wash's luminance is {raw_lum:.3} and the ground's is \
                 {ground_lum:.3}, so measuring against one is not measuring against the other"
            );
        }
    }

    use super::*;
    use crate::{appearance::Appearance, theme::Theme};

    #[test]
    fn test_token_names_match_the_mutable_walk() {
        let mut paint = Theme::for_appearance(Appearance::Dark).paint;
        assert_eq!(paint.mutable_tokens().len(), Paint::TOKEN_NAMES.len());
        assert_eq!(paint.read_tokens().count(), Paint::TOKEN_NAMES.len());
    }

    #[test]
    fn test_mutable_walk_and_read_walk_agree_on_order() {
        // If they disagree, `get` returns the wrong token and the appearance
        // switcher paints a surface where a label should be.
        let paint = Theme::for_appearance(Appearance::Dark).paint;
        let read: Vec<Vec4f> = paint.read_tokens().collect();
        let mut mutable = paint;
        let written: Vec<Vec4f> = mutable
            .mutable_tokens()
            .into_iter()
            .map(|v| *v)
            .collect();
        assert_eq!(read, written);
    }

    #[test]
    fn test_get_finds_every_named_token() {
        let paint = Theme::for_appearance(Appearance::Dark).paint;
        for name in Paint::TOKEN_NAMES {
            assert!(paint.get(name).is_some(), "{name}");
        }
        assert!(paint.get("nope").is_none());
    }

    #[test]
    fn test_get_returns_the_named_field() {
        let paint = Theme::for_appearance(Appearance::Dark).paint;
        assert_eq!(paint.get("bg"), Some(paint.bg));
        assert_eq!(paint.get("text"), Some(paint.text));
        assert_eq!(paint.get("diff_hunk_bg"), Some(paint.diff_hunk_bg));
    }

    #[test]
    fn test_shadow_grow_contains_the_offset_it_paints() {
        let s = Shadow {
            color: Vec4f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 0.1,
            },
            sigma: 6.0,
            radius: 15.0,
            offset_y: 10.0,
        };
        assert_eq!(s.grow(), 25.0);
        assert!(s.is_visible());
    }

    #[test]
    fn test_no_shadow_is_invisible_and_grows_nothing() {
        assert!(!Shadow::NONE.is_visible());
        assert_eq!(Shadow::NONE.grow(), 0.0);
    }

    #[test]
    fn test_a_zero_alpha_shadow_is_not_visible_however_large() {
        let s = Shadow {
            color: Vec4f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 0.0,
            },
            sigma: 8.0,
            radius: 20.0,
            offset_y: 4.0,
        };
        assert!(!s.is_visible());
    }
}
