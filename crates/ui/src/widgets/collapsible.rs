use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpCollapsible - Expand/collapse content container
    // Inspired by shadcn/ui Collapsible + macOS disclosure triangle
    // ============================================================

    // Collapsible trigger header
    mod.widgets.MpCollapsibleTriggerBase = #(MpCollapsibleTrigger::register_widget(vm))
    mod.widgets.MpCollapsibleTrigger = set_type_default() do mod.widgets.MpCollapsibleTriggerBase{
        width: Fill
        height: Fit
        padding: Inset{left: 8.0, right: 8.0, top: 6.0, bottom: 6.0}
        flow: Right
        align: Align{y: 0.5}
        spacing: 8.0
        cursor: MouseCursor.Hand

        show_bg: true
        draw_bg +: {
            bg_color: instance(#x00000000)
            bg_color_hover: instance(ELEMENT_HOVER)
            hover: instance(0.0)
            rotation: instance(0.0)
            disclosure_color: instance(#x64748b)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)

                // Background
                sdf.rect(0.0, 0.0, self.rect_size.x, self.rect_size.y)
                let bg = mix(self.bg_color, self.bg_color_hover, self.hover)
                sdf.fill(bg)

                // Disclosure triangle (chevron), left-aligned with 8px indent
                let c = vec2(16.0, self.rect_size.y * 0.5)

                // Rotate around center
                sdf.rotate(self.rotation, c.x, c.y)

                let size = 5.0
                sdf.move_to(c.x - size, c.y - size * 0.5)
                sdf.line_to(c.x, c.y + size * 0.5)
                sdf.line_to(c.x + size, c.y - size * 0.5)
                sdf.stroke(self.disclosure_color, 1.5)

                return sdf.result
            }
        }

        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_bg: {hover: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.1}}
                    apply: {draw_bg: {hover: 1.0}}
                }
            }
            expanded: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.2}}
                    redraw: true
                    apply: {draw_bg: {rotation: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.2}}
                    redraw: true
                    apply: {draw_bg: {rotation: 1.57079633}} // pi/2: chevron points down->right
                }
            }
        }

        label := Label{
            width: Fit
            height: Fit
            margin: Inset{left: 14.0}
            draw_text +: {
                text_style: theme.font_bold{font_size: 13.0}
                color: TEXT
            }
            text: "Collapsible"
        }
    }

    // Collapsible content wrapper
    mod.widgets.MpCollapsibleContent = mod.widgets.View{
        width: Fill
        height: Fit
        padding: Inset{left: 24.0, right: 8.0, top: 4.0, bottom: 4.0}
        flow: Down
    }
}

// ============================================================
// Rust Implementation
// ============================================================

#[derive(Script, Widget, Animator)]
pub struct MpCollapsibleTrigger {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[apply_default]
    animator: Animator,

    #[live]
    expanded: bool,
}

impl ScriptHook for MpCollapsibleTrigger {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        vm.with_cx_mut(|cx| {
            let expanded = self.expanded;
            self.animator_toggle(
                cx,
                expanded,
                Animate::No,
                ids!(expanded.on),
                ids!(expanded.off),
            );
        });
    }
}

#[derive(Clone, Debug, Default)]
pub enum MpCollapsibleAction {
    Toggle,
    #[default]
    None,
}

impl Widget for MpCollapsibleTrigger {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }

        match event.hits(cx, self.view.area()) {
            Hit::FingerHoverIn(_) => {
                cx.set_cursor(MouseCursor::Hand);
                self.animator_play(cx, ids!(hover.on));
            }
            Hit::FingerHoverOut(_) => {
                self.animator_play(cx, ids!(hover.off));
            }
            Hit::FingerDown(_) => {
                self.expanded = !self.expanded;
                self.animator_toggle(
                    cx,
                    self.expanded,
                    Animate::Yes,
                    ids!(expanded.on),
                    ids!(expanded.off),
                );
                cx.widget_action(self.widget_uid(), MpCollapsibleAction::Toggle);
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpCollapsibleTrigger {
    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    pub fn set_expanded(&mut self, cx: &mut Cx, expanded: bool) {
        if self.expanded != expanded {
            self.expanded = expanded;
            self.animator_toggle(
                cx,
                expanded,
                Animate::Yes,
                ids!(expanded.on),
                ids!(expanded.off),
            );
            self.redraw(cx);
        }
    }
}

impl MpCollapsibleTriggerRef {
    pub fn toggled(&self, actions: &Actions) -> bool {
        if let Some(item) = actions.find_widget_action(self.widget_uid()) {
            matches!(item.cast(), MpCollapsibleAction::Toggle)
        } else {
            false
        }
    }

    pub fn is_expanded(&self) -> bool {
        if let Some(inner) = self.borrow() {
            inner.expanded
        } else {
            false
        }
    }

    pub fn set_expanded(&self, cx: &mut Cx, expanded: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_expanded(cx, expanded);
        }
    }
}
