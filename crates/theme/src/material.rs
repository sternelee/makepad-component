//! Surfaces: the two shipped glasses, the frost scale, and the numbers that
//! resolve them.
//!
//! A Makepad surface blurs its backdrop in process (`widgets::backdrop` runs a
//! Gaussian mip chain), so this is a real material rather than a tinted
//! rectangle. The numbers below are the measured ones, and the shader that
//! consumes them lives with the surface widget in `ui`.

use crate::appearance::Appearance;

/// SwiftUI's two glasses — `Glass.regular` and `Glass.clear`. A closed variant
/// rather than knobs: Apple exposes no numbers on glass either, only the
/// variant and a tint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Glass {
    /// The everyday material: it blurs what it covers and dims it hard.
    #[default]
    Regular,
    /// Near-transparent — the backdrop reads through, bent only at the rim.
    Clear,
}

/// SwiftUI's frost scale.
///
/// Measured 2026-08-31: the five thicknesses are ONE material at five
/// opacities — the tone implied by `tint / (1 - gain)` holds to within 9%
/// across the scale, and the sigma does not move at all. So this is a knob,
/// not five looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Material {
    UltraThin,
    Thin,
    Regular,
    Thick,
    UltraThick,
}

impl Material {
    /// Every thickness, thinnest first.
    pub const ALL: [Material; 5] = [
        Material::UltraThin,
        Material::Thin,
        Material::Regular,
        Material::Thick,
        Material::UltraThick,
    ];

    /// How much of the backdrop the material covers. Measured off SwiftUI in
    /// dark; the steps come out even to within a point.
    pub fn opacity(self) -> f32 {
        match self {
            Material::UltraThin => 0.440,
            Material::Thin => 0.543,
            Material::Regular => 0.638,
            Material::Thick => 0.737,
            Material::UltraThick => 0.825,
        }
    }

    /// The material's script name.
    pub const fn name(self) -> &'static str {
        match self {
            Material::UltraThin => "ultra_thin",
            Material::Thin => "thin",
            Material::Regular => "regular",
            Material::Thick => "thick",
            Material::UltraThick => "ultra_thick",
        }
    }
}

/// Which surface a caller names. Material and glass are different things with
/// different vocabularies — a material has thickness, a glass has a variant —
/// and they meet only at the numbers they resolve to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SurfaceStyle {
    Material(Material),
    Glass(Glass),
}

impl Default for SurfaceStyle {
    fn default() -> Self {
        Self::Glass(Glass::Regular)
    }
}

/// The frost material, before a thickness picks its opacity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialSpec {
    /// The material's own tone, at full coverage.
    pub tone: makepad_widgets::Vec4f,
    /// Its chroma push, as [`SurfaceSpec::saturation`].
    pub saturation: f32,
    /// Its sigma, which does not move with thickness.
    pub blur: f32,
    /// The hairline at the boundary.
    pub edge: f32,
    pub edge_width: f32,
    /// The coverage ramp at the shape's own boundary. 0.5 is one device pixel
    /// at 2x; 0 is a hard edge.
    pub edge_aa: f32,
}

impl MaterialSpec {
    /// This material at one thickness.
    ///
    /// The transfer is `out = gain * saturated(backdrop) + tint`, so a
    /// thickness buys coverage and nothing else. The tint is pre-multiplied by
    /// that coverage — `tone * opacity` rather than `tone` with an alpha —
    /// because the shader sums it as an opaque offset. Storing the tone
    /// unpremultiplied here is what made an earlier version of this type report
    /// five different tones for one material.
    pub fn at(&self, thickness: Material) -> SurfaceSpec {
        let opacity = thickness.opacity();
        SurfaceSpec {
            gain: 1.0 - opacity,
            saturation: self.saturation,
            tint: crate::color::with_alpha(
                crate::color::mix(
                    makepad_widgets::Vec4f {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        w: 1.0,
                    },
                    self.tone,
                    opacity,
                ),
                1.0,
            ),
            coverage: opacity,
            blur: self.blur,
            rim: 0.0,
            reach: 0.0,
            edge: self.edge,
            edge_width: self.edge_width,
            edge_aa: self.edge_aa,
            // A wash has nothing to bend, so it needs an edge to lift off the
            // page rather than a shadow.
            shadow: true,
        }
    }
}

/// One surface's numbers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceSpec {
    /// Slope of the transfer line. Below 1 compresses contrast toward `tint`.
    pub gain: f32,
    /// How far the backdrop's chroma is pushed from its own grey, before the
    /// gain drops the level. A gain alone moves level and colour together, so
    /// this is the only way a surface goes dark and keeps its colours. 1 is
    /// pass-through.
    pub saturation: f32,
    /// The look's own tone, **pre-multiplied by [`SurfaceSpec::coverage`]** —
    /// the opaque offset the shader adds. Divide by the coverage to read the
    /// tone back out ([`SurfaceSpec::tone`]).
    pub tint: makepad_widgets::Vec4f,
    /// How much of the backdrop this surface covers, 0..1. The one number a
    /// thickness moves.
    pub coverage: f32,
    /// Gaussian sigma under the lens, in logical pixels.
    pub blur: f32,
    /// How far in from the rim the lens bends the backdrop, in logical pixels.
    /// A length rather than a share of the box: the displacement curve is one
    /// curve, the same from a 96pt box to a 320pt one and from r24 to r84.
    pub rim: f32,
    /// How far the outermost pixel drags what it samples, in logical pixels.
    pub reach: f32,
    /// How much white the lit rim adds at the edge itself, 0..1.
    pub edge: f32,
    /// How far in that light falls off to nothing, in logical pixels.
    pub edge_width: f32,
    /// The coverage ramp at the shape's own boundary, in logical pixels.
    pub edge_aa: f32,
    /// Whether the surface casts a shadow.
    pub shadow: bool,
}

impl Default for SurfaceSpec {
    fn default() -> Self {
        Self {
            gain: 0.0,
            saturation: 1.0,
            tint: crate::color::grey(0x16),
            coverage: 1.0,
            blur: 0.0,
            rim: 0.0,
            reach: 0.0,
            edge: 0.0,
            edge_width: 1.0,
            edge_aa: 0.5,
            shadow: true,
        }
    }
}

impl SurfaceSpec {
    /// How opaque the surface ends up.
    pub fn coverage(&self) -> f32 {
        self.coverage
    }

    /// The effective tone at full coverage — `tint / coverage`.
    ///
    /// The readings that established the frost scale were taken this way: one
    /// tone falls out of every thickness, which is what makes thickness a knob
    /// rather than five looks.
    pub fn tone(&self) -> makepad_widgets::Vec4f {
        let divisor = self.coverage.max(1e-4);
        makepad_widgets::Vec4f {
            x: self.tint.x / divisor,
            y: self.tint.y / divisor,
            z: self.tint.z / divisor,
            w: 1.0,
        }
    }

    /// Whether this surface actually blurs anything.
    pub fn blurs(&self) -> bool {
        self.blur > 0.0 && self.gain > 0.0
    }

    /// How far a sampling footprint reaches outside the surface's rect. A
    /// consumer that snapshots a backdrop must expand by this much or content
    /// moving just past the rim changes what the glass shows without a new
    /// snapshot.
    pub fn support(&self) -> f32 {
        // Two smooth-upsample kernels plus the lens's own reach.
        2.0 * self.blur + self.reach.max(0.0)
    }
}

/// The frosted material for an appearance.
///
/// Measured 2026-08-31 on macOS 26. Three greys give the line at rms 0.9
/// levels, four saturated tones agree on the saturation to 0.1, and the sigma
/// is the Gaussian that best fits a 70pt bar edge at the centre of a 320pt
/// glass, rms 0.4 levels.
pub fn material(appearance: Appearance) -> MaterialSpec {
    match appearance {
        Appearance::Dark => MaterialSpec {
            tone: crate::color::grey(47),
            saturation: 2.1,
            blur: 21.0,
            edge: 0.10,
            edge_width: 1.0,
            edge_aa: 0.5,
        },
        Appearance::Light => MaterialSpec {
            tone: crate::color::grey(235),
            saturation: 2.1,
            // Light keeps its opacity (86%) and swaps a 19% grey base for a
            // 97% white one, which is why it reads as ordinary frost here.
            blur: 21.0,
            edge: 0.10,
            edge_width: 1.0,
            edge_aa: 0.5,
        },
    }
}

/// The numbers behind one glass variant.
///
/// Measured 2026-08-31 off a real `NSGlassEffectView`, one whole-canvas tone at
/// a time: a fill the size of the probe cannot be contaminated by a 10pt blur,
/// which is what every earlier reading of this look got wrong.
pub fn glass(appearance: Appearance, variant: Glass) -> SurfaceSpec {
    match (appearance, variant) {
        (Appearance::Dark, Glass::Regular) => SurfaceSpec {
            gain: 0.311,
            saturation: 2.55,
            tint: crate::color::ink(11.0 / 255.0, 1.0),
            coverage: 11.0 / 255.0,
            // An on-screen choice, 2026-09-01. The measurement reads 10.8, at
            // which the interior is a flat wash and every seam in the lens
            // shows as a step in it.
            blur: 4.0,
            rim: 18.75,
            reach: 47.0,
            // Read over a black canvas, where the coverage blend can only pull
            // down, so anything above the interior is rim light: +25 levels at
            // the boundary and gone by 1pt, the same in both appearances.
            edge: 0.10,
            edge_width: 1.0,
            edge_aa: 0.5,
            shadow: false,
        },
        (Appearance::Dark, Glass::Clear) => SurfaceSpec {
            // Refit 2026-08-30 over the gallery's own backdrops, rms 0.4 levels
            // on backdrop 0..212. The sigma is off a 2pt rule, which a 48pt
            // band is too wide to resolve. The rim is off the position-coded
            // backdrop, pooled over four shapes from 96pt to 320pt and r24 to
            // r84: one curve, rms 1.8pt.
            gain: 1.029,
            saturation: 1.0,
            tint: crate::color::ink(16.0 / 255.0, 1.0),
            coverage: 16.0 / 255.0,
            blur: 1.2,
            rim: 18.75,
            reach: 47.0,
            edge: 0.26,
            edge_width: 1.4,
            edge_aa: 0.5,
            shadow: false,
        },
        (Appearance::Light, Glass::Regular) => SurfaceSpec {
            // Very nearly a white sheet — 84% of the output is tint — so the
            // little backdrop that survives is pushed much harder to keep its
            // colour: saturation 4.27 against dark's 2.55, each within 0.1 over
            // four hues.
            gain: 0.139,
            saturation: 4.27,
            tint: crate::color::ink(214.0 / 255.0, 1.0),
            coverage: 214.0 / 255.0,
            blur: 8.9,
            rim: 18.75,
            reach: 47.0,
            edge: 0.10,
            edge_width: 1.0,
            edge_aa: 0.5,
            shadow: false,
        },
        (Appearance::Light, Glass::Clear) => SurfaceSpec {
            // Carries dark's rim, sigma and edge: light has not been
            // re-measured since the instrument learned to hold the window key.
            gain: 1.041,
            saturation: 1.0,
            tint: crate::color::ink(18.8 / 255.0, 1.0),
            coverage: 18.8 / 255.0,
            blur: 1.2,
            rim: 18.75,
            reach: 47.0,
            edge: 0.26,
            edge_width: 1.4,
            edge_aa: 0.5,
            shadow: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frost_thicknesses_are_monotonic() {
        let opacities: Vec<f32> = Material::ALL.iter().map(|m| m.opacity()).collect();
        for w in opacities.windows(2) {
            assert!(w[0] < w[1], "{opacities:?}");
        }
        for o in opacities {
            assert!((0.0..1.0).contains(&o), "{o}");
        }
    }

    #[test]
    fn test_material_keeps_one_tone_across_the_whole_scale() {
        // The reading that makes thickness a knob: `tint / (1 - gain)` is one
        // tone at every step, which is why there are not five materials.
        for appearance in [Appearance::Dark, Appearance::Light] {
            let m = material(appearance);
            let tones: Vec<f32> = Material::ALL
                .iter()
                .map(|t| m.at(*t).tone().x)
                .collect();
            let first = tones[0];
            for t in &tones {
                let drift = (t - first).abs() / first;
                assert!(drift < 0.10, "{appearance:?}: {tones:?}");
            }
        }
    }

    #[test]
    fn test_material_blur_does_not_move_with_thickness() {
        let m = material(Appearance::Dark);
        let blurs: Vec<f32> = Material::ALL.iter().map(|t| m.at(*t).blur).collect();
        assert!(blurs.windows(2).all(|w| w[0] == w[1]), "{blurs:?}");
    }

    #[test]
    fn test_thickness_only_moves_coverage() {
        let m = material(Appearance::Dark);
        let specs: Vec<SurfaceSpec> = Material::ALL.iter().map(|t| m.at(*t)).collect();
        let coverages: Vec<f32> = specs.iter().map(|s| s.coverage()).collect();
        for w in coverages.windows(2) {
            assert!(w[0] < w[1], "{coverages:?}");
        }
        // Everything but the coverage is shared.
        assert!(specs.windows(2).all(|w| w[0].blur == w[1].blur));
        assert!(specs.windows(2).all(|w| w[0].saturation == w[1].saturation));
    }

    #[test]
    fn test_glass_and_material_disagree_about_bendability() {
        // `Clear` bends its backdrop hard and blurs it barely; a frost blurs
        // hard and does not bend at all. Two different vocabularies.
        let clear = glass(Appearance::Dark, Glass::Clear);
        assert!(clear.rim > 0.0);
        assert!(clear.blurs());
        let frost = material(Appearance::Dark).at(Material::Regular);
        assert_eq!(frost.rim, 0.0);
        assert!(frost.blurs());
    }

    #[test]
    fn test_regular_glass_is_not_opaque_in_any_appearance() {
        for appearance in [Appearance::Dark, Appearance::Light] {
            let g = glass(appearance, Glass::Regular);
            assert!(g.coverage() < 0.9, "{appearance:?} {}", g.coverage());
        }
    }

    #[test]
    fn test_light_frost_is_the_brighter_sheet() {
        let d = material(Appearance::Dark).at(Material::Regular);
        let l = material(Appearance::Light).at(Material::Regular);
        assert!(l.tint.x > d.tint.x);
    }

    #[test]
    fn test_support_covers_the_blur_and_the_lens_reach() {
        let clear = glass(Appearance::Dark, Glass::Clear);
        // 47pt of reach must be inside the footprint, not outside it.
        assert!(clear.support() >= clear.reach);
        let frost = material(Appearance::Dark).at(Material::Thick);
        assert!(frost.support() >= 2.0 * frost.blur);
    }

    #[test]
    fn test_variant_names_are_distinct() {
        let mut names: Vec<&str> = Material::ALL.iter().map(|m| m.name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 5);
    }

    #[test]
    fn test_default_style_is_glass_regular() {
        assert_eq!(SurfaceStyle::default(), SurfaceStyle::Glass(Glass::Regular));
    }
}
