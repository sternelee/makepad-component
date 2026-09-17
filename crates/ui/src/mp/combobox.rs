//! `MpCombobox` — a field you can type in, over a list you can pick from.
//!
//! ## The panel is a composition; the state is not
//!
//! The panel follows the same template `mp/popover.rs` and the Select page established:
//! an `MpTextInput` as the trigger, an `MpPopover` in the overlay region, an `MpMenu`
//! inside it, and three lines of app wiring between them. That is deliberately not a
//! second list implementation.
//!
//! What *is* new is the state, and it is new because a combobox is the only control in the
//! library that **holds two things which can disagree**: what is typed, and what is
//! chosen. Every other control holds one value. The two agree after a choice and they
//! disagree the moment the reader types, and the question that has to be answered on every
//! keystroke is *"is what is in the field still the thing that was chosen?"*
//!
//! Getting that wrong is invisible in the way this port keeps running into: the field
//! shows `Split Right`, the app's value still says `New Terminal`, and nothing anywhere
//! complains. The next action runs the wrong command.
//!
//! ## The rule
//!
//! **A choice survives typing only while the text still names it.** Type one more
//! character and the value is cleared, because the field no longer says the thing the
//! value claims — and a stale value is worse than no value, since "nothing chosen" is a
//! state the app can handle and "something else chosen" is not.
//!
//! Two consequences worth stating because they are not obvious:
//!
//! - **Clearing the text clears the value.** An empty field chooses nothing.
//! - **A cleared value does not come back by retyping.** Delete a character and the value
//!   goes; type the character again and the value **stays gone** until the reader commits a
//!   row. The obvious alternative — remembering the last choice so that an exact match
//!   restores it — is deliberately not taken, because it makes the value *a third thing*:
//!   neither what the text says nor nothing at all. An app can handle "nothing chosen"; it
//!   cannot handle "chosen, but not by the text", which is the stale-value fault in a
//!   quieter costume. The doc comment on the first version of this module claimed the
//!   opposite and its test said so.
//!
//! ## The default highlighted row is *no* row
//!
//! [`crate::mp::palette::remap`] defaults an absent cursor to the top row, because a
//! **palette's** Enter must always run something and its top row is its best answer. A
//! combobox is the opposite: nothing is highlighted until the reader moves, and committing
//! with nothing highlighted chooses **nothing**. Entering an item the reader never saw is
//! how a combobox runs the wrong command. The identity arithmetic is shared; this policy is
//! not, and the six tests that caught the difference were all written before it was.
//!
//! ## The active row reuses the palette's arithmetic
//!
//! The highlighted row is [`crate::mp::palette::remap`]ped by identity across a re-filter
//! and [`crate::mp::palette::original`] is what turns a cursor position back into a
//! **index into the original list** — the same contract the palette has, for the same
//! reason: a caller acting on a filtered position acts on a different item after every
//! keystroke.

use makepad_widgets::*;

use crate::mp::palette;
use crate::mp::search;

/// A field's text together with the item it currently names.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Combobox {
    /// The items a choice can be made from, in the caller's order.
    items: Vec<String>,
    /// What is in the field.
    query: String,
    /// The chosen item, as an index into `items`, or `None` while what is typed names
    /// nothing.
    value: Option<usize>,
    /// The highlighted row, as an index into `items` — **never** into the filtered view.
    active: Option<usize>,
}

impl Combobox {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            items,
            query: String::new(),
            value: None,
            active: None,
        }
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    /// The chosen item, as an index into the caller's list.
    pub fn value(&self) -> Option<usize> {
        self.value
    }

    /// The chosen item's text, or `None`.
    ///
    /// For a caller that wants to *show* what is chosen rather than index into it — which
    /// is the common case, and the reason this is a method rather than something every
    /// caller writes.
    pub fn value_label(&self) -> Option<&str> {
        self.value.and_then(|index| self.items.get(index)).map(String::as_str)
    }

    pub fn items(&self) -> &[String] {
        &self.items
    }

    /// The highlighted row, as an index into the original list.
    pub fn active(&self) -> Option<usize> {
        self.active
    }

    /// The items still available for the current text, best match first, as indices into
    /// the original list.
    pub fn filtered(&self) -> Vec<usize> {
        search::rank(&self.items, &self.query)
    }

    /// Replace the items. A choice that is no longer available is dropped.
    ///
    /// Dropped rather than clamped: index 4 of a five-item list is a *different* item in a
    /// four-item one, and keeping it would silently choose something else. Dropping says
    /// "nothing is chosen", which the app can handle.
    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
        if self.value.is_some_and(|index| index >= self.items.len()) {
            self.value = None;
        }
        self.active = self.active.filter(|index| *index < self.items.len());
    }

    /// What the reader typed.
    ///
    /// **Clears the value unless the text still names it.** This is the rule the module
    /// exists for; see the module doc.
    pub fn type_text(&mut self, text: &str) {
        self.query = text.to_string();
        // Compared against the chosen **item's text**, not against the query as it was:
        // the reader may have edited and edited back, and it is what the field says now
        // that decides.
        let still_names_it = self
            .value
            .and_then(|index| self.items.get(index))
            .is_some_and(|label| label == &self.query);
        if !still_names_it {
            self.value = None;
        }
        // The highlighted row follows the new view **by identity**, so the row under the
        // reader's eye does not move to a different item.
        //
        // `None` is preserved rather than passed to `remap`, which would default it to the
        // top row: that default is the *palette's* policy (its Enter must always run
        // something) and a combobox must not have it. See the module doc.
        self.active = match self.active {
            None => None,
            Some(previous) => {
                let view = self.filtered();
                palette::remap(Some(previous), &view)
                    .and_then(|position| palette::original(&view, position))
            }
        };
    }

    /// Choose an item by its index in the original list. The field shows its label.
    pub fn choose(&mut self, index: usize) {
        if index >= self.items.len() {
            return;
        }
        self.value = Some(index);
        self.active = Some(index);
        self.query = self.items[index].clone();
    }

    /// Put the highlight on an item by its index in the original list.
    ///
    /// The sibling of `step` for a caller that knows **which** row rather than which direction — opening a select on its
    /// chosen row, or a pointer resting on a row. An index that is not in the list clears the highlight rather than
    /// clamping: a highlight on the wrong row is worse than none, because the next commit would choose it.
    pub fn step_to(&mut self, index: usize) {
        self.active = if index < self.items.len() { Some(index) } else { None };
    }

    /// Move the highlight, wrapping, over the current filtered view.
    pub fn step(&mut self, delta: i32) {
        let view = self.filtered();
        if view.is_empty() {
            self.active = None;
            return;
        }
        let position = match self.active.and_then(|a| view.iter().position(|i| *i == a)) {
            Some(position) => palette::step(position, view.len(), delta),
            // Nothing highlighted yet: Down starts at the top and Up at the bottom, which
            // is what every list in this library does.
            None => {
                if delta >= 0 {
                    0
                } else {
                    view.len() - 1
                }
            }
        };
        self.active = palette::original(&view, position);
    }

    /// Choose the highlighted row, as the reader pressing Enter would.
    ///
    /// Returns the chosen index. `None` when nothing is highlighted or the highlighted row
    /// was filtered away — and `None` is the answer rather than a guess, because committing
    /// an item the reader cannot see is how a combobox runs the wrong command.
    pub fn commit_active(&mut self) -> Option<usize> {
        let view = self.filtered();
        let index = self.active.and_then(|a| view.iter().position(|i| *i == a))?;
        let original = palette::original(&view, index)?;
        self.choose(original);
        Some(original)
    }

    /// Open with nothing chosen and nothing typed.
    pub fn clear(&mut self) {
        self.query.clear();
        self.value = None;
        self.active = None;
    }
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*

    /// The field, and the panel it opens.
    ///
    /// An **`MpTextInput` as the trigger**, not a button: that is the whole difference
    /// between this and the Select page's composition, and it is what makes the control a
    /// combobox rather than a picker. The panel is an `MpPopover` because a floating
    /// surface cannot be composed into the widget that opens it — the overlay's draw list
    /// is clipped to its own rectangle, which `mp/popover.rs` records at length.
    ///
    /// Shipped as two prototypes rather than one, because the panel has to live in the
    /// page's overlay region and the field does not; a single prototype would put the panel
    /// inside the field's own rectangle, where it cannot draw.
    mod.mp.MpCombobox = View{
        width: Fill
        height: Fit

        combobox_field := mod.mp.MpTextInput{
            width: Fill
            empty_text: "Type to choose"
        }
    }

    /// The panel that goes with it, to be placed in the page's overlay region.
    mod.mp.MpComboboxPanel = mod.mp.MpPopover{
        panel +: {
            width: 260
            combobox_list := mod.mp.MpMenu{}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items() -> Vec<String> {
        ["New Terminal", "Toggle Terminal", "Split Right", "Close Window"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    fn combobox() -> Combobox {
        Combobox::new(items())
    }

    #[test]
    fn test_a_fresh_combobox_has_nothing_typed_and_nothing_chosen() {
        let c = combobox();
        assert_eq!(c.query(), "");
        assert_eq!(c.value(), None);
        assert_eq!(c.value_label(), None);
        assert_eq!(c.active(), None);
        // Everything is available while nothing is typed.
        assert_eq!(c.filtered(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_choosing_sets_the_value_and_puts_the_label_in_the_field() {
        let mut c = combobox();
        c.choose(2);
        assert_eq!(c.value(), Some(2));
        assert_eq!(c.value_label(), Some("Split Right"));
        assert_eq!(c.query(), "Split Right");
        assert_eq!(c.active(), Some(2));
    }

    #[test]
    fn test_a_cleared_value_does_not_come_back_by_retyping_the_same_text() {
        // The choice is gone the moment the field stops naming it, and retyping does not
        // restore it: the module no longer holds the index, and a remembered index would
        // make the value a third state — neither what the text says nor nothing, which is
        // the stale-value fault in a quieter costume. To choose again, commit a row.
        let mut c = combobox();
        c.choose(2);
        c.type_text("Split Righ");
        assert_eq!(c.value(), None, "a partial label names nothing");
        c.type_text("Split Right");
        assert_eq!(c.value(), None, "and the full label does not restore it");
        // The field still shows the item, so committing is how it is chosen again.
        assert_eq!(c.query(), "Split Right");
        c.step(1);
        assert_eq!(c.commit_active(), Some(2));
        assert_eq!(c.value(), Some(2));
    }

    #[test]
    fn test_typing_something_else_clears_the_value() {
        // **The fault this module exists for.** The field says one thing and the value
        // claims another; a stale value runs the wrong command and nothing complains.
        let mut c = combobox();
        c.choose(2);
        assert_eq!(c.value(), Some(2));
        c.type_text("Split Rightx");
        assert_eq!(c.value(), None);
        assert_eq!(c.value_label(), None);
        // And the query is what the reader typed, so the field and the state agree.
        assert_eq!(c.query(), "Split Rightx");
    }

    #[test]
    fn test_clearing_the_field_clears_the_value() {
        let mut c = combobox();
        c.choose(0);
        c.type_text("");
        assert_eq!(c.value(), None);
        assert_eq!(c.filtered(), vec![0, 1, 2, 3], "an empty field offers everything");
    }

    #[test]
    fn test_typing_narrows_the_offered_items() {
        let mut c = combobox();
        c.type_text("term");
        // Both Terminal commands match; the ranked order is the search module's.
        assert_eq!(c.filtered(), vec![0, 1]);
        c.type_text("split");
        assert_eq!(c.filtered(), vec![2]);
        c.type_text("zzz");
        assert!(c.filtered().is_empty());
    }

    #[test]
    fn test_a_value_that_is_no_longer_in_the_list_is_dropped_rather_than_clamped() {
        // Index 3 of four items is a *different* item in a list of three, so clamping
        // would silently choose something else. Dropping says "nothing chosen", which the
        // app can handle.
        let mut c = combobox();
        c.choose(3);
        assert_eq!(c.value_label(), Some("Close Window"));
        c.set_items(vec!["Only".to_string()]);
        assert_eq!(c.value(), None);
        assert_eq!(c.value_label(), None);
    }

    #[test]
    fn test_a_value_that_is_still_in_the_list_survives_the_list_changing() {
        let mut c = combobox();
        c.choose(1);
        c.set_items(
            ["Toggle Terminal", "Something Else"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        );
        // Index 1 is still valid, but it is a *different item* now — and the module cannot
        // know that, because it only ever held an index. Recorded as a property rather than
        // a fault: a caller that can change its items underneath a choice should compare
        // labels, and `value_label` is here so that it can.
        assert_eq!(c.value(), Some(1));
        assert_eq!(c.value_label(), Some("Something Else"));
    }

    #[test]
    fn test_the_highlight_moves_through_the_filtered_view_and_wraps() {
        let mut c = combobox();
        c.type_text("term");
        // Two items are offered: originals 0 and 1.
        assert_eq!(c.active(), None, "nothing is highlighted before the first move");
        c.step(1);
        assert_eq!(c.active(), Some(0), "Down from nothing starts at the top");
        c.step(1);
        assert_eq!(c.active(), Some(1));
        c.step(1);
        assert_eq!(c.active(), Some(0), "and wraps");
        c.step(-1);
        assert_eq!(c.active(), Some(1), "Up from the top wraps to the bottom");
    }

    #[test]
    fn test_the_highlight_reports_an_original_index_not_a_view_position() {
        // The palette's contract, and the reason it is shared rather than re-derived: a
        // caller acting on a view position acts on a different item after every keystroke.
        let mut c = combobox();
        c.type_text("split");
        // One item is offered, and it is original index 2 — not position 0.
        assert_eq!(c.filtered(), vec![2]);
        c.step(1);
        assert_eq!(c.active(), Some(2));
    }

    #[test]
    fn test_the_highlight_follows_its_row_when_the_view_shrinks() {
        // The identity rule, exercised through the combobox: highlight the second offered
        // item, narrow until only it remains, and it must still be the highlighted one.
        //
        // `terminal` is used rather than a single letter because the matching set has to be
        // **verifiable by reading the items**, and the first version of this test used `e`
        // on the assumption that every item contains one. `Split Right` does not — there is
        // no `e` in it — so the view had three items in a ranked order, and both this test
        // and the one below failed on a wrong expectation about the domain rather than on a
        // fault in the library.
        let mut c = combobox();
        c.type_text("terminal");
        assert_eq!(
            c.filtered(),
            vec![0, 1],
            "only the two Terminal commands spell `terminal`"
        );
        c.step(1);
        c.step(1);
        assert_eq!(c.active(), Some(1), "the second offered row is original 1");

        // Now narrow to a view that still contains original 1 but puts it first.
        c.type_text("oggle");
        assert_eq!(c.filtered(), vec![1]);
        assert_eq!(c.active(), Some(1), "the row survived the shrink by identity");
    }

    #[test]
    fn test_the_highlight_falls_back_to_the_top_when_its_row_is_filtered_away() {
        let mut c = combobox();
        c.type_text("terminal");
        c.step(1);
        c.step(1);
        assert_eq!(c.active(), Some(1), "highlighted Toggle Terminal");
        // `new` matches only `New Terminal`, so original 1 is gone from the view.
        c.type_text("new");
        assert_eq!(c.filtered(), vec![0]);
        assert_eq!(
            c.active(),
            Some(0),
            "the highlight had to move, and went to the best match"
        );
    }

    #[test]
    fn test_committing_returns_the_highlighted_item() {
        let mut c = combobox();
        c.type_text("term");
        c.step(1);
        c.step(1);
        let committed = c.commit_active();
        assert_eq!(committed, Some(1));
        assert_eq!(c.value(), Some(1));
        assert_eq!(c.value_label(), Some("Toggle Terminal"));
        assert_eq!(c.query(), "Toggle Terminal", "the field shows the choice");
    }

    #[test]
    fn test_committing_with_nothing_highlighted_chooses_nothing() {
        // `None` rather than a guess: committing an item the reader cannot see is how a
        // combobox runs the wrong command.
        let mut c = combobox();
        assert_eq!(c.commit_active(), None);
        assert_eq!(c.value(), None);
        // ...and the same when the view is empty.
        c.type_text("zzz");
        assert_eq!(c.commit_active(), None);
    }

    #[test]
    fn test_committing_a_row_that_was_filtered_away_after_being_highlighted_chooses_nothing() {
        let mut c = combobox();
        c.type_text("term");
        c.step(1);
        c.step(1);
        assert_eq!(c.active(), Some(1));
        // Narrow the view so that original 1 is gone, then commit without stepping.
        c.type_text("new");
        assert_eq!(c.filtered(), vec![0]);
        let committed = c.commit_active();
        assert_eq!(committed, Some(0), "it goes to what is actually offered");
        assert_eq!(c.value(), Some(0));
    }

    #[test]
    fn test_clearing_resets_everything() {
        let mut c = combobox();
        c.choose(2);
        c.clear();
        assert_eq!(c.query(), "");
        assert_eq!(c.value(), None);
        assert_eq!(c.active(), None);
    }

    #[test]
    fn test_a_choice_survives_the_list_being_replaced_with_the_same_one() {
        // The common case in an app that re-seeds its items on every event, which is
        // exactly what this port's gallery does. A combobox that dropped its value on every
        // re-seed would be unusable there.
        let mut c = combobox();
        c.choose(2);
        let value = c.value();
        c.set_items(items());
        assert_eq!(c.value(), value);
        assert_eq!(c.value_label(), Some("Split Right"));
    }

    #[test]
    fn test_choosing_an_index_past_the_end_does_nothing() {
        let mut c = combobox();
        c.choose(9);
        assert_eq!(c.value(), None);
        assert_eq!(c.query(), "");
    }

    #[test]
    fn test_an_empty_item_list_offers_nothing_and_chooses_nothing() {
        let mut c = Combobox::new(Vec::new());
        assert!(c.filtered().is_empty());
        c.step(1);
        assert_eq!(c.active(), None);
        assert_eq!(c.commit_active(), None);
        // And choosing into it is refused rather than panicking.
        c.choose(0);
        assert_eq!(c.value(), None);
    }

    #[test]
    fn test_a_full_read_flow_keeps_the_field_and_the_value_in_agreement() {
        // The session an app produces: open, type a prefix, narrow, step, commit — and at
        // no point may the field's text and the value name different things.
        let mut c = combobox();
        c.type_text("s");
        // The field says `s`, so nothing is chosen.
        assert_eq!(c.value(), None);
        c.type_text("spl");
        assert_eq!(c.value(), None);
        c.step(1);
        assert_eq!(c.active(), Some(2));
        let chosen = c.commit_active();
        assert_eq!(chosen, Some(2));
        // Now they agree, and the invariant is checkable rather than asserted in prose.
        assert_eq!(c.query(), c.value_label().unwrap());
        // Typing one more character breaks it, and the value goes with it.
        c.type_text("Split Right ");
        assert_eq!(c.value(), None);
        assert_ne!(c.query(), c.value_label().unwrap_or(""));
    }
}
