use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

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
            bg_color: instance(INPUT_BG)
            border_color: instance(BORDER)
            focus_color: instance(ACCENT)
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

    // Variants (size system drives the metrics now)
    mod.widgets.MpTextAreaSmall = mod.widgets.MpTextArea{
        size: MpSize.Small
    }

    mod.widgets.MpTextAreaLarge = mod.widgets.MpTextArea{
        size: MpSize.Large
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
    size: MpSize,

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
        // Metrics per size (default Medium = 12/10 padding, 13px font)
        self.layout.padding = Inset {
            left: self.size.padding_h(),
            right: self.size.padding_h(),
            top: self.size.padding_v() + 4.0,
            bottom: self.size.padding_v() + 4.0,
        };
        self.draw_text.text_style.font_size = self.size.font_size();

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
    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            self.redraw(cx);
        }
    }

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

    pub fn size(&self) -> MpSize {
        self.borrow().map_or(MpSize::default(), |inner| inner.size())
    }

    pub fn set_size(&self, cx: &mut Cx, size: MpSize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_size(cx, size);
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
