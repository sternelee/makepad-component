use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpScrollArea - themed scrolling containers (bezel style)
    // ============================================================

    // Vertical scroll area
    mod.widgets.MpScrollArea = mod.widgets.ScrollYView{
        width: Fill
        height: Fill
        flow: Down

        scroll_bars +: {
            show_scroll_x: false
            show_scroll_y: true
            scroll_bar_y +: {
                bar_size: 10.0
                bar_side_margin: 3.0
                draw_bg +: {
                    size: uniform(6.0)
                    border_radius: uniform(3.0)
                    border_size: uniform(0.0)

                    color: uniform(TEXT_FAINT)
                    color_hover: uniform(TEXT_MUTED)
                    color_drag: uniform(TEXT_MUTED)
                }
            }
        }
    }

    // Both-axis scroll area
    mod.widgets.MpScrollXYArea = mod.widgets.ScrollXYView{
        width: Fill
        height: Fill
        flow: Down

        scroll_bars +: {
            show_scroll_x: true
            show_scroll_y: true
            scroll_bar_x +: {
                bar_size: 10.0
                bar_side_margin: 3.0
                draw_bg +: {
                    size: uniform(6.0)
                    border_radius: uniform(3.0)
                    border_size: uniform(0.0)

                    color: uniform(TEXT_FAINT)
                    color_hover: uniform(TEXT_MUTED)
                    color_drag: uniform(TEXT_MUTED)
                }
            }
            scroll_bar_y +: {
                bar_size: 10.0
                bar_side_margin: 3.0
                draw_bg +: {
                    size: uniform(6.0)
                    border_radius: uniform(3.0)
                    border_size: uniform(0.0)

                    color: uniform(TEXT_FAINT)
                    color_hover: uniform(TEXT_MUTED)
                    color_drag: uniform(TEXT_MUTED)
                }
            }
        }
    }
}
