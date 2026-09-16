//! Editing a document: what a key does, and the invariant no key may break.
//!
//! ## Why the flat model pays off here
//!
//! The module doc in `lib.rs` says the shape was chosen because *"editing a flat list means Enter splits,
//! Backspace merges and Tab indents — all list operations"*. This is where that claim is cashed: every
//! operation below is a `Vec` operation plus a byte splice, and none of them traverses anything. On a
//! nested tree, "the previous block" would be a traversal and each of these would be a restructure.
//!
//! ## The invariant every operation maintains
//!
//! [`Doc::is_well_formed`] — the first block at indent 0, and no block more than one level deeper than the
//! one before it. **Every operation here preserves it**, and that is not a happy accident: the serializer
//! relies on it, so an edit that broke it would produce a document that could not be written back
//! faithfully. The two places it constrains are Tab, which may not indent past `previous + 1`, and Enter
//! on a list item, which must not create a child of a block that cannot have one.
//!
//! There is a property test at the bottom of this file that applies thousands of random shortcut
//! sequences and asserts the invariant after **every** one, because the way to break it is a combination
//! — Enter then Tab then Backspace — rather than any single key.
//!
//! ## The editing rules, and where each comes from
//!
//! Most of them are Notion's, because that is the model this crate adopted:
//!
//! | key | in | does |
//! |---|---|---|
//! | Enter | end of a heading/quote/code block | a new **paragraph**, not another heading |
//! | Enter | a list item | continues the list, and an ordered one **renumbers** |
//! | Enter | an **empty** list item | outdents it, and at indent 0 turns it into a paragraph |
//! | Backspace | at offset 0 of an indented item | outdents it first |
//! | Backspace | at offset 0 otherwise | merges into the previous block |
//! | Tab | anywhere | indents, **bounded by the invariant** |
//!
//! The empty-item rule is the one a reader notices: without it there is no way to leave a list except by
//! deleting the marker, and a list is the block kind a document has most of.

use std::ops::Range;

use crate::{Block, BlockKind, Doc, Mark, MarkSpan, Text};

/// Where the caret is, and what is selected.
///
/// A range rather than an offset, because a shortcut has to know what text it is marking: `ToggleMark`
/// over an empty range is a no-op, and `Backspace` over one eats a character while over a range it eats the
/// range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    pub block: usize,
    pub range: Range<usize>,
}

/// A caret at the start of the document.
///
/// Exists because a host's state fields must be constructible before the first event — the reason
/// `makepad-editor`'s stack has one too — and because "nothing selected, at the top" is the only
/// sensible default for a selection.
impl Default for Selection {
    fn default() -> Self {
        Self::caret(0, 0)
    }
}

impl Selection {
    /// A caret with nothing selected.
    pub fn caret(block: usize, offset: usize) -> Self {
        Self {
            block,
            range: offset..offset,
        }
    }

    pub fn is_caret(&self) -> bool {
        self.range.start == self.range.end
    }

    pub fn offset(&self) -> usize {
        self.range.end
    }
}

/// What a key does to a document.
///
/// A closed set, like every other vocabulary here, and **deliberately not the whole keyboard**: a
/// shortcut in this enum is one that changes the *document*, and moving a caret is not one of them. The
/// paint half owns the keys that only move.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shortcut {
    /// Split the block at the caret.
    Enter,
    /// Delete the selection, or the character before the caret, or merge with the previous block.
    Backspace,
    /// Delete the selection, or the character after the caret.
    Delete,
    /// Indent by one level, bounded by the invariant.
    Indent,
    /// Outdent by one level.
    Outdent,
    /// Turn a mark on or off over the selection.
    ToggleMark(Mark),
    /// Turn the block into another kind, keeping its text.
    ///
    /// One variant rather than a `Bullet`/`Heading`/... per name: the set of kinds is already closed in
    /// `BlockKind`, and a shortcut that repeated it would be a second list to keep in step.
    SetKind(SetKind),
}

/// The kinds a shortcut can turn a block into, as data.
///
/// A small closed enum rather than a `BlockKind`, because two of the model's variants carry data a key
/// cannot supply: an `Image` needs a URL and a `Code` block needs a language. What a key can do is choose
/// a *shape*, and this is exactly the set of shapes it can choose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetKind {
    Paragraph,
    Heading1,
    Heading2,
    Heading3,
    Bullet,
    Ordered,
    Task,
    Quote,
    Code,
    Divider,
}

impl SetKind {
    /// The block kind this sets, with the data a key cannot supply defaulted.
    pub fn kind(self, text: Text) -> BlockKind {
        match self {
            SetKind::Paragraph => BlockKind::Paragraph(text),
            SetKind::Heading1 => BlockKind::Heading { level: 1, text },
            SetKind::Heading2 => BlockKind::Heading { level: 2, text },
            SetKind::Heading3 => BlockKind::Heading { level: 3, text },
            SetKind::Bullet => BlockKind::Bullet(text),
            // **One** rather than the block's old number: a block becoming an ordered item starts a list,
            // and `Doc::renumber` fixes the run it landed in.
            SetKind::Ordered => BlockKind::Ordered { number: 1, text },
            SetKind::Task => BlockKind::Task {
                checked: false,
                text,
            },
            SetKind::Quote => BlockKind::Quote(text),
            SetKind::Code => BlockKind::Code {
                language: None,
                code: text,
            },
            SetKind::Divider => BlockKind::Divider,
        }
    }

    /// Whether this kind can hold text a caret could be in.
    pub fn holds_text(self) -> bool {
        !matches!(self, SetKind::Divider)
    }
}

/// The document and selection after an edit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edited {
    pub doc: Doc,
    pub selection: Selection,
}

/// Apply a shortcut.
///
/// `None` when the shortcut would change nothing — a `Backspace` at the very start of the document, an
/// `Indent` that the invariant forbids, a `ToggleMark` over an empty range. A `None` is what lets the paint
/// half leave the key unhandled rather than swallowing it, which matters for `Tab` in a document that
/// cannot indent further.
pub fn apply(doc: &Doc, selection: &Selection, shortcut: Shortcut) -> Option<Edited> {
    if doc.blocks.is_empty() {
        return None;
    }
    if selection.block >= doc.blocks.len() {
        return None;
    }
    let original = doc.clone();
    let mut doc = doc.clone();
    let mut sel = selection.clone();

    match shortcut {
        Shortcut::Enter => enter(&mut doc, &mut sel),
        Shortcut::Backspace => backspace(&mut doc, &mut sel),
        Shortcut::Delete => delete(&mut doc, &mut sel),
        Shortcut::Indent => indent(&mut doc, &mut sel, 1),
        Shortcut::Outdent => indent(&mut doc, &mut sel, -1),
        Shortcut::ToggleMark(mark) => toggle_mark(&mut doc, &sel, mark),
        Shortcut::SetKind(kind) => set_kind(&mut doc, &mut sel, kind),
    }
    // **A fence carries no marks.** Markdown has no marks inside a fence — a code block's text is written
    // raw — so a mark that a `SetKind::Code` or a merge left there is *unreachable state that would be
    // silently lost on save*, and the fixed point would break. The random-session test found exactly that:
    // a `Bold` mark inside a code block diverged on the first write. `parse` never creates one; an edit can,
    // so it is cleared here, after every operation rather than inside each.
    for block in &mut doc.blocks {
        if let BlockKind::Code { code, .. } = &mut block.kind {
            code.marks.clear();
        }
    }
    // **The indent invariant, enforced after every edit.** It is a property of the *document* — the first
    // block is at 0 and no block is more than one deeper than the one before it — and it is the serializer's
    // assumption, so a document that violates it cannot be written back faithfully.
    //
    // Every operation above already tries to maintain it, and that was not enough: removing the first block
    // (Backspace merging, Delete) leaves whatever becomes first at its old indent, and one random step found
    // `[Task at indent 1]` as a whole document. **An invariant that every operation must remember is an
    // invariant that one will forget**, so it is established here, once, after whatever happened.
    normalize_indents(&mut doc);
    // **Renumbered after every edit.** An ordered list's numbers depend on what is around it, so a splice
    // that added, removed or re-kinded a block can have changed a run — and doing it here rather than in
    // each operation means no operation can forget.
    doc.renumber();
    // **A shortcut that changed nothing is `None`.** The doc comment promises it, and it is what lets the
    // paint half leave a key unhandled rather than swallowing it — `Backspace` at the very start of a
    // document and a `ToggleMark` over an empty range are both no-ops, and the first version returned
    // `Some(unchanged)` for both.
    if doc == original && sel == *selection {
        return None;
    }
    debug_assert!(
        doc.is_well_formed(),
        "a shortcut broke the indent invariant: {doc:?}"
    );
    Some(Edited { doc, selection: sel })
}

/// Establish the indent invariant on a document.
///
/// The first block at 0, and no block more than one level deeper than the one before it — the two rules
/// [`Doc::is_well_formed`] checks and [`crate::serialize`] relies on. Applied after every edit rather than
/// inside each operation; see the note in `apply`.
fn normalize_indents(doc: &mut Doc) {
    let mut previous = 0u8;
    for (index, block) in doc.blocks.iter_mut().enumerate() {
        if index == 0 {
            block.indent = 0;
        } else if block.indent > previous.saturating_add(1) {
            block.indent = previous.saturating_add(1);
        }
        previous = block.indent;
    }
}

/// The block's text as a `Text`, or an empty one for a divider.
fn text_of(block: &Block) -> Text {
    block.kind.text().cloned().unwrap_or_else(|| Text::plain(""))
}

/// Replace a block's text, keeping its kind.
///
/// A divider has no text, so this leaves it alone — which is what makes Backspace on a divider delete the
/// divider rather than silently doing nothing.
fn set_text(block: &mut Block, text: Text) {
    block.kind = match std::mem::replace(&mut block.kind, BlockKind::Divider) {
        BlockKind::Paragraph(_) => BlockKind::Paragraph(text),
        BlockKind::Heading { level, .. } => BlockKind::Heading { level, text },
        BlockKind::Bullet(_) => BlockKind::Bullet(text),
        BlockKind::Ordered { number, .. } => BlockKind::Ordered { number, text },
        BlockKind::Task { checked, .. } => BlockKind::Task { checked, text },
        BlockKind::Quote(_) => BlockKind::Quote(text),
        BlockKind::Code { language, .. } => BlockKind::Code { language, code: text },
        BlockKind::Divider => BlockKind::Divider,
    };
}

/// The kind a **new** block gets when Enter splits an existing one.
///
/// A heading, a quote and a fence all give a **paragraph**, which is the rule a reader feels most: pressing
/// Enter at the end of a heading is how you stop writing a heading, and a list is the one kind that
/// continues.
fn continuation(kind: &BlockKind) -> BlockKind {
    match kind {
        BlockKind::Bullet(_) => BlockKind::Bullet(Text::plain("")),
        BlockKind::Ordered { number, .. } => BlockKind::Ordered {
            number: number.saturating_add(1),
            text: Text::plain(""),
        },
        BlockKind::Task { .. } => BlockKind::Task {
            // A new task is **unchecked**, because a checked one would be a claim the reader did not make.
            checked: false,
            text: Text::plain(""),
        },
        // Everything else continues as a paragraph.
        _ => BlockKind::Paragraph(Text::plain("")),
    }
}

/// Split the block at the caret.
fn enter(doc: &mut Doc, sel: &mut Selection) {
    let index = sel.block;
    let block = doc.blocks[index].clone();
    let indent = block.indent;
    let text = text_of(&block);

    // A divider has no text to split: Enter after one inserts a paragraph **below** it, which is the only
    // thing a caret under a rule can mean.
    if matches!(block.kind, BlockKind::Divider) {
        doc.blocks.insert(
            index + 1,
            Block {
                indent,
                kind: BlockKind::Paragraph(Text::plain("")),
            },
        );
        *sel = Selection::caret(index + 1, 0);
        return;
    }

    // **The whitespace at the split is dropped, not given to either half.** `parse` trims every line, so a
    // block ending or beginning with a space is not representable — and a split that left one produced a
    // document whose wire form changed on the next read. Dropping it is also what a reader means: pressing
    // Enter at a space should not leave the space dangling at the start of the next line.
    // The whitespace at the split is **dropped**, not given to either half: `parse` trims every line, so a
    // block that ends or begins with a space is not representable, and a split leaving one produced a wire
    // form that changed on the next read. So the head ends at the last non-space before the caret and the
    // tail begins at the first after it.
    //
    // The first version advanced a single `at` past the spaces, which **gives them to the head** —
    // `split_text(text, 5)` is `text[..5]`, which is `"para "`. Two offsets rather than one.
    let caret = clamp_to_boundary(&text.text, sel.range.end);
    let head_end = trim_spaces_before(&text.text, caret);
    let tail_from = skip_spaces_after(&text.text, caret);

    // **Enter on an empty list item leaves the list.** Outdent it; at indent 0 turn it into a paragraph.
    // Without this rule there is no way out of a list except deleting the marker.
    if text.text.is_empty() {
        let is_list = block.kind.is_list_item();
        if is_list && indent > 0 {
            doc.blocks[index].indent = indent - 1;
            *sel = Selection::caret(index, 0);
            return;
        }
        if is_list || matches!(block.kind, BlockKind::Quote(_)) {
            doc.blocks[index].kind = BlockKind::Paragraph(Text::plain(""));
            *sel = Selection::caret(index, 0);
            return;
        }
    }

    // Split the text at the caret, keeping the marks on the side they belong to.
    let (before, after) = split_text(&text, head_end, tail_from);
    set_text(&mut doc.blocks[index], before);
    let new_kind = continuation(&block.kind).with_text(after);
    doc.blocks.insert(
        index + 1,
        Block {
            indent,
            kind: new_kind,
        },
    );
    *sel = Selection::caret(index + 1, 0);
}

impl BlockKind {
    /// The same kind carrying other text, for a split's second half.
    fn with_text(self, text: Text) -> BlockKind {
        match self {
            BlockKind::Bullet(_) => BlockKind::Bullet(text),
            BlockKind::Ordered { number, .. } => BlockKind::Ordered { number, text },
            BlockKind::Task { checked, .. } => BlockKind::Task { checked, text },
            _ => BlockKind::Paragraph(text),
        }
    }
}

/// Split a `Text` at a byte offset, putting each mark on the side it covers.
///
/// A mark that straddles the split is **cut in two** rather than dropped or given wholesale to one side:
/// `**bold text**` split in the middle is bold on both halves, which is what a reader who then types on
/// either side expects.
fn split_text(text: &Text, head_end: usize, tail_from: usize) -> (Text, Text) {
    let at = head_end;
    let head = text.text[..head_end].to_string();
    let tail = text.text[tail_from.min(text.text.len())..].to_string();
    let mut before = Vec::new();
    let mut after = Vec::new();
    for span in &text.marks {
        let start = span.range.start.min(at);
        let end = span.range.end.min(at);
        if start < end {
            before.push(MarkSpan {
                range: start..end,
                mark: span.mark,
            });
        }
        let start = span.range.start.max(tail_from) - tail_from;
        let end = span.range.end.max(tail_from) - tail_from;
        if start < end {
            after.push(MarkSpan {
                range: start..end,
                mark: span.mark,
            });
        }
    }
    let mut head_text = Text {
        text: head,
        marks: before,
    };
    head_text.normalize();
    let mut tail_text = Text {
        text: tail,
        marks: after,
    };
    tail_text.normalize();
    (head_text, tail_text)
}

/// Delete the selection, or one character, or merge with the previous block.
fn backspace(doc: &mut Doc, sel: &mut Selection) {
    let index = sel.block;
    let block = doc.blocks[index].clone();
    let text = text_of(&block);

    // A range: delete it, whatever the block is.
    if !sel.is_caret() {
        let start = clamp_to_boundary(&text.text, sel.range.start);
        let end = clamp_to_boundary(&text.text, sel.range.end);
        let mut spliced = text.clone();
        spliced.text.replace_range(start..end, "");
        shift_marks(&mut spliced, start, end, 0);
        if matches!(block.kind, BlockKind::Divider) {
            // A divider has no text, so a selection across it can only mean the block itself.
            doc.blocks.remove(index);
            *sel = Selection::caret(index.saturating_sub(1), 0);
            return;
        }
        set_text(&mut doc.blocks[index], spliced);
        *sel = Selection::caret(index, start);
        return;
    }

    let at = clamp_to_boundary(&text.text, sel.range.start);

    // A divider is deleted as a block: there is no character to eat.
    if matches!(block.kind, BlockKind::Divider) {
        if doc.blocks.len() > 1 {
            doc.blocks.remove(index);
            let new_index = index.min(doc.blocks.len() - 1);
            *sel = Selection::caret(new_index, 0);
        }
        return;
    }

    // **At the start of an indented item, outdent instead of merging.** A reader pressing Backspace at the
    // start of a nested bullet means "bring this out a level", and merging it into the block above is a
    // much bigger edit than they asked for.
    if at == 0 && block.indent > 0 {
        doc.blocks[index].indent = block.indent - 1;
        *sel = Selection::caret(index, 0);
        return;
    }

    // At the start of the document there is nothing to merge into.
    if at == 0 {
        if index == 0 {
            return;
        }
        // Merge into the previous block, and put the caret at the join.
        let previous = doc.blocks[index - 1].clone();
        // A divider cannot hold the merged text, so it gains a paragraph instead — the block below moves
        // up as its own block rather than vanishing into the rule.
        if matches!(previous.kind, BlockKind::Divider) {
            return;
        }
        let mut previous_text = text_of(&previous);
        let join = previous_text.text.len();
        previous_text.text.push_str(&text.text);
        for span in &text.marks {
            previous_text.marks.push(MarkSpan {
                range: span.range.start + join..span.range.end + join,
                mark: span.mark,
            });
        }
        previous_text.normalize();
        set_text(&mut doc.blocks[index - 1], previous_text);
        doc.blocks.remove(index);
        *sel = Selection::caret(index - 1, join);
        return;
    }

    // Otherwise eat one character backwards.
    let start = previous_boundary(&text.text, at);
    let mut spliced = text.clone();
    spliced.text.replace_range(start..at, "");
    shift_marks(&mut spliced, start, at, 0);
    set_text(&mut doc.blocks[index], spliced);
    // The caret moves back by the number of **characters** removed, which is what a caller that tracks a
    // visual column needs; the byte offset is `start`.
    let _ = start;
    *sel = Selection::caret(index, start);
}

/// Delete the selection, or the character after the caret.
fn delete(doc: &mut Doc, sel: &mut Selection) {
    let index = sel.block;
    let block = doc.blocks[index].clone();
    let text = text_of(&block);
    if !sel.is_caret() {
        backspace(doc, sel);
        return;
    }
    let at = clamp_to_boundary(&text.text, sel.range.start);
    if at >= text.text.len() {
        // At the end: merge the **next** block up into this one, which is the mirror of Backspace.
        if index + 1 >= doc.blocks.len() {
            return;
        }
        let next = doc.blocks[index + 1].clone();
        if matches!(next.kind, BlockKind::Divider) {
            doc.blocks.remove(index + 1);
            return;
        }
        let mut merged = text.clone();
        let next_text = text_of(&next);
        let join = merged.text.len();
        merged.text.push_str(&next_text.text);
        for span in &next_text.marks {
            merged.marks.push(MarkSpan {
                range: span.range.start + join..span.range.end + join,
                mark: span.mark,
            });
        }
        merged.normalize();
        set_text(&mut doc.blocks[index], merged);
        doc.blocks.remove(index + 1);
        return;
    }
    let end = next_boundary(&text.text, at);
    let mut spliced = text.clone();
    spliced.text.replace_range(at..end, "");
    shift_marks(&mut spliced, at, end, 0);
    set_text(&mut doc.blocks[index], spliced);
}

/// Indent or outdent a block, **bounded by the invariant**.
fn indent(doc: &mut Doc, sel: &mut Selection, delta: i32) {
    let index = sel.block;
    let indent = doc.blocks[index].indent;
    let wanted = (indent as i32 + delta).max(0) as u8;
    if wanted == indent {
        return;
    }
    // **Tab may not indent past `previous + 1`.** The serializer relies on the invariant, so an indent that
    // broke it would produce a document that cannot be written back faithfully — and the key that breaks it
    // is the one a reader presses most.
    if delta > 0 {
        let ceiling = match index {
            0 => 0,
            _ => doc.blocks[index - 1].indent.saturating_add(1),
        };
        if wanted > ceiling {
            return;
        }
    }
    doc.blocks[index].indent = wanted;
    // **The children move with their parent**, or outdenting would orphan them into the invariant's
    // forbidden jump; and indenting without them would leave a child less deep than its parent.
    let mut child = index + 1;
    while child < doc.blocks.len() && doc.blocks[child].indent > indent {
        let adjusted = (doc.blocks[child].indent as i32 + delta).max(0) as u8;
        doc.blocks[child].indent = adjusted.min(doc.blocks[child - 1].indent.saturating_add(1));
        child += 1;
    }
    let _ = sel;
}

/// Turn a mark on or off over the selection.
fn toggle_mark(doc: &mut Doc, sel: &Selection, mark: Mark) {
    // A caret toggles **nothing**: a mark covers text, and an empty range has none. What a caret-and-no-
    // selection does is a *pending* mark the next character carries, which is state the paint half owns
    // because it is not in the document.
    if sel.is_caret() {
        return;
    }
    let index = sel.block;
    let Some(text) = doc.blocks[index].kind.text().cloned() else {
        return;
    };
    let start = clamp_to_boundary(&text.text, sel.range.start);
    let end = clamp_to_boundary(&text.text, sel.range.end);
    if start >= end {
        return;
    }
    let mut updated = text.clone();
    // Off if the **whole** range already carries it, on otherwise — which is the rule a reader expects from
    // a toggle button: pressing bold inside a bold run does not un-bold half of it.
    let covered = updated.marks.iter().any(|span| span.mark == mark);
    if covered {
        updated
            .marks
            .retain(|span| !(span.mark == mark && span.range.start >= start && span.range.end <= end));
        // A mark that covered *more* than the selection is split around it.
        let mut split = Vec::new();
        for span in updated.marks.drain(..) {
            if span.mark == mark && span.range.start < start && span.range.end > end {
                split.push(MarkSpan {
                    range: span.range.start..start,
                    mark,
                });
                split.push(MarkSpan {
                    range: end..span.range.end,
                    mark,
                });
            } else {
                split.push(span);
            }
        }
        updated.marks = split;
    } else {
        updated.marks.push(MarkSpan {
            range: start..end,
            mark,
        });
    }
    updated.normalize();
    set_text(&mut doc.blocks[index], updated);
}

/// Turn a block into another kind.
fn set_kind(doc: &mut Doc, sel: &mut Selection, kind: SetKind) {
    let index = sel.block;
    let text = text_of(&doc.blocks[index]);
    if kind == SetKind::Divider {
        doc.blocks[index].kind = BlockKind::Divider;
        *sel = Selection::caret(index, 0);
        return;
    }
    if matches!(doc.blocks[index].kind, BlockKind::Divider) {
        // A divider has no text, so it becomes an **empty** block rather than gaining the paragraph next
        // to it.
        doc.blocks[index].kind = kind.kind(Text::plain(""));
        *sel = Selection::caret(index, 0);
        return;
    }
    let offset = clamp_to_boundary(&text.text, sel.range.end);
    // A block becoming a fence keeps its text as code; nothing else changes the text.
    doc.blocks[index].kind = match kind {
        SetKind::Code => BlockKind::Code {
            language: None,
            code: text,
        },
        other => other.kind(text),
    };
    *sel = Selection::caret(index, offset);
}

/// Move the marks after a byte range that was replaced by `inserted` bytes of text.
fn shift_marks(text: &mut Text, start: usize, end: usize, inserted: usize) {
    let removed = end.saturating_sub(start);
    let mut marks = Vec::new();
    for span in text.marks.drain(..) {
        if span.range.end <= start {
            marks.push(span);
            continue;
        }
        if span.range.start >= end {
            let shift = inserted as isize - removed as isize;
            let start_shifted = (span.range.start as isize + shift).max(start as isize) as usize;
            let end_shifted = (span.range.end as isize + shift).max(start_shifted as isize) as usize;
            marks.push(MarkSpan {
                range: start_shifted..end_shifted,
                mark: span.mark,
            });
            continue;
        }
        // Straddles the edit: keep the part before, and the part after shifted.
        if span.range.start < start {
            marks.push(MarkSpan {
                range: span.range.start..start,
                mark: span.mark,
            });
        }
        if span.range.end > end {
            let start_shifted = start + inserted;
            let end_shifted = span.range.end - end + start_shifted;
            marks.push(MarkSpan {
                range: start_shifted..end_shifted,
                mark: span.mark,
            });
        }
    }
    text.marks = marks;
    text.normalize();
}

/// Move an offset back past any spaces that precede it.
fn trim_spaces_before(text: &str, at: usize) -> usize {
    let mut index = at;
    while index > 0 && text[..index].ends_with(' ') {
        index -= 1;
    }
    index
}

/// Move an offset past any spaces that follow it.
///
/// **A block may not begin with a space**, because `parse` trims every line — so a split that left one at the
/// start of the new block would produce a document whose wire form changes on the next read: `para` then
/// ` here` writes as `para\n\n here`, which reads back as `para` then `here`. Moving the spaces to the end of
/// the first block is also what a reader means: pressing Enter before a space should not leave the space
/// dangling at the start of the next line.
fn skip_spaces_after(text: &str, at: usize) -> usize {
    let mut index = at;
    while index < text.len() && text[index..].starts_with(' ') {
        index += 1;
    }
    index
}

/// Round a byte offset down to a character boundary.
fn clamp_to_boundary(text: &str, offset: usize) -> usize {
    let mut at = offset.min(text.len());
    while at > 0 && !text.is_char_boundary(at) {
        at -= 1;
    }
    at
}

/// The boundary before `at`.
fn previous_boundary(text: &str, at: usize) -> usize {
    let mut prev = at.saturating_sub(1);
    while prev > 0 && !text.is_char_boundary(prev) {
        prev -= 1;
    }
    prev
}

/// The boundary after `at`.
fn next_boundary(text: &str, at: usize) -> usize {
    let mut next = (at + 1).min(text.len());
    while next < text.len() && !text.is_char_boundary(next) {
        next += 1;
    }
    next
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse, serialize};

    fn doc(source: &str) -> Doc {
        parse(source)
    }

    /// Apply a shortcut and unwrap, for the tests where the operation must do something.
    fn edit(source: &str, selection: Selection, shortcut: Shortcut) -> Edited {
        apply(&doc(source), &selection, shortcut).expect("the shortcut applied")
    }

    /// The document's text, one string per block, for an assertion that reads as the document does.
    fn blocks(doc: &Doc) -> Vec<String> {
        doc.blocks
            .iter()
            .map(|block| {
                block
                    .kind
                    .text()
                    .map(|text| text.text.clone())
                    .unwrap_or_else(|| "---".to_string())
            })
            .collect()
    }

    fn kinds(doc: &Doc) -> Vec<&'static str> {
        doc.blocks
            .iter()
            .map(|block| match &block.kind {
                BlockKind::Paragraph(_) => "p",
                BlockKind::Heading { level, .. } => match level {
                    1 => "h1",
                    2 => "h2",
                    _ => "hN",
                },
                BlockKind::Bullet(_) => "ul",
                BlockKind::Ordered { .. } => "ol",
                BlockKind::Task { .. } => "task",
                BlockKind::Quote(_) => "quote",
                BlockKind::Code { .. } => "code",
                BlockKind::Divider => "hr",
            })
            .collect()
    }

    // ---- Enter ------------------------------------------------------------

    #[test]
    fn test_enter_splits_a_paragraph_at_the_caret() {
        let out = edit(
            "hello world\n",
            Selection::caret(0, 6),
            Shortcut::Enter,
        );
        // **The space is dropped, not kept on the first half.** `parse` trims every line, so a block ending
        // with a space is not representable — the first version of this expectation kept it and the wire form
        // then drifted on the next read.
        assert_eq!(blocks(&out.doc), vec!["hello", "world"]);
        assert_eq!(kinds(&out.doc), vec!["p", "p"]);
        // The caret is at the start of the new block.
        assert_eq!(out.selection, Selection::caret(1, 0));
    }

    #[test]
    fn test_enter_at_the_end_of_a_heading_gives_a_paragraph() {
        // **The rule a reader feels most.** Enter at the end of a heading is how you stop writing a heading;
        // producing another heading is the fault that makes an editor feel broken.
        let out = edit("# Title\n", Selection::caret(0, 7), Shortcut::Enter);
        assert_eq!(kinds(&out.doc), vec!["h1", "p"]);
        assert_eq!(blocks(&out.doc), vec!["Title", ""]);
    }

    #[test]
    fn test_enter_in_a_list_continues_the_list() {
        let out = edit("- one\n", Selection::caret(0, 5), Shortcut::Enter);
        assert_eq!(kinds(&out.doc), vec!["ul", "ul"]);
        let out = edit("- [x] done\n", Selection::caret(0, 10), Shortcut::Enter);
        // A new task is **unchecked**, because a checked one would be a claim the reader did not make.
        assert_eq!(kinds(&out.doc), vec!["task", "task"]);
        match &out.doc.blocks[1].kind {
            BlockKind::Task { checked, .. } => assert!(!*checked),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn test_enter_in_an_ordered_list_numbers_the_new_item_next() {
        let out = edit("3. three\n", Selection::caret(0, 8), Shortcut::Enter);
        assert_eq!(kinds(&out.doc), vec!["ol", "ol"]);
        let numbers: Vec<u64> = out
            .doc
            .blocks
            .iter()
            .filter_map(|block| match &block.kind {
                BlockKind::Ordered { number, .. } => Some(*number),
                _ => None,
            })
            .collect();
        assert_eq!(numbers, vec![3, 4]);
    }

    #[test]
    fn test_enter_on_an_empty_list_item_leaves_the_list() {
        // **Without this rule there is no way out of a list** except deleting its marker, and a list is the
        // block kind a document has most of.
        // **Built by editing, because `parse` cannot produce an empty bullet.** `"- "` trims to `"-"`, so it
        // parses as a paragraph whose text is a minus — the first version of this test asserted against a
        // state the parser never makes. Pressing Enter at the end of a real bullet is how an empty one
        // happens, which is exactly the state the rule is for.
        let bullet = edit("- a\n", Selection::caret(0, 3), Shortcut::Enter);
        assert_eq!(kinds(&bullet.doc), vec!["ul", "ul"]);
        assert_eq!(blocks(&bullet.doc), vec!["a", ""]);
        let out = apply(&bullet.doc, &bullet.selection, Shortcut::Enter).expect("the second Enter");
        assert_eq!(kinds(&out.doc), vec!["ul", "p"]);
        // And the original expectation, for the case the parser *can* make: an indented empty item.
        assert_eq!(blocks(&out.doc), vec!["a", ""]);
        // Indented, the first Enter outdents and a second leaves — built by indenting the empty bullet,
        // because a source line of `  - ` trims to `  -` and parses as a paragraph.
        let indented = apply(&bullet.doc, &Selection::caret(1, 0), Shortcut::Indent)
            .expect("the empty bullet indented");
        assert_eq!(indented.doc.blocks[1].indent, 1);
        let first = apply(&indented.doc, &indented.selection, Shortcut::Enter)
            .expect("the first Enter on the indented empty item");
        assert_eq!(kinds(&first.doc), vec!["ul", "ul"]);
        assert_eq!(first.doc.blocks[1].indent, 0, "it did not outdent");
        let second = apply(&first.doc, &first.selection, Shortcut::Enter)
            .expect("the second Enter");
        assert_eq!(kinds(&second.doc), vec!["ul", "p"]);
    }

    #[test]
    fn test_enter_on_an_empty_quote_gives_a_paragraph() {
        let out = edit(">\n", Selection::caret(0, 0), Shortcut::Enter);
        assert_eq!(kinds(&out.doc), vec!["p"]);
    }

    #[test]
    fn test_enter_on_a_divider_inserts_a_paragraph_below_it() {
        // A divider has no text to split, so the only thing a caret under a rule can mean is "a new block
        // here".
        let out = edit("---\n", Selection::caret(0, 0), Shortcut::Enter);
        assert_eq!(kinds(&out.doc), vec!["hr", "p"]);
        assert_eq!(out.selection, Selection::caret(1, 0));
    }

    #[test]
    fn test_a_split_cuts_a_mark_in_two_rather_than_dropping_it() {
        // `**bold text**` split in the middle is bold on **both** halves, which is what a reader who then
        // types on either side expects. Dropping it from one half is a silent loss of formatting.
        let source = "**bold**\n";
        let parsed = doc(source);
        assert_eq!(parsed.blocks[0].kind.text().unwrap().marks.len(), 1);
        // **Offset 2 in the *text*, not 4 in the source.** The parsed text is `bold` with a bold mark over
        // 0..4, so splitting at the source offset 4 would split at the text's end and put nothing on the
        // second half — the library was right and the offset was mine.
        let out = edit(source, Selection::caret(0, 2), Shortcut::Enter);
        for index in 0..2 {
            let text = out.doc.blocks[index].kind.text().expect("text");
            assert!(
                !text.marks.is_empty(),
                "block {index} lost its mark: {text:?}"
            );
        }
        assert_eq!(blocks(&out.doc), vec!["bo", "ld"]);
    }

    // ---- Backspace and Delete ---------------------------------------------

    #[test]
    fn test_backspace_eats_one_character() {
        let out = edit("hello\n", Selection::caret(0, 3), Shortcut::Backspace);
        assert_eq!(blocks(&out.doc), vec!["helo"]);
        assert_eq!(out.selection, Selection::caret(0, 2));
    }

    #[test]
    fn test_backspace_at_the_start_merges_into_the_previous_block() {
        let out = edit("one\n\ntwo\n", Selection::caret(1, 0), Shortcut::Backspace);
        assert_eq!(blocks(&out.doc), vec!["onetwo"]);
        assert_eq!(out.doc.blocks.len(), 1);
        // The caret is at the join.
        assert_eq!(out.selection, Selection::caret(0, 3));
    }

    #[test]
    fn test_backspace_at_the_start_of_an_indented_item_outdents_first() {
        // A reader pressing Backspace at the start of a nested bullet means "bring this out a level", and
        // merging it into the block above is a much bigger edit than they asked for.
        let out = edit("- outer\n  - inner\n", Selection::caret(1, 0), Shortcut::Backspace);
        assert_eq!(out.doc.blocks.len(), 2, "it merged instead of outdenting");
        assert_eq!(out.doc.blocks[1].indent, 0);
    }

    #[test]
    fn test_backspace_at_the_very_start_does_nothing() {
        // `None` is what lets the paint half leave the key unhandled rather than swallowing it.
        assert_eq!(
            apply(&doc("hello\n"), &Selection::caret(0, 0), Shortcut::Backspace),
            None
        );
    }

    #[test]
    fn test_backspace_with_a_selection_deletes_the_selection() {
        let out = edit(
            "hello world\n",
            Selection {
                block: 0,
                range: 5..11,
            },
            Shortcut::Backspace,
        );
        assert_eq!(blocks(&out.doc), vec!["hello"]);
        assert_eq!(out.selection, Selection::caret(0, 5));
    }

    #[test]
    fn test_backspace_on_a_divider_deletes_the_divider() {
        let out = edit("a\n\n---\n\nb\n", Selection::caret(1, 0), Shortcut::Backspace);
        assert_eq!(kinds(&out.doc), vec!["p", "p"]);
    }

    #[test]
    fn test_delete_removes_the_character_after_the_caret() {
        let out = edit("hello\n", Selection::caret(0, 1), Shortcut::Delete);
        assert_eq!(blocks(&out.doc), vec!["hllo"]);
        // The caret does not move, because the character ahead of it went.
        assert_eq!(out.selection, Selection::caret(0, 1));
    }

    #[test]
    fn test_delete_at_the_end_merges_the_next_block_up() {
        let out = edit("one\n\ntwo\n", Selection::caret(0, 3), Shortcut::Delete);
        assert_eq!(blocks(&out.doc), vec!["onetwo"]);
        assert_eq!(out.selection, Selection::caret(0, 3));
    }

    #[test]
    fn test_editing_a_multibyte_character_removes_the_whole_character() {
        // A byte-wise Backspace would leave half a character, which is not a string at all. This is the test
        // that catches it.
        // The caret is after `é`, which is byte **3** — `h` is byte 0 and `é` occupies 1 and 2. The first
        // version used 2, which is inside `é`, and the library correctly clamped it to the character's start
        // and removed the `h` before it.
        let out = edit("héllo\n", Selection::caret(0, 3), Shortcut::Backspace);
        assert_eq!(blocks(&out.doc), vec!["hllo"]);
        assert_eq!(out.selection, Selection::caret(0, 1), "the caret moved two bytes");
        // Full-width characters too.
        // Offset 6 is the start of `語`, so Backspace removes the character **before** the caret — `本`. The
        // first version expected `日本`, which is removing the character *at* the caret.
        let out = edit("日本語\n", Selection::caret(0, 6), Shortcut::Backspace);
        assert_eq!(blocks(&out.doc), vec!["日語"]);
    }

    // ---- Indent -----------------------------------------------------------

    #[test]
    fn test_tab_indents_and_shift_tab_outdents() {
        let out = edit("- a\n- b\n", Selection::caret(1, 0), Shortcut::Indent);
        assert_eq!(out.doc.blocks[1].indent, 1);
        let back = apply(&out.doc, &out.selection, Shortcut::Outdent).expect("outdented");
        assert_eq!(back.doc.blocks[1].indent, 0);
    }

    #[test]
    fn test_tab_may_not_indent_past_the_invariant() {
        // **The key that would break the serializer.** A block may be at most one level deeper than the one
        // before it, and Tab is the key a reader presses most — so it returns `None` rather than producing a
        // document that cannot be written back.
        let source = "- a\n- b\n";
        let out = apply(&doc(source), &Selection::caret(0, 0), Shortcut::Indent);
        assert_eq!(out, None, "the first block was indented past the invariant");
        // The second may go one deeper than the first, and no deeper.
        let first = edit(source, Selection::caret(1, 0), Shortcut::Indent);
        assert_eq!(first.doc.blocks[1].indent, 1);
        assert_eq!(
            apply(&first.doc, &first.selection, Shortcut::Indent),
            None,
            "it indented two levels past its parent"
        );
    }

    #[test]
    fn test_indenting_carries_the_children_with_it() {
        // Otherwise a child ends up less deep than its parent, which is the invariant's other direction.
        // Block 1 may move from 0 to 1, because its predecessor is at 0 — and its children follow it from
        // 1 and 2 to 2 and 3. The first version of this test tried to indent a block that was *already* one
        // deeper than its predecessor, which the invariant refuses, so the shortcut returned `None` and the
        // test failed on `expect`.
        let source = "- a\n- b\n  - c\n    - d\n";
        let out = edit(source, Selection::caret(1, 0), Shortcut::Indent);
        assert_eq!(out.doc.blocks[1].indent, 1);
        assert_eq!(out.doc.blocks[2].indent, 2, "the child did not follow");
        assert_eq!(out.doc.blocks[3].indent, 3, "the grandchild did not follow");
        assert!(out.doc.is_well_formed());
    }

    #[test]
    fn test_outdenting_carries_the_children_and_never_orphans_them() {
        let source = "- a\n  - b\n    - c\n";
        let out = edit(source, Selection::caret(1, 0), Shortcut::Outdent);
        assert_eq!(out.doc.blocks[1].indent, 0);
        // The grandchild follows, bounded by the invariant.
        assert!(out.doc.is_well_formed(), "{:?}", out.doc);
    }

    // ---- Marks ------------------------------------------------------------

    #[test]
    fn test_toggling_a_mark_on_adds_it_to_the_range() {
        let out = edit(
            "hello\n",
            Selection {
                block: 0,
                range: 0..5,
            },
            Shortcut::ToggleMark(Mark::Bold),
        );
        let text = out.doc.blocks[0].kind.text().expect("text");
        assert_eq!(text.marks.len(), 1);
        assert_eq!(text.marks[0].range, 0..5);
        assert_eq!(text.marks[0].mark, Mark::Bold);
    }

    #[test]
    fn test_toggling_a_mark_off_removes_it_and_nothing_else() {
        let out = edit(
            "**bold**\n",
            Selection {
                block: 0,
                range: 0..4,
            },
            Shortcut::ToggleMark(Mark::Bold),
        );
        let text = out.doc.blocks[0].kind.text().expect("text");
        assert!(text.marks.is_empty(), "{:?}", text.marks);
        assert_eq!(text.text, "bold");
    }

    #[test]
    fn test_a_caret_toggles_nothing() {
        // A mark covers text and an empty range has none. What a caret does is set a *pending* mark the next
        // character carries, which is state the paint half owns because it is not in the document.
        assert_eq!(
            apply(
                &doc("hello\n"),
                &Selection::caret(0, 2),
                Shortcut::ToggleMark(Mark::Bold)
            ),
            None
        );
    }

    #[test]
    fn test_a_mark_covering_more_than_the_selection_is_split_around_it() {
        // Toggling off the middle of a bold run must not un-bold the ends, which is what a reader sees as
        // "it removed more than I selected".
        let out = edit(
            "**abcdef**\n",
            Selection {
                block: 0,
                range: 2..4,
            },
            Shortcut::ToggleMark(Mark::Bold),
        );
        let text = out.doc.blocks[0].kind.text().expect("text");
        assert_eq!(text.text, "abcdef");
        assert_eq!(text.marks.len(), 2, "{:?}", text.marks);
        assert_eq!(text.marks[0].range, 0..2);
        assert_eq!(text.marks[1].range, 4..6);
    }

    // ---- SetKind ----------------------------------------------------------

    #[test]
    fn test_setting_the_kind_keeps_the_text() {
        let out = edit("title\n", Selection::caret(0, 5), Shortcut::SetKind(SetKind::Heading1));
        assert_eq!(kinds(&out.doc), vec!["h1"]);
        assert_eq!(blocks(&out.doc), vec!["title"]);
        // And back again.
        let back = apply(
            &out.doc,
            &out.selection,
            Shortcut::SetKind(SetKind::Paragraph),
        )
        .expect("set back");
        assert_eq!(kinds(&back.doc), vec!["p"]);
        assert_eq!(blocks(&back.doc), vec!["title"]);
    }

    #[test]
    fn test_becoming_a_divider_loses_the_text_and_becoming_a_paragraph_restores_an_empty_one() {
        // A divider has no text, so the round trip through one cannot keep it — and the honest result is an
        // empty block rather than a silently retained string nothing can show.
        let out = edit("gone\n", Selection::caret(0, 4), Shortcut::SetKind(SetKind::Divider));
        assert_eq!(kinds(&out.doc), vec!["hr"]);
        assert_eq!(out.selection, Selection::caret(0, 0));
        let back = apply(
            &out.doc,
            &out.selection,
            Shortcut::SetKind(SetKind::Paragraph),
        )
        .expect("set back");
        assert_eq!(kinds(&back.doc), vec!["p"]);
        assert_eq!(blocks(&back.doc), vec![""]);
    }

    #[test]
    fn test_becoming_an_ordered_item_starts_at_one_and_renumbers() {
        let out = edit("a\n\nb\n", Selection::caret(1, 1), Shortcut::SetKind(SetKind::Ordered));
        assert_eq!(kinds(&out.doc), vec!["p", "ol"]);
        let numbers: Vec<u64> = out
            .doc
            .blocks
            .iter()
            .filter_map(|block| match &block.kind {
                BlockKind::Ordered { number, .. } => Some(*number),
                _ => None,
            })
            .collect();
        assert_eq!(numbers, vec![1]);
    }

    // ---- The property ------------------------------------------------------

    #[test]
    fn test_every_shortcut_on_every_block_kind_is_safe() {
        // The exhaustive sweep: every shortcut applied at every offset of every kind, with the invariant
        // asserted after each. A single key is easy to get right; it is combinations that break the
        // invariant, and this is the cheap half of that search.
        let source = "# h\n\npara here\n\n- bullet\n  - nested\n\n1. one\n\n- [ ] task\n\n> quote\n\n```\ncode\n```\n\n---\n";
        let base = doc(source);
        let shortcuts = [
            Shortcut::Enter,
            Shortcut::Backspace,
            Shortcut::Delete,
            Shortcut::Indent,
            Shortcut::Outdent,
            Shortcut::ToggleMark(Mark::Bold),
            Shortcut::SetKind(SetKind::Bullet),
            Shortcut::SetKind(SetKind::Paragraph),
            Shortcut::SetKind(SetKind::Heading2),
            Shortcut::SetKind(SetKind::Ordered),
            Shortcut::SetKind(SetKind::Code),
            Shortcut::SetKind(SetKind::Divider),
        ];
        let mut applied = 0usize;
        for block in 0..base.blocks.len() {
            let len = base.blocks[block]
                .kind
                .text()
                .map(|text| text.text.len())
                .unwrap_or(0);
            for offset in 0..=len {
                // Only character boundaries, since that is the only offset a caret can be at.
                let text = base.blocks[block].kind.text().map(|t| t.text.clone());
                if let Some(text) = &text {
                    if !text.is_char_boundary(offset) {
                        continue;
                    }
                }
                for shortcut in shortcuts {
                    let selection = Selection {
                        block,
                        range: offset..offset,
                    };
                    if let Some(out) = apply(&base, &selection, shortcut) {
                        assert!(
                            out.doc.is_well_formed(),
                            "{shortcut:?} at block {block} offset {offset} broke the invariant: {:?}",
                            out.doc
                        );
                        // And the result still **writes stably**, which is what an editor needs: saving,
                        // loading and saving again must not drift.
                        //
                        // The stronger `parse(serialize(doc)) == doc` does **not** hold for an edited
                        // document, and cannot: markdown has no spelling for an *empty paragraph*, so a
                        // document holding one is absorbed by the blank line that already separates
                        // blocks. What cannot change is the wire form, and that is what is asserted.
                        let written = serialize(&out.doc);
                        let rewritten = serialize(&parse(&written));
                        assert_eq!(
                            rewritten, written,
                            "{shortcut:?} at block {block} offset {offset} produced a wire form that \
                             drifts on a second save: {:?}",
                            out.doc
                        );
                        applied += 1;
                    }
                }
            }
        }
        assert!(applied > 200, "only {applied} edits were checked");
    }

    #[test]
    fn test_a_long_random_session_never_breaks_the_invariant() {
        // **The combination search.** The way to break the invariant is Enter then Tab then Backspace, not
        // any single key — so this walks a few thousand random shortcuts, asserting after every one that the
        // document is well formed, still round trips through the wire form, and still has a valid selection.
        let mut state = Edited {
            doc: doc("# Title\n\nA paragraph with **bold** in it.\n\n- one\n  - nested\n\n> quote\n\n```\ncode\n```\n"),
            selection: Selection::caret(0, 0),
        };
        let shortcuts = [
            Shortcut::Enter,
            Shortcut::Backspace,
            Shortcut::Delete,
            Shortcut::Indent,
            Shortcut::Outdent,
            Shortcut::ToggleMark(Mark::Bold),
            Shortcut::ToggleMark(Mark::Italic),
            Shortcut::SetKind(SetKind::Bullet),
            Shortcut::SetKind(SetKind::Ordered),
            Shortcut::SetKind(SetKind::Task),
            Shortcut::SetKind(SetKind::Quote),
            Shortcut::SetKind(SetKind::Heading1),
            Shortcut::SetKind(SetKind::Paragraph),
            Shortcut::SetKind(SetKind::Code),
            Shortcut::SetKind(SetKind::Divider),
        ];
        // A deterministic pseudo-random source, so a failure is reproducible.
        let mut seed = 0x5eed_2001u64;
        let mut next = || {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            seed >> 33
        };
        let mut applied = 0usize;
        for step in 0..4000usize {
            if state.doc.blocks.is_empty() {
                state = Edited {
                    doc: doc("\n"),
                    selection: Selection::caret(0, 0),
                };
            }
            let block = (next() as usize) % state.doc.blocks.len();
            let len = state.doc.blocks[block]
                .kind
                .text()
                .map(|text| text.text.len())
                .unwrap_or(0);
            let mut offset = (next() as usize) % (len + 1);
            if let Some(text) = state.doc.blocks[block].kind.text() {
                while offset > 0 && !text.text.is_char_boundary(offset) {
                    offset -= 1;
                }
            }
            let shortcut = shortcuts[(next() as usize) % shortcuts.len()];
            let selection = Selection {
                block,
                range: offset..offset,
            };
            if let Some(out) = apply(&state.doc, &selection, shortcut) {
                state = out;
                applied += 1;
            }
            assert!(
                state.doc.is_well_formed(),
                "step {step} ({shortcut:?}) broke the invariant: {:?}",
                state.doc
            );
            assert!(
                !state.doc.blocks.is_empty(),
                "step {step} ({shortcut:?}) emptied the document"
            );
            assert!(
                state.selection.block < state.doc.blocks.len(),
                "step {step} ({shortcut:?}) left the caret past the last block"
            );
            // The editor's guarantee: the **wire form** is stable across a save/load/save, which is the
            // strongest thing true of an edited document. See the exhaustive sweep above on why the
            // document itself cannot be.
            let written = serialize(&state.doc);
            assert_eq!(
                serialize(&parse(&written)),
                written,
                "step {step} ({shortcut:?}) produced a wire form that drifts on a second save: {:?}",
                state.doc
            );
        }
        assert!(applied > 1000, "only {applied} edits were applied in 4000 steps");
    }

    #[test]
    fn test_a_document_is_never_emptied_by_an_edit() {
        // A document with no blocks has no caret and nothing to write, so no shortcut may remove the last
        // block. Backspace on the only block is the one that could, and it returns `None` instead.
        assert_eq!(
            apply(&doc("only\n"), &Selection::caret(0, 0), Shortcut::Backspace),
            None
        );
        // A divider is the other candidate, since deleting it removes a whole block.
        let single = doc("---\n");
        assert_eq!(
            apply(&single, &Selection::caret(0, 0), Shortcut::Backspace),
            None
        );
    }
}
