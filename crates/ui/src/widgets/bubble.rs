use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // Alignment / variant enums exposed to script (PopupMenuPosition pattern)
    mod.widgets.MpBubbleAlignment = #(MpBubbleAlignment::script_api(vm))
    mod.widgets.MpBubbleVariant = #(MpBubbleVariant::script_api(vm))

    mod.widgets.MpBubble = set_type_default() do #(MpBubble::register_widget(vm)){
        width: Fit
        height: Fit

        // Palette baked for Rust-side variant resolution
        c_accent: ACCENT
        c_on_accent: ON_ACCENT
        c_surface: SURFACE_CARD
        c_text: TEXT
        c_border: BORDER
        c_danger_muted: DANGER_MUTED
        c_on_solid: ON_SOLID

        alignment: mod.widgets.MpBubbleAlignment.Start
        variant: mod.widgets.MpBubbleVariant.Secondary
        message: ""
    }
}

#[derive(Clone, Debug, Default)]
pub enum MpBubbleAction {
    #[default]
    None,
}

/// Which side of the conversation the bubble sits on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Script, ScriptHook)]
pub enum MpBubbleAlignment {
    /// Assistant / incoming (left).
    #[default]
    Start,
    /// User / outgoing (right).
    End,
}

/// Visual treatment for the bubble surface (gpui BubbleVariant subset).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Script, ScriptHook)]
pub enum MpBubbleVariant {
    /// Accent surface with on-accent text.
    Filled,
    /// Neutral raised surface (default).
    #[default]
    Secondary,
    /// Background surface with a visible border.
    Outline,
    /// Destructive surface for failed or invalid content.
    Destructive,
}

// Bubble surface shader: bg + border resolved from Rust per variant
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpBubble {
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
}

#[derive(Script, ScriptHook, Widget)]
pub struct MpBubble {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_bg: DrawMpBubble,

    #[live]
    draw_text: DrawText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    /// The message text.
    #[live]
    message: ArcStringMut,

    /// Which side of the conversation this bubble belongs to.
    #[live]
    alignment: MpBubbleAlignment,

    /// Visual treatment of the surface.
    #[live]
    variant: MpBubbleVariant,

    /// Five-step size driving the message font and padding.
    #[live]
    size: MpSize,

    // Palette baked from the theme
    #[live]
    c_accent: Vec4f,
    #[live]
    c_on_accent: Vec4f,
    #[live]
    c_surface: Vec4f,
    #[live]
    c_text: Vec4f,
    #[live]
    c_border: Vec4f,
    #[live]
    c_danger_muted: Vec4f,
    #[live]
    c_on_solid: Vec4f,
}

struct BubbleStyle {
    bg: Vec4f,
    fg: Vec4f,
    border: Vec4f,
    border_width: f32,
}

impl MpBubble {
    fn resolve_style(&self) -> BubbleStyle {
        match self.variant {
            MpBubbleVariant::Filled => BubbleStyle {
                bg: self.c_accent,
                fg: self.c_on_accent,
                border: self.c_accent,
                border_width: 0.0,
            },
            MpBubbleVariant::Secondary => BubbleStyle {
                bg: self.c_surface,
                fg: self.c_text,
                border: self.c_surface,
                border_width: 0.0,
            },
            MpBubbleVariant::Outline => BubbleStyle {
                bg: Vec4f { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
                fg: self.c_text,
                border: self.c_border,
                border_width: 1.0,
            },
            MpBubbleVariant::Destructive => BubbleStyle {
                bg: self.c_danger_muted,
                fg: self.c_on_solid,
                border: self.c_danger_muted,
                border_width: 0.0,
            },
        }
    }
}

impl Widget for MpBubble {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {}

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let size = self.size;
        let style = self.resolve_style();

        // Surface metrics: roomier than controls (display component)
        let pad_h = size.padding_h() + 2.0;
        let pad_v = size.padding_v() + 2.0;
        self.layout.padding = Inset {
            left: pad_h,
            right: pad_h,
            top: pad_v,
            bottom: pad_v,
        };

        self.draw_bg.bg = style.bg;
        self.draw_bg.border_color = style.border;
        self.draw_bg.border_width = style.border_width;
        self.draw_bg.radius = 12.0;

        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_text
            .draw_walk(cx, Walk::fit(), Align::default(), self.message.as_ref());
        self.draw_bg.end(cx);

        DrawStep::done()
    }
}

impl MpBubble {
    pub fn set_message(&mut self, cx: &mut Cx, message: &str) {
        self.message.as_mut_empty().push_str(message);
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

impl MpBubbleRef {
    pub fn set_message(&self, cx: &mut Cx, message: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_message(cx, message);
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
