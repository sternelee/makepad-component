use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpEmptyState - centered icon + title + description placeholder
    // ============================================================

    mod.widgets.MpEmptyState = View{
        width: Fill
        height: Fill
        flow: Down
        spacing: 8
        align: Align{x: 0.5, y: 0.5}

        icon := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 32.0}
                color: TEXT_FAINT
            }
            text: "○"
        }

        title := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_bold{font_size: 15.0}
                color: TEXT
            }
            text: "Nothing here yet"
        }

        description := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 13.0}
                color: TEXT_MUTED
            }
            text: "Get started by adding your first item."
        }
    }
}
