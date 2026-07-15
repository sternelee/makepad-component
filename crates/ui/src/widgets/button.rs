use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp_theme.*

    // Base button component - macOS style
    mod.widgets.MpButtonBase = #(MpButton::register_widget(vm))
    mod.widgets.MpButton = set_type_default() do mod.widgets.MpButtonBase{
        width: Fit
        height: Fit
        align: Align{x: 0.5, y: 0.5}
        padding: Inset{left: 16.0, right: 16.0, top: 8.0, bottom: 8.0}

        draw_bg +: {
            radius: instance(6.0)
            border_width: instance(0.0)
            border_color: instance(#x0000)
            hover: instance(0.0)
            pressed: instance(0.0)
            disabled: instance(0.0)
            color: instance(PRIMARY)
            color_hover: instance(PRIMARY_HOVER)
            color_pressed: instance(PRIMARY_ACTIVE)
            color_disabled: instance(#x8f9bb3)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(
                    self.border_width
                    self.border_width
                    self.rect_size.x - self.border_width * 2.0
                    self.rect_size.y - self.border_width * 2.0
                    max(1.0, self.radius)
                )

                // Subtle hover: slightly brighten
                let hover_color = mix(self.color, self.color_hover, self.hover * 0.6)
                // Pressed: darken more noticeably
                let pressed_color = mix(hover_color, self.color_pressed, self.pressed * 0.8)
                let final_color = mix(pressed_color, self.color_disabled, self.disabled)

                sdf.fill_keep(final_color)

                if (self.border_width > 0.0) {
                    sdf.stroke(self.border_color, self.border_width)
                }

                return sdf.result
            }
        }

        draw_text +: {
            text_style: theme.font_regular{font_size: 13.0}
            color: PRIMARY_FOREGROUND
            disabled: instance(0.0)
            color_disabled: instance(#xe6e9ef)
            get_color: fn() {
                return mix(self.color, self.color_disabled, self.disabled)
            }
        }

        text: ""

        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_bg: {hover: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_bg: {hover: 1.0}}
                }
            }
            pressed: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.08}}
                    apply: {draw_bg: {pressed: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.08}}
                    apply: {draw_bg: {pressed: 1.0}}
                }
            }
            disabled: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.0}}
                    apply: {draw_bg: {disabled: 0.0} draw_text: {disabled: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.0}}
                    apply: {draw_bg: {disabled: 1.0} draw_text: {disabled: 1.0}}
                }
            }
        }
    }

    // Variant: Primary Button (macOS system blue)
    mod.widgets.MpButtonPrimary = mod.widgets.MpButton{
        draw_bg +: {
            color: instance(PRIMARY)
            color_hover: instance(PRIMARY_HOVER)
            color_pressed: instance(PRIMARY_ACTIVE)
        }
        draw_text +: {
            color: PRIMARY_FOREGROUND
        }
    }

    // Variant: Secondary Button - macOS style (lighter, subtle)
    mod.widgets.MpButtonSecondary = mod.widgets.MpButton{
        padding: Inset{left: 14.0, right: 14.0, top: 7.0, bottom: 7.0}
        draw_bg +: {
            color: instance(SECONDARY)
            color_hover: instance(SECONDARY_HOVER)
            color_pressed: instance(SECONDARY_ACTIVE)
            border_width: instance(0.0)
        }
        draw_text +: {
            text_style: theme.font_regular{font_size: 13.0}
            color: SECONDARY_FOREGROUND
        }
    }

    // Variant: Danger Button (macOS system red)
    mod.widgets.MpButtonDanger = mod.widgets.MpButton{
        draw_bg +: {
            color: instance(DANGER)
            color_hover: instance(DANGER_HOVER)
            color_pressed: instance(DANGER_ACTIVE)
        }
        draw_text +: {
            color: DANGER_FOREGROUND
        }
    }

    // Variant: Success Button (macOS system green)
    mod.widgets.MpButtonSuccess = mod.widgets.MpButton{
        draw_bg +: {
            color: instance(SUCCESS)
            color_hover: instance(SUCCESS_HOVER)
            color_pressed: instance(SUCCESS_ACTIVE)
        }
        draw_text +: {
            color: SUCCESS_FOREGROUND
        }
    }

    // Variant: Warning Button (macOS system orange)
    mod.widgets.MpButtonWarning = mod.widgets.MpButton{
        draw_bg +: {
            color: instance(WARNING)
            color_hover: instance(WARNING_HOVER)
            color_pressed: instance(WARNING_ACTIVE)
        }
        draw_text +: {
            color: WARNING_FOREGROUND
        }
    }

    // Variant: Ghost Button - macOS style (transparent with hover fill)
    mod.widgets.MpButtonGhost = mod.widgets.MpButton{
        draw_bg +: {
            color: instance(TRANSPARENT)
            color_hover: instance(SECONDARY)
            color_pressed: instance(SECONDARY_ACTIVE)
        }
        draw_text +: {
            text_style: theme.font_regular{font_size: 13.0}
            color: PRIMARY
        }
    }

    // Variant: Outline Button - macOS style
    mod.widgets.MpButtonOutline = mod.widgets.MpButton{
        padding: Inset{left: 14.0, right: 14.0, top: 7.0, bottom: 7.0}
        draw_bg +: {
            color: instance(TRANSPARENT)
            color_hover: instance(SECONDARY)
            color_pressed: instance(SECONDARY_ACTIVE)
            border_width: instance(1.0)
            border_color: instance(BORDER)
        }
        draw_text +: {
            text_style: theme.font_regular{font_size: 13.0}
            color: FOREGROUND
        }
    }

    // Size: Small - macOS compact size
    mod.widgets.MpButtonSmall = mod.widgets.MpButton{
        padding: Inset{left: 12.0, right: 12.0, top: 4.0, bottom: 4.0}
        draw_text +: {
            text_style: theme.font_regular{font_size: 11.0}
        }
    }

    // Size: Large
    mod.widgets.MpButtonLarge = mod.widgets.MpButton{
        padding: Inset{left: 24.0, right: 24.0, top: 10.0, bottom: 10.0}
        draw_text +: {
            text_style: theme.font_regular{font_size: 15.0}
        }
    }
}

// Rust implementation
#[derive(Script, Widget, Animator)]
pub struct MpButton {
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

impl ScriptHook for MpButton {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        vm.with_cx_mut(|cx| {
            let disabled = self.disabled;
            self.animator_toggle(
                cx,
                disabled,
                Animate::No,
                ids!(disabled.on),
                ids!(disabled.off),
            );
        });
    }
}

// Custom Actions
#[derive(Clone, Debug, Default)]
pub enum MpButtonAction {
    Clicked,
    Pressed,
    Released,
    #[default]
    None,
}

impl Widget for MpButton {
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
                cx.set_cursor(MouseCursor::Default);
                self.animator_play(cx, ids!(hover.off));
            }
            Hit::FingerDown(_) => {
                self.animator_play(cx, ids!(pressed.on));
                cx.widget_action(uid, MpButtonAction::Pressed);
            }
            Hit::FingerUp(fe) => {
                self.animator_play(cx, ids!(pressed.off));
                if fe.is_over {
                    cx.widget_action(uid, MpButtonAction::Clicked);
                }
                cx.widget_action(uid, MpButtonAction::Released);
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

impl MpButton {
    pub fn clicked(&self, actions: &Actions) -> bool {
        if let Some(action) = actions.find_widget_action(self.widget_uid()) {
            matches!(action.cast(), MpButtonAction::Clicked)
        } else {
            false
        }
    }

    pub fn pressed(&self, actions: &Actions) -> bool {
        if let Some(action) = actions.find_widget_action(self.widget_uid()) {
            matches!(action.cast(), MpButtonAction::Pressed)
        } else {
            false
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text.as_mut_empty().push_str(text);
    }

    pub fn set_disabled(&mut self, cx: &mut Cx, disabled: bool) {
        self.disabled = disabled;
        self.animator_toggle(
            cx,
            disabled,
            Animate::No,
            ids!(disabled.on),
            ids!(disabled.off),
        );
        self.redraw(cx);
    }
}

impl MpButtonRef {
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

    pub fn set_disabled(&self, cx: &mut Cx, disabled: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_disabled(cx, disabled);
        }
    }
}
