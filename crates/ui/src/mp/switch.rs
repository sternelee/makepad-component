//! `MpSwitch` — a boolean control whose two states are *positions*.
//!
//! A checkbox and a switch hold the same value and are not the same control. A
//! checkbox says "include this"; a switch says "this thing is on", and it says
//! it by moving, which is why a switch is committed instantly and a checkbox in
//! a form is often not. That difference is in the widget's *use*, so it is not
//! a difference in the code — both go through [`control::handle`], and both
//! report [`Changed`](MpSwitchAction::Changed) the same way.
//!
//! The one thing a switch has and a checkbox does not is direction: the knob
//! travels, so the checked state is a number the layout of the shader depends
//! on rather than only a colour.
//!
//! [`control::handle`]: crate::mp::control::handle

use makepad_widgets::*;

use makepad_theme::ControlSize;

use crate::mp::control;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    set_type_default() do #(DrawMpSwitch::script_shader(vm)){
        ..mod.draw.DrawQuad

        hover: 0.0
        press: 0.0
        focus: 0.0
        disabled: 0.0
        // How far the knob has travelled, 0..1.
        checked: 0.0

        off_fill: #x00000000
        off_fill_hover: #x00000000
        on_fill: #x00000000
        on_fill_hover: #x00000000
        // The knob's two tones. The shader mixes them by `checked`, the same
        // number that places the knob, so its colour and its position can never
        // disagree — which they did while the colour came from the Rust bool
        // and the position from the animator.
        knob_off: #x00000000
        knob_on: #x00000000
        border_color: #x00000000
        border_width: 1.0
        ring_color: #x00000000
        ring_width: 2.0

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let w = self.rect_size.x
            let h = self.rect_size.y
            let r = h * 0.5
            let bw = self.border_width

            // The track. Its two fills are different tones rather than one
            // tone at two alphas: an off switch is a *well* and an on switch is
            // a *plate*, and a well cannot be reached by lightening a plate.
            let off = mix(self.off_fill, self.off_fill_hover, self.hover)
            let on = mix(self.on_fill, self.on_fill_hover, self.hover)
            let track = mix(off, on, self.checked)
            let alpha = track.w * (1.0 - self.disabled * 0.55)

            sdf.box(bw, bw, w - bw * 2.0, h - bw * 2.0, r)
            sdf.fill_keep(vec4(track.x, track.y, track.z, alpha))

            // The ring sits on the track's own outline, so it reads as the
            // switch being focused rather than as a box around it.
            let ring_w = mix(bw, self.ring_width, self.focus)
            let ring_c = mix(self.border_color, self.ring_color, self.focus)
            sdf.stroke(ring_c, ring_w)

            // The knob. Its inset is proportional to the track's height, so the
            // switch holds its proportions at every control size rather than
            // only at the one it was drawn for.
            let pad = max(1.5, h * 0.12)
            let k = (h - pad * 2.0) * 0.5
            let cx = mix(pad + k, w - pad - k, self.checked)
            sdf.circle(cx, h * 0.5, k)
            let knob = mix(self.knob_off, self.knob_on, self.checked)
            sdf.fill(vec4(knob.x, knob.y, knob.z, knob.w * (1.0 - self.disabled * 0.55)))

            return sdf.result
        }
    }

    mod.mp.MpSwitchBase = #(MpSwitch::register_widget(vm))

    // A switch has no label: it is a state indicator, and what it controls is
    // named by a label beside it. A labelled switch is a form row, which is a
    // different component.
    mod.mp.MpSwitch = set_type_default() do mod.mp.MpSwitchBase{
        width: 36
        height: mod.mpc.layout.control.regular.height

        checked: false
        disabled: false
        control: mod.mpc.ControlSize.Regular

        animator: mod.mp.ControlAnimator{
            checked: {
                default: @off
                off: AnimatorState{
                    ease: mod.motion.hover_fade.ease
                    from: {all: Forward{duration: mod.motion.hover_fade.duration}}
                    apply: {draw_bg: {checked: 0.0}}
                }
                on: AnimatorState{
                    ease: mod.motion.hover_fade.ease
                    from: {all: Forward{duration: mod.motion.hover_fade.duration}}
                    apply: {draw_bg: {checked: 1.0}}
                }
            }
        }
    }

    mod.mp.MpSwitchSmall = mod.mp.MpSwitch{
        width: 28
        height: mod.mpc.layout.control.small.height
        control: mod.mpc.ControlSize.Small
    }
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpSwitch {
    #[deref]
    draw_super: DrawQuad,

    #[live]
    hover: f32,
    #[live]
    press: f32,
    #[live]
    focus: f32,
    #[live]
    disabled: f32,
    #[live]
    checked: f32,

    #[live]
    off_fill: Vec4f,
    #[live]
    off_fill_hover: Vec4f,
    #[live]
    on_fill: Vec4f,
    #[live]
    on_fill_hover: Vec4f,
    #[live]
    knob_off: Vec4f,
    #[live]
    knob_on: Vec4f,
    #[live]
    border_color: Vec4f,
    #[live]
    border_width: f32,
    #[live]
    ring_color: Vec4f,
    #[live]
    ring_width: f32,
}

/// What a switch reports.
#[derive(Clone, Debug, Default)]
pub enum MpSwitchAction {
    /// The value changed, carrying it.
    Changed(bool),
    #[default]
    None,
}

// No `ScriptHook` derive: this widget implements it by hand to put the
// animator where its initial value says it belongs.
#[derive(Script, Widget, Animator)]
pub struct MpSwitch {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[apply_default]
    animator: Animator,

    #[live]
    checked: bool,
    #[live]
    disabled: bool,
    #[live]
    control: ControlSize,

    #[redraw]
    #[live]
    draw_bg: DrawMpSwitch,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[rust]
    area: Area,
}

impl MpSwitch {
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    pub fn checked(&self, actions: &Actions) -> Option<bool> {
        crate::mp::action::first::<MpSwitchAction>(self.widget_uid(), actions).and_then(|a| {
            match a {
                MpSwitchAction::Changed(v) => Some(*v),
                MpSwitchAction::None => None,
            }
        })
    }

    /// The animator group whose two states are "on" and "off".
    ///
    /// The same name and the same meaning as the checkbox's, so a reader who
    /// knows one control's animator knows all of them.
    pub const CHECKED: &'static [LiveId] = ids!(checked);

    pub fn set_checked(&mut self, cx: &mut Cx, checked: bool) {
        if self.checked == checked {
            return;
        }
        self.checked = checked;
        self.animator_play(
            cx,
            if checked {
                ids!(checked.on)
            } else {
                ids!(checked.off)
            },
        );
        cx.widget_action(self.widget_uid(), MpSwitchAction::Changed(checked));
        self.redraw(cx);
    }

    pub fn toggle(&mut self, cx: &mut Cx) {
        self.set_checked(cx, !self.checked);
    }

    pub fn set_disabled(&mut self, cx: &mut Cx, disabled: bool) {
        if self.disabled == disabled {
            return;
        }
        self.disabled = disabled;
        control::set_disabled(&mut self.animator, cx, disabled);
        self.redraw(cx);
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

impl ScriptHook for MpSwitch {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        let checked = self.checked;
        vm.with_cx_mut(|cx| {
            control::init_checked(&mut self.animator, cx, checked);
        });
    }
}

impl Widget for MpSwitch {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let signals = control::handle(&mut self.animator, cx, event, self.area);
        if signals.redraw {
            self.redraw(cx);
        }
        if self.disabled || !signals.activate {
            return;
        }
        self.toggle(cx);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let theme = makepad_theme::Theme::of(cx.cx);
        let p = &theme.paint;

        // Off is a well, on is the prominent ink — the same two tones the
        // checkbox uses, so a form with both reads as one system.
        self.draw_bg.off_fill = p.input_bg;
        self.draw_bg.off_fill_hover = control::plates::hover(p.input_bg, p);
        self.draw_bg.on_fill = p.solid;
        self.draw_bg.on_fill_hover = control::plates::solid_hover(p.solid, p.on_solid);
        // The knob inverts with the plate, so it is always the thing that
        // contrasts most with what it sits on.
        self.draw_bg.knob_off = p.text_muted;
        self.draw_bg.knob_on = p.on_solid;
        self.draw_bg.border_color = p.border_strong;
        self.draw_bg.ring_color = p.caret;

        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        control::register(cx, self.widget_uid(), self.area, self.disabled);
        DrawStep::done()
    }
}

impl MpSwitchRef {
    pub fn is_checked(&self) -> bool {
        self.borrow().is_some_and(|inner| inner.is_checked())
    }

    pub fn checked(&self, actions: &Actions) -> Option<bool> {
        self.borrow().and_then(|inner| inner.checked(actions))
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
}
