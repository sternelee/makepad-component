use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

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
            color: #x0a0a0a
            color_hover: #x0a0a0a
            color_focus: #x0a0a0a
            color_down: #x0a0a0a
            color_disabled: #x737373

            text_style: theme.font_regular{font_size: 14.0}
        }

        draw_bg +: {
            border_size: 1.0
            border_radius: 6.0
            color_dither: 0.0

            color: #xFFFFFF
            color_hover: #xFAFAFA
            color_focus: #xFFFFFF
            color_down: #xF5F5F5
            color_disabled: #xF5F5F5

            border_color: #xe5e5e5
            border_color_hover: #xd4d4d4
            border_color_focus: #x0284c7
            border_color_down: #x0284c7
            border_color_disabled: #xf5f5f5

            border_color_2: #xe5e5e5
            border_color_2_hover: #xd4d4d4
            border_color_2_focus: #x0284c7
            border_color_2_down: #x0284c7
            border_color_2_disabled: #xf5f5f5

            arrow_color: #x666666
            arrow_color_hover: #x333333
            arrow_color_focus: #x4A90D9
            arrow_color_down: #x4A90D9
            arrow_color_disabled: #x9E9E9E
        }

        popup_menu: mod.widgets.PopupMenuFlat{
            draw_bg +: {
                color: #xFFFFFFFF
                border_color: #xd4d4d4
                border_color_2: #xd4d4d4
                border_radius: 6.0
            }

            menu_item: mod.widgets.PopupMenuItem{
                draw_text +: {
                    text_style: theme.font_regular{font_size: 14.0}
                    color: #x0a0a0a
                    color_hover: #x0a0a0a
                    color_active: #x0a0a0a
                    color_disabled: #x737373
                }
                draw_bg +: {
                    color: #xFFFFFFFF
                    color_hover: #xf5f5f5
                    color_active: #xEAEAEA
                    color_disabled: #xFAFAFA
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
            color: #x0a0a0a
            color_hover: #x0a0a0a
            color_focus: #x0a0a0a
            color_down: #x0a0a0a
            color_disabled: #x737373

            text_style: theme.font_regular{font_size: 12.0}
        }

        draw_bg +: {
            border_size: 1.0
            border_radius: 4.0
            color_dither: 0.0

            color: #xFFFFFF
            color_hover: #xFAFAFA
            color_focus: #xFFFFFF
            color_down: #xF5F5F5
            color_disabled: #xF5F5F5

            border_color: #xe5e5e5
            border_color_hover: #xd4d4d4
            border_color_focus: #x0284c7
            border_color_down: #x0284c7
            border_color_disabled: #xf5f5f5

            border_color_2: #xe5e5e5
            border_color_2_hover: #xd4d4d4
            border_color_2_focus: #x0284c7
            border_color_2_down: #x0284c7
            border_color_2_disabled: #xf5f5f5

            arrow_color: #x666666
            arrow_color_hover: #x333333
            arrow_color_focus: #x4A90D9
            arrow_color_down: #x4A90D9
            arrow_color_disabled: #x9E9E9E
        }

        popup_menu: mod.widgets.PopupMenuFlat{
            draw_bg +: {
                color: #xFFFFFFFF
                border_color: #xd4d4d4
                border_color_2: #xd4d4d4
                border_radius: 6.0
            }

            menu_item: mod.widgets.PopupMenuItem{
                draw_text +: {
                    text_style: theme.font_regular{font_size: 12.0}
                    color: #x0a0a0a
                    color_hover: #x0a0a0a
                    color_active: #x0a0a0a
                    color_disabled: #x737373
                }
                draw_bg +: {
                    color: #xFFFFFFFF
                    color_hover: #xf5f5f5
                    color_active: #xEAEAEA
                    color_disabled: #xFAFAFA
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
            color: #x0a0a0a
            color_hover: #x0a0a0a
            color_focus: #x0a0a0a
            color_down: #x0a0a0a
            color_disabled: #x737373

            text_style: theme.font_regular{font_size: 16.0}
        }

        draw_bg +: {
            border_size: 1.0
            border_radius: 8.0
            color_dither: 0.0

            color: #xFFFFFF
            color_hover: #xFAFAFA
            color_focus: #xFFFFFF
            color_down: #xF5F5F5
            color_disabled: #xF5F5F5

            border_color: #xe5e5e5
            border_color_hover: #xd4d4d4
            border_color_focus: #x0284c7
            border_color_down: #x0284c7
            border_color_disabled: #xf5f5f5

            border_color_2: #xe5e5e5
            border_color_2_hover: #xd4d4d4
            border_color_2_focus: #x0284c7
            border_color_2_down: #x0284c7
            border_color_2_disabled: #xf5f5f5

            arrow_color: #x666666
            arrow_color_hover: #x333333
            arrow_color_focus: #x4A90D9
            arrow_color_down: #x4A90D9
            arrow_color_disabled: #x9E9E9E
        }

        popup_menu: mod.widgets.PopupMenuFlat{
            draw_bg +: {
                color: #xFFFFFFFF
                border_color: #xd4d4d4
                border_color_2: #xd4d4d4
                border_radius: 6.0
            }

            menu_item: mod.widgets.PopupMenuItem{
                draw_text +: {
                    text_style: theme.font_regular{font_size: 16.0}
                    color: #x0a0a0a
                    color_hover: #x0a0a0a
                    color_active: #x0a0a0a
                    color_disabled: #x737373
                }
                draw_bg +: {
                    color: #xFFFFFFFF
                    color_hover: #xf5f5f5
                    color_active: #xEAEAEA
                    color_disabled: #xFAFAFA
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
            color: #x0a0a0a
            color_hover: #x0a0a0a
            color_focus: #x0a0a0a
            color_down: #x0a0a0a
            color_disabled: #x737373

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

            arrow_color: #x666666
            arrow_color_hover: #x333333
            arrow_color_focus: #x0284c7
            arrow_color_down: #x0284c7
            arrow_color_disabled: #x9E9E9E
        }

        popup_menu: mod.widgets.PopupMenuFlat{
            draw_bg +: {
                color: #xFFFFFFFF
                border_color: #xd4d4d4
                border_color_2: #xd4d4d4
                border_radius: 6.0
            }

            menu_item: mod.widgets.PopupMenuItem{
                draw_text +: {
                    text_style: theme.font_regular{font_size: 14.0}
                    color: #x0a0a0a
                    color_hover: #x0a0a0a
                    color_active: #x0a0a0a
                    color_disabled: #x737373
                }
                draw_bg +: {
                    color: #xFFFFFFFF
                    color_hover: #xf5f5f5
                    color_active: #xEAEAEA
                    color_disabled: #xFAFAFA
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
            color: #x0a0a0a
            color_hover: #x0a0a0a
            color_focus: #x0a0a0a
            color_down: #x0a0a0a
            color_disabled: #x737373

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

            border_color: #xd4d4d4
            border_color_hover: #xa3a3a3
            border_color_focus: #x0284c7
            border_color_down: #x0284c7
            border_color_disabled: #xe5e5e5

            border_color_2: #xd4d4d4
            border_color_2_hover: #xa3a3a3
            border_color_2_focus: #x0284c7
            border_color_2_down: #x0284c7
            border_color_2_disabled: #xe5e5e5

            arrow_color: #x666666
            arrow_color_hover: #x333333
            arrow_color_focus: #x0284c7
            arrow_color_down: #x0284c7
            arrow_color_disabled: #x9E9E9E
        }

        popup_menu: mod.widgets.PopupMenuFlat{
            draw_bg +: {
                color: #xFFFFFFFF
                border_color: #xd4d4d4
                border_color_2: #xd4d4d4
                border_radius: 6.0
            }

            menu_item: mod.widgets.PopupMenuItem{
                draw_text +: {
                    text_style: theme.font_regular{font_size: 14.0}
                    color: #x0a0a0a
                    color_hover: #x0a0a0a
                    color_active: #x0a0a0a
                    color_disabled: #x737373
                }
                draw_bg +: {
                    color: #xFFFFFFFF
                    color_hover: #xf5f5f5
                    color_active: #xEAEAEA
                    color_disabled: #xFAFAFA
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
            color: #x0a0a0a
            color_hover: #x0a0a0a
            color_focus: #x0a0a0a
            color_down: #x0a0a0a
            color_disabled: #x737373

            text_style: theme.font_regular{font_size: 14.0}
        }

        draw_bg +: {
            border_size: 0.0
            border_radius: 6.0
            color_dither: 0.0

            color: #xf5f5f5
            color_hover: #xe5e5e5
            color_focus: #xf5f5f5
            color_down: #xd4d4d4
            color_disabled: #xfafafa

            border_color: #x00000000
            border_color_hover: #x00000000
            border_color_focus: #x0284c7
            border_color_down: #x0284c7
            border_color_disabled: #x00000000

            border_color_2: #x00000000
            border_color_2_hover: #x00000000
            border_color_2_focus: #x0284c7
            border_color_2_down: #x0284c7
            border_color_2_disabled: #x00000000

            arrow_color: #x525252
            arrow_color_hover: #x333333
            arrow_color_focus: #x0284c7
            arrow_color_down: #x0284c7
            arrow_color_disabled: #x9E9E9E
        }

        popup_menu: mod.widgets.PopupMenuFlat{
            draw_bg +: {
                color: #xFFFFFFFF
                border_color: #xd4d4d4
                border_color_2: #xd4d4d4
                border_radius: 6.0
            }

            menu_item: mod.widgets.PopupMenuItem{
                draw_text +: {
                    text_style: theme.font_regular{font_size: 14.0}
                    color: #x0a0a0a
                    color_hover: #x0a0a0a
                    color_active: #x0a0a0a
                    color_disabled: #x737373
                }
                draw_bg +: {
                    color: #xFFFFFFFF
                    color_hover: #xf5f5f5
                    color_active: #xEAEAEA
                    color_disabled: #xFAFAFA
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
            color: #xFFFFFF
            color_hover: #xFFFFFF
            color_focus: #xFFFFFF
            color_down: #xFFFFFF
            color_disabled: #x94a3b8

            text_style: theme.font_regular{font_size: 14.0}
        }

        draw_bg +: {
            border_size: 0.0
            border_radius: 6.0
            color_dither: 0.0

            color: #x0284c7
            color_hover: #x0369a1
            color_focus: #x0284c7
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

            arrow_color: #xFFFFFF
            arrow_color_hover: #xFFFFFF
            arrow_color_focus: #xFFFFFF
            arrow_color_down: #xFFFFFF
            arrow_color_disabled: #x94a3b8
        }

        popup_menu: mod.widgets.PopupMenuFlat{
            draw_bg +: {
                color: #xFFFFFFFF
                border_color: #xd4d4d4
                border_color_2: #xd4d4d4
                border_radius: 6.0
            }

            menu_item: mod.widgets.PopupMenuItem{
                draw_text +: {
                    text_style: theme.font_regular{font_size: 14.0}
                    color: #x0a0a0a
                    color_hover: #x0a0a0a
                    color_active: #x0a0a0a
                    color_disabled: #x737373
                }
                draw_bg +: {
                    color: #xFFFFFFFF
                    color_hover: #xf5f5f5
                    color_active: #xEAEAEA
                    color_disabled: #xFAFAFA
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
            color: #xFFFFFF
            color_hover: #xFFFFFF
            color_focus: #xFFFFFF
            color_down: #xFFFFFF
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

            arrow_color: #xFFFFFF
            arrow_color_hover: #xFFFFFF
            arrow_color_focus: #xFFFFFF
            arrow_color_down: #xFFFFFF
            arrow_color_disabled: #xfca5a5
        }

        popup_menu: mod.widgets.PopupMenuFlat{
            draw_bg +: {
                color: #xFFFFFFFF
                border_color: #xd4d4d4
                border_color_2: #xd4d4d4
                border_radius: 6.0
            }

            menu_item: mod.widgets.PopupMenuItem{
                draw_text +: {
                    text_style: theme.font_regular{font_size: 14.0}
                    color: #x0a0a0a
                    color_hover: #x0a0a0a
                    color_active: #x0a0a0a
                    color_disabled: #x737373
                }
                draw_bg +: {
                    color: #xFFFFFFFF
                    color_hover: #xf5f5f5
                    color_active: #xEAEAEA
                    color_disabled: #xFAFAFA
                }
            }
        }
    }
}
