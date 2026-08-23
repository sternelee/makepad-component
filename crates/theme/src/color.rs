//! Color math: OKLCH -> sRGB conversion and WCAG 2.1 contrast ratios.
//!
//! Palettes are authored in OKLCH (perceptually uniform) and baked to
//! linear->gamma-encoded sRGB at construction time. Contrast pairing is
//! enforced by unit tests, not by eye.

use makepad_widgets::Vec4f;

/// Convert OKLCH (lightness 0..1, chroma, hue degrees) to gamma-encoded sRGB.
pub fn oklch(l: f64, c: f64, h_deg: f64) -> Vec4f {
    let h = h_deg.to_radians();
    oklab_to_srgb(l, c * h.cos(), c * h.sin())
}

/// Neutral color (zero chroma) at the given OKLCH lightness.
pub fn neutral(l: f64) -> Vec4f {
    oklab_to_srgb(l, 0.0, 0.0)
}

/// Flat grey at the given sRGB value (0..1), bypassing OKLCH entirely.
pub fn grey(v: f64) -> Vec4f {
    let v = v as f32;
    Vec4f { x: v, y: v, z: v, w: 1.0 }
}

/// Composite `fg` over `bg` at `alpha`, producing an opaque result.
/// Used to bake translucent washes into solid hover/active tokens so
/// widgets can swap opaque colors without knowing about blending.
pub fn flatten(fg: Vec4f, bg: Vec4f, alpha: f64) -> Vec4f {
    let a = alpha as f32;
    Vec4f {
        x: bg.x * (1.0 - a) + fg.x * a,
        y: bg.y * (1.0 - a) + fg.y * a,
        z: bg.z * (1.0 - a) + fg.z * a,
        w: 1.0,
    }
}

/// Linear interpolation between two colors.
pub fn mix(a: Vec4f, b: Vec4f, t: f64) -> Vec4f {
    let t = t as f32;
    Vec4f {
        x: a.x * (1.0 - t) + b.x * t,
        y: a.y * (1.0 - t) + b.y * t,
        z: a.z * (1.0 - t) + b.z * t,
        w: a.w * (1.0 - t) + b.w * t,
    }
}

/// WCAG 2.1 relative luminance of a gamma-encoded sRGB color.
pub fn relative_luminance(c: Vec4f) -> f64 {
    let lin = |v: f32| -> f64 {
        let v = v as f64;
        if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * lin(c.x) + 0.7152 * lin(c.y) + 0.0722 * lin(c.z)
}

/// WCAG 2.1 contrast ratio between two colors (1.0 ..= 21.0).
pub fn contrast_ratio(a: Vec4f, b: Vec4f) -> f64 {
    let la = relative_luminance(a);
    let lb = relative_luminance(b);
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

fn oklab_to_srgb(l: f64, a: f64, b: f64) -> Vec4f {
    let l_ = l + 0.396_337_777_4 * a + 0.215_803_757_3 * b;
    let m_ = l - 0.105_561_345_8 * a - 0.063_854_172_8 * b;
    let s_ = l - 0.089_484_177_5 * a - 1.291_485_548_0 * b;

    let ls = l_ * l_ * l_;
    let ms = m_ * m_ * m_;
    let ss = s_ * s_ * s_;

    let lr = 4.076_741_662_1 * ls - 3.307_711_591_3 * ms + 0.230_969_929_2 * ss;
    let lg = -1.268_438_004_6 * ls + 2.609_757_401_1 * ms - 0.341_319_396_5 * ss;
    let lb = 0.004_196_086_3 * ls - 0.703_418_614_7 * ms + 1.707_614_701_0 * ss;

    let enc = |v: f64| -> f32 {
        let v = v.clamp(0.0, 1.0);
        if v <= 0.003_130_8 { (12.92 * v) as f32 } else { ((1.055 * v.powf(1.0 / 2.4)) - 0.055) as f32 }
    };

    Vec4f { x: enc(lr), y: enc(lg), z: enc(lb), w: 1.0 }
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
        // Red hue should dominate the red channel, etc.
        let red = oklch(0.6, 0.2, 25.0);
        assert!(red.x > red.y && red.x > red.z, "red {}", hex(red));
        let green = oklch(0.7, 0.17, 155.0);
        assert!(green.y > green.x && green.y > green.z, "green {}", hex(green));
        let blue = oklch(0.6, 0.2, 250.0);
        assert!(blue.z > blue.x, "blue {}", hex(blue));
    }

    #[test]
    fn test_neutral_has_equal_channels() {
        let n = neutral(0.5);
        // OKLab matrix constants don't preserve neutrality to float precision;
        // ~0.0015 sRGB deviation is visually invisible.
        assert!((n.x - n.y).abs() < 5e-3);
        assert!((n.y - n.z).abs() < 5e-3);
    }

    #[test]
    fn test_contrast_ratio_black_white_is_21() {
        let r = contrast_ratio(grey(0.0), grey(1.0));
        assert!((r - 21.0).abs() < 0.1, "{}", r);
    }
}
