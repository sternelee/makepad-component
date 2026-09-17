//! Layout metrics. Numbers drive layout, colours are paint: nothing here
//! depends on which colour is painted.
//!
//! Every number records where it was read and when (law 5). Where the platform
//! names one value we ship one and no more.

use std::sync::atomic::{AtomicU32, Ordering};

use makepad_widgets::*;

use crate::{theme::Theme, typography::TextStyle};

/// A control's size — SwiftUI's `ControlSize`.
///
/// Three rungs, each carrying a role from the type ladder. The two that were
/// measured are [`ControlSize::Small`] and [`ControlSize::Regular`]; `Large` is
/// the same rule carried one step further, and is marked as such rather than
/// passed off as a reading.
///
/// Registered on the script heap as `mod.mpc.ControlSize`, so a DSL block writes
/// `control: mod.mpc.ControlSize.Small` and the widget reads back the same rung
/// the caller named rather than a private copy of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Script, ScriptHook)]
pub enum ControlSize {
    /// The chip: `Callout` on a `control_radius` corner. 20pt — measured.
    #[live]
    Small,
    /// The form row: `Body` on a `button_radius` corner. 24pt — measured.
    #[pick]
    #[default]
    #[live]
    Regular,
    /// One step up from the form row, for a primary action standing alone.
    /// 28pt — derived by the same +4pt step the platform uses from `small` to
    /// `regular`, not separately measured.
    #[live]
    Large,
}

impl ControlSize {
    /// The type role the size paints at, which decides the rest.
    pub const fn text(self) -> TextStyle {
        match self {
            ControlSize::Small => TextStyle::Callout,
            ControlSize::Regular => TextStyle::Body,
            ControlSize::Large => TextStyle::Title3,
        }
    }

    /// The control's height in points.
    ///
    /// Measured 2026-09-01: `NSButton`, `NSTextField` and `NSPopUpButton` all
    /// report 20 at `.small` and 24 at `.regular` — `Body`'s 16pt line box with
    /// 4 above and below.
    pub const fn height(self) -> f32 {
        match self {
            ControlSize::Small => 20.0,
            ControlSize::Regular => 24.0,
            ControlSize::Large => 28.0,
        }
    }

    /// Horizontal padding: the visual format's `-`, one step tighter than the
    /// 8pt sibling gap because a control's edge is not a gap between peers.
    pub const fn pad_x(self) -> f32 {
        match self {
            ControlSize::Small => 8.0,
            ControlSize::Regular => 12.0,
            ControlSize::Large => 14.0,
        }
    }

    /// What the height has left over its role's line box, halved.
    ///
    /// Derived rather than stored: the pair drifted once already, when the line
    /// box moved and the two heights stayed where the old leading had put them.
    pub const fn pad_y(self) -> f32 {
        (self.height() - self.text().line_height()) / 2.0
    }

    /// The corner this size's control is cut with.
    pub fn radius(self) -> f32 {
        match self {
            ControlSize::Small => Theme::control_radius(),
            ControlSize::Regular | ControlSize::Large => Theme::button_radius(),
        }
    }

    /// The role's metrics — where the size's line box comes from.
    pub fn metrics(self) -> crate::typography::Metrics {
        self.text().into()
    }

    /// The size's stable name, used for the `mod.mpc.layout.control.<name>`
    /// entry and for diagnostics.
    pub const fn name(self) -> &'static str {
        match self {
            ControlSize::Small => "small",
            ControlSize::Regular => "regular",
            ControlSize::Large => "large",
        }
    }
}

/// The branded base radius, as raw `f32` bits.
///
/// Read through a process-wide mirror rather than the `Cx` global so the
/// accessors are callable from `const`-ish and `&self` paint contexts alike:
/// a radius is one number for the whole app, and every corner is a ratio of it.
static BASE_RADIUS: AtomicU32 = AtomicU32::new(Theme::BASE_RADIUS.to_bits());

/// Point every radius accessor at a new base. Called by
/// [`Theme::install`](crate::theme::Theme::install).
pub(crate) fn set_base_radius(radius: f32) {
    BASE_RADIUS.store(radius.to_bits(), Ordering::Relaxed);
}

/// The base radius the accessors are currently reading.
pub fn base_radius() -> f32 {
    f32::from_bits(BASE_RADIUS.load(Ordering::Relaxed))
}

/// A corner as a multiple of the branded base radius.
fn radius(ratio: f32) -> f32 {
    base_radius() * ratio
}

/// The layout metrics: measurements the platform named, plus the ratios derived
/// from them. Carried on [`Theme`](crate::theme::Theme) so a widget reads one
/// value instead of writing a literal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    /// The gap between siblings. Measured on macOS 26, 2026-08-31:
    /// `NSStackView().spacing`, visual format's `-`, and
    /// `constraint(equalToSystemSpacingAfter:multiplier: 1)` all report 8.
    pub space: f32,
    /// The margin from content to its container's edge. Same measurement,
    /// visual format's `|-`.
    pub content_margin: f32,
    /// Main-panel header height (the reference `h-11`).
    pub header_height: f32,
    /// The unified window titlebar: traffic lights + cluster + tabs. Content
    /// rides [`Layout::titlebar_top_pad`] lower than centre so the air above
    /// matches the perceived gap to the inset card below.
    pub titlebar_height: f32,
    /// Downward shift of titlebar content within the bar.
    pub titlebar_top_pad: f32,
    /// Leading room the macOS traffic lights need where AppKit puts them —
    /// the same 78pt a Tauri window measured. An app that *moves* the lights
    /// owns this number too.
    pub traffic_light_inset: f32,
    /// Reserved status strip under the content outlet (the reference `h-6`);
    /// reserving it keeps the composer from shifting.
    pub status_strip_height: f32,
    /// Height of the gradient that fades a transcript into the panel
    /// background at its bottom edge. The transcript's last row must pad itself
    /// past this band so settled content never sits inside the fade when
    /// scrolled to the bottom.
    pub transcript_fade_band: f32,
    /// A list row's height: the form row plus the gap that follows it, because
    /// a row owns its own separator spacing.
    pub row_height: f32,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            space: Theme::SPACE,
            content_margin: Theme::CONTENT_MARGIN,
            header_height: Theme::HEADER_HEIGHT,
            titlebar_height: Theme::TITLEBAR_HEIGHT,
            titlebar_top_pad: Theme::TITLEBAR_TOP_PAD,
            traffic_light_inset: Theme::TRAFFIC_LIGHT_INSET,
            status_strip_height: Theme::STATUS_STRIP_HEIGHT,
            transcript_fade_band: Theme::TRANSCRIPT_FADE_BAND,
            row_height: Theme::ROW_HEIGHT,
        }
    }
}

/// The shipped layout. One value, because every number in it is a platform
/// reading rather than a choice.
impl Theme {
    /// The gap between siblings. Measured on macOS 26, 2026-08-31:
    /// `NSStackView().spacing`, visual format's `-`, and
    /// `constraint(equalToSystemSpacingAfter:multiplier: 1)` all report 8.
    ///
    /// Carried by [`stack::row`](crate::stack::row) and
    /// [`stack::column`](crate::stack::column), so a call site that wants the
    /// standard gap writes no number at all — SwiftUI's shape, where
    /// `VStack(spacing:)` takes the system's when given nothing.
    pub const SPACE: f32 = 8.0;
    /// The margin from content to its container's edge. Visual format's `|-`.
    pub const CONTENT_MARGIN: f32 = 20.0;
    /// Main-panel header height (the reference `h-11`).
    pub const HEADER_HEIGHT: f32 = 44.0;
    /// The unified window titlebar.
    pub const TITLEBAR_HEIGHT: f32 = 38.0;
    /// Downward shift of titlebar content within the bar.
    pub const TITLEBAR_TOP_PAD: f32 = 2.0;
    /// Leading room the macOS traffic lights need where AppKit puts them.
    pub const TRAFFIC_LIGHT_INSET: f32 = if cfg!(target_os = "macos") { 78.0 } else { 0.0 };
    /// Reserved status strip under the content outlet.
    pub const STATUS_STRIP_HEIGHT: f32 = 24.0;
    /// The fade band at a transcript's bottom edge.
    pub const TRANSCRIPT_FADE_BAND: f32 = 24.0;
    /// A list row: the form row plus the sibling gap that follows it.
    pub const ROW_HEIGHT: f32 = 32.0;
    /// The base corner. Every other corner is a ratio of this one, so
    /// [`Brand::radius`](crate::brand::Brand::radius) moves the whole set
    /// together.
    pub const BASE_RADIUS: f32 = 8.0;

    /// Message bubble corner.
    pub fn bubble_radius() -> f32 {
        radius(2.0)
    }
    /// Floating-surface corner — popovers, menus, the command palette, group
    /// boxes.
    ///
    /// A glass surface paints this on its border **and** hands the same number
    /// to its backdrop blur. The two must agree: a blur cut to a different
    /// radius frosts square corners outside a round border, and it shows only
    /// on glass and only at the corners. So the radius is named once and read
    /// at both ends.
    pub fn surface_radius() -> f32 {
        radius(1.5)
    }
    /// Panel / card corner.
    pub fn panel_radius() -> f32 {
        radius(1.25)
    }
    /// Button, text field and select-trigger corner.
    pub fn button_radius() -> f32 {
        radius(1.0)
    }
    /// Small control corner (chips, tags, steppers) — a size down from
    /// [`Theme::button_radius`], for things that sit inside a control rather
    /// than being one.
    pub fn control_radius() -> f32 {
        radius(0.75)
    }

    /// The concentric child of a surface: a row inset by `inset` inside a
    /// container of radius `outer` keeps its corners parallel to the
    /// container's, rather than looking pasted onto it.
    ///
    /// SwiftUI's `ContainerRelativeShape` rule done as arithmetic. The
    /// relationship is stated where the child is *defined*, so a container that
    /// changes its padding carries its rows with it and the derived value never
    /// becomes a constant of its own.
    pub const fn inset_radius(outer: f32, inset: f32) -> f32 {
        if outer > inset { outer - inset } else { 0.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measured_control_heights_hold() {
        assert_eq!(ControlSize::Small.height(), 20.0);
        assert_eq!(ControlSize::Regular.height(), 24.0);
    }

    #[test]
    fn test_regular_height_is_the_body_line_box_plus_its_padding() {
        // The rule behind 24: Body's measured 16pt line box with 4 above and
        // below. If the ladder moves, this fails rather than the widget
        // silently clipping its own label.
        let size = ControlSize::Regular;
        let derived = size.text().line_height() + 2.0 * size.pad_y();
        assert!((derived - size.height()).abs() < 1e-6);
        assert!((size.pad_y() - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_pad_y_is_never_negative() {
        for size in [
            ControlSize::Small,
            ControlSize::Regular,
            ControlSize::Large,
        ] {
            assert!(size.pad_y() >= 0.0, "{size:?}");
        }
    }

    #[test]
    fn test_sizes_are_strictly_ordered() {
        assert!(ControlSize::Small.height() < ControlSize::Regular.height());
        assert!(ControlSize::Regular.height() < ControlSize::Large.height());
    }

    #[test]
    fn test_radii_descend_from_bubble_to_control() {
        let r = [
            Theme::bubble_radius(),
            Theme::surface_radius(),
            Theme::panel_radius(),
            Theme::button_radius(),
            Theme::control_radius(),
        ];
        for w in r.windows(2) {
            assert!(w[0] > w[1], "{r:?}");
        }
        assert_eq!(Theme::button_radius(), Theme::BASE_RADIUS);
    }

    #[test]
    fn test_branded_base_scales_every_radius_and_restores() {
        let before = Theme::surface_radius();
        set_base_radius(12.0);
        assert_eq!(Theme::button_radius(), 12.0);
        assert_eq!(Theme::surface_radius(), 18.0);
        set_base_radius(Theme::BASE_RADIUS);
        assert_eq!(Theme::surface_radius(), before);
    }

    #[test]
    fn test_inset_radius_floors_at_zero() {
        assert_eq!(Theme::inset_radius(12.0, 4.0), 8.0);
        assert_eq!(Theme::inset_radius(12.0, 12.0), 0.0);
        assert_eq!(Theme::inset_radius(4.0, 12.0), 0.0);
    }
}
