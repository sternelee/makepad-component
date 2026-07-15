use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp_theme.*

    // ============================================
    // Font Size Constants
    // ============================================
    let TEXT_FONT_SIZE_XS = 10.0
    let TEXT_FONT_SIZE_SM = 12.0
    let TEXT_FONT_SIZE_MD = 14.0
    let TEXT_FONT_SIZE_LG = 16.0
    let TEXT_FONT_SIZE_XL = 18.0

    // Line height
    let TEXT_LINE_HEIGHT = 1.6

    // ============================================
    // Base Text Component (Paragraph-like)
    // ============================================
    mod.widgets.MpTextBase = #(MpText::register_widget(vm))
    mod.widgets.MpText = set_type_default() do mod.widgets.MpTextBase{
        width: Fill
        height: Fit
        // Word wrapping is controlled by the turtle flow in Makepad 2.0
        flow: Flow.Right{wrap: true}

        draw_text +: {
            text_style: theme.font_regular{
                font_size: TEXT_FONT_SIZE_MD
                line_spacing: TEXT_LINE_HEIGHT
            }
            color: FOREGROUND
        }

        text: ""
    }

    // ============================================
    // Size Variants
    // ============================================
    mod.widgets.MpTextXs = mod.widgets.MpText{
        draw_text +: { text_style: theme.font_regular{font_size: TEXT_FONT_SIZE_XS, line_spacing: TEXT_LINE_HEIGHT} }
    }

    mod.widgets.MpTextSm = mod.widgets.MpText{
        draw_text +: { text_style: theme.font_regular{font_size: TEXT_FONT_SIZE_SM, line_spacing: TEXT_LINE_HEIGHT} }
    }

    mod.widgets.MpTextMd = mod.widgets.MpText{
        draw_text +: { text_style: theme.font_regular{font_size: TEXT_FONT_SIZE_MD, line_spacing: TEXT_LINE_HEIGHT} }
    }

    mod.widgets.MpTextLg = mod.widgets.MpText{
        draw_text +: { text_style: theme.font_regular{font_size: TEXT_FONT_SIZE_LG, line_spacing: TEXT_LINE_HEIGHT} }
    }

    mod.widgets.MpTextXl = mod.widgets.MpText{
        draw_text +: { text_style: theme.font_regular{font_size: TEXT_FONT_SIZE_XL, line_spacing: TEXT_LINE_HEIGHT} }
    }

    // ============================================
    // Color Variants
    // ============================================
    mod.widgets.MpTextMuted = mod.widgets.MpText{
        draw_text +: { color: MUTED_FOREGROUND }
    }

    mod.widgets.MpTextPrimary = mod.widgets.MpText{
        draw_text +: { color: PRIMARY }
    }

    mod.widgets.MpTextDanger = mod.widgets.MpText{
        draw_text +: { color: DANGER }
    }

    mod.widgets.MpTextSuccess = mod.widgets.MpText{
        draw_text +: { color: SUCCESS }
    }

    mod.widgets.MpTextWarning = mod.widgets.MpText{
        draw_text +: { color: WARNING }
    }

    // ============================================
    // Weight Variants
    // ============================================
    mod.widgets.MpTextBold = mod.widgets.MpText{
        draw_text +: {
            text_style: theme.font_bold{
                font_size: TEXT_FONT_SIZE_MD
                line_spacing: TEXT_LINE_HEIGHT
            }
        }
    }

    // ============================================
    // Special Variants
    // ============================================

    // Inline text (no word wrap, fits content)
    mod.widgets.MpTextInline = mod.widgets.MpText{
        width: Fit
        flow: Flow.Right{wrap: false}
    }

    // Code/Monospace text
    mod.widgets.MpTextCode = mod.widgets.MpText{
        width: Fit
        flow: Flow.Right{wrap: false}
        padding: Inset{left: 4.0, right: 4.0, top: 2.0, bottom: 2.0}

        show_bg: true
        draw_bg +: {
            color: instance(MUTED)
            radius: instance(4.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, self.radius)
                sdf.fill(self.color)
                return sdf.result
            }
        }

        draw_text +: {
            text_style: theme.font_code{
                font_size: 13.0
            }
        }
    }

    // Blockquote style
    mod.widgets.MpTextBlockquote = mod.widgets.View{
        width: Fill
        height: Fit
        padding: Inset{left: 16.0, top: 8.0, bottom: 8.0}
        margin: Inset{top: 8.0, bottom: 8.0}

        show_bg: true
        draw_bg +: {
            border_color: instance(MUTED_FOREGROUND)
            bg_color: instance(#x00000008)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                // Left border
                sdf.rect(0.0, 0.0, 3.0, self.rect_size.y)
                sdf.fill(self.border_color)
                // Background
                sdf.rect(0.0, 0.0, self.rect_size.x, self.rect_size.y)
                sdf.fill(self.bg_color)
                return sdf.result
            }
        }

        mp_text := mod.widgets.MpText{
            draw_text +: {
                color: MUTED_FOREGROUND
                text_style: theme.font_regular{
                    font_size: TEXT_FONT_SIZE_MD
                    line_spacing: TEXT_LINE_HEIGHT
                }
            }
        }
    }

    // Lead text (larger, intro paragraph)
    mod.widgets.MpTextLead = mod.widgets.MpText{
        draw_text +: {
            text_style: theme.font_regular{
                font_size: TEXT_FONT_SIZE_XL
                line_spacing: 1.7
            }
            color: MUTED_FOREGROUND
        }
    }

    // Small/Caption text
    mod.widgets.MpTextCaption = mod.widgets.MpText{
        draw_text +: {
            text_style: theme.font_regular{
                font_size: TEXT_FONT_SIZE_XS
                line_spacing: TEXT_LINE_HEIGHT
            }
            color: MUTED_FOREGROUND
        }
    }
}

/// Text widget for paragraph-like text display with word wrapping
#[derive(Script, ScriptHook, Widget)]
pub struct MpText {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_text: DrawText,
    #[live]
    draw_bg: DrawQuad,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[live(false)]
    show_bg: bool,

    /// Text content
    #[live]
    text: ArcStringMut,

    /// Maximum number of lines (0 = unlimited)
    #[live(0usize)]
    max_lines: usize,

    /// Text overflow indicator (e.g., "...")
    #[live]
    overflow: ArcStringMut,

    #[rust]
    area: Area,
}

impl Widget for MpText {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {
        // Text is non-interactive by default
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let text = self.text.as_ref();

        if self.show_bg {
            self.draw_bg.begin(cx, walk, self.layout);
            self.draw_text
                .draw_walk(cx, Walk::fit(), Align::default(), text);
            self.draw_bg.end(cx);
            self.area = self.draw_bg.area();
        } else {
            cx.begin_turtle(walk, self.layout);
            self.draw_text
                .draw_walk(cx, Walk::fit(), Align::default(), text);
            cx.end_turtle_with_area(&mut self.area);
        }

        DrawStep::done()
    }
}

impl MpText {
    /// Set the text content
    pub fn set_text(&mut self, text: &str) {
        self.text.as_mut_empty().push_str(text);
    }

    /// Get the text content
    pub fn text(&self) -> &str {
        self.text.as_ref()
    }

    /// Set maximum number of lines (0 = unlimited)
    pub fn set_max_lines(&mut self, lines: usize) {
        self.max_lines = lines;
    }

    /// Set the overflow indicator
    pub fn set_overflow(&mut self, overflow: &str) {
        self.overflow.as_mut_empty().push_str(overflow);
    }
}

impl MpTextRef {
    /// Set the text content
    pub fn set_text(&self, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(text);
        }
    }

    /// Get the text content
    pub fn text(&self) -> String {
        if let Some(inner) = self.borrow() {
            inner.text().to_string()
        } else {
            String::new()
        }
    }

    /// Set maximum number of lines
    pub fn set_max_lines(&self, lines: usize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_max_lines(lines);
        }
    }
}
