use makepad_theme::TOKEN_NAMES;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    // Register MpThemeState as a widget accessible from mod.widgets
    mod.widgets.MpThemeProviderBase = #(MpThemeState::register_widget(vm))
    mod.widgets.MpThemeProvider = set_type_default() do mod.widgets.MpThemeProviderBase{
        width: Fit
        height: Fit
    }
}

#[derive(Clone, Debug, Default)]
pub enum MpThemeAction {
    Toggle,
    SetDark,
    SetLight,
    #[default]
    None,
}

/// Theme state holder - tracks light/dark mode.
/// Switching copies the full palette over the `mod.mpc_theme` active
/// namespace on the script heap (dark/light source namespaces stay
/// pristine), then requests a global script re-apply so every widget
/// re-evaluates its token references.
#[derive(Script, ScriptHook, Widget)]
pub struct MpThemeState {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    /// Whether dark mode is active (default: dark)
    #[live(true)]
    dark_mode: bool,
}

impl Widget for MpThemeState {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpThemeState {
    /// Toggle between light and dark mode
    pub fn toggle(&mut self, cx: &mut Cx) {
        self.set_dark_mode(cx, !self.dark_mode);
        cx.widget_action(self.widget_uid(), MpThemeAction::Toggle);
    }

    /// Set dark mode directly
    pub fn set_dark_mode(&mut self, cx: &mut Cx, dark: bool) {
        if self.dark_mode == dark {
            return;
        }
        self.dark_mode = dark;
        let action = if dark {
            MpThemeAction::SetDark
        } else {
            MpThemeAction::SetLight
        };
        cx.widget_action(self.widget_uid(), action);
        Self::copy_palette(cx, dark);
        self.redraw(cx);
    }

    /// Copy every token from the `mpc_theme.dark`/`mpc_theme.light`
    /// source namespace over the active `mod.mpc_theme` namespace.
    fn copy_palette(cx: &mut Cx, dark: bool) {
        cx.with_vm(|vm| {
            let active = vm.bx.heap.module(id!(mpc_theme));
            let dark_obj = vm.bx.heap.value(active, id!(dark).into(), NoTrap);
            let light_obj = vm.bx.heap.value(active, id!(light).into(), NoTrap);
            let src: ScriptObject = if dark { dark_obj } else { light_obj }.into();
            for name in TOKEN_NAMES {
                let key: ScriptValue = LiveId::from_str(name).into();
                let v = vm.bx.heap.value(src, key, NoTrap);
                vm.bx.heap.set_value(active, key, v, NoTrap);
            }
        });
        // Re-apply all scripts so widgets re-evaluate their token references
        cx.request_script_reapply();
    }

    pub fn is_dark(&self) -> bool {
        self.dark_mode
    }
}

impl MpThemeStateRef {
    pub fn toggle(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.toggle(cx);
        }
    }

    pub fn set_dark_mode(&self, cx: &mut Cx, dark: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_dark_mode(cx, dark);
        }
    }

    pub fn is_dark(&self) -> bool {
        if let Some(inner) = self.borrow() {
            inner.is_dark()
        } else {
            false
        }
    }
}
