use makepad_widgets::*;

live_design! {
    // macOS System Colors (Apple HIG compliant)
    // Primary accent color - System Blue
    pub PRIMARY = #007AFF
    pub PRIMARY_HOVER = #0066CC
    pub PRIMARY_ACTIVE = #0055AA
    pub PRIMARY_FOREGROUND = #ffffff

    // Secondary - System Gray
    pub SECONDARY = #F5F5F7
    pub SECONDARY_HOVER = #E8E8ED
    pub SECONDARY_ACTIVE = #D1D1D6
    pub SECONDARY_FOREGROUND = #1D1D1F

    // Danger - System Red
    pub DANGER = #FF3B30
    pub DANGER_HOVER = #D70015
    pub DANGER_ACTIVE = #B9000D
    pub DANGER_FOREGROUND = #ffffff

    // Success - System Green (for switches, etc.)
    pub SUCCESS = #34C759
    pub SUCCESS_HOVER = #2DB840
    pub SUCCESS_ACTIVE = #249A33
    pub SUCCESS_FOREGROUND = #ffffff

    // Warning - System Orange
    pub WARNING = #FF9500
    pub WARNING_HOVER = #E08600
    pub WARNING_ACTIVE = #CC7700
    pub WARNING_FOREGROUND = #ffffff

    // Info - System Teal
    pub INFO = #5AC8FA
    pub INFO_HOVER = #4AB8EB
    pub INFO_ACTIVE = #39A8DC
    pub INFO_FOREGROUND = #ffffff

    // UI colors - macOS style (subtle grays)
    pub BACKGROUND = #FFFFFF
    pub FOREGROUND = #1D1D1F
    pub BORDER = #D1D1D6
    pub INPUT = #F5F5F7
    pub RING = #007AFF
    pub MUTED = #F5F5F7
    pub MUTED_FOREGROUND = #86868B

    // Card colors
    pub CARD = #FFFFFF
    pub CARD_FOREGROUND = #1D1D1F

    // Accent - System Purple (for variety)
    pub ACCENT = #AF52DE
    pub ACCENT_HOVER = #9B41C9
    pub ACCENT_FOREGROUND = #ffffff

    // Transparent
    pub TRANSPARENT = #0000

    // Additional macOS system colors for specific components
    // Switch track colors (OFF state uses subtle gray)
    pub SWITCH_TRACK_OFF = #E9E9EB
    pub SWITCH_TRACK_OFF_HOVER = #D1D1D6
    pub SWITCH_THUMB = #FFFFFF

    // Separator / Divider
    pub DIVIDER = #C6C6C8

    // Selection / Highlight
    pub SELECTION = #007AFF
    pub SELECTION_FOREGROUND = #FFFFFF
}
