//! makepad-theme: design tokens, the type ladder, layout metrics and the
//! material layer for the component library.
//!
//! The single source of truth is Rust. The script heap copy a widget's DSL
//! block names (`mod.mpc_theme.*`, `mod.mpc_type.*`, `mod.mpc_layout.*`) is
//! *generated* from these structures by [`install`], so there is exactly one
//! place a token is written down.
//!
//! ```no_run
//! use makepad_theme::{appearance::Appearance, theme::Theme};
//!
//! // At startup, once:
//! # fn install(cx: &mut makepad_widgets::Cx) {
//! Theme::install(Theme::dark(), cx);
//! # }
//!
//! // Anywhere that paints:
//! # fn paint(cx: &mut makepad_widgets::Cx) {
//! let theme = Theme::of(cx);
//! let ink = theme.paint.text;
//! let body = theme.metrics(makepad_theme::typography::TextStyle::Body);
//! # let _ = (ink, body);
//! # }
//! ```

pub mod appearance;
pub mod brand;
pub mod color;
pub mod install;
pub mod layout;
pub mod legacy;
pub mod material;
pub mod paint;
pub mod palette;
pub mod syntax;
pub mod theme;
pub mod typography;

/// The v2 token names, re-exported so `MpThemeState` keeps compiling while the
/// v2 widget set is replaced. **Temporary** — see [`legacy`].
pub use legacy::TOKEN_NAMES;

/// Everything a component needs, in one import.
pub mod prelude {
    pub use crate::{
        appearance::Appearance,
        brand::{Brand, Tint},
        layout::ControlSize,
        material::{Glass, Material, SurfaceStyle},
        paint::{Paint, Shadow},
        theme::Theme,
        typography::{Face, Metrics, TextStyle, Weight},
    };
}

pub use appearance::Appearance;
pub use brand::Brand;
pub use layout::ControlSize;
pub use material::{Glass, Material, SurfaceStyle};
pub use paint::{Paint, Shadow};
pub use theme::Theme;
pub use typography::{Face, Metrics, TextStyle, Weight};

use makepad_widgets::*;

// Register the theme namespaces on the script heap.
//
// The modules are created here — before any widget registers — so a widget's
// `script_mod!` block can name them while the crate is loading, and every one
// starts at the shipped dark palette. `Theme::install` rewrites them when the
// app says which theme it wants.
//
// `mod.mpc.*` is the v3 vocabulary. `mod.mpc_theme` is the v2 one, kept alive
// for the widgets that have not been replaced yet; see [`legacy`].
script_mod! {
    use mod.prelude.widgets_internal.*

    mod.mpc = {}
    mod.mpc.tokens = #(crate::install::paint_namespace(vm, &crate::palette::dark()))
    mod.mpc.type = #(crate::install::type_namespace(vm))
    mod.mpc.ControlSize = set_type_default() do #(crate::layout::ControlSize::script_api(vm))
    // The syntax kind vocabulary, so a highlighter built on this theme names its kinds the same way
    // every other vocabulary here is named. Registered even though no widget reads it yet: the
    // crate's convention is that a public vocabulary reaches the script heap, and a vocabulary that
    // exists in Rust only is one the next layer would re-declare.
    mod.mpc.HighlightKind = set_type_default() do #(crate::syntax::HighlightKind::script_api(vm))
    mod.mpc.text = mod.mpc.type
    mod.mpc.layout = #(crate::install::layout_namespace(vm, &crate::layout::Layout::default()))
    mod.mpc.material = #(crate::install::material_namespace(vm, &crate::theme::Theme::dark()))
    mod.mpc_theme = #(crate::legacy::namespace(vm, &crate::palette::dark(), crate::Appearance::Dark))
}

/// Install the shipped dark theme, branded neutral.
///
/// A convenience for an app that wants the default look without naming it. The
/// presentation layer calls this from the widget that owns the switch, because
/// install needs a `&mut Cx` and the widget that draws first has one.
pub fn install_default(cx: &mut Cx) {
    Theme::install(Theme::dark(), cx);
}

#[cfg(test)]
mod tests {
    use crate::{color, Appearance, Theme};

    #[test]
    fn test_the_crate_root_re_exports_the_prelude_surface() {
        // Guards against a re-export being dropped from `lib.rs` while the
        // module still compiles — the prelude is the public contract.
        let theme = Theme::for_appearance(Appearance::Light);
        assert_eq!(theme.appearance, Appearance::Light);
        let _: crate::Paint = theme.paint;
        let _: crate::ControlSize = crate::ControlSize::Regular;
        let _: crate::Material = crate::Material::Regular;
    }

    #[test]
    fn test_default_theme_is_legible_end_to_end() {
        // The one assertion worth making at the top level: whatever a caller
        // gets without asking, body text on the page is readable.
        let theme = Theme::dark();
        let ratio = color::contrast_ratio(theme.paint.text, theme.paint.bg);
        assert!(ratio >= 4.5, "{ratio}");
    }
}
