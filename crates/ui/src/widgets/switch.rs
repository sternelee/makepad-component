use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // Expose the label-side enum proto to the script heap so DSL can write
    // `label_side: MpLabelSide.Left`.
    let MpLabelSide = set_type_default() do #(MpLabelSide::script_api(vm))
    mod.widgets.MpLabelSide = MpLabelSide

    // Switch toggle component - bezel-style pill track with a sliding knob,
    // plus an optional built-in label (`text` + `label_side`), mirroring the
    // gpui-component Switch (label line sits on the same hit area, track and
    // knob drawn in a single SDF shader like makepad's Toggle pattern).
    // Aligned with gpui-bezel Controls::toggle: 32x18 capsule at Medium,
    // on-state flips to the max-contrast plate (SOLID), thumb = track - 4px
    // muted -> on_solid. Size is driven by `size: MpSize.*` (track height
    // 14..26, width keeps the 32:18 ratio). Disabled fades the whole control
    // to half opacity (gpui fades the grouped control, so the track never
    // bleeds through the thumb).
    set_type_default() do #(DrawMpSwitch::script_shader(vm)){
        ..mod.draw.DrawQuad

        on: 0.0
        hover: 0.0
        focus: 0.0
        disabled: 0.0

        track_off: ELEMENT_ACTIVE
        track_on: SOLID
        thumb_off: TEXT_FAINT
        thumb_on: ON_SOLID
        focus_color: CARET

        // Theme palette (read from Rust to resolve the custom checked color
        // and the disabled label blend)
        c_solid: SOLID
        c_text: TEXT
        c_text_faint: TEXT_FAINT

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

            // Thumb: knob sliding left -> right as `on` goes 0 -> 1
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

            // Disabled: fade the whole control (track + thumb + ring) to
            // half opacity, matching gpui's grouped 0.5 fade.
            let fade = mix(1.0, 0.5, self.disabled)
            let res = sdf.result
            return vec4(res.x, res.y, res.z, res.w * fade)
        }
    }

    // Base switch component
    mod.widgets.MpSwitchBase = #(MpSwitch::register_widget(vm))
    mod.widgets.MpSwitch = set_type_default() do mod.widgets.MpSwitchBase{
        width: Fit
        height: Fit
        flow: Right
        spacing: 8.0
        align: Align{y: 0.5}

        size: MpSize.Medium
        label_side: MpLabelSide.Right

        // Outer hit area (transparent); the capsule itself is draw_track
        draw_bg +: {
            pixel: fn() {
                return vec4(0.0, 0.0, 0.0, 0.0)
            }
        }

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
                    apply: {draw_track: {hover: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_track: {hover: 1.0}}
                }
            }
            on: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.2}}
                    apply: {draw_track: {on: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.2}}
                    apply: {draw_track: {on: 1.0}}
                }
            }
            focus: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_track: {focus: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_track: {focus: 1.0}}
                }
            }
        }
    }
}

/// Which side of the track the optional label renders on
/// (gpui-component `Side` port for the Switch label).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Script, ScriptHook)]
pub enum MpLabelSide {
    Left,
    #[pick]
    #[default]
    Right,
}

/// SDF paint for the switch track + thumb; instances are written from Rust
/// in draw_walk (the 2.0 replacement for apply_over).
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpSwitch {
    #[deref]
    draw_super: DrawQuad,

    // Animator-driven state
    #[live]
    on: f32,
    #[live]
    hover: f32,
    #[live]
    focus: f32,
    // Written from Rust each draw (1.0 = disabled)
    #[live]
    disabled: f32,

    // Paint (track_on is overridden from Rust when a custom color is set)
    #[live]
    track_off: Vec4f,
    #[live]
    track_on: Vec4f,
    #[live]
    thumb_off: Vec4f,
    #[live]
    thumb_on: Vec4f,
    #[live]
    focus_color: Vec4f,

    // Theme palette (baked at apply time; read from Rust)
    #[live]
    c_solid: Vec4f,
    #[live]
    c_text: Vec4f,
    #[live]
    c_text_faint: Vec4f,
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
    #[live]
    draw_track: DrawMpSwitch,
    #[live]
    draw_label: DrawText,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[live]
    size: MpSize,
    #[live]
    label_side: MpLabelSide,

    #[live]
    on: bool,
    #[live]
    disabled: bool,
    #[live]
    text: ArcStringMut,

    /// Runtime override of the checked-track color (gpui `Switch.color`
    /// port). `None` keeps the theme default; written to `draw_bg.track_on`
    /// in draw_walk.
    #[rust]
    color: Option<Vec4f>,

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
            Hit::FingerDown(_) => {
                // Claim key focus on click (bezel focus ring)
                cx.set_key_focus(self.area);
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
        // Track height per size (width keeps the 32:18 capsule ratio).
        let track_h = match self.size {
            MpSize::XSmall => 14.0,
            MpSize::Small => 16.0,
            MpSize::Medium => 18.0,
            MpSize::Large => 22.0,
            MpSize::XLarge => 26.0,
        };
        let track_w = track_h * (32.0 / 18.0);

        // Custom checked color (gpui Switch.color port): only when set, so a
        // DSL-level `draw_track: {track_on: ...}` override keeps working.
        if let Some(color) = self.color {
            self.draw_track.track_on = color;
        }

        // Disabled fades the track+thumb in the shader and mutes the label.
        let disabled = if self.disabled { 1.0f32 } else { 0.0 };
        self.draw_track.disabled = disabled;

        let text = self.draw_track.c_text;
        let faint = self.draw_track.c_text_faint;
        self.draw_label.color = Vec4f {
            x: text.x + (faint.x - text.x) * disabled,
            y: text.y + (faint.y - text.y) * disabled,
            z: text.z + (faint.z - text.z) * disabled,
            w: text.w + (faint.w - text.w) * disabled,
        };

        let has_label = !self.text.as_ref().is_empty();
        self.draw_label.text_style.font_size = self.size.font_size();

        self.draw_bg.begin(cx, walk, self.layout);

        // Track first, then label (Right), or label first (Left); the
        // container's Right flow + spacing places them side by side.
        if has_label && self.label_side == MpLabelSide::Left {
            self.draw_label
                .draw_walk(cx, Walk::fit(), Align::default(), self.text.as_ref());
            self.draw_track.draw_walk(cx, Walk::fixed(track_w, track_h));
        } else {
            self.draw_track.draw_walk(cx, Walk::fixed(track_w, track_h));
            if has_label {
                self.draw_label
                    .draw_walk(cx, Walk::fit(), Align::default(), self.text.as_ref());
            }
        }

        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        if !self.disabled {
            crate::widgets::focus::register(cx, self.widget_uid(), self.area);
        }
        DrawStep::done()
    }
}

impl MpSwitch {
    pub fn is_on(&self) -> bool {
        self.on
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

    pub fn set_text(&mut self, text: &str) {
        self.text.as_mut_empty().push_str(text);
    }

    pub fn label_side(&self) -> MpLabelSide {
        self.label_side
    }

    pub fn set_label_side(&mut self, cx: &mut Cx, side: MpLabelSide) {
        if self.label_side != side {
            self.label_side = side;
            self.redraw(cx);
        }
    }

    /// Override the checked-track color at runtime (gpui `Switch.color`
    /// port). Pass `None` to reset to the theme default.
    pub fn set_color(&mut self, cx: &mut Cx, color: Option<Vec4f>) {
        if self.color != color {
            self.color = color;
            self.redraw(cx);
        }
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

    pub fn set_text(&self, cx: &mut Cx, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(text);
            inner.redraw(cx);
        }
    }

    pub fn set_color(&self, cx: &mut Cx, color: Option<Vec4f>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_color(cx, color);
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
