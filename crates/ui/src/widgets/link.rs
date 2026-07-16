use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp_theme.*

    // ============================================================
    // MpLink - Text hyperlink component
    // Inspired by shadcn/ui Link + macOS hyperlink style
    // ============================================================

    mod.widgets.MpLinkBase = #(MpLink::register_widget(vm))
    mod.widgets.MpLink = set_type_default() do mod.widgets.MpLinkBase{
        width: Fit
        height: Fit
        draw_text +: {
            text_style: theme.font_regular{font_size: 13.0}
            color: #x3B82F6
            underline: instance(0.0)
            get_color: fn() {
                // Blend between normal and muted for disabled state
                return self.color
            }
        }

        text: ""
        disabled: false

        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_text: {underline: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_text: {underline: 1.0}}
                }
            }
            disabled: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.0}}
                    apply: {}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.0}}
                    apply: {}
                }
            }
        }
    }

    // Variant: Muted link (secondary color)
    mod.widgets.MpLinkMuted = mod.widgets.MpLink{
        draw_text +: {
            color: #x94A3B8
        }
    }

    // Variant: Ghost link (muted foreground, no primary color)
    mod.widgets.MpLinkGhost = mod.widgets.MpLink{
        draw_text +: {
            color: #x1D1D1F
        }
    }

    // Variant: Destructive link
    mod.widgets.MpLinkDestructive = mod.widgets.MpLink{
        draw_text +: {
            color: #xEF4444
        }
    }

    // Size variants
    mod.widgets.MpLinkSmall = mod.widgets.MpLink{
        draw_text +: {
            text_style: theme.font_regular{font_size: 11.0}
        }
    }

    mod.widgets.MpLinkLarge = mod.widgets.MpLink{
        draw_text +: {
            text_style: theme.font_regular{font_size: 15.0}
        }
    }
}

// ============================================================
// Rust Implementation
// ============================================================

#[derive(Script, ScriptHook, Widget, Animator)]
pub struct MpLink {
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
    disabled: bool,

    #[rust]
    area: Area,
}

#[derive(Clone, Debug, Default)]
pub enum MpLinkAction {
    Clicked,
    #[default]
    None,
}

impl Widget for MpLink {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let uid = self.widget_uid();

        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }

        if self.disabled {
            return;
        }

        match event.hits(cx, self.area) {
            Hit::FingerHoverIn(_) => {
                cx.set_cursor(MouseCursor::Hand);
                self.animator_play(cx, ids!(hover.on));
            }
            Hit::FingerHoverOut(_) => {
                self.animator_play(cx, ids!(hover.off));
            }
            Hit::FingerUp(fe) => {
                if fe.is_over {
                    cx.widget_action(uid, MpLinkAction::Clicked);
                }
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_text
            .draw_walk(cx, Walk::fit(), Align::default(), self.text.as_ref());
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        DrawStep::done()
    }
}

impl MpLink {
    pub fn clicked(&self, actions: &Actions) -> bool {
        if let Some(action) = actions.find_widget_action(self.widget_uid()) {
            matches!(action.cast(), MpLinkAction::Clicked)
        } else {
            false
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text.as_mut_empty().push_str(text);
    }
}

impl MpLinkRef {
    pub fn clicked(&self, actions: &Actions) -> bool {
        if let Some(inner) = self.borrow() {
            inner.clicked(actions)
        } else {
            false
        }
    }

    pub fn set_text(&self, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(text);
        }
    }
}
