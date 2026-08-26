use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // Base button component - macOS style
    mod.widgets.MpButtonBase = #(MpButton::register_widget(vm))
    mod.widgets.MpButton = set_type_default() do mod.widgets.MpButtonBase{
        width: Fit
        height: Fit
        align: Align{x: 0.5, y: 0.5}
        padding: Inset{left: 12.0, right: 12.0, top: 6.0, bottom: 6.0}

        draw_bg +: {
            radius: instance(8.0)
            border_width: instance(0.0)
            border_color: instance(#x0000)
            border_color_focus: instance(CARET)
            focus: instance(0.0)
            hover: instance(0.0)
            pressed: instance(0.0)
            disabled: instance(0.0)
            color: instance(ACCENT)
            color_hover: instance(ACCENT_HOVER)
            color_pressed: instance(ACCENT_HOVER)
            color_disabled: instance(TEXT_FAINT)

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

                // Focus ring: CARET border at 2px when focused (overrides any existing border)
                let bw = mix(self.border_width, 2.0, self.focus)
                let bc = mix(self.border_color, self.border_color_focus, self.focus)
                if (bw > 0.0) {
                    sdf.stroke(bc, bw)
                }

                return sdf.result
            }
        }

        draw_text +: {
            text_style: theme.font_regular{font_size: 13.0}
            color: ON_ACCENT
            disabled: instance(0.0)
            color_disabled: instance(ON_SOLID)
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
            focus: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_bg: {focus: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_bg: {focus: 1.0}}
                }
            }
        }
    }

    // Variant: Prominent — the solid plate (bezel ButtonStyle.Prominent)
    mod.widgets.MpButtonProminent = mod.widgets.MpButton{
        draw_bg +: {
            color: instance(SOLID)
            color_hover: instance(SOLID_HOVER)
            color_pressed: instance(SOLID_HOVER)
        }
        draw_text +: {
            color: ON_SOLID
        }
    }

    // Variant: Ghost — muted label, hover wash (bezel ButtonStyle.Ghost)
    mod.widgets.MpButtonGhost = mod.widgets.MpButton{
        draw_bg +: {
            color: instance(TRANSPARENT)
            color_hover: instance(ELEMENT_HOVER)
            color_pressed: instance(ELEMENT_ACTIVE)
            border_width: instance(0.0)
        }
        draw_text +: {
            text_style: theme.font_regular{font_size: 13.0}
            color: TEXT_MUTED
        }
    }

    // Variant: Outline — hairline border, body-text label
    mod.widgets.MpButtonOutline = mod.widgets.MpButton{
        draw_bg +: {
            color: instance(TRANSPARENT)
            color_hover: instance(ELEMENT_HOVER)
            color_pressed: instance(ELEMENT_ACTIVE)
            border_width: instance(1.0)
            border_color: instance(BORDER_STRONG)
        }
        draw_text +: {
            text_style: theme.font_regular{font_size: 13.0}
            color: TEXT
        }
    }

    // Variant: Destructive — danger solid plate
    mod.widgets.MpButtonDestructive = mod.widgets.MpButton{
        draw_bg +: {
            color: instance(DANGER)
            color_hover: instance(DANGER_HOVER)
            color_pressed: instance(DANGER_HOVER)
        }
        draw_text +: {
            color: ON_SOLID
        }
    }

    // Size: Small — compact control
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

    #[live(false)]
    focused: bool,
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

        // Sync focus state from Cx
        let has_focus = cx.has_key_focus(self.area);
        if has_focus != self.focused {
            self.focused = has_focus;
            self.animator_toggle(cx, has_focus, Animate::Yes, ids!(focus.on), ids!(focus.off));
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
                // Claim key focus on click (bezel focus ring)
                cx.set_key_focus(self.area);
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

        // Keyboard activation (Enter/Space) when this widget has key focus
        if self.focused {
            if let Event::KeyDown(ke) = event {
                if ke.key_code == KeyCode::ReturnKey || ke.key_code == KeyCode::Space {
                    if !ke.is_repeat {
                        cx.widget_action(uid, MpButtonAction::Clicked);
                    }
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_text
            .draw_walk(cx, Walk::fit(), Align::default(), self.text.as_ref());
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        if !self.disabled {
            crate::widgets::focus::register(cx, self.widget_uid(), self.area);
        }
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
