use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpDialogInner - Dialog card (defined FIRST so MpDialog can use it)
    // ============================================================
    mod.widgets.MpDialogInner = View{
        width: 420
        height: Fit
        flow: Down

        draw_bg +: {
            bg_color: (SURFACE_CARD)
            radius: instance(12.0)
            border_color: (BORDER)
            shadow_color: #x00000033
            shadow_offset_y: instance(8.0)
            shadow_blur: instance(24.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0.0, self.shadow_offset_y, self.rect_size.x, self.rect_size.y, self.radius)
                sdf.blur = self.shadow_blur
                sdf.fill(self.shadow_color)
                sdf.blur = 0.0
                sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                sdf.fill_keep(self.bg_color)
                sdf.stroke(self.border_color, 1.0)
                return sdf.result
            }
        }

        header := View{
            width: Fill, height: Fit,
            padding: Inset{left: 24, right: 24, top: 24, bottom: 8},
            flow: Down, spacing: 4,
            title := Label{
                width: Fill, height: Fit,
                draw_text +: { text_style: theme.font_bold{font_size: 18.0} color: TEXT }
                text: ""
            }
            description := Label{
                width: Fill, height: Fit, visible: false,
                draw_text +: { text_style: theme.font_regular{font_size: 14.0} color: TEXT_MUTED }
                text: ""
            }
        }

        body := View{
            width: Fill, height: Fit,
            padding: Inset{left: 24, right: 24, top: 8, bottom: 8},
            flow: Down, spacing: 8,
        }

        footer := View{
            width: Fill, height: Fit,
            padding: Inset{left: 24, right: 24, top: 16, bottom: 24},
            flow: Right, spacing: 8,
            align: Align{x: 1.0, y: 0.5},
        }
    }

    // ============================================================
    // MpDialog - Full-screen animated modal dialog
    // ============================================================
    mod.widgets.MpDialogBase = #(MpDialog::register_widget(vm))
    mod.widgets.MpDialog = set_type_default() do mod.widgets.MpDialogBase{
        width: Fill
        height: Fill
        flow: Overlay
        visible: false

        backdrop := View{
            width: Fill, height: Fill, visible: true,
            draw_bg +: {
                bg_color: #x000000
                opacity: instance(0.0)
                pixel: fn() { return self.bg_color * self.opacity }
            }
        }

        content := View{
            width: Fill, height: Fill,
            align: Align{x: 0.5, y: 0.5},
            dialog := mod.widgets.MpDialogInner{}
        }

        animator: Animator{
            show: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.0}}
                    redraw: true
                    apply: { backdrop: {opacity: 0.0} }
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.2}}
                    redraw: true
                    apply: { backdrop: {opacity: 0.8} }
                }
            }
        }
    }

    // ============================================================
    // Size variants
    // ============================================================
    mod.widgets.MpDialogSmall = mod.widgets.MpDialogInner{ width: 320 }
    mod.widgets.MpDialogLarge = mod.widgets.MpDialogInner{ width: 560 }

    // ============================================================
    // AlertDialog (confirm/cancel)
    // ============================================================
    mod.widgets.MpAlertDialogNew = mod.widgets.MpDialogInner{
        width: 360
        header := View{
            width: Fill, height: Fit,
            padding: Inset{left: 24, right: 24, top: 24, bottom: 8},
            align: Align{x: 0.5},
            title := Label{
                width: Fit, height: Fit,
                draw_text +: { text_style: theme.font_bold{font_size: 18.0} color: TEXT }
                text: ""
            }
        }
        body := View{
            width: Fill, height: Fit,
            padding: Inset{left: 24, right: 24, top: 8, bottom: 20},
            align: Align{x: 0.5},
            description := Label{
                width: Fit, height: Fit,
                draw_text +: { text_style: theme.font_regular{font_size: 14.0} color: TEXT_MUTED }
                text: ""
            }
        }
        footer := View{
            width: Fill, height: Fit,
            padding: Inset{left: 24, right: 24, top: 0, bottom: 24},
            flow: Right, spacing: 12,
            align: Align{x: 0.5},
        }
    }
}

// ============================================================
// Rust Implementation
// ============================================================

#[derive(Clone, Debug, Default)]
pub enum MpDialogAction {
    Open,
    Close,
    #[default]
    None,
}

#[derive(Script, Widget, Animator)]
pub struct MpDialog {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[apply_default]
    animator: Animator,
    #[live]
    open: bool,
}

impl ScriptHook for MpDialog {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        vm.with_cx_mut(|cx| {
            if self.open {
                self.animator_play(cx, ids!(show.on));
            }
        });
    }
}

impl Widget for MpDialog {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        if !self.open {
            return;
        }
        self.view.handle_event(cx, event, _scope);
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
        let uid = self.widget_uid();
        if let Hit::FingerUp(fe) = event.hits(cx, self.view.view(cx, ids!(backdrop)).area()) {
            if fe.is_over {
                cx.widget_action(uid, MpDialogAction::Close);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if !self.open {
            return DrawStep::done();
        }
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpDialog {
    pub fn open(&mut self, cx: &mut Cx) {
        if !self.open {
            self.open = true;
            self.animator_play(cx, ids!(show.on));
            self.redraw(cx);
        }
    }
    pub fn close(&mut self, cx: &mut Cx) {
        if self.open {
            self.open = false;
            self.redraw(cx);
        }
    }
    pub fn set_open(&mut self, cx: &mut Cx, open: bool) {
        if open {
            self.open(cx);
        } else {
            self.close(cx);
        }
    }
    pub fn is_open(&self) -> bool {
        self.open
    }
    pub fn set_title(&mut self, cx: &mut Cx, title: &str) {
        self.view
            .label(cx, ids!(content.dialog.header.title))
            .set_text(cx, title);
    }
    pub fn set_description(&mut self, cx: &mut Cx, desc: &str) {
        let d = self.view.label(cx, ids!(content.dialog.header.description));
        d.set_text(cx, desc);
        self.view
            .view(cx, ids!(content.dialog.header.description))
            .set_visible(cx, !desc.is_empty());
    }
}

impl MpDialogRef {
    pub fn open(&self, cx: &mut Cx) {
        if let Some(mut i) = self.borrow_mut() {
            i.open(cx);
        }
    }
    pub fn close(&self, cx: &mut Cx) {
        if let Some(mut i) = self.borrow_mut() {
            i.close(cx);
        }
    }
    pub fn set_open(&self, cx: &mut Cx, open: bool) {
        if let Some(mut i) = self.borrow_mut() {
            i.set_open(cx, open);
        }
    }
    pub fn is_open(&self) -> bool {
        if let Some(i) = self.borrow() {
            i.is_open()
        } else {
            false
        }
    }
    pub fn set_title(&self, cx: &mut Cx, title: &str) {
        if let Some(mut i) = self.borrow_mut() {
            i.set_title(cx, title);
        }
    }
    pub fn set_description(&self, cx: &mut Cx, desc: &str) {
        if let Some(mut i) = self.borrow_mut() {
            i.set_description(cx, desc);
        }
    }
    pub fn dialog_closed(&self, actions: &Actions) -> bool {
        if let Some(inner) = self.borrow() {
            if let Some(item) = actions.find_widget_action(inner.widget_uid()) {
                return matches!(item.cast::<MpDialogAction>(), MpDialogAction::Close);
            }
        }
        false
    }
}
