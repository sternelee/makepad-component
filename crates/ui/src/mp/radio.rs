//! `MpRadio` — one choice among several.
//!
//! The one control in the family that is *not* a toggle. Activating it selects;
//! it never deselects, because "none of these" is a different question from
//! "which of these" and a radio that turns itself off cannot answer the second
//! one. That is the whole of the difference, which is why the widget routes
//! activation to [`MpRadio::select`] rather than to a toggle.
//!
//! ## The group is the caller's
//!
//! There is no `group` field and no registry of siblings. gpui-bezel made the
//! same call: the caller owns the value, so it holds which index is selected and
//! clears the others. A registry inside the widget would have to be told when a
//! sibling unregisters, which is the kind of bookkeeping that leaks when a tree
//! is rebuilt — and the caller already knows, because it is the one holding the
//! answer.
//!
//! ```ignore
//! if let Some(index) = radios.iter().position(|r| r.selected(actions)) {
//!     for (i, r) in radios.iter().enumerate() {
//!         r.set_selected(cx, i == index);
//!     }
//! }
//! ```

use makepad_widgets::*;

use makepad_theme::ControlSize;

use crate::mp::control;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    set_type_default() do #(DrawMpRadio::script_shader(vm)){
        ..mod.draw.DrawQuad

        hover: 0.0
        press: 0.0
        focus: 0.0
        disabled: 0.0
        // How far the dot has grown in, 0..1.
        checked: 0.0

        fill: #x00000000
        fill_hover: #x00000000
        fill_press: #x00000000
        dot_color: #x00000000
        border_color: #x00000000
        border_width: 1.0
        ring_color: #x00000000
        ring_width: 2.0

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let b = self.rect_size.y
            let r = b * 0.5
            let bw = self.border_width
            let c = vec2(r, r)

            // The well. A radio's plate does not fill when selected — the dot
            // is what carries the state — because a filled circle among filled
            // circles is harder to scan than a dot among outlines.
            let rested = mix(self.fill, self.fill_hover, self.hover)
            let pressed = mix(rested, self.fill_press, self.press)
            let alpha = pressed.w * (1.0 - self.disabled * 0.55)

            sdf.circle(c.x, c.y, r - bw)
            sdf.fill_keep(vec4(pressed.x, pressed.y, pressed.z, alpha))

            // The edge carries both the resting outline and the focus ring, and
            // takes the dot's ink as the dot arrives — so a selected radio is a
            // solid disc rather than an outline with something in it.
            let edge = mix(self.border_color, self.dot_color, self.checked * 0.55)
            let ring_w = mix(bw, self.ring_width, self.focus)
            let ring_c = mix(edge, self.ring_color, self.focus)
            sdf.stroke(ring_c, ring_w)

            // The dot: a circle that grows from nothing as `checked` arrives.
            // Growing rather than fading, because a dot that fades in at full
            // size reads as a rendering glitch at this scale.
            let max_dot = (r - bw) * 0.52
            sdf.circle(c.x, c.y, max_dot * self.checked)
            let dot = self.dot_color
            sdf.fill(vec4(dot.x, dot.y, dot.z, dot.w * (1.0 - self.disabled * 0.55)))

            return sdf.result
        }
    }

    mod.mp.MpRadioBase = #(MpRadio::register_widget(vm))

    mod.mp.MpRadio = set_type_default() do mod.mp.MpRadioBase{
        width: Fit
        height: mod.mpc.layout.control.regular.height
        flow: Right
        align: Align{x: 0.0, y: 0.5}

        selected: false
        disabled: false
        control: mod.mpc.ControlSize.Regular
        label_gap: 8.0

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

        text: "Radio"
    }

    mod.mp.MpRadioSmall = mod.mp.MpRadio{
        control: mod.mpc.ControlSize.Small
        height: mod.mpc.layout.control.small.height
    }
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpRadio {
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
    dot_color: Vec4f,
    #[live]
    border_color: Vec4f,
    #[live]
    border_width: f32,
    #[live]
    ring_color: Vec4f,
    #[live]
    ring_width: f32,
}

/// What a radio reports.
#[derive(Clone, Debug, Default)]
pub enum MpRadioAction {
    /// This radio was chosen. Emitted only on a change *into* selected, so a
    /// caller can treat it as "the user picked this" without filtering.
    Selected,
    #[default]
    None,
}

// No `ScriptHook` derive: this widget implements it by hand to put the
// animator where its initial value says it belongs.
#[derive(Script, Widget, Animator)]
pub struct MpRadio {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[apply_default]
    animator: Animator,

    #[live]
    selected: bool,
    #[live]
    disabled: bool,
    #[live]
    control: ControlSize,
    #[live]
    label_gap: f64,

    #[redraw]
    #[live]
    draw_bg: DrawMpRadio,
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

impl MpRadio {
    pub fn is_selected(&self) -> bool {
        self.selected
    }

    /// Whether this radio reported being chosen in this batch.
    pub fn selected(&self, actions: &Actions) -> bool {
        crate::mp::action::is::<MpRadioAction>(self.widget_uid(), actions, |a| {
            matches!(a, MpRadioAction::Selected)
        })
    }

    /// The animator group whose two states are "chosen" and "not".
    pub const CHECKED: &'static [LiveId] = ids!(checked);

    /// Choose this radio.
    ///
    /// Idempotent in the sense that matters: selecting an already-selected
    /// radio animates nothing and reports nothing, because the caller's
    /// "clear the others" pass runs on every click including one on the
    /// already-chosen item.
    pub fn select(&mut self, cx: &mut Cx) {
        if self.selected {
            return;
        }
        self.selected = true;
        self.animator_play(cx, ids!(checked.on));
        cx.widget_action(self.widget_uid(), MpRadioAction::Selected);
        self.redraw(cx);
    }

    /// Set the value directly, without reporting.
    ///
    /// This is the "clear the others" half of a group: it has to be silent,
    /// because a radio that reported its own deselection would make every
    /// caller filter the action down to the one that was chosen.
    pub fn set_selected(&mut self, cx: &mut Cx, selected: bool) {
        if self.selected == selected {
            return;
        }
        self.selected = selected;
        self.animator_play(
            cx,
            if selected {
                ids!(checked.on)
            } else {
                ids!(checked.off)
            },
        );
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

impl ScriptHook for MpRadio {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        let selected = self.selected;
        vm.with_cx_mut(|cx| {
            control::init_checked(&mut self.animator, cx, selected);
        });
    }
}

impl Widget for MpRadio {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let signals = control::handle(&mut self.animator, cx, event, self.area);
        if signals.redraw {
            self.redraw(cx);
        }
        if self.disabled || !signals.activate {
            return;
        }
        // Select, never toggle.
        self.select(cx);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let theme = makepad_theme::Theme::of(cx.cx);
        let p = &theme.paint;
        let size = self.control;
        let metrics = size.metrics();

        self.layout.padding = Inset {
            left: size.height() as f64 + self.label_gap,
            right: 0.0,
            top: 0.0,
            bottom: 0.0,
        };

        self.draw_bg.fill = p.input_bg;
        self.draw_bg.fill_hover = control::plates::hover(p.input_bg, p);
        self.draw_bg.fill_press = control::plates::active(p.input_bg, p);
        self.draw_bg.dot_color = p.solid;
        self.draw_bg.border_color = p.border_strong;
        self.draw_bg.ring_color = p.caret;

        self.draw_text.text_style.font_size = metrics.size();
        self.draw_text.text_style.line_spacing = metrics.leading;
        // Same rule as the checkbox: a chosen option reads at full strength and
        // the rest recede, so a group is scannable without reading the labels.
        self.draw_text.color = if self.selected { p.text } else { p.text_muted };

        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_text
            .draw_walk(cx, Walk::fit(), Align::default(), self.text.as_ref());
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        control::register(cx, self.widget_uid(), self.area, self.disabled);
        DrawStep::done()
    }
}

impl MpRadioRef {
    pub fn is_selected(&self) -> bool {
        self.borrow().is_some_and(|inner| inner.is_selected())
    }

    pub fn selected(&self, actions: &Actions) -> bool {
        self.borrow().is_some_and(|inner| inner.selected(actions))
    }

    pub fn set_selected(&self, cx: &mut Cx, selected: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_selected(cx, selected);
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
