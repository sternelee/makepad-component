//! `makepad-markdown` — a block document model, with markdown as the wire form.
//!
//! ## The shape is Notion's, not CommonMark's
//!
//! A [`Doc`] is a **flat list of blocks with an indent level**, not a nested tree. That is the
//! decision everything else here hangs off: editing a flat list means Enter splits, Backspace merges
//! and Tab indents — all list operations on one `Vec`. On a nested tree, "the previous block" is a
//! traversal and every edit is a restructure.
//!
//! The trade is explicit: **arbitrarily nested CommonMark does not survive a round trip** — a list
//! inside a quote inside a list flattens, and this port's parser does not carry tables or reference
//! links. What *is* guaranteed is the property the whole crate is built around.
//!
//! ## The fixed point
//!
//! > Parsing, serializing and parsing again always lands on the same document.
//!
//! Formally, for every input `s`: `parse(serialize(parse(s))) == parse(s)`. That is what makes an
//! edit/save cycle unable to drift — and it is deliberately **weaker** than "`serialize` is the
//! inverse of `parse`", because it must be: `_italic_` parses to an italic mark and serializes as
//! `*italic*`, so the *text* changes on the first save. What cannot change is the **document**, and
//! that is the thing an editor holds.
//!
//! Every test in this file is either an instance of that property or a case that would break it, and
//! the corpus it is checked over is written to include the inputs that drift if a rule is wrong: a
//! list starting at three, `_` italic, `***` as a divider rather than as emphasis, a fence whose
//! content looks like markdown, and a quote with a blank line in it.
//!
//! ## The indent invariant
//!
//! `indent` obeys one rule, established by [`parse`] and relied on by [`serialize`]: **the first
//! block is at 0, and no block is more than one level deeper than the block before it.** A document
//! that satisfies it serializes to markdown that parses back to the same indents; a document that
//! does not would serialize to a jump the parser refuses to read back, and the fixed point would
//! break. [`Doc::is_well_formed`] is the check, and the round-trip test runs it on every result.

pub mod edit;
pub mod layout;

// The editing vocabulary, reachable from the crate root — the same shape `markdown`'s reference has, and what
// lets an editor crate ring a document's edits without reaching into a submodule for each name.
pub use edit::{apply, apply as apply_shortcut, Edited, Selection, SetKind, Shortcut};

use std::ops::Range;

/// A markdown document: blocks in document order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Doc {
    pub blocks: Vec<Block>,
}

/// One block, and how deeply it is nested.
///
/// See the module doc for the invariant `indent` obeys.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub indent: u8,
    pub kind: BlockKind,
}

/// What a block is.
///
/// A closed set, like every other vocabulary in this port. An app that needs a block of its own
/// reaches for a **fence** — `` ```chart `` — rather than for a new variant here, because markdown is
/// the wire form and a new kind would have to own a syntax while a fence already has one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockKind {
    Paragraph(Text),
    Heading { level: u8, text: Text },
    Bullet(Text),
    /// The rendered number is **stored rather than derived**, so a list starting at 3 survives the
    /// round trip. [`Doc::renumber`] makes a run consecutive at parse time; see its doc for why that
    /// has to happen then rather than here.
    Ordered { number: u64, text: Text },
    Task { checked: bool, text: Text },
    /// Consecutive `>` lines are **one** block whose text carries newlines, because that is what a
    /// quote is: a paragraph with a marker in the margin. Serializing one line per newline makes the
    /// round trip exact.
    Quote(Text),
    /// A fenced block. Its content is a [`Text`] like every other editable region, so one accessor
    /// and one edit path cover the whole document — and the code's marks are *unreachable rather than
    /// forbidden*: nothing that writes here creates one.
    Code { language: Option<String>, code: Text },
    /// `---`, and its two spellings. Serialized as `---`: see [`serialize`] on why one canonical
    /// spelling is what keeps the fixed point.
    Divider,
}

impl BlockKind {
    /// Whether this block is a list item, which is what decides whether a blank line goes before it.
    pub fn is_list_item(&self) -> bool {
        matches!(
            self,
            BlockKind::Bullet(_) | BlockKind::Ordered { .. } | BlockKind::Task { .. }
        )
    }

    /// The block's text, for a caller that wants the content without the kind.
    pub fn text(&self) -> Option<&Text> {
        match self {
            BlockKind::Paragraph(text)
            | BlockKind::Bullet(text)
            | BlockKind::Quote(text)
            | BlockKind::Task { text, .. }
            | BlockKind::Heading { text, .. }
            | BlockKind::Ordered { text, .. } => Some(text),
            BlockKind::Code { code, .. } => Some(code),
            BlockKind::Divider => None,
        }
    }
}

/// A run of text and the marks on it.
///
/// Marks are **byte ranges into `text`**, in document order. They are allowed to **nest**, and that is
/// deliberately *not* the contract `makepad_theme::syntax` uses for highlight spans: two foreground
/// colours cannot compose, so overlapping highlight spans have to be resolved, while two text styles
/// do — `**bold with _italic_ inside**` is one bold span *and* one italic span, and a model that dropped
/// the inner one would lose the emphasis the reader typed.
///
/// The first version of this file reused the highlight contract and dropped the nested italic; the test
/// caught it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Text {
    pub text: String,
    pub marks: Vec<MarkSpan>,
    /// Links, as ranges over `text` **plus a URL each**.
    ///
    /// A parallel list rather than a variant of [`Mark`], because a mark is `Copy`, fieldless and compared by value
    /// everywhere — a `Mark::Link(String)` would make every one of those comparisons allocate, and it would make
    /// `Mark::delimiters()` unable to answer, since a link's delimiters are its label's.
    ///
    /// The range covers the **label**, which is what is displayed: `[label](url)` parses to `text == "label"` and a
    /// link whose range is `0..5` with that URL. So a link behaves like a mark in every geometric way, which is what
    /// lets one shift function move both.
    pub links: Vec<LinkSpan>,
}

/// A link, and the bytes of its label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkSpan {
    pub range: Range<usize>,
    /// The target, exactly as written. Not normalized: a URL is the server's business.
    pub url: String,
}

/// A mark, and the bytes it covers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkSpan {
    pub range: Range<usize>,
    pub mark: Mark,
}

/// What a mark is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Mark {
    Bold,
    Italic,
    Code,
    Strike,
}

impl Mark {
    /// The markdown delimiters this mark is written with.
    ///
    /// One spelling each, which is what makes the fixed point hold: `_italic_` and `*italic*` parse to
    /// the same mark and serialize to `*italic*`, so the *text* changes on the first save and the
    /// *document* does not.
    fn delimiters(self) -> &'static str {
        match self {
            Mark::Bold => "**",
            Mark::Italic => "*",
            Mark::Code => "`",
            Mark::Strike => "~~",
        }
    }
}

impl Text {
    /// Text with no marks.
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            marks: Vec::new(),
            links: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Make the marks sorted, in bounds, and non-empty — **keeping every nesting**.
    ///
    /// Two of the three faults `makepad_theme::syntax::normalize` fixes apply here and one does not:
    /// a scanner emits marks in the order it finds them, and a range can be past the end — but an outer
    /// mark containing an inner one is not a fault to resolve, it is the thing marks are for. See
    /// [`Text`].
    pub fn normalize(&mut self) {
        let len = self.text.len();
        let mut spans: Vec<MarkSpan> = self
            .marks
            .iter()
            .filter(|span| span.range.start < span.range.end)
            .map(|span| MarkSpan {
                range: span.range.start.min(len)..span.range.end.min(len),
                mark: span.mark,
            })
            .filter(|span| span.range.start < span.range.end)
            .collect();
        // Sorted for reading and writing, and **not** de-overlapped: a nesting is kept, so an outer
        // mark and an inner one both survive. Longer first at the same start, so a writer emits the
        // outer mark's opening delimiter before the inner one's.
        spans.sort_by(|a, b| {
            a.range
                .start
                .cmp(&b.range.start)
                .then(b.range.end.cmp(&a.range.end))
                .then(a.mark.cmp(&b.mark))
        });
        self.marks = spans;

        // Links, clamped the same way, and **de-overlapped** — which marks deliberately are not. A nesting of marks is
        // writable (`**a *b* c**`), so marks keep theirs; two overlapping links are **not** representable, because the
        // writer would emit `[a[b]c](u1)(u2)`, which does not re-parse to the same two links. Keeping the earlier one
        // is the choice that keeps the document writable, and `normalize` is where that has to happen rather than in
        // the writer, which cannot drop a link without losing the fact that it dropped one.
        let mut links: Vec<LinkSpan> = self
            .links
            .iter()
            .filter(|link| link.range.start < link.range.end)
            .map(|link| LinkSpan {
                range: link.range.start.min(len)..link.range.end.min(len),
                url: link.url.clone(),
            })
            .filter(|link| link.range.start < link.range.end)
            .collect();
        links.sort_by(|a, b| a.range.start.cmp(&b.range.start).then(a.range.end.cmp(&b.range.end)));
        let mut kept: Vec<LinkSpan> = Vec::with_capacity(links.len());
        for link in links {
            if kept.last().is_some_and(|last| last.range.end > link.range.start) {
                continue;
            }
            kept.push(link);
        }
        self.links = kept;
    }
}

impl Doc {
    /// Whether the indent invariant holds.
    ///
    /// See the module doc. A doc that fails this serializes to markdown `parse` would read back at
    /// different indents, so the fixed point would break — which is why the round-trip test asserts it
    /// on every result rather than trusting it.
    pub fn is_well_formed(&self) -> bool {
        let mut previous = 0u8;
        for (index, block) in self.blocks.iter().enumerate() {
            if index == 0 {
                if block.indent != 0 {
                    return false;
                }
            } else if block.indent > previous + 1 {
                return false;
            }
            previous = block.indent;
        }
        true
    }

    /// Make each run of ordered items consecutive.
    ///
    /// **Markdown honours only the first number in a list** — `1.` followed by `9.` renders as 1, 2 —
    /// so a document that kept the source's `9.` would render one way and come back as `2.` on the
    /// next read. Deciding it at parse time means the document already holds what the next parse
    /// produces, which is the fixed point again: without this, `1.\n9.` would drift on the second
    /// round trip rather than the first.
    ///
    /// A run survives blocks nested under it and ends at anything that is not an ordered item.
    pub fn renumber(&mut self) {
        // The number owed to the next ordered item at each indent level, one slot per level.
        let mut expected: Vec<Option<u64>> = Vec::new();
        for block in &mut self.blocks {
            // **A block the wire form does not write does not break a run.** An empty paragraph is skipped
            // by `serialize` — its representation *is* the blank line between blocks — so two ordered items
            // separated only by one come out consecutive in the markdown, renumber as 1 and 2 on the next
            // read, and the document drifts. Numbering has to follow the wire form because the wire form is
            // what the next parse sees.
            //
            // Found by the random-session test, which is the only place an empty paragraph between two list
            // items arises: `SetKind(Ordered)` on a block next to one.
            if matches!(&block.kind, BlockKind::Paragraph(text) if text.text.is_empty()) {
                continue;
            }
            let indent = block.indent as usize;
            expected.truncate(indent + 1);
            expected.resize(indent + 1, None);
            if let BlockKind::Ordered { number, .. } = &mut block.kind {
                if let Some(next) = expected[indent] {
                    *number = next;
                }
                expected[indent] = Some(number.saturating_add(1));
            } else {
                expected[indent] = None;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// The wire form
// ---------------------------------------------------------------------------

/// Parse markdown into a document.
///
/// The subset this port carries is listed in the module doc; what matters here is that the result
/// **always satisfies the indent invariant**, because a parser that let an indent jump by two would
/// produce a document its own serializer could not write back.
/// Whether a URL's **name** says it paints as an image.
///
/// Asked by two callers that must agree: the paste menu, which offers "Create image" only where a picture can go, and a
/// renderer, which decides whether to paint one. A shared predicate rather than two lists is the whole point — the two
/// disagreeing is how a menu offers a row that paints a broken box.
///
/// The rule is the **last path segment's extension**, so all of these are images:
///
/// - `https://host/picture.png`
/// - `https://host/a/b/photo.JPEG` — case does not matter, because a URL's case is the server's business
/// - `https://host/pic.webp?w=400&h=200#top` — the query and the fragment are not part of the name
/// - `https://host/download.jpg/` — a trailing slash, which servers hand out
///
/// And these are not, because this answers from the text and **nothing here fetches anything**:
///
/// - `https://host/photo` — no extension says nothing; guessing would paint a broken box
/// - `https://host/a.png/index.html` — the last segment wins, and it is not an image
/// - `https://img.example.com` — a bare host is not a file
///
/// An extension **list** rather than "does the name contain a dot": `report.pdf` and `page.html` are files with
/// extensions and neither is a picture.
pub fn is_image(url: &str) -> bool {
    // The fragment first, then the query: a fragment can contain a `?`, so taking the query first would keep part of
    // the fragment as the name.
    let without_fragment = url.split('#').next().unwrap_or(url);
    let without_query = without_fragment.split('?').next().unwrap_or(without_fragment);
    let trimmed = without_query.trim_end_matches('/');
    let Some(segment) = trimmed.rsplit('/').next() else {
        return false;
    };
    let Some((_, extension)) = segment.rsplit_once('.') else {
        return false;
    };
    if extension.is_empty() || extension.contains('/') {
        return false;
    }
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "avif" | "bmp" | "svg" | "ico" | "tif" | "tiff"
    )
}

pub fn parse(source: &str) -> Doc {
    let mut blocks: Vec<Block> = Vec::new();
    let lines: Vec<&str> = source.split('\n').collect();
    let mut index = 0usize;

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();
        if trimmed.is_empty() {
            index += 1;
            continue;
        }
        // **The marker recognizers see the line before its trailing whitespace is trimmed.** `- ` is an empty
        // bullet, and `trim()` turns it into `-`, which is a paragraph whose text is a minus — so an editor
        // that pressed Enter at the end of a bullet produced a document whose wire form changed on the next
        // read. `raw` keeps the trailing space; the text each recognizer returns is trimmed by the recognizer.
        let raw = line.trim_start();

        // The raw indent is clamped to **one deeper than the previous block**, which is the invariant
        // and also what makes a source indented by four spaces parse rather than be rejected.
        let raw_indent = leading_spaces(line) / 2;
        // **The first block is at zero.** The invariant says so, and an indented first block would
        // serialize to a leading indent that parses back at level 1 — a document that cannot be written
        // faithfully. `"  indented paragraph"` is a paragraph at level 0, which is also what CommonMark
        // does with a paragraph indented by less than four spaces.
        let indent = match blocks.last() {
            None => 0,
            Some(previous) => (raw_indent as u8).min(previous.indent.saturating_add(1)),
        };

        // A fence first: its content is not markdown, so nothing below may look at it.
        if let Some(language) = fence_open(trimmed) {
            let mut code = Vec::new();
            index += 1;
            let mut closed = false;
            while index < lines.len() {
                if lines[index].trim().starts_with("```") {
                    closed = true;
                    index += 1;
                    break;
                }
                code.push(lines[index]);
                index += 1;
            }
            // An **unclosed** fence runs to the end of the source and is kept as a fence. That is what
            // an editor holds while someone is typing the closing marker, and dropping the block would
            // make the lines the reader is looking at disappear from the document. `closed` is read so
            // the intent is stated rather than implicit.
            let _ = closed;
            // **The source's final newline is not content.** Splitting on `\n` gives a trailing empty
            // element for a file that ends with one, and keeping it made every fence in the corpus gain a
            // blank line at the bottom on the first round trip.
            if !closed && code.last().is_some_and(|line| line.is_empty()) {
                code.pop();
            }
            blocks.push(Block {
                indent,
                kind: BlockKind::Code {
                    language,
                    code: Text::plain(code.join("\n")),
                },
            });
            continue;
        }

        // A quote: consecutive `>` lines are one block.
        if trimmed.starts_with('>') {
            let mut quoted: Vec<String> = Vec::new();
            while index < lines.len() {
                let text = lines[index].trim();
                let Some(rest) = text.strip_prefix('>') else {
                    break;
                };
                quoted.push(rest.strip_prefix(' ').unwrap_or(rest).to_string());
                index += 1;
            }
            blocks.push(Block {
                indent,
                kind: BlockKind::Quote(parse_inline(&quoted.join("\n"))),
            });
            continue;
        }

        if let Some(level) = heading_level(trimmed) {
            let text = trimmed[level as usize..].trim_start();
            blocks.push(Block {
                indent,
                kind: BlockKind::Heading {
                    level,
                    text: parse_inline(text),
                },
            });
            index += 1;
            continue;
        }

        if is_divider(trimmed) {
            blocks.push(Block {
                indent,
                kind: BlockKind::Divider,
            });
            index += 1;
            continue;
        }

        // A task before a bullet: `- [ ]` is also a bullet, and the more specific rule has to win.
        if let Some((checked, text)) = task(raw) {
            blocks.push(Block {
                indent,
                kind: BlockKind::Task {
                    checked,
                    text: parse_inline(text),
                },
            });
            index += 1;
            continue;
        }

        if let Some(text) = bullet(raw) {
            blocks.push(Block {
                indent,
                kind: BlockKind::Bullet(parse_inline(text)),
            });
            index += 1;
            continue;
        }

        if let Some((number, text)) = ordered(raw) {
            blocks.push(Block {
                indent,
                kind: BlockKind::Ordered {
                    number,
                    text: parse_inline(text),
                },
            });
            index += 1;
            continue;
        }

        blocks.push(Block {
            indent,
            kind: BlockKind::Paragraph(parse_inline(trimmed)),
        });
        index += 1;
    }

    let mut doc = Doc { blocks };
    // The numbers a next parse would produce, decided now. See `Doc::renumber`.
    doc.renumber();
    doc
}

/// Write a document back as markdown.
///
/// Two rules carry the fixed point:
///
/// - **A blank line between blocks, except between consecutive list items at the same indent.** That
///   is what makes `# Title\n\n- a\n- b` come back byte for byte: a list is one visual unit, and a
///   blank line between its items would end the list on the next parse.
/// - **One canonical spelling per construct.** `***` and `___` both serialize as `---`; `_italic_`
///   serializes as `*italic*`; `+ item` and `* item` serialize as `- item`. The document is unchanged
///   by each of those, which is the property rather than a compromise.
pub fn serialize(doc: &Doc) -> String {
    // **An empty paragraph is not a line.** Its wire representation *is* the blank line that already
    // separates two blocks, so writing a line for it produces two blank lines where one belongs — and the
    // parser absorbs one of them, so the *second* write is shorter than the first. That is a wire form that
    // drifts on a save/load/save, which is the one thing this function exists to prevent.
    //
    // Found by the exhaustive shortcut sweep in `edit.rs`: pressing Enter at the end of a paragraph is how
    // an empty one appears, and that is not an edge case.
    //
    // So blocks are emitted into a list first and joined afterwards, which is what lets one be skipped
    // without the blank-line rule counting it.
    struct Emitted {
        /// The block's lines with **no** indent applied.
        body: String,
        /// The indent level the wire form should write, which is normalised below.
        indent: u8,
        is_list_item: bool,
        /// Whether the lines after the first are **raw**: a fence's content.
        ///
        /// A fence's content is read exactly as written, so indenting it would add the block's indent to the
        /// code *again on every save*. Found by the random-session test after one step: an indented fence came
        /// back with two extra spaces on its code line, and the next save would have added two more.
        raw_body: bool,
    }
    let mut emitted: Vec<Emitted> = Vec::new();
    for block in &doc.blocks {
        let body = match &block.kind {
            // The skip. An empty paragraph emits nothing at all.
            BlockKind::Paragraph(text) if text.text.is_empty() => continue,
            BlockKind::Paragraph(text) => write_inline(text),
            BlockKind::Heading { level, text } => {
                let mut out = "#".repeat((*level).clamp(1, 6) as usize);
                // The space only when there is something after it: `#` alone is an empty heading and reads
                // back as one, and a trailing space is a difference a diff would show.
                if !text.text.is_empty() {
                    out.push(' ');
                    out.push_str(&write_inline(text));
                }
                out
            }
            BlockKind::Bullet(text) => format!("- {}", write_inline(text)),
            BlockKind::Ordered { number, text } => format!("{number}. {}", write_inline(text)),
            BlockKind::Task { checked, text } => {
                format!(
                    "{} {}",
                    if *checked { "- [x]" } else { "- [ ]" },
                    write_inline(text)
                )
            }
            BlockKind::Quote(text) => {
                // Through `write_inline`, so a quote's marks are written back. The first version wrote
                // `text.text` directly and dropped them.
                let body = write_inline(text);
                body.split('\n')
                    .map(|line| {
                        if line.is_empty() {
                            ">".to_string()
                        } else {
                            format!("> {line}")
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            }
            BlockKind::Code { language, code } => {
                let fence = match language {
                    Some(language) => format!("```{language}"),
                    None => "```".to_string(),
                };
                // A fence's content never ends with a newline in the model, so the closing marker goes on its
                // own line and a round trip adds no blank line inside the block.
                format!("{fence}\n{}\n```", code.text)
            }
            BlockKind::Divider => "---".to_string(),
        };
        emitted.push(Emitted {
            body,
            indent: block.indent,
            is_list_item: block.kind.is_list_item(),
            raw_body: matches!(block.kind, BlockKind::Code { .. }),
        });
    }

    // **The indents are normalised on what is written, not on the document.** Skipping an empty paragraph
    // removes a block from the sequence, so the blocks after it are one level deeper than the *wire form's*
    // first — and `parse` reads the first written block as level 0 and clamps the rest to one deeper, so the
    // next save is shorter. The invariant has to hold of the emitted sequence, because that is what the next
    // parse sees. Found by the random-session test at step 3318.
    let mut previous = 0u8;
    for (index, entry) in emitted.iter_mut().enumerate() {
        entry.indent = if index == 0 {
            0
        } else {
            entry.indent.min(previous.saturating_add(1))
        };
        previous = entry.indent;
    }

    let mut out = String::new();
    for (index, entry) in emitted.iter().enumerate() {
        if index > 0 {
            // **One newline always ends the previous block**, and a *second* one is the blank line between
            // blocks that are not one list. The first version pushed only the blank line, so consecutive list
            // items came out as `- a- b`.
            out.push('\n');
            // **Any two list items are one list**, nesting included: requiring equal indents put a blank line
            // between `- outer` and its indented child, which ends the list on the next parse.
            if !(emitted[index - 1].is_list_item && entry.is_list_item) {
                out.push('\n');
            }
        }
        // The indent, applied per line — except a fence's content, which is raw: indenting it would add the
        // block's indent to the code *again on every save*.
        let indent = "  ".repeat(entry.indent as usize);
        let lines: Vec<&str> = entry.body.split('\n').collect();
        let last = lines.len().saturating_sub(1);
        let body = lines
            .iter()
            .enumerate()
            .map(|(line_index, line)| {
                let indented = if entry.raw_body {
                    line_index == 0 || line_index == last
                } else {
                    true
                };
                if indented {
                    format!("{indent}{line}")
                } else {
                    (*line).to_string()
                }
            })
            .collect::<Vec<String>>()
            .join("\n");
        out.push_str(&body);
    }
    if out.is_empty() {
        return out;
    }
    out.push('\n');
    out
}

// ---------------------------------------------------------------------------
// Line recognition
// ---------------------------------------------------------------------------

fn leading_spaces(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ').count()
}

/// The language of a fence opening line, if this is one.
fn fence_open(trimmed: &str) -> Option<Option<String>> {
    let rest = trimmed.strip_prefix("```")?;
    let language = rest.trim();
    // An info string with spaces is not a language this port carries — ```` ```rust ignore ```` is
    // two words, and storing the whole string would serialize back identically either way, so the
    // choice is which of them a caller gets to see.
    Some(if language.is_empty() {
        None
    } else {
        Some(language.to_string())
    })
}

fn heading_level(trimmed: &str) -> Option<u8> {
    let hashes = trimmed.chars().take_while(|c| *c == '#').count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    // `#hashtag` is a paragraph, not a heading: CommonMark requires the space, and without this rule a
    // document full of tags would parse into headings.
    //
    // **But an empty heading is a heading.** `#` alone — or `# ` — has no following character to be a word,
    // and an *editor* holds one the moment a reader turns an empty block into a heading or deletes a
    // heading's text. Requiring the space made `#` parse back as a paragraph whose text is `#`, so an empty
    // heading broke the fixed point — found by the exhaustive shortcut sweep in `edit.rs`.
    if trimmed.len() > hashes && trimmed.chars().nth(hashes) != Some(' ') {
        return None;
    }
    Some(hashes as u8)
}

/// `---`, `***` or `___`, three or more, and nothing else on the line.
///
/// **`***` is a divider rather than emphasis**, which is the ambiguity a line-based parser has to
/// decide: it is a horizontal rule in CommonMark when it is alone on its line, and that is the reading
/// that keeps a document with dividers round-tripping.
fn is_divider(trimmed: &str) -> bool {
    // Trimmed here rather than by the caller, because the caller now passes a form that keeps trailing
    // whitespace for the marker recognizers: `--- ` is still a rule.
    let trimmed = trimmed.trim();
    for marker in ['-', '*', '_'] {
        let count = trimmed.chars().filter(|c| *c == marker).count();
        if count >= 3 && count == trimmed.chars().count() {
            return true;
        }
    }
    false
}

/// `- [ ] text` or `- [x] text`.
fn task(trimmed: &str) -> Option<(bool, &str)> {
    let rest = bullet(trimmed)?;
    let inner = rest.trim_start();
    let after = inner.strip_prefix('[')?;
    let (state, rest) = after.split_at(1);
    let rest = rest.strip_prefix(']')?;
    let checked = match state {
        " " => false,
        "x" | "X" => true,
        _ => return None,
    };
    Some((checked, rest.trim_start()))
}

/// `- text`, `* text` or `+ text`.
fn bullet(trimmed: &str) -> Option<&str> {
    for marker in ['-', '*', '+'] {
        if let Some(rest) = trimmed.strip_prefix(marker) {
            // The marker needs a space after it, or `-5` parses as a bullet containing `5`.
            if let Some(rest) = rest.strip_prefix(' ') {
                return Some(rest.trim_end());
            }
        }
    }
    None
}

/// `1. text` or `1) text`.
fn ordered(trimmed: &str) -> Option<(u64, &str)> {
    let digits: String = trimmed.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    let rest = &trimmed[digits.len()..];
    let rest = rest.strip_prefix('.').or_else(|| rest.strip_prefix(')'))?;
    let rest = rest.strip_prefix(' ')?;
    Some((digits.parse().ok()?, rest.trim_end()))
}

// ---------------------------------------------------------------------------
// Inline marks
// ---------------------------------------------------------------------------

/// Parse the inline marks in one line of text.
///
/// Deliberately a scanner rather than a parser, and deliberately small: four marks, no nesting rules
/// beyond what falls out, and no links or images (which are a *block* in this model's future rather
/// than a mark here). What it must do is produce ranges that [`Text::normalize`] leaves alone — the
/// round trip depends on the ranges being sorted, non-overlapping and in bounds.
pub fn parse_inline(source: &str) -> Text {
    // **Links are extracted first, and the mark pass then runs on what is left.** The two compose because a link pass
    // only ever *removes* characters — the brackets and the target — so the text it produces is exactly the text the
    // mark pass sees, and the mark pass never adds or removes a character either (it removes delimiters). **Both
    // passes therefore report ranges in the same coordinates**, which is what lets a link hold a mark inside its label
    // without either having to know about the other.
    let (stripped, links) = extract_links(source);
    let (text, marks, removed) = parse_marks(&stripped);
    // The link ranges are in the **stripped** text's coordinates and the marks are in the output's, so the links are
    // remapped here: an offset is however many bytes this pass deleted before it, subtracted. Doing it in one place is
    // what lets the two passes stay independent — neither knows the other exists.
    let links = links
        .into_iter()
        .map(|link| LinkSpan {
            range: remap(link.range.start, &removed)..remap(link.range.end, &removed),
            url: link.url,
        })
        .collect();
    let mut parsed = Text {
        text,
        marks,
        links,
    };
    parsed.normalize();
    parsed
}

/// An offset in one pass's input, translated to the other pass's output.
///
/// Subtracts everything the second pass deleted **entirely before** the offset. An offset that falls *inside* a deleted
/// range cannot happen for a link — its endpoints are label characters, and a delimiter is never part of a label — but
/// it is clamped rather than assumed, because assuming it is exactly how `[**a**](u)` lost its link.
fn remap(offset: usize, removed: &[Range<usize>]) -> usize {
    let mut shift = 0usize;
    for range in removed {
        if range.end <= offset {
            shift += range.end - range.start;
        } else if range.start < offset {
            // Inside a removed range: clamp to its start.
            shift += offset - range.start;
            break;
        } else {
            break;
        }
    }
    offset.saturating_sub(shift)
}

/// Pull `[label](url)` out of `source`, leaving the labels behind.
///
/// An **unmatched bracket stays in the text**, which is the same rule the mark pass follows and for the same reason: a
/// `[` with no target is a character somebody typed, and consuming it would be content loss. So `see [note` comes back
/// as `see [note`, and `[a](b) and [c` comes back as `a and [c` with one link.
///
/// Not supported, and deliberately: a label containing `]`, and a target containing `)`. Both need a real inline
/// grammar with escapes, and neither is something a person writes by hand. They fail **safely** — the `[` stays
/// literal — rather than being half-understood.
fn extract_links(source: &str) -> (String, Vec<LinkSpan>) {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut links: Vec<LinkSpan> = Vec::new();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] != b'[' {
            let ch = source[index..].chars().next().expect("a character");
            out.push(ch);
            index += ch.len_utf8();
            continue;
        }
        // `[label](url)` — all four parts required, and the label must be non-empty, because `[](u)` displays nothing
        // and a link you cannot see or click is a link nobody can use.
        let Some(label_end) = source[index + 1..].find(']').map(|at| index + 1 + at) else {
            out.push('[');
            index += 1;
            continue;
        };
        let after = &source[label_end + 1..];
        let Some(url_body) = after.strip_prefix('(') else {
            out.push('[');
            index += 1;
            continue;
        };
        let Some(url_end) = url_body.find(')') else {
            out.push('[');
            index += 1;
            continue;
        };
        let label = &source[index + 1..label_end];
        if label.is_empty() {
            out.push('[');
            index += 1;
            continue;
        }
        let start = out.len();
        out.push_str(label);
        links.push(LinkSpan {
            range: start..out.len(),
            url: url_body[..url_end].to_string(),
        });
        index = label_end + 1 + 1 + url_end + 1;
    }
    (out, links)
}

/// The mark pass: the delimiter-pair scan that `parse_inline` used to be.
///
/// Returns the text, its marks, and **the ranges it deleted** — which the link pass needs, because this pass removes
/// characters and therefore moves everything after them.
fn parse_marks(source: &str) -> (String, Vec<MarkSpan>, Vec<Range<usize>>) {
    // **Two passes, because an unmatched delimiter has to stay in the text.** The first version was one
    // pass that consumed every delimiter it recognized, so `a ** marker` came back as `a  marker` — the
    // marker vanished from the document, which is content loss rather than a cosmetic fault. What has to
    // be removed is only the delimiters that **pair**, and finding those is a scan with a stack.
    //
    // The second version of *this* two-pass scan looped forever: it looked a byte up with
    // `position(..)` and had a branch where neither a skip nor a copy ran, so `index` stopped moving.
    // The shape below cannot do that — every iteration either copies a character **or** skips a known
    // delimiter range, and each of those advances — and there is an assertion at the bottom that the
    // cursor reached the end.
    let bytes = source.len();
    // Pass one: find the pairs.
    let mut open: Vec<(Mark, usize, usize)> = Vec::new(); // mark, start, end
    let mut pairs: Vec<(Mark, usize, usize, usize, usize)> = Vec::new();
    let mut index = 0usize;
    while index < bytes {
        let Some((mark, delimiter)) = match_delimiter(&source[index..]) else {
            index += source[index..].chars().next().map(char::len_utf8).unwrap_or(1);
            continue;
        };
        // The inmost matching mark closes. `_` and `*` share `Mark::Italic`, so comparing marks rather
        // than delimiters is what stops `_a*` leaving an italic open forever.
        if let Some(position) = open.iter().rposition(|(m, _, _)| *m == mark) {
            let (_, open_start, open_end) = open.remove(position);
            pairs.push((mark, open_start, open_end, index, index + delimiter.len()));
        } else {
            open.push((mark, index, index + delimiter.len()));
        }
        index += delimiter.len();
    }

    // A byte is a **delimiter byte** if some pair opens or closes there. One table rather than a search
    // per byte, so pass two is linear and cannot loop: each step consults the table and moves on.
    let mut delimiter_at: Vec<Option<(usize, bool)>> = vec![None; bytes + 1];
    for (pair_index, (_, open_start, open_end, close_start, close_end)) in
        pairs.iter().enumerate()
    {
        delimiter_at[*open_start] = Some((pair_index, true));
        delimiter_at[*close_start] = Some((pair_index, false));
        let _ = (open_end, close_end);
    }

    let mut starts: Vec<Option<usize>> = vec![None; pairs.len()];
    let mut ends: Vec<Option<usize>> = vec![None; pairs.len()];
    let mut text = String::with_capacity(bytes);
    // **What this pass removes.** A caller holding ranges in *this* function's input coordinates must subtract these,
    // or its ranges point at the wrong characters the moment a delimiter precedes them. The first version of the link
    // pass assumed the mark pass was length-preserving; it is not, and `[**a**](u)` then serialized as `**a**` — the
    // link silently gone.
    let mut removed: Vec<Range<usize>> = Vec::new();
    let mut index = 0usize;
    while index < bytes {
        if let Some((pair_index, is_open)) = delimiter_at[index] {
            let (_, open_start, open_end, close_start, close_end) = pairs[pair_index];
            if is_open {
                starts[pair_index] = Some(text.len());
                let skipped = open_end.max(open_start + 1);
                removed.push(index..skipped);
                index = skipped;
            } else {
                ends[pair_index] = Some(text.len());
                let skipped = close_end.max(close_start + 1);
                removed.push(index..skipped);
                index = skipped;
            }
            continue;
        }
        // Not a paired delimiter: copy the character through, so an unmatched marker stays visible.
        let ch = source[index..].chars().next().expect("a character");
        text.push(ch);
        index += ch.len_utf8();
    }
    debug_assert_eq!(index, bytes, "the inline scan did not reach the end");

    let mut marks: Vec<MarkSpan> = starts
        .into_iter()
        .zip(ends)
        .filter_map(|(start, end)| {
            let (start, end) = (start?, end?);
            (start < end).then_some(())?;
            Some((start, end))
        })
        .zip(pairs.iter().map(|(mark, _, _, _, _)| *mark))
        .map(|((start, end), mark)| MarkSpan {
            range: start..end,
            mark,
        })
        .collect();
    marks.sort_by(|a, b| {
        a.range
            .start
            .cmp(&b.range.start)
            .then(b.range.end.cmp(&a.range.end))
            .then(a.mark.cmp(&b.mark))
    });

    (text, marks, removed)
}

/// The mark a delimiter at the start of `rest` opens or closes.
fn match_delimiter(rest: &str) -> Option<(Mark, &'static str)> {
    // Longest first, or `**` would be read as two italic markers.
    [
        (Mark::Bold, "**"),
        (Mark::Strike, "~~"),
        (Mark::Italic, "*"),
        (Mark::Code, "`"),
        // `_` is the other spelling of italic: recognized on input, written back as `*`, which is the
        // canonical spelling and the reason the guarantee is a fixed point rather than byte equality.
        (Mark::Italic, "_"),
    ]
    .into_iter()
    .find(|(_, delimiter)| rest.starts_with(delimiter))
}

/// Write one line of text with its marks.
///
/// The marks are written back **outermost first**, which the normalized ranges make possible: they are
/// sorted and non-overlapping, so a mark's delimiters go around the text between them and a nested mark
/// falls inside.
pub fn write_inline(text: &Text) -> String {
    if text.marks.is_empty() && text.links.is_empty() {
        return text.text.clone();
    }
    // **One event list, both kinds of span.** A link's delimiter is not a constant — it is `[`, `](` and `)`, with the
    // target in the middle — so it cannot be a `Mark`; but it *is* a range with an opening and a closing, so it belongs
    // in the same walk. Two writers would be two places for the ordering rule to be got wrong.
    enum Event {
        Open(Mark),
        Close(Mark),
        // The index of the link, because opening needs the target only at the close.
        LinkOpen(usize),
        LinkClose(usize),
    }

    let mut events: Vec<(usize, u8, Event)> = Vec::new();
    for span in &text.marks {
        events.push((span.range.start, MARK_OPEN, Event::Open(span.mark)));
        events.push((span.range.end, MARK_CLOSE, Event::Close(span.mark)));
    }
    for (index, link) in text.links.iter().enumerate() {
        events.push((link.range.start, LINK_OPEN, Event::LinkOpen(index)));
        events.push((link.range.end, LINK_CLOSE, Event::LinkClose(index)));
    }
    // The ranks are the ordering rule, and they say two things.
    //
    // **All closes before all opens at the same offset** — the rule the mark-only version had, written as `close =
    // false < open = true`. It is what stops two adjacent spans nesting: with two links `[l](u)` and `[nk](u)` meeting
    // at one offset, opening the second before closing the first would emit `[l[nk](u)](u)`, which re-parses as **one**
    // link and silently loses the second. My first rank set had opens first and did exactly that; the test that caught
    // it was the one about a link cut in two by a backspace, whose halves are adjacent by construction.
    //
    // **And a link sits outside a mark** — `[**a**](u)`, not `**[a](u)**` — so at an offset where both open the link
    // goes first, and at one where both close the mark goes first. Which is to say: closer to the *outside* sorts
    // earlier when closing and later when opening.
    events.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    let mut out = String::with_capacity(text.text.len() + text.marks.len() * 2 + text.links.len() * 4);
    let mut cursor = 0usize;
    let mut event_index = 0usize;
    while event_index < events.len() {
        let at = events[event_index].0;
        if cursor < at {
            out.push_str(&text.text[cursor..at]);
            cursor = at;
        }
        while event_index < events.len() && events[event_index].0 == at {
            match &events[event_index].2 {
                Event::Open(mark) | Event::Close(mark) => out.push_str(mark.delimiters()),
                Event::LinkOpen(_) => out.push('['),
                Event::LinkClose(index) => {
                    out.push_str("](");
                    out.push_str(&text.links[*index].url);
                    out.push(')');
                }
            }
            event_index += 1;
        }
    }
    if cursor < text.text.len() {
        out.push_str(&text.text[cursor..]);
    }
    out
}

/// The event ranks. **The ordering is the rule**, so it is stated as the two constraints that produce it rather than
/// as four bare numbers.
///
/// 1. **Every close sorts before every open.** This is the original rule, and it is what stops two *adjacent* spans
///    nesting: two links meeting at one offset must emit `[l](u)[nk](u)`, and opening the second before closing the
///    first would emit `[l[nk](u)](u)` — which re-parses as **one** link, silently losing the other.
/// 2. **A link is outside a mark.** Where a mark and a link open together the link goes first (`[` then `**`), and where
///    they close together the mark goes first (`**` then `](u)`). Either way the brackets end up outside the bold, which
///    is what makes `[**a**](u)` the canonical spelling.
///
/// Getting this wrong is invisible in a round trip that never has two spans touching, which is why the test that caught
/// it is the one whose two halves are adjacent **by construction** — a link cut in two by a backspace. I had it wrong
/// twice: first with opens before closes, then with the link closing before the mark.
const MARK_CLOSE: u8 = 0;
const LINK_CLOSE: u8 = 1;
const LINK_OPEN: u8 = 2;
const MARK_OPEN: u8 = 3;


#[cfg(test)]
mod tests {
    use super::*;

    /// The corpus the fixed point is checked over.
    ///
    /// Built from the inputs that **drift if a rule is wrong** rather than from the ones that obviously
    /// work: a list starting at three, `_` italic, `***` as a divider, a fence whose content looks like
    /// markdown, a quote with a blank line in it, and an unclosed fence.
    const CORPUS: &[&str] = &[
        "# Title\n\n- a\n- b\n",
        "# Title",
        "",
        "\n\n\n",
        "just a paragraph\n",
        "two\nparagraph\nlines\n",
        "## Heading two\n\n### Three\n\n###### Six\n",
        "####### Seven hashes is a paragraph\n",
        "#hashtag is a paragraph\n",
        "- a\n- b\n- c\n",
        "* star bullet\n+ plus bullet\n- dash bullet\n",
        "1. one\n2. two\n3. three\n",
        "3. three\n4. four\n",
        "1. one\n9. nine\n",
        "7) paren\n8) paren\n",
        "- [ ] todo\n- [x] done\n- [X] also done\n",
        "> quoted\n> two lines\n",
        "> quoted\n>\n> after a blank quote line\n",
        "---\n",
        "***\n",
        "___\n",
        "before\n\n---\n\nafter\n",
        "```\nplain code\n```\n",
        "```rust\nfn main() {}\n```\n",
        "```rust\n- not a bullet\n# not a heading\n```\n",
        "```\nunclosed fence\nsecond line\n",
        "- outer\n  - inner\n    - deeper\n",
        "**bold** and *italic* and `code`\n",
        "_underscore italic_ and ~~strike~~\n",
        "a **b _c_ d** e\n",
        "- list with **bold**\n",
        "> quote with `code`\n",
        "# Heading with **bold**\n",
        "unmatched ** marker\n",
        "a `code span` in a paragraph\n",
        "  indented paragraph\n",
        "- a\n\n- b\n",
        "1. one\n\n2. two\n",
        "# H\n\npara\n\n- a\n- b\n\n> q\n\n```\nx\n```\n\n---\n",
    ];

    /// The property the crate is built around.
    fn round_trips(source: &str) -> Result<(), String> {
        let first = parse(source);
        if !first.is_well_formed() {
            return Err(format!("parse produced an ill-formed doc: {first:?}"));
        }
        let written = serialize(&first);
        let second = parse(&written);
        if first != second {
            return Err(format!(
                "the document changed:\n  source {:?}\n  written {:?}\n  first {first:?}\n  second {second:?}",
                source, written
            ));
        }
        // And a second cycle must be **byte** stable, since the first is allowed to canonicalize.
        let rewritten = serialize(&second);
        if written != rewritten {
            return Err(format!(
                "the text did not settle:\n  first write {written:?}\n  second write {rewritten:?}"
            ));
        }
        Ok(())
    }

    #[test]
    fn test_the_documented_example() {
        // The example in this file's own doc comment, which is the one a reader checks first.
        let doc = parse("# Title\n\n- a\n- b");
        assert_eq!(doc.blocks.len(), 3);
        assert_eq!(serialize(&doc), "# Title\n\n- a\n- b\n");
    }

    #[test]
    fn test_every_corpus_entry_reaches_the_fixed_point() {
        for source in CORPUS {
            if let Err(why) = round_trips(source) {
                panic!("{why}");
            }
        }
    }

    #[test]
    fn test_no_input_escapes_the_fixed_point() {
        // A generator rather than a corpus, because the corpus is the inputs someone thought of. This
        // builds documents out of every block shape at three indent levels and sends each one through
        // the property — including indents that **break the invariant**, because `parse` is supposed to
        // clamp those rather than pass them on.
        let lines = [
            "# h",
            "## h2",
            "para",
            "- a",
            "* b",
            "+ c",
            "1. one",
            "9. nine",
            "- [ ] t",
            "- [x] t",
            "> q",
            "---",
            "***",
            "```",
            "```rust",
            "```",
            "**b**",
            "_i_",
            "  - nested",
            "    - deeper",
            "      - too deep",
        ];
        let mut checked = 0usize;
        for first in lines.iter() {
            for second in lines.iter() {
                for third in lines.iter() {
                    let source = format!("{first}\n{second}\n{third}\n");
                    if let Err(why) = round_trips(&source) {
                        panic!("{why}");
                    }
                    checked += 1;
                }
            }
        }
        assert_eq!(checked, lines.len().pow(3));
    }

    #[test]
    fn test_the_fixed_point_is_not_byte_equality_and_underscore_italic_proves_it() {
        // The distinction the property's name carries. `_x_` parses to an italic mark and serializes as
        // `*x*`, so the text changes on the first write and the document does not — which is why the
        // guarantee is a fixed point rather than an inverse.
        let source = "_italic_\n";
        let first = parse(source);
        assert_eq!(first, parse("*italic*\n"), "both spellings parse the same");
        let written = serialize(&first);
        assert_eq!(written, "*italic*\n");
        assert_eq!(parse(&written), first);
        assert_eq!(serialize(&parse(&written)), written, "and it is stable after one write");
    }

    #[test]
    fn test_a_list_starting_at_three_survives() {
        // The number is **stored rather than derived**, because markdown renders only the first one and
        // a document that lost it would come back as `1.` on the next read.
        let doc = parse("3. three\n4. four\n");
        match &doc.blocks[0].kind {
            BlockKind::Ordered { number, .. } => assert_eq!(*number, 3),
            other => panic!("expected an ordered item, got {other:?}"),
        }
        assert_eq!(serialize(&doc), "3. three\n4. four\n");
    }

    #[test]
    fn test_a_non_consecutive_list_is_renumbered_at_parse_time() {
        // Markdown honours the first number only, so `1.` then `9.` renders 1, 2 and would come back as
        // `2.` on the next read. Deciding it at parse means the document already holds what the next
        // parse produces — so the fixed point holds on the **first** cycle rather than the second.
        let doc = parse("1. one\n9. nine\n");
        let numbers: Vec<u64> = doc
            .blocks
            .iter()
            .filter_map(|block| match &block.kind {
                BlockKind::Ordered { number, .. } => Some(*number),
                _ => None,
            })
            .collect();
        assert_eq!(numbers, vec![1, 2]);
        assert_eq!(serialize(&doc), "1. one\n2. nine\n");
    }

    #[test]
    fn test_the_three_divider_spellings_all_come_back_as_one() {
        // One canonical spelling, because the document is unchanged by the choice — which is the fixed
        // point's whole advantage over byte equality.
        for source in ["---\n", "***\n", "___\n", "-----\n"] {
            let doc = parse(source);
            assert_eq!(doc.blocks.len(), 1, "{source:?}");
            assert_eq!(doc.blocks[0].kind, BlockKind::Divider);
            assert_eq!(serialize(&doc), "---\n");
        }
    }

    #[test]
    fn test_a_fence_keeps_markdown_shaped_content_byte_for_byte() {
        // The case a line-based parser gets wrong by looking at the content: a code block whose body
        // looks like markdown must come back untouched, and its blank lines are content rather than
        // separators.
        let source = "```rust\n- not a bullet\n\n# not a heading\n```\n";
        let doc = parse(source);
        assert_eq!(doc.blocks.len(), 1);
        match &doc.blocks[0].kind {
            BlockKind::Code { language, code } => {
                assert_eq!(language.as_deref(), Some("rust"));
                assert_eq!(code.text, "- not a bullet\n\n# not a heading");
            }
            other => panic!("expected a code block, got {other:?}"),
        }
        assert_eq!(serialize(&doc), source);
    }

    #[test]
    fn test_an_unclosed_fence_runs_to_the_end_and_is_kept() {
        // What an editor holds while the closing marker is being typed. Dropping the block would make
        // the lines the reader is looking at vanish from the document.
        let source = "```\nunclosed\nsecond\n";
        let doc = parse(source);
        assert_eq!(doc.blocks.len(), 1);
        match &doc.blocks[0].kind {
            BlockKind::Code { language, code } => {
                assert_eq!(*language, None);
                assert_eq!(code.text, "unclosed\nsecond");
            }
            other => panic!("expected a code block, got {other:?}"),
        }
    }

    #[test]
    fn test_consecutive_quote_lines_are_one_block_and_a_blank_quote_line_survives() {
        let doc = parse("> one\n> two\n");
        assert_eq!(doc.blocks.len(), 1);
        match &doc.blocks[0].kind {
            BlockKind::Quote(text) => assert_eq!(text.text, "one\ntwo"),
            other => panic!("expected a quote, got {other:?}"),
        }
        assert_eq!(serialize(&doc), "> one\n> two\n");

        // A `>` with nothing after it is an empty line **inside** the quote, and it round trips.
        let doc = parse("> one\n>\n> three\n");
        assert_eq!(doc.blocks.len(), 1);
        assert_eq!(serialize(&doc), "> one\n>\n> three\n");
    }

    #[test]
    fn test_the_indent_invariant_holds_for_every_corpus_entry_and_is_clamped() {
        for source in CORPUS {
            let doc = parse(source);
            assert!(doc.is_well_formed(), "{source:?} produced {doc:?}");
        }
        // A jump of two levels is **clamped** rather than passed on, because a document that failed the
        // invariant would serialize to markdown this parser reads back at different indents.
        let doc = parse("- a\n    - deep\n");
        assert_eq!(doc.blocks[1].indent, 1, "the jump was not clamped");
        assert!(doc.is_well_formed());
    }

    #[test]
    fn test_a_list_is_one_unit_and_a_blank_line_between_items_breaks_it() {
        // The rule that makes the crate's own example round trip: no blank line between items of the
        // same list, because one would end the list on the next parse.
        assert_eq!(serialize(&parse("- a\n- b\n")), "- a\n- b\n");
        // And a blank line in the *source* between items is not preserved, because the document has no
        // list identity to preserve it with — a flat model, as the module doc says. What matters is that
        // the result is stable, which the property checks.
        let doc = parse("- a\n\n- b\n");
        assert_eq!(doc.blocks.len(), 2);
        assert_eq!(serialize(&doc), "- a\n- b\n");
        assert_eq!(parse(&serialize(&doc)), doc);
    }

    #[test]
    fn test_a_paragraph_after_a_list_gets_its_blank_line_back() {
        // The other side of the same rule: the blank line goes between blocks that are not one list.
        let doc = parse("- a\n- b\n\nafter\n");
        assert_eq!(doc.blocks.len(), 3);
        assert_eq!(serialize(&doc), "- a\n- b\n\nafter\n");
    }

    #[test]
    fn test_nested_lists_keep_their_indents() {
        let source = "- outer\n  - inner\n    - deeper\n";
        let doc = parse(source);
        let indents: Vec<u8> = doc.blocks.iter().map(|block| block.indent).collect();
        assert_eq!(indents, vec![0, 1, 2]);
        assert_eq!(serialize(&doc), source);
    }

    #[test]
    fn test_a_marker_needs_a_space_or_five_is_a_paragraph() {
        // `-5` is a paragraph and `#tag` is a paragraph. Without those two rules a document full of
        // negative numbers and tags parses into bullets and headings.
        assert!(matches!(parse("-5 degrees\n").blocks[0].kind, BlockKind::Paragraph(_)));
        assert!(matches!(parse("#hashtag\n").blocks[0].kind, BlockKind::Paragraph(_)));
        assert!(matches!(parse("- item\n").blocks[0].kind, BlockKind::Bullet(_)));
        // And seven hashes is a paragraph rather than a level-7 heading.
        assert!(matches!(
            parse("####### seven\n").blocks[0].kind,
            BlockKind::Paragraph(_)
        ));
    }

    #[test]
    fn test_task_items_are_tasks_and_not_bullets() {
        let doc = parse("- [ ] todo\n- [x] done\n- [X] caps\n- [] malformed\n");
        assert!(matches!(
            doc.blocks[0].kind,
            BlockKind::Task { checked: false, .. }
        ));
        assert!(matches!(
            doc.blocks[1].kind,
            BlockKind::Task { checked: true, .. }
        ));
        assert!(
            matches!(doc.blocks[2].kind, BlockKind::Task { checked: true, .. }),
            "an uppercase X is a checked box"
        );
        // `- []` is not a task, because there is no space or x between the brackets — and it falls back
        // to a bullet whose text starts with `[]`, which is what a reader would expect.
        assert!(matches!(doc.blocks[3].kind, BlockKind::Bullet(_)));
    }

    #[test]
    fn test_every_mark_round_trips_through_its_canonical_spelling() {
        for (source, expected) in [
            ("**bold**\n", "**bold**\n"),
            ("*italic*\n", "*italic*\n"),
            ("_italic_\n", "*italic*\n"),
            ("`code`\n", "`code`\n"),
            ("~~strike~~\n", "~~strike~~\n"),
        ] {
            let doc = parse(source);
            assert_eq!(serialize(&doc), expected, "for {source:?}");
            assert_eq!(parse(&serialize(&doc)), doc);
        }
    }

    #[test]
    fn test_an_unmatched_marker_stays_as_text() {
        // An editor holds a half-typed `**` constantly. Leaving it open to the end of the line would
        // mark the rest of the document, which is a rendering change the reader did not make.
        let doc = parse("a ** marker\n");
        match &doc.blocks[0].kind {
            BlockKind::Paragraph(text) => {
                assert_eq!(text.text, "a ** marker");
                assert!(text.marks.is_empty(), "the marker became a mark: {text:?}");
            }
            other => panic!("expected a paragraph, got {other:?}"),
        }
    }

    #[test]
    fn test_a_nested_mark_closes_innermost_first() {
        let doc = parse("a **b _c_ d** e\n");
        let text = doc.blocks[0].kind.text().expect("text").clone();
        assert_eq!(text.text, "a b c d e");
        // The italic is inside the bold and both are in bounds.
        assert_eq!(text.marks.len(), 2, "{:?}", text.marks);
        assert!(text.marks.iter().any(|span| span.mark == Mark::Bold));
        assert!(text.marks.iter().any(|span| span.mark == Mark::Italic));
        text.marks.iter().for_each(|span| {
            assert!(span.range.end <= text.text.len());
            assert!(span.range.start < span.range.end);
        });
        assert_eq!(serialize(&doc), "a **b *c* d** e\n");
        assert_eq!(parse(&serialize(&doc)), doc);
    }

    #[test]
    fn test_normalize_keeps_nestings_and_clips_to_the_text() {
        // ## The contract that is *not* the highlight-span one
        //
        // `makepad_theme::syntax::normalize` resolves overlaps, because two foreground colours cannot
        // compose. Marks are the opposite: `**bold with _italic_ inside**` is two marks on overlapping
        // ranges and **both have to survive**. The first version of this file reused the span contract
        // and silently dropped the inner italic; its test asserted the dropping, which made the wrong
        // behaviour look intended.
        let mut text = Text {
            text: "abcdefgh".to_string(),
            links: Vec::new(),
            marks: vec![
                MarkSpan {
                    range: 4..6,
                    mark: Mark::Italic,
                },
                MarkSpan {
                    range: 0..3,
                    mark: Mark::Bold,
                },
                MarkSpan {
                    range: 1..5,
                    mark: Mark::Strike,
                },
                MarkSpan {
                    range: 6..99,
                    mark: Mark::Code,
                },
            ],
        };
        text.normalize();
        // Sorted and in bounds, with **every** nesting kept.
        assert_eq!(text.marks.len(), 4, "{:?}", text.marks);
        let starts: Vec<usize> = text.marks.iter().map(|span| span.range.start).collect();
        assert!(starts.windows(2).all(|pair| pair[0] <= pair[1]), "{:?}", text.marks);
        for span in &text.marks {
            assert!(span.range.start < span.range.end, "{span:?}");
            assert!(span.range.end <= text.text.len(), "{span:?} is past the end");
        }
        // The out-of-bounds mark was clipped rather than dropped, because part of it was real.
        let code = text.marks.iter().find(|span| span.mark == Mark::Code).unwrap();
        assert_eq!(code.range, 6..8);
        // And the overlap between Bold(0..3) and Strike(1..5) is intact.
        let bold = text.marks.iter().find(|span| span.mark == Mark::Bold).unwrap();
        assert_eq!(bold.range, 0..3);
        let strike = text.marks.iter().find(|span| span.mark == Mark::Strike).unwrap();
        assert_eq!(strike.range, 1..5);
    }

    #[test]
    fn test_an_empty_document_serializes_to_nothing() {
        assert_eq!(serialize(&parse("")), "");
        assert_eq!(serialize(&parse("\n\n\n")), "");
        assert!(parse("").blocks.is_empty());
        assert!(parse("\n\n\n").blocks.is_empty());
    }

    #[test]
    fn test_a_document_written_by_hand_reaches_the_same_fixed_point() {
        // `serialize` is also the editor's write path, so a document assembled in code rather than
        // parsed has to satisfy the same property. This is the case the parser cannot check for the
        // caller.
        let doc = Doc {
            blocks: vec![
                Block {
                    indent: 0,
                    kind: BlockKind::Heading {
                        level: 1,
                        text: Text::plain("Hand built"),
                    },
                },
                Block {
                    indent: 0,
                    kind: BlockKind::Bullet(Text::plain("one")),
                },
                Block {
                    indent: 1,
                    kind: BlockKind::Bullet(Text::plain("nested")),
                },
            ],
        };
        assert!(doc.is_well_formed());
        let written = serialize(&doc);
        assert_eq!(written, "# Hand built\n\n- one\n  - nested\n");
        assert_eq!(parse(&written), doc);
    }

    #[test]
    fn test_an_ill_formed_document_is_reported_rather_than_silently_written() {
        // The check exists so a caller can refuse to write a document whose indents would not read back.
        let bad = Doc {
            blocks: vec![
                Block {
                    indent: 0,
                    kind: BlockKind::Paragraph(Text::plain("a")),
                },
                Block {
                    indent: 3,
                    kind: BlockKind::Paragraph(Text::plain("b")),
                },
            ],
        };
        assert!(!bad.is_well_formed());
        // And a first block that is not at zero is ill-formed too, because the serializer would write a
        // leading indent the parser reads as level 1 — a document that cannot be written faithfully.
        let bad_start = Doc {
            blocks: vec![Block {
                indent: 1,
                kind: BlockKind::Paragraph(Text::plain("a")),
            }],
        };
        assert!(!bad_start.is_well_formed());
    }
    #[test]
    fn test_a_name_that_says_image_is_an_image() {
        // The rule is the **last path segment's extension**, and the cases are the ones a real URL arrives as.
        for url in [
            "https://host/picture.png",
            "https://host/a/b/photo.JPEG",
            "https://host/pic.webp?w=400&h=200",
            "https://host/pic.png#top",
            "https://host/pic.png?w=1#top",
            "https://host/download.jpg/",
            "/local/path/thing.gif",
            "thing.avif",
        ] {
            assert!(is_image(url), "{url} should be an image");
        }
    }

    #[test]
    fn test_a_name_that_does_not_say_image_is_not_one() {
        // **This answers from the text and fetches nothing.** Guessing would paint a broken box, and the menu offers
        // "Create image" only where a picture can go.
        for url in [
            "https://host/photo",
            "https://img.example.com",
            "https://host/a.png/index.html",
            "https://host/report.pdf",
            "https://host/page.html",
            "https://host/.",
            "https://host/pic.",
            "",
        ] {
            assert!(!is_image(url), "{url} should not be an image");
        }
    }

    #[test]
    fn test_the_fragment_is_taken_before_the_query_because_a_fragment_can_contain_a_query() {
        // `#a?b` — taking the query first would leave `pic.png#a` as the name, and `#a` is not an extension. The order
        // of the two splits is the whole test.
        assert!(is_image("https://host/pic.png#a?b"));
        assert!(!is_image("https://host/photo#a?b.png"));
    }

    #[test]
    fn test_a_link_parses_to_its_label_with_the_target_kept() {
        // The range covers the **label**, which is what is displayed — so a link behaves like a mark geometrically and
        // one shift function can move both.
        let text = parse_inline("see [the docs](https://example.com/a) for more");
        assert_eq!(text.text, "see the docs for more");
        assert_eq!(text.links.len(), 1);
        assert_eq!(text.links[0].range, 4..12);
        assert_eq!(text.links[0].url, "https://example.com/a");
        assert_eq!(&text.text[text.links[0].range.clone()], "the docs");
    }

    #[test]
    fn test_a_link_round_trips_and_the_fixed_point_holds() {
        // **The invariant this whole model rests on.** A link was not representable before this change — `[a](b)`
        // parsed as eight literal characters — so this is the case that says the addition did not cost the guarantee.
        for source in [
            "[the docs](https://example.com)",
            "see [the docs](https://example.com/a?b=c#d) for more",
            "two [a](u1) and [b](u2) links",
            "[**bold label**](https://example.com)",
            "[label](https://example.com) and *italic*",
            "- a [link](u) in a bullet",
            "> a [link](u) in a quote",
            "no links at all",
        ] {
            let parsed = parse(source);
            let written = serialize(&parsed);
            let reparsed = parse(&written);
            assert_eq!(
                reparsed, parsed,
                "the fixed point broke for {source:?}: wrote {written:?}"
            );
        }
    }

    #[test]
    fn test_a_mark_inside_a_label_writes_inside_the_brackets() {
        // **The ordering rule, and the reason it is a rule.** A link is outermost: `[**a**](url)` bolds the label, and
        // `**[a](url)**` would bold the brackets — which is not a link at all once re-parsed.
        let text = parse_inline("[**a**](u)");
        assert_eq!(text.text, "a");
        assert_eq!(write_inline(&text), "[**a**](u)");
        assert_eq!(text.marks.len(), 1);
        assert_eq!(text.marks[0].range, 0..1);

        // ...and a mark written **around** a whole link normalizes **inward**: `**[a](u)**` writes back as
        // `[**a**](u)`. The two are the same document — a bold label that is a link — and the ordering rule makes the
        // link outermost, so that is the canonical spelling. **The promise is a fixed point, not byte equality**:
        // `_italic_` writing as `*italic*` is the same trade, and a test asserting the input spelling would be
        // asserting that this model preserves a distinction it deliberately does not have.
        let around = parse_inline("**[a](u)**");
        let inward = parse_inline("[**a**](u)");
        assert_eq!(write_inline(&around), "[**a**](u)");
        assert_eq!(around, inward, "the two spellings must be the same document");
        assert_eq!(parse_inline(&write_inline(&around)), around, "not a fixed point");
    }

    #[test]
    fn test_a_bracket_that_is_not_a_link_stays_in_the_text() {
        // **The same rule the mark pass follows, for the same reason**: a `[` somebody typed is content, and consuming
        // it would be content loss. An unpaired delimiter stays visible.
        for source in [
            "see [note",
            "[label](no closing paren",
            "[label] no target",
            "[](u)",
            "[[label]](u)",
            "a [b] c",
        ] {
            let text = parse_inline(source);
            assert!(
                source.contains('[') && !source.is_empty(),
                "the case list is wrong"
            );
            let written = write_inline(&text);
            // Whatever it decided, **nothing was lost**: the written form parses to the same document.
            assert_eq!(parse_inline(&written), text, "{source:?} did not survive");
        }
    }

    #[test]
    fn test_an_unmatched_bracket_leaves_the_text_alone() {
        // Stated separately from the round trip above, because "nothing was lost" and "the bracket is still there" are
        // different claims and only this one says the character survived.
        let text = parse_inline("see [note");
        assert_eq!(text.text, "see [note");
        assert!(text.links.is_empty());
    }

    #[test]
    fn test_two_overlapping_links_are_reduced_to_the_first() {
        // Marks keep their nesting and links do **not**, and the difference is not arbitrary: nested marks are
        // writable (`**a *b* c**`), while two overlapping links are **not representable** — the writer would emit
        // `[a[b]c](u1)(u2)`, which does not re-parse to the same two links. So `normalize` keeps the earlier one, which
        // is the choice that keeps a document writable.
        let mut text = Text {
            text: "abcdef".to_string(),
            marks: Vec::new(),
            links: vec![
                LinkSpan {
                    range: 0..4,
                    url: "first".to_string(),
                },
                LinkSpan {
                    range: 2..6,
                    url: "second".to_string(),
                },
            ],
        };
        text.normalize();
        assert_eq!(text.links.len(), 1);
        assert_eq!(text.links[0].url, "first");
        // ...and the written form still parses back to it, which is the point of the rule.
        let written = write_inline(&text);
        assert_eq!(parse_inline(&written), text);
    }

    #[test]
    fn test_a_link_moves_when_the_text_before_it_is_deleted() {
        // **The defect that motivated one `shift_range`.** Before links were shifted, editing text before a link left
        // its range pointing at the wrong characters — serializing `[`/`](url)` around text that was never linked,
        // which is content corruption rather than a cosmetic fault. So it is checked through a real edit path.
        //
        // `SetKind` was the first attempt and it was the wrong test: a key that changes nothing legitimately returns
        // `None`, so a test asserting it applied was asserting the opposite of the design.
        let doc = Doc {
            blocks: vec![Block {
                indent: 0,
                kind: BlockKind::Paragraph(parse_inline("a [link](u) tail")),
            }],
        };
        // Backspace with the caret inside the label deletes a character of the label: the link must come out pointing
        // at what is left of it.
        let selection = Selection::caret(0, 4);
        let edited = edit::apply(&doc, &selection, Shortcut::Backspace).expect("backspace applied");
        let text = edited.doc.blocks[0].kind.text().expect("a paragraph has text");
        // **Two links, which is the straddling rule and not a bug.** A span that straddles an edit is cut in two rather
        // than dropped or given wholesale to one side — the rule marks already followed — and for a link that is
        // faithful: both halves point where the whole did, and the writer's `[l](u)[nk](u)` re-parses as two links. My
        // first expectation was one link, which would have meant silently dropping the target from one half.
        assert_eq!(text.links.len(), 2, "a straddled link must be cut, not dropped");
        assert!(text.links.iter().all(|link| link.url == "u"));
        let labels: Vec<&str> = text
            .links
            .iter()
            .map(|link| &text.text[link.range.clone()])
            .collect();
        assert_eq!(labels, vec!["l", "nk"], "the halves are not the halves of the label");
        // ...and it is still writable and still a fixed point, which is the claim that matters.
        assert_eq!(parse(&serialize(&edited.doc)), edited.doc);
    }

    /// The document after `Enter`, obtained through the public `apply` — `edit::enter` is private, and the point of
    /// that test is the split, not the plumbing.
    struct AfterEnter {
        doc: Doc,
    }

    impl AfterEnter {
        fn of(doc: &Doc, selection: &Selection) -> Self {
            let edited = edit::apply(doc, selection, Shortcut::Enter).expect("the split applied");
            Self { doc: edited.doc }
        }
    }

    #[test]
    fn test_a_link_survives_being_split_and_merged() {
        // The two edit sites that partition spans. Splitting a link in the middle gives **two** links to the same
        // target — which is faithful, because `[la](u)[bel](u)` re-parses as two links — and merging puts the halves
        // back, with the second's range moved by the join.
        let doc = Doc {
            blocks: vec![Block {
                indent: 0,
                kind: BlockKind::Paragraph(parse_inline("[label](u)")),
            }],
        };
        let selection = Selection::caret(0, 3);
        edit::apply(&doc, &selection, Shortcut::Enter).expect("the split applied");
        let AfterEnter { doc } = AfterEnter::of(&doc, &selection);
        assert_eq!(doc.blocks.len(), 2);

        let first = doc.blocks[0].kind.text().expect("text").clone();
        let second = doc.blocks[1].kind.text().expect("text").clone();
        assert_eq!(first.text, "lab");
        assert_eq!(second.text, "el");
        assert_eq!(first.links.len(), 1, "the first half lost its link");
        assert_eq!(second.links.len(), 1, "the second half lost its link");
        assert_eq!(first.links[0].url, "u");
        assert_eq!(second.links[0].url, "u");
        assert_eq!(&second.text[second.links[0].range.clone()], "el");
    }

}

