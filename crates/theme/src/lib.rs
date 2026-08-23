//! makepad-theme: design tokens + color math for the component library.
//!
//! Single source of truth lives in Rust (`color` / `palette`); the
//! `script_mod` below bakes both palettes into the script heap:
//!
//! - `mod.mpc_theme.dark`  — dark appearance
//! - `mod.mpc_theme.light` — light appearance
//! - `mod.mpc_theme`       — ACTIVE copy; widgets import this one.
//!   Appearance switching rewrites these values on the heap
//!   (see MpThemeState) then requests a script re-apply.

pub mod color;
pub mod palette;

use makepad_widgets::*;

// Named layout constants. Numbers drive layout; colors are paint.
pub const RADIUS_SURFACE: f64 = 12.0;
pub const RADIUS_PANEL: f64 = 10.0;
pub const RADIUS_CONTROL: f64 = 8.0;
pub const RADIUS_SMALL: f64 = 6.0;

/// Concentric-radius rule (SwiftUI ContainerRelativeShape arithmetic):
/// a child inset by `inset` px inside a parent with `outer` radius.
pub fn inset_radius(outer: f64, inset: f64) -> f64 {
    (outer - inset).max(0.0)
}

/// All token names in emission order. MpThemeState iterates this list
/// when copying a full palette over the active namespace on the heap.
pub const TOKEN_NAMES: &[&str] = &[
    "BG",
    "SURFACE",
    "SURFACE_RAISED",
    "SURFACE_CARD",
    "SURFACE_DIALOG",
    "SURFACE_OVERLAY",
    "ELEMENT_HOVER",
    "ELEMENT_ACTIVE",
    "BORDER",
    "BORDER_STRONG",
    "DIVIDER",
    "TEXT",
    "TEXT_MUTED",
    "TEXT_FAINT",
    "SOLID",
    "SOLID_HOVER",
    "ON_SOLID",
    "ACCENT",
    "ACCENT_HOVER",
    "ON_ACCENT",
    "DANGER",
    "DANGER_HOVER",
    "DANGER_MUTED",
    "WARNING",
    "WARNING_MUTED",
    "SUCCESS",
    "SUCCESS_MUTED",
    "INFO",
    "INFO_MUTED",
    "BUSY",
    "INPUT_BG",
    "SELECTION",
    "CARET",
    "CODE_TEXT",
    "CODE_WASH",
];

script_mod! {
    use mod.prelude.widgets_internal.*

    // Create the namespace first; nested field assignment requires it.
    mod.mpc_theme = {}

    mod.mpc_theme.dark = {
        BG: #(crate::palette::dark().bg)
        SURFACE: #(crate::palette::dark().surface)
        SURFACE_RAISED: #(crate::palette::dark().surface_raised)
        SURFACE_CARD: #(crate::palette::dark().surface_card)
        SURFACE_DIALOG: #(crate::palette::dark().surface_dialog)
        SURFACE_OVERLAY: #(crate::palette::dark().surface_overlay)
        ELEMENT_HOVER: #(crate::palette::dark().element_hover)
        ELEMENT_ACTIVE: #(crate::palette::dark().element_active)
        BORDER: #(crate::palette::dark().border)
        BORDER_STRONG: #(crate::palette::dark().border_strong)
        DIVIDER: #(crate::palette::dark().divider)
        TEXT: #(crate::palette::dark().text)
        TEXT_MUTED: #(crate::palette::dark().text_muted)
        TEXT_FAINT: #(crate::palette::dark().text_faint)
        SOLID: #(crate::palette::dark().solid)
        SOLID_HOVER: #(crate::palette::dark().solid_hover)
        ON_SOLID: #(crate::palette::dark().on_solid)
        ACCENT: #(crate::palette::dark().accent)
        ACCENT_HOVER: #(crate::palette::dark().accent_hover)
        ON_ACCENT: #(crate::palette::dark().on_accent)
        DANGER: #(crate::palette::dark().danger)
        DANGER_HOVER: #(crate::palette::dark().danger_hover)
        DANGER_MUTED: #(crate::palette::dark().danger_muted)
        WARNING: #(crate::palette::dark().warning)
        WARNING_MUTED: #(crate::palette::dark().warning_muted)
        SUCCESS: #(crate::palette::dark().success)
        SUCCESS_MUTED: #(crate::palette::dark().success_muted)
        INFO: #(crate::palette::dark().info)
        INFO_MUTED: #(crate::palette::dark().info_muted)
        BUSY: #(crate::palette::dark().busy)
        INPUT_BG: #(crate::palette::dark().input_bg)
        SELECTION: #(crate::palette::dark().selection)
        CARET: #(crate::palette::dark().caret)
        CODE_TEXT: #(crate::palette::dark().code_text)
        CODE_WASH: #(crate::palette::dark().code_wash)
        TRANSPARENT: #(
            Vec4f{x: 0.0, y: 0.0, z: 0.0, w: 0.0}
        )
    }

    mod.mpc_theme.light = {
        BG: #(crate::palette::light().bg)
        SURFACE: #(crate::palette::light().surface)
        SURFACE_RAISED: #(crate::palette::light().surface_raised)
        SURFACE_CARD: #(crate::palette::light().surface_card)
        SURFACE_DIALOG: #(crate::palette::light().surface_dialog)
        SURFACE_OVERLAY: #(crate::palette::light().surface_overlay)
        ELEMENT_HOVER: #(crate::palette::light().element_hover)
        ELEMENT_ACTIVE: #(crate::palette::light().element_active)
        BORDER: #(crate::palette::light().border)
        BORDER_STRONG: #(crate::palette::light().border_strong)
        DIVIDER: #(crate::palette::light().divider)
        TEXT: #(crate::palette::light().text)
        TEXT_MUTED: #(crate::palette::light().text_muted)
        TEXT_FAINT: #(crate::palette::light().text_faint)
        SOLID: #(crate::palette::light().solid)
        SOLID_HOVER: #(crate::palette::light().solid_hover)
        ON_SOLID: #(crate::palette::light().on_solid)
        ACCENT: #(crate::palette::light().accent)
        ACCENT_HOVER: #(crate::palette::light().accent_hover)
        ON_ACCENT: #(crate::palette::light().on_accent)
        DANGER: #(crate::palette::light().danger)
        DANGER_HOVER: #(crate::palette::light().danger_hover)
        DANGER_MUTED: #(crate::palette::light().danger_muted)
        WARNING: #(crate::palette::light().warning)
        WARNING_MUTED: #(crate::palette::light().warning_muted)
        SUCCESS: #(crate::palette::light().success)
        SUCCESS_MUTED: #(crate::palette::light().success_muted)
        INFO: #(crate::palette::light().info)
        INFO_MUTED: #(crate::palette::light().info_muted)
        BUSY: #(crate::palette::light().busy)
        INPUT_BG: #(crate::palette::light().input_bg)
        SELECTION: #(crate::palette::light().selection)
        CARET: #(crate::palette::light().caret)
        CODE_TEXT: #(crate::palette::light().code_text)
        CODE_WASH: #(crate::palette::light().code_wash)
        TRANSPARENT: #(
            Vec4f{x: 0.0, y: 0.0, z: 0.0, w: 0.0}
        )
    }

    // Active copy — defaults to dark.
    mod.mpc_theme = {
        BG: #(crate::palette::dark().bg)
        SURFACE: #(crate::palette::dark().surface)
        SURFACE_RAISED: #(crate::palette::dark().surface_raised)
        SURFACE_CARD: #(crate::palette::dark().surface_card)
        SURFACE_DIALOG: #(crate::palette::dark().surface_dialog)
        SURFACE_OVERLAY: #(crate::palette::dark().surface_overlay)
        ELEMENT_HOVER: #(crate::palette::dark().element_hover)
        ELEMENT_ACTIVE: #(crate::palette::dark().element_active)
        BORDER: #(crate::palette::dark().border)
        BORDER_STRONG: #(crate::palette::dark().border_strong)
        DIVIDER: #(crate::palette::dark().divider)
        TEXT: #(crate::palette::dark().text)
        TEXT_MUTED: #(crate::palette::dark().text_muted)
        TEXT_FAINT: #(crate::palette::dark().text_faint)
        SOLID: #(crate::palette::dark().solid)
        SOLID_HOVER: #(crate::palette::dark().solid_hover)
        ON_SOLID: #(crate::palette::dark().on_solid)
        ACCENT: #(crate::palette::dark().accent)
        ACCENT_HOVER: #(crate::palette::dark().accent_hover)
        ON_ACCENT: #(crate::palette::dark().on_accent)
        DANGER: #(crate::palette::dark().danger)
        DANGER_HOVER: #(crate::palette::dark().danger_hover)
        DANGER_MUTED: #(crate::palette::dark().danger_muted)
        WARNING: #(crate::palette::dark().warning)
        WARNING_MUTED: #(crate::palette::dark().warning_muted)
        SUCCESS: #(crate::palette::dark().success)
        SUCCESS_MUTED: #(crate::palette::dark().success_muted)
        INFO: #(crate::palette::dark().info)
        INFO_MUTED: #(crate::palette::dark().info_muted)
        BUSY: #(crate::palette::dark().busy)
        INPUT_BG: #(crate::palette::dark().input_bg)
        SELECTION: #(crate::palette::dark().selection)
        CARET: #(crate::palette::dark().caret)
        CODE_TEXT: #(crate::palette::dark().code_text)
        CODE_WASH: #(crate::palette::dark().code_wash)
        TRANSPARENT: #(
            Vec4f{x: 0.0, y: 0.0, z: 0.0, w: 0.0}
        )
    }
}

use crate::palette::Tokens;

impl Tokens {
    /// Read a token by name; used by MpThemeState for heap-copy switching.
    pub fn get_by_name(&self, name: &str) -> Option<Vec4f> {
        let v: Vec4f = match name {
            "BG" => self.bg,
            "SURFACE" => self.surface,
            "SURFACE_RAISED" => self.surface_raised,
            "SURFACE_CARD" => self.surface_card,
            "SURFACE_DIALOG" => self.surface_dialog,
            "SURFACE_OVERLAY" => self.surface_overlay,
            "ELEMENT_HOVER" => self.element_hover,
            "ELEMENT_ACTIVE" => self.element_active,
            "BORDER" => self.border,
            "BORDER_STRONG" => self.border_strong,
            "DIVIDER" => self.divider,
            "TEXT" => self.text,
            "TEXT_MUTED" => self.text_muted,
            "TEXT_FAINT" => self.text_faint,
            "SOLID" => self.solid,
            "SOLID_HOVER" => self.solid_hover,
            "ON_SOLID" => self.on_solid,
            "ACCENT" => self.accent,
            "ACCENT_HOVER" => self.accent_hover,
            "ON_ACCENT" => self.on_accent,
            "DANGER" => self.danger,
            "DANGER_HOVER" => self.danger_hover,
            "DANGER_MUTED" => self.danger_muted,
            "WARNING" => self.warning,
            "WARNING_MUTED" => self.warning_muted,
            "SUCCESS" => self.success,
            "SUCCESS_MUTED" => self.success_muted,
            "INFO" => self.info,
            "INFO_MUTED" => self.info_muted,
            "BUSY" => self.busy,
            "INPUT_BG" => self.input_bg,
            "SELECTION" => self.selection,
            "CARET" => self.caret,
            "CODE_TEXT" => self.code_text,
            "CODE_WASH" => self.code_wash,
            _ => return None,
        };
        Some(v)
    }
}
