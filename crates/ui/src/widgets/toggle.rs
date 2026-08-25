use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpToggle - shadcn-style toggle button (two-state on/off)
    // ============================================================

    // Base Toggle
    mod.widgets.MpToggleBase = #(MpToggle::register_widget(vm))
    mod.widgets.MpToggle = set_type_default() do mod.widgets.MpToggleBase{
        width: Fit
        height: Fit
        padding: Inset{left: 12.0, right: 12.0, top: 8.0, bottom: 8.0}
        align: Align{x: 0.5, y: 0.5}

        draw_bg +: {
            radius: instance(6.0)
            border_width: instance(1.0)
            border_color: instance(BORDER)
            border_color_focus: instance(CARET)
            focus: instance(0.0)
            bg_color: instance(#x0000)
            bg_hover: instance(ELEMENT_HOVER)
            bg_active: instance(ELEMENT_ACTIVE)
            bg_checked: instance(ACCENT)
            hover: instance(0.0)
            pressed: instance(0.0)
            active: instance(0.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)

                // Box with rounded corners
                sdf.box(
                    self.border_width,
                    self.border_width,
                    self.rect_size.x - self.border_width * 2.0,
                    self.rect_size.y - self.border_width * 2.0,
                    self.radius
                )

                // Background: checked ? primary : normal with hover/press
                let normal_bg = mix(self.bg_color, self.bg_hover, self.hover)
                let pressed_bg = mix(normal_bg, self.bg_active, self.pressed)
                let final_bg = mix(pressed_bg, self.bg_checked, self.active)

                sdf.fill_keep(final_bg)

                // Border (fade when active); focus ring: CARET at 2px overrides
                let final_border = mix(self.border_color, self.bg_checked, self.active)
                let bw = mix(self.border_width, 2.0, self.focus)
                let bc = mix(final_border, self.border_color_focus, self.focus)
                if (bw > 0.0 && (self.active < 0.5 || self.focus > 0.5)) {
                    sdf.stroke(bc, bw)
                }

                return sdf.result
            }
        }

        draw_text +: {
            text_style: theme.font_bold{font_size: 13.0}
            color: (TEXT)
            color_active: instance(ON_ACCENT)
            active: instance(0.0)
            get_color: fn() {
                return mix(self.color, self.color_active, self.active)
            }
        }

        text: ""
        active: false

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
                    apply: {draw_bg: {pressed: 0.0} draw_text: {active: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.08}}
                    apply: {draw_bg: {pressed: 1.0} draw_text: {active: 1.0}}
                }
            }
            active: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.2}}
                    redraw: true
                    apply: {draw_bg: {active: 0.0} draw_text: {active: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.2}}
                    redraw: true
                    apply: {draw_bg: {active: 1.0} draw_text: {active: 1.0}}
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

    // Variants
    // Ghost Toggle (no border)
    mod.widgets.MpToggleGhost = mod.widgets.MpToggle{
        draw_bg +: {
            border_width: instance(0.0)
        }
    }

    // Size: Small
    mod.widgets.MpToggleSmall = mod.widgets.MpToggle{
        padding: Inset{left: 8.0, right: 8.0, top: 4.0, bottom: 4.0}
        draw_text +: {
            text_style: theme.font_bold{font_size: 11.0}
        }
    }

    // Size: Large
    mod.widgets.MpToggleLarge = mod.widgets.MpToggle{
        padding: Inset{left: 16.0, right: 16.0, top: 10.0, bottom: 10.0}
        draw_text +: {
            text_style: theme.font_bold{font_size: 15.0}
        }
    }

    // ============================================================
    // Toggle Group (radio-style, only one active)
    // ============================================================
    mod.widgets.MpToggleGroup = mod.widgets.View{
        width: Fit
        height: Fit
        flow: Right
        spacing: 4.0
    }
}

// ============================================================
// Rust Implementation
// ============================================================

#[derive(Script, Widget, Animator)]
pub struct MpToggle {
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
    active: bool,

    #[rust]
    area: Area,

    #[live(false)]
    focused: bool,
}

impl ScriptHook for MpToggle {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        vm.with_cx_mut(|cx| {
            let active = self.active;
            self.animator_toggle(cx, active, Animate::No, ids!(active.on), ids!(active.off));
        });
    }
}

#[derive(Clone, Debug, Default)]
pub enum MpToggleAction {
    Toggle,
    Active(bool),
    #[default]
    None,
}

impl Widget for MpToggle {
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

        match event.hits(cx, self.area) {
            Hit::FingerHoverIn(_) => {
                cx.set_cursor(MouseCursor::Hand);
                self.animator_play(cx, ids!(hover.on));
            }
            Hit::FingerHoverOut(_) => {
                self.animator_play(cx, ids!(hover.off));
            }
            Hit::FingerDown(_) => {
                self.animator_play(cx, ids!(pressed.on));
                // Claim key focus on click (bezel focus ring)
                cx.set_key_focus(self.area);
            }
            Hit::FingerUp(fe) => {
                self.animator_play(cx, ids!(pressed.off));
                if fe.is_over {
                    self.active = !self.active;
                    self.animator_toggle(
                        cx,
                        self.active,
                        Animate::Yes,
                        ids!(active.on),
                        ids!(active.off),
                    );
                    cx.widget_action(uid, MpToggleAction::Toggle);
                    cx.widget_action(uid, MpToggleAction::Active(self.active));
                }
            }
            _ => {}
        }

        // Keyboard activation (Space/Enter) when focused
        if self.focused {
            if let Event::KeyDown(ke) = event {
                if ke.key_code == KeyCode::Space || ke.key_code == KeyCode::ReturnKey {
                    if !ke.is_repeat {
                        self.active = !self.active;
                        self.animator_toggle(
                            cx,
                            self.active,
                            Animate::Yes,
                            ids!(active.on),
                            ids!(active.off),
                        );
                        cx.widget_action(uid, MpToggleAction::Toggle);
                        cx.widget_action(uid, MpToggleAction::Active(self.active));
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
        DrawStep::done()
    }
}

impl MpToggle {
    pub fn toggled(&self, actions: &Actions) -> Option<bool> {
        if let Some(action) = actions.find_widget_action(self.widget_uid()) {
            if let MpToggleAction::Active(v) = action.cast() {
                return Some(v);
            }
        }
        None
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, cx: &mut Cx, active: bool) {
        if self.active != active {
            self.active = active;
            self.animator_toggle(cx, active, Animate::Yes, ids!(active.on), ids!(active.off));
            self.redraw(cx);
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text.as_mut_empty().push_str(text);
    }
}

impl MpToggleRef {
    pub fn toggled(&self, actions: &Actions) -> Option<bool> {
        if let Some(inner) = self.borrow() {
            inner.toggled(actions)
        } else {
            None
        }
    }

    pub fn is_active(&self) -> bool {
        if let Some(inner) = self.borrow() {
            inner.is_active()
        } else {
            false
        }
    }

    pub fn set_active(&self, cx: &mut Cx, active: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_active(cx, active);
        }
    }

    pub fn set_text(&self, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(text);
        }
    }
}
