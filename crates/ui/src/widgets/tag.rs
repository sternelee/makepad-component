use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpTag - small status indicator (gpui Tag port)
    // Filled = muted surface + strong color text; Outline = transparent
    // surface + strong color border/text. Colors resolved from Rust.
    // ============================================================

    mod.widgets.MpTagVariant = #(MpTagVariant::script_api(vm))
    mod.widgets.MpTagStyle = #(MpTagStyle::script_api(vm))

    mod.widgets.MpTag = set_type_default() do #(MpTag::register_widget(vm)){
        width: Fit
        height: Fit
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 10.0, right: 10.0, top: 3.0, bottom: 3.0}
        spacing: 5.0

        // Tag surface: rounded box + optional leading status dot
        set_type_default() do #(DrawMpTag::script_shader(vm)){
            ..mod.draw.DrawQuad

            bg: #x0000
            border_color: #x0000
            border_width: 0.0
            radius: 4.0
            dot: 0.0
            dot_color: #x0000

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let sz = self.rect_size

                sdf.box(0.5, 0.5, sz.x - 1.0, sz.y - 1.0, self.radius)
                if (self.bg.w > 0.0) {
                    sdf.fill_keep(self.bg)
                }
                if (self.border_width > 0.0) {
                    sdf.stroke(self.border_color, self.border_width)
                }
                if (self.dot > 0.5) {
                    sdf.circle(6.5, sz.y * 0.5, 2.5)
                    sdf.fill(self.dot_color)
                }
                return sdf.result
            }
        }

        text: ""
        variant: mod.widgets.MpTagVariant.Secondary
        style: mod.widgets.MpTagStyle.Filled
        dot: false

        // Text style (font family + size); color is overwritten from Rust
        // each draw with the resolved variant foreground
        draw_text +: {
            text_style: theme.font_regular{font_size: 13.0}
            color: TEXT
        }

        // Palette baked for Rust-side variant resolution
        c_accent: ACCENT
        c_accent_muted: ACCENT_MUTED
        c_on_accent: ON_ACCENT
        c_surface: SURFACE_CARD
        c_text: TEXT
        c_text_muted: TEXT_MUTED
        c_border: BORDER
        c_danger: DANGER
        c_danger_muted: DANGER_MUTED
        c_warning: WARNING
        c_warning_muted: WARNING_MUTED
        c_success: SUCCESS
        c_success_muted: SUCCESS_MUTED
        c_info: INFO
        c_info_muted: INFO_MUTED
    }
}

#[derive(Clone, Debug, Default)]
pub enum MpTagAction {
    #[default]
    None,
}

/// Semantic color family of the tag.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Script, ScriptHook)]
pub enum MpTagVariant {
    /// Accent (brand) tag.
    Primary,
    /// Neutral tag (default).
    #[default]
    Secondary,
    /// Destructive / error status.
    Danger,
    /// Success / completed status.
    Success,
    /// Caution status.
    Warning,
    /// Informational status.
    Info,
}

/// Filled = muted surface + strong text; Outline = transparent + strong border.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Script, ScriptHook)]
pub enum MpTagStyle {
    #[default]
    Filled,
    Outline,
}

// Tag surface shader: bg + border + optional leading dot, resolved from Rust
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpTag {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    bg: Vec4f,
    #[live]
    border_color: Vec4f,
    #[live]
    border_width: f32,
    #[live]
    radius: f32,
    #[live]
    dot: f32,
    #[live]
    dot_color: Vec4f,
}

#[derive(Script, ScriptHook, Widget)]
pub struct MpTag {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_bg: DrawMpTag,

    #[live]
    draw_text: DrawText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    /// The tag label.
    #[live]
    text: ArcStringMut,

    /// Semantic color family.
    #[live]
    variant: MpTagVariant,

    /// Filled or Outline treatment.
    #[live]
    style: MpTagStyle,

    /// Show a leading status dot in the variant color.
    #[live]
    dot: bool,

    /// Five-step size driving padding and font.
    #[live]
    size: MpSize,

    // Palette baked from the theme
    #[live]
    c_accent: Vec4f,
    #[live]
    c_accent_muted: Vec4f,
    #[live]
    c_on_accent: Vec4f,
    #[live]
    c_surface: Vec4f,
    #[live]
    c_text: Vec4f,
    #[live]
    c_text_muted: Vec4f,
    #[live]
    c_border: Vec4f,
    #[live]
    c_danger: Vec4f,
    #[live]
    c_danger_muted: Vec4f,
    #[live]
    c_warning: Vec4f,
    #[live]
    c_warning_muted: Vec4f,
    #[live]
    c_success: Vec4f,
    #[live]
    c_success_muted: Vec4f,
    #[live]
    c_info: Vec4f,
    #[live]
    c_info_muted: Vec4f,
}

struct TagStyle {
    bg: Vec4f,
    fg: Vec4f,
    border: Vec4f,
    border_width: f32,
    dot_color: Vec4f,
}

impl MpTag {
    fn resolve_style(&self) -> TagStyle {
        // Strong color per variant (used for text / border / dot)
        let (strong, muted, neutral) = match self.variant {
            MpTagVariant::Primary => (self.c_accent, self.c_accent_muted, false),
            MpTagVariant::Secondary => (self.c_text_muted, self.c_surface, true),
            MpTagVariant::Danger => (self.c_danger, self.c_danger_muted, false),
            MpTagVariant::Success => (self.c_success, self.c_success_muted, false),
            MpTagVariant::Warning => (self.c_warning, self.c_warning_muted, false),
            MpTagVariant::Info => (self.c_info, self.c_info_muted, false),
        };

        match self.style {
            MpTagStyle::Filled => TagStyle {
                bg: muted,
                fg: if neutral { self.c_text } else { strong },
                border: muted,
                border_width: 0.0,
                dot_color: strong,
            },
            MpTagStyle::Outline => TagStyle {
                bg: Vec4f { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
                fg: if neutral { self.c_text_muted } else { strong },
                border: if neutral { self.c_border } else { strong },
                border_width: 1.0,
                dot_color: strong,
            },
        }
    }
}

impl Widget for MpTag {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let size = self.size;
        let style = self.resolve_style();

        let pad_h = size.padding_h() as f64 - 2.0;
        let pad_v = size.padding_v() as f64 * 0.75;
        let dot_extra = if self.dot { 9.0 } else { 0.0 };
        self.layout.padding = Inset {
            left: pad_h + dot_extra,
            right: pad_h,
            top: pad_v,
            bottom: pad_v,
        };

        self.draw_bg.bg = style.bg;
        self.draw_bg.border_color = style.border;
        self.draw_bg.border_width = style.border_width;
        self.draw_bg.radius = (size.min_height() as f32) * 0.28;
        self.draw_bg.dot = if self.dot { 1.0 } else { 0.0 };
        self.draw_bg.dot_color = style.dot_color;

        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_text.text_style.font_size = size.font_size();
        self.draw_text.color = style.fg;
        self.draw_text
            .draw_walk(cx, Walk::fit(), Align::default(), self.text.as_ref());
        self.draw_bg.end(cx);

        DrawStep::done()
    }
}

impl MpTag {
    pub fn set_text(&mut self, cx: &mut Cx, text: &str) {
        self.text.as_mut_empty().push_str(text);
        self.redraw(cx);
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
}

impl MpTagRef {
    pub fn set_text(&self, cx: &mut Cx, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(cx, text);
        }
    }

    pub fn size(&self) -> MpSize {
        if let Some(inner) = self.borrow() {
            inner.size()
        } else {
            MpSize::default()
        }
    }

    pub fn set_size(&self, cx: &mut Cx, size: MpSize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_size(cx, size);
        }
    }
}
