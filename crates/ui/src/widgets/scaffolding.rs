use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpPageHeader - page title + subtitle stack
    // ============================================================

    mod.widgets.MpPageHeader = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 4.0

        title := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_bold{font_size: 20.0}
                color: TEXT
            }
            text: "Title"
        }

        subtitle := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 13.0}
                color: TEXT_MUTED
            }
            text: ""
        }
    }

    // ============================================================
    // MpCardRow - horizontal row wrapper for cards/options
    // ============================================================

    mod.widgets.MpCardRow = View{
        width: Fill
        height: Fit
        flow: Right
        spacing: 8.0
        align: Align{x: 0.0, y: 0.5}
    }

    // ============================================================
    // MpGroupBox - titled bordered group container
    // ============================================================

    mod.widgets.MpGroupBox = set_type_default() do #(MpGroupBox::register_widget(vm)){
        width: Fill
        height: Fit
        flow: Down
        spacing: 10.0
        padding: Inset{left: 16.0, right: 16.0, top: 12.0, bottom: 12.0}

        cursor: MouseCursor.Default

        show_bg: true
        draw_bg +: {
            bg_color: instance(SURFACE_CARD)
            border_color: instance(BORDER)
            border_width: instance(1.0)
            radius: instance(12.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(
                    self.border_width,
                    self.border_width,
                    self.rect_size.x - self.border_width * 2.0,
                    self.rect_size.y - self.border_width * 2.0,
                    self.radius
                )
                sdf.fill(self.bg_color)
                if (self.border_width > 0.0) {
                    sdf.stroke(self.border_color, self.border_width)
                }
                return sdf.result
            }
        }

        title := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 13.0}
                color: TEXT_MUTED
            }
            text: "Group"
        }

        content := View{
            width: Fill
            height: Fit
            flow: Down
            spacing: 8.0
        }
    }
}

// ============================================================
// MpGroupBox - titled bordered group container
// ============================================================

#[derive(Script, ScriptHook, Widget)]
pub struct MpGroupBox {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    #[live]
    title_text: ArcStringMut,

    /// Five-step size driving padding and title font.
    #[live]
    size: MpSize,

    /// Last size applied to the title (avoids re-applying every draw).
    #[rust]
    applied_size: Option<MpSize>,
}

impl Widget for MpGroupBox {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Metrics from the size system (Medium = the DSL 16/12 look)
        self.view.layout.padding = Inset {
            left: self.size.padding_h() + 4.0,
            right: self.size.padding_h() + 4.0,
            top: self.size.padding_v() + 6.0,
            bottom: self.size.padding_v() + 6.0,
        };

        if self.applied_size != Some(self.size) {
            self.applied_size = Some(self.size);
            if let Some(mut title_label) = self.view.label(cx, ids!(title)).borrow_mut() {
                title_label.draw_text.text_style.font_size = self.size.font_size();
            }
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpGroupBox {
    pub fn set_title(&mut self, cx: &mut Cx, title: &str) {
        self.title_text.as_mut_empty().push_str(title);
        self.view
            .label(cx, ids!(title))
            .set_text(cx, title);
    }

    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            // Title re-syncs on the next draw_walk.
            self.applied_size = None;
            self.redraw(cx);
        }
    }
}

impl MpGroupBoxRef {
    pub fn set_title(&self, cx: &mut Cx, title: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_title(cx, title);
        }
    }

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
