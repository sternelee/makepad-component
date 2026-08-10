use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp_theme.*

    // ============================================================
    // MpTextArea - Multi-line text input
    // Inspired by shadcn/ui Textarea + macOS multi-line text
    // ============================================================

    mod.widgets.MpTextAreaBase = #(MpTextArea::register_widget(vm))
    mod.widgets.MpTextArea = set_type_default() do mod.widgets.MpTextAreaBase{
        width: Fill
        height: Fit
        padding: Inset{left: 12.0, right: 12.0, top: 10.0, bottom: 10.0}



        draw_bg +: {
            bg_color: instance(INPUT)
            border_color: instance(BORDER)
            focus_color: instance(RING)
            has_focus: instance(0.0)
            border_width: instance(1.0)
            radius: instance(6.0)
            hover: instance(0.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(
                    self.border_width,
                    self.border_width,
                    self.rect_size.x - self.border_width * 2.0,
                    self.rect_size.y - self.border_width * 2.0,
                    self.radius
                )

                // Background
                sdf.fill(self.bg_color)

                // Border (with focus ring)
                let border_color = mix(self.border_color, self.focus_color, self.has_focus)
                let border_w = mix(self.border_width, 2.0, self.has_focus)
                sdf.stroke(border_color, border_w)

                return sdf.result
            }
        }

        placeholder: ""

        animator: Animator{
            focus: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_bg: {has_focus: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_bg: {has_focus: 1.0}}
                }
            }
            hover: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.1}}
                    apply: {draw_bg: {hover: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.1}}
                    apply: {draw_bg: {hover: 1.0}}
                }
            }
        }
    }

    // Variants
    mod.widgets.MpTextAreaSmall = mod.widgets.MpTextArea{
        padding: Inset{left: 10.0, right: 10.0, top: 6.0, bottom: 6.0}
        draw_text +: {
            text_style: theme.font_regular{font_size: 12.0}
        }
    }

    mod.widgets.MpTextAreaLarge = mod.widgets.MpTextArea{
        padding: Inset{left: 14.0, right: 14.0, top: 12.0, bottom: 12.0}
        draw_text +: {
            text_style: theme.font_regular{font_size: 15.0}
        }
    }
}

// ============================================================
// Rust Implementation
// ============================================================

#[derive(Script, ScriptHook, Widget, Animator)]
pub struct MpTextArea {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[apply_default]
    animator: Animator,

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
    #[live]
    placeholder: ArcStringMut,

    #[rust]
    area: Area,
}

#[derive(Clone, Debug, Default)]
pub enum MpTextAreaAction {
    Changed,
    #[default]
    None,
}

impl Widget for MpTextArea {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let _uid = self.widget_uid();

        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }

        match event.hits(cx, self.area) {
            Hit::FingerHoverIn(_) => {
                self.animator_play(cx, ids!(hover.on));
            }
            Hit::FingerHoverOut(_) => {
                self.animator_play(cx, ids!(hover.off));
            }
            Hit::FingerDown(_) => {
                self.animator_play(cx, ids!(focus.on));
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_bg.begin(cx, walk, self.layout);
        // Draw placeholder if no text
        let display_text = if self.text.as_ref().is_empty() {
            self.placeholder.as_ref()
        } else {
            self.text.as_ref()
        };
        self.draw_text
            .draw_walk(cx, Walk::fit(), Align::default(), display_text);
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        DrawStep::done()
    }
}

impl MpTextArea {
    pub fn set_text(&mut self, text: &str) {
        self.text.as_mut_empty().push_str(text);
    }

    pub fn set_placeholder(&mut self, text: &str) {
        self.placeholder.as_mut_empty().push_str(text);
    }
}

impl MpTextAreaRef {
    pub fn changed(&self, actions: &Actions) -> bool {
        if let Some(item) = actions.find_widget_action(self.widget_uid()) {
            matches!(item.cast(), MpTextAreaAction::Changed)
        } else {
            false
        }
    }

    pub fn set_text(&self, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(text);
        }
    }

    pub fn set_placeholder(&self, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_placeholder(text);
        }
    }
}
