use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // Switch toggle component - bezel-style pill track with a sliding knob.
    // Track and knob are drawn in a single SDF shader (the 2.0 animator cannot
    // target named child views, so the old track/thumb_wrap/thumb view tree is
    // merged here, matching the Toggle pattern in makepad's check_box.rs).
    // Aligned with gpui-bezel Controls::toggle: 32x18 capsule, on-state flips
    // to the max-contrast plate (SOLID), thumb 14px muted -> on_solid.
    mod.widgets.MpSwitchBase = #(MpSwitch::register_widget(vm))
    mod.widgets.MpSwitch = set_type_default() do mod.widgets.MpSwitchBase{
        width: 32.0
        height: 18.0

        draw_bg +: {
            on: instance(0.0)
            hover: instance(0.0)
            focus: instance(0.0)
            track_off: instance(ELEMENT_ACTIVE)
            track_on: instance(SOLID)
            thumb_off: instance(TEXT_FAINT)
            thumb_on: instance(ON_SOLID)
            focus_color: instance(CARET)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let sz = self.rect_size
                let r = sz.y * 0.5

                // Track capsule: left circle + rectangle + right circle
                sdf.circle(r, r, r)
                sdf.rect(r, 0.0, sz.x - sz.y, sz.y)
                sdf.circle(sz.x - r, r, r)

                let mut color = mix(self.track_off, self.track_on, self.on)
                // Subtle lift on hover
                color = mix(color, self.track_on, self.hover * 0.15)

                sdf.fill(color)

                // Thumb: 14px knob sliding left -> right as `on` goes 0 -> 1
                let thumb_r = r - 2.0
                let knob_x = mix(r, sz.x - r, self.on)
                let thumb_color = mix(self.thumb_off, self.thumb_on, self.on)
                sdf.circle(knob_x, r, thumb_r)
                sdf.fill(thumb_color)

                // Focus ring: CARET hairline around the capsule when focused
                if (self.focus > 0.5) {
                    sdf.circle(r, r, r + 0.5)
                    sdf.rect(r, -0.5, sz.x - sz.y, sz.y + 1.0)
                    sdf.circle(sz.x - r, r, r + 0.5)
                    sdf.stroke(self.focus_color, 1.5)
                }

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
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_bg: {hover: 1.0}}
                }
            }
            on: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.2}}
                    apply: {draw_bg: {on: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.2}}
                    apply: {draw_bg: {on: 1.0}}
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
}

#[derive(Script, Widget, Animator)]
pub struct MpSwitch {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[apply_default]
    animator: Animator,

    #[redraw]
    #[live]
    draw_bg: DrawQuad,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[live]
    on: bool,
    #[live]
    disabled: bool,

    #[rust]
    area: Area,

    #[live(false)]
    focused: bool,
}

impl ScriptHook for MpSwitch {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        vm.with_cx_mut(|cx| {
            let on = self.on;
            self.animator_toggle(cx, on, Animate::No, ids!(on.on), ids!(on.off));
        });
    }
}

#[derive(Clone, Debug, Default)]
pub enum MpSwitchAction {
    Changed(bool),
    #[default]
    None,
}

impl Widget for MpSwitch {
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
            Hit::FingerUp(fe)
                if fe.is_over => {
                    self.on = !self.on;
                    self.animator_toggle(cx, self.on, Animate::Yes, ids!(on.on), ids!(on.off));
                    cx.widget_action(uid, MpSwitchAction::Changed(self.on));
                    self.redraw(cx);
                }
            _ => {}
        }

        // Keyboard activation (Space/Enter) when focused
        if self.focused {
            if let Event::KeyDown(ke) = event {
                if ke.key_code == KeyCode::Space || ke.key_code == KeyCode::ReturnKey {
                    if !ke.is_repeat {
                        self.on = !self.on;
                        self.animator_toggle(cx, self.on, Animate::Yes, ids!(on.on), ids!(on.off));
                        cx.widget_action(uid, MpSwitchAction::Changed(self.on));
                        self.redraw(cx);
                    }
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        DrawStep::done()
    }
}

impl MpSwitch {
    pub fn is_on(&self) -> bool {
        self.on
    }

    pub fn set_on(&mut self, cx: &mut Cx, on: bool) {
        self.on = on;
        self.animator_toggle(cx, on, Animate::Yes, ids!(on.on), ids!(on.off));
        self.redraw(cx);
    }

    pub fn changed(&self, actions: &Actions) -> Option<bool> {
        if let Some(action) = actions.find_widget_action(self.widget_uid()) {
            if let MpSwitchAction::Changed(on) = action.cast() {
                return Some(on);
            }
        }
        None
    }
}

impl MpSwitchRef {
    pub fn is_on(&self) -> bool {
        if let Some(inner) = self.borrow() {
            inner.on
        } else {
            false
        }
    }

    pub fn set_on(&self, cx: &mut Cx, on: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_on(cx, on);
        }
    }

    pub fn changed(&self, actions: &Actions) -> Option<bool> {
        if let Some(inner) = self.borrow() {
            inner.changed(actions)
        } else {
            None
        }
    }
}
