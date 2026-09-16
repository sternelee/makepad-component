//! Color math: OKLCH → sRGB, WCAG contrast, tinting and compositing.
//!
//! Palettes are authored in OKLCH (perceptually uniform) and baked to
//! gamma-encoded sRGB at construction time, because that is what the shaders
//! sample. Every colour in this crate is a [`Vec4f`] with `w` as alpha.
//!
//! Contrast pairing is enforced by unit tests, not by eye.

use makepad_widgets::Vec4f;

/// An opaque colour from 8-bit sRGB channels — `rgb(0x0d, 0x0d, 0x0d)`.
pub const fn rgb(r: u8, g: u8, b: u8) -> Vec4f {
    Vec4f {
        x: r as f32 / 255.0,
        y: g as f32 / 255.0,
        z: b as f32 / 255.0,
        w: 1.0,
    }
}

/// An exact achromatic tone from an 8-bit channel value (`grey(13)` ≡ `#0d0d0d`)
/// — for surfaces matched against reference-screenshot samples.
pub const fn grey(value: u8) -> Vec4f {
    Vec4f {
        x: value as f32 / 255.0,
        y: value as f32 / 255.0,
        z: value as f32 / 255.0,
        w: 1.0,
    }
}

/// A translucent ink: white at 8% is `ink(1.0, 0.08)`.
pub const fn ink(level: f32, alpha: f32) -> Vec4f {
    Vec4f {
        x: level,
        y: level,
        z: level,
        w: alpha,
    }
}

/// The same colour at a different alpha.
pub const fn with_alpha(color: Vec4f, alpha: f32) -> Vec4f {
    Vec4f { w: alpha, ..color }
}

/// Convert OKLCH (lightness 0..1, chroma, hue degrees) to gamma-encoded sRGB.
pub fn oklch(l: f32, c: f32, h_deg: f32) -> Vec4f {
    let h = h_deg.to_radians();
    oklab_to_srgb(l, c * h.cos(), c * h.sin())
}

/// Neutral colour (zero chroma) at the given OKLCH lightness.
pub fn neutral(l: f32) -> Vec4f {
    oklab_to_srgb(l, 0.0, 0.0)
}

/// An HSL colour, all components 0..1 with hue wrapping — CSS's `hsl()`.
///
/// The palettes author their chromatic tokens in OKLCH and their selection and
/// diff washes in HSL, because a wash is quoted in the units the reference
/// screenshot was sampled in. Both land in the same sRGB space.
pub fn hsl(h: f32, s: f32, l: f32, a: f32) -> Vec4f {
    let [r, g, b] = hsl_to_rgb(h, s, l);
    Vec4f { x: r, y: g, z: b, w: a }
}

/// HSL (all 0..1, hue wrapping) → sRGB components 0..1.
pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [f32; 3] {
    if s <= f32::EPSILON {
        return [l, l, l];
    }
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let hue = |mut t: f32| {
        t = t.rem_euclid(1.0);
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };
    [
        hue(h + 1.0 / 3.0),
        hue(h),
        hue(h - 1.0 / 3.0),
    ]
}

/// Composite `fg` (which may be translucent) over `bg`, returning the opaque
/// result — the colour the eye actually receives.
///
/// Baked washes let a widget swap opaque colours without knowing about
/// blending, which is what keeps layout independent of paint (law 4).
pub fn flatten(fg: Vec4f, bg: Vec4f) -> Vec4f {
    flatten_a(fg, bg, fg.w)
}

/// [`flatten`] at an explicit coverage, ignoring `fg`'s own alpha.
pub fn flatten_a(fg: Vec4f, bg: Vec4f, alpha: f32) -> Vec4f {
    let a = alpha.clamp(0.0, 1.0);
    Vec4f {
        x: bg.x * (1.0 - a) + fg.x * a,
        y: bg.y * (1.0 - a) + fg.y * a,
        z: bg.z * (1.0 - a) + fg.z * a,
        w: 1.0,
    }
}

/// Linear per-component mix of two colours.
pub fn mix(a: Vec4f, b: Vec4f, t: f32) -> Vec4f {
    let t = t.clamp(0.0, 1.0);
    Vec4f {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
        z: a.z + (b.z - a.z) * t,
        w: a.w + (b.w - a.w) * t,
    }
}

/// Whether a colour carries no hue — r, g and b equal to within a whisper.
///
/// [`tint`] leaves chromatic colours alone, which is how `danger` stays red
/// while `surface` takes a brand's hue.
pub fn is_grey(c: Vec4f) -> bool {
    (c.x - c.y).abs() < 1e-4 && (c.y - c.z).abs() < 1e-4
}

/// WCAG 2.1 relative luminance of a gamma-encoded sRGB colour.
pub fn relative_luminance(c: Vec4f) -> f32 {
    let lin = |v: f32| {
        if v <= 0.040_45 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(c.x) + 0.7152 * lin(c.y) + 0.0722 * lin(c.z)
}

/// WCAG 2.1 contrast ratio between two colours (1.0 … 21.0).
pub fn contrast_ratio(a: Vec4f, b: Vec4f) -> f32 {
    let (la, lb) = (relative_luminance(a), relative_luminance(b));
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// The OKLCH lightness of a colour: the inverse of [`neutral`], by binary
/// search because the forward transform is not analytically invertible per
/// channel once the sRGB matrix is involved.
pub fn lightness(c: Vec4f) -> f32 {
    let target = relative_luminance(c);
    let (mut lo, mut hi) = (0.0f32, 1.0f32);
    for _ in 0..24 {
        let mid = 0.5 * (lo + hi);
        if relative_luminance(neutral(mid)) < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// The most chroma sRGB can hold at this lightness and hue, up to `chroma`.
///
/// Near black and near white the gamut is a needle: asking for a mid-ramp
/// chroma there produces an out-of-range component, and clamping it per
/// channel shifts the hue instead of dropping the saturation. Tailwind's
/// neutral ramps taper their chroma at both ends by hand for the same reason;
/// here the taper is whatever the gamut allows, so no ramp is tabulated.
fn fit_chroma(l: f32, chroma: f32, hue: f32) -> f32 {
    let fits = |c: f32| {
        let h = hue.to_radians();
        oklab_to_linear(l, c * h.cos(), c * h.sin())
            .iter()
            .all(|x| (-1e-4..=1.0 + 1e-4).contains(x))
    };
    if fits(chroma) {
        return chroma;
    }
    let (mut lo, mut hi) = (0.0f32, chroma);
    for _ in 0..24 {
        let mid = 0.5 * (lo + hi);
        if fits(mid) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

/// Re-emit a colour at `hue`, carrying as much `chroma` as its lightness can
/// hold, with the same lightness and alpha.
pub fn tint(color: Vec4f, hue: f32, chroma: f32) -> Vec4f {
    if chroma <= 0.0 {
        return color;
    }
    let l = lightness(color);
    let mut out = oklch(l, fit_chroma(l, chroma, hue), hue);
    out.w = color.w;
    out
}

fn oklab_to_srgb(l: f32, a: f32, b: f32) -> Vec4f {
    let [r, g, b] = oklab_to_linear(l, a, b);
    Vec4f {
        x: gamma_encode(r),
        y: gamma_encode(g),
        z: gamma_encode(b),
        w: 1.0,
    }
}

/// OKLab → *linear* sRGB, unclamped: a component outside 0..1 is a colour the
/// display cannot make, which is what [`fit_chroma`] tests for.
fn oklab_to_linear(l: f32, a: f32, b: f32) -> [f32; 3] {
    let l_ = l + 0.396_337_78 * a + 0.215_803_76 * b;
    let m_ = l - 0.105_561_346 * a - 0.063_854_17 * b;
    let s_ = l - 0.089_484_18 * a - 1.291_485_5 * b;
    let (l3, m3, s3) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);

    [
        4.076_741_7 * l3 - 3.307_711_6 * m3 + 0.230_969_93 * s3,
        -1.268_438 * l3 + 2.609_757_4 * m3 - 0.341_319_4 * s3,
        -0.004_196_086_3 * l3 - 0.703_418_6 * m3 + 1.707_614_7 * s3,
    ]
}

fn gamma_encode(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    if x <= 0.003_130_8 {
        12.92 * x
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(c: Vec4f) -> String {
        format!(
            "#{:02x}{:02x}{:02x}",
            (c.x * 255.0).round() as u8,
            (c.y * 255.0).round() as u8,
            (c.z * 255.0).round() as u8
        )
    }

    #[test]
    fn test_oklch_pure_hues_are_recognizable() {
        let red = oklch(0.6, 0.2, 25.0);
        assert!(red.x > red.y && red.x > red.z, "red {}", hex(red));
        let green = oklch(0.7, 0.17, 155.0);
        assert!(
            green.y > green.x && green.y > green.z,
            "green {}",
            hex(green)
        );
        let blue = oklch(0.6, 0.2, 250.0);
        assert!(blue.z > blue.x, "blue {}", hex(blue));
    }

    #[test]
    fn test_neutral_has_equal_channels() {
        let n = neutral(0.5);
        // The OKLab matrix rows do not sum to exactly 1 in float, and
        // `oklab_to_srgb` runs each channel through an independent gamma
        // encode, so a 0.0005 sRGB drift is expected and invisible.
        assert!((n.x - n.y).abs() < 1e-3, "{}", hex(n));
        assert!((n.y - n.z).abs() < 1e-3, "{}", hex(n));
        assert!(is_grey(n), "{}", hex(n));
    }

    #[test]
    fn test_contrast_ratio_black_white_is_21() {
        let r = contrast_ratio(grey(0), grey(255));
        assert!((r - 21.0).abs() < 0.1, "{r}");
    }

    #[test]
    fn test_contrast_ratio_is_symmetric() {
        let a = neutral(0.2);
        let b = neutral(0.9);
        assert!((contrast_ratio(a, b) - contrast_ratio(b, a)).abs() < 1e-6);
    }

    #[test]
    fn test_lightness_round_trips_a_neutral() {
        // `lightness` is a search over the forward transform, so it must find
        // the value it inverted; the tolerance is the search's own step.
        for l in [0.1f32, 0.25, 0.5, 0.75, 0.922] {
            let got = lightness(neutral(l));
            assert!((got - l).abs() < 2e-4, "l={l} got={got}");
        }
    }

    #[test]
    fn test_tint_leaves_greys_grey_at_zero_chroma() {
        let c = neutral(0.5);
        let t = tint(c, 257.4, 0.0);
        assert_eq!((t.x, t.y, t.z), (c.x, c.y, c.z));
    }

    #[test]
    fn test_tint_gives_a_grey_a_hue_and_keeps_its_alpha() {
        let c = with_alpha(neutral(0.5), 0.4);
        let t = tint(c, 257.4, 0.046);
        assert!(!is_grey(t), "{}", hex(t));
        assert!((t.w - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_tint_taper_at_the_extremes_does_not_clip() {
        // Near white the gamut needle cannot hold the requested chroma; the
        // answer must be a colour the display can actually make.
        let t = tint(neutral(0.985), 264.4, 0.027);
        for v in [t.x, t.y, t.z] {
            assert!((0.0..=1.0).contains(&v), "{v} in {}", hex(t));
        }
        assert!(is_grey(t) || (t.x - t.y).abs() < 0.02, "{}", hex(t));
    }

    #[test]
    fn test_hsl_zero_saturation_is_the_lightness() {
        let c = hsl(0.4, 0.0, 0.5, 1.0);
        assert!((c.x - 0.5).abs() < 1e-6);
        assert!((c.y - 0.5).abs() < 1e-6);
        assert!((c.z - 0.5).abs() < 1e-6);
        assert!(is_grey(c));
    }

    #[test]
    fn test_hsl_hue_selects_the_channel() {
        // 0 is red, 1/3 is green, 2/3 is blue.
        let red = hsl(0.0, 1.0, 0.5, 1.0);
        assert!(red.x > red.y && red.x > red.z, "{}", hex(red));
        let green = hsl(1.0 / 3.0, 1.0, 0.5, 1.0);
        assert!(green.y > green.x && green.y > green.z, "{}", hex(green));
        let blue = hsl(2.0 / 3.0, 1.0, 0.5, 1.0);
        assert!(blue.z > blue.x && blue.z > blue.y, "{}", hex(blue));
    }

    #[test]
    fn test_flatten_bakes_translucent_ink_to_opaque() {
        let white_8 = ink(1.0, 0.08);
        let bg = grey(6);
        let out = flatten(white_8, bg);
        assert!((out.w - 1.0).abs() < 1e-6);
        // 8% white over near-black lifts every channel a little.
        assert!(out.x > bg.x, "{} vs {}", hex(out), hex(bg));
        assert!(is_grey(out), "{}", hex(out));
    }

    #[test]
    fn test_flatten_a_uses_the_given_coverage() {
        let out = flatten_a(ink(1.0, 1.0), grey(0), 0.5);
        assert!((out.x - 0.5).abs() < 1e-6, "{}", hex(out));
    }

    #[test]
    fn test_mix_endpoints_are_exact() {
        let a = neutral(0.2);
        let b = neutral(0.8);
        assert_eq!(mix(a, b, 0.0).x, a.x);
        assert_eq!(mix(a, b, 1.0).x, b.x);
        // Out-of-range t clamps rather than extrapolating into broken colours.
        assert_eq!(mix(a, b, 2.0).x, b.x);
        assert_eq!(mix(a, b, -1.0).x, a.x);
    }
}
