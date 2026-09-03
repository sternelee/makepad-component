//! Unified size system (gpui-component `Size`/`Sizable` port).
//!
//! Every control takes a single `size: MpSize.XSmall..XLarge` value and
//! derives its metrics (font size, padding, radius, min height) from it,
//! replacing the old one-widget-per-size DSL types.

use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*

    // Expose the MpSize enum proto so DSL can write `size: MpSize.Small`
    let MpSize = set_type_default() do #(MpSize::script_api(vm))
    mod.widgets.MpSize = MpSize
}

/// The five standard control sizes.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Script, ScriptHook)]
pub enum MpSize {
    XSmall,
    Small,
    #[pick]
    #[default]
    Medium,
    Large,
    XLarge,
}

impl MpSize {
    /// Body font size for text inside a control of this size.
    pub fn font_size(self) -> f32 {
        match self {
            Self::XSmall => 10.0,
            Self::Small => 12.0,
            Self::Medium => 13.0,
            Self::Large => 15.0,
            Self::XLarge => 17.0,
        }
    }

    /// Default horizontal padding.
    pub fn padding_h(self) -> f64 {
        match self {
            Self::XSmall => 8.0,
            Self::Small => 10.0,
            Self::Medium => 12.0,
            Self::Large => 16.0,
            Self::XLarge => 20.0,
        }
    }

    /// Default vertical padding.
    pub fn padding_v(self) -> f64 {
        match self {
            Self::XSmall => 3.0,
            Self::Small => 4.0,
            Self::Medium => 6.0,
            Self::Large => 8.0,
            Self::XLarge => 10.0,
        }
    }

    /// Corner radius for a control of this size.
    pub fn radius(self) -> f32 {
        match self {
            Self::XSmall => 5.0,
            Self::Small => 6.0,
            Self::Medium => 8.0,
            Self::Large => 10.0,
            Self::XLarge => 12.0,
        }
    }

    /// Minimum control height (used by fixed-height controls like inputs).
    pub fn min_height(self) -> f64 {
        match self {
            Self::XSmall => 22.0,
            Self::Small => 26.0,
            Self::Medium => 32.0,
            Self::Large => 38.0,
            Self::XLarge => 44.0,
        }
    }

    /// Icon size inside a control of this size.
    pub fn icon_size(self) -> f64 {
        match self {
            Self::XSmall => 12.0,
            Self::Small => 14.0,
            Self::Medium => 16.0,
            Self::Large => 18.0,
            Self::XLarge => 20.0,
        }
    }

    /// Spacing between a control's children (icon + label).
    pub fn spacing(self) -> f64 {
        match self {
            Self::XSmall => 3.0,
            Self::Small => 4.0,
            Self::Medium => 6.0,
            Self::Large => 8.0,
            Self::XLarge => 10.0,
        }
    }

    /// Thickness of a linear progress/bar element (display components).
    pub fn bar_thickness(self) -> f64 {
        match self {
            Self::XSmall => 3.0,
            Self::Small => 4.0,
            Self::Medium => 4.0,
            Self::Large => 6.0,
            Self::XLarge => 8.0,
        }
    }
}
