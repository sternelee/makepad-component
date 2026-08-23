use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpAccordion - Collapsible panel component
    // ============================================================

    // Accordion item header. The chevron is drawn by the header's own
    // draw_bg shader so the animator can drive its `rotation` instance
    // directly (animator apply paths only reach the widget's own fields).
    mod.widgets.MpAccordionHeaderBase = #(MpAccordionHeader::register_widget(vm))
    mod.widgets.MpAccordionHeader = set_type_default() do mod.widgets.MpAccordionHeaderBase{
        width: Fill
        height: Fit
        padding: Inset{left: 16.0, right: 16.0, top: 12.0, bottom: 12.0}
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
            icon_color: instance(TEXT_MUTED)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)

                // Background
                sdf.rect(0.0, 0.0, self.rect_size.x, self.rect_size.y)
                sdf.fill(mix(self.bg_color, self.bg_color_hover, self.hover))

                // Chevron (down arrow), centered where the old 20x20 icon
                // child sat: 16.0 right padding + 10.0 half icon width.
                let c = vec2(self.rect_size.x - 26.0, self.rect_size.y * 0.5)
                sdf.rotate(self.rotation, c.x, c.y)

                let size = 6.0
                sdf.move_to(c.x - size, c.y - size * 0.5)
                sdf.line_to(c.x, c.y + size * 0.5)
                sdf.line_to(c.x + size, c.y - size * 0.5)
                sdf.stroke(self.icon_color, 1.5)

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
                    // pi: chevron points up when expanded
                    apply: {draw_bg: {rotation: 3.14159265}}
                }
            }
        }

        // Title label. Right margin keeps the text clear of the chevron
        // (old icon child: 20.0 wide + 8.0 spacing).
        label := Label{
            width: Fill
            height: Fit
            margin: Inset{right: 28.0}
            draw_text +: {
                text_style: theme.font_bold{font_size: 14.0}
                color: TEXT
            }
            text: "Accordion Title"
        }
    }

    // Accordion content wrapper
    mod.widgets.MpAccordionContentBase = mod.widgets.View{
        width: Fill
        height: Fit
        padding: Inset{left: 16.0, right: 16.0, top: 0.0, bottom: 12.0}
        flow: Down
    }

    // ============================================================
    // Accordion Item (header + content)
    // ============================================================

    mod.widgets.MpAccordionItem = mod.widgets.SolidView{
        width: Fill
        height: Fit
        flow: Down

        draw_bg +: {
            color: SURFACE_CARD
        }

        header := mod.widgets.MpAccordionHeader{}
        body := mod.widgets.MpAccordionContentBase{}
    }

    // ============================================================
    // Accordion Container
    // ============================================================

    mod.widgets.MpAccordion = mod.widgets.RoundedView{
        width: Fill
        height: Fit
        flow: Down

        draw_bg +: {
            color: SURFACE_CARD
            border_radius: 8.0
            border_color: BORDER
        }
    }

    // ============================================================
    // Variants
    // ============================================================

    // Bordered accordion (each item has border)
    mod.widgets.MpAccordionBordered = mod.widgets.View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 8.0
    }

    mod.widgets.MpAccordionItemBordered = mod.widgets.RoundedView{
        width: Fill
        height: Fit
        flow: Down

        draw_bg +: {
            color: SURFACE_CARD
            border_radius: 6.0
            border_color: BORDER
        }

        header := mod.widgets.MpAccordionHeader{}
        body := mod.widgets.MpAccordionContentBase{}
    }

    // Ghost accordion (no background)
    mod.widgets.MpAccordionGhost = mod.widgets.View{
        width: Fill
        height: Fit
        flow: Down
    }

    mod.widgets.MpAccordionItemGhost = mod.widgets.View{
        width: Fill
        height: Fit
        flow: Down

        header := mod.widgets.MpAccordionHeader{
            draw_bg +: {
                bg_color: #x00000000
                bg_color_hover: ELEMENT_HOVER
            }
        }
        body := mod.widgets.MpAccordionContentBase{}
    }

    // Divider between items
    mod.widgets.MpAccordionDivider = mod.widgets.SolidView{
        width: Fill
        height: 1.0
        draw_bg +: {
            color: BORDER
        }
    }
}

// ============================================================
// Rust Implementation
// ============================================================

#[derive(Script, Widget, Animator)]
pub struct MpAccordionHeader {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[apply_default]
    animator: Animator,

    #[live]
    expanded: bool,
}

impl ScriptHook for MpAccordionHeader {
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
pub enum MpAccordionAction {
    Toggle,
    #[default]
    None,
}

impl Widget for MpAccordionHeader {
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
                cx.widget_action(self.widget_uid(), MpAccordionAction::Toggle);
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpAccordionHeader {
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

impl MpAccordionHeaderRef {
    pub fn toggled(&self, actions: &Actions) -> bool {
        if let Some(item) = actions.find_widget_action(self.widget_uid()) {
            matches!(item.cast(), MpAccordionAction::Toggle)
        } else {
            false
        }
    }
}
