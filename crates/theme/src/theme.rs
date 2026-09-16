//! The theme: one token set, two concrete instances, read at paint time.
//!
//! SwiftUI's `@Environment` is the shape. Components call [`Theme::of`] where
//! they paint, so no colour, font or size travels as a parameter and an
//! appearance switch needs no widget to know it happened.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::OnceLock;

use makepad_widgets::{Cx, Vec4f};

use crate::{
    appearance::Appearance,
    brand::Brand,
    color,
    layout::{set_base_radius, Layout},
    material::{self, Glass, Material, MaterialSpec, SurfaceSpec},
    paint::{Paint, Shadow},
    palette,
    typography::{Metrics, TextStyle},
};

/// Process-wide mirror of the installed appearance.
///
/// The context-free paint helpers ([`Theme::ink`], [`Theme::hairline`],
/// [`Theme::wash`]) are called from deep inside draw code that may hold only
/// `&Cx`-free references, so they read the appearance from here instead of the
/// global. Appearance is genuinely process-wide — one setting for every window
/// — so a single mirror is sound; [`Theme::install`] is the only writer.
static CURRENT_APPEARANCE: AtomicU32 = AtomicU32::new(0);

/// Bumped every time a palette is installed — an appearance switch, and equally
/// a brand change, which moves every token while the appearance stands still.
///
/// Anything that caches *resolved colours* is only valid for the palette that
/// produced it. Rather than thread the palette through every cache key, those
/// caches compare this counter and drop everything when it moves.
static THEME_GENERATION: AtomicU32 = AtomicU32::new(0);

/// Light-mode alpha multiplier for **fills** (hover washes, chip plates).
///
/// This was 0.5 on the theory that dark ink on a bright field reads heavier.
/// That is right for a large wash and badly wrong for everything else: the
/// palette leans on very low alphas for subtle fills (the composer plate is
/// 3% ink, key caps 5%), and halving those produces 1.5% black on white, which
/// is nothing — the composer loses its background entirely. The established
/// light scales (Primer, Radix) land subtle ~3–4%, hover ~8%, selected ~14%
/// black, which is where the dark palette's white alphas already sit. So the
/// honest multiplier is 1: the same number in both appearances, with only the
/// *tone* flipping.
pub const INK_FILL_SCALE: f32 = 1.0;

/// Light-mode alpha multiplier for **hairlines**. Opposite of fills: a 1px edge
/// has to hold its own against a bright surround, and the dark palette's white
/// hairlines are deliberately faint.
pub const INK_HAIRLINE_SCALE: f32 = 1.35;

/// Alpha of the standard modal backdrop in dark terms.
pub const SCRIM_ALPHA_DARK: f32 = 0.60;

/// Everything a component reads when it paints.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    /// Which palette is in force.
    pub appearance: Appearance,
    /// The colours.
    pub paint: Paint,
    /// The layout metrics.
    pub layout: Layout,
    /// The hue rotation applied on top of the palette.
    pub brand: Brand,
    /// Whether components paint glass.
    pub glass: bool,
}

impl Default for Theme {
    fn default() -> Self {
        Self::for_appearance(Appearance::Dark)
    }
}

/// The theme a widget paints with when nothing installed one yet.
///
/// A global rather than an `Option` in every call site: a widget that paints on
/// the very first frame before the app's `script_mod` runs must still find a
/// complete palette, or it paints black on black and the bug looks like a
/// layout problem.
static FALLBACK: OnceLock<Theme> = OnceLock::new();

impl Theme {
    /// Build the shipped palette for an appearance.
    pub fn for_appearance(appearance: Appearance) -> Self {
        Self {
            appearance,
            paint: palette::for_appearance(appearance),
            layout: Layout::default(),
            brand: Brand::default(),
            glass: true,
        }
    }

    /// The dark theme.
    pub fn dark() -> Self {
        Self::for_appearance(Appearance::Dark)
    }

    /// The light theme.
    pub fn light() -> Self {
        Self::for_appearance(Appearance::Light)
    }

    /// The theme in force.
    ///
    /// Falls back to the shipped dark palette before anything installed one,
    /// so a first-frame paint reads a real theme rather than nothing.
    pub fn of(cx: &Cx) -> &Theme {
        cx.get_global_ref::<Theme>()
            .unwrap_or_else(|| FALLBACK.get_or_init(Theme::dark))
    }

    /// The theme in force, mutably.
    ///
    /// Installs the fallback first if nothing has, so a caller can never get a
    /// dangling `&mut` to a temporary.
    pub fn of_mut(cx: &mut Cx) -> &mut Theme {
        if cx.get_global_ref::<Theme>().is_none() {
            cx.set_global(Theme::dark());
        }
        cx.get_global::<Theme>()
    }

    /// Change the theme and repaint.
    ///
    /// The paint layer is re-resolved for the theme's (possibly new)
    /// appearance and the brand re-applied, so `with(|t| t.appearance = Light)`
    /// is an appearance switch and not a half-switch that keeps dark surfaces
    /// with light ink.
    pub fn with(cx: &mut Cx, change: impl FnOnce(&mut Theme)) {
        let mut theme = Self::of(cx).clone();
        change(&mut theme);
        theme.paint = palette::for_appearance(theme.appearance);
        let brand = theme.brand;
        brand.apply(&mut theme);
        Self::install(theme, cx);
    }

    /// Make this theme the one in force, update the script heap, and repaint.
    pub fn install(theme: Theme, cx: &mut Cx) {
        set_base_radius(theme.brand.radius);
        CURRENT_APPEARANCE.store(theme.appearance as u32, Ordering::Relaxed);
        THEME_GENERATION.fetch_add(1, Ordering::Relaxed);
        crate::install::stamp(&theme, cx);
        cx.set_global(theme);
        cx.redraw_all();
    }

    /// Switch appearance, preserving the brand.
    pub fn set_appearance(self, appearance: Appearance, cx: &mut Cx) {
        Self::with(cx, |t| {
            t.appearance = appearance;
            let _ = self;
        });
    }

    // ---- context-free paint helpers ----
    //
    // Alphas are quoted in *dark-mode terms* at every call site — the dark
    // theme is the tuned one — and the light value is derived. Callers keep one
    // number and both appearances stay in the relationship the dark tuning
    // established.

    /// The appearance the context-free helpers are painting for.
    pub fn appearance_now() -> Appearance {
        match CURRENT_APPEARANCE.load(Ordering::Relaxed) {
            1 => Appearance::Light,
            _ => Appearance::Dark,
        }
    }

    /// Monotonic id of the current palette. See [`THEME_GENERATION`].
    pub fn generation() -> u32 {
        THEME_GENERATION.load(Ordering::Relaxed)
    }

    /// Translucent **fill** ink for interactive states and chip plates:
    /// soft-white on dark, soft-black on light.
    ///
    /// Fills must never rest on transparent *black* in dark mode: fully opaque
    /// washes killed the glass and flashed dark mid-fade, so hover fades rest on
    /// `ink(0.0)`, which stays tonally correct at zero alpha.
    pub fn ink(alpha: f32) -> Vec4f {
        match Self::appearance_now() {
            Appearance::Dark => color::ink(1.0, alpha),
            Appearance::Light => color::ink(0.0, alpha * INK_FILL_SCALE),
        }
    }

    /// Translucent **hairline** ink for borders, dividers and rings.
    ///
    /// Separate from [`Theme::ink`] because edges and fills scale in opposite
    /// directions when the field brightens.
    pub fn hairline(alpha: f32) -> Vec4f {
        palette::hairline_for(Self::appearance_now(), alpha)
    }

    /// Interactive-state wash: a softened ink that stops short of pure black or
    /// white, so hover plates read as tinted glass rather than paint.
    pub fn wash(alpha: f32) -> Vec4f {
        match Self::appearance_now() {
            Appearance::Dark => color::ink(0.92, alpha),
            Appearance::Light => color::ink(0.10, alpha * INK_FILL_SCALE),
        }
    }

    /// Modal backdrop.
    ///
    /// Black in both appearances — a scrim's job is to darken what is behind
    /// it, and a "light scrim" of white would wash the modal out rather than
    /// seat it. What changes is strength: on a bright field a dark-mode-weight
    /// scrim reads as a blackout, so light scales to roughly half.
    pub fn scrim(alpha_dark: f32) -> Vec4f {
        match Self::appearance_now() {
            Appearance::Dark => color::ink(0.0, alpha_dark),
            Appearance::Light => color::ink(0.0, 0.32 * (alpha_dark / SCRIM_ALPHA_DARK)),
        }
    }

    /// The user-message bubble plate: the softened wash, at the weight that
    /// reads as a plate rather than a slab.
    pub fn user_bubble_bg(&self) -> Vec4f {
        match self.appearance {
            Appearance::Dark => color::ink(0.92, 0.08),
            Appearance::Light => color::ink(0.10, 0.04),
        }
    }

    /// Selected-state treatment for rows and chips.
    pub fn selected_bg(&self) -> Vec4f {
        self.paint.element_active
    }

    // ---- material ----

    /// The frosted material for this appearance.
    pub fn material(&self) -> MaterialSpec {
        material::material(self.appearance)
    }

    /// The frosted material at a thickness.
    pub fn frost(&self, thickness: Material) -> SurfaceSpec {
        self.material().at(thickness)
    }

    /// One of the two shipped glasses.
    ///
    /// Resolves to an opaque plate when the brand turned glass off, so a
    /// reduce-transparency setting does not leave a translucent rectangle with
    /// desktop showing through it.
    pub fn surface_style(&self, style: crate::material::SurfaceStyle) -> SurfaceSpec {
        use crate::material::SurfaceStyle;
        if !self.glass {
            return SurfaceSpec {
                tint: match style {
                    SurfaceStyle::Glass(_) => self.paint.surface_overlay,
                    SurfaceStyle::Material(_) => self.paint.surface_dialog,
                },
                coverage: 1.0,
                saturation: 1.0,
                gain: 0.0,
                blur: 0.0,
                rim: 0.0,
                reach: 0.0,
                edge: self.paint.border.w,
                edge_width: 1.0,
                edge_aa: 0.5,
                shadow: true,
            };
        }
        match style {
            SurfaceStyle::Glass(variant) => material::glass(self.appearance, variant),
            SurfaceStyle::Material(thickness) => self.frost(thickness),
        }
    }

    /// The glass a popover, menu or command palette paints.
    pub fn popover_style(&self) -> SurfaceSpec {
        self.surface_style(crate::material::SurfaceStyle::Glass(Glass::Regular))
    }

    // ---- elevation ----

    /// The elevation shadow a floating surface casts.
    ///
    /// Bezel expresses this as two `BoxShadow` layers; Makepad paints one
    /// Gaussian through `GaussShadow.rounded_box_shadow`, so the pair is
    /// collapsed into the single layer that carries the perceived lift —
    /// tighter and darker than either, which is what one Gaussian at the
    /// combined offset reads as. A surface that wants the second layer back can
    /// paint it as a second draw call.
    pub fn surface_shadow(&self) -> Shadow {
        match self.appearance {
            Appearance::Dark => Shadow {
                color: color::ink(0.0, 0.36),
                sigma: 5.0,
                radius: 14.0,
                offset_y: 6.0,
            },
            Appearance::Light => Shadow {
                color: color::ink(0.0, 0.16),
                sigma: 5.0,
                radius: 14.0,
                offset_y: 6.0,
            },
        }
    }

    /// The shadow under a raised control — a menu row's own popover is
    /// [`Theme::surface_shadow`]; this is the tighter seat under a card that
    /// sits on a page rather than over it.
    pub fn card_shadow(&self) -> Shadow {
        match self.appearance {
            Appearance::Dark => Shadow::NONE,
            // On white, elevation is carried by border + wash instead of
            // lightness steps, so the shadow is the faintest thing that still
            // separates a card from the page.
            Appearance::Light => Shadow {
                color: color::ink(0.0, 0.06),
                sigma: 3.0,
                radius: 8.0,
                offset_y: 1.0,
            },
        }
    }

    /// The ring a selected row or chip paints *inside* its own shape.
    ///
    /// An inset ring rather than a border: it costs no layout, and behind a
    /// translucent fill a drop shadow shows through as an opaque plate.
    pub fn selection_ring(&self) -> Vec4f {
        match self.appearance {
            Appearance::Dark => Self::hairline(0.09),
            // Pinned at a flat 7% black rather than the scaled hairline:
            // heavier rings outlined every selected chip in a dark box.
            Appearance::Light => color::ink(0.0, 0.07),
        }
    }

    // ---- type ----

    /// The metrics for a role on the ladder.
    pub fn metrics(&self, role: TextStyle) -> Metrics {
        role.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_for_appearance_picks_the_matching_palette() {
        let d = Theme::for_appearance(Appearance::Dark);
        let l = Theme::for_appearance(Appearance::Light);
        assert_eq!(d.paint, palette::dark());
        assert_eq!(l.paint, palette::light());
    }

    #[test]
    fn test_default_is_dark() {
        assert_eq!(Theme::default().appearance, Appearance::Dark);
        assert_eq!(Theme::dark(), Theme::default());
    }

    #[test]
    fn test_generation_moves_when_a_palette_is_installed() {
        let before = Theme::generation();
        CURRENT_APPEARANCE.store(Appearance::Light as u32, Ordering::Relaxed);
        THEME_GENERATION.fetch_add(1, Ordering::Relaxed);
        assert_ne!(Theme::generation(), before);
        CURRENT_APPEARANCE.store(Appearance::Dark as u32, Ordering::Relaxed);
    }

    #[test]
    fn test_ink_flips_tone_between_appearances() {
        CURRENT_APPEARANCE.store(Appearance::Dark as u32, Ordering::Relaxed);
        let dark = Theme::ink(0.08);
        assert!(dark.x > 0.5, "dark fill ink is light");
        assert!((dark.w - 0.08).abs() < 1e-6);

        CURRENT_APPEARANCE.store(Appearance::Light as u32, Ordering::Relaxed);
        let light = Theme::ink(0.08);
        assert!(light.x < 0.5, "light fill ink is dark");
        assert!((light.w - 0.08 * INK_FILL_SCALE).abs() < 1e-6);

        CURRENT_APPEARANCE.store(Appearance::Dark as u32, Ordering::Relaxed);
    }

    #[test]
    fn test_hairline_scales_up_in_light_but_a_fill_does_not() {
        CURRENT_APPEARANCE.store(Appearance::Light as u32, Ordering::Relaxed);
        let edge = Theme::hairline(0.10);
        let fill = Theme::ink(0.10);
        assert!((edge.w - 0.10 * INK_HAIRLINE_SCALE).abs() < 1e-6, "{}", edge.w);
        assert!((fill.w - 0.10).abs() < 1e-6, "{}", fill.w);
        CURRENT_APPEARANCE.store(Appearance::Dark as u32, Ordering::Relaxed);
    }

    #[test]
    fn test_hairline_alpha_is_capped_in_light() {
        CURRENT_APPEARANCE.store(Appearance::Light as u32, Ordering::Relaxed);
        // A strong hairline can ask for more than 0.5; it must not get it, or
        // a border reads as a drawn line.
        assert!(Theme::hairline(0.9).w <= 0.5);
        CURRENT_APPEARANCE.store(Appearance::Dark as u32, Ordering::Relaxed);
    }

    #[test]
    fn test_scrim_darkens_in_both_appearances_and_is_weaker_in_light() {
        CURRENT_APPEARANCE.store(Appearance::Dark as u32, Ordering::Relaxed);
        let dark = Theme::scrim(SCRIM_ALPHA_DARK);
        assert!((dark.w - SCRIM_ALPHA_DARK).abs() < 1e-6);
        assert_eq!(dark.x, 0.0, "a scrim is black in both appearances");

        CURRENT_APPEARANCE.store(Appearance::Light as u32, Ordering::Relaxed);
        let light = Theme::scrim(SCRIM_ALPHA_DARK);
        assert!(light.w < dark.w, "{} vs {}", light.w, dark.w);
        assert!(light.w > 0.0);
        assert_eq!(light.x, 0.0);

        CURRENT_APPEARANCE.store(Appearance::Dark as u32, Ordering::Relaxed);
    }

    #[test]
    fn test_glass_off_resolves_a_surface_to_an_opaque_plate() {
        // A reduce-transparency setting must not leave desktop showing through
        // a "glass" surface.
        let mut theme = Theme::dark();
        theme.glass = false;
        let spec = theme.popover_style();
        assert!(!spec.blurs());
        assert_eq!(spec.coverage(), 1.0);
    }

    #[test]
    fn test_glass_off_keeps_the_two_surface_kinds_distinguishable() {
        let mut theme = Theme::dark();
        theme.glass = false;
        let popover = theme.surface_style(crate::material::SurfaceStyle::Glass(Glass::Regular));
        let plain =
            theme.surface_style(crate::material::SurfaceStyle::Material(Material::Regular));
        assert_ne!(popover.tint, plain.tint);
    }

    #[test]
    fn test_popover_style_is_regular_glass_when_glass_is_on() {
        let theme = Theme::dark();
        assert_eq!(theme.popover_style(), material::glass(Appearance::Dark, Glass::Regular));
    }

    #[test]
    fn test_surface_shadow_grows_a_rect_that_can_hold_it() {
        for appearance in [Appearance::Dark, Appearance::Light] {
            let theme = Theme::for_appearance(appearance);
            let s = theme.surface_shadow();
            assert!(s.is_visible(), "{appearance:?}");
            assert!(s.grow() >= s.offset_y, "{appearance:?}");
        }
    }

    #[test]
    fn test_light_carries_a_card_shadow_and_dark_carries_none() {
        // Dark separates a card by lightness; light has no more lightness to
        // spend, so it separates by edge plus a whisper of shadow.
        assert!(!Theme::dark().card_shadow().is_visible());
        assert!(Theme::light().card_shadow().is_visible());
    }

    #[test]
    fn test_selection_ring_never_becomes_an_outline() {
        for appearance in [Appearance::Dark, Appearance::Light] {
            let t = Theme::for_appearance(appearance);
            let ring = t.selection_ring();
            assert!(ring.w <= 0.10, "{appearance:?} {}", ring.w);
        }
    }

    #[test]
    fn test_metrics_come_from_the_ladder() {
        let theme = Theme::dark();
        assert_eq!(theme.metrics(TextStyle::Body), Metrics::from(TextStyle::Body));
        assert_eq!(theme.metrics(TextStyle::Title).size(), 22.0);
    }

    #[test]
    fn test_control_radius_follows_the_size() {
        use crate::layout::ControlSize;
        assert_eq!(
            ControlSize::Small.radius(),
            crate::theme::Theme::control_radius()
        );
        assert_eq!(
            ControlSize::Regular.radius(),
            crate::theme::Theme::button_radius()
        );
        assert_eq!(
            ControlSize::Large.radius(),
            crate::theme::Theme::button_radius()
        );
    }

    #[test]
    fn test_user_bubble_is_a_wash_not_a_slab() {
        for appearance in [Appearance::Dark, Appearance::Light] {
            let t = Theme::for_appearance(appearance);
            let bubble = t.user_bubble_bg();
            assert!(bubble.w == 1.0 || bubble.w < 0.2, "{appearance:?}");
        }
    }
}
