//! `MpCodeBlock` — source, painted with the theme's highlight kinds.
//!
//! ## It does not classify anything
//!
//! [`MpCodeBlock::set_highlighted`] takes the source **and** its spans. The classifier lives in
//! `makepad-syntax`, and this widget is deliberately not a client of it: a widget that reached for a
//! tokenizer would have to pick one, and the classifier is the one part of this stack that is
//! explicitly allowed to change. The app wires the two together, which is one line and keeps the
//! layering one-directional.
//!
//! ## Why it draws its own runs
//!
//! `DrawText` paints **one** string in **one** colour, so a line with four kinds on it is four
//! draw calls. That is why this widget exists rather than a `Label` per span: a column of labels
//! breaks the line layout, and a code block is read by lines.
//!
//! So the pen is advanced by hand — which is what [`crate::mp::text::Face::Mono`] is for. A
//! monospace advance is **exact** (one advance per character), so the runs butt up against each
//! other with no gaps and no overlap; a proportional estimate would leave a seam wherever it was
//! under and a collision wherever it was over.
//!
//! ## The traps, all of which are tested
//!
//! - **A span's byte range can straddle a line break.** A classifier reports byte offsets into the
//!   whole document, and a multi-line token — a block comment, a template string — covers newlines.
//!   The run for such a span has to be **clipped to the line** and continue on the next one, which is
//!   the fault that would otherwise put the rest of the document on one line.
//! - **The monotone advance means a run's x is the sum of the widths before it**, not something a
//!   layout engine computes. That is deliberate and it is why `line_runs` is a pure function with
//!   tests rather than a loop inside `draw_walk`.
//! - **A span that covers nothing on this line produces no run**, so a gap in the document is a gap
//!   in the paint — which is correct, because text outside the spans is unhighlighted.

use makepad_widgets::*;

use makepad_theme::syntax::{Highlight, HighlightKind, SyntaxPalette};

use crate::mp::text::{self, Face};

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    /// The block's plate. One shader drawn twice would be a second copy of `sdf.box`, so this is the
    /// same plate `mp/date.rs` and `mp/list.rs` use — a fill, a hairline and a corner.
    set_type_default() do #(DrawMpCodeBlock::script_shader(vm)){
        ..mod.draw.DrawQuad

        fill: #x00000000
        border: #x00000000
        border_width: 0.0
        radius: 8.0

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

    mod.mp.MpCodeBlockBase = #(MpCodeBlock::register_widget(vm))

    mod.mp.MpCodeBlock = set_type_default() do mod.mp.MpCodeBlockBase{
        width: Fill
        height: Fit

        draw_bg +: {
            fill: code_wash
            border: border
            border_width: 1.0
            radius: 8.0
        }
        draw_code +: {
            // The code face, which is the one a monospace advance describes.
            text_style: theme.font_code{font_size: 12.0}
            color: #x00000000
        }
    }
}

/// A run of source on **one line**, with the colour it paints in.
#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    /// The run's text, already clipped to its line.
    pub text: String,
    pub kind: HighlightKind,
    /// Where the run starts on its line, in **characters**.
    ///
    /// Characters rather than bytes because the pen advances by character: a monospace face gives
    /// every character one advance whatever its width in bytes, so `héllo` and `hello` put their
    /// second run at the same x.
    pub column: usize,
}

/// Split a line into the runs to paint on it.
///
/// `line_start` is the byte offset of the line in the **document**, because a span's range is a
/// document range. Every span that touches the line contributes a run, clipped to the line — see the
/// module doc on multi-line tokens.
pub fn line_runs(line: &str, line_start: usize, spans: &[Highlight]) -> Vec<Run> {
    let line_end = line_start + line.len();
    let mut runs = Vec::new();
    for span in spans {
        // Half-open ranges: a span that ends exactly where the line starts touches nothing, and one
        // that starts exactly where the line ends touches nothing either. The strict comparisons are
        // what stop a newline's own byte from drawing a zero-width run on the next line.
        if span.range.end <= line_start || span.range.start >= line_end {
            continue;
        }
        let start = span.range.start.max(line_start);
        let end = span.range.end.min(line_end);
        let Some(text) = line.get(start - line_start..end - line_start) else {
            // A range that is not on a character boundary. Dropped rather than panicking: a
            // classifier reporting a byte offset inside a multi-byte character is a fault worth
            // seeing as a missing run rather than as a crash in the paint.
            continue;
        };
        if text.is_empty() {
            continue;
        }
        let column = line[..start - line_start].chars().count();
        runs.push(Run {
            text: text.to_string(),
            kind: span.kind,
            column,
        });
    }
    runs.sort_by_key(|run| run.column);
    runs
}

/// Where a run's pen starts, given the gap the spans left before it.
///
/// A gap in the spans is a gap in the paint, so a run after unspanned text starts further along —
/// which is the one place a column is needed, since the pen otherwise advances by measurement. The
/// `advance` here is the estimate, and it is only ever used to cross a **gap**: a few points of error
/// in unspanned whitespace is invisible, where the same error under a run caused progressive overlap.
fn run_start_x(column: usize, advance: f64) -> f64 {
    column as f64 * advance
}

/// How many columns wide the source is, for a block that states its own width.
pub fn widest_line(source: &str) -> usize {
    source
        .split('\n')
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0)
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpCodeBlock {
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
pub struct MpCodeBlock {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_bg: DrawMpCodeBlock,
    #[live]
    draw_code: DrawText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    /// The source, line-split once.
    #[rust]
    lines: Vec<String>,
    /// Where each line starts in the document, so a document-range span can be clipped to it.
    #[rust]
    line_starts: Vec<usize>,
    /// The classification, as the caller supplied it.
    #[rust]
    spans: Vec<Highlight>,
    /// The palette, read from the theme on every draw rather than stored — a stored copy is a copy
    /// that survives an appearance change.
    #[rust]
    declared_label: String,
}

impl MpCodeBlock {
    /// Set the source and its spans.
    ///
    /// The spans are **not** normalized here: [`makepad_theme::syntax::normalize`] is the
    /// classifier's responsibility, and normalizing again in the widget would hide a classifier that
    /// forgot. `line_runs` tolerates overlap either way — it clips every span to the line
    /// independently — so a caller that skips normalization paints the later span over the earlier
    /// one rather than doing anything worse.
    pub fn set_highlighted(&mut self, cx: &mut Cx, source: &str, spans: &[Highlight]) {
        self.lines = source.split('\n').map(|line| line.to_string()).collect();
        self.line_starts.clear();
        let mut at = 0usize;
        for line in &self.lines {
            self.line_starts.push(at);
            at += line.len() + 1; // the newline this line ended with
        }
        self.spans = spans.to_vec();
        self.redraw(cx);
    }

    /// Replace only the spans, for a caller that re-classified the same source.
    pub fn set_spans(&mut self, cx: &mut Cx, spans: &[Highlight]) {
        self.spans = spans.to_vec();
        self.redraw(cx);
    }

    pub fn source_lines(&self) -> &[String] {
        &self.lines
    }

    pub fn spans(&self) -> &[Highlight] {
        &self.spans
    }

    /// What the widget is holding, for a page that reports it rather than only painting it.
    pub fn label(&self) -> &str {
        &self.declared_label
    }

    pub fn set_label(&mut self, cx: &mut Cx, label: &str) {
        self.declared_label = label.to_string();
        self.redraw(cx);
    }

    /// The height one line occupies, at the size the pen advances by.
    fn line_height(font: f64) -> f64 {
        font * 1.5
    }

    /// The width the block needs: the widest line, in the monospace advance.
    fn content_width(&self, font: f64) -> f64 {
        text::width_in(
            &"m".repeat(widest_line_of(&self.lines)),
            font,
            Face::Mono,
        ) + 24.0
    }
}

/// The widest line, in characters.
fn widest_line_of(lines: &[String]) -> usize {
    lines.iter().map(|line| line.chars().count()).max().unwrap_or(0)
}

/// Empty for the same reason `MpSegmented`'s is: no animator to seat, nothing to place.
impl ScriptHook for MpCodeBlock {
    fn on_after_new(&mut self, _vm: &mut ScriptVm) {}
}

impl Widget for MpCodeBlock {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {
        // A code block is not interactive. It has no hover, no selection and no caret — those belong
        // to the editor this port has not built, and inventing a hover wash here would promise a
        // selection the widget cannot make.
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let (panel, border, radius) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            let p = &theme.paint;
            (
                p.code_wash,
                p.border,
                makepad_theme::Theme::panel_radius() as f32,
            )
        };
        // Read every draw rather than stored, so an appearance change reaches it. The palette is
        // cheap to build and a cached copy is a cache that outlives the thing it cached.
        let palette = SyntaxPalette::for_appearance(makepad_theme::Theme::of(cx.cx).appearance);

        let font = self.draw_code.text_style.font_size as f64;
        let line_height = Self::line_height(font);

        // States its own size: every line is drawn by `draw_abs` and nothing else can measure it.
        // See `mp/table.rs`.
        self.walk.width = Size::Fixed(self.content_width(font));
        self.walk.height = Size::Fixed(self.lines.len() as f64 * line_height + 24.0);
        let walk = self.walk;

        self.draw_bg.fill = panel;
        self.draw_bg.border = border;
        self.draw_bg.border_width = 1.0;
        self.draw_bg.radius = radius;
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);

        let rect = self.draw_bg.area().rect(cx.cx);
        let advance = text::width_in("m", font, Face::Mono);
        if std::env::var("MP_CODE_DEBUG").is_ok() {
            // The numbers have to come out of the widget: a pen that advances too little produces
            // progressive overlap, which a screenshot shows as garbled text and a log shows not at
            // all.
            let widest = widest_line_of(&self.lines);
            println!(
                "CODE font={font} advance={advance:.3} widest={widest}                  computed_w={:.2} drawn_w={:.2} drawn_h={:.2} lines={}",
                widest as f64 * advance + 24.0,
                rect.size.x,
                rect.size.y,
                self.lines.len()
            );
        }
        let origin = rect.pos + dvec2(12.0, 12.0);

        for (index, line) in self.lines.iter().enumerate() {
            let line_start = self.line_starts[index];
            let runs = line_runs(line, line_start, &self.spans);
            let y = origin.y + index as f64 * line_height;
            if runs.is_empty() {
                // A line with no spans is unhighlighted text. Nothing to draw, and drawing it in the
                // body colour would be a *third* colour the palette does not have.
                continue;
            }
            // **The pen advances by the rect the draw returns, not by an estimate.** `draw_walk`
            // returns the `Rect` the run occupied, so the next run starts exactly where this one
            // ended — for any face, at any size, with no measurement of its own.
            //
            // The first version advanced by `text::width_in(.., Mono)`, which is an *estimate*, and
            // it came out under: the runs **overlapped progressively down each line** and the
            // longest overflowed the panel. A screenshot showed it as garbled text; nothing in the
            // log showed it at all. `run.column` is still used for the line's starting x, because a
            // gap in the spans is a gap in the paint.
            let first_column = runs.first().map(|run| run.column).unwrap_or(0);
            let mut x = origin.x + run_start_x(first_column, advance);
            for run in runs {
                self.draw_code.color = palette.color(run.kind);
                let drawn = self.draw_code.draw_walk(
                    cx,
                    Walk::fit().with_abs_pos(dvec2(x, y)),
                    Align::default(),
                    &run.text,
                );
                if std::env::var("MP_CODE_DEBUG").is_ok() && run.text.chars().count() > 1 {
                    let per_char = drawn.size.x / run.text.chars().count() as f64;
                    println!(
                        "CODE run {:?} chars={} drawn_w={:.2} per_char={per_char:.4} \
                         estimated={:.4}",
                        run.text,
                        run.text.chars().count(),
                        drawn.size.x,
                        advance,
                    );
                }
                // Advanced by what was drawn, and by nothing else.
                x = drawn.pos.x + drawn.size.x;
            }
        }

        DrawStep::done()
    }
}

impl MpCodeBlockRef {
    pub fn set_highlighted(&self, cx: &mut Cx, source: &str, spans: &[Highlight]) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_highlighted(cx, source, spans);
        }
    }

    pub fn set_spans(&self, cx: &mut Cx, spans: &[Highlight]) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_spans(cx, spans);
        }
    }

    pub fn set_label(&self, cx: &mut Cx, label: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_label(cx, label);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(start: usize, end: usize, kind: HighlightKind) -> Highlight {
        Highlight {
            range: start..end,
            kind,
        }
    }

    fn columns(runs: &[Run]) -> Vec<usize> {
        runs.iter().map(|run| run.column).collect()
    }

    fn texts(runs: &[Run]) -> Vec<&str> {
        runs.iter().map(|run| run.text.as_str()).collect()
    }

    #[test]
    fn test_a_line_splits_into_its_runs_in_column_order() {
        // `{"a": 1}` with a key and a number — the shape a JSON line has.
        let line = "{\"a\": 1}";
        let spans = [
            span(0, 1, HighlightKind::Punctuation),
            span(1, 4, HighlightKind::Attribute),
            span(4, 5, HighlightKind::Punctuation),
            span(6, 7, HighlightKind::Number),
            span(7, 8, HighlightKind::Punctuation),
        ];
        let runs = line_runs(line, 0, &spans);
        assert_eq!(texts(&runs), vec!["{", "\"a\"", ":", "1", "}"]);
        assert_eq!(columns(&runs), vec![0, 1, 4, 6, 7]);
        assert_eq!(runs[1].kind, HighlightKind::Attribute);
        assert_eq!(runs[3].kind, HighlightKind::Number);
    }

    #[test]
    fn test_a_span_straddling_a_line_break_is_clipped_to_each_line() {
        // **The fault that would put the rest of the document on one line.** A block comment or a
        // template string is one span covering several newlines; the pen has to continue it on the
        // next line rather than draw the newline as a glyph.
        let source = "/* one\ntwo */\nafter";
        let comment = span(0, 13, HighlightKind::Comment);
        let spans = [comment];

        let first = line_runs("/* one", 0, &spans);
        assert_eq!(texts(&first), vec!["/* one"]);
        assert_eq!(first[0].kind, HighlightKind::Comment);

        // The second line starts at byte 7, after the newline.
        let second = line_runs("two */", 7, &spans);
        assert_eq!(texts(&second), vec!["two */"]);
        assert_eq!(second[0].column, 0, "the continuation starts at the line's left edge");

        // The third line is past the span's end, so it has nothing.
        assert!(line_runs("after", 14, &spans).is_empty());
    }

    #[test]
    fn test_a_span_that_ends_exactly_at_a_line_break_does_not_reach_the_next_line() {
        // The half-open-range case, which is one byte wide and would draw a phantom run: a span
        // ending at the newline's own offset touches the next line under a `<=` comparison.
        let source = "ab\ncd";
        let spans = [span(0, 2, HighlightKind::String)];
        assert_eq!(texts(&line_runs("ab", 0, &spans)), vec!["ab"]);
        assert!(
            line_runs("cd", 3, &spans).is_empty(),
            "the span reached past its own end"
        );
    }

    #[test]
    fn test_a_span_touching_a_line_only_from_before_contributes_a_run() {
        // The mirror case: a span that starts before this line and ends inside it. This is the
        // second half of a multi-line token, reached from the *other* direction.
        let spans = [span(0, 10, HighlightKind::Comment)];
        let runs = line_runs("bcd", 5, &spans);
        assert_eq!(texts(&runs), vec!["bcd"]);
        assert_eq!(runs[0].column, 0);
    }

    #[test]
    fn test_a_gap_between_spans_is_a_gap_in_the_paint() {
        // Text outside the spans is unhighlighted, so a line whose middle is unspanned has two runs
        // with a hole between them — and the columns say where the hole is.
        let line = "abcdef";
        let spans = [
            span(0, 1, HighlightKind::Keyword),
            span(4, 6, HighlightKind::String),
        ];
        let runs = line_runs(line, 0, &spans);
        assert_eq!(texts(&runs), vec!["a", "ef"]);
        assert_eq!(columns(&runs), vec![0, 4], "the hole is not filled in");
    }

    #[test]
    fn test_columns_are_characters_so_a_multi_byte_prefix_does_not_shift_the_pen() {
        // A monospace face gives every character one advance whatever its width in bytes, so a run
        // after `héllo` starts at column 5 — not at byte 6. A byte-based column would leave a gap
        // the width of the extra byte, and only on lines with accents in them.
        let line = "héllo: 1";
        // `héllo` is 6 bytes, so the colon is at byte 6 and the `1` at byte 8.
        let spans = [
            span(0, 6, HighlightKind::Variable),
            span(6, 7, HighlightKind::Punctuation),
            span(8, 9, HighlightKind::Number),
        ];
        let runs = line_runs(line, 0, &spans);
        assert_eq!(texts(&runs), vec!["héllo", ":", "1"]);
        assert_eq!(columns(&runs), vec![0, 5, 7]);
        // And each column really is where the text is, counted in characters.
        assert_eq!(line.chars().nth(5), Some(':'));
        assert_eq!(line.chars().nth(7), Some('1'));
    }

    #[test]
    fn test_a_span_that_is_not_on_a_character_boundary_is_dropped_rather_than_panicking() {
        // A classifier reporting a byte offset inside a multi-byte character is a fault worth seeing
        // as a missing run rather than as a crash in the paint.
        let line = "héllo";
        // `h` is byte 0, so `é` occupies bytes 1 and 2 and `l` starts at 3. Byte **2** is therefore
        // the second byte of `é` and not a boundary, so `2..4` cannot be sliced. The first version of
        // this test used `1..3`, which is on boundaries on both sides and slices cleanly to `é` — the
        // library was right and the byte arithmetic was mine.
        assert!(!line.is_char_boundary(2), "the test's premise is wrong for this string");
        let spans = [span(2, 4, HighlightKind::String)];
        assert!(line_runs(line, 0, &spans).is_empty());
        // ...and the boundary case really does work, so this is not a test of a function that drops
        // everything.
        assert_eq!(texts(&line_runs(line, 0, &[span(1, 3, HighlightKind::String)])), vec!["é"]);
    }

    #[test]
    fn test_runs_come_back_in_column_order_whatever_order_the_spans_were_in() {
        // The classifier normalizes, so this should not happen — but a widget that painted in
        // whatever order it was handed would draw an earlier run over a later one, and the sort is
        // one line.
        let line = "abcdef";
        let spans = [
            span(4, 6, HighlightKind::String),
            span(0, 2, HighlightKind::Keyword),
            span(2, 4, HighlightKind::Number),
        ];
        assert_eq!(columns(&line_runs(line, 0, &spans)), vec![0, 2, 4]);
    }

    #[test]
    fn test_the_widest_line_is_measured_in_characters() {
        assert_eq!(widest_line("ab\ncdef\n"), 4);
        assert_eq!(widest_line(""), 0);
        assert_eq!(widest_line("héllo"), 5, "five characters, six bytes");
    }

    #[test]
    fn test_the_monospace_advance_is_exact_where_the_proportional_one_is_an_estimate() {
        // The property the pen depends on: in a monospace face a run's width is its character count
        // times one advance, so `text::width_in(.., Mono)` is not an estimate at all. Four `i`s and
        // four `m`s measure the same, which the proportional face deliberately does not do.
        let chars = "iiii";
        let wide = "mmmm";
        assert_eq!(
            text::width_in(chars, 12.0, Face::Mono),
            text::width_in(wide, 12.0, Face::Mono)
        );
        assert_ne!(
            text::width(chars, 12.0),
            text::width(wide, 12.0),
            "the proportional estimator is supposed to tell these apart"
        );
        // And it is the character count times the advance, exactly.
        assert_eq!(
            text::width_in("abcd", 10.0, Face::Mono),
            4.0 * 10.0 * text::MONO_ADVANCE
        );
    }
}
