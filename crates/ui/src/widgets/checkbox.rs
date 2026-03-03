use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::theme::colors::*;

    // Checkbox component - uses DrawQuad begin/end pattern for reliable hit testing
    pub MpCheckbox = {{MpCheckbox}} {
        width: Fit,
        height: Fit,
        align: { y: 0.5 }
        padding: { left: 4, right: 4, top: 4, bottom: 4 }
        flow: Right
        spacing: 8

        // Outer draw_bg for hit testing area (transparent background)
        draw_bg: {
            fn pixel(self) -> vec4 {
                return vec4(0.0, 0.0, 0.0, 0.0);
            }
        }

        // The checkbox box with checkmark
        draw_check: {
            instance checked: 0.0
            instance hover: 0.0
            instance radius: 4.0

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                let sz = self.rect_size;

                // Background box
                sdf.box(1.0, 1.0, sz.x - 2.0, sz.y - 2.0, self.radius);

                // Colors
                let bg_unchecked = #ffffff;
                let bg_checked = (PRIMARY);
                let border_unchecked = mix((BORDER), (PRIMARY), self.hover * 0.5);
                let border_checked = (PRIMARY);

                // Interpolate based on checked state
                let bg = mix(bg_unchecked, bg_checked, self.checked);
                let border = mix(border_unchecked, border_checked, self.checked);

                sdf.fill_keep(bg);
                sdf.stroke(border, 1.5);

                // Draw checkmark when checked
                if self.checked > 0.5 {
                    let check_color = #ffffff;
                    let cx = sz.x * 0.5;
                    let cy = sz.y * 0.5;

                    // Checkmark path (two lines)
                    sdf.move_to(cx - 4.0, cy);
                    sdf.line_to(cx - 1.0, cy + 3.0);
                    sdf.line_to(cx + 4.0, cy - 3.0);
                    sdf.stroke(check_color, 2.0);
                }

                return sdf.result;
            }
        }

        // Label text
        draw_label: {
            text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }
            color: (FOREGROUND)
        }

        text: ""

        animator: {
            hover = {
                default: off
                off = {
                    from: { all: Forward { duration: 0.15 } }
                    apply: { draw_check: { hover: 0.0 } }
                }
                on = {
                    from: { all: Forward { duration: 0.1 } }
                    apply: { draw_check: { hover: 1.0 } }
                }
            }
            checked = {
                default: off
                off = {
                    from: { all: Forward { duration: 0.15 } }
                    apply: { draw_check: { checked: 0.0 } }
                }
                on = {
                    from: { all: Forward { duration: 0.15 } }
                    apply: { draw_check: { checked: 1.0 } }
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct MpCheckbox {
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
    text: ArcStringMut,
    #[live]
    checked: bool,
    #[live]
    disabled: bool,

    #[animator]
    animator: Animator,

    #[rust]
    area: Area,
}

#[derive(Clone, Debug, DefaultNone)]
pub enum MpCheckboxAction {
    Changed(bool),
    None,
}

impl Widget for MpCheckbox {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
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
                // Finger down - no action yet, wait for FingerUp
            }
            Hit::FingerUp(fe) => {
                if fe.is_over {
                    self.checked = !self.checked;

                    if self.checked {
                        self.animator_play(cx, ids!(checked.on));
                    } else {
                        self.animator_play(cx, ids!(checked.off));
                    }

                    cx.widget_action(uid, &scope.path, MpCheckboxAction::Changed(self.checked));
                    self.redraw(cx);
                }
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // Begin outer container (provides hit testing area)
        self.draw_bg.begin(cx, walk, self.layout);

        // Draw checkbox box (18x18)
        let check_walk = Walk {
            width: Size::Fixed(18.0),
            height: Size::Fixed(18.0),
            ..Walk::default()
        };
        self.draw_check.draw_walk(cx, check_walk);

        // Draw label text
        if !self.text.as_ref().is_empty() {
            self.draw_label.draw_walk(cx, Walk::fit(), Align::default(), self.text.as_ref());
        }

        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();

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

    pub fn set_checked(&mut self, cx: &mut Cx, checked: bool) {
        if self.checked != checked {
            self.checked = checked;
            if checked {
                self.animator_play(cx, ids!(checked.on));
            } else {
                self.animator_play(cx, ids!(checked.off));
            }
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

    pub fn changed(&self, actions: &Actions) -> Option<bool> {
        if let Some(inner) = self.borrow() {
            inner.changed(actions)
        } else {
            None
        }
    }
}
