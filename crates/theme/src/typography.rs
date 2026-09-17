//! The type ladder: eleven roles, each carrying a size, a leading and a weight.
//!
//! Sizes measured on macOS 26, 2026-08-31, through
//! `NSFont.preferredFont(forTextStyle:)`; line heights 2026-09-01, through
//! `NSLayoutManager.defaultLineHeight(for:)` on the same fonts.
//!
//! The ladder is pure data. [`install`](crate::install) bakes each role into a
//! Makepad `TextStyle` on the script heap (`mod.mpc_type.*`), because a Makepad
//! `TextStyle` is a script object rather than a plain Rust value. A widget
//! writes `text_style: mod.mpc_type.body` and no number at all.

use std::sync::atomic::{AtomicU32, Ordering};

/// A role in the type ladder — SwiftUI's `Font.TextStyle`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextStyle {
    LargeTitle,
    Title,
    Title2,
    Title3,
    Headline,
    Subheadline,
    Body,
    Callout,
    Footnote,
    Caption,
    Caption2,
}

/// Which bundled face a role is set in.
///
/// Makepad's font policy installs two text faces and a mono face
/// (`font_regular`, `font_bold`, `font_code`), so the ladder's three weights
/// collapse onto two. [`Weight::Medium`] is recorded rather than dropped —
/// `Caption2` is a medium role — and resolves to the regular face because the
/// bundled family ships no medium instance. A vendored medium face would make
/// this a one-line change here and nowhere else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weight {
    Normal,
    Medium,
    Bold,
}

impl Weight {
    /// The Makepad theme font this weight paints in.
    pub const fn face(self) -> Face {
        match self {
            // No medium instance ships with the bundled family.
            Weight::Normal | Weight::Medium => Face::Regular,
            Weight::Bold => Face::Bold,
        }
    }
}

/// A face from Makepad's installed font policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    /// `mod.theme.font_regular`
    Regular,
    /// `mod.theme.font_bold`
    Bold,
    /// `mod.theme.font_code`
    Code,
}

impl Face {
    /// The script path this face is read from.
    pub const fn script_name(self) -> &'static str {
        match self {
            Face::Regular => "font_regular",
            Face::Bold => "font_bold",
            Face::Code => "font_code",
        }
    }
}

/// The body size every painted role is scaled against, as raw `f32` bits.
static BASE: AtomicU32 = AtomicU32::new(TextStyle::Body.size().to_bits());

/// Set the body size in points; every other role keeps its ratio to it, the way
/// every corner is a ratio of [`Theme::BASE_RADIUS`](crate::theme::Theme::BASE_RADIUS).
///
/// A probe for the chrome that does not grow with the text — header heights,
/// status strips and every fixed `pad_y`. The measured ramp is non-linear per
/// role, so one ratio finds that coupling without describing the ramp;
/// [`TextStyle::size`] stays the measured table at any setting.
pub fn set_base_text_size(points: f32) {
    BASE.store(points.to_bits(), Ordering::Relaxed);
}

/// The body size in points. [`TextStyle::Body`]'s own size paints the measured
/// ladder.
pub fn base_text_size() -> f32 {
    f32::from_bits(BASE.load(Ordering::Relaxed))
}

impl TextStyle {
    /// Every role, in ladder order. Carried so `install` can iterate the ladder
    /// instead of restating it.
    pub const ALL: [TextStyle; 11] = [
        TextStyle::LargeTitle,
        TextStyle::Title,
        TextStyle::Title2,
        TextStyle::Title3,
        TextStyle::Headline,
        TextStyle::Subheadline,
        TextStyle::Body,
        TextStyle::Callout,
        TextStyle::Footnote,
        TextStyle::Caption,
        TextStyle::Caption2,
    ];

    /// The role's measured size in points.
    pub const fn size(self) -> f32 {
        match self {
            Self::LargeTitle => 26.0,
            Self::Title => 22.0,
            Self::Title2 => 17.0,
            Self::Title3 => 15.0,
            Self::Headline | Self::Body => 13.0,
            Self::Callout => 12.0,
            Self::Subheadline => 11.0,
            Self::Footnote | Self::Caption | Self::Caption2 => 10.0,
        }
    }

    /// The size this role paints at, which [`set_base_text_size`] moves.
    pub fn painted(self) -> f32 {
        self.size() * base_text_size() / Self::Body.size()
    }

    /// The role's measured line height in points, at its measured
    /// [`Self::size`].
    ///
    /// A table beside `size`, because the ratio is not one number: it runs 1.18
    /// at `Title` up to 1.33 at `Title3`, and does not move monotonically with
    /// the size.
    pub const fn line_height(self) -> f32 {
        match self {
            Self::LargeTitle => 32.0,
            Self::Title => 26.0,
            Self::Title2 => 22.0,
            Self::Title3 => 20.0,
            Self::Headline | Self::Body => 16.0,
            Self::Callout => 15.0,
            Self::Subheadline => 14.0,
            Self::Footnote | Self::Caption | Self::Caption2 => 13.0,
        }
    }

    /// The line box this role paints in, which [`set_base_text_size`] moves.
    pub fn painted_line_height(self) -> f32 {
        self.line_height() * base_text_size() / Self::Body.size()
    }

    /// The role's weight. Three roles share 13pt and three share 10pt, so this
    /// is what separates them.
    pub const fn weight(self) -> Weight {
        match self {
            Self::Headline => Weight::Bold,
            Self::Caption2 => Weight::Medium,
            _ => Weight::Normal,
        }
    }

    /// The leading as a multiple of the painted size — Makepad's
    /// `line_spacing`, which is a multiplier rather than an absolute box.
    ///
    /// Coming off the measured table rather than a constant is what keeps the
    /// ratio honest: 1.23 on a 13pt body against a flat 1.2 everywhere, which
    /// is the kind of drift that leaves a 26pt title clipped.
    pub const fn leading(self) -> f32 {
        self.line_height() / self.size()
    }

    /// The face this role is set in.
    pub const fn face(self) -> Face {
        self.weight().face()
    }

    /// The role's stable name, used for the script heap export and for
    /// diagnostics.
    pub const fn name(self) -> &'static str {
        match self {
            Self::LargeTitle => "large_title",
            Self::Title => "title",
            Self::Title2 => "title2",
            Self::Title3 => "title3",
            Self::Headline => "headline",
            Self::Subheadline => "subheadline",
            Self::Body => "body",
            Self::Callout => "callout",
            Self::Footnote => "footnote",
            Self::Caption => "caption",
            Self::Caption2 => "caption2",
        }
    }
}

/// One role as it is actually set: a rung on the ladder, the leading it
/// carries, and the weight it is set in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    pub role: TextStyle,
    /// Line height as a multiple of the painted size, so leading follows the
    /// type wherever [`set_base_text_size`] puts it.
    pub leading: f32,
    /// The ladder carries one bold cell, so a set needing several heading
    /// weights names its own here rather than reading it off the role.
    pub weight: Weight,
    /// A factor over the painted ladder, for one surface sized apart from the
    /// rest — a document the reader has zoomed. 1.0 is the ladder itself.
    pub scale: f32,
}

impl Metrics {
    pub const fn new(role: TextStyle, leading: f32, weight: Weight) -> Self {
        Self {
            role,
            leading,
            weight,
            scale: 1.0,
        }
    }

    /// The same metrics at `scale` times the ladder. Replaces rather than
    /// compounds, so a slider handing over an absolute factor cannot drift.
    pub const fn scaled(self, scale: f32) -> Self {
        Self { scale, ..self }
    }

    pub fn size(self) -> f32 {
        self.role.painted() * self.scale
    }

    pub fn line_height(self) -> f32 {
        self.size() * self.leading
    }
}

impl Default for Metrics {
    fn default() -> Self {
        TextStyle::Body.into()
    }
}

impl From<TextStyle> for Metrics {
    /// The ladder's own setting for a role: its measured leading and weight.
    /// A set that wants prose leading names its own through [`Metrics::new`].
    fn from(role: TextStyle) -> Self {
        Self::new(role, role.leading(), role.weight())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ladder_covers_the_measured_sizes_exactly() {
        // Enum order is SwiftUI's declaration order, not size order: the
        // ladder runs `Headline, Subheadline, Body` and Subheadline (11) is
        // genuinely smaller than Body (13). So the invariant is the *set* of
        // sizes, sorted — the ramp the type is chosen from.
        let mut sizes: Vec<f32> = TextStyle::ALL.iter().map(|r| r.size()).collect();
        sizes.sort_by(|a, b| b.total_cmp(a));
        assert_eq!(
            sizes,
            vec![26.0, 22.0, 17.0, 15.0, 13.0, 13.0, 12.0, 11.0, 10.0, 10.0, 10.0]
        );
    }

    #[test]
    fn test_every_role_sits_within_the_largest_and_smallest_rung() {
        let largest = TextStyle::LargeTitle.size();
        let smallest = TextStyle::Caption2.size();
        for role in TextStyle::ALL {
            assert!((smallest..=largest).contains(&role.size()), "{role:?}");
        }
    }

    #[test]
    fn test_all_names_are_distinct_and_cover_every_role() {
        let mut names: Vec<&str> = TextStyle::ALL.iter().map(|r| r.name()).collect();
        assert_eq!(names.len(), 11);
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 11, "duplicate role name");
    }

    #[test]
    fn test_leading_reproduces_the_measured_line_box() {
        // The whole point of storing leading as a computed ratio: `Metrics`
        // multiplied back out must land on the measured number.
        for role in TextStyle::ALL {
            let m = Metrics::from(role);
            assert!(
                (m.line_height() - role.line_height()).abs() < 1e-4,
                "{role:?}: {} vs {}",
                m.line_height(),
                role.line_height()
            );
        }
    }

    #[test]
    fn test_leading_is_never_a_flat_constant() {
        // A flat 1.2 everywhere is the bug this table exists to prevent.
        let leadings: Vec<f32> = TextStyle::ALL.iter().map(|r| r.leading()).collect();
        let first = leadings[0];
        assert!(
            leadings.iter().any(|l| (l - first).abs() > 0.01),
            "{leadings:?}"
        );
    }

    #[test]
    fn test_leading_is_plausible_for_text() {
        for role in TextStyle::ALL {
            let l = role.leading();
            assert!((1.0..1.6).contains(&l), "{role:?} leading {l}");
        }
    }

    #[test]
    fn test_roles_sharing_a_size_are_separated_by_weight() {
        // Headline and Body are both 13pt; Headline must be the bold one.
        assert_eq!(TextStyle::Headline.size(), TextStyle::Body.size());
        assert_ne!(TextStyle::Headline.weight(), TextStyle::Body.weight());
        assert_eq!(TextStyle::Headline.face(), Face::Bold);
        assert_eq!(TextStyle::Body.face(), Face::Regular);
    }

    #[test]
    fn test_medium_roles_resolve_to_the_only_face_that_ships() {
        // Recorded approximation, not an oversight: no medium instance is
        // bundled. If one lands, only `Weight::face` changes.
        assert_eq!(Weight::Medium.face(), Face::Regular);
        assert_eq!(TextStyle::Caption2.weight(), Weight::Medium);
    }

    #[test]
    fn test_painted_size_follows_the_base() {
        set_base_text_size(26.0);
        assert!((TextStyle::Body.painted() - 26.0).abs() < 1e-4);
        assert!((TextStyle::Title.painted() - 44.0).abs() < 1e-4);
        // Ratios are preserved, so leading stays a ratio rather than a box.
        assert!((TextStyle::Body.leading() - 16.0 / 13.0).abs() < 1e-4);
        set_base_text_size(TextStyle::Body.size());
        assert!((TextStyle::Body.painted() - 13.0).abs() < 1e-4);
    }

    #[test]
    fn test_scaled_metrics_replace_rather_than_compound() {
        let m = Metrics::from(TextStyle::Body).scaled(2.0).scaled(3.0);
        assert_eq!(m.scale, 3.0);
        assert!((m.size() - 13.0 * 3.0).abs() < 1e-4);
    }

    #[test]
    fn test_faces_name_real_script_bindings() {
        for face in [Face::Regular, Face::Bold, Face::Code] {
            assert!(!face.script_name().is_empty());
        }
        assert_eq!(Face::Code.script_name(), "font_code");
    }
}
