use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // PopupMenuPosition is not exposed to script by makepad (see makepad AGENTS.md
    // pitfall "Enums not exposed to script"). Expose it the same way makepad exposes
    // Flow/MouseCursor so we can keep the old `popup_menu_position: BelowInput`
    // behavior (the 2.0 DropDown default is OnSelected).
    mod.widgets.PopupMenuPosition = #(PopupMenuPosition::script_api(vm))

    // Default style dropdown with light theme popup
    mod.widgets.MpDropdown = mod.widgets.DropDownFlat{
        width: Fit
        height: Fit

        padding: Inset{left: 12.0, right: 24.0, top: 8.0, bottom: 8.0}
        popup_menu_position: mod.widgets.PopupMenuPosition.BelowInput

        draw_text +: {
            color: TEXT
            color_hover: TEXT
            color_focus: TEXT
            color_down: TEXT
            color_disabled: TEXT_FAINT

            text_style: theme.font_regular{font_size: 14.0}
        }

        draw_bg +: {
            border_size: 1.0
            border_radius: 6.0
            color_dither: 0.0

            color: INPUT_BG
            color_hover: ELEMENT_HOVER
            color_focus: INPUT_BG
            color_down: ELEMENT_ACTIVE
            color_disabled: ELEMENT_ACTIVE

            border_color: BORDER
            border_color_hover: BORDER_STRONG
            border_color_focus: ACCENT
            border_color_down: ACCENT
            border_color_disabled: SURFACE

            border_color_2: BORDER
            border_color_2_hover: BORDER_STRONG
            border_color_2_focus: ACCENT
            border_color_2_down: ACCENT
            border_color_2_disabled: SURFACE

            arrow_color: TEXT_MUTED
            arrow_color_hover: TEXT
            arrow_color_focus: ACCENT
            arrow_color_down: ACCENT
            arrow_color_disabled: TEXT_FAINT
        }

        popup_menu: mod.widgets.PopupMenuFlat{
            draw_bg +: {
                color: #xFFFFFFFF
                border_color: BORDER_STRONG
                border_color_2: BORDER_STRONG
                border_radius: 6.0
            }

            menu_item: mod.widgets.PopupMenuItem{
                draw_text +: {
                    text_style: theme.font_regular{font_size: 14.0}
                    color: TEXT
                    color_hover: TEXT
                    color_active: TEXT
                    color_disabled: TEXT_FAINT
                }
                draw_bg +: {
                    color: #xFFFFFFFF
                    color_hover: SURFACE
                    color_active: #xEAEAEA
                    color_disabled: ELEMENT_HOVER
                }
            }
        }
    }

    // Small size variant
    mod.widgets.MpDropdownSmall = mod.widgets.DropDownFlat{
        width: Fit
        height: Fit

        padding: Inset{left: 8.0, right: 20.0, top: 4.0, bottom: 4.0}
        popup_menu_position: mod.widgets.PopupMenuPosition.BelowInput

        draw_text +: {
            color: TEXT
            color_hover: TEXT
            color_focus: TEXT
            color_down: TEXT
            color_disabled: TEXT_FAINT

            text_style: theme.font_regular{font_size: 12.0}
        }

        draw_bg +: {
            border_size: 1.0
            border_radius: 4.0
            color_dither: 0.0

            color: INPUT_BG
            color_hover: ELEMENT_HOVER
            color_focus: INPUT_BG
            color_down: ELEMENT_ACTIVE
            color_disabled: ELEMENT_ACTIVE

            border_color: BORDER
            border_color_hover: BORDER_STRONG
            border_color_focus: ACCENT
            border_color_down: ACCENT
            border_color_disabled: SURFACE

            border_color_2: BORDER
            border_color_2_hover: BORDER_STRONG
            border_color_2_focus: ACCENT
            border_color_2_down: ACCENT
            border_color_2_disabled: SURFACE

            arrow_color: TEXT_MUTED
            arrow_color_hover: TEXT
            arrow_color_focus: ACCENT
            arrow_color_down: ACCENT
            arrow_color_disabled: TEXT_FAINT
        }

        popup_menu: mod.widgets.PopupMenuFlat{
            draw_bg +: {
                color: #xFFFFFFFF
                border_color: BORDER_STRONG
                border_color_2: BORDER_STRONG
                border_radius: 6.0
            }

            menu_item: mod.widgets.PopupMenuItem{
                draw_text +: {
                    text_style: theme.font_regular{font_size: 12.0}
                    color: TEXT
                    color_hover: TEXT
                    color_active: TEXT
                    color_disabled: TEXT_FAINT
                }
                draw_bg +: {
                    color: #xFFFFFFFF
                    color_hover: SURFACE
                    color_active: #xEAEAEA
                    color_disabled: ELEMENT_HOVER
                }
            }
        }
    }

    // Large size variant
    mod.widgets.MpDropdownLarge = mod.widgets.DropDownFlat{
        width: Fit
        height: Fit

        padding: Inset{left: 16.0, right: 28.0, top: 12.0, bottom: 12.0}
        popup_menu_position: mod.widgets.PopupMenuPosition.BelowInput

        draw_text +: {
            color: TEXT
            color_hover: TEXT
            color_focus: TEXT
            color_down: TEXT
            color_disabled: TEXT_FAINT

            text_style: theme.font_regular{font_size: 16.0}
        }

        draw_bg +: {
            border_size: 1.0
            border_radius: 8.0
            color_dither: 0.0

            color: INPUT_BG
            color_hover: ELEMENT_HOVER
            color_focus: INPUT_BG
            color_down: ELEMENT_ACTIVE
            color_disabled: ELEMENT_ACTIVE

            border_color: BORDER
            border_color_hover: BORDER_STRONG
            border_color_focus: ACCENT
            border_color_down: ACCENT
            border_color_disabled: SURFACE

            border_color_2: BORDER
            border_color_2_hover: BORDER_STRONG
            border_color_2_focus: ACCENT
            border_color_2_down: ACCENT
            border_color_2_disabled: SURFACE

            arrow_color: TEXT_MUTED
            arrow_color_hover: TEXT
            arrow_color_focus: ACCENT
            arrow_color_down: ACCENT
            arrow_color_disabled: TEXT_FAINT
        }

        popup_menu: mod.widgets.PopupMenuFlat{
            draw_bg +: {
                color: #xFFFFFFFF
                border_color: BORDER_STRONG
                border_color_2: BORDER_STRONG
                border_radius: 6.0
            }

            menu_item: mod.widgets.PopupMenuItem{
                draw_text +: {
                    text_style: theme.font_regular{font_size: 16.0}
                    color: TEXT
                    color_hover: TEXT
                    color_active: TEXT
                    color_disabled: TEXT_FAINT
                }
                draw_bg +: {
                    color: #xFFFFFFFF
                    color_hover: SURFACE
                    color_active: #xEAEAEA
                    color_disabled: ELEMENT_HOVER
                }
            }
        }
    }

    // Ghost variant - transparent background, no border
    mod.widgets.MpDropdownGhost = mod.widgets.DropDownFlat{
        width: Fit
        height: Fit

        padding: Inset{left: 12.0, right: 24.0, top: 8.0, bottom: 8.0}
        popup_menu_position: mod.widgets.PopupMenuPosition.BelowInput

        draw_text +: {
            color: TEXT
            color_hover: TEXT
            color_focus: TEXT
            color_down: TEXT
            color_disabled: TEXT_FAINT

            text_style: theme.font_regular{font_size: 14.0}
        }

        draw_bg +: {
            border_size: 0.0
            border_radius: 6.0
            color_dither: 0.0

            color: #x00000000
            color_hover: #x0000000D
            color_focus: #x00000000
            color_down: #x0000001A
            color_disabled: #x00000000

            border_color: #x00000000
            border_color_hover: #x00000000
            border_color_focus: #x00000000
            border_color_down: #x00000000
            border_color_disabled: #x00000000

            border_color_2: #x00000000
            border_color_2_hover: #x00000000
            border_color_2_focus: #x00000000
            border_color_2_down: #x00000000
            border_color_2_disabled: #x00000000

            arrow_color: TEXT_MUTED
            arrow_color_hover: TEXT
            arrow_color_focus: ACCENT
            arrow_color_down: ACCENT
            arrow_color_disabled: TEXT_FAINT
        }

        popup_menu: mod.widgets.PopupMenuFlat{
            draw_bg +: {
                color: #xFFFFFFFF
                border_color: BORDER_STRONG
                border_color_2: BORDER_STRONG
                border_radius: 6.0
            }

            menu_item: mod.widgets.PopupMenuItem{
                draw_text +: {
                    text_style: theme.font_regular{font_size: 14.0}
                    color: TEXT
                    color_hover: TEXT
                    color_active: TEXT
                    color_disabled: TEXT_FAINT
                }
                draw_bg +: {
                    color: #xFFFFFFFF
                    color_hover: SURFACE
                    color_active: #xEAEAEA
                    color_disabled: ELEMENT_HOVER
                }
            }
        }
    }

    // Outline variant - transparent background with border
    mod.widgets.MpDropdownOutline = mod.widgets.DropDownFlat{
        width: Fit
        height: Fit

        padding: Inset{left: 12.0, right: 24.0, top: 8.0, bottom: 8.0}
        popup_menu_position: mod.widgets.PopupMenuPosition.BelowInput

        draw_text +: {
            color: TEXT
            color_hover: TEXT
            color_focus: TEXT
            color_down: TEXT
            color_disabled: TEXT_FAINT

            text_style: theme.font_regular{font_size: 14.0}
        }

        draw_bg +: {
            border_size: 1.0
            border_radius: 6.0
            color_dither: 0.0

            color: #x00000000
            color_hover: #x0000000D
            color_focus: #x00000000
            color_down: #x0000001A
            color_disabled: #x00000000

            border_color: BORDER_STRONG
            border_color_hover: #xa3a3a3
            border_color_focus: ACCENT
            border_color_down: ACCENT
            border_color_disabled: BORDER

            border_color_2: BORDER_STRONG
            border_color_2_hover: #xa3a3a3
            border_color_2_focus: ACCENT
            border_color_2_down: ACCENT
            border_color_2_disabled: BORDER

            arrow_color: TEXT_MUTED
            arrow_color_hover: TEXT
            arrow_color_focus: ACCENT
            arrow_color_down: ACCENT
            arrow_color_disabled: TEXT_FAINT
        }

        popup_menu: mod.widgets.PopupMenuFlat{
            draw_bg +: {
                color: #xFFFFFFFF
                border_color: BORDER_STRONG
                border_color_2: BORDER_STRONG
                border_radius: 6.0
            }

            menu_item: mod.widgets.PopupMenuItem{
                draw_text +: {
                    text_style: theme.font_regular{font_size: 14.0}
                    color: TEXT
                    color_hover: TEXT
                    color_active: TEXT
                    color_disabled: TEXT_FAINT
                }
                draw_bg +: {
                    color: #xFFFFFFFF
                    color_hover: SURFACE
                    color_active: #xEAEAEA
                    color_disabled: ELEMENT_HOVER
                }
            }
        }
    }

    // Filled variant - light gray background
    mod.widgets.MpDropdownFilled = mod.widgets.DropDownFlat{
        width: Fit
        height: Fit

        padding: Inset{left: 12.0, right: 24.0, top: 8.0, bottom: 8.0}
        popup_menu_position: mod.widgets.PopupMenuPosition.BelowInput

        draw_text +: {
            color: TEXT
            color_hover: TEXT
            color_focus: TEXT
            color_down: TEXT
            color_disabled: TEXT_FAINT

            text_style: theme.font_regular{font_size: 14.0}
        }

        draw_bg +: {
            border_size: 0.0
            border_radius: 6.0
            color_dither: 0.0

            color: SURFACE
            color_hover: BORDER
            color_focus: SURFACE
            color_down: BORDER_STRONG
            color_disabled: #xfafafa

            border_color: #x00000000
            border_color_hover: #x00000000
            border_color_focus: ACCENT
            border_color_down: ACCENT
            border_color_disabled: #x00000000

            border_color_2: #x00000000
            border_color_2_hover: #x00000000
            border_color_2_focus: ACCENT
            border_color_2_down: ACCENT
            border_color_2_disabled: #x00000000

            arrow_color: #x525252
            arrow_color_hover: TEXT
            arrow_color_focus: ACCENT
            arrow_color_down: ACCENT
            arrow_color_disabled: TEXT_FAINT
        }

        popup_menu: mod.widgets.PopupMenuFlat{
            draw_bg +: {
                color: #xFFFFFFFF
                border_color: BORDER_STRONG
                border_color_2: BORDER_STRONG
                border_radius: 6.0
            }

            menu_item: mod.widgets.PopupMenuItem{
                draw_text +: {
                    text_style: theme.font_regular{font_size: 14.0}
                    color: TEXT
                    color_hover: TEXT
                    color_active: TEXT
                    color_disabled: TEXT_FAINT
                }
                draw_bg +: {
                    color: #xFFFFFFFF
                    color_hover: SURFACE
                    color_active: #xEAEAEA
                    color_disabled: ELEMENT_HOVER
                }
            }
        }
    }

    // Accent/Primary variant - blue accent color
    mod.widgets.MpDropdownAccent = mod.widgets.DropDownFlat{
        width: Fit
        height: Fit

        padding: Inset{left: 12.0, right: 24.0, top: 8.0, bottom: 8.0}
        popup_menu_position: mod.widgets.PopupMenuPosition.BelowInput

        draw_text +: {
            color: INPUT_BG
            color_hover: INPUT_BG
            color_focus: INPUT_BG
            color_down: INPUT_BG
            color_disabled: #x94a3b8

            text_style: theme.font_regular{font_size: 14.0}
        }

        draw_bg +: {
            border_size: 0.0
            border_radius: 6.0
            color_dither: 0.0

            color: ACCENT
            color_hover: #x0369a1
            color_focus: ACCENT
            color_down: #x075985
            color_disabled: #xbae6fd

            border_color: #x00000000
            border_color_hover: #x00000000
            border_color_focus: #x00000000
            border_color_down: #x00000000
            border_color_disabled: #x00000000

            border_color_2: #x00000000
            border_color_2_hover: #x00000000
            border_color_2_focus: #x00000000
            border_color_2_down: #x00000000
            border_color_2_disabled: #x00000000

            arrow_color: INPUT_BG
            arrow_color_hover: INPUT_BG
            arrow_color_focus: INPUT_BG
            arrow_color_down: INPUT_BG
            arrow_color_disabled: #x94a3b8
        }

        popup_menu: mod.widgets.PopupMenuFlat{
            draw_bg +: {
                color: #xFFFFFFFF
                border_color: BORDER_STRONG
                border_color_2: BORDER_STRONG
                border_radius: 6.0
            }

            menu_item: mod.widgets.PopupMenuItem{
                draw_text +: {
                    text_style: theme.font_regular{font_size: 14.0}
                    color: TEXT
                    color_hover: TEXT
                    color_active: TEXT
                    color_disabled: TEXT_FAINT
                }
                draw_bg +: {
                    color: #xFFFFFFFF
                    color_hover: SURFACE
                    color_active: #xEAEAEA
                    color_disabled: ELEMENT_HOVER
                }
            }
        }
    }

    // Danger variant - red color for destructive actions
    mod.widgets.MpDropdownDanger = mod.widgets.DropDownFlat{
        width: Fit
        height: Fit

        padding: Inset{left: 12.0, right: 24.0, top: 8.0, bottom: 8.0}
        popup_menu_position: mod.widgets.PopupMenuPosition.BelowInput

        draw_text +: {
            color: INPUT_BG
            color_hover: INPUT_BG
            color_focus: INPUT_BG
            color_down: INPUT_BG
            color_disabled: #xfca5a5

            text_style: theme.font_regular{font_size: 14.0}
        }

        draw_bg +: {
            border_size: 0.0
            border_radius: 6.0
            color_dither: 0.0

            color: #xdc2626
            color_hover: #xb91c1c
            color_focus: #xdc2626
            color_down: #x991b1b
            color_disabled: #xfecaca

            border_color: #x00000000
            border_color_hover: #x00000000
            border_color_focus: #x00000000
            border_color_down: #x00000000
            border_color_disabled: #x00000000

            border_color_2: #x00000000
            border_color_2_hover: #x00000000
            border_color_2_focus: #x00000000
            border_color_2_down: #x00000000
            border_color_2_disabled: #x00000000

            arrow_color: INPUT_BG
            arrow_color_hover: INPUT_BG
            arrow_color_focus: INPUT_BG
            arrow_color_down: INPUT_BG
            arrow_color_disabled: #xfca5a5
        }

        popup_menu: mod.widgets.PopupMenuFlat{
            draw_bg +: {
                color: #xFFFFFFFF
                border_color: BORDER_STRONG
                border_color_2: BORDER_STRONG
                border_radius: 6.0
            }

            menu_item: mod.widgets.PopupMenuItem{
                draw_text +: {
                    text_style: theme.font_regular{font_size: 14.0}
                    color: TEXT
                    color_hover: TEXT
                    color_active: TEXT
                    color_disabled: TEXT_FAINT
                }
                draw_bg +: {
                    color: #xFFFFFFFF
                    color_hover: SURFACE
                    color_active: #xEAEAEA
                    color_disabled: ELEMENT_HOVER
                }
            }
        }
    }
}
