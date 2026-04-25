use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::theme::colors::*;

    // Base button component - macOS style
    pub MpButton = {{MpButton}} {
        width: Fit,
        height: Fit,
        align: { x: 0.5, y: 0.5 }
        padding: { left: 16, right: 16, top: 8, bottom: 8 }

        draw_bg: {
            instance radius: 6.0
            instance border_width: 0.0
            instance border_color: #0000
            instance hover: 0.0
            instance pressed: 0.0
            instance disabled: 0.0
            instance color: (PRIMARY)
            instance color_hover: (PRIMARY_HOVER)
            instance color_pressed: (PRIMARY_ACTIVE)
            instance color_disabled: #8f9bb3

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(
                    self.border_width,
                    self.border_width,
                    self.rect_size.x - self.border_width * 2.0,
                    self.rect_size.y - self.border_width * 2.0,
                    max(1.0, self.radius)
                );

                // Subtle hover: slightly brighten
                let hover_color = mix(self.color, self.color_hover, self.hover * 0.6);
                // Pressed: darken more noticeably
                let pressed_color = mix(hover_color, self.color_pressed, self.pressed * 0.8);
                let final_color = mix(pressed_color, self.color_disabled, self.disabled);

                sdf.fill_keep(final_color);

                if self.border_width > 0.0 {
                    sdf.stroke(self.border_color, self.border_width);
                }

                return sdf.result;
            }
        }

        draw_text: {
            text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }
            color: (PRIMARY_FOREGROUND)
            instance disabled: 0.0
            instance color_disabled: #e6e9ef
            fn get_color(self) -> vec4 {
                return mix(self.color, self.color_disabled, self.disabled);
            }
        }

        text: ""

        animator: {
            hover = {
                default: off
                off = {
                    from: { all: Forward { duration: 0.15 } }
                    apply: { draw_bg: { hover: 0.0 } }
                }
                on = {
                    from: { all: Forward { duration: 0.15 } }
                    apply: { draw_bg: { hover: 1.0 } }
                }
            }
            pressed = {
                default: off
                off = {
                    from: { all: Forward { duration: 0.08 } }
                    apply: { draw_bg: { pressed: 0.0 } }
                }
                on = {
                    from: { all: Forward { duration: 0.08 } }
                    apply: { draw_bg: { pressed: 1.0 } }
                }
            }
        }
    }

    // Variant: Primary Button (macOS system blue)
    pub MpButtonPrimary = <MpButton> {
        draw_bg: {
            color: (PRIMARY)
            color_hover: (PRIMARY_HOVER)
            color_pressed: (PRIMARY_ACTIVE)
        }
        draw_text: {
            color: (PRIMARY_FOREGROUND)
        }
    }

    // Variant: Secondary Button - macOS style (lighter, subtle)
    pub MpButtonSecondary = <MpButton> {
        padding: { left: 14, right: 14, top: 7, bottom: 7 }
        draw_bg: {
            color: (SECONDARY)
            color_hover: (SECONDARY_HOVER)
            color_pressed: (SECONDARY_ACTIVE)
            border_width: 0.0
        }
        draw_text: {
            text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }
            color: (SECONDARY_FOREGROUND)
        }
    }

    // Variant: Danger Button (macOS system red)
    pub MpButtonDanger = <MpButton> {
        draw_bg: {
            color: (DANGER)
            color_hover: (DANGER_HOVER)
            color_pressed: (DANGER_ACTIVE)
        }
        draw_text: {
            color: (DANGER_FOREGROUND)
        }
    }

    // Variant: Success Button (macOS system green)
    pub MpButtonSuccess = <MpButton> {
        draw_bg: {
            color: (SUCCESS)
            color_hover: (SUCCESS_HOVER)
            color_pressed: (SUCCESS_ACTIVE)
        }
        draw_text: {
            color: (SUCCESS_FOREGROUND)
        }
    }

    // Variant: Warning Button (macOS system orange)
    pub MpButtonWarning = <MpButton> {
        draw_bg: {
            color: (WARNING)
            color_hover: (WARNING_HOVER)
            color_pressed: (WARNING_ACTIVE)
        }
        draw_text: {
            color: (WARNING_FOREGROUND)
        }
    }

    // Variant: Ghost Button - macOS style (transparent with hover fill)
    pub MpButtonGhost = <MpButton> {
        draw_bg: {
            color: (TRANSPARENT)
            color_hover: (SECONDARY)
            color_pressed: (SECONDARY_ACTIVE)
        }
        draw_text: {
            text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }
            color: (PRIMARY)
        }
    }

    // Variant: Outline Button - macOS style
    pub MpButtonOutline = <MpButton> {
        padding: { left: 14, right: 14, top: 7, bottom: 7 }
        draw_bg: {
            color: (TRANSPARENT)
            color_hover: (SECONDARY)
            color_pressed: (SECONDARY_ACTIVE)
            border_width: 1.0
            border_color: (BORDER)
        }
        draw_text: {
            text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }
            color: (FOREGROUND)
        }
    }

    // Size: Small - macOS compact size
    pub MpButtonSmall = <MpButton> {
        padding: { left: 12, right: 12, top: 4, bottom: 4 }
        draw_text: {
            text_style: <THEME_FONT_REGULAR>{ font_size: 11.0 }
        }
    }

    // Size: Large
    pub MpButtonLarge = <MpButton> {
        padding: { left: 24, right: 24, top: 10, bottom: 10 }
        draw_text: {
            text_style: <THEME_FONT_REGULAR>{ font_size: 15.0 }
        }
    }
}

// Rust implementation
#[derive(Live, LiveHook, Widget)]
pub struct MpButton {
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

    #[animator]
    animator: Animator,

    #[rust]
    area: Area,
}

// Custom Actions
#[derive(Clone, Debug, DefaultNone)]
pub enum MpButtonAction {
    Clicked,
    Pressed,
    Released,
    None,
}

impl Widget for MpButton {
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
                self.animator_play(cx, ids!(pressed.on));
                cx.widget_action(uid, &scope.path, MpButtonAction::Pressed);
            }
            Hit::FingerUp(fe) => {
                self.animator_play(cx, ids!(pressed.off));
                if fe.is_over {
                    cx.widget_action(uid, &scope.path, MpButtonAction::Clicked);
                }
                cx.widget_action(uid, &scope.path, MpButtonAction::Released);
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let disabled_f = if self.disabled { 1.0 } else { 0.0 };
        self.draw_bg
            .apply_over(cx, live! { disabled: (disabled_f) });
        self.draw_text
            .apply_over(cx, live! { disabled: (disabled_f) });
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
            matches!(action.cast::<MpButtonAction>(), MpButtonAction::Clicked)
        } else {
            false
        }
    }

    pub fn pressed(&self, actions: &Actions) -> bool {
        if let Some(action) = actions.find_widget_action(self.widget_uid()) {
            matches!(action.cast::<MpButtonAction>(), MpButtonAction::Pressed)
        } else {
            false
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text.as_mut_empty().push_str(text);
    }

    pub fn set_disabled(&mut self, cx: &mut Cx, disabled: bool) {
        self.disabled = disabled;
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
