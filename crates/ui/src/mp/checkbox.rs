//! `MpCheckbox` — a boolean control, and the first user of [`control`].
//!
//! The whole widget is 120 lines of behaviour and one shader. Everything it
//! shares with the switch, the radio and the toggle — hover, press, focus,
//! disabled, the keyboard contract, the Tab order — comes from
//! [`control::handle`], so the only thing written here is what makes it a
//! *checkbox*: it has two independent states, and activating it flips them.
//!
//! Contrast the v2 checkbox: 446 lines, of which the thirty-line hit block was
//! one of five near-identical copies.
//!
//! ## One draw, not two
//!
//! The box is square and sits at the leading edge of the widget's own rect, so
//! the shader can derive it from `rect_size` and the widget needs no child
//! `View` for the box. The label's inset is the same number the shader uses,
//! both read from the control ladder — `box()` below is the one place it is
//! written.

use makepad_widgets::*;

use makepad_theme::ControlSize;

use crate::mp::control;


script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    set_type_default() do #(DrawMpCheckbox::script_shader(vm)){
        ..mod.draw.DrawQuad

        // Plain values, not `instance(..)`: on a custom `script_shader` the
        // storage class comes from the Rust struct's `#[live]` fields.
        hover: 0.0
        press: 0.0
        focus: 0.0
        disabled: 0.0
        // How far the tick has drawn in, 0..1.
        checked: 0.0

        fill: #x00000000
        fill_hover: #x00000000
        fill_press: #x00000000
        // The tick's ink, and the plate it sits on when the box is filled.
        check_color: #x00000000
        checked_fill: #x00000000
        border_color: #x00000000
        border_width: 1.0
        radius: 4.0
        ring_color: #x00000000
        ring_width: 2.0

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            // The box is square and as tall as the row, so its geometry falls
            // out of the widget's own height.
            let b = self.rect_size.y
            let bw = self.border_width

            // The plate. Unchecked it is a quiet well; checked it takes the
            // prominent ink, which is the same signal a Prominent button makes
            // — "this is decided".
            let rested = mix(self.fill, self.fill_hover, self.hover)
            let pressed = mix(rested, self.fill_press, self.press)
            let plate = mix(pressed, self.checked_fill, self.checked)
            let alpha = plate.w * (1.0 - self.disabled * 0.55)

            sdf.box(bw, bw, b - bw * 2.0, b - bw * 2.0, max(1.0, self.radius))
            sdf.fill_keep(vec4(plate.x, plate.y, plate.z, alpha))

            // The edge. When checked the plate is the ink, so the edge is the
            // ink too and the box gains a pixel of size rather than a seam.
            let edge = mix(self.border_color, self.checked_fill, self.checked)
            let ring_w = mix(bw, self.ring_width, self.focus)
            let ring_c = mix(edge, self.ring_color, self.focus)
            sdf.stroke(ring_c, ring_w)

            // The tick: a closed polygon, because a stroked open path in an
            // SDF outlines the polyline rather than following it.
            let t = max(1.5, b * 0.09)
            let cx0 = b * 0.22
            let cy0 = b * 0.53
            let cx1 = b * 0.41
            let cy1 = b * 0.71
            let cx2 = b * 0.79
            let cy2 = b * 0.30
            sdf.move_to(cx0, cy0)
            sdf.line_to(cx1, cy1)
            sdf.line_to(cx2, cy2)
            sdf.line_to(cx2 - t, cy2 - t)
            sdf.line_to(cx1, cy1 - t * 2.0)
            sdf.line_to(cx0 + t, cy0 - t)
            sdf.close_path()
            // The tick's own ink, faded by how far it has drawn in.
            let tick = mix(self.check_color, self.check_color, 1.0)
            sdf.fill(vec4(tick.x, tick.y, tick.z, tick.w * self.checked))

            return sdf.result
        }
    }

    mod.mp.MpCheckboxBase = #(MpCheckbox::register_widget(vm))

    mod.mp.MpCheckbox = set_type_default() do mod.mp.MpCheckboxBase{
        width: Fit
        height: mod.mpc.layout.control.regular.height
        flow: Right
        align: Align{x: 0.0, y: 0.5}

        checked: false
        disabled: false
        control: mod.mpc.ControlSize.Regular
        // The label's inset past the box. `2.0` is the sibling gap divided by
        // four — the box is a control edge, not a peer, so it sits tighter than
        // `space`; the widget recomputes this at paint from the rung.
        label_gap: 8.0

        // The four shared tracks, then the one that makes this a checkbox.
        animator: mod.mp.ControlAnimator{
            checked: {
                default: @off
                off: AnimatorState{
                    ease: mod.motion.fade_quick.ease
                    from: {all: Forward{duration: mod.motion.fade_quick.duration}}
                    apply: {draw_bg: {checked: 0.0}}
                }
                on: AnimatorState{
                    ease: mod.motion.fade_quick.ease
                    from: {all: Forward{duration: mod.motion.fade_quick.duration}}
                    apply: {draw_bg: {checked: 1.0}}
                }
            }
        }

        draw_text +: {
            text_style: mod.mpc.type.body
            color: #x00000000
        }

        text: "Checkbox"
    }

    mod.mp.MpCheckboxSmall = mod.mp.MpCheckbox{
        control: mod.mpc.ControlSize.Small
        height: mod.mpc.layout.control.small.height
    }
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpCheckbox {
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
    fill: Vec4f,
    #[live]
    fill_hover: Vec4f,
    #[live]
    fill_press: Vec4f,
    #[live]
    check_color: Vec4f,
    #[live]
    checked_fill: Vec4f,
    #[live]
    border_color: Vec4f,
    #[live]
    border_width: f32,
    #[live]
    radius: f32,
    #[live]
    ring_color: Vec4f,
    #[live]
    ring_width: f32,
}

/// What a checkbox reports.
#[derive(Clone, Debug, Default)]
pub enum MpCheckboxAction {
    /// The value changed, carrying it.
    Changed(bool),
    #[default]
    None,
}

// No `ScriptHook` derive: this widget implements it by hand to put the
// animator where its initial value says it belongs.
#[derive(Script, Widget, Animator)]
pub struct MpCheckbox {
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
    #[live]
    label_gap: f64,

    #[redraw]
    #[live]
    draw_bg: DrawMpCheckbox,
    #[live]
    draw_text: DrawText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[live]
    text: ArcStringMut,

    #[rust]
    area: Area,
}

impl MpCheckbox {
    /// The box's side, which is the row's own height.
    ///
    /// Public because it is also the label's inset: the shader derives the box
    /// from `rect_size.y` and the widget derives the text offset from this, and
    /// the two agreeing is the whole reason the box draws at all.
    pub fn box_side(&self) -> f64 {
        self.control.height() as f64
    }

    pub fn checked(&self, actions: &Actions) -> Option<bool> {
        crate::mp::action::first::<MpCheckboxAction>(self.widget_uid(), actions).and_then(|a| {
            match a {
                MpCheckboxAction::Changed(v) => Some(*v),
                MpCheckboxAction::None => None,
            }
        })
    }

    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// The animator group whose two states are "ticked" and "not".
    ///
    /// Public because the switch and the radio declare the same group with the
    /// same meaning: one name for one idea across the family.
    pub const CHECKED: &'static [LiveId] = ids!(checked);

    /// Set the value, animating the tick and reporting the change.
    ///
    /// The single mutation path: a click, a key, a programmatic `set_checked`
    /// and a revert all come through here, so the animator can never disagree
    /// with the field.
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
        cx.widget_action(self.widget_uid(), MpCheckboxAction::Changed(checked));
        self.redraw(cx);
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

    pub fn set_text(&mut self, cx: &mut Cx, text: &str) {
        self.text.as_mut_empty().push_str(text);
        self.redraw(cx);
    }
}

impl ScriptHook for MpCheckbox {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        // A checkbox built as `checked: true` has to *paint* checked: the
        // field is true from the start but the animator is not, and the paint
        // reads the animator.
        let checked = self.checked;
        let disabled = self.disabled;
        vm.with_cx_mut(|cx| {
            control::init_checked(&mut self.animator, cx, checked);
            // ...and the same for `disabled`, which the v2 set tracked in an
            // animator field it never read at construction either.
            control::init_disabled(&mut self.animator, cx, disabled);
        });
    }
}

impl Widget for MpCheckbox {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let signals = control::handle(&mut self.animator, cx, event, self.area);
        if signals.redraw {
            self.redraw(cx);
        }
        if self.disabled || !signals.activate {
            return;
        }
        self.set_checked(cx, !self.checked);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let theme = makepad_theme::Theme::of(cx.cx);
        let p = &theme.paint;
        let size = self.control;
        let metrics = size.metrics();

        self.layout.padding = Inset {
            left: self.box_side() + self.label_gap,
            right: 0.0,
            top: 0.0,
            bottom: 0.0,
        };

        // The box: a quiet well when unchecked, the prominent ink when checked.
        self.draw_bg.fill = p.input_bg;
        self.draw_bg.fill_hover = control::plates::hover(self.draw_bg.fill, p);
        self.draw_bg.fill_press = control::plates::active(self.draw_bg.fill, p);
        self.draw_bg.checked_fill = p.solid;
        self.draw_bg.check_color = p.on_solid;
        self.draw_bg.border_color = p.border_strong;
        self.draw_bg.ring_color = p.caret;
        self.draw_bg.radius = size.radius() as f32;

        self.draw_text.text_style.font_size = metrics.size();
        self.draw_text.text_style.line_spacing = metrics.leading;
        // A checked box's label is body ink; an unchecked one is muted, so a
        // form reads at a glance without counting ticks.
        self.draw_text.color = if self.checked { p.text } else { p.text_muted };

        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_text
            .draw_walk(cx, Walk::fit(), Align::default(), self.text.as_ref());
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        control::register(cx, self.widget_uid(), self.area, self.disabled);
        DrawStep::done()
    }
}

impl MpCheckboxRef {
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

    pub fn set_text(&self, cx: &mut Cx, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(cx, text);
        }
    }
}
