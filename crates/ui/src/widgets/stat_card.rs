use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpStatCard - dashboard metric tile (label + value + delta)
    // ============================================================

    mod.widgets.MpStatCard = View{
        width: 200
        height: Fit
        flow: Down
        spacing: 4
        padding: Inset{top: 16, right: 16, bottom: 16, left: 16}

        show_bg: true
        draw_bg +: {
            bg_color: instance(SURFACE_CARD)
            border_color: instance(BORDER)
            radius: instance(12.0)
            border_width: instance(1.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(self.border_width, self.border_width, self.rect_size.x - self.border_width * 2.0, self.rect_size.y - self.border_width * 2.0, self.radius)
                sdf.fill(self.bg_color)
                sdf.stroke(self.border_color, self.border_width)
                return sdf.result
            }
        }

        stat_label := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 12.0}
                color: TEXT_MUTED
            }
            text: "Label"
        }

        stat_value := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_bold{font_size: 24.0}
                color: TEXT
            }
            text: "0"
        }

        stat_delta := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 12.0}
                color: SUCCESS
            }
            text: "+0.0%"
        }
    }
}
