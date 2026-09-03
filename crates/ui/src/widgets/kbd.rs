use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

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

    /// Five-step size driving padding and font. Kbd chips are compact, so
    /// paddings are scaled down from the control metrics.
    #[live]
    size: MpSize,
}

impl Widget for MpKbd {
    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // Compact metric mapping: half paddings, slightly smaller font.
        self.layout.padding = Inset {
            left: self.size.padding_h() * 0.5,
            right: self.size.padding_h() * 0.5,
            top: self.size.padding_v() * 0.33,
            bottom: self.size.padding_v() * 0.5,
        };
        self.draw_text.text_style.font_size = self.size.font_size() - 2.0;
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_text
            .draw_walk(cx, Walk::fit(), Align::default(), self.text.as_ref());
        self.draw_bg.end(cx);
        DrawStep::done()
    }
}

impl MpKbd {
    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            self.redraw(cx);
        }
    }
}

impl MpKbdRef {
    pub fn size(&self) -> MpSize {
        if let Some(inner) = self.borrow() {
            inner.size()
        } else {
            MpSize::default()
        }
    }

    pub fn set_size(&self, cx: &mut Cx, size: MpSize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_size(cx, size);
        }
    }
}
