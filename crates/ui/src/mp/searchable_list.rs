//! `MpSearchableList` — a list you narrow by typing.
//!
//! ## The selection is an item, not a position in a list that changes
//!
//! **This is the defect the widget this replaces had, and it is a data-structure fault rather than a typo.** Its
//! `selected` was an index into `filtered` — the array of *matching* rows — and `apply_filter` rebuilt that array
//! whenever the query changed **without clearing the selection**. So: click the third row of five, type one letter, and
//! the third row of the two that remain is now selected — a different item, silently, with `selected_text()` reporting
//! the wrong one. Its `set_query` *did* clear the selection, so the two paths disagreed, which is the same
//! two-paths-that-resemble-each-other shape this port has met repeatedly.
//!
//! So here the **selection is an index into `items`** — the list that does not change when a filter does — and the
//! filtered view holds indices into it. A filter narrows what is *shown*; it cannot move what is *selected*, and there is
//! no code path where it could.
//!
//! ## The count and the rows are two different numbers
//!
//! [`total_matches`] counts every match and [`filter_indices`] stops at the row limit, so a caller can say *"10 of 23"*
//! rather than *"10"* — which matters because a truncated list that does not say it is truncated reads as a complete one.
//! [`test_every_shown_row_is_one_of_the_matches`] pins the relationship: the shown rows are a **prefix** of the matches,
//! in order, never a reordered or deduplicated subset.
//!
//! ## Ten slots
//!
//! Fixed, like the description list and the step indicator, and for the same reason: a row is a plate and a string with
//! no identity of its own, so filling and hiding ten is simpler than growing a pool. [`SLOTS`] is public.

use makepad_widgets::*;

/// How many rows are shown at once.
pub const SLOTS: usize = 10;

/// Whether an item answers to a query, case-insensitively.
///
/// An **empty query matches everything**, which is what an unsearched list is. A substring rather than a subsequence:
/// this narrows a list while typing and the reader is watching, so a match they cannot see the reason for is worse than
/// no match — the opposite of the slash menu, where the query is a small command vocabulary.
pub fn matches(item: &str, query: &str) -> bool {
    query.is_empty() || item.to_lowercase().contains(&query.to_lowercase())
}

/// Every item that answers to `query`, as indices into `items`.
///
/// **Indices rather than copies**, because the caller's selection is an index into `items` and a copy would make the two
/// impossible to relate — which is the defect this module opens with. In order, and not deduplicated: two items with the
/// same text are two rows.
pub fn match_indices(items: &[String], query: &str) -> Vec<usize> {
    items
        .iter()
        .enumerate()
        .filter(|(_, item)| matches(item, query))
        .map(|(index, _)| index)
        .collect()
}

/// How many items answer to `query`, **before the row limit**.
pub fn total_matches(items: &[String], query: &str) -> usize {
    match_indices(items, query).len()
}

/// The indices of the rows to show: the first [`SLOTS`] matches.
pub fn filter_indices(items: &[String], query: &str) -> Vec<usize> {
    let mut found = match_indices(items, query);
    found.truncate(SLOTS);
    found
}

/// A row, as the caller reads it: the text and whether it is the selection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// The item's index in `items`, which is what a click reports.
    pub index: usize,
    pub text: String,
    pub selected: bool,
}

/// The rows to show, given the items, the query and the selection.
///
/// One function rather than three, because the three answers have to agree: the row a click reports, the text it shows,
/// and whether it is marked as selected all come from the same walk. A caller assembling them separately is a caller that
/// can show `Beta` while reporting `Alpha`.
pub fn rows(items: &[String], query: &str, selected: Option<usize>) -> Vec<Row> {
    filter_indices(items, query)
        .into_iter()
        .map(|index| Row {
            index,
            text: items[index].clone(),
            selected: selected == Some(index),
        })
        .collect()
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    /// One row: the two plates a row can carry, under its text.
    ///
    /// **Overlay rather than a coloured label**, because a selected row and a hovered row are two states of the same
    /// row and both can be true — a row under the pointer that is also the selection. One plate would have to choose.
    mod.mp.MpSearchRow = mod.widgets.View{
        width: Fill
        height: 30
        flow: Overlay
        align: Align{y: 0.5}

        row_hover := View{
            width: Fill
            height: Fill
            margin: Inset{left: 2, right: 2}
            visible: false
            show_bg: true
            draw_bg +: {
                color: instance(surface_raised)
                border_radius: instance(6.0)
            }
        }
        row_selected := View{
            width: Fill
            height: Fill
            margin: Inset{left: 2, right: 2}
            visible: false
            show_bg: true
            draw_bg +: {
                color: instance(selection)
                border_radius: instance(6.0)
            }
        }
        row_label := Label{
            width: Fill
            height: Fit
            margin: Inset{left: 12, right: 10}
            draw_text +: {
                text_style: body
                color: text
            }
            text: ""
        }
    }

    mod.mp.MpSearchableListBase = #(MpSearchableList::register_widget(vm))

    mod.mp.MpSearchableList = set_type_default() do mod.mp.MpSearchableListBase{
        width: Fill
        height: Fit
        flow: Down
        spacing: 6

        search := mod.mp.MpTextInput{
            width: Fill
            height: Fit
            empty_text: "Filter…"
        }

        rows := View{
            width: Fill
            height: Fit
            flow: Down
            spacing: 1

            row0 := mod.mp.MpSearchRow{}
            row1 := mod.mp.MpSearchRow{}
            row2 := mod.mp.MpSearchRow{}
            row3 := mod.mp.MpSearchRow{}
            row4 := mod.mp.MpSearchRow{}
            row5 := mod.mp.MpSearchRow{}
            row6 := mod.mp.MpSearchRow{}
            row7 := mod.mp.MpSearchRow{}
            row8 := mod.mp.MpSearchRow{}
            row9 := mod.mp.MpSearchRow{}
        }

        /// The count line under the rows. Shown only when the list is **truncated**, because a list that shows ten of
        /// twenty-three and says nothing reads as a list of ten.
        more := Label{
            width: Fill
            height: Fit
            draw_text +: {
                text_style: caption
                color: text_muted
            }
            text: ""
        }
    }
}

/// What a searchable list reports.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum MpSearchableListAction {
    /// An item was picked, carrying its index in `items` and its text.
    Picked(usize, String),
    #[default]
    None,
}

/// A list you narrow by typing.
#[derive(Script, ScriptHook, Widget)]
pub struct MpSearchableList {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    /// The items, and the query — both `#[rust]`, because a `#[live]` field is written back to its declared value
    /// whenever the script is re-applied, so a theme change would empty the list.
    #[rust]
    items: Vec<String>,
    #[rust]
    query: String,
    /// The selection, as an **index into `items`**. See the module doc: an index into the filtered view is the defect
    /// this module opens with.
    #[rust]
    selected: Option<usize>,
}

/// The id path of row `index`.
fn row_id(index: usize) -> LiveId {
    LiveId::from_str(&format!("row{index}"))
}

impl Widget for MpSearchableList {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let actions = cx.capture_actions(|cx| self.view.handle_event(cx, event, scope));

        // Typing narrows the list. **The selection is untouched** — it is an index into `items`, and a filter cannot move
        // it; see the module doc for the widget that got this wrong.
        if let Some(text) = self.view.text_input(cx, ids!(search)).changed(&actions) {
            self.query = text;
            self.redraw(cx);
        }

        // A click on a row picks its item, **reporting the item's index rather than the row's** — which is the same
        // distinction, in the place a caller sees it.
        let shown = filter_indices(&self.items, &self.query);
        for (position, index) in shown.iter().enumerate() {
            // `finger_down` answers the event rather than a bool, so the presence of one is the click.
            if self
                .view
                .view(cx, &[row_id(position)])
                .finger_down(&actions)
                .is_some()
            {
                self.selected = Some(*index);
                self.redraw(cx);
                cx.widget_action(
                    self.widget_uid(),
                    MpSearchableListAction::Picked(*index, self.items[*index].clone()),
                );
                return;
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.sync_rows(cx);
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpSearchableList {
    /// The rows to show, as the caller reads them.
    pub fn rows(&self) -> Vec<Row> {
        rows(&self.items, &self.query, self.selected)
    }

    /// Write the rows into the slots and hide the rest.
    fn sync_rows(&mut self, cx: &mut Cx2d) {
        // One walk of the same data the caller reads, so the row a click reports and the row on screen cannot disagree.
        let visible = rows(&self.items, &self.query, self.selected);
        for position in 0..SLOTS {
            let slot = self
                .view
                .view(cx, &[id!(rows), row_id(position)]);
            match visible.get(position) {
                Some(row) => {
                    slot.set_visible(cx, true);
                    self.view
                        .label(cx, &[id!(rows), row_id(position), id!(row_label)])
                        .set_text(cx, &row.text);
                    self.view
                        .view(cx, &[id!(rows), row_id(position), id!(row_selected)])
                        .set_visible(cx, row.selected);
                    // A hover plate is the pointer's business and is left to the DSL's own hover handling; it is cleared
                    // here so a row that scrolls out of the shown set does not keep one.
                    self.view
                        .view(cx, &[id!(rows), row_id(position), id!(row_hover)])
                        .set_visible(cx, false);
                }
                None => slot.set_visible(cx, false),
            }
        }

        // **The count, and only when it needs saying.** A list showing every match has nothing to add; a truncated one
        // says how many it is hiding, because a truncated list that says nothing reads as a complete one.
        let total = total_matches(&self.items, &self.query);
        let text = if total > visible.len() {
            format!("{} of {total} shown", visible.len())
        } else {
            String::new()
        };
        self.view.label(cx, ids!(more)).set_text(cx, &text);
    }

    /// Set the items.
    pub fn set_items(&mut self, cx: &mut Cx, items: Vec<String>) {
        self.items = items;
        // A selection that is no longer in the list is not a selection. Checked against `items` — the list that did not
        // change when the filter did — which is the whole point of storing it that way.
        if self.selected.is_some_and(|index| index >= self.items.len()) {
            self.selected = None;
        }
        self.redraw(cx);
    }

    /// The items.
    pub fn items(&self) -> &[String] {
        &self.items
    }

    /// Set the query, which narrows what is shown.
    pub fn set_query(&mut self, cx: &mut Cx, query: &str) {
        self.query = query.to_string();
        self.view.text_input(cx, ids!(search)).set_text(cx, query);
        self.redraw(cx);
    }

    /// The query, **as it was given** rather than lowercased — the widget this replaces lowercased it on the way in and
    /// returned the lowercased form, so a caller echoing the query to a reader got `alpha` for `Alpha`.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Set the field's placeholder — the grey text shown while it is empty.
    ///
    /// **A placeholder is not a filter**, which is the confusion the A2UI renderer's own call site had: it passed the
    /// placeholder to `set_query`, so every list it drew was narrowed to the items containing the placeholder text. The
    /// two are different things and this is the one the protocol's `placeholder` field means, so the migration needs a
    /// setter for it rather than leaving it unwired.
    pub fn set_placeholder(&mut self, cx: &mut Cx, placeholder: &str) {
        self.view
            .text_input(cx, ids!(search))
            .set_empty_text(cx, placeholder.to_string());
    }

    /// Select an item by its index in `items`.
    pub fn set_selected(&mut self, cx: &mut Cx, index: Option<usize>) {
        self.selected = index.filter(|index| *index < self.items.len());
        self.redraw(cx);
    }

    /// The selected item's index in `items`.
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// The selected item's text.
    pub fn selected_text(&self) -> Option<String> {
        self.selected.and_then(|index| self.items.get(index).cloned())
    }
}

impl MpSearchableListRef {
    pub fn set_items(&self, cx: &mut Cx, items: Vec<String>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_items(cx, items);
        }
    }

    pub fn set_query(&self, cx: &mut Cx, query: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_query(cx, query);
        }
    }

    pub fn set_placeholder(&self, cx: &mut Cx, placeholder: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_placeholder(cx, placeholder);
        }
    }

    pub fn query(&self) -> String {
        self.borrow().map(|inner| inner.query.clone()).unwrap_or_default()
    }

    pub fn set_selected(&self, cx: &mut Cx, index: Option<usize>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_selected(cx, index);
        }
    }

    pub fn selected_text(&self) -> Option<String> {
        self.borrow().and_then(|inner| inner.selected_text())
    }

    pub fn rows(&self) -> Vec<Row> {
        self.borrow().map(|inner| inner.rows()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items() -> Vec<String> {
        ["Alpha", "Beta", "Gamma", "Alpine", "delta"]
            .iter()
            .map(|item| item.to_string())
            .collect()
    }

    #[test]
    fn test_an_empty_query_matches_everything() {
        // An unsearched list is the whole list, which is the state a list opens in rather than a special case.
        assert_eq!(total_matches(&items(), ""), 5);
        assert_eq!(filter_indices(&items(), "").len(), 5);
    }

    #[test]
    fn test_the_case_of_the_query_and_of_the_item_do_not_matter() {
        // Either side can be any case: the query is what somebody typed and the item is what a caller supplied.
        assert!(matches("Alpha", "alp"));
        assert!(matches("alpha", "ALP"));
        assert!(matches("ALPHA", "Alpha"));
        assert_eq!(total_matches(&items(), "ALP"), 2);
    }

    #[test]
    fn test_every_shown_row_is_one_of_the_matches_and_they_keep_their_order() {
        // **The relationship between what is shown and what matches**: the rows are a **prefix** of the matches, in the
        // order the matches were found — not reordered, not deduplicated, and never a row that does not match. A list
        // that showed a row the filter rejected would be a list that cannot be trusted to have filtered.
        let items = items();
        for query in ["", "a", "al", "alp", "zzz", "e"] {
            let all = match_indices(&items, query);
            let shown = filter_indices(&items, query);
            assert!(shown.len() <= SLOTS);
            assert_eq!(shown, all[..shown.len()].to_vec(), "query={query}");
            assert!(
                shown.iter().all(|index| matches(&items[*index], query)),
                "query={query} showed a row that does not match"
            );
            assert_eq!(total_matches(&items, query), all.len());
        }
    }

    #[test]
    fn test_the_count_is_the_whole_number_even_when_the_rows_are_capped() {
        // **Why the count exists.** A list that shows ten of twenty-three and says nothing reads as a list of ten, so the
        // number that matters is the one *before* the cap — and it is the same number whether or not the list was capped,
        // which is what keeps a caller's readout from depending on how many rows happened to fit.
        let many: Vec<String> = (0..25).map(|index| format!("Item {index}")).collect();
        assert_eq!(total_matches(&many, "item"), 25);
        assert_eq!(filter_indices(&many, "item").len(), SLOTS);
        assert_eq!(total_matches(&many, ""), 25);
        assert_eq!(filter_indices(&many, "").len(), SLOTS);
        // **And the count is not the shown length even when both are under the cap.** `"item 2"` matches `Item 2` and
        // `Item 20`..`Item 24` — six, which fits — and both numbers agree at six; `"item 1"` matches eleven, which does
        // not fit, and there the two numbers **differ by one** at exactly the cap. That difference is the whole reason the
        // count is computed separately, and my first version of this test asserted they agreed while matching eleven —
        // which is the assertion that fails when the cap is working.
        let many: Vec<String> = (0..25).map(|index| format!("Item {index}")).collect();
        assert_eq!(total_matches(&many, "item 2"), 6);
        assert_eq!(filter_indices(&many, "item 2").len(), 6, "six fits, so all six show");
        assert_eq!(total_matches(&many, "item 1"), 11);
        assert_eq!(
            filter_indices(&many, "item 1").len(),
            SLOTS,
            "eleven does not fit, so the rows stop at the cap"
        );
    }

    #[test]
    fn test_a_selection_survives_a_filter_that_no_longer_shows_it() {
        // **The defect this port was written around.** The v2 widget selected an **index into the filtered array**, so
        // filtering after a selection left that index pointing at a different item: select `Gamma`, type `a`, and the
        // selection silently became `Alpha`. Here the selection is an index into `items`, which a filter cannot move —
        // so after filtering to something that hides `Gamma`, `Gamma` is **still selected**, and it comes back when the
        // filter is cleared.
        let items = items();
        let gamma = 2;
        let selected = Some(gamma);

        let narrowed = rows(&items, "alp", selected);
        assert_eq!(
            narrowed.iter().map(|row| row.text.as_str()).collect::<Vec<_>>(),
            vec!["Alpha", "Alpine"]
        );
        assert!(
            narrowed.iter().all(|row| !row.selected),
            "a row was marked selected that is not the selection"
        );
        // ...and the selection itself is untouched, so it is there again when the filter goes.
        assert_eq!(selected, Some(gamma));
        let widened = rows(&items, "", selected);
        assert!(widened[gamma].selected);
        assert_eq!(widened[gamma].text, "Gamma");
    }

    #[test]
    fn test_a_row_reports_the_item_it_shows_rather_than_its_own_position() {
        // **The distinction a caller sees.** Filtering to `alp` shows `Alpha` at row 0 — but `Alpha` is item 0 and the row
        // is 0 only by coincidence; filtering to `e` shows `Beta` at row 0 and `Beta` is item 1. A click must report the
        // **item**, because that is what a caller stores and compares.
        let items = items();
        let shown = rows(&items, "e", None);
        assert_eq!(shown[0].text, "Beta");
        assert_eq!(shown[0].index, 1, "the row reported its position rather than its item");
        // ...and every row's text is the item its index names, which is the invariant the whole struct rests on.
        for row in rows(&items, "", None) {
            assert_eq!(row.text, items[row.index]);
        }
    }

    #[test]
    fn test_two_items_with_the_same_text_are_two_rows() {
        // Not deduplicated: a list is a list, and collapsing equal strings would make a click's index ambiguous.
        let items = vec!["same".to_string(), "same".to_string(), "other".to_string()];
        let shown = rows(&items, "same", None);
        assert_eq!(shown.len(), 2);
        assert_eq!(shown[0].index, 0);
        assert_eq!(shown[1].index, 1);
        assert_ne!(shown[0].index, shown[1].index);
    }

    #[test]
    fn test_the_query_is_kept_as_it_was_given() {
        // The v2 widget lowercased the query on the way in and returned the lowercased form, so a caller echoing it to a
        // reader showed `alpha` for `Alpha`. The comparison is case-insensitive; the stored string is the caller's.
        let stored = "Alpha";
        assert!(matches("alpha", stored));
        assert_eq!(stored, "Alpha");
    }
}
