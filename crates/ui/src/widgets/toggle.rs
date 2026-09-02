use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpToggle - shadcn-style toggle button (two-state on/off)
    // ============================================================

    // Toggle shader: plate with optional hairline border. Instances are
    // written from Rust in draw_walk (the 2.0 replacement for apply_over);
    // the palette instances are baked from the theme at apply time.
    // Disabled fades the whole control to half opacity and mutes the label.
    set_type_default() do #(DrawMpToggle::script_shader(vm)){
        ..mod.draw.DrawQuad

        hover: 0.0
        pressed: 0.0
        focus: 0.0
        active: 0.0
        disabled: 0.0

        radius: 6.0
        border_width: 1.0
        border_color: BORDER
        border_color_focus: CARET
        bg_color: #x0000
        bg_hover: ELEMENT_HOVER
        bg_active: ELEMENT_ACTIVE
        bg_checked: ACCENT

        // Theme palette (read from Rust to resolve the label color)
        c_text: TEXT
        c_text_faint: TEXT_FAINT
        on_accent: ON_ACCENT

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

            // Background: checked ? accent : normal with hover/press
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

            // Disabled: fade the whole control to half opacity
            let fade = mix(1.0, 0.5, self.disabled)
            let res = sdf.result
            return vec4(res.x, res.y, res.z, res.w * fade)
        }
    }

    // Base Toggle
    mod.widgets.MpToggleBase = #(MpToggle::register_widget(vm))
    mod.widgets.MpToggle = set_type_default() do mod.widgets.MpToggleBase{
        width: Fit
        height: Fit
        align: Align{x: 0.5, y: 0.5}

        size: MpSize.Medium

        draw_text +: {
            text_style: theme.font_bold{font_size: 13.0}
            color: (TEXT)
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
                    apply: {draw_bg: {pressed: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.08}}
                    apply: {draw_bg: {pressed: 1.0}}
                }
            }
            active: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.2}}
                    redraw: true
                    apply: {draw_bg: {active: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.2}}
                    redraw: true
                    apply: {draw_bg: {active: 1.0}}
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
            border_width: 0.0
        }
    }

    // Size aliases (size system drives the metrics now)
    mod.widgets.MpToggleSmall = mod.widgets.MpToggle{
        size: MpSize.Small
    }

    mod.widgets.MpToggleLarge = mod.widgets.MpToggle{
        size: MpSize.Large
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

/// SDF paint for the toggle plate; instances are written from Rust in
/// draw_walk (the 2.0 replacement for apply_over).
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpToggle {
    #[deref]
    draw_super: DrawQuad,

    // Animator-driven state
    #[live]
    hover: f32,
    #[live]
    pressed: f32,
    #[live]
    focus: f32,
    #[live]
    active: f32,
    // Written from Rust each draw (1.0 = disabled)
    #[live]
    disabled: f32,

    // Paint
    #[live]
    radius: f32,
    #[live]
    border_width: f32,
    #[live]
    border_color: Vec4f,
    #[live]
    border_color_focus: Vec4f,
    #[live]
    bg_color: Vec4f,
    #[live]
    bg_hover: Vec4f,
    #[live]
    bg_active: Vec4f,
    #[live]
    bg_checked: Vec4f,

    // Theme palette (baked at apply time; read from Rust)
    #[live]
    c_text: Vec4f,
    #[live]
    c_text_faint: Vec4f,
    #[live]
    on_accent: Vec4f,
}

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
    draw_bg: DrawMpToggle,
    #[live]
    draw_text: DrawText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[live]
    size: MpSize,

    #[live]
    text: ArcStringMut,
    #[live]
    active: bool,
    #[live]
    disabled: bool,

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
        // Metrics from the size system (default Medium = 12/6 padding, 13px)
        self.layout.padding = Inset {
            left: self.size.padding_h(),
            right: self.size.padding_h(),
            top: self.size.padding_v(),
            bottom: self.size.padding_v(),
        };
        self.draw_text.text_style.font_size = self.size.font_size();
        self.draw_bg.radius = self.size.radius();

        let disabled = if self.disabled { 1.0f32 } else { 0.0 };
        self.draw_bg.disabled = disabled;

        // Label: text -> on_accent when active, toward faint when disabled
        let active = if self.active { 1.0f32 } else { 0.0 };
        let text = self.draw_bg.c_text;
        let on_accent = self.draw_bg.on_accent;
        let faint = self.draw_bg.c_text_faint;
        let base = Vec4f {
            x: text.x + (on_accent.x - text.x) * active,
            y: text.y + (on_accent.y - text.y) * active,
            z: text.z + (on_accent.z - text.z) * active,
            w: text.w + (on_accent.w - text.w) * active,
        };
        self.draw_text.color = Vec4f {
            x: base.x + (faint.x - base.x) * disabled,
            y: base.y + (faint.y - base.y) * disabled,
            z: base.z + (faint.z - base.z) * disabled,
            w: base.w + (faint.w - base.w) * disabled,
        };

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

    pub fn set_text(&self, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(text);
        }
    }
}
