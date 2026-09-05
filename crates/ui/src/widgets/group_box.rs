use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpGroupBox - styled container with an optional title that
    // groups related content together (gpui GroupBox port).
    // Variants: MpGroupBox (normal), MpGroupBoxFill, MpGroupBoxOutline.
    // ============================================================

    mod.widgets.MpGroupBoxBase = #(MpGroupBox::register_widget(vm))
    mod.widgets.MpGroupBox = set_type_default() do mod.widgets.MpGroupBoxBase{
        width: Fill
        height: Fit
        flow: Down
        spacing: 8.0

        title := Label{
            width: Fill, height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 13.0}
                color: TEXT_MUTED
            }
            text: ""
        }

        content := RoundedView{
            width: Fill, height: Fit
            flow: Down
            spacing: 16.0
            draw_bg +: {
                color: #x0000
                border_radius: 8.0
            }
        }
    }

    // Fill variant: muted card surface behind the content.
    mod.widgets.MpGroupBoxFill = mod.widgets.MpGroupBox{
        spacing: 4.0
        content := RoundedView{
            padding: Inset{left: 16, right: 16, top: 16, bottom: 16}
            draw_bg +: {
                color: SURFACE_CARD
                border_radius: 8.0
            }
        }
    }

    // Outline variant: 1px border around the content.
    mod.widgets.MpGroupBoxOutline = mod.widgets.MpGroupBox{
        spacing: 4.0
        content := RoundedView{
            padding: Inset{left: 16, right: 16, top: 16, bottom: 16}
            draw_bg +: {
                bg_color: instance(#x0000)
                border_color: instance(BORDER)
                border_width: instance(1.0)
                radius: instance(8.0)

                pixel: fn() {
                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                    sdf.box(self.border_width, self.border_width, self.rect_size.x - self.border_width*2.0, self.rect_size.y - self.border_width*2.0, self.radius)
                    sdf.fill(self.bg_color)
                    sdf.stroke(self.border_color, self.border_width)
                    return sdf.result
                }
            }
        }
    }
}

/// Group box actions
#[derive(Clone, Debug, Default)]
pub enum MpGroupBoxAction {
    #[default]
    None,
}

/// Rust handle for the group box: owns the title text and its visibility.
#[derive(Script, ScriptHook, Widget)]
pub struct MpGroupBox {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
}

impl Widget for MpGroupBox {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpGroupBox {
    /// Set the group title; an empty title hides the label row entirely.
    pub fn set_title(&mut self, cx: &mut Cx, title: &str) {
        self.view.label(cx, ids!(title)).set_text(cx, title);
        self.view
            .widget(cx, ids!(title))
            .set_visible(cx, !title.is_empty());
    }
}

impl MpGroupBoxRef {
    pub fn set_title(&self, cx: &mut Cx, title: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_title(cx, title);
        }
    }
}
