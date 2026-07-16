use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp_theme.*

    // ============================================================
    // MpSheet - Side panel that slides in from the edge
    // Inspired by shadcn/ui Sheet + macOS side panels
    // ============================================================

    // Sheet trigger button
    mod.widgets.MpSheetTriggerBase = #(MpSheetTrigger::register_widget(vm))
    mod.widgets.MpSheetTrigger = set_type_default() do mod.widgets.MpSheetTriggerBase{
        width: Fit
        height: Fit
    }

    // Sheet container (modal overlay root)
    mod.widgets.MpSheetBase = #(MpSheet::register_widget(vm))
    mod.widgets.MpSheet = set_type_default() do mod.widgets.MpSheetBase{
        width: Fill
        height: Fill
        visible: false

        // Overlay backdrop
        overlay := SolidView{
            width: Fill
            height: Fill
            visible: false
            draw_bg +: {
                color: #x00000000
                overlay_opacity: instance(0.0)
                pixel: fn() {
                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                    sdf.rect(0.0, 0.0, self.rect_size.x, self.rect_size.y)
                    // Semi-transparent black overlay
                    sdf.fill(#x00000080 * self.overlay_opacity)
                    return sdf.result
                }
            }
        }

        // Content panel (slides in from right)
        content := RoundedView{
            width: 400
            height: Fill
            align: Align{x: 0.0, y: 0.0}
            flow: Down
            visible: false
            draw_bg +: {
                color: CARD
                border_radius: 0.0
            }

            // Header
            header := View{
                width: Fill
                height: Fit
                padding: Inset{left: 24.0, right: 24.0, top: 20.0, bottom: 16.0}
                flow: Right
                align: Align{y: 0.5}

                title := Label{
                    width: Fill
                    height: Fit
                    draw_text +: {
                        text_style: theme.font_bold{font_size: 18.0}
                        color: FOREGROUND
                    }
                    text: ""
                }

                close_btn := View{
                    width: 32
                    height: 32
                    cursor: MouseCursor.Hand
                    draw_bg +: {
                        bg_color: instance(#x00000000)
                        bg_hover: instance(#xf1f5f9)
                        hover: instance(0.0)
                        pixel: fn() {
                            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                            sdf.rect(0.0, 0.0, self.rect_size.x, self.rect_size.y)
                            sdf.fill(mix(self.bg_color, self.bg_hover, self.hover))

                            // X icon
                            let c = self.rect_size * 0.5
                            let s = 6.0
                            sdf.move_to(c.x - s, c.y - s)
                            sdf.line_to(c.x + s, c.y + s)
                            sdf.move_to(c.x + s, c.y - s)
                            sdf.line_to(c.x - s, c.y + s)
                            sdf.stroke(#x64748b, 1.5)
                            return sdf.result
                        }
                    }
                }
            }

            separator := SolidView{
                width: Fill
                height: 1
                draw_bg +: { color: BORDER }
            }

            // Body content area
            body := View{
                width: Fill
                height: Fill
                padding: Inset{left: 24.0, right: 24.0, top: 16.0, bottom: 24.0}
                flow: Down
            }
        }

        animator: Animator{
            open: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.0}}
                    apply: {
                        overlay: {overlay_opacity: 0.0}
                    }
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.2}}
                    redraw: true
                    apply: {
                        overlay: {overlay_opacity: 1.0}
                    }
                }
            }
        }
    }
}

// ============================================================
// Rust Implementation
// ============================================================

#[derive(Clone, Debug, Default)]
pub enum MpSheetAction {
    Open,
    Close,
    #[default]
    None,
}

// MpSheetTrigger - clickable button that emits Open action
#[derive(Script, ScriptHook, Widget)]
pub struct MpSheetTrigger {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
}

impl Widget for MpSheetTrigger {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        match event.hits(cx, self.view.area()) {
            Hit::FingerHoverIn(_) => {
                cx.set_cursor(MouseCursor::Hand);
            }
            Hit::FingerUp(fe) => {
                if fe.is_over {
                    cx.widget_action(self.widget_uid(), MpSheetAction::Open);
                }
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpSheetTriggerRef {
    pub fn opened(&self, actions: &Actions) -> bool {
        if let Some(item) = actions.find_widget_action(self.widget_uid()) {
            matches!(item.cast(), MpSheetAction::Open)
        } else {
            false
        }
    }
}

// MpSheet - manages overlay + content visibility
#[derive(Script, Widget, Animator)]
pub struct MpSheet {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[apply_default]
    animator: Animator,

    #[live]
    open: bool,
}

impl ScriptHook for MpSheet {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        vm.with_cx_mut(|cx| {
            let open = self.open;
            self.animator_toggle(
                cx,
                open,
                Animate::No,
                ids!(open.on),
                ids!(open.off),
            );
        });
    }
}

impl Widget for MpSheet {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }

        // Close on overlay click
        if let Hit::FingerUp(fe) = event.hits(cx, self.view.widget(cx, ids!(overlay)).area()) {
            if fe.is_over && self.open {
                self.set_open(cx, false);
                cx.widget_action(self.widget_uid(), MpSheetAction::Close);
            }
        }

        // Close on X button click
        if let Hit::FingerUp(fe) = event.hits(cx, self.view.widget(cx, ids!(close_btn)).area()) {
            if fe.is_over && self.open {
                self.set_open(cx, false);
                cx.widget_action(self.widget_uid(), MpSheetAction::Close);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Show/hide overlay and content based on open state
        if self.open {
            self.view.widget(cx, ids!(overlay)).set_visible(cx, true);
            self.view.widget(cx, ids!(content)).set_visible(cx, true);
        }
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpSheet {
    pub fn set_open(&mut self, cx: &mut Cx, open: bool) {
        if self.open != open {
            self.open = open;
            self.animator_toggle(
                cx,
                open,
                Animate::Yes,
                ids!(open.on),
                ids!(open.off),
            );
            if open {
                self.view.widget(cx, ids!(overlay)).set_visible(cx, true);
                self.view.widget(cx, ids!(content)).set_visible(cx, true);
            }
            self.redraw(cx);
        }
    }

    pub fn set_title(&mut self, cx: &mut Cx, title: &str) {
        self.view.widget(cx, ids!(title)).set_text(cx, title);
    }
}

impl MpSheetRef {
    pub fn closed(&self, actions: &Actions) -> bool {
        if let Some(item) = actions.find_widget_action(self.widget_uid()) {
            matches!(item.cast(), MpSheetAction::Close)
        } else {
            false
        }
    }

    pub fn set_open(&self, cx: &mut Cx, open: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_open(cx, open);
        }
    }

    pub fn set_title(&self, cx: &mut Cx, title: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_title(cx, title);
        }
    }
}
