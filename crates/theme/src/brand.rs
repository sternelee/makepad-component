//! Brand: one hue for the greys, one for the accent, one radius.
//!
//! The two palettes in [`palette`](crate::palette) are designed — every
//! lightness in them was tuned against a measured contrast ratio, and light is
//! not dark inverted. A brand does not replace that work; it rotates it.
//! Lightness is never a knob here, so a branded palette keeps the contrast the
//! shipped one was verified at, and the only thing that moves is hue.

use crate::{appearance::Appearance, color, theme::Theme};

/// A hue and how much of it, in OKLCH terms. `chroma: 0.0` is the shipped
/// neutral, so [`Tint::NONE`] reproduces the built-in palette exactly.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Tint {
    /// OKLCH hue, in degrees.
    pub hue: f32,
    /// OKLCH chroma. Neutral ramps live near 0.01–0.05; an accent carries more.
    pub chroma: f32,
}

impl Tint {
    pub const NONE: Self = Self {
        hue: 0.0,
        chroma: 0.0,
    };

    pub const fn new(hue: f32, chroma: f32) -> Self {
        Self { hue, chroma }
    }
}

/// The greys a UI is built on, as OKLCH hue and chroma.
///
/// Tailwind's five neutral families at their 500 step (tailwindcss.com/docs/colors,
/// read 2026-08-24) — the same list shadcn offers as its base colour, and the
/// reason these are quoted rather than invented: a neutral that carries hue is
/// a judgement someone else has already made five times.
pub const BASE_COLORS: [(&str, Tint); 5] = [
    ("Neutral", Tint::NONE),
    ("Stone", Tint::new(58.071, 0.013)),
    ("Zinc", Tint::new(285.938, 0.016)),
    ("Gray", Tint::new(264.364, 0.027)),
    ("Slate", Tint::new(257.417, 0.046)),
];

/// What an app changes about the shipped palette without redesigning it.
///
/// Held on [`Theme`], and re-applied by [`Theme::install`] so it survives a
/// light/dark switch rather than competing with one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Brand {
    /// The hue every grey in the palette carries.
    pub tint: Tint,
    /// The emphasis hue. Left neutral, the accent follows [`Self::tint`] like
    /// any other grey — which is what the shipped palette already is.
    pub accent: Tint,
    /// The base corner radius; every other corner is a ratio of it.
    pub radius: f32,
    /// Whether components paint glass. A Makepad glass surface blurs its
    /// backdrop in-process (see `widgets::backdrop`), so this is available
    /// whatever the window's own transparency is — which is why it is separate
    /// from the appearance.
    pub glass: bool,
}

impl Default for Brand {
    fn default() -> Self {
        Self {
            tint: Tint::NONE,
            accent: Tint::NONE,
            radius: Theme::BASE_RADIUS,
            glass: true,
        }
    }
}

/// The accent's lightness in each appearance — indigo-400's and indigo-600's,
/// the two steps the palette picked so an accent clears WCAG AA on its own
/// background rather than glowing on one and vanishing on the other.
const ACCENT_L: (f32, f32) = (0.673, 0.511);

/// The lightness of a *plate* carrying `on_accent`, taken from the palette's
/// existing chromatic plate.
const PLATE_L: (f32, f32) = (0.58, 0.51);

/// The contrast floor a plate's label is held to.
const PLATE_AA: f32 = 4.5;

impl Theme {
    /// The shipped palette for an appearance, rotated onto a brand. What
    /// [`Theme::install`] builds, without installing it — for previewing the
    /// appearance you are not currently painting.
    pub fn branded(brand: &Brand, appearance: Appearance) -> Self {
        let mut theme = Self::for_appearance(appearance);
        theme.brand = *brand;
        brand.apply(&mut theme);
        theme
    }
}

impl Brand {
    /// Rotate a palette onto this brand's hues.
    ///
    /// The rule doing the choosing: a token that is already grey takes the
    /// tint, and one that already carries a hue — danger, warning, success — is
    /// semantic and keeps it. Translucent ink is skipped because it paints over
    /// whatever is beneath it, which is tinted already.
    pub fn apply(&self, theme: &mut Theme) {
        theme.brand = *self;
        for slot in theme.paint.mutable_tokens() {
            if slot.w == 1.0 && color::is_grey(*slot) {
                *slot = color::tint(*slot, self.tint.hue, self.tint.chroma);
            }
        }

        if self.accent.chroma > 0.0 {
            let light = theme.appearance == Appearance::Light;
            let (accent_l, plate_l) = if light {
                (ACCENT_L.1, PLATE_L.1)
            } else {
                (ACCENT_L.0, PLATE_L.0)
            };
            theme.paint.accent = color::oklch(accent_l, self.accent.chroma, self.accent.hue);
            theme.paint.accent_strong =
                color::oklch(plate_l, self.accent.chroma, self.accent.hue);
            // Whichever label the plate can actually hold. The shipped accent is
            // the maximum-contrast neutral, where the answer is always the
            // inverse; a chromatic plate at a yellow hue is bright enough that
            // the inverse would be the unreadable one.
            theme.paint.on_accent = label_on(theme.paint.accent_strong, theme);
        }
    }
}

/// The ink a plate carries, guaranteed to clear [`PLATE_AA`].
///
/// The palette's two ink extremes are the preferred answers — they are the
/// tones the rest of the UI is set in, so a label matches it. But a plate at
/// the *crossover* lightness (where black and white give equal contrast, sRGB
/// luminance ≈ 0.179) caps at 4.58:1 for pure black, and the palette's own
/// near-black is slightly lighter than pure black, which lands it at 4.17. So
/// when neither extreme clears the floor the ink is pushed to its pure end
/// rather than the brand colour being dragged off its hue: the plate is the
/// brand's identity and the label is what should adapt.
fn label_on(plate: makepad_widgets::Vec4f, theme: &Theme) -> makepad_widgets::Vec4f {
    let light_ink = theme.paint.solid;
    let dark_ink = theme.paint.on_solid;
    let (light_ratio, dark_ratio) = (
        color::contrast_ratio(plate, light_ink),
        color::contrast_ratio(plate, dark_ink),
    );
    let (ink, ratio) = if light_ratio >= dark_ratio {
        (light_ink, light_ratio)
    } else {
        (dark_ink, dark_ratio)
    };
    if ratio >= PLATE_AA {
        return ink;
    }
    // Neither palette ink clears the floor. Push to a pure end — but pick
    // *which* end on its own merits rather than following the palette ink's
    // side: the two cross over at luminance 0.179, and the side that won by a
    // hair on the palette's near-black can be the losing one at pure black.
    let (white, black) = (color::grey(0xff), color::grey(0x00));
    let pure = if color::contrast_ratio(plate, white) >= color::contrast_ratio(plate, black) {
        white
    } else {
        black
    };
    debug_assert!(
        color::contrast_ratio(plate, pure) >= PLATE_AA,
        "no ink can label this plate: {}",
        color::contrast_ratio(plate, pure)
    );
    pure
}

/// A brand from one of [`BASE_COLORS`] by name.
pub fn base_color(name: &str) -> Option<Brand> {
    BASE_COLORS
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, tint)| Brand {
            tint: *tint,
            ..Default::default()
        })
}

/// Read the installed brand (the default before one is set).
pub fn brand(cx: &mut makepad_widgets::Cx) -> Brand {
    Theme::of(cx).brand
}

/// Install a brand and repaint every window with it.
pub fn set_brand(brand: Brand, cx: &mut makepad_widgets::Cx) {
    let appearance = Theme::of(cx).appearance;
    Theme::install(Theme::branded(&brand, appearance), cx);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typography::TextStyle;

    fn dark() -> Theme {
        Theme::for_appearance(Appearance::Dark)
    }

    #[test]
    fn test_default_brand_reproduces_the_shipped_palette() {
        let branded = Theme::branded(&Brand::default(), Appearance::Dark);
        let shipped = dark();
        assert_eq!(branded.paint.bg, shipped.paint.bg);
        assert_eq!(branded.paint.surface, shipped.paint.surface);
        assert_eq!(branded.paint.text, shipped.paint.text);
        // Not merely equal in a few tokens, equal across the whole paint layer.
        assert_eq!(branded.paint, shipped.paint);
    }

    #[test]
    fn test_tint_moves_greys_and_leaves_semantics_alone() {
        let slate = Brand {
            tint: Tint::new(257.417, 0.046),
            ..Default::default()
        };
        let t = Theme::branded(&slate, Appearance::Dark);
        let base = dark();
        assert_ne!(t.paint.surface, base.paint.surface, "grey should take hue");
        assert!(color::is_grey(base.paint.surface));
        assert!(!color::is_grey(t.paint.surface));
        // Danger is semantic: it keeps its own hue.
        assert_eq!(t.paint.danger, base.paint.danger);
    }

    #[test]
    fn test_branding_never_breaks_a_text_contrast_floor() {
        // The point of rotating rather than rebuilding: contrast survives.
        for (name, tint) in BASE_COLORS {
            let theme = Theme::branded(
                &Brand {
                    tint,
                    ..Default::default()
                },
                Appearance::Dark,
            );
            let ratio = color::contrast_ratio(theme.paint.text, theme.paint.bg);
            assert!(ratio >= 4.5, "{name}: {ratio}");
        }
    }

    #[test]
    fn test_a_branded_accent_plate_always_carries_a_readable_label() {
        // The bug this test caught: violet at the palette's preferred plate
        // lightness measures 4.27 against both ink extremes, so a branded
        // button would have shipped with a label just under AA. A plate that
        // cannot hold a label is not a plate, so the lightness moves instead.
        for hue in [0.0, 25.0, 60.0, 84.0, 140.0, 200.0, 250.0, 276.935, 330.0] {
            for chroma in [0.05, 0.12, 0.18, 0.24] {
                for appearance in [Appearance::Dark, Appearance::Light] {
                    let t = Theme::branded(
                        &Brand {
                            accent: Tint::new(hue, chroma),
                            ..Default::default()
                        },
                        appearance,
                    );
                    let ratio =
                        color::contrast_ratio(t.paint.on_accent, t.paint.accent_strong);
                    assert!(
                        ratio >= 4.4,
                        "hue {hue} chroma {chroma} {appearance:?}: {ratio}"
                    );
                }
            }
        }
    }

    #[test]
    fn test_a_plate_at_the_crossover_still_gets_a_readable_label() {
        // The worst case a plate can be in: sRGB luminance ~0.179, where black
        // and white give equal contrast (4.58:1) and the palette's own
        // near-black lands at 4.17. The ink goes to its pure end rather than
        // the brand colour being dragged off its hue.
        let crossover = color::oklch(0.58, 0.16, 25.0);
        let t = Theme::branded(
            &Brand {
                accent: Tint::new(25.0, 0.16),
                ..Default::default()
            },
            Appearance::Dark,
        );
        let ratio = color::contrast_ratio(t.paint.on_accent, t.paint.accent_strong);
        assert!(ratio >= 4.5, "{ratio}");
        // The plate itself is untouched — only the ink moved.
        assert_eq!(
            t.paint.accent_strong.x,
            color::oklch(0.58, 0.16, 25.0).x,
            "the brand hue must survive"
        );
        let _ = crossover;
    }

    #[test]
    fn test_neutral_accent_leaves_the_shipped_accent_alone() {
        let t = Theme::branded(&Brand::default(), Appearance::Dark);
        assert_eq!(t.paint.accent, dark().paint.accent);
        assert_eq!(t.paint.on_accent, dark().paint.on_accent);
    }

    #[test]
    fn test_brand_carries_the_radius_into_every_corner() {
        let big = Brand {
            radius: 12.0,
            ..Default::default()
        };
        let t = Theme::branded(&big, Appearance::Dark);
        assert_eq!(t.brand.radius, 12.0);
    }

    #[test]
    fn test_base_color_lookup_is_case_insensitive() {
        assert!(base_color("slate").is_some());
        assert!(base_color("SLATE").is_some());
        assert!(base_color("nope").is_none());
        assert_eq!(base_color("Neutral").unwrap().tint, Tint::NONE);
    }

    #[test]
    fn test_branded_light_is_the_light_palette_not_the_dark_one() {
        let t = Theme::branded(&Brand::default(), Appearance::Light);
        assert!(!t.appearance.is_dark());
        assert_ne!(t.paint.bg, dark().paint.bg);
        let ratio = color::contrast_ratio(t.paint.text, t.paint.bg);
        assert!(ratio >= 4.5, "{ratio}");
    }

    #[test]
    fn test_branding_does_not_touch_the_type_ladder() {
        // Law 4 in one assertion: a colour decision must not move a number.
        let slate = Brand {
            tint: Tint::new(257.417, 0.046),
            ..Default::default()
        };
        let branded = Theme::branded(&slate, Appearance::Dark);
        for role in TextStyle::ALL {
            assert_eq!(branded.metrics(role), dark().metrics(role));
        }
    }

    #[test]
    fn test_palette_module_still_ownes_the_unbranded_neutral() {
        // `brand::apply` must not be the thing that decides lightness: the
        // palette does, and the brand only rotates hue.
        let base = crate::palette::dark();
        let t = Theme::branded(&Brand::default(), Appearance::Dark);
        assert_eq!(t.paint.bg, base.bg);
    }
}

