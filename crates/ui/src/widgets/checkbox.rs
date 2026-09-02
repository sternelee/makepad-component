use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // Checkbox component - uses DrawQuad begin/end pattern for reliable hit testing
    mod.widgets.MpCheckboxBase = #(MpCheckbox::register_widget(vm))
    mod.widgets.MpCheckbox = set_type_default() do mod.widgets.MpCheckboxBase{
        width: Fit
        height: Fit
        align: Align{y: 0.5}
        padding: Inset{left: 4.0, right: 4.0, top: 4.0, bottom: 4.0}
        flow: Right
        spacing: 8.0

        // Outer draw_bg for hit testing area (transparent background)
        draw_bg +: {
            pixel: fn() {
                return vec4(0.0, 0.0, 0.0, 0.0)
            }
        }

        // The checkbox box with checkmark.
        // Aligned with gpui-bezel Controls::checkbox: unchecked is a quiet
        // INPUT_BG square with a BORDER_STRONG outline; checked fills with the
        // max-contrast plate (SOLID) and paints the tick in ON_SOLID.
        draw_check +: {
            checked: instance(0.0)
            hover: instance(0.0)
            focus: instance(0.0)
            radius: instance(4.0)
            bg_off: instance(INPUT_BG)
            bg_on: instance(SOLID)
            border_off: instance(BORDER_STRONG)
            border_on: instance(SOLID)
            border_color_focus: instance(CARET)
            check_color: instance(ON_SOLID)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let sz = self.rect_size

                // Background box
                sdf.box(1.0, 1.0, sz.x - 2.0, sz.y - 2.0, self.radius)

                // Colors
                let bg = mix(self.bg_off, self.bg_on, self.checked)
                let border = mix(self.border_off, self.border_on, self.checked)

                sdf.fill_keep(bg)

                // Focus ring: CARET border at 2px when focused
                let bw = mix(1.5, 2.0, self.focus)
                let bc = mix(border, self.border_color_focus, self.focus)
                sdf.stroke(bc, bw)

                // Draw checkmark when checked
                if (self.checked > 0.5) {
                    let cx = sz.x * 0.5
                    let cy = sz.y * 0.5

                    // Checkmark path (two lines)
                    sdf.move_to(cx - 4.0, cy)
                    sdf.line_to(cx - 1.0, cy + 3.0)
                    sdf.line_to(cx + 4.0, cy - 3.0)
                    sdf.stroke(self.check_color, 2.0)
                }

                return sdf.result
            }
        }

        // Label text
        draw_label +: {
            text_style: theme.font_regular{font_size: 13.0}
            color: TEXT
        }

        text: ""

        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_check: {hover: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_check: {hover: 1.0}}
                }
            }
            checked: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_check: {checked: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_check: {checked: 1.0}}
                }
            }
            focus: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_check: {focus: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_check: {focus: 1.0}}
                }
            }
        }
    }
}

#[derive(Script, Widget, Animator)]
pub struct MpCheckbox {
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
    draw_check: DrawQuad,
    #[live]
    draw_label: DrawText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[live]
    size: MpSize,

    #[live]
    text: ArcStringMut,
    #[live]
    checked: bool,
    #[live]
    disabled: bool,

    #[rust]
    area: Area,

    #[live(false)]
    focused: bool,
}

impl ScriptHook for MpCheckbox {
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
pub enum MpCheckboxAction {
    Changed(bool),
    #[default]
    None,
}

impl Widget for MpCheckbox {
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
                // Claim key focus on click (bezel focus ring)
                cx.set_key_focus(self.area);
            }
            Hit::FingerUp(fe)
                if fe.is_over => {
                    self.checked = !self.checked;
                    self.animator_toggle(
                        cx,
                        self.checked,
                        Animate::Yes,
                        ids!(checked.on),
                        ids!(checked.off),
                    );
                    cx.widget_action(uid, MpCheckboxAction::Changed(self.checked));
                    self.redraw(cx);
                }
            _ => {}
        }

        // Keyboard activation (Space/Enter) when focused
        if self.focused {
            if let Event::KeyDown(ke) = event {
                if ke.key_code == KeyCode::Space || ke.key_code == KeyCode::ReturnKey {
                    if !ke.is_repeat {
                        self.checked = !self.checked;
                        self.animator_toggle(
                            cx,
                            self.checked,
                            Animate::Yes,
                            ids!(checked.on),
                            ids!(checked.off),
                        );
                        cx.widget_action(uid, MpCheckboxAction::Changed(self.checked));
                        self.redraw(cx);
                    }
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // Checkbox square size per MpSize (default Medium = 16).
        let box_size = match self.size {
            MpSize::XSmall => 12.0,
            MpSize::Small => 14.0,
            MpSize::Medium => 16.0,
            MpSize::Large => 19.0,
            MpSize::XLarge => 22.0,
        };

        // Begin outer container (provides hit testing area)
        self.draw_bg.begin(cx, walk, self.layout);

        // Draw checkbox box
        self.draw_check.draw_walk(cx, Walk::fixed(box_size, box_size));

        // Draw label text
        if !self.text.as_ref().is_empty() {
            self.draw_label
                .draw_walk(cx, Walk::fit(), Align::default(), self.text.as_ref());
        }

        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        if !self.disabled {
            crate::widgets::focus::register(cx, self.widget_uid(), self.area);
        }

        DrawStep::done()
    }
}

impl MpCheckbox {
    pub fn set_text(&mut self, text: &str) {
        self.text.as_mut_empty().push_str(text);
    }

    pub fn is_checked(&self) -> bool {
        self.checked
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn set_disabled(&mut self, cx: &mut Cx, disabled: bool) {
        if self.disabled != disabled {
            self.disabled = disabled;
            self.redraw(cx);
        }
    }

    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            self.redraw(cx);
        }
    }

    pub fn set_checked(&mut self, cx: &mut Cx, checked: bool) {
        if self.checked != checked {
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
    }

    pub fn changed(&self, actions: &Actions) -> Option<bool> {
        if let Some(action) = actions.find_widget_action(self.widget_uid()) {
            if let MpCheckboxAction::Changed(checked) = action.cast() {
                return Some(checked);
            }
        }
        None
    }
}

impl MpCheckboxRef {
    pub fn is_checked(&self) -> bool {
        if let Some(inner) = self.borrow() {
            inner.checked
        } else {
            false
        }
    }

    pub fn set_checked(&self, cx: &mut Cx, checked: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_checked(cx, checked);
        }
    }

    pub fn set_disabled(&self, cx: &mut Cx, disabled: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_disabled(cx, disabled);
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

    pub fn changed(&self, actions: &Actions) -> Option<bool> {
        if let Some(inner) = self.borrow() {
            inner.changed(actions)
        } else {
            None
        }
    }
}
