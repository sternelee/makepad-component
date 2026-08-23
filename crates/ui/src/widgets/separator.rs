use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpSeparator - shadcn-style visual divider
    // ============================================================

    // Horizontal Separator
    mod.widgets.MpSeparator = mod.widgets.SolidView{
        width: Fill
        height: 1
        show_bg: true
        draw_bg +: { color: BORDER }
    }

    // Vertical Separator
    mod.widgets.MpSeparatorVertical = mod.widgets.SolidView{
        width: 1
        height: Fill
        show_bg: true
        draw_bg +: { color: BORDER }
    }

    // Separator with margin/padding
    mod.widgets.MpSeparatorWithMargin = mod.widgets.View{
        width: Fill
        height: Fit
        padding: Inset{top: 8.0, bottom: 8.0}

        SolidView{
            width: Fill
            height: 1
            show_bg: true
            draw_bg +: { color: BORDER }
        }
    }

    // Separator with label (text centered between lines - shadcn style)
    mod.widgets.MpSeparatorWithLabel = mod.widgets.View{
        width: Fill
        height: Fit
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{top: 12.0, bottom: 12.0}

        left_line := SolidView{
            width: Fill
            height: 1
            show_bg: true
            draw_bg +: { color: BORDER }
        }

        label := Label{
            width: Fit
            margin: Inset{left: 16.0, right: 16.0}
            draw_text +: {
                text_style: theme.font_regular{font_size: 12.0}
                color: TEXT_MUTED
            }
            text: "CONTINUE"
        }

        right_line := SolidView{
            width: Fill
            height: 1
            show_bg: true
            draw_bg +: { color: BORDER }
        }
    }

    // Subtle separator (lighter color)
    mod.widgets.MpSeparatorSubtle = mod.widgets.SolidView{
        width: Fill
        height: 1
        show_bg: true
        draw_bg +: { color: #xf1f5f9 }
    }

    // Thick Separator
    mod.widgets.MpSeparatorThick = mod.widgets.SolidView{
        width: Fill
        height: 2
        show_bg: true
        draw_bg +: { color: BORDER }
    }
}
