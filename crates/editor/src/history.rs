//! Undo and redo over a document, and **what counts as one step**.
//!
//! ## Why this is a separate thing from the stack
//!
//! [`SnapshotHistory`] is a `Vec` of states and a cursor, and it says so in its own doc: it cannot know what a
//! state *is*, and it cannot know when two edits should become one undo step — **"only the caller knows what
//! an edit means"**. This is that caller, and the whole of its job is the policy:
//!
//! | a run of | is |
//! |---|---|
//! | typing | **one** step, however many characters |
//! | deleting | one step |
//! | a structural change — Enter, Tab, a kind change, a merge | **its own** step, always |
//! | a caret move | the **end** of the run before it |
//!
//! The line between the first and third rows is the one a reader feels. Typing ten characters then pressing
//! undo should remove the ten characters, not the tenth; pressing Enter between them should make the
//! characters before it one step and the split another, because **the split is a place they can see**.
//!
//! ## The stack holds **states**, and the base is explicit
//!
//! [`History::new`] takes the document an editor has open, so the state before the first edit is in the stack
//! and an undo can reach it. [`History::record`] is then called **after** each edit with the state that edit
//! produced, and a run of the same coalescing kind **overwrites its own entry** rather than adding one — which
//! is what makes ten keystrokes a single step whose snapshot is the state after the tenth.
//!
//! The first version of this recorded the state *before* each edit instead, and it was wrong in a way worth
//! keeping: the stack's current entry was then the state before the most recent **run**, so one undo skipped a
//! run — typing `ab`, pausing, typing `cd` and pressing undo gave the empty document rather than `ab`. Two of
//! the tests below failed on exactly that, and the fault was the model rather than the arithmetic.
//!
//! ## The limit is deeper than it looks
//!
//! [`DEFAULT_UNDO_LIMIT`] is a hundred **steps**, not a hundred edits: a run of typing is one step however
//! long it is, so the reader's reach into the past is a hundred *pauses*. That is the number bezel uses, and
//! the reason it is a step count rather than an edit count is the coalescing.

use makepad_markdown::{Doc, Selection};

use crate::snapshot::SnapshotHistory;

/// How many undo steps a document keeps.
///
/// A hundred *steps*: see the module doc on why that is deeper than a hundred edits.
pub const DEFAULT_UNDO_LIMIT: usize = 100;

/// What an edit did, so a run of the same kind can become one step.
///
/// The set is small on purpose. A `EditKind` per operation would mean every new operation has to decide its
/// own coalescing, and the decision that matters is only ever "is this the same as what came before".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditKind {
    /// Typed characters. Coalesces with `Typing`.
    Typing,
    /// Deletions. Coalesces with `Deleting` — and **not** with `Typing`, so typing and then backspacing are two
    /// steps. A reader who types a word, deletes it and presses undo expects the word back, not the deletion
    /// undone.
    Deleting,
    /// Formatting a range: a mark, a kind change with a selection.
    ///
    /// Coalesces, because holding a key repeat on a formatting shortcut is one thing the reader did.
    Formatting,
    /// A structural change: Enter, Backspace merging two blocks, Tab, a kind change from the marker.
    ///
    /// **Never coalesces**, because these are the steps a reader wants to land on.
    Structural,
}

impl EditKind {
    /// Whether a run of this kind becomes one undo step.
    pub fn coalesces(self) -> bool {
        !matches!(self, EditKind::Structural)
    }
}

/// One undo step: the document **before** it, and where the caret was.
///
/// The caret is part of it because undoing without moving the caret leaves the reader looking at a different
/// place from the one they edited — which reads as undo having done the wrong thing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub doc: Doc,
    pub selection: Selection,
}

/// The document's undo and redo.
#[derive(Clone, Debug)]
pub struct History {
    steps: SnapshotHistory<Step>,
    /// The kind of the last **recorded** edit, or `None` when a run has ended.
    ///
    /// `None` and not "the kind of the last entry": a caret move or an interrupt ends a run without recording
    /// anything, and that is exactly what makes typing after a caret move a second step.
    last: Option<EditKind>,
}

impl History {
    /// A history for a document an editor has just opened.
    ///
    /// The document is the stack's **base**, which is what lets an undo reach the state before the first edit.
    /// A history with no base cannot, and the first version of this had none — see the module doc.
    pub fn new(doc: Doc, selection: Selection) -> Self {
        Self::with_limit(DEFAULT_UNDO_LIMIT, doc, selection)
    }

    pub fn with_limit(limit: usize, doc: Doc, selection: Selection) -> Self {
        Self {
            steps: SnapshotHistory::with_capacity(Step { doc, selection }, limit.max(1)),
            last: None,
        }
    }

    /// Record the state an edit **produced**, and say what the edit was.
    ///
    /// Called after the edit is applied. A run of the same coalescing kind **overwrites its own entry** rather
    /// than adding one, so the run's snapshot is the state after its last edit and one undo reaches the state
    /// before the run.
    pub fn record(&mut self, kind: EditKind, doc: &Doc, selection: &Selection) {
        let step = Step {
            doc: doc.clone(),
            selection: selection.clone(),
        };
        // `can_undo` as well as the kind: a record after an undo starts a **new** branch, and the entry it
        // would overwrite is the base.
        let extends = kind.coalesces() && self.last == Some(kind) && self.steps.can_undo();
        if extends {
            self.steps.replace_current(step);
        } else {
            self.steps.push(step);
        }
        self.last = Some(kind);
    }

    /// The caret moved, or a run otherwise ended **without** an edit.
    ///
    /// This is what makes "type, click elsewhere, type" two undo steps rather than one — and it is
    /// bezel's `landed`, named for the same reason: a selection landed somewhere, so the next keystroke starts
    /// a new step.
    pub fn landed(&mut self) {
        self.last = None;
    }

    /// End a run for a reason that is not a caret move: focus left, a menu opened, a mode changed.
    pub fn interrupt(&mut self) {
        self.last = None;
    }

    /// Step back, returning the document and caret to restore.
    ///
    /// `None` when there is nothing to undo.
    pub fn undo(&mut self) -> Option<Step> {
        let step = self.steps.undo().cloned()?;
        // **A run cannot survive an undo.** Otherwise the next keystroke would coalesce with the step the
        // reader just undid, and the undo would look like it had not happened.
        self.last = None;
        Some(step)
    }

    /// Step forward, returning the document and caret to restore.
    pub fn redo(&mut self) -> Option<Step> {
        let step = self.steps.redo().cloned()?;
        self.last = None;
        Some(step)
    }

    /// Start again on another document — opening a file is not an edit, and an undo must not take the reader
    /// back into the previous one.
    pub fn reset(&mut self, doc: Doc, selection: Selection) {
        self.steps = SnapshotHistory::with_capacity(
            Step { doc, selection },
            self.steps.capacity().unwrap_or(DEFAULT_UNDO_LIMIT),
        );
        self.last = None;
    }

    pub fn can_undo(&self) -> bool {
        self.steps.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.steps.can_redo()
    }

    /// How many steps back and forward are available, for a pair of buttons.
    pub fn depth(&self) -> (usize, usize) {
        self.steps.depth()
    }

    /// How many steps are retained, the current one included.
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use makepad_markdown::{apply, parse, serialize, Shortcut};

    /// The document as text, for an assertion that reads as the document does.
    fn text(doc: &Doc) -> String {
        serialize(doc).trim_end().to_string()
    }

    /// A document with **one empty paragraph**, which is the state an editor is in when the reader has just
    /// opened a blank one.
    ///
    /// Built rather than parsed, because `parse("\n")` gives a document with **no blocks at all** — a blank line
    /// is a separator, not a block. The first version of these tests used `parse("\n")` and indexed block 0 of
    /// an empty `Vec`.
    fn empty_doc() -> Doc {
        use makepad_markdown::{Block, BlockKind, Text};
        Doc {
            blocks: vec![Block {
                indent: 0,
                kind: BlockKind::Paragraph(Text::plain("")),
            }],
        }
    }

    /// Type a character at the end of a block, recording the step the way an editor does — **after** applying
    /// it, with the state the edit produced.
    fn type_char(history: &mut History, doc: &mut Doc, selection: &mut Selection, ch: char) {
        let block = selection.block;
        if let Some(existing) = doc.blocks[block].kind.text().cloned() {
            let mut updated = existing;
            updated.text.push(ch);
            updated.normalize();
            let end = updated.text.len();
            doc.blocks[block].kind = replace_text(doc.blocks[block].kind.clone(), updated);
            *selection = Selection::caret(block, end);
        }
        history.record(EditKind::Typing, doc, selection);
    }

    /// The same kind carrying other text, for the test's text input.
    fn replace_text(kind: makepad_markdown::BlockKind, text: makepad_markdown::Text) -> makepad_markdown::BlockKind {
        use makepad_markdown::BlockKind;
        match kind {
            BlockKind::Paragraph(_) => BlockKind::Paragraph(text),
            BlockKind::Heading { level, .. } => BlockKind::Heading { level, text },
            BlockKind::Bullet(_) => BlockKind::Bullet(text),
            BlockKind::Ordered { number, .. } => BlockKind::Ordered { number, text },
            BlockKind::Task { checked, .. } => BlockKind::Task { checked, text },
            BlockKind::Quote(_) => BlockKind::Quote(text),
            BlockKind::Code { language, .. } => BlockKind::Code { language, code: text },
            BlockKind::Divider => BlockKind::Divider,
        }
    }

    #[test]
    fn test_a_run_of_typing_is_one_undo_step() {
        // **The rule a reader feels.** Ten characters then undo should remove the ten, not the tenth.
        let mut doc = empty_doc();
        let mut selection = Selection::caret(0, 0);
        let mut history = History::new(doc.clone(), selection.clone());
        let empty = doc.clone();
        for ch in "hello".chars() {
            type_char(&mut history, &mut doc, &mut selection, ch);
        }
        assert_eq!(text(&doc), "hello");
        assert_eq!(history.depth(), (1, 0), "each keystroke became a step");
        let step = history.undo().expect("an undo");
        assert_eq!(text(&step.doc), text(&empty), "undo did not go back to the start of the run");
        assert!(!history.can_undo(), "the run was more than one step");
    }

    #[test]
    fn test_a_caret_move_ends_the_run_so_the_next_keystroke_is_a_new_step() {
        let mut doc = empty_doc();
        let mut selection = Selection::caret(0, 0);
        let mut history = History::new(doc.clone(), selection.clone());
        for ch in "ab".chars() {
            type_char(&mut history, &mut doc, &mut selection, ch);
        }
        assert_eq!(history.depth(), (1, 0));
        // The reader clicks elsewhere: the run ends.
        history.landed();
        for ch in "cd".chars() {
            type_char(&mut history, &mut doc, &mut selection, ch);
        }
        assert_eq!(history.depth(), (2, 0), "the run did not end at the caret move");
        let back = history.undo().expect("an undo");
        assert_eq!(text(&back.doc), "ab", "undo went past the run boundary");
        let back = history.undo().expect("a second undo");
        assert_eq!(text(&back.doc), "");
    }

    #[test]
    fn test_typing_and_then_deleting_are_two_steps() {
        // A reader who types a word, deletes it and presses undo expects the word back — not the deletion
        // undone, which would leave the document as it was after typing.
        let mut doc = parse("word\n");
        let selection = Selection::caret(0, 4);
        let mut history = History::new(doc.clone(), selection.clone());
        history.record(EditKind::Typing, &doc, &selection);
        // A deletion of the whole word, applied with the real operation.
        let edited = apply(&doc, &selection, Shortcut::Backspace).expect("the deletion applied");
        doc = edited.doc;
        history.record(EditKind::Deleting, &doc, &selection);
        // The deletion was a different kind, so it pushed rather than extending the typing step.
        assert_eq!(history.depth(), (2, 0));
    }

    #[test]
    fn test_a_structural_change_is_always_its_own_step() {
        // Enter is a place the reader can see, so it does not coalesce with the typing before it.
        let mut doc = parse("one\n");
        let selection = Selection::caret(0, 3);
        let mut history = History::new(doc.clone(), selection.clone());
        history.record(EditKind::Typing, &doc, &selection);
        let edited = apply(&doc, &selection, Shortcut::Enter).expect("the split applied");
        doc = edited.doc;
        history.record(EditKind::Structural, &doc, &edited.selection);
        let edited = apply(&doc, &edited.selection, Shortcut::Enter).expect("the second split");
        history.record(EditKind::Structural, &doc, &edited.selection);
        // Three steps: the typing, and each Enter.
        assert_eq!(history.depth(), (3, 0));
    }

    #[test]
    fn test_an_interrupt_ends_a_run_without_a_caret_move() {
        // Focus left, a menu opened: the next keystroke is a new step.
        let mut doc = empty_doc();
        let mut selection = Selection::caret(0, 0);
        let mut history = History::new(doc.clone(), selection.clone());
        type_char(&mut history, &mut doc, &mut selection, 'a');
        history.interrupt();
        type_char(&mut history, &mut doc, &mut selection, 'b');
        assert_eq!(history.depth(), (2, 0));
    }

    #[test]
    fn test_a_run_cannot_survive_an_undo() {
        // Otherwise the next keystroke would coalesce with the step the reader just undid, and the undo would
        // look like it had not happened.
        let mut doc = empty_doc();
        let mut selection = Selection::caret(0, 0);
        let mut history = History::new(doc.clone(), selection.clone());
        for ch in "ab".chars() {
            type_char(&mut history, &mut doc, &mut selection, ch);
        }
        assert_eq!(history.depth(), (1, 0));
        history.undo();
        for ch in "cd".chars() {
            type_char(&mut history, &mut doc, &mut selection, ch);
        }
        assert_eq!(
            history.depth(),
            (1, 0),
            "the run continued across an undo, so the undo looks like it did nothing"
        );
    }

    #[test]
    fn test_redo_after_an_undo_steps_forward() {
        let mut doc = empty_doc();
        let mut selection = Selection::caret(0, 0);
        let mut history = History::new(doc.clone(), selection.clone());
        type_char(&mut history, &mut doc, &mut selection, 'a');
        history.landed();
        type_char(&mut history, &mut doc, &mut selection, 'b');
        let after_two = doc.clone();
        history.undo();
        assert_eq!(history.depth(), (1, 1));
        let forward = history.redo().expect("a redo");
        assert_eq!(text(&forward.doc), text(&after_two));
        assert_eq!(history.depth(), (2, 0));
    }

    #[test]
    fn test_recording_after_an_undo_abandons_the_redo_branch() {
        // The stack's rule, checked through the document history: a new edit after an undo is a new future.
        let mut doc = empty_doc();
        let mut selection = Selection::caret(0, 0);
        let mut history = History::new(doc.clone(), selection.clone());
        type_char(&mut history, &mut doc, &mut selection, 'a');
        history.landed();
        type_char(&mut history, &mut doc, &mut selection, 'b');
        history.undo();
        assert!(history.can_redo());
        history.landed();
        type_char(&mut history, &mut doc, &mut selection, 'c');
        assert!(!history.can_redo(), "the abandoned branch is still reachable");
    }

    #[test]
    fn test_the_limit_is_a_limit_on_steps_and_coalescing_makes_it_deeper() {
        // A hundred *steps*: two hundred keystrokes with no pause is one step, so a reader's reach into the
        // past is a hundred pauses.
        let doc = parse("a\n");
        let selection = Selection::caret(0, 0);
        let mut history = History::with_limit(3, doc.clone(), selection.clone());
        for _ in 0..10 {
            history.record(EditKind::Typing, &doc, &selection);
        }
        // `depth`, not `len`: the stack holds the base entry as well, so one run is two entries and **one**
        // undoable step — which is the number a reader cares about and the one the limit counts.
        assert_eq!(history.depth(), (1, 0), "a run of typing took more than one step");
        history.landed();
        for index in 0..10 {
            history.record(EditKind::Structural, &doc, &selection);
            history.landed();
            let _ = index;
        }
        assert!(
            history.len() <= 3,
            "the limit of 3 steps was exceeded: {} retained",
            history.len()
        );
    }

    #[test]
    fn test_a_fresh_history_offers_nothing_in_either_direction() {
        let doc = empty_doc();
        let history = History::new(doc, Selection::caret(0, 0));
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.depth(), (0, 0));
    }

    #[test]
    fn test_reset_forgets_everything_for_a_replaced_document() {
        // Opening a file is not an edit: undo must not take the reader back into the previous document.
        let mut doc = empty_doc();
        let mut selection = Selection::caret(0, 0);
        let mut history = History::new(doc.clone(), selection.clone());
        type_char(&mut history, &mut doc, &mut selection, 'a');
        assert!(history.can_undo());
        history.reset(parse("other\n"), Selection::caret(0, 0));
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_undo_restores_the_caret_as_well_as_the_document() {
        // Undoing without moving the caret leaves the reader looking at a different place from the one they
        // edited, which reads as undo having done the wrong thing.
        let mut doc = parse("hello world\n");
        let mut selection = Selection::caret(0, 5);
        let mut history = History::new(doc.clone(), selection.clone());
        let edited = apply(&doc, &selection, Shortcut::Enter).expect("the split");
        doc = edited.doc;
        history.record(EditKind::Structural, &parse("hello world\n"), &selection);
        let step = history.undo().expect("an undo");
        assert_eq!(step.selection, selection, "the caret did not come back");
        assert_eq!(text(&step.doc), "hello world");
    }

    #[test]
    fn test_a_real_session_undoes_to_where_the_reader_expects() {
        // The session a reader actually has: type a word, press Enter, type another, then undo twice. The
        // first undo removes the second word, the second removes the split — and the first word is still
        // there, which is the whole point of coalescing.
        let mut doc = parse("first\n");
        let mut selection = Selection::caret(0, 5);
        let mut history = History::new(doc.clone(), selection.clone());

        // Type " word" at the end of the first block. **Apply, then record** — the order the model needs, and
        // the inline loops here still had the old order after the helper was converted, which is why this test
        // failed a step later than the others.
        for ch in " word".chars() {
            let mut text = doc.blocks[0].kind.text().cloned().expect("text");
            text.text.push(ch);
            doc.blocks[0].kind = replace_text(doc.blocks[0].kind.clone(), text);
            history.record(EditKind::Typing, &doc, &selection);
        }
        selection = Selection::caret(0, 10);
        history.landed();

        // Enter, which is its own step.
        let edited = apply(&doc, &selection, Shortcut::Enter).expect("the split");
        doc = edited.doc;
        selection = edited.selection;
        // After the edit, with the state it produced.
        history.record(EditKind::Structural, &doc, &selection);

        // Type "second", applied then recorded like the loop above.
        for ch in "second".chars() {
            let mut text = doc.blocks[selection.block].kind.text().cloned().expect("text");
            text.text.push(ch);
            let end = text.text.len();
            doc.blocks[selection.block].kind =
                replace_text(doc.blocks[selection.block].kind.clone(), text);
            selection = Selection::caret(selection.block, end);
            history.record(EditKind::Typing, &doc, &selection);
        }
        // Serialized, the two paragraphs are separated by a blank line — the split made a second block and the
        // typing filled it. My first expectation wrote them adjacent, which is what `parse` does with a *list*,
        // not with two paragraphs.
        assert_eq!(text(&doc), "first word\n\nsecond");

        // Undo once: the second word goes.
        let step = history.undo().expect("the first undo");
        // Back to the state before the typing run: the split is still there, and its second block is **empty**
        // — which serializes away, so the text is one line. The first version of this expected a trailing
        // newline, from before there was a base entry.
        assert_eq!(text(&step.doc), "first word", "the first undo removed the wrong thing");

        // Undo again: the split goes.
        let step = history.undo().expect("the second undo");
        assert_eq!(text(&step.doc), "first word", "the second undo removed the wrong thing");

        // Undo again: the typing goes, all of it.
        let step = history.undo().expect("the third undo");
        assert_eq!(text(&step.doc), "first", "the typing was not one step");
    }
}
