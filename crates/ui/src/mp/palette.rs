//! `MpPalette` — the panel that turns [`crate::mp::search::rank`] into something a
//! person can use.
//!
//! ## The one contract that matters
//!
//! **A selection is reported as an index into the ORIGINAL item list, never into
//! the filtered view.** A caller matching on a filtered position would run the
//! wrong command the moment a query is typed — the whole point of a palette is that
//! the list shrinks under the reader's hands, so `ranked[2]` means a different
//! command after every keystroke. [`crate::mp::search::rank`] already returns
//! original indices for exactly this reason, and [`original`] is the one place the
//! mapping back is allowed to happen.
//!
//! ## The bug this module exists to prevent
//!
//! Everything else about a palette is composition — a field, a divider, a list —
//! and this crate already had all three. The part that is genuinely easy to get
//! wrong is what happens to the **cursor** when the view changes underneath it.
//!
//! Type `de`, the cursor is on `Delete` at position 2. Type one more character and
//! the list shrinks to four rows: position 2 is now a different command, and a
//! cursor that was left at position 2 has silently moved the reader's target. That
//! is a bug that runs the wrong command and leaves no trace in any log, so it is
//! [remap]ped by *identity* — the original index the cursor was on — and only falls
//! back to the first row when that row is genuinely gone.
//!
//! The maths here is pure, so it is tested exhaustively with no window at all,
//! which is the only way a claim about a cursor that follows a shrinking list can be
//! checked.
//!
//! ## A trap for the caller, found by running this
//!
//! [`crate::mp::list::MpList::select`] **emits the same `Selected` action a click
//! does**, and the action carries a *position*, not an identity. So a caller that
//! re-filters and re-selects in one pass reads its own highlight back as a
//! selection — and if the view shrank in between, the position it reads is stale and
//! can be out of range.
//!
//! That is not hypothetical. The run that proved the cursor survives a shrink ended
//! with a selection for a position of 1 after the view had gone from three rows to
//! one, which resolved to no command at all. **Filtering by identity is not enough**
//! — `None` is not equal to the current cursor either, so an out-of-range position
//! slips through a `selected != active` check. The only sound guard is to remember
//! the position you asked for and ignore an action that names it.

use makepad_widgets::*;

/// Where the cursor should land after the filtered view changes.
///
/// `previous` is an **original** index — the one the caller was told about — or
/// `None` for a palette that has not been moved yet. `ranked` is the new view, also
/// in original indices, as [`crate::mp::search::rank`] returns it.
///
/// Returns the cursor's new **position in `ranked`**. The row the reader was on
/// stays under them if it survived the query; otherwise the cursor goes to the top,
/// which is where the best match is.
pub fn remap(previous: Option<usize>, ranked: &[usize]) -> Option<usize> {
    if ranked.is_empty() {
        return None;
    }
    match previous {
        // By identity, not by position: this is the whole point of the module.
        Some(original) => ranked
            .iter()
            .position(|candidate| *candidate == original)
            .or(Some(0)),
        None => Some(0),
    }
}

/// Step the cursor by `delta`, wrapping at both ends.
///
/// Wrapping rather than clamping, because a palette's list is short and the reader
/// pressing Down on the last row means "back to the top", not "nothing happened" —
/// a clamp reads as a stuck key.
///
/// An empty view has no rows to move through, so the cursor is 0 and the caller is
/// expected to check [`original`] before acting on it.
pub fn step(active: usize, len: usize, delta: i32) -> usize {
    if len == 0 {
        return 0;
    }
    let len_i = len as i64;
    let moved = (active as i64 + delta as i64).rem_euclid(len_i);
    moved as usize
}

/// The original index a filtered position stands for, or `None` if there is no such
/// row.
///
/// The only sanctioned way to turn a cursor position back into something a caller
/// can act on.
pub fn original(ranked: &[usize], position: usize) -> Option<usize> {
    ranked.get(position).copied()
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    /// A command palette: a query field over a filtered list.
    ///
    /// A composition, not a widget with its own state — the same conclusion
    /// `mp/popover.rs` reached. The field, the divider and the list all exist, and
    /// the three things this module actually contributes are the cursor arithmetic
    /// above, the contract that a selection is an original index, and the shape of
    /// the panel.
    ///
    /// The panel is `surface_dialog` rather than `surface_card`, because a palette
    /// floats over the window and is the topmost thing on screen — the same rung
    /// every other overlay in the library uses, so a palette and a dialog read as
    /// the same distance from the page.
    mod.mp.MpPalette = View{
        width: 520
        height: Fit
        flow: Down
        spacing: 0

        draw_bg +: {
            color: surface_dialog
            border_color: border
            border_size: 1.0
            border_radius: 8.0
        }

        palette_field := mod.mp.MpTextInputSearch{
            width: Fill
            empty_text: "Type a command"
        }
        palette_sep := mod.mp.Divider{
            width: Fill
        }
        palette_list := mod.mp.MpMenu{
            width: Fill
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_an_empty_view_has_no_cursor() {
        assert_eq!(remap(Some(3), &[]), None);
        assert_eq!(remap(None, &[]), None);
    }

    #[test]
    fn test_a_fresh_palette_starts_on_the_first_row() {
        assert_eq!(remap(None, &[7, 2, 9]), Some(0));
    }

    #[test]
    fn test_the_cursor_follows_its_row_when_the_view_shrinks() {
        // THE case this module exists for. The reader is on original index 9, which
        // was at position 2; a query narrows the view and 9 is now at position 1.
        // A cursor left at position 2 would have silently moved the target onto a
        // different command.
        assert_eq!(remap(Some(9), &[7, 9]), Some(1));
    }

    #[test]
    fn test_the_cursor_follows_its_row_when_the_view_reorders() {
        // `rank` reorders as well as filters, so identity has to survive a move in
        // the other direction too.
        assert_eq!(remap(Some(7), &[9, 7, 2]), Some(1));
        assert_eq!(remap(Some(2), &[2, 7, 9]), Some(0));
    }

    #[test]
    fn test_the_cursor_falls_back_to_the_best_match_when_its_row_is_filtered_out() {
        // The row is genuinely gone, so the target has to move — and it moves to the
        // top, because rank's first row is its best answer to the new query.
        assert_eq!(remap(Some(9), &[7, 2]), Some(0));
    }

    #[test]
    fn test_a_cursor_on_the_only_row_survives_a_shrink_to_that_row() {
        assert_eq!(remap(Some(4), &[4]), Some(0));
    }

    #[test]
    fn test_stepping_down_wraps_past_the_last_row() {
        // A palette's list is short, so Down on the last row means "back to the
        // top"; a clamp reads as a stuck key.
        assert_eq!(step(2, 3, 1), 0);
        assert_eq!(step(0, 3, 1), 1);
    }

    #[test]
    fn test_stepping_up_wraps_past_the_first_row() {
        assert_eq!(step(0, 3, -1), 2);
        assert_eq!(step(2, 3, -1), 1);
    }

    #[test]
    fn test_stepping_an_empty_view_stays_at_zero() {
        assert_eq!(step(0, 0, 1), 0);
        assert_eq!(step(0, 0, -1), 0);
    }

    #[test]
    fn test_a_selection_reports_the_original_index_not_the_position() {
        // The contract the module doc leads with, stated as a test: the view is
        // [7, 2, 9] and the reader is on the second row, so the caller is told 2 —
        // the original index — and never 1, its position.
        let ranked = [7usize, 2, 9];
        assert_eq!(original(&ranked, 1), Some(2));
        assert_ne!(original(&ranked, 1), Some(1));
        assert_eq!(original(&ranked, 3), None);
    }

    #[test]
    fn test_a_full_typing_session_never_moves_the_target_silently() {
        // A walk through the real sequence, with the ORIGINAL command list:
        //   0 New Terminal, 1 Toggle Terminal, 2 Go to File, 3 Command Palette,
        //   4 Split Right, 5 Delete
        //
        // The reader types `de`, gets [5, 3], moves down once onto original 3, then
        // types `l` — which filters 3 out. The cursor must land on the top row of
        // the new view, and it must land on a row that is actually in it.
        let after_de = [5usize, 3];
        let cursor = remap(None, &after_de);
        assert_eq!(cursor, Some(0));
        let cursor = remap(cursor.map(|p| after_de[p]), &after_de);
        assert_eq!(cursor, Some(0), "re-ranking the same query holds the row");
        let cursor = step(cursor.unwrap(), after_de.len(), 1);
        assert_eq!(cursor, 1);
        assert_eq!(original(&after_de, 1), Some(3));

        let after_del = [5usize];
        let cursor = remap(Some(3), &after_del);
        assert_eq!(cursor, Some(0));
        assert_eq!(original(&after_del, 0), Some(5));

        // And the invariant that makes all of it safe: a remapped cursor is always
        // a position that exists.
        for view in [&after_de[..], &after_del[..]] {
            let p = remap(Some(3), view).unwrap();
            assert!(p < view.len());
            assert!(original(view, p).is_some());
        }
    }
}
