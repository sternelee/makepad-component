use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpKbd - keyboard shortcut chip
    // Usage: mod.widgets.MpKbd{ text: "⌘K" }
    // ============================================================

    mod.widgets.MpKbdBase = #(MpKbd::register_widget(vm))
    mod.widgets.MpKbd = set_type_default() do mod.widgets.MpKbdBase{
        width: Fit
        height: Fit
        padding: Inset{left: 6, right: 6, top: 2, bottom: 3}
        align: Align{x: 0.5, y: 0.5}

        draw_bg +: {
            bg_color: instance(SURFACE_CARD)
            border_color: instance(BORDER)
            radius: instance(4.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                sdf.fill_keep(self.bg_color)
                sdf.stroke(self.border_color, 1.0)
                return sdf.result
            }
        }

        draw_text +: {
            text_style: theme.font_regular{font_size: 11.0}
            color: TEXT_MUTED
        }

        text: "Key"
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct MpKbd {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_bg: DrawQuad,

    #[live]
    draw_text: DrawText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[live]
    text: ArcStringMut,
}

impl Widget for MpKbd {
    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_text
            .draw_walk(cx, Walk::fit(), Align::default(), self.text.as_ref());
        self.draw_bg.end(cx);
        DrawStep::done()
    }
}
