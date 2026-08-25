use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpControlBar - floating bar with equal-flex rails around a
    // truly centred middle cluster (bezel control_bar port).
    // Width is the consumer's: equal rails need free space to be
    // equal about. Height 56, gap/end-inset 8.
    // ============================================================

    mod.widgets.MpControlBarBase = #(MpControlBar::register_widget(vm))

    mod.widgets.MpControlBar = set_type_default() do mod.widgets.MpControlBarBase{
        width: Fill
        height: 56.0
        flow: Right
        spacing: 8.0
        padding: Inset{left: 8.0, right: 8.0, top: 8.0, bottom: 8.0}
        align: Align{x: 0.0, y: 0.5}

        show_bg: true
        draw_bg +: {
            bg_color: instance(SURFACE_OVERLAY)
            border_color: instance(BORDER)
            radius: instance(28.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                sdf.fill(self.bg_color)
                sdf.stroke(self.border_color, 1.0)
                return sdf.result
            }
        }

        // Equal-flex rails keep the centre on axis regardless of
        // cluster widths — the classic toolbar bug avoided.
        leading := View{
            width: Fill
            height: Fit
            flow: Right
            spacing: 8.0
            align: Align{x: 0.0, y: 0.5}
        }

        center := View{
            width: Fit
            height: Fit
            flow: Right
            spacing: 8.0
            align: Align{x: 0.5, y: 0.5}
        }

        trailing := View{
            width: Fill
            height: Fit
            flow: Right
            spacing: 8.0
            align: Align{x: 1.0, y: 0.5}
        }
    }

    // Stadium variant - radius is half the height (media transport look)
    mod.widgets.MpControlBarPill = mod.widgets.MpControlBar{
        draw_bg +: {
            radius: instance(28.0)
        }
    }

    // Rounded variant at BUBBLE_RADIUS - composer/toolbar look
    mod.widgets.MpControlBarRounded = mod.widgets.MpControlBar{
        draw_bg +: {
            radius: instance(16.0)
        }
    }
}

// ============================================================
// MpControlBar - floating control bar
// ============================================================

#[derive(Script, ScriptHook, Widget)]
pub struct MpControlBar {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
}

impl Widget for MpControlBar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
