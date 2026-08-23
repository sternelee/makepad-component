use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // Switch toggle component - macOS style pill track with a sliding knob.
    // Track and knob are drawn in a single SDF shader (the 2.0 animator cannot
    // target named child views, so the old track/thumb_wrap/thumb view tree is
    // merged here, matching the Toggle pattern in makepad's check_box.rs).
    mod.widgets.MpSwitchBase = #(MpSwitch::register_widget(vm))
    mod.widgets.MpSwitch = set_type_default() do mod.widgets.MpSwitchBase{
        width: 44.0
        height: 24.0

        draw_bg +: {
            on: instance(0.0)
            hover: instance(0.0)
            track_off: uniform(SURFACE_RAISED)
            track_on: uniform(SUCCESS)
            thumb_color: uniform(#xf8fafc)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let sz = self.rect_size
                let r = sz.y * 0.5

                // Track capsule: left circle + rectangle + right circle
                sdf.circle(r, r, r)
                sdf.rect(r, 0.0, sz.x - sz.y, sz.y)
                sdf.circle(sz.x - r, r, r)

                // macOS style colors: subtle gray when off, system green when on
                let mut color = mix(self.track_off, self.track_on, self.on)
                // Subtle brighten on hover
                color = mix(color, #xffffff, self.hover * 0.15)

                sdf.fill(color)

                // Thumb: white knob (18px at the default 44x24 size, 3px padding)
                // sliding from the left end to the right end as `on` goes 0 -> 1
                let thumb_r = r - 3.0
                let knob_x = mix(r, sz.x - r, self.on)
                sdf.circle(knob_x, r, thumb_r)
                sdf.fill(self.thumb_color)

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
