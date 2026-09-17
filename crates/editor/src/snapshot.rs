//! Undo and redo, as a value: the **generic** stack, with no policy of its own.
//!
//! This is bezel's `SnapshotHistory` — a `Vec` of states and a cursor — and it is deliberately ignorant of
//! what a state *is* and of when two edits should become one undo step. That policy is `history.rs`, one
//! level up, because **only the caller knows what an edit means**.
//!
//! It used to live in `crates/ui/src/mp/history.rs`, a **widget** crate, which is the wrong home for a data
//! structure with no widgets in it. Moving it here is what let the document history be written on top of it
//! rather than beside it.
//!
//! ## Why this is a type and not a pair of buttons
//!
//! An undo stack is the piece of an editor that is **wrong in a way nobody reports**:
//! the button lights up and the document goes somewhere it was never in. Every fault is
//! a state-order fault, so none of them shows in a screenshot and none of them throws.
//! Two of them are worth naming because they are the two that actually happen:
//!
//! - **A push after an undo must truncate the redo branch.** Undo three steps, type one
//!   character, and the forward history is gone — that is what every editor does, and a
//!   stack that keeps the old branch will "redo" its way into a state that never followed
//!   from what is on screen.
//! - **Dropping the oldest entry when the stack is full must move the cursor with it.**
//!   A bounded history is the normal case (an editor keeps a hundred steps, not all of
//!   them), and a cursor that is not shifted when the front is dropped silently makes undo
//!   skip a state — the one fault here that produces a *wrong document* rather than a
//!   wrong button.
//!
//! ## The shape
//!
//! One `Vec` with a cursor, which is the shape that makes both faults above expressible
//! as arithmetic on one index:
//!
//! ```text
//!   entries: [ A, B, C, D ]
//!   cursor:        ^        current() == C, can_redo() because D follows
//! ```
//!
//! A `Vec` of entries plus a cursor is preferred to two stacks (`undo` and `redo`)
//! because the two-stack shape makes the truncation rule an operation on *two* things
//! that have to stay consistent, and the bounded case an operation on one of them while
//! the other holds the states that were just dropped.
//!
//! ## What it does not do
//!
//! No coalescing (typing ten characters is ten steps here, one step in an editor that
//! groups by word), and no transactions. Both are policies layered on top, and both need
//! to know what the states *mean* — which is the caller's business, not this type's.

/// A linear undo/redo stack over states of type `T`.
///
/// `T` is cloned in and cloned out, which is what keeps the type free of any assumption
/// about how a state is produced. An editor clones a document; a settings pane clones a
/// struct; neither has to hand over ownership.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotHistory<T> {
    /// Every retained state, **oldest first**.
    entries: Vec<T>,
    /// The index of the current state. Always a valid index while `entries` is non-empty,
    /// and `entries` is never empty because `new` takes an initial state.
    cursor: usize,
    /// How many states to retain. `None` is unbounded.
    capacity: Option<usize>,
}

impl<T: Clone> SnapshotHistory<T> {
    /// A history whose only state is `initial`.
    ///
    /// Nothing is undoable yet, which is the correct answer for a document that has not
    /// been edited: a button that offered to undo to *nothing* would be lying.
    pub fn new(initial: T) -> Self {
        Self {
            entries: vec![initial],
            cursor: 0,
            capacity: None,
        }
    }

    /// A history that retains at most `capacity` states.
    ///
    /// A capacity of zero is treated as one: a history with no states at all has no
    /// current state, and every method here would have to answer for that. One state is
    /// the smallest thing that can be called a history, and it is what a capacity of zero
    /// means by the time it reaches [`History::push`].
    pub fn with_capacity(initial: T, capacity: usize) -> Self {
        Self {
            entries: vec![initial],
            cursor: 0,
            capacity: Some(capacity.max(1)),
        }
    }

    /// The state to show.
    pub fn current(&self) -> &T {
        // `entries` is never empty, so this cannot fail; `get` with a fallback rather
        // than indexing because a panic in a history is worse than a stale state.
        self.entries.get(self.cursor).unwrap_or(&self.entries[0])
    }

    pub fn can_undo(&self) -> bool {
        self.cursor > 0
    }

    pub fn can_redo(&self) -> bool {
        self.cursor + 1 < self.entries.len()
    }

    /// Record `state` as the new current state.
    ///
    /// **Discards the redo branch.** Undo three steps, type a character, and the forward
    /// history is gone — that is what every editor does. A stack that kept it would let
    /// redo walk into a state that never followed from what is on screen, which is the
    /// fault that produces a document the user never typed.
    ///
    /// Pushing a state **equal to the current one** is recorded rather than ignored. This
    /// type cannot know whether two equal states are one edit or two, and silently
    /// dropping a push would make undo skip the step — the same class of fault as the
    /// cursor bug above. Coalescing is the caller's policy, because only the caller knows
    /// what a state means.
    pub fn push(&mut self, state: T) {
        // Truncate the branch being abandoned.
        self.entries.truncate(self.cursor + 1);
        self.entries.push(state);
        self.cursor = self.entries.len() - 1;
        self.enforce_capacity();
    }

    /// Step back, returning the state to show.
    ///
    /// `None` when there is nothing to undo, and the state is unchanged.
    pub fn undo(&mut self) -> Option<&T> {
        if !self.can_undo() {
            return None;
        }
        self.cursor -= 1;
        Some(self.current())
    }

    /// Step forward, returning the state to show.
    ///
    /// `None` when there is nothing to redo, and the state is unchanged.
    pub fn redo(&mut self) -> Option<&T> {
        if !self.can_redo() {
            return None;
        }
        self.cursor += 1;
        Some(self.current())
    }

    /// Replace the current state **without** moving the cursor.
    ///
    /// For coalescing: a run of edits that counts as one undo step keeps the *first* state of the run as the
    /// step's snapshot, so a later state in the same run has to overwrite it rather than push. Written as an
    /// explicit method because the alternative — remembering to push only the first time — is a rule a caller
    /// has to get right in two places.
    pub fn replace_current(&mut self, state: T) -> bool {
        match self.entries.get_mut(self.cursor) {
            Some(slot) => {
                *slot = state;
                true
            }
            None => false,
        }
    }

    /// How many states are retained, the current one included.
    /// The limit this stack was built with, or `None` when it has none.
    ///
    /// An accessor rather than a public field, and it exists for `History::reset`: replacing the contents of a
    /// history must not silently drop the limit the caller chose.
    pub fn capacity(&self) -> Option<usize> {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        // Never, by construction — `new` puts a state in and nothing removes the last
        // one. Present because a `len` without an `is_empty` is a lint.
        self.entries.is_empty()
    }

    /// How many steps back and forward are available, for a pair of buttons.
    pub fn depth(&self) -> (usize, usize) {
        (self.cursor, self.entries.len() - 1 - self.cursor)
    }

    /// Every retained state, oldest first. For a page that shows the stack.
    pub fn entries(&self) -> &[T] {
        &self.entries
    }

    /// Where the cursor is, for a page that shows the stack.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Drop the oldest states until the capacity is met, **moving the cursor with them**.
    ///
    /// The `- dropped` is the whole reason this is a method rather than an inline loop: a
    /// cursor left pointing at its old index after the front was removed points at a
    /// *newer* state than the one the user was on, so the next undo skips a step. That is
    /// the fault here that produces a wrong document rather than a wrong button.
    fn enforce_capacity(&mut self) {
        let Some(capacity) = self.capacity else {
            return;
        };
        if self.entries.len() <= capacity {
            return;
        }
        let dropped = self.entries.len() - capacity;
        self.entries.drain(..dropped);
        self.cursor = self.cursor.saturating_sub(dropped);
    }
}

/// The empty history: the one whose only state is the type's default.
///
/// A history is never truly empty — it always has a current state — so "default" means the
/// smallest thing that is still a history. It is here because a host needs its state
/// fields to be constructible before the first event, and a state container that could not
/// be defaulted would force every caller to wrap it in an `Option` for that reason alone.
impl<T: Clone + Default> Default for SnapshotHistory<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A history of single characters, which is the smallest state that reads clearly.
    fn history() -> SnapshotHistory<char> {
        SnapshotHistory::new('a')
    }

    #[test]
    fn test_the_default_history_holds_the_default_state_and_offers_nothing() {
        // What a host gets before it has seeded anything.
        let h = SnapshotHistory::<u32>::default();
        assert_eq!(*h.current(), 0);
        assert!(!h.can_undo());
        assert!(!h.can_redo());
        assert_eq!(h.len(), 1);
    }

    #[test]
    fn test_a_fresh_history_offers_nothing_in_either_direction() {
        // A button offering to undo to *nothing* is lying.
        let h = history();
        assert_eq!(*h.current(), 'a');
        assert!(!h.can_undo());
        assert!(!h.can_redo());
        assert_eq!(h.depth(), (0, 0));
        assert_eq!(h.len(), 1);
    }

    #[test]
    fn test_undo_and_redo_walk_the_states_in_order() {
        let mut h = history();
        h.push('b');
        h.push('c');
        h.push('d');
        assert_eq!(*h.current(), 'd');
        assert_eq!(h.depth(), (3, 0));

        assert_eq!(h.undo(), Some(&'c'));
        assert_eq!(h.undo(), Some(&'b'));
        assert_eq!(h.undo(), Some(&'a'));
        assert_eq!(h.depth(), (0, 3));

        assert_eq!(h.redo(), Some(&'b'));
        assert_eq!(h.redo(), Some(&'c'));
        assert_eq!(h.redo(), Some(&'d'));
        assert_eq!(*h.current(), 'd');
    }

    #[test]
    fn test_undo_stops_at_the_beginning_and_redo_at_the_end() {
        // Both return `None` and neither moves, which is what a disabled button needs.
        let mut h = history();
        h.push('b');
        assert_eq!(h.undo(), Some(&'a'));
        assert_eq!(h.undo(), None);
        assert_eq!(*h.current(), 'a', "a refused undo must not move");
        assert_eq!(h.redo(), Some(&'b'));
        assert_eq!(h.redo(), None);
        assert_eq!(*h.current(), 'b', "a refused redo must not move");
    }

    #[test]
    fn test_a_push_after_an_undo_abandons_the_redo_branch() {
        // **The fault that produces a document nobody typed.** Undo to `a`, then push
        // `x`: `b` and `c` never followed from `x`, so redo must not reach them.
        let mut h = history();
        h.push('b');
        h.push('c');
        h.undo();
        h.undo();
        assert_eq!(*h.current(), 'a');
        assert!(h.can_redo());

        h.push('x');
        assert_eq!(*h.current(), 'x');
        assert!(
            !h.can_redo(),
            "the abandoned branch is still reachable: {:?}",
            h.entries()
        );
        assert_eq!(h.entries(), &['a', 'x']);
        // And undo goes back to `a`, not into the abandoned branch.
        assert_eq!(h.undo(), Some(&'a'));
        assert_eq!(h.undo(), None);
    }

    #[test]
    fn test_a_push_from_the_middle_truncates_and_not_merely_overwrites() {
        // A longer branch, so that a truncation off by one would leave an entry behind.
        let mut h = history();
        for c in ['b', 'c', 'd', 'e', 'f'] {
            h.push(c);
        }
        h.undo();
        h.undo();
        h.undo();
        // current == 'c', with 'd', 'e', 'f' ahead of it.
        assert_eq!(h.depth(), (2, 3));
        h.push('z');
        assert_eq!(h.entries(), &['a', 'b', 'c', 'z']);
        assert_eq!(h.depth(), (3, 0));
    }

    #[test]
    fn test_pushing_the_current_state_again_is_recorded() {
        // Recorded rather than ignored, and the reason is stated in the type's doc: this
        // type cannot know whether two equal states are one edit or two, and silently
        // dropping a push makes undo skip the step. Coalescing is the caller's policy.
        let mut h = history();
        h.push('a');
        assert_eq!(h.len(), 2);
        assert_eq!(h.undo(), Some(&'a'));
        assert!(!h.can_undo());
    }

    #[test]
    fn test_a_bounded_history_drops_the_oldest_and_keeps_the_current_state() {
        // The capacity case, without the cursor being tested yet: what the user sees must
        // not change when an old state falls off the back.
        let mut h = SnapshotHistory::with_capacity('a', 3);
        for c in ['b', 'c', 'd'] {
            h.push(c);
        }
        assert_eq!(h.entries(), &['b', 'c', 'd']);
        assert_eq!(*h.current(), 'd');
        assert_eq!(h.depth(), (2, 0));
    }

    #[test]
    fn test_a_bounded_history_does_not_skip_a_state_when_the_front_is_dropped() {
        // **The cursor fault.** After the oldest entry is dropped the cursor has to move
        // with it, or undo jumps two states at once and the user sees a document that
        // skipped an edit. Undone step by step here, so a skip shows as a missing value.
        let mut h = SnapshotHistory::with_capacity('a', 3);
        for c in ['b', 'c', 'd'] {
            h.push(c);
        }
        // The buffer is now [b, c, d] and the cursor is on `d`.
        assert_eq!(h.undo(), Some(&'c'));
        assert_eq!(h.undo(), Some(&'b'));
        assert_eq!(h.undo(), None, "`a` was dropped, so this is the floor");
        assert_eq!(h.depth(), (0, 2));
    }

    #[test]
    fn test_a_bounded_history_drops_from_the_front_while_the_cursor_is_in_the_middle() {
        // The case where the cursor is not at the end when the front is dropped: it still
        // has to point at the same *state*, not the same index.
        let mut h = SnapshotHistory::with_capacity('a', 3);
        h.push('b');
        h.push('c');
        // [a, b, c], cursor on `c`. Go back to `a`.
        h.undo();
        h.undo();
        assert_eq!(*h.current(), 'a');
        // Push `d`: the branch `b`,`c` is abandoned, giving [a, d] — which is *within*
        // the capacity of three, so nothing is dropped yet. The first version of this
        // test expected a drop here and the library was right: a bounded history drops
        // when it is full, not when it has room.
        h.push('d');
        assert_eq!(h.entries(), &['a', 'd']);
        h.push('e');
        assert_eq!(h.entries(), &['a', 'd', 'e'], "three of three, nothing dropped");

        // Now one more than the capacity, and the front goes — taking the cursor with it.
        h.push('f');
        assert_eq!(h.entries(), &['d', 'e', 'f']);
        assert_eq!(*h.current(), 'f');
        // The steps, one at a time: a cursor that had not moved would skip `e` here.
        assert_eq!(h.undo(), Some(&'e'));
        assert_eq!(h.undo(), Some(&'d'));
        assert_eq!(h.undo(), None, "`a` was dropped, so `d` is the floor");
        assert_eq!(h.depth(), (0, 2));
    }

    #[test]
    fn test_a_capacity_of_one_retains_only_the_current_state() {
        // The degenerate bound, where every push drops the previous state — so undo is
        // never available and that is the correct answer rather than a bug.
        let mut h = SnapshotHistory::with_capacity('a', 1);
        assert_eq!(h.len(), 1);
        h.push('b');
        assert_eq!(h.entries(), &['b']);
        assert!(!h.can_undo());
        assert_eq!(*h.current(), 'b');
    }

    #[test]
    fn test_a_capacity_of_zero_is_treated_as_one_rather_than_as_no_states() {
        // Zero would mean a history with no current state, which every method would have
        // to answer for. One is the smallest thing that is still a history.
        let mut h = SnapshotHistory::with_capacity('a', 0);
        assert_eq!(h.len(), 1);
        assert_eq!(*h.current(), 'a');
        h.push('b');
        assert_eq!(h.len(), 1);
        assert_eq!(*h.current(), 'b');
    }

    #[test]
    fn test_an_unbounded_history_keeps_everything() {
        let mut h = history();
        for c in 'b'..='z' {
            h.push(c);
        }
        assert_eq!(h.len(), 26);
        assert_eq!(h.depth(), (25, 0));
        assert_eq!(*h.current(), 'z');
    }

    /// A deterministic pseudo-random source, so a failure is reproducible.
    fn lcg(seed: &mut u64) -> u64 {
        *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *seed >> 33
    }

    #[test]
    fn test_a_walk_all_the_way_back_and_forward_returns_to_where_it_started() {
        // The property that covers the boundary cases nobody writes a test for: whatever
        // mixture of pushes, undos and redos an editor performs, reaching the floor and
        // the ceiling and coming back must land on the same state. If the cursor and the
        // entries could disagree, this finds it.
        let mut seed = 0x5eed_1234_u64;
        for capacity in [1usize, 2, 3, 8, 64] {
            let mut h = SnapshotHistory::with_capacity(0u32, capacity);
            for step in 0..400u32 {
                match lcg(&mut seed) % 3 {
                    0 => h.push(step),
                    1 => {
                        h.undo();
                    }
                    _ => {
                        h.redo();
                    }
                }
                // Invariants that must hold after every single operation.
                assert!(!h.is_empty(), "a history always has a current state");
                assert!(h.cursor() < h.len(), "the cursor is always a valid index");
                assert!(
                    h.entries().len() <= capacity,
                    "capacity {capacity} was exceeded: {} entries",
                    h.entries().len()
                );
                assert_eq!(
                    *h.current(),
                    h.entries()[h.cursor()],
                    "current() must be the entry at the cursor"
                );
                let (undoable, redoable) = h.depth();
                assert_eq!(undoable, h.cursor());
                assert_eq!(undoable + redoable + 1, h.len());
            }

            // Walk to the floor, then to the ceiling, and check both ends are consistent.
            let start = h.current().clone();
            while h.can_undo() {
                h.undo();
            }
            assert_eq!(*h.current(), h.entries()[0]);
            assert_eq!(h.depth().0, 0);
            while h.can_redo() {
                h.redo();
            }
            assert_eq!(h.depth().1, 0);
            assert_eq!(h.cursor(), h.len() - 1);
            // Coming back to the same position must give the same state, which is the
            // claim that the two directions share one cursor rather than two.
            let end = h.current().clone();
            let at_end = h.entries().len() - 1;
            while h.can_undo() {
                h.undo();
            }
            while h.can_redo() && h.cursor() < at_end {
                h.redo();
            }
            assert_eq!(*h.current(), end, "a round trip moved the state (capacity {capacity})");
            let _ = start;
        }
    }

    #[test]
    fn test_a_real_editing_session_reads_the_way_an_editor_behaves() {
        // A session in the shape an editor produces, so the type is exercised the way it
        // will be used rather than only in the abstract: type, undo, retype, then edit
        // past the capacity.
        let mut h = SnapshotHistory::with_capacity(String::new(), 4);
        let mut doc = String::new();
        for ch in ['h', 'e', 'l', 'l', 'o'] {
            doc.push(ch);
            h.push(doc.clone());
        }
        assert_eq!(h.current(), "hello");
        assert_eq!(h.depth(), (3, 0), "a capacity of four keeps four states");

        // Undo twice: "hel"
        h.undo();
        assert_eq!(h.current(), "hell");
        h.undo();
        assert_eq!(h.current(), "hel");
        assert!(h.can_redo());

        // Retype a different character: the old branch is gone.
        doc.truncate(3);
        doc.push('p');
        h.push(doc.clone());
        assert_eq!(h.current(), "help");
        assert!(!h.can_redo());
        assert!(
            !h.entries().iter().any(|s| s == "hello"),
            "the abandoned branch survived: {:?}",
            h.entries()
        );
    }
}
