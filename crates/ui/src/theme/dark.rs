use makepad_widgets::*;

// ============================================================
// Dark Mode Theme Palette
// ============================================================

script_mod! {
    mod.mp_theme_dark = {
        PRIMARY: #x60A5FA
        PRIMARY_HOVER: #x3B82F6
        PRIMARY_ACTIVE: #x2563EB
        PRIMARY_FOREGROUND: #x0F172A

        SECONDARY: #x1E293B
        SECONDARY_HOVER: #x334155
        SECONDARY_ACTIVE: #x475569
        SECONDARY_FOREGROUND: #xF1F5F9

        DANGER: #xF87171
        DANGER_HOVER: #xEF4444
        DANGER_ACTIVE: #xDC2626
        DANGER_FOREGROUND: #x0F172A

        SUCCESS: #x4ADE80
        SUCCESS_HOVER: #x22C55E
        SUCCESS_ACTIVE: #x16A34A
        SUCCESS_FOREGROUND: #x0F172A

        WARNING: #xFBBF24
        WARNING_HOVER: #xF59E0B
        WARNING_ACTIVE: #xD97706
        WARNING_FOREGROUND: #x0F172A

        INFO: #x38BDF8
        INFO_HOVER: #x0EA5E9
        INFO_ACTIVE: #x0284C7
        INFO_FOREGROUND: #x0F172A

        BACKGROUND: #x0F172A
        FOREGROUND: #xF8FAFC
        BORDER: #x334155
        INPUT: #x1E293B
        RING: #x60A5FA
        MUTED: #x1E293B
        MUTED_FOREGROUND: #x94A3B8

        CARD: #x1E293B
        CARD_FOREGROUND: #xF8FAFC

        ACCENT: #xC084FC
        ACCENT_HOVER: #xA855F7
        ACCENT_FOREGROUND: #x0F172A

        TRANSPARENT: #x0000

        SWITCH_TRACK_OFF: #x334155
        SWITCH_TRACK_OFF_HOVER: #x475569
        SWITCH_THUMB: #xF8FAFC

        DIVIDER: #x334155
        SELECTION: #x3B82F6
        SELECTION_FOREGROUND: #xF8FAFC
    }
}

// ============================================================
// MpThemeState - Central theme state holder
// ============================================================

#[derive(Clone, Debug, Default)]
pub enum MpThemeAction {
    Toggle,
    SetDark,
    SetLight,
    #[default]
    None,
}

/// Theme state holder - tracks light/dark mode.
/// Uses `#[deref] view: View` so it can be placed in the widget tree.
#[derive(Script, ScriptHook, Widget)]
pub struct MpThemeState {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    /// Whether dark mode is active
    #[live(false)]
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
        self.dark_mode = !self.dark_mode;
        let action = if self.dark_mode {
            MpThemeAction::SetDark
        } else {
            MpThemeAction::SetLight
        };
        cx.widget_action(self.widget_uid(), MpThemeAction::Toggle);
        cx.widget_action(self.widget_uid(), action);
        self.redraw(cx);
    }

    /// Set dark mode directly
    pub fn set_dark_mode(&mut self, cx: &mut Cx, dark: bool) {
        if self.dark_mode != dark {
            self.dark_mode = dark;
            let action = if dark {
                MpThemeAction::SetDark
            } else {
                MpThemeAction::SetLight
            };
            cx.widget_action(self.widget_uid(), action);
            self.redraw(cx);
        }
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
