use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // Expose the variant enum proto to the script heap.
    let MpButtonVariant = set_type_default() do #(MpButtonVariant::script_api(vm))
    mod.widgets.MpButtonVariant = MpButtonVariant

    // Button shader: dead-simple plate. All colors/radius are instance fields
    // written from Rust in draw_walk (the 2.0 replacement for apply_over);
    // the palette instances are baked from the theme at apply time.
    set_type_default() do #(DrawMpButton::script_shader(vm)){
        ..mod.draw.DrawQuad

        hover: 0.0
        pressed: 0.0
        focus: 0.0
        disabled: 0.0

        bg: ACCENT
        bg_hover: ACCENT_HOVER
        bg_pressed: ACCENT_HOVER
        bg_disabled: ACCENT
        border_color: #x0000
        border_width: 0.0
        radius: 8.0
        focus_color: CARET

        // Theme palette (read from Rust to resolve variant colors)
        c_solid: SOLID
        c_solid_hover: SOLID_HOVER
        on_solid: ON_SOLID
        c_accent: ACCENT
        c_accent_hover: ACCENT_HOVER
        on_accent: ON_ACCENT
        c_secondary: SECONDARY
        c_secondary_hover: SECONDARY_HOVER
        on_secondary: ON_SECONDARY
        c_danger: DANGER
        c_danger_hover: DANGER_HOVER
        wash_hover: ELEMENT_HOVER
        wash_active: ELEMENT_ACTIVE
        fg: TEXT
        fg_muted: TEXT_MUTED
        fg_faint: TEXT_FAINT
        border: BORDER_STRONG

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            sdf.box(
                self.border_width
                self.border_width
                self.rect_size.x - self.border_width * 2.0
                self.rect_size.y - self.border_width * 2.0
                max(1.0, self.radius)
            )

            // Subtle hover: slightly brighten
            let hover_color = mix(self.bg, self.bg_hover, self.hover * 0.6)
            // Pressed: darken more noticeably
            let pressed_color = mix(hover_color, self.bg_pressed, self.pressed * 0.8)
            let final_color = mix(pressed_color, self.bg_disabled, self.disabled)

            sdf.fill_keep(final_color)

            // Focus ring: CARET border at 2px when focused (overrides any existing border)
            let bw = mix(self.border_width, 2.0, self.focus)
            let bc = mix(self.border_color, self.focus_color, self.focus)
            if (bw > 0.0) {
                sdf.stroke(bc, bw)
            }

            return sdf.result
        }
    }

    // Base button component
    mod.widgets.MpButtonBase = #(MpButton::register_widget(vm))
    mod.widgets.MpButton = set_type_default() do mod.widgets.MpButtonBase{
        width: Fit
        height: Fit
        flow: Right
        spacing: 6
        align: Align{x: 0.5, y: 0.5}

        variant: MpButtonVariant.Default
        size: MpSize.Medium

        draw_text +: {
            text_style: theme.font_regular{font_size: 13.0}
            color: TEXT
        }

        text: ""
    }

    // ---- Backward-compat aliases (old per-variant DSL types) ----
    mod.widgets.MpButtonProminent = mod.widgets.MpButton{
        variant: MpButtonVariant.Prominent
    }
    mod.widgets.MpButtonAccent = mod.widgets.MpButton{
        variant: MpButtonVariant.Accent
    }
    mod.widgets.MpButtonGhost = mod.widgets.MpButton{
        variant: MpButtonVariant.Ghost
    }
    mod.widgets.MpButtonOutline = mod.widgets.MpButton{
        variant: MpButtonVariant.Outline
    }
    mod.widgets.MpButtonDestructive = mod.widgets.MpButton{
        variant: MpButtonVariant.Destructive
    }
    mod.widgets.MpButtonSecondary = mod.widgets.MpButton{
        variant: MpButtonVariant.Secondary
    }
    mod.widgets.MpButtonLink = mod.widgets.MpButton{
        variant: MpButtonVariant.Link
    }
    mod.widgets.MpButtonText = mod.widgets.MpButton{
        variant: MpButtonVariant.Text
    }
    mod.widgets.MpButtonSmall = mod.widgets.MpButton{
        size: MpSize.Small
    }
    mod.widgets.MpButtonLarge = mod.widgets.MpButton{
        size: MpSize.Large
    }
}

/// Visual variant of a button (gpui-component `ButtonVariant` port).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Script, ScriptHook)]
pub enum MpButtonVariant {
    /// Subdued plate with a hairline border.
    #[pick]
    #[default]
    Default,
    /// Inverted solid plate (the high-contrast primary action).
    Prominent,
    /// Brand-accent solid plate.
    Accent,
    /// Subdued filled plate, no border.
    Secondary,
    /// Transparent, muted label, hover wash.
    Ghost,
    /// Transparent with hairline border, body-text label.
    Outline,
    /// Danger solid plate.
    Destructive,
    /// Transparent, accent-colored label (hyperlink style).
    Link,
    /// Transparent, plain text with no padding wash.
    Text,
}

/// Per-variant resolved paint style, computed in Rust each draw.
struct ButtonStyle {
    bg: Vec4f,
    bg_hover: Vec4f,
    bg_pressed: Vec4f,
    border_color: Vec4f,
    border_width: f32,
    fg: Vec4f,
    /// Horizontal padding multiplier (text/link buttons are tighter).
    pad_scale: f64,
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpButton {
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
    disabled: f32,

    // Resolved paint (written from Rust each draw)
    #[live]
    bg: Vec4f,
    #[live]
    bg_hover: Vec4f,
    #[live]
    bg_pressed: Vec4f,
    #[live]
    bg_disabled: Vec4f,
    #[live]
    border_color: Vec4f,
    #[live]
    border_width: f32,
    #[live]
    radius: f32,
    #[live]
    focus_color: Vec4f,

    // Theme palette (baked at apply time; read from Rust)
    #[live]
    c_solid: Vec4f,
    #[live]
    c_solid_hover: Vec4f,
    #[live]
    on_solid: Vec4f,
    #[live]
    c_accent: Vec4f,
    #[live]
    c_accent_hover: Vec4f,
    #[live]
    on_accent: Vec4f,
    #[live]
    c_secondary: Vec4f,
    #[live]
    c_secondary_hover: Vec4f,
    #[live]
    on_secondary: Vec4f,
    #[live]
    c_danger: Vec4f,
    #[live]
    c_danger_hover: Vec4f,
    #[live]
    wash_hover: Vec4f,
    #[live]
    wash_active: Vec4f,
    #[live]
    fg: Vec4f,
    #[live]
    fg_muted: Vec4f,
    #[live]
    fg_faint: Vec4f,
    #[live]
    border: Vec4f,
}

// Rust implementation
#[derive(Script, Widget, Animator)]
pub struct MpButton {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[apply_default]
    animator: Animator,

    #[live]
    variant: MpButtonVariant,
    #[live]
    size: MpSize,

    #[redraw]
    #[live]
    draw_bg: DrawMpButton,
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

    #[rust]
    area: Area,

    #[live(false)]
    focused: bool,
}

fn alpha_scaled(c: Vec4f, k: f32) -> Vec4f {
    Vec4f { x: c.x, y: c.y, z: c.z, w: c.w * k }
}

impl ScriptHook for MpButton {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        vm.with_cx_mut(|cx| {
            let disabled = self.disabled;
            self.animator_toggle(
                cx,
                disabled,
                Animate::No,
                ids!(disabled.on),
                ids!(disabled.off),
            );
        });
    }
}

// Custom Actions
#[derive(Clone, Debug, Default)]
pub enum MpButtonAction {
    Clicked,
    Pressed,
    Released,
    #[default]
    None,
}

impl MpButton {
    /// Resolve the variant's paint style from the baked theme palette.
    fn resolve_style(&self) -> ButtonStyle {
        let d = &self.draw_bg;
        let transparent = Vec4f { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };
        match self.variant {
            MpButtonVariant::Prominent => ButtonStyle {
                bg: d.c_solid,
                bg_hover: d.c_solid_hover,
                bg_pressed: d.c_solid_hover,
                border_color: transparent,
                border_width: 0.0,
                fg: d.on_solid,
                pad_scale: 1.0,
            },
            MpButtonVariant::Accent => ButtonStyle {
                bg: d.c_accent,
                bg_hover: d.c_accent_hover,
                bg_pressed: d.c_accent_hover,
                border_color: transparent,
                border_width: 0.0,
                fg: d.on_accent,
                pad_scale: 1.0,
            },
            MpButtonVariant::Secondary => ButtonStyle {
                bg: d.c_secondary,
                bg_hover: d.c_secondary_hover,
                bg_pressed: d.c_secondary_hover,
                border_color: transparent,
                border_width: 0.0,
                fg: d.on_secondary,
                pad_scale: 1.0,
            },
            MpButtonVariant::Destructive => ButtonStyle {
                bg: d.c_danger,
                bg_hover: d.c_danger_hover,
                bg_pressed: d.c_danger_hover,
                border_color: transparent,
                border_width: 0.0,
                fg: d.on_solid,
                pad_scale: 1.0,
            },
            MpButtonVariant::Ghost => ButtonStyle {
                bg: transparent,
                bg_hover: d.wash_hover,
                bg_pressed: d.wash_active,
                border_color: transparent,
                border_width: 0.0,
                fg: d.fg_muted,
                pad_scale: 0.7,
            },
            MpButtonVariant::Outline => ButtonStyle {
                bg: transparent,
                bg_hover: d.wash_hover,
                bg_pressed: d.wash_active,
                border_color: d.border,
                border_width: 1.0,
                fg: d.fg,
                pad_scale: 1.0,
            },
            MpButtonVariant::Link => ButtonStyle {
                bg: transparent,
                bg_hover: transparent,
                bg_pressed: transparent,
                border_color: transparent,
                border_width: 0.0,
                fg: d.c_accent,
                pad_scale: 0.35,
            },
            MpButtonVariant::Text => ButtonStyle {
                bg: transparent,
                bg_hover: transparent,
                bg_pressed: transparent,
                border_color: transparent,
                border_width: 0.0,
                fg: d.fg,
                pad_scale: 0.35,
            },
            MpButtonVariant::Default => ButtonStyle {
                bg: d.c_secondary,
                bg_hover: d.c_secondary_hover,
                bg_pressed: d.c_secondary_hover,
                border_color: d.border,
                border_width: 1.0,
                fg: d.fg,
                pad_scale: 1.0,
            },
        }
    }
}

impl Widget for MpButton {
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
                self.animator_play(cx, ids!(pressed.on));
                cx.widget_action(uid, MpButtonAction::Pressed);
                // Claim key focus on click (bezel focus ring)
                cx.set_key_focus(self.area);
            }
            Hit::FingerUp(fe) => {
                self.animator_play(cx, ids!(pressed.off));
                if fe.is_over {
                    cx.widget_action(uid, MpButtonAction::Clicked);
                }
                cx.widget_action(uid, MpButtonAction::Released);
            }
            _ => {}
        }

        // Keyboard activation (Enter/Space) when this widget has key focus
        if self.focused {
            if let Event::KeyDown(ke) = event {
                if ke.key_code == KeyCode::ReturnKey || ke.key_code == KeyCode::Space {
                    if !ke.is_repeat {
                        cx.widget_action(uid, MpButtonAction::Clicked);
                    }
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let size = self.size;
        let style = self.resolve_style();
        let disabled = if self.disabled { 1.0f32 } else { 0.0 };

        // Metrics from the size system
        let pad_h = size.padding_h() * style.pad_scale;
        let pad_v = size.padding_v() * if style.pad_scale < 1.0 { 0.5 } else { 1.0 };
        self.layout.padding = Inset {
            left: pad_h,
            right: pad_h,
            top: pad_v,
            bottom: pad_v,
        };
        self.draw_text.text_style.font_size = size.font_size();
        self.draw_bg.radius = size.radius();

        // Paint from the resolved variant style
        self.draw_bg.bg = style.bg;
        self.draw_bg.bg_hover = style.bg_hover;
        self.draw_bg.bg_pressed = style.bg_pressed;
        self.draw_bg.bg_disabled = alpha_scaled(style.bg, 0.4);
        self.draw_bg.border_color = style.border_color;
        self.draw_bg.border_width = style.border_width;

        // Text color (disabled blends toward faint)
        let fg = style.fg;
        let faint = self.draw_bg.fg_faint;
        self.draw_text.color = Vec4f {
            x: fg.x + (faint.x - fg.x) * disabled,
            y: fg.y + (faint.y - fg.y) * disabled,
            z: fg.z + (faint.z - fg.z) * disabled,
            w: fg.w + (faint.w - fg.w) * disabled,
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

impl MpButton {
    pub fn clicked(&self, actions: &Actions) -> bool {
        if let Some(action) = actions.find_widget_action(self.widget_uid()) {
            matches!(action.cast(), MpButtonAction::Clicked)
        } else {
            false
        }
    }

    pub fn pressed(&self, actions: &Actions) -> bool {
        if let Some(action) = actions.find_widget_action(self.widget_uid()) {
            matches!(action.cast(), MpButtonAction::Pressed)
        } else {
            false
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text.as_mut_empty().push_str(text);
    }

    pub fn variant(&self) -> MpButtonVariant {
        self.variant
    }

    /// Switch the visual variant at runtime.
    pub fn set_variant(&mut self, cx: &mut Cx, variant: MpButtonVariant) {
        self.variant = variant;
        self.redraw(cx);
    }

    pub fn size(&self) -> MpSize {
        self.size
    }

    /// Switch the control size at runtime.
    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        self.size = size;
        self.redraw(cx);
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn set_disabled(&mut self, cx: &mut Cx, disabled: bool) {
        self.disabled = disabled;
        self.animator_toggle(
            cx,
            disabled,
            Animate::Yes,
            ids!(disabled.on),
            ids!(disabled.off),
        );
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

    pub fn set_variant(&self, cx: &mut Cx, variant: MpButtonVariant) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_variant(cx, variant);
        }
    }

    pub fn set_size(&self, cx: &mut Cx, size: MpSize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_size(cx, size);
        }
    }

    pub fn set_disabled(&self, cx: &mut Cx, disabled: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_disabled(cx, disabled);
        }
    }
}
