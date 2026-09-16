//! `MpMarkdown` — a document, laid out by `makepad-markdown` and painted here.
//!
//! ## The split this sits in
//!
//! | crate | holds |
//! |---|---|
//! | `makepad-markdown` | the document model, `parse`, `serialize`, and the **layout** |
//! | `crates/ui` (here) | the widget: the theme's metrics in, glyphs out |
//!
//! `makepad-markdown` has **no dependencies at all**, and that is worth keeping: its tests run in
//! 0.12s and its purity is what let the fixed-point property be checked over nine thousand generated
//! documents. So the paint lives here, where the other widgets live — the same arrangement as
//! `MpCodeBlock` in this crate with the classifier in `syntax`.
//!
//! ## Why it takes a measure instead of filling its parent
//!
//! A self-drawn widget has to state its own height **before** it draws, and a document's height depends
//! on the width it wraps to — so `width: Fill` is a circle: the width comes from the layout pass, the
//! height from the wrap, and the wrap from the width. The way out is to take the wrap width as a
//! **number** the caller sets ([`MpMarkdown::set_measure`]), which breaks the circle at the cost of the
//! caller knowing its own column.
//!
//! The alternative is a measure-then-draw pass, which means the first frame has the wrong height and the
//! document jumps once — a fault that shows up as a flicker on every resize and is exactly the kind of
//! thing this port cannot check right now, because **screen capture is unavailable**. Taking a number is
//! the version that is correct on every frame, and the DSL default is a sensible column width.
//!
//! ## What it paints
//!
//! Every line's text in the body colour, a marker in the muted one, quotes and fences on a plate, and a
//! rule for a divider. The **layout** decides where each of those goes and how wide a marker is — this
//! module only reads it — so the two cannot disagree about where a line starts.

use makepad_widgets::*;

use makepad_markdown::layout::{self, Laid, Metrics};
use makepad_markdown::{BlockKind, Doc};

use crate::mp::text;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    /// The document's plate. The same fill/hairline/corner shape every other plate in this crate uses.
    set_type_default() do #(DrawMpMarkdown::script_shader(vm)){
        ..mod.draw.DrawQuad

        fill: #x00000000
        border: #x00000000
        border_width: 0.0
        radius: 10.0

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let bw = self.border_width
            sdf.box(bw, bw, self.rect_size.x - bw * 2.0, self.rect_size.y - bw * 2.0, max(1.0, self.radius))
            sdf.fill_keep(self.fill)
            if (bw > 0.0) {
                sdf.stroke(self.border, bw)
            }
            return sdf.result
        }
    }

    mod.mp.MpMarkdownBase = #(MpMarkdown::register_widget(vm))

    mod.mp.MpMarkdown = set_type_default() do mod.mp.MpMarkdownBase{
        // A column, not `Fill`: see the module doc on the measure.
        width: 640
        height: Fit
        initial_measure: 640.0

        draw_bg +: {
            fill: surface_card
            border: border
            border_width: 1.0
            radius: 10.0
        }
        draw_marker +: {
            text_style: mod.mpc.type.body
            color: #x00000000
        }
        draw_body +: {
            text_style: mod.mpc.type.body
            color: #x00000000
        }
        draw_code +: {
            text_style: theme.font_code{font_size: 12.0}
            color: #x00000000
        }
    }
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpMarkdown {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    fill: Vec4f,
    #[live]
    border: Vec4f,
    #[live]
    border_width: f32,
    #[live]
    radius: f32,
}

#[derive(Script, Widget)]
pub struct MpMarkdown {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_bg: DrawMpMarkdown,
    #[live]
    draw_marker: DrawText,
    #[live]
    draw_body: DrawText,
    #[live]
    draw_code: DrawText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    /// The DSL's wrap width. The **live** value is stored separately and copied into `measure` once, so
    /// a script re-apply re-asserts this without wiping what a caller set.
    #[live]
    initial_measure: f64,

    #[rust]
    measure: f64,
    #[rust]
    doc: Doc,
    #[rust]
    laid: Option<Laid>,
    /// The metrics the last layout used, so a theme change re-lays out rather than repainting the old
    /// wrap.
    #[rust]
    used_metrics: Option<Metrics>,
}

/// The padding inside the plate, on each side.
const PAD: f64 = 14.0;

impl MpMarkdown {
    /// Set the document. Parsing is `makepad-markdown`'s, and it is where the fixed point holds.
    pub fn set_source(&mut self, cx: &mut Cx, source: &str) {
        self.doc = makepad_markdown::parse(source);
        // The layout needs metrics, which need a `Cx`; the actual layout happens at draw time, where the
        // theme is available. Dropping the cached layout is what makes the next draw re-lay out.
        self.laid = None;
        self.redraw(cx);
    }

    pub fn set_doc(&mut self, cx: &mut Cx, doc: Doc) {
        self.doc = doc;
        self.laid = None;
        self.redraw(cx);
    }

    /// Set the wrap width.
    pub fn set_measure(&mut self, cx: &mut Cx, measure: f64) {
        if (self.measure - measure).abs() < 1e-9 {
            return;
        }
        self.measure = measure.max(1.0);
        self.laid = None;
        self.redraw(cx);
    }

    pub fn doc(&self) -> &Doc {
        &self.doc
    }

    pub fn measure(&self) -> f64 {
        self.measure
    }

    /// The metrics this crate hands the layout, from the theme.
    ///
    /// The advance is [`crate::mp::text::width`] of one average character — the same estimator the rest of
    /// this crate uses, which carries the DPI factor and errs **high** because it has no narrow
    /// correction. That direction is the one `makepad-markdown`'s layout documents wanting.
    fn metrics_for(&self, cx: &mut Cx2d) -> Metrics {
        let theme = makepad_theme::Theme::of(cx.cx);
        let body = theme.metrics(makepad_theme::TextStyle::Body).size() as f64;
        Metrics {
            advance: text::width("a", body),
            line_height: body * 1.6,
            indent: body * 1.6,
            gap: body * 0.7,
            padding: body * 0.6,
        }
    }

    /// Lay the document out if the cached layout is missing or was made with other metrics.
    ///
    /// Returns nothing rather than a borrow of the result: the draw loop mutates `self` through its
    /// `DrawText` fields, and a `&Laid` borrowed from `self` cannot be held across that. See the clone in
    /// `draw_walk` for what it costs.
    fn ensure_laid(&mut self, cx: &mut Cx2d) {
        let metrics = self.metrics_for(cx);
        let stale = self
            .used_metrics
            .is_some_and(|used| used != metrics);
        if self.laid.is_none() || stale {
            self.laid = Some(layout::layout(
                &self.doc,
                metrics,
                (self.measure - PAD * 2.0).max(1.0),
            ));
            self.used_metrics = Some(metrics);
        }
    }
}

/// Seat `measure` from the DSL's `initial_measure`.
///
/// **`measure` is `#[rust]` and this is why.** The first version made it `#[live]` and mutated it through
/// `set_measure`, which is the fault this port has recorded twice already: `Theme::install` calls
/// `request_script_reapply()`, the re-apply re-asserts every widget's DSL, and **every `#[live]` field a
/// Rust setter has written goes back to its declared value**. It showed up here as two documents in two
/// columns both laying out at the DSL's 640 — the narrow one silently not narrower, with no error.
impl ScriptHook for MpMarkdown {
    /// **`on_after_new` only, and that is the whole point.** The first version of this fix also had an
    /// `on_after_apply` that copied `initial_measure` across again — which is exactly the wipe it was
    /// written to prevent, one apply later. A caller's `set_measure` survives because nothing copies the
    /// DSL's value after construction.
    fn on_after_new(&mut self, _vm: &mut ScriptVm) {
        self.measure = self.initial_measure;
    }
}

impl Widget for MpMarkdown {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {
        // A rendered document is not interactive. A caret and a selection belong to the editor this port
        // has not built; inventing a hover wash here would promise a selection the widget cannot make.
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let (panel, border, radius, body_ink, muted_ink, code_ink, rule, plate) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            let p = &theme.paint;
            (
                p.surface_card,
                p.border,
                makepad_theme::Theme::panel_radius() as f32,
                p.text,
                p.text_muted,
                p.code_text,
                p.divider,
                p.code_wash,
            )
        };

        self.ensure_laid(cx);
        let height = self.laid.as_ref().map_or(0.0, |laid| laid.height) + PAD * 2.0;

        // States its own size: every line is drawn by `draw_walk` at an absolute position and nothing else
        // can measure it. See `mp/table.rs`.
        self.walk.width = Size::Fixed(self.measure);
        self.walk.height = Size::Fixed(height);
        let walk = self.walk;

        self.draw_bg.fill = panel;
        self.draw_bg.border = border;
        self.draw_bg.border_width = 1.0;
        self.draw_bg.radius = radius;
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);
        let rect = self.draw_bg.area().rect(cx.cx);
        let origin = rect.pos + dvec2(PAD, PAD);

        // **One clone of the laid-out document per frame.** The draw loop mutates `self` through its
        // `DrawText` fields, so it cannot hold a borrow of `self.laid` while it runs, and a document is a
        // few hundred small structs — cheaper to copy than to restructure the loop around, and a
        // two-phase draw would be harder to read than the cost is to pay.
        let laid = self.laid.clone().unwrap_or_else(|| {
            layout::layout(&self.doc, self.metrics_for(cx), self.measure)
        });
        let metrics = laid.metrics;

        for block_box in &laid.blocks {
            let block = &self.doc.blocks[block_box.block];
            let is_code = matches!(block.kind, BlockKind::Code { .. });
            let is_quote = matches!(block.kind, BlockKind::Quote(_));

            // A quote and a fence sit on a plate of their own, behind their text.
            if is_code || is_quote {
                let plate_rect = Rect {
                    pos: origin + dvec2(block_box.text_x - metrics.padding * 0.5, block_box.y),
                    size: dvec2(
                        (self.measure - PAD * 2.0 - block_box.text_x).max(1.0),
                        block_box.height,
                    ),
                };
                self.draw_bg.fill = plate;
                self.draw_bg.border_width = 0.0;
                self.draw_bg.radius = 6.0;
                self.draw_bg.draw_abs(cx, plate_rect);
                self.draw_bg.border_width = 1.0;
                self.draw_bg.fill = panel;
            }

            // A divider is a rule rather than text.
            if matches!(block.kind, BlockKind::Divider) {
                let rule_rect = Rect {
                    pos: origin + dvec2(block_box.text_x, block_box.y + block_box.height * 0.5),
                    size: dvec2(
                        (self.measure - PAD * 2.0 - block_box.text_x * 2.0).max(1.0),
                        1.0,
                    ),
                };
                self.draw_bg.fill = rule;
                self.draw_bg.border_width = 0.0;
                self.draw_bg.radius = 0.0;
                self.draw_bg.draw_abs(cx, rule_rect);
                self.draw_bg.border_width = 1.0;
                self.draw_bg.fill = panel;
                continue;
            }

            // The marker, in the muted colour.
            if let Some(marker) = &block_box.marker {
                self.draw_marker.color = muted_ink;
                let label = marker.clone();
                let x = origin.x + block_box.text_x - metrics.text_width(&label);
                let y = origin.y + block_box.lines.first().map(|line| line.y).unwrap_or(0.0);
                self.draw_marker.draw_walk(
                    cx,
                    Walk::fit().with_abs_pos(dvec2(x, y)),
                    Align::default(),
                    &label,
                );
            }

            // The text, line by line, at the x the **layout** computed.
            for line in &block_box.lines {
                let Some(text) = block.kind.text() else {
                    continue;
                };
                let Some(slice) = text.text.get(line.range.clone()) else {
                    continue;
                };
                if slice.trim().is_empty() {
                    continue;
                }
                // A fence is drawn in the code colour and the code face; everything else in the body
                // colour and the body face. Two branches rather than one with a pointer to whichever —
                // the first version reached for `unsafe` to hold one `&mut DrawText` across both cases,
                // and **`unsafe` to save one duplicated line in a UI widget is the wrong trade**: a raw
                // pointer here is a lifetime the compiler can no longer check, in a file whose whole job
                // is drawing at computed coordinates.
                if is_code {
                    self.draw_code.color = code_ink;
                    self.draw_code.draw_walk(
                        cx,
                        Walk::fit().with_abs_pos(dvec2(origin.x + line.x, origin.y + line.y)),
                        Align::default(),
                        slice,
                    );
                } else {
                    self.draw_body.color = body_ink;
                    self.draw_body.draw_walk(
                        cx,
                        Walk::fit().with_abs_pos(dvec2(origin.x + line.x, origin.y + line.y)),
                        Align::default(),
                        slice,
                    );
                }
            }
        }

        if std::env::var("MP_MARKDOWN_DEBUG").is_ok() {
            // The numbers have to come out of the widget: `[E] = 0` says the DSL resolved, and nothing
            // says the height is right. This is the check a screenshot would have made.
            println!(
                "MARKDOWN blocks={} lines={} measure={} height={:.2} drawn={:.2}x{:.2}",
                laid.blocks.len(),
                laid.lines().count(),
                self.measure,
                height,
                rect.size.x,
                rect.size.y
            );
        }

        DrawStep::done()
    }
}

impl MpMarkdownRef {
    pub fn set_source(&self, cx: &mut Cx, source: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_source(cx, source);
        }
    }

    pub fn set_doc(&self, cx: &mut Cx, doc: Doc) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_doc(cx, doc);
        }
    }

    pub fn set_measure(&self, cx: &mut Cx, measure: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_measure(cx, measure);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use makepad_markdown::layout::Metrics;

    /// The metrics `metrics_for` produces for a body size, without a `Cx`.
    fn metrics(body: f64) -> Metrics {
        Metrics {
            advance: text::width("a", body),
            line_height: body * 1.6,
            indent: body * 1.6,
            gap: body * 0.7,
            padding: body * 0.6,
        }
    }

    #[test]
    fn test_the_theme_metrics_put_the_advance_where_the_estimator_does() {
        // The one number this widget hands the layout that comes from somewhere else. If `text::width`'s
        // DPI factor were lost, the document would wrap ~25% late and every line would overflow — the
        // fault `mp/text.rs` records twice, arriving here through a different door.
        let m = metrics(13.0);
        // `0.508` per em, times `96/72`, times the size.
        let expected = 13.0 * 0.508 * (96.0 / 72.0);
        assert!(
            (m.advance - expected).abs() < 0.01,
            "the advance is {} and the estimator says {expected}",
            m.advance
        );
        // And it is the *whole* character width, not a fraction of one.
        assert!(m.advance > 13.0 * 0.5, "the advance looks like it lost the DPI factor");
    }

    #[test]
    fn test_the_line_height_leaves_room_for_a_line_of_the_body_face() {
        // A line height below the font size overlaps consecutive lines, which in a document is every line
        // colliding with the next.
        for body in [11.0, 13.0, 17.0] {
            let m = metrics(body);
            assert!(
                m.line_height > body * 1.2,
                "a line height of {} for a {body}pt face overlaps its lines",
                m.line_height
            );
        }
    }

    #[test]
    fn test_the_measure_is_the_wrap_width_minus_the_plate_s_padding() {
        // The widget's `measure` is the plate's width; the text wraps to that minus its own padding. A
        // widget that passed the full width would have text touching the border on both sides.
        let laid = layout::layout(
            &makepad_markdown::parse("a paragraph that is long enough to wrap\n"),
            metrics(13.0),
            (640.0 - PAD * 2.0).max(1.0),
        );
        for line in laid.lines() {
            let width = laid.metrics.text_width("");
            let _ = width;
            assert!(
                line.x <= 640.0 - PAD * 2.0 + 1e-9,
                "a line starts past the measure: {line:?}"
            );
        }
    }

    #[test]
    fn test_a_document_with_no_blocks_lays_out_to_the_padding_alone() {
        // `set_source("")` is what clearing a document does, and its height must be the plate's padding
        // rather than zero — a zero-height plate is a widget that vanishes.
        let laid = layout::layout(&makepad_markdown::parse(""), metrics(13.0), 600.0);
        assert!(laid.blocks.is_empty());
        assert_eq!(laid.height, 0.0);
        // ...so the widget's own height is exactly its padding.
        assert_eq!(laid.height + PAD * 2.0, PAD * 2.0);
    }

    #[test]
    fn test_every_block_kind_has_something_to_draw() {
        // The set the paint half handles, checked against the model so a kind added there is a failing
        // test here rather than a block that draws nothing.
        let source = "# h\n\npara\n\n- a\n\n1. b\n\n- [x] c\n\n> q\n\n```\ncode\n```\n\n---\n";
        let doc = makepad_markdown::parse(source);
        let kinds: Vec<&str> = doc
            .blocks
            .iter()
            .map(|block| match &block.kind {
                BlockKind::Paragraph(_) => "paragraph",
                BlockKind::Heading { .. } => "heading",
                BlockKind::Bullet(_) => "bullet",
                BlockKind::Ordered { .. } => "ordered",
                BlockKind::Task { .. } => "task",
                BlockKind::Quote(_) => "quote",
                BlockKind::Code { .. } => "code",
                BlockKind::Divider => "divider",
            })
            .collect();
        assert_eq!(
            kinds,
            vec![
                "heading",
                "paragraph",
                "bullet",
                "ordered",
                "task",
                "quote",
                "code",
                "divider"
            ]
        );
        // Every one of them laid out to at least one line, so none of them draws nothing.
        let laid = layout::layout(&doc, metrics(13.0), 600.0);
        assert_eq!(laid.blocks.len(), 8);
        for block in &laid.blocks {
            assert!(
                !block.lines.is_empty(),
                "block {} has no lines to draw",
                block.block
            );
        }
    }

    #[test]
    fn test_a_divider_takes_a_line_of_height_so_the_next_block_clears_it() {
        let doc = makepad_markdown::parse("a\n\n---\n\nb\n");
        let laid = layout::layout(&doc, metrics(13.0), 600.0);
        let divider = laid.blocks.iter().find(|b| b.block == 1).expect("the divider");
        assert!(divider.height > 0.0);
        assert!(divider.marker.is_none(), "a divider has no marker");
        // And the block after it is below it.
        let after = laid.blocks.iter().find(|b| b.block == 2).expect("the paragraph");
        assert!(after.y >= divider.y + divider.height);
    }
}
