//! `MpButton` — a labeled button in one of four shipped looks.
//!
//! The v2 button is the clearest example of what the old architecture cost:
//!
//! | | v2 | here |
//! |---|---|---|
//! | Palette instance fields in the shader | 16 (`c_solid` … `border`), existing only so Rust could read tokens back out | **0** — `Theme::of(cx)` |
//! | Looks | 9 (`Default`, `Prominent`, `Accent`, `Secondary`, `Ghost`, `Outline`, `Destructive`, `Link`, `Text`) | **4** — bezel's closed set |
//! | Durations | literal `0.15` / `0.09` in the animator block | `mod.motion.hover_fade` / `press` |
//! | Action read | `find_widget_action(uid).cast()` — silently false behind any other action for the same uid | [`action::is`] |
//! | Size | its own `MpSize` ladder with its own font sizes | `ControlSize`, the same ladder the theme exports |
//!
//! The nine looks collapsed because most of them were not looks: `Accent` was
//! `Prominent` in another hue, which is a [`Brand`](makepad_theme::Brand)
//! decision rather than a button variant; `Secondary` was `Default`; `Link` and
//! `Text` were `Ghost` with tighter padding, which is a content decision the
//! caller already owns by writing `padding`.

use makepad_widgets::*;

use makepad_theme::ControlSize;

use crate::mp::{action, control};

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.motion.*

    // The closed set of shipped looks. An enum rather than knobs: a caller
    // picks one, and free-form radius, colour and padding never become API.
    mod.mp.ButtonStyle = set_type_default() do #(MpButtonStyle::script_api(vm))

    // The plate. Every colour here is written from Rust each paint, read from
    // `Theme::of(cx)` — so switching appearance needs a redraw rather than a
    // whole-heap script re-apply.
    set_type_default() do #(DrawMpButton::script_shader(vm)){
        ..mod.draw.DrawQuad

        // Plain values, not `instance(..)`: on a custom `script_shader` the
        // shader compiler derives the storage class from the Rust struct's
        // `#[live]` fields, and `instance()` there asks for an object where a
        // number is expected. It is only for *overriding* an inherited field's
        // storage class — which is what `draw_bg +: {color: instance(..)}` on a
        // `RoundedView` is doing.
        //
        // Animator-driven state, all 0..1.
        hover: 0.0
        press: 0.0
        focus: 0.0
        disabled: 0.0

        // Resolved paint, written from Rust every paint.
        fill: #x00000000
        fill_hover: #x00000000
        fill_press: #x00000000
        border_color: #x00000000
        border_width: 0.0
        radius: 8.0
        // The focus ring's colour — the theme's caret, so a focused control and
        // a caret are the same signal.
        ring_color: #x00000000

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let bw = self.border_width
            sdf.box(
                bw
                bw
                self.rect_size.x - bw * 2.0
                self.rect_size.y - bw * 2.0
                max(1.0, self.radius)
            )

            // Hover and press are sequential mixes of the same plate, so a
            // press that begins mid-hover lands where the hover had got to
            // rather than snapping back to rest.
            let rested = mix(self.fill, self.fill_hover, self.hover)
            let down = mix(rested, self.fill_press, self.press)
            // Disabled cuts the plate's coverage rather than swapping its
            // colour, so a variant keeps its identity while unavailable.
            let alpha = down.w * (1.0 - self.disabled * 0.55)
            sdf.fill_keep(vec4(down.x, down.y, down.z, alpha))

            // The focus ring replaces the border rather than sitting beside it:
            // two hairlines a pixel apart read as a mistake.
            let ring_w = mix(bw, 2.0, self.focus)
            let ring_c = mix(self.border_color, self.ring_color, self.focus)
            if (ring_w > 0.0) {
                sdf.stroke(ring_c, ring_w)
            }

            return sdf.result
        }
    }

    mod.mp.MpButtonBase = #(MpButton::register_widget(vm))

    mod.mp.MpButton = set_type_default() do mod.mp.MpButtonBase{
        width: Fit
        height: mod.mpc.layout.control.regular.height
        flow: Right
        spacing: 6
        align: Align{x: 0.5, y: 0.5}

        style: mod.mp.ButtonStyle.Default
        control: mod.mpc.ControlSize.Regular

        draw_text +: {
            text_style: mod.mpc.type.body
            color: #x00000000
        }

        text: "Button"
    }

    // The size variants read the same `mod.mpc.layout` the widget does, so a
    // branded radius or a moved line box reaches both without a second edit.
    mod.mp.MpButtonSmall = mod.mp.MpButton{
        control: mod.mpc.ControlSize.Small
        height: mod.mpc.layout.control.small.height
    }
    mod.mp.MpButtonLarge = mod.mp.MpButton{
        control: mod.mpc.ControlSize.Large
        height: mod.mpc.layout.control.large.height
    }
}

/// The shipped looks. A closed set: adding a fifth means deciding what it *is*,
/// not what numbers it happens to paint.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Script, ScriptHook)]
pub enum MpButtonStyle {
    /// The everyday plate: a raised fill and a hairline edge. The default
    /// action of a form.
    #[pick]
    #[default]
    #[live]
    Default,
    /// The maximum-contrast plate — the primary action of a view. One per view,
    /// because its whole job is to be the only one.
    #[live]
    Prominent,
    /// Quiet text on a translucent wash. The dismiss, the cancel, the
    /// third action.
    #[live]
    Ghost,
    /// The muted red fill — it carries the destructive semantics in its paint
    /// rather than in its label.
    #[live]
    Destructive,
}

/// The paint one style resolves to, before the animator mixes it.
///
/// Produced by [`MpButton::style`], which reads the theme. Nothing here is
/// stored on the widget, so a theme change is visible on the next redraw with
/// no invalidation step.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Plate {
    fill: Vec4f,
    fill_hover: Vec4f,
    fill_press: Vec4f,
    border: Vec4f,
    border_width: f32,
    ink: Vec4f,
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpButton {
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
    fill: Vec4f,
    #[live]
    fill_hover: Vec4f,
    #[live]
    fill_press: Vec4f,
    #[live]
    border_color: Vec4f,
    #[live]
    border_width: f32,
    #[live]
    radius: f32,
    #[live]
    ring_color: Vec4f,
}

// No `ScriptHook` derive: it is implemented by hand to seat `disabled`.
#[derive(Script, Widget, Animator)]
pub struct MpButton {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[apply_default]
    animator: Animator,

    #[live]
    style: MpButtonStyle,
    #[live]
    control: ControlSize,
    #[live]
    text: ArcStringMut,
    #[live]
    disabled: bool,

    #[redraw]
    #[live]
    draw_bg: DrawMpButton,
    #[live]
    draw_text: DrawText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[rust]
    area: Area,
}

/// What a button reports to whoever is listening.
#[derive(Clone, Debug, Default)]
pub enum MpButtonAction {
    Clicked,
    Pressed,
    Released,
    #[default]
    None,
}

impl MpButton {
    /// Resolve the style against the theme in force.
    ///
    /// Read fresh every paint. The v2 button read these out of sixteen shader
    /// instance fields that had been baked at script-apply time, which is what
    /// made an appearance switch a whole-heap re-apply instead of a redraw.
    fn plate(&self, cx: &Cx) -> Plate {
        let theme = makepad_theme::Theme::of(cx);
        let p = &theme.paint;
        let transparent = Vec4f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        };
        match self.style {
            MpButtonStyle::Default => Plate {
                fill: p.surface_raised,
                fill_hover: p.surface_raised_hover,
                fill_press: p.element_active,
                border: p.border,
                border_width: 1.0,
                ink: p.text,
            },
            MpButtonStyle::Prominent => Plate {
                fill: p.solid,
                fill_hover: crate::mp::control::plates::solid_hover(p.solid, p.on_solid),
                fill_press: crate::mp::control::plates::solid_press(p.solid, p.on_solid),
                border: transparent,
                border_width: 0.0,
                ink: p.on_solid,
            },
            MpButtonStyle::Ghost => Plate {
                fill: transparent,
                fill_hover: p.element_hover,
                fill_press: p.element_active,
                border: transparent,
                border_width: 0.0,
                ink: p.text_muted,
            },
            MpButtonStyle::Destructive => Plate {
                // `danger_strong` is the plate the palette verified a label
                // against; `danger` is the ink.
                fill: p.danger_strong,
                fill_hover: crate::mp::control::plates::solid_hover(p.danger_strong, p.on_solid),
                fill_press: crate::mp::control::plates::solid_press(p.danger_strong, p.on_solid),
                border: transparent,
                border_width: 0.0,
                ink: p.on_solid,
            },
        }
    }

    pub fn clicked(&self, actions: &Actions) -> bool {
        action::is::<MpButtonAction>(self.widget_uid(), actions, |a| {
            matches!(a, MpButtonAction::Clicked)
        })
    }

    pub fn pressed(&self, actions: &Actions) -> bool {
        action::is::<MpButtonAction>(self.widget_uid(), actions, |a| {
            matches!(a, MpButtonAction::Pressed)
        })
    }

    pub fn set_text(&mut self, cx: &mut Cx, text: &str) {
        self.text.as_mut_empty().push_str(text);
        self.redraw(cx);
    }

    pub fn style(&self) -> MpButtonStyle {
        self.style
    }

    pub fn set_style(&mut self, cx: &mut Cx, style: MpButtonStyle) {
        self.style = style;
        self.redraw(cx);
    }

    pub fn control(&self) -> ControlSize {
        self.control
    }

    pub fn set_control(&mut self, cx: &mut Cx, control: ControlSize) {
        self.control = control;
        // The caller named a rung; the height it implies is the DSL's job only
        // because a Makepad walk cannot be recomputed here without clobbering a
        // height the caller set on purpose. The DSL variants read the same
        // `mod.mpc.layout.control` entry this does, so they cannot drift.
        self.redraw(cx);
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn set_disabled(&mut self, cx: &mut Cx, disabled: bool) {
        if self.disabled == disabled {
            return;
        }
        self.disabled = disabled;
        self.animator_toggle(
            cx,
            disabled,
            Animate::Yes,
            ids!(disabled.on),
            ids!(disabled.off),
        );
        self.redraw(cx);
    }
}

impl ScriptHook for MpButton {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        // Same hole the checkbox had: a button built `disabled: true` paints
        // enabled, because the field is true from construction while the
        // animator is still in its default `off` state.
        let disabled = self.disabled;
        vm.with_cx_mut(|cx| {
            crate::mp::control::init_disabled(&mut self.animator, cx, disabled);
        });
    }
}

impl Widget for MpButton {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        // The shared contract, like every other control in the crate. This
        // widget was written before `control.rs` existed and kept its own copy
        // of the hit handling — the last duplication of the thing that module
        // was created to remove, and the reason a tooltip could not treat a
        // button the way it treats a checkbox.
        let signals = control::handle(&mut self.animator, cx, event, self.area);
        // Tell whoever owns the one tooltip that this control is under the
        // pointer. A control cannot own a tooltip itself; see `mp/tooltip.rs`.
        control::emit_hover(cx, self.widget_uid(), signals);
        // Tell whoever owns the one tooltip that this control is under the
        // pointer. A control cannot own a tooltip itself; see `mp/tooltip.rs`.
        control::emit_hover(cx, self.widget_uid(), signals);
        if signals.redraw {
            self.redraw(cx);
        }
        if self.disabled {
            return;
        }

        // `Pressed` and `Released` bracket the gesture; `activate` is the press
        // that landed, or Enter/Space while focused — which is why there is no
        // separate keyboard block here any more.
        if signals.down {
            cx.widget_action(self.widget_uid(), MpButtonAction::Pressed);
        }
        if signals.up {
            cx.widget_action(self.widget_uid(), MpButtonAction::Released);
        }
        if signals.activate {
            cx.widget_action(self.widget_uid(), MpButtonAction::Clicked);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let plate = self.plate(cx.cx);
        let size = self.control;
        let metrics = size.metrics();
        let theme = makepad_theme::Theme::of(cx.cx);

        // Layout from the ladder, not from a private size table: the height is
        // the role's line box plus the padding the size names, so moving the
        // type ladder moves the control with it.
        self.layout.padding = Inset {
            left: size.pad_x() as f64,
            right: size.pad_x() as f64,
            top: size.pad_y() as f64,
            bottom: size.pad_y() as f64,
        };

        self.draw_bg.radius = size.radius() as f32;
        self.draw_bg.fill = plate.fill;
        self.draw_bg.fill_hover = plate.fill_hover;
        self.draw_bg.fill_press = plate.fill_press;
        self.draw_bg.border_color = plate.border;
        self.draw_bg.border_width = plate.border_width;
        self.draw_bg.ring_color = theme.paint.caret;

        self.draw_text.text_style.font_size = metrics.size();
        self.draw_text.text_style.line_spacing = metrics.leading;
        self.draw_text.color = plate.ink;

        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_text.draw_walk(
            cx,
            Walk::fit(),
            Align::default(),
            self.text.as_ref(),
        );
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();

        if !self.disabled {
            crate::widgets::focus::register(cx, self.widget_uid(), self.area);
        }
        DrawStep::done()
    }
}

impl MpButtonRef {
    pub fn clicked(&self, actions: &Actions) -> bool {
        self.borrow().is_some_and(|inner| inner.clicked(actions))
    }

    pub fn pressed(&self, actions: &Actions) -> bool {
        self.borrow().is_some_and(|inner| inner.pressed(actions))
    }

    pub fn set_text(&self, cx: &mut Cx, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(cx, text);
        }
    }

    pub fn style(&self) -> Option<MpButtonStyle> {
        self.borrow().map(|inner| inner.style())
    }

    pub fn set_style(&self, cx: &mut Cx, style: MpButtonStyle) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_style(cx, style);
        }
    }

    pub fn set_control(&self, cx: &mut Cx, control: ControlSize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_control(cx, control);
        }
    }

    pub fn set_disabled(&self, cx: &mut Cx, disabled: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_disabled(cx, disabled);
        }
    }

    pub fn is_disabled(&self) -> bool {
        self.borrow().is_some_and(|inner| inner.is_disabled())
    }
}
