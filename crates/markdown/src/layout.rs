//! Laying a document out: line breaks, block positions, the caret, and hit testing.
//!
//! ## Why this is separate from the painting
//!
//! Everything here is arithmetic on a [`Doc`] and a [`Metrics`], so all of it is testable with no
//! window — which matters for this crate more than usual, because a document's layout is where the
//! faults are: a line break in the wrong place, a caret one character off, a click landing between two
//! blocks. What the paint half then does is place glyphs at the positions this computes.
//!
//! ## The widths are an estimate, and they err **high** on purpose
//!
//! A character's width here is `advance`, doubled for the characters that are full-width in the faces
//! this port bundles (CJK, emoji). That is an approximation of a proportional face, and it is
//! deliberately **one-sided**: there is no narrow correction, so a document full of `i` and `l` wraps
//! earlier than it has to. The reason is the one `mp/text.rs` paid for twice — an estimate that comes
//! out **under** puts content past the edge it was laid out to, while one that comes out over leaves a
//! little air. Wrapping a word early is invisible; overflowing the measure is not.
//!
//! A real per-glyph measure belongs to the paint half, which has the fonts; this half has a number and
//! a promise about which way its error goes.
//!
//! ## Byte offsets throughout
//!
//! A [`Doc`]'s text carries marks as **byte ranges**, so the layout reports byte ranges too. Every
//! offset this module produces is on a character boundary — asserted by its own tests, because an offset
//! inside a multi-byte character is a panic in a slicing renderer rather than a wrong pixel.

use std::ops::Range;

use crate::{BlockKind, Doc, Text};

/// The numbers a layout needs, all of them the *environment's*.
///
/// A value rather than a theme lookup, because this module is pure and has no `Cx`. The caller — the
/// paint half, which has a theme — fills it in, which keeps the one place that knows a font size
/// separate from the arithmetic that uses it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    /// One character's width at [`Metrics::body_size`], in points.
    ///
    /// A **size-relative** number: a heading's characters are wider because its size is larger, so the width of a
    /// character is `advance * size / body_size`. That is what lets one advance serve every block without a second
    /// table.
    pub advance: f64,
    /// The size of a paragraph, in points. Every other size is stated against it.
    pub body_size: f64,
    /// The size of a level 1, 2 and 3 heading, in points.
    ///
    /// A **heading is bigger**, and that is not only a paint decision: a taller block occupies more vertical space,
    /// so a layout that used one line height for everything would paint a heading over the block below it. It was
    /// the one thing a screenshot found that no test could — the marked-up heading looked exactly like a
    /// paragraph — because the *layout* had no notion of a size per kind.
    pub heading_size: [f64; 3],
    /// One line's height at [`Metrics::body_size`], in points.
    pub line_height: f64,
    /// One indent level, in points.
    pub indent: f64,
    /// The vertical space between two blocks, in points.
    pub gap: f64,
    /// The margin inside a quote or a code block, in points.
    pub padding: f64,
}

impl Default for Metrics {
    fn default() -> Self {
        // Chosen for a 13pt body at a 1.5 line height, and public so a caller that knows better can say
        // so. The paint half overrides every one of these from the theme.
        Self {
            advance: 13.0 * 0.6 * (96.0 / 72.0),
            body_size: 13.0,
            heading_size: [22.0, 17.0, 15.0],
            line_height: 13.0 * 1.5,
            indent: 22.0,
            gap: 10.0,
            padding: 8.0,
        }
    }
}

impl Metrics {
    /// The painted size of a block's text.
    ///
    /// A heading is larger than a paragraph, and levels beyond three are clamped: this port's type ladder has three
    /// title rungs, and a level 4–6 heading is a *heading* rather than a new size.
    pub fn size_for(&self, kind: &BlockKind) -> f64 {
        match kind {
            BlockKind::Heading { level, .. } => {
                self.heading_size[(*level as usize).clamp(1, 3) - 1]
            }
            _ => self.body_size,
        }
    }

    /// One line's height for a block, so a taller block takes more vertical space.
    pub fn line_height_for(&self, kind: &BlockKind) -> f64 {
        // The same ratio as the body's, so a heading is scaled rather than separately chosen — one number instead
        // of four.
        self.line_height * (self.size_for(kind) / self.body_size)
    }

    /// How wide one character is at a block's size, with the full-width characters doubled.
    ///
    /// See the module doc on why there is no narrow correction.
    pub fn char_width_at(&self, ch: char, size: f64) -> f64 {
        let base = self.advance * (size / self.body_size);
        if is_full_width(ch) {
            base * 2.0
        } else {
            base
        }
    }

    /// How wide one character is at the body size.
    pub fn char_width(&self, ch: char) -> f64 {
        self.char_width_at(ch, self.body_size)
    }

    /// How wide a string is at a block's size.
    pub fn text_width_at(&self, text: &str, size: f64) -> f64 {
        text.chars().map(|ch| self.char_width_at(ch, size)).sum()
    }

    /// How wide a string is at the body size.
    pub fn text_width(&self, text: &str) -> f64 {
        self.text_width_at(text, self.body_size)
    }
}

/// Whether a character takes two advances.
///
/// The ranges are the ones the faces this port bundles draw full-width: CJK, Hangul, Kana, and the
/// emoji blocks. A miss here is a line that wraps late, so the list is generous rather than exact.
fn is_full_width(ch: char) -> bool {
    matches!(ch as u32,
        0x1100..=0x115F      // Hangul Jamo
        | 0x2E80..=0x303E    // CJK radicals, Kangxi, CJK symbols
        | 0x3041..=0x33FF    // Hiragana, Katakana, Bopomofo, CJK compatibility
        | 0x3400..=0x4DBF    // CJK extension A
        | 0x4E00..=0x9FFF    // CJK unified ideographs
        | 0xA000..=0xA4CF    // Yi
        | 0xAC00..=0xD7A3    // Hangul syllables
        | 0xF900..=0xFAFF    // CJK compatibility ideographs
        | 0xFE30..=0xFE6F    // CJK compatibility forms
        | 0xFF00..=0xFF60    // full-width forms
        | 0xFFE0..=0xFFE6
        | 0x1F300..=0x1F64F  // emoji
        | 0x1F900..=0x1F9FF
        | 0x20000..=0x3FFFD  // CJK extensions B and beyond
    )
}

/// One laid-out line of one block.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    /// The bytes of the block's own text on this line.
    pub range: Range<usize>,
    /// The line's left edge, in the document's coordinates — a block's indent, plus a quote's padding.
    pub x: f64,
    /// The line's baseline-to-baseline top, in the document's coordinates.
    pub y: f64,
}

/// Where a block ended up.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockBox {
    /// The block's index in the document.
    pub block: usize,
    /// The block's top, in the document's coordinates.
    pub y: f64,
    pub height: f64,
    /// The marker drawn before a list item or a quote — `- `, `1. `, `> `. The **layout** decides it
    /// because it takes horizontal room, and a paint half that decided it too would be a second place
    /// that has to agree about how wide it is.
    pub marker: Option<String>,
    /// How far the text starts from the block's own left edge, past the marker and any padding.
    pub text_x: f64,
    /// The wrapped lines of the block's own text, in order.
    pub lines: Vec<Line>,
}

/// A laid-out document.
#[derive(Clone, Debug, PartialEq)]
pub struct Laid {
    pub metrics: Metrics,
    /// The measure the blocks were wrapped to.
    pub width: f64,
    pub blocks: Vec<BlockBox>,
    /// The document's total height, which is the last block's bottom.
    pub height: f64,
}

impl Laid {
    /// Every line of every block, in document order.
    pub fn lines(&self) -> impl Iterator<Item = &Line> {
        self.blocks.iter().flat_map(|block| block.lines.iter())
    }

    /// The block a document coordinate falls in.
    ///
    /// `None` above the first block or below the last, and clamped to the nearest block in between —
    /// a click in the gap under a block belongs to that block rather than to nothing, because a reader
    /// clicking between two paragraphs means the one they can see under the pointer.
    pub fn block_at(&self, y: f64) -> Option<usize> {
        if self.blocks.is_empty() {
            return None;
        }
        if y < self.blocks[0].y {
            return Some(0);
        }
        // The last block whose **top** is at or above the click. Inside a block that is the block; in the
        // gap under one it is still that block, which is what the doc says and what an editor does — the
        // first version tested `y < block.y + block.height`, so a click in the gap fell through to the
        // *next* block and the doc was a lie.
        let mut found = 0usize;
        for (index, block) in self.blocks.iter().enumerate() {
            if block.y <= y {
                found = index;
            }
        }
        Some(found)
    }

    /// The line a document coordinate falls on, with the block it belongs to.
    pub fn line_at(&self, y: f64) -> Option<(usize, usize)> {
        let block = self.block_at(y)?;
        let block_box = &self.blocks[block];
        if block_box.lines.is_empty() {
            return None;
        }
        for (index, line) in block_box.lines.iter().enumerate() {
            if y < line.y + self.metrics.line_height {
                return Some((block, index));
            }
        }
        Some((block, block_box.lines.len() - 1))
    }
}

/// Lay a document out.
///
/// Blocks are stacked top to bottom with [`Metrics::gap`] between them; each block's text is wrapped to
/// `width` minus its own indent, its marker and its padding.
pub fn layout(doc: &Doc, metrics: Metrics, width: f64) -> Laid {
    let mut blocks = Vec::with_capacity(doc.blocks.len());
    let mut y = 0.0f64;

    for (index, block) in doc.blocks.iter().enumerate() {
        if index > 0 {
            y += metrics.gap;
        }
        let indent = block.indent as f64 * metrics.indent;
        let marker = marker_for(&block.kind);
        let marker_width = marker
            .as_ref()
            .map(|marker| metrics.text_width(marker))
            .unwrap_or(0.0);
        // A quote and a code block carry a margin of their own, so their text is inset further than a
        // list's marker takes.
        let padding = match block.kind {
            BlockKind::Quote(_) | BlockKind::Code { .. } => metrics.padding,
            _ => 0.0,
        };
        let text_x = indent + marker_width + padding;
        let size = metrics.size_for(&block.kind);
        let line_height = metrics.line_height_for(&block.kind);
        let available = (width - text_x - indent.min(0.0)).max(metrics.advance);

        let text = block.kind.text();
        let ranges = match text {
            // A `Divider` has no text and one line of its own.
            None => Vec::new(),
            // **A fence is not rewrapped.** Rewrapping code changes what the reader reads — a document's
            // job is to show the characters that are there — so its lines are its own newlines however
            // wide they are, and the paint half clips a long one instead. The first version wrapped it
            // like a paragraph and the test caught the code contradicting this doc's neighbour.
            Some(text) if matches!(block.kind, BlockKind::Code { .. }) => hard_lines(&text.text),
            Some(text) => wrap(text, &metrics, size, available),
        };
        let lines = if ranges.is_empty() {
            vec![Line {
                range: 0..0,
                x: indent,
                y: y + line_height * 0.5,
            }]
        } else {
            ranges
                .into_iter()
                .enumerate()
                .map(|(line_index, range)| Line {
                    range,
                    x: indent + marker_width + padding,
                    y: y + line_index as f64 * line_height,
                })
                .collect()
        };

        // A code block's height is its line count, and it is not wrapped — the paint half clips it
        // horizontally instead, because rewrapping code changes what the reader reads. So its lines are
        // its own newlines.
        let height = lines.len().max(1) as f64 * line_height;
        blocks.push(BlockBox {
            block: index,
            y,
            height,
            marker,
            text_x,
            lines,
        });
        y += height;
    }

    Laid {
        metrics,
        width,
        blocks,
        height: y,
    }
}

/// The marker drawn before a block's text, if it has one.
fn marker_for(kind: &BlockKind) -> Option<String> {
    match kind {
        BlockKind::Bullet(_) => Some("- ".to_string()),
        BlockKind::Ordered { number, .. } => Some(format!("{number}. ")),
        BlockKind::Task { checked, .. } => {
            Some(if *checked { "[x] " } else { "[ ] " }.to_string())
        }
        BlockKind::Quote(_) => Some("> ".to_string()),
        _ => None,
    }
}

/// Break one block's text into lines that fit `width`.
///
/// **Breaks at spaces where it can and at characters where it must**, which is what a reader expects:
/// a paragraph breaks between words, and a URL or an identifier longer than the measure breaks inside
/// itself rather than overflowing. The returned ranges are on character boundaries and cover the text in
/// order, with the space that ended a line left at the end of that line rather than moved to the next.
pub fn wrap(text: &Text, metrics: &Metrics, size: f64, width: f64) -> Vec<Range<usize>> {
    let source = text.text.as_str();
    if source.is_empty() {
        // An empty paragraph is still a line — otherwise a blank block would have no height and the
        // caret could not sit in it, which is the state an editor is in the moment Enter is pressed.
        return vec![0..0];
    }

    let mut lines = Vec::new();
    let mut line_start = 0usize;
    let mut used = 0.0f64;
    // The last byte offset at which this line could break: the position just after a space.
    let mut last_break: Option<usize> = None;

    let mut index = 0usize;
    while index < source.len() {
        let ch = source[index..].chars().next().expect("a character");
        let ch_width = metrics.char_width_at(ch, size);

        // A newline is a hard break: this is a code block or a quote's internal line.
        if ch == '\n' {
            lines.push(line_start..index + 1);
            index += 1;
            line_start = index;
            used = 0.0;
            last_break = None;
            continue;
        }

        if used + ch_width > width && index > line_start {
            // Break at the last space if there was one, otherwise here. The space stays with the line it
            // ended, so the next line does not begin with a space that would shift it.
            let break_at = last_break.unwrap_or(index);
            lines.push(line_start..break_at);
            line_start = break_at;
            used = metrics.text_width_at(&source[line_start..index], size);
            last_break = None;
        }

        used += ch_width;
        index += ch.len_utf8();
        if ch == ' ' {
            last_break = Some(index);
        }
    }
    if line_start < source.len() {
        lines.push(line_start..source.len());
    }
    if lines.is_empty() {
        lines.push(0..0);
    }
    lines
}

/// Split text at its newlines and nowhere else, keeping each newline on the line it ended.
///
/// The code-block rule: an editor shows a long line as a long line. See the note in [`layout`].
pub fn hard_lines(text: &str) -> Vec<Range<usize>> {
    if text.is_empty() {
        return vec![0..0];
    }
    let mut lines = Vec::new();
    let mut start = 0usize;
    for (index, ch) in text.char_indices() {
        if ch == '\n' {
            lines.push(start..index + 1);
            start = index + 1;
        }
    }
    if start < text.len() {
        lines.push(start..text.len());
    }
    lines
}

/// Where to draw the caret for a byte offset in a block's own text.
///
/// Takes the [`Doc`] as well as the [`Laid`], because the laid-out geometry carries ranges into text the
/// layout does not own. A `Laid` that cloned the text would be a second copy of the document to keep in
/// step, and the paint half holds both anyway.
///
/// Returns the line's index and the x offset **from the document's left edge**. An offset in the middle
/// of a multi-byte character is rounded **down** to the character's start, because a caret cannot be
/// drawn inside one.
pub fn caret(doc: &Doc, laid: &Laid, block: usize, offset: usize) -> Option<(usize, f64)> {
    let block_box = laid.blocks.iter().find(|box_| box_.block == block)?;
    let text = doc.blocks.get(block)?.kind.text()?.text.as_str();
    let offset = floor_to_boundary(text, offset);
    for (index, line) in block_box.lines.iter().enumerate() {
        if offset < line.range.end {
            let x = line.x + laid.metrics.text_width(&text[line.range.start..offset]);
            return Some((index, x));
        }
    }
    let last = block_box.lines.last()?;
    let x = last.x + laid.metrics.text_width(&text[last.range.start..]);
    Some((block_box.lines.len() - 1, x))
}

/// The document offset nearest a point.
///
/// Clamped rather than `None`-ed for a point outside the text: a click above the document belongs to the
/// start and one below it to the end, which is what a reader dragging past the edge means. See [`caret`]
/// on why this takes the document too.
pub fn offset_at(doc: &Doc, laid: &Laid, x: f64, y: f64) -> Option<usize> {
    let (block, line_index) = laid.line_at(y)?;
    let block_box = &laid.blocks[block];
    let line = block_box.lines.get(line_index)?;
    let text = doc.blocks.get(block)?.kind.text()?.text.as_str();
    let local = text.get(line.range.clone())?;

    // Walk the line's characters, accumulating width, and stop at the character whose midpoint the x
    // passed — which is what makes a click between two characters choose the nearer one rather than
    // always the left one.
    let mut best = line.range.start;
    let mut used = 0.0;
    for (byte, ch) in local.char_indices() {
        let width = laid.metrics.char_width(ch);
        if line.x + used + width * 0.5 > x {
            break;
        }
        used += width;
        best = line.range.start + byte + ch.len_utf8();
    }
    Some(best)
}

/// Round an offset down to the start of the character it lands inside.
fn floor_to_boundary(text: &str, offset: usize) -> usize {
    if offset >= text.len() {
        return text.len();
    }
    let mut at = offset;
    while at > 0 && !text.is_char_boundary(at) {
        at -= 1;
    }
    at
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn metrics() -> Metrics {
        // Ten points per character, twenty per line: every expected value in this module is arithmetic a
        // reader can check by hand.
        Metrics {
            advance: 10.0,
            body_size: 10.0,
            // Ten, twenty and thirty points, so a test can tell a heading's line from a paragraph's by arithmetic
            // alone: a body line is 20 high, an h1 line 40.
            heading_size: [20.0, 15.0, 12.0],
            line_height: 20.0,
            indent: 30.0,
            gap: 5.0,
            padding: 4.0,
        }
    }

    fn laid(source: &str, width: f64) -> Laid {
        layout(&parse(source), metrics(), width)
    }

    fn line_texts(doc: &Doc, block: usize, wrap_width: f64) -> Vec<String> {
        let laid = layout(doc, metrics(), wrap_width);
        let box_ = laid
            .blocks
            .iter()
            .find(|box_| box_.block == block)
            .expect("a block");
        let text = doc.blocks[block].kind.text().expect("text");
        box_.lines
            .iter()
            .map(|line| text.text[line.range.clone()].to_string())
            .collect()
    }

    #[test]
    fn test_a_short_paragraph_is_one_line() {
        let doc = parse("hello\n");
        assert_eq!(line_texts(&doc, 0, 1000.0), vec!["hello"]);
    }

    #[test]
    fn test_a_paragraph_wraps_at_spaces_and_not_inside_words() {
        // 100 wide, 10 per character: ten characters fit. `one two three` must break after `two ` rather
        // than inside `three`.
        let doc = parse("one two three\n");
        assert_eq!(line_texts(&doc, 0, 100.0), vec!["one two ", "three"]);
    }

    #[test]
    fn test_a_word_longer_than_the_measure_breaks_inside_itself() {
        // A URL or an identifier. A wrapper that only broke at spaces would let this overflow the
        // measure, which is the fault the module doc says the estimate must never cause.
        let doc = parse("abcdefghijklmnopqrstuvwxyz\n");
        let lines = line_texts(&doc, 0, 100.0);
        assert_eq!(lines.len(), 3, "{lines:?}");
        assert_eq!(lines[0], "abcdefghij");
        assert_eq!(lines[1], "klmnopqrst");
        assert_eq!(lines[2], "uvwxyz");
        // And every line fits.
        for line in &lines {
            assert!(
                metrics().text_width(line) <= 100.0,
                "{line:?} is wider than the measure"
            );
        }
    }

    #[test]
    fn test_wrapping_covers_the_text_in_order_with_no_gaps() {
        // The property a caret and a hit test both depend on: every byte of the block's text is on
        // exactly one line, in order.
        let source = "alpha beta gamma delta epsilon zeta eta theta\n";
        let doc = parse(source);
        let laid = layout(&doc, metrics(), 80.0);
        let text = doc.blocks[0].kind.text().expect("text");
        let mut expected = 0usize;
        for line in laid.blocks[0].lines.iter() {
            assert_eq!(line.range.start, expected, "{:?}", laid.blocks[0].lines);
            assert!(line.range.end >= line.range.start);
            expected = line.range.end;
        }
        assert_eq!(expected, text.text.len(), "the lines do not cover the text");
    }

    #[test]
    fn test_every_line_boundary_is_on_a_character_boundary() {
        // An offset inside a multi-byte character is a panic in a slicing renderer. Full-width
        // characters are doubled, so a line can break in the middle of a run of them.
        let doc = parse("日本語のテキストがここにあります\n");
        let laid = layout(&doc, metrics(), 60.0);
        let text = doc.blocks[0].kind.text().expect("text");
        assert!(laid.blocks[0].lines.len() > 1, "the text did not wrap");
        for line in &laid.blocks[0].lines {
            assert!(
                text.text.is_char_boundary(line.range.start)
                    && text.text.is_char_boundary(line.range.end),
                "{:?} is not on a boundary",
                line.range
            );
        }
    }

    #[test]
    fn test_a_full_width_character_costs_two_advances() {
        let m = metrics();
        assert_eq!(m.char_width('a'), 10.0);
        assert_eq!(m.char_width('中'), 20.0);
        assert_eq!(m.text_width("ab"), 20.0);
        assert_eq!(m.text_width("中文"), 40.0);
        // ...and it wraps accordingly: a hundred points hold **five** full-width characters, not ten.
        // The first version of this expectation said four, which is arithmetic rather than behaviour.
        let doc = parse("中文字符串啊\n");
        assert_eq!(line_texts(&doc, 0, 100.0), vec!["中文字符串", "啊"]);
    }

    #[test]
    fn test_an_empty_block_still_has_one_line() {
        // The state an editor is in the moment Enter is pressed. A block with no lines would have no
        // height, so the caret could not sit in it and the next block would move up under the cursor.
        let doc = parse("a\n\nb\n");
        let laid = layout(&doc, metrics(), 1000.0);
        assert_eq!(laid.blocks.len(), 2, "a blank line is not a block in this model");
        // And an explicitly empty paragraph does:
        let doc = Doc {
            blocks: vec![crate::Block {
                indent: 0,
                kind: BlockKind::Paragraph(Text::plain("")),
            }],
        };
        let laid = layout(&doc, metrics(), 1000.0);
        assert_eq!(laid.blocks[0].lines.len(), 1);
        assert_eq!(laid.blocks[0].height, 20.0);
    }

    #[test]
    fn test_blocks_stack_with_a_gap_between_them() {
        // A heading, a paragraph and a list item, each one line, with a gap of five between them.
        let doc = parse("# H\n\npara\n\n- item\n");
        let laid = layout(&doc, metrics(), 1000.0);
        assert_eq!(laid.blocks.len(), 3);
        // **The heading's line is twice a paragraph's**, because its size is: the fixture's `heading_size[0]` is
        // 20 against a body of 10, and the line height is scaled by the same ratio. So the paragraph starts at
        // 40 + 5 rather than 20 + 5 — which is the whole point of a size per kind, and the first version of this
        // test asserted the paragraph's height for every block.
        assert_eq!(laid.blocks[0].y, 0.0);
        assert_eq!(laid.blocks[0].height, 40.0);
        assert_eq!(laid.blocks[1].y, 45.0);
        assert_eq!(laid.blocks[1].height, 20.0);
        assert_eq!(laid.blocks[2].y, 70.0);
        assert_eq!(laid.height, 90.0, "the document's height is the last block's bottom");
    }

    #[test]
    fn test_a_heading_takes_more_vertical_space_than_a_paragraph() {
        // **The property the per-kind size exists for.** A layout that used one line height for everything paints a
        // heading over the block below it, and the marked-up heading looks exactly like a paragraph — which is
        // what a screenshot found and no test could, because the layout had no notion of a size per kind.
        let doc = parse("# Heading\n\nParagraph\n");
        let laid = layout(&doc, metrics(), 1000.0);
        assert!(
            laid.blocks[0].height > laid.blocks[1].height,
            "the heading is {} high and the paragraph {}",
            laid.blocks[0].height,
            laid.blocks[1].height
        );
        // And the taller block pushes the next one down by the difference, which is what stops the overlap.
        assert_eq!(laid.blocks[1].y, laid.blocks[0].height + laid.metrics.gap);
        // A deeper heading is smaller than a shallower one, so the three levels are ordered.
        let doc = parse("# One\n\n## Two\n\n### Three\n\n#### Four\n");
        let laid = layout(&doc, metrics(), 1000.0);
        let heights: Vec<f64> = laid.blocks.iter().map(|block| block.height).collect();
        assert!(heights[0] > heights[1], "{heights:?}");
        assert!(heights[1] > heights[2], "{heights:?}");
        // ...and a level 4 is clamped to level 3's size rather than inventing a fourth rung.
        assert_eq!(heights[2], heights[3], "{heights:?}");
    }

    #[test]
    fn test_a_block_that_wraps_pushes_the_next_one_down_by_its_own_height() {
        let doc = parse("one two three four five six\n\nafter\n");
        let laid = layout(&doc, metrics(), 100.0);
        let first = &laid.blocks[0];
        assert!(first.lines.len() > 1, "the paragraph did not wrap");
        assert_eq!(
            laid.blocks[1].y,
            first.y + first.height + metrics().gap,
            "the second block did not clear the first"
        );
    }

    #[test]
    fn test_an_indent_moves_the_text_and_narrows_the_measure() {
        let doc = parse("- outer\n  - inner\n");
        let laid = layout(&doc, metrics(), 200.0);
        assert_eq!(laid.blocks[0].text_x, 0.0 + metrics().text_width("- "));
        assert_eq!(
            laid.blocks[1].text_x,
            metrics().indent + metrics().text_width("- "),
            "the nested item is one indent in"
        );
    }

    #[test]
    fn test_markers_are_the_layout_s_decision_and_come_out_of_the_measure() {
        let doc = parse("- a\n1. b\n- [x] c\n> d\n");
        let laid = layout(&doc, metrics(), 200.0);
        let markers: Vec<Option<&str>> = laid
            .blocks
            .iter()
            .map(|block| block.marker.as_deref())
            .collect();
        assert_eq!(
            markers,
            vec![Some("- "), Some("1. "), Some("[x] "), Some("> ")]
        );
        // And each marker's width was taken out of the measure.
        for block in &laid.blocks {
            let marker_width = block
                .marker
                .as_ref()
                .map(|marker| laid.metrics.text_width(marker))
                .unwrap_or(0.0);
            let padding = if block.marker.as_deref() == Some("> ") {
                laid.metrics.padding
            } else {
                0.0
            };
            assert_eq!(block.text_x, marker_width + padding, "{block:?}");
        }
    }

    #[test]
    fn test_the_caret_at_the_start_and_the_end_of_a_block() {
        let doc = parse("hello\n");
        let laid = layout(&doc, metrics(), 1000.0);
        let text = doc.blocks[0].kind.text().expect("text");
        // Offset 0 is at the text's left edge.
        let (line, x) = caret(&doc, &laid, 0, 0).expect("a caret");
        assert_eq!(line, 0);
        assert_eq!(x, 0.0);
        // And the end is the text's width along.
        let (line, x) = caret(&doc, &laid, 0, text.text.len()).expect("a caret");
        assert_eq!(line, 0);
        assert_eq!(x, 50.0, "five characters at ten points");
    }

    #[test]
    fn test_the_caret_follows_the_wrap_onto_the_second_line() {
        let doc = parse("one two three\n");
        let laid = layout(&doc, metrics(), 100.0);
        let text = doc.blocks[0].kind.text().expect("text");
        // `three` starts at byte 8, which is on the second line.
        let (line, x) = caret(&doc, &laid, 0, 8).expect("a caret");
        assert_eq!(line, 1, "the caret did not move to the second line");
        assert_eq!(x, 0.0, "and it starts at that line's left edge");
    }

    #[test]
    fn test_a_caret_offset_inside_a_character_rounds_down() {
        // A caret cannot be drawn inside a character, and a caller with a byte offset from a search or a
        // selection can produce one.
        let doc = parse("héllo\n");
        let laid = layout(&doc, metrics(), 1000.0);
        let text = doc.blocks[0].kind.text().expect("text");
        // `é` occupies bytes 1 and 2, so byte 2 is not a boundary.
        assert!(!text.text.is_char_boundary(2));
        let (_, x) = caret(&doc, &laid, 0, 2).expect("a caret");
        assert_eq!(x, 10.0, "the caret rounded down to the start of `é`");
    }

    #[test]
    fn test_hit_testing_round_trips_through_the_caret() {
        // **The property that ties the two together.** For every character boundary in a document, the
        // point the caret is drawn at must hit-test back to that same offset — otherwise clicking where
        // the caret is drawn puts the caret somewhere else, which is the fault a reader notices most.
        let doc = parse(
            "# A heading\n\nA paragraph that is long enough to wrap onto more than one line.\n\n- a list item\n  - nested\n\n> a quote\n\n```\ncode\n```\n",
        );
        let laid = layout(&doc, metrics(), 200.0);
        let mut checked = 0usize;
        for block_box in &laid.blocks {
            let text = doc.blocks[block_box.block].kind.text().map(|t| t.text.clone());
            let Some(text) = text else { continue };
            for line in &block_box.lines {
                for offset in line.range.clone() {
                    if !text.is_char_boundary(offset) {
                        continue;
                    }
                    let (_, x) = caret(&doc, &laid, block_box.block, offset).expect("a caret");
                    let (_, line_index) = laid.line_at(line.y + 1.0).expect("a line");
                    let back = offset_at(&doc, &laid, x, line.y + 1.0);
                    assert_eq!(
                        back,
                        Some(offset),
                        "offset {offset} in block {} drew at x={x} on line {line_index} and hit-tested to {back:?}",
                        block_box.block
                    );
                    checked += 1;
                }
            }
        }
        assert!(checked > 50, "only {checked} offsets were checked");
    }

    #[test]
    fn test_a_click_past_the_end_of_a_line_lands_at_its_end() {
        let doc = parse("ab\n");
        let laid = layout(&doc, metrics(), 1000.0);
        assert_eq!(offset_at(&doc, &laid, 10_000.0, 1.0), Some(2));
        // And a click to the left of it lands at the start.
        assert_eq!(offset_at(&doc, &laid, -10_000.0, 1.0), Some(0));
    }

    #[test]
    fn test_a_click_between_two_characters_chooses_the_nearer_one() {
        let doc = parse("abcd\n");
        let laid = layout(&doc, metrics(), 1000.0);
        // Each character is ten wide, so the midpoint of `a` is at five. Just under it is offset 0; just
        // over it is offset 1.
        assert_eq!(offset_at(&doc, &laid, 4.9, 1.0), Some(0));
        assert_eq!(offset_at(&doc, &laid, 5.1, 1.0), Some(1));
        assert_eq!(offset_at(&doc, &laid, 24.9, 1.0), Some(2));
        assert_eq!(offset_at(&doc, &laid, 25.1, 1.0), Some(3));
    }

    #[test]
    fn test_a_click_above_or_below_the_document_is_clamped_to_its_ends() {
        // A reader dragging past the edge means the end, not nothing.
        let doc = parse("a\n\nb\n");
        let laid = layout(&doc, metrics(), 1000.0);
        assert_eq!(laid.block_at(-100.0), Some(0));
        assert_eq!(laid.block_at(100_000.0), Some(laid.blocks.len() - 1));
        assert_eq!(laid.line_at(-100.0).map(|(block, _)| block), Some(0));
        assert_eq!(
            laid.line_at(100_000.0).map(|(block, _)| block),
            Some(laid.blocks.len() - 1)
        );
    }

    #[test]
    fn test_a_click_in_the_gap_under_a_block_belongs_to_that_block() {
        // The gap is ten points of nothing, and a reader clicking in it means the block they can see
        // above the pointer.
        let doc = parse("a\n\nb\n");
        let laid = layout(&doc, metrics(), 1000.0);
        let first_bottom = laid.blocks[0].y + laid.blocks[0].height;
        assert!(first_bottom + 2.0 < laid.blocks[1].y, "there is a gap");
        assert_eq!(laid.block_at(first_bottom + 2.0), Some(0));
    }

    #[test]
    fn test_a_divider_gets_a_line_of_its_own() {
        let doc = parse("a\n\n---\n\nb\n");
        let laid = layout(&doc, metrics(), 1000.0);
        assert_eq!(laid.blocks.len(), 3);
        assert_eq!(laid.blocks[1].lines.len(), 1);
        assert!(laid.blocks[1].marker.is_none());
        // It still takes a line's height, so the block after it clears it.
        assert_eq!(laid.blocks[1].height, 20.0);
    }

    #[test]
    fn test_a_code_block_is_not_rewrapped() {
        // Rewrapping code changes what the reader reads — a markdown document's job is to show the
        // characters that are there. So its lines are its own newlines, however wide they are.
        let doc = parse("```\nshort\nthis line is much much much wider than the measure\n```\n");
        let laid = layout(&doc, metrics(), 100.0);
        assert_eq!(laid.blocks.len(), 1);
        assert_eq!(laid.blocks[0].lines.len(), 2, "{:?}", laid.blocks[0].lines);
        // ...and this is where `wrap`'s `<` matters: a `\n` breaks regardless of width, so the second
        // line is over the measure and is left alone.
        let text = doc.blocks[0].kind.text().expect("text");
        let second = &laid.blocks[0].lines[1];
        assert!(
            metrics().text_width(&text.text[second.range.clone()]) > 100.0,
            "the long line was rewrapped"
        );
    }

    #[test]
    fn test_the_height_is_the_sum_of_the_blocks_and_the_gaps() {
        // The number a scroll view needs, asserted as arithmetic rather than read back from the layout.
        let doc = parse("# H\n\npara that wraps onto a second line for sure\n\n- a\n- b\n\n```\nx\n```\n");
        let laid = layout(&doc, metrics(), 150.0);
        let blocks_height: f64 = laid.blocks.iter().map(|block| block.height).sum();
        let gaps = laid.metrics.gap * (laid.blocks.len() - 1) as f64;
        assert!(
            (laid.height - (blocks_height + gaps)).abs() < 1e-9,
            "height {} is not {blocks_height} + {gaps}",
            laid.height
        );
    }

    #[test]
    fn test_the_layout_does_not_depend_on_marks() {
        // Marks are the paint's business. A layout that moved when the text was emphasised would make
        // the caret jump the moment a reader pressed a formatting key.
        let plain = layout(&parse("bold text here\n"), metrics(), 100.0);
        let marked = layout(&parse("**bold** text here\n"), metrics(), 100.0);
        // The text differs, so the lines' ranges shift — but the **wrap points in characters** are the
        // same, which is what a caret measures in.
        let chars = |laid: &Laid, doc: &Doc| -> Vec<usize> {
            let text = doc.blocks[0].kind.text().expect("text");
            laid.blocks[0]
                .lines
                .iter()
                .map(|line| text.text[..line.range.start].chars().count())
                .collect()
        };
        assert_eq!(
            chars(&plain, &parse("bold text here\n")),
            chars(&marked, &parse("**bold** text here\n"))
        );
    }

    #[test]
    fn test_an_empty_document_lays_out_to_nothing() {
        let laid = layout(&parse(""), metrics(), 100.0);
        assert!(laid.blocks.is_empty());
        assert_eq!(laid.height, 0.0);
        assert_eq!(laid.block_at(50.0), None);
        assert_eq!(laid.line_at(50.0), None);
    }
}
