use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // Radio button component - uses DrawQuad begin/end pattern for reliable hit testing
    mod.widgets.MpRadioBase = #(MpRadio::register_widget(vm))
    mod.widgets.MpRadio = set_type_default() do mod.widgets.MpRadioBase{
        width: Fit
        height: Fit
        align: Align{y: 0.5}
        flow: Right
        spacing: 8.0

        // Outer draw_bg for hit testing area (transparent background)
        draw_bg +: {
            pixel: fn() {
                return vec4(0.0, 0.0, 0.0, 0.0)
            }
        }

        // The radio circle with inner dot.
        // Aligned with gpui-bezel Controls::radio_button: a 16px ring; the
        // selected ring and inner dot take the max-contrast plate (SOLID),
        // unselected is a quiet BORDER_STRONG ring over INPUT_BG.
        draw_circle +: {
            checked: instance(0.0)
            hover: instance(0.0)
            ring_off: instance(BORDER_STRONG)
            ring_on: instance(SOLID)
            bg_off: instance(INPUT_BG)
            dot_color: instance(SOLID)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let sz = self.rect_size
                let center = sz * 0.5
                let radius = center.x - 1.0

                // Outer circle
                sdf.circle(center.x, center.y, radius)

                // Colors
                let bg = mix(self.bg_off, self.bg_off, self.checked)
                let border = mix(self.ring_off, self.ring_on, self.checked)

                sdf.fill_keep(bg)
                sdf.stroke(border, 1.5)

                // Inner dot when checked
                if (self.checked > 0.5) {
                    let dot_radius = radius * 0.5
                    sdf.circle(center.x, center.y, dot_radius)
                    sdf.fill(self.dot_color)
                }

                return sdf.result
            }
        }

        // Label text
        draw_label +: {
            text_style: theme.font_regular{font_size: 14.0}
            color: TEXT
        }

        text: ""

        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_circle: {hover: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.1}}
                    apply: {draw_circle: {hover: 1.0}}
                }
            }
            checked: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_circle: {checked: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_circle: {checked: 1.0}}
                }
            }
        }
    }
}

#[derive(Script, Widget, Animator)]
pub struct MpRadio {
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
    draw_circle: DrawQuad,
    #[live]
    draw_label: DrawText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[live]
    text: ArcStringMut,
    #[live]
    checked: bool,
    #[live]
    disabled: bool,
    #[live]
    value: ArcStringMut,

    #[rust]
    area: Area,
}

impl ScriptHook for MpRadio {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        vm.with_cx_mut(|cx| {
            let checked = self.checked;
            self.animator_toggle(
                cx,
                checked,
                Animate::No,
                ids!(checked.on),
                ids!(checked.off),
            );
        });
    }
}

#[derive(Clone, Debug, Default)]
pub enum MpRadioAction {
    Changed(bool),
    #[default]
    None,
}

impl Widget for MpRadio {
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
                if fe.is_over && !self.checked => {
                    // Radio can only be checked, not unchecked by clicking
                    self.checked = true;
                    self.animator_play(cx, ids!(checked.on));
                    cx.widget_action(uid, MpRadioAction::Changed(true));
                    self.redraw(cx);
                }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // Begin outer container (provides hit testing area)
        self.draw_bg.begin(cx, walk, self.layout);

        // Draw radio circle (18x18)
        self.draw_circle.draw_walk(cx, Walk::fixed(16.0, 16.0));

        // Draw label text
        if !self.text.as_ref().is_empty() {
            self.draw_label
                .draw_walk(cx, Walk::fit(), Align::default(), self.text.as_ref());
        }

        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();

        DrawStep::done()
    }
}

impl MpRadio {
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    pub fn value(&self) -> &str {
        self.value.as_ref()
    }

    pub fn set_checked(&mut self, cx: &mut Cx, checked: bool) {
        self.checked = checked;
        self.animator_toggle(
            cx,
            checked,
            Animate::Yes,
            ids!(checked.on),
            ids!(checked.off),
        );
        self.redraw(cx);
    }

    pub fn changed(&self, actions: &Actions) -> Option<bool> {
        if let Some(action) = actions.find_widget_action(self.widget_uid()) {
            if let MpRadioAction::Changed(checked) = action.cast() {
                return Some(checked);
            }
        }
        None
    }
}

impl MpRadioRef {
    pub fn is_checked(&self) -> bool {
        if let Some(inner) = self.borrow() {
            inner.checked
        } else {
            false
        }
    }

    pub fn value(&self) -> String {
        if let Some(inner) = self.borrow() {
            inner.value.as_ref().to_string()
        } else {
            String::new()
        }
    }

    pub fn set_checked(&self, cx: &mut Cx, checked: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_checked(cx, checked);
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
