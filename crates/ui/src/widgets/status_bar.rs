use makepad_widgets::*;

/// Default status bar height (content rows sit inside this). Layouts that
/// pin the bar to the bottom of a pane can reserve this much.
pub const STATUS_BAR_HEIGHT: f64 = 28.0;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpStatusBar - bottom status bar with left / center / right regions
    // (gpui StatusBar port, pure-DSL container like MpCard)
    //
    // Usage:
    //   mod.widgets.MpStatusBar{
    //       left +: { mod.widgets.MpStatusText{ text: "Ready" } }
    //       right +: { mod.widgets.MpStatusText{ text: "UTF-8" } }
    //   }
    // ============================================================

    // Muted label tuned for the status bar (11.5px, muted)
    mod.widgets.MpStatusText = Label{
        width: Fit
        height: Fit
        draw_text +: {
            text_style: theme.font_regular{font_size: 11.5}
            color: TEXT_MUTED
        }
        text: ""
    }

    // Circular status LED used in status bars (Online/Away/Error style)
    mod.widgets.MpStatusLed = View{
        width: 8.0
        height: 8.0
        margin: Inset{right: 0.0}

        show_bg: true
        draw_bg +: {
            color: instance(SUCCESS)
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let r = self.rect_size.x * 0.5
                sdf.circle(r, r, r)
                sdf.fill(self.color)
                return sdf.result
            }
        }
    }

    mod.widgets.MpStatusBar = View{
        width: Fill
        height: Fit
        flow: Right
        align: Align{y: 0.5}
        spacing: 8.0
        padding: Inset{left: 12.0, right: 12.0, top: 5.0, bottom: 5.0}

        show_bg: true
        draw_bg +: {
            color: instance(SURFACE_RAISED)
            border_color: instance(BORDER)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let sz = self.rect_size

                sdf.rect(0.0, 0.0, sz.x, sz.y)
                sdf.fill(self.color)
                // Top hairline separates the bar from the content above
                sdf.move_to(0.0, 0.5)
                sdf.line_to(sz.x, 0.5)
                sdf.stroke(self.border_color, 1.0)
                return sdf.result
            }
        }

        // Pinned left region
        left := View{
            width: Fit, height: Fit,
            flow: Right,
            spacing: 8.0,
            align: Align{y: 0.5}
        }

        // Center region: fills the middle; content centers when both ends
        // are populated, end-aligns with only left, start-aligns otherwise
        center := View{
            width: Fill, height: Fit,
            flow: Right,
            spacing: 8.0,
            align: Align{y: 0.5}
        }

        // Pinned right region
        right := View{
            width: Fit, height: Fit,
            flow: Right,
            spacing: 8.0,
            align: Align{y: 0.5}
        }
    }
}
