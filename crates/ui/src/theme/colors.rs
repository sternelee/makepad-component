use makepad_widgets::*;

script_mod! {
    mod.mp_theme = {
        // macOS System Colors (Apple HIG compliant)
        // Primary accent color - System Blue
        PRIMARY: #x007AFF
        PRIMARY_HOVER: #x0066CC
        PRIMARY_ACTIVE: #x0055AA
        PRIMARY_FOREGROUND: #xffffff

        // Secondary - System Gray
        SECONDARY: #xF5F5F7
        SECONDARY_HOVER: #xE8E8ED
        SECONDARY_ACTIVE: #xD1D1D6
        SECONDARY_FOREGROUND: #x1D1D1F

        // Danger - System Red
        DANGER: #xFF3B30
        DANGER_HOVER: #xD70015
        DANGER_ACTIVE: #xB9000D
        DANGER_FOREGROUND: #xffffff

        // Success - System Green (for switches, etc.)
        SUCCESS: #x34C759
        SUCCESS_HOVER: #x2DB840
        SUCCESS_ACTIVE: #x249A33
        SUCCESS_FOREGROUND: #xffffff

        // Warning - System Orange
        WARNING: #xFF9500
        WARNING_HOVER: #xE08600
        WARNING_ACTIVE: #xCC7700
        WARNING_FOREGROUND: #xffffff

        // Info - System Teal
        INFO: #x5AC8FA
        INFO_HOVER: #x4AB8EB
        INFO_ACTIVE: #x39A8DC
        INFO_FOREGROUND: #xffffff

        // UI colors - macOS style (subtle grays)
        BACKGROUND: #xFFFFFF
        FOREGROUND: #x1D1D1F
        BORDER: #xD1D1D6
        INPUT: #xF5F5F7
        RING: #x007AFF
        MUTED: #xF5F5F7
        MUTED_FOREGROUND: #x86868B

        // Card colors
        CARD: #xFFFFFF
        CARD_FOREGROUND: #x1D1D1F

        // Accent - System Purple (for variety)
        ACCENT: #xAF52DE
        ACCENT_HOVER: #x9B41C9
        ACCENT_FOREGROUND: #xffffff

        // Transparent
        TRANSPARENT: #x0000

        // Additional macOS system colors for specific components
        // Switch track colors (OFF state uses subtle gray)
        SWITCH_TRACK_OFF: #xE9E9EB
        SWITCH_TRACK_OFF_HOVER: #xD1D1D6
        SWITCH_THUMB: #xFFFFFF

        // Separator / Divider
        DIVIDER: #xC6C6C8

        // Selection / Highlight
        SELECTION: #x007AFF
        SELECTION_FOREGROUND: #xFFFFFF
    }
}
