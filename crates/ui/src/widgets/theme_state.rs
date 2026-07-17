use makepad_widgets::*;

// Re-export the MpThemeState struct defined in theme/dark
pub use crate::theme::dark::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp_theme.*

    // Register MpThemeState as a widget accessible from mod.widgets
    mod.widgets.MpThemeProviderBase = #(MpThemeState::register_widget(vm))
    mod.widgets.MpThemeProvider = set_type_default() do mod.widgets.MpThemeProviderBase{
        width: Fit
        height: Fit
    }
}
