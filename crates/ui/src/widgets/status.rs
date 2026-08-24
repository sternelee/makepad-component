use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpStepRow - one operation as a row: icon, title, detail, meta
    // ============================================================

    mod.widgets.MpStepRow = set_type_default() do #(MpStepRow::register_widget(vm)){
        width: Fill
        height: Fit
        flow: Right
        spacing: 8.0
        align: Align{x: 0.0, y: 0.5}
        padding: Inset{left: 10.0, right: 10.0, top: 6.0, bottom: 6.0}

        cursor: MouseCursor.Default

        icon_label := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 14.0}
                color: TEXT_MUTED
            }
            text: "○"
        }

        title_label := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 12.5}
                color: TEXT
            }
            text: "Step"
        }

        detail_label := Label{
            width: Fill
            height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 12.0}
                color: TEXT_MUTED
            }
            text: ""
        }

        meta_label := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 12.0}
                color: TEXT_MUTED
            }
            text: ""
        }
    }

    // ============================================================
    // MpStepOutput - monospace-ish output box under a step row
    // ============================================================

    mod.widgets.MpStepOutput = View{
        width: Fill
        height: Fit
        flow: Down
        padding: Inset{left: 10.0, right: 10.0, top: 8.0, bottom: 6.0}

        show_bg: true
        draw_bg +: {
            border_color: instance(BORDER)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.rect(0.0, 1.0, self.rect_size.x, self.rect_size.y - 1.0)
                sdf.fill(#x0000)
                sdf.rect(0.0, 0.0, self.rect_size.x, 1.0)
                sdf.fill(self.border_color)
                return sdf.result
            }
        }

        output_label := Label{
            width: Fill
            height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 12.0}
                color: TEXT_MUTED
            }
            text: ""
        }
    }

    // ============================================================
    // MpErrorStrip - dismissible-style red alert strip
    // ============================================================

    mod.widgets.MpErrorStrip = View{
        width: Fill
        height: Fit
        flow: Right
        spacing: 8.0
        align: Align{x: 0.0, y: 0.0}
        padding: Inset{left: 16.0, right: 16.0, top: 12.0, bottom: 12.0}

        show_bg: true
        draw_bg +: {
            accent_color: instance(DANGER)
            radius: instance(12.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let c = self.accent_color
                let bg = vec4(c.rgb, c.a * 0.06)
                let bc = vec4(c.rgb, c.a * 0.2)
                sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                sdf.fill(bg)
                sdf.stroke(bc, 1.0)
                return sdf.result
            }
        }

        strip_icon := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 14.0}
                color: DANGER_MUTED
            }
            text: "▲"
        }

        message_label := Label{
            width: Fill
            height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 12.5}
                color: DANGER_MUTED
            }
            text: ""
        }
    }

    // ============================================================
    // MpWarningStrip - amber warning strip
    // ============================================================

    mod.widgets.MpWarningStrip = View{
        width: Fill
        height: Fit
        flow: Right
        spacing: 8.0
        align: Align{x: 0.0, y: 0.0}
        padding: Inset{left: 16.0, right: 16.0, top: 10.0, bottom: 10.0}

        show_bg: true
        draw_bg +: {
            accent_color: instance(WARNING)
            radius: instance(12.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let c = self.accent_color
                let bg = vec4(c.rgb, c.a * 0.06)
                let bc = vec4(c.rgb, c.a * 0.2)
                sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                sdf.fill(bg)
                sdf.stroke(bc, 1.0)
                return sdf.result
            }
        }

        strip_icon := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 13.0}
                color: WARNING_MUTED
            }
            text: "▲"
        }

        message_label := Label{
            width: Fill
            height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 12.0}
                color: WARNING_MUTED
            }
            text: ""
        }
    }
}

// ============================================================
// MpStepRow - one operation as a row
// ============================================================

#[derive(Script, ScriptHook, Widget)]
pub struct MpStepRow {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
}

impl Widget for MpStepRow {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpStepRow {
    pub fn set_icon(&mut self, cx: &mut Cx, icon: &str) {
        self.view.label(cx, ids!(icon_label)).set_text(cx, icon);
    }

    pub fn set_title(&mut self, cx: &mut Cx, title: &str) {
        self.view.label(cx, ids!(title_label)).set_text(cx, title);
    }

    pub fn set_detail(&mut self, cx: &mut Cx, detail: &str) {
        self.view.label(cx, ids!(detail_label)).set_text(cx, detail);
    }

    pub fn set_meta(&mut self, cx: &mut Cx, meta: &str) {
        self.view.label(cx, ids!(meta_label)).set_text(cx, meta);
    }

    pub fn set_failed(&mut self, cx: &mut Cx, failed: bool) {
        let _ = failed;
        self.redraw(cx);
    }
}

impl MpStepRowRef {
    pub fn set_step(
        &self,
        cx: &mut Cx,
        icon: &str,
        title: &str,
        detail: &str,
        meta: &str,
    ) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_icon(cx, icon);
            inner.set_title(cx, title);
            inner.set_detail(cx, detail);
            inner.set_meta(cx, meta);
        }
    }
}

// MpStepOutput / MpErrorStrip / MpWarningStrip are style-only View
// classes — set their child labels via `.label(cx, ids!(output_label))`
// / `.label(cx, ids!(message_label))` lookups from the consumer side.
