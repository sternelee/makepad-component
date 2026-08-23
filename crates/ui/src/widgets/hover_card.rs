use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpHoverCard - Rich card that appears on hover
    // Inspired by shadcn/ui HoverCard
    // ============================================================

    // HoverCard trigger - wraps content that triggers the card
    mod.widgets.MpHoverCardTriggerBase = #(MpHoverCardTrigger::register_widget(vm))
    mod.widgets.MpHoverCardTrigger = set_type_default() do mod.widgets.MpHoverCardTriggerBase{
        width: Fit
        height: Fit
    }

    // HoverCard content - shown on hover
    mod.widgets.MpHoverCardContentBase = #(MpHoverCardContent::register_widget(vm))
    mod.widgets.MpHoverCardContent = set_type_default() do mod.widgets.MpHoverCardContentBase{
        width: Fit
        height: Fit


        padding: Inset{left: 12.0, right: 12.0, top: 12.0, bottom: 12.0}
        flow: Down
        spacing: 8.0
        visible: false

        draw_bg +: {
            bg_color: instance(SURFACE_CARD)
            border_color: instance(BORDER)
            radius: instance(8.0)
            hover: instance(0.0)
            popover_opacity: instance(1.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                sdf.fill_keep(self.bg_color)
                sdf.stroke(self.border_color, 1.0)
                return sdf.result
            }
        }

        animator: Animator{
            show: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    redraw: true
                    apply: {draw_bg: {popover_opacity: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    redraw: true
                    apply: {draw_bg: {popover_opacity: 1.0}}
                }
            }
        }
    }

    // HoverCard container (wraps trigger + content)
    mod.widgets.MpHoverCard = mod.widgets.View{
        width: Fit
        height: Fit
        flow: Overlay

        trigger := mod.widgets.MpHoverCardTrigger{}
        content := mod.widgets.MpHoverCardContentBase{}
    }
}

// ============================================================
// Rust Implementation
// ============================================================

#[derive(Script, ScriptHook, Widget)]
pub struct MpHoverCardTrigger {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
}

impl Widget for MpHoverCardTrigger {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

#[derive(Script, ScriptHook, Widget, Animator)]
pub struct MpHoverCardContent {
    #[source]
    source: ScriptObjectRef,
    #[apply_default]
    animator: Animator,
    #[deref]
    view: View,
}

impl Widget for MpHoverCardContent {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpHoverCardContent {
    pub fn show(&mut self, cx: &mut Cx) {
        self.view.set_visible(cx, true);
        self.animator_play(cx, ids!(show.on));
    }

    pub fn hide(&mut self, cx: &mut Cx) {
        self.animator_play(cx, ids!(show.off));
        self.view.set_visible(cx, false);
    }
}

impl MpHoverCardContentRef {
    pub fn show(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.show(cx);
        }
    }

    pub fn hide(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.hide(cx);
        }
    }
}
