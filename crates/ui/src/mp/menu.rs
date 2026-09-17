//! A dropped menu — the [`Item`]s in it, and the [`Cursor`] that says where you are among them.
//!
//! ## One cursor for two input devices
//!
//! **The idea this whole module exists for.** A menu that tracked hover separately from the keyboard could have two rows
//! lit at once, and a submenu hanging off a row that is not the live one — the state would be two states, and nothing
//! keeps them in agreement. So there is one [`Cursor`]: pointer and keyboard both move it, and the only way the two can
//! disagree is if the caller invents a second one.
//!
//! Everything the pointer does arrives as a [`Hit`], **including dismissal** — and `Hit::Dismiss` exists because the
//! obvious alternative is wrong: an "out-click" on the card sees only the card's own bounds, so a click inside a submenu
//! would read as a click away and close the menu you were using.
//!
//! ## What the caller owns
//!
//! The open state. Only the caller knows what "open" means — a popover for one application, a field for another — so this
//! module never opens or closes anything: it answers where the cursor is and what a path names, and the caller decides
//! when to draw. That is also what makes the model testable without a window, which is what the tests below rely on.
//!
//! ## The row rules are not decoration
//!
//! - **Separators and disabled rows are stepped over, and both ends wrap.** Stepping counts landable rows, not rows;
//!   [`next_selectable`] is that and nothing else.
//! - **A submenu with nothing selectable inside it is not selectable itself.** A row that opens onto an empty panel is a
//!   row that looks available and leads nowhere, so `enabled: true` alone is not enough — see [`Item::selectable`].
//! - **A menu with nothing selectable lands on nothing**, and [`next_selectable`] gives up after one pass rather than
//!   wrapping forever. A loop that never terminates inside a key handler is a frozen window, and the guard is one bound.
//! - **A stale chain is truncated rather than trusted.** A menu can be rebuilt under a cursor that named a row which is
//!   gone; [`open_depth`] is how much of the chain still names something open-able, and painting and dismissal both stop
//!   there.
//!
//! ## What is not here yet
//!
//! The panel and the bar. `card()` — the popover that paints these rows and the submenus hanging off them — and a menubar
//! that switches between cards are the other half of this component, and they are not in this file: this is the model, and
//! the model is the half with the rules in it. Say so rather than letting the module look finished.

use makepad_widgets::*;

/// A row in a dropped menu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Item {
    /// A row you choose, which does something.
    Action {
        label: String,
        /// A second line under the label, for a row whose name does not say enough on its own. `None` keeps the row one
        /// line tall — nothing reserves space for a description the way the glyph gutter reserves space for an icon,
        /// because a row's two lines read as one block and a blank second line would read as a gap.
        description: Option<String>,
        /// The leading glyph, by name. A menu where no row has one keeps no room for it.
        icon: Option<String>,
        /// The accelerator to **print**. The binding itself is the application's, and a menu that showed a keystroke it
        /// did not own would be documenting a lie.
        keystroke: Option<String>,
        /// The choice the menu is currently on, marked with a trailing check.
        checked: bool,
        enabled: bool,
    },
    /// A row that drops a menu of its own. It carries no accelerator and nothing to check: the only thing choosing it
    /// does is open.
    Submenu {
        label: String,
        icon: Option<String>,
        enabled: bool,
        items: Vec<Item>,
    },
    /// A line between groups. Never selectable.
    Separator,
}

impl Item {
    /// A plain action row.
    pub fn action(label: impl Into<String>) -> Self {
        Item::Action {
            label: label.into(),
            description: None,
            icon: None,
            keystroke: None,
            checked: false,
            enabled: true,
        }
    }

    /// The same, with a second line.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        if let Item::Action { description: slot, .. } = &mut self {
            *slot = Some(description.into());
        }
        self
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        match &mut self {
            Item::Action { icon: slot, .. } | Item::Submenu { icon: slot, .. } => *slot = Some(icon.into()),
            Item::Separator => {}
        }
        self
    }

    pub fn with_keystroke(mut self, keystroke: impl Into<String>) -> Self {
        if let Item::Action { keystroke: slot, .. } = &mut self {
            *slot = Some(keystroke.into());
        }
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        if let Item::Action { checked: slot, .. } = &mut self {
            *slot = checked;
        }
        self
    }

    pub fn disabled(mut self) -> Self {
        match &mut self {
            Item::Action { enabled, .. } | Item::Submenu { enabled, .. } => *enabled = false,
            Item::Separator => {}
        }
        self
    }

    /// A row that drops `items`.
    pub fn submenu(label: impl Into<String>, items: Vec<Item>) -> Self {
        Item::Submenu {
            label: label.into(),
            icon: None,
            enabled: true,
            items,
        }
    }

    /// The row's text, or `None` for a separator.
    pub fn label(&self) -> Option<&str> {
        match self {
            Item::Action { label, .. } | Item::Submenu { label, .. } => Some(label),
            Item::Separator => None,
        }
    }

    /// Whether the cursor may land on this row.
    ///
    /// **A submenu is selectable only when something inside it is.** `enabled` says the row was not switched off, which is
    /// a different claim from "choosing it goes somewhere" — and a row that opens an empty panel looks available and leads
    /// nowhere. A one-line `enabled` check here would be the kind of subtle wrongness that only shows up when someone
    /// builds a menu from filtered data, which is exactly when it matters.
    pub fn selectable(&self) -> bool {
        match self {
            Item::Action { enabled, .. } => *enabled,
            Item::Submenu { enabled, items, .. } => *enabled && items.iter().any(Item::selectable),
            Item::Separator => false,
        }
    }

    /// The items this row drops, if it is an enabled submenu.
    ///
    /// Note what is **not** checked: whether the children are selectable. A disabled-but-present submenu still has a
    /// panel to paint if a caller asks for it; `selectable` is what keeps the cursor off such a row in the first place.
    pub fn opens(&self) -> Option<&[Item]> {
        match self {
            Item::Submenu {
                enabled: true,
                items,
                ..
            } => Some(items),
            _ => None,
        }
    }
}

/// The menu at `path`, or `None` if the path does not name an open-able chain.
pub fn items_at<'a>(items: &'a [Item], path: &[usize]) -> Option<&'a [Item]> {
    let mut level = items;
    for &row in path {
        level = level.get(row)?.opens()?;
    }
    Some(level)
}

/// How many levels of `open` still name an open-able submenu.
///
/// A menu can be rebuilt under a cursor that named a row which no longer exists, so a chain can be **stale**: the rows it
/// names are there, or were. Painting and dismissal both stop at this depth rather than trusting the chain, which is what
/// keeps a stale cursor from hanging a panel off nothing.
pub fn open_depth(items: &[Item], open: &[usize]) -> usize {
    let mut level = items;
    for (depth, &row) in open.iter().enumerate() {
        let Some(inner) = level.get(row).and_then(Item::opens) else {
            return depth;
        };
        level = inner;
    }
    open.len()
}

/// The row `delta` steps to, with separators and disabled rows stepped over and **both ends wrapping**.
///
/// `from` of `None` enters the menu at the edge the direction comes from — the top going down, the bottom going up, because
/// that is where a menu's first move should land.
///
/// `None` back means **nothing in the menu is selectable**, and the loop is bounded by the row count rather than trusting
/// the wrap to terminate. A menu of nothing but separators is a menu whose key handler would otherwise spin: probing each
/// row once and giving up is the whole guard, and it is one line.
pub fn next_selectable(items: &[Item], from: Option<usize>, delta: isize) -> Option<usize> {
    let count = items.len();
    if count == 0 {
        return None;
    }
    let step = if delta >= 0 { 1 } else { -1 };
    let wrap = |at: usize| (at as isize + step).rem_euclid(count as isize) as usize;
    // Entering, the first candidate is the edge itself; moving, it is the row after the one you are on. The `min` guards
    // a row index from a cursor that has outlived the menu it named.
    let mut at = match from {
        None if step > 0 => 0,
        None => count - 1,
        Some(at) => wrap(at.min(count - 1)),
    };
    for _ in 0..count {
        if items[at].selectable() {
            return Some(at);
        }
        at = wrap(at);
    }
    None
}

/// Where an open menu is being worked.
///
/// `open` is the chain of submenu rows currently down, outermost first, and `row` is the live row in the menu that chain
/// ends at. One cursor for both input devices; see the module doc.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Cursor {
    open: Vec<usize>,
    row: Option<usize>,
}

impl Cursor {
    /// The submenu rows currently down, outermost first.
    pub fn open(&self) -> &[usize] {
        &self.open
    }

    /// The live row in the innermost open menu.
    pub fn row(&self) -> Option<usize> {
        self.row
    }

    /// Whether a submenu is down.
    ///
    /// This is where a menubar's `left` closes a level instead of crossing to the previous menu — the difference between
    /// moving inside a menu and leaving it.
    pub fn nested(&self) -> bool {
        !self.open.is_empty()
    }

    /// The full path to the live row, outermost first.
    pub fn path(&self) -> Option<Vec<usize>> {
        let row = self.row?;
        let mut path = self.open.clone();
        path.push(row);
        Some(path)
    }

    /// Nothing down, nothing live — a menu opens here, and closes back to it.
    pub fn clear(&mut self) {
        self.open.clear();
        self.row = None;
    }

    /// The row lit in the panel at `depth`: the row holding the submenu open above the innermost panel, the live row in
    /// it, nothing below.
    ///
    /// One function rather than the caller subtracting, because this is where an off-by-one would light a row in the
    /// wrong panel — two rows lit at once, which is the failure the single cursor exists to prevent.
    pub fn lit(&self, depth: usize) -> Option<usize> {
        match depth.cmp(&self.open.len()) {
            std::cmp::Ordering::Less => Some(self.open[depth]),
            std::cmp::Ordering::Equal => self.row,
            std::cmp::Ordering::Greater => None,
        }
    }

    /// Move within the innermost open panel.
    pub fn step(&mut self, root: &[Item], delta: isize) {
        let Some(items) = items_at(root, &self.open) else {
            return;
        };
        self.row = next_selectable(items, self.row, delta);
    }

    /// Open the submenu under the live row and land on its first row.
    ///
    /// `false` when the live row is not one — which is a menubar's cue that `right` meant the next menu instead. The
    /// answer is not a convenience: it is the difference between "moved into" and "moved past".
    pub fn descend(&mut self, root: &[Item]) -> bool {
        let Some(row) = self.row else { return false };
        let Some(inner) = items_at(root, &self.open)
            .and_then(|items| items.get(row))
            .and_then(Item::opens)
        else {
            return false;
        };
        self.row = next_selectable(inner, None, 1);
        self.open.push(row);
        true
    }

    /// Close the innermost submenu, landing back on the row that opened it.
    ///
    /// `false` at the top level, where closing is the whole menu's to do — so a caller can let the key fall through rather
    /// than swallowing a `left` that means something else.
    pub fn ascend(&mut self) -> bool {
        match self.open.pop() {
            Some(row) => {
                self.row = Some(row);
                true
            }
            None => false,
        }
    }

    /// Put the cursor on the row `path` names, opening the chain above it and, if it is a submenu row, itself —
    /// **pointing at a submenu row is what opens it**. Nothing is lit inside the fresh panel until something moves into it.
    ///
    /// Answers whether that changed anything, because the pointer reports every move and only a change is worth a repaint.
    pub fn point_at(&mut self, root: &[Item], path: &[usize]) -> bool {
        let before = self.clone();
        let (above, row) = match path.split_last() {
            Some((row, above)) => (above, *row),
            None => {
                // An empty path is a pointer outside every panel, which is a dismissal's business rather than a move's.
                return false;
            }
        };
        // **The chain is validated rather than trusted**, and truncated to what still names an open-able submenu: a menu
        // can be rebuilt between two pointer moves, and a stale chain would otherwise open a panel off a row that is no
        // longer a submenu.
        let depth = open_depth(root, above);
        if depth < above.len() {
            return false;
        }
        let Some(items) = items_at(root, above) else {
            return false;
        };
        let Some(item) = items.get(row) else {
            return false;
        };
        if !item.selectable() {
            return false;
        }
        self.open = above.to_vec();
        if item.opens().is_some() {
            // Pointing at a submenu row opens it and lights nothing inside, so the pointer cannot light a row it is not on.
            self.open.push(row);
            self.row = None;
        } else {
            self.row = Some(row);
        }
        *self != before
    }
}

/// What the pointer did, for the caller to act on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Hit {
    /// The pointer is on this row, or a submenu row was clicked. Feed it to [`Cursor::point_at`], which is also what opens
    /// a submenu.
    Point(Vec<usize>),
    /// An action row was chosen.
    Choose(Vec<usize>),
    /// A press outside every panel of the open tree.
    Dismiss,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<Item> {
        vec![
            Item::action("New"),
            Item::action("Open"),
            Item::Separator,
            Item::submenu(
                "Share",
                vec![
                    Item::action("Copy link"),
                    Item::Separator,
                    Item::action("Email"),
                ],
            ),
            Item::action("Disabled").disabled(),
            Item::action("Quit"),
        ]
    }

    #[test]
    fn test_a_separator_and_a_disabled_row_are_stepped_over() {
        // **The rule that makes stepping count landable rows rather than rows.** From "Open" (row 1), a step down goes to
        // "Share" (3) — not to the separator at 2; from "Share" it goes to "Quit" (5) — not to the disabled row at 4.
        let items = sample();
        assert_eq!(next_selectable(&items, Some(1), 1), Some(3));
        assert_eq!(next_selectable(&items, Some(3), 1), Some(5));
        assert_eq!(next_selectable(&items, Some(5), -1), Some(3));
        assert_eq!(next_selectable(&items, Some(3), -1), Some(1));
    }

    #[test]
    fn test_both_ends_wrap_and_an_entry_lands_at_the_edge_it_came_from() {
        // A menu's first move going down lands at the top and going up lands at the bottom: entering from nowhere should
        // not skip the first row. And stepping past either end wraps rather than stopping, which is what makes a menu
        // walkable with one key held down.
        let items = sample();
        assert_eq!(next_selectable(&items, None, 1), Some(0), "entering downward lands at the top");
        assert_eq!(next_selectable(&items, None, -1), Some(5), "entering upward lands at the bottom");
        assert_eq!(next_selectable(&items, Some(5), 1), Some(0), "past the end wraps");
        assert_eq!(next_selectable(&items, Some(0), -1), Some(5), "before the start wraps");
        // Zero counts as downward, since the sign is what chooses the direction.
        assert_eq!(next_selectable(&items, Some(0), 0), Some(1));
    }

    #[test]
    fn test_a_menu_with_nothing_selectable_lands_on_nothing_rather_than_looping() {
        // **The bound that keeps a key handler from freezing the window.** Every row here is unselectable, so probing
        // each of them once and giving up is the only ending. A version that trusted the wrap to terminate would spin.
        let nothing = vec![Item::Separator, Item::action("off").disabled(), Item::Separator];
        assert_eq!(next_selectable(&nothing, None, 1), None);
        assert_eq!(next_selectable(&nothing, Some(0), 1), None);
        assert_eq!(next_selectable(&[], None, 1), None, "an empty menu, same answer");
    }

    #[test]
    fn test_a_submenu_with_nothing_selectable_inside_is_not_selectable_itself() {
        // `enabled: true` says the row was not switched off, which is **not** the same claim as "choosing it goes
        // somewhere". A row that opens an empty panel looks available and leads nowhere, and building a menu from filtered
        // data is exactly how you get one.
        let empty = Item::submenu("Share", vec![Item::Separator]);
        let all_disabled = Item::submenu("Share", vec![Item::action("Email").disabled()]);
        let one_usable = Item::submenu("Share", vec![Item::Separator, Item::action("Email")]);
        assert!(!empty.selectable(), "a panel with nothing in it is not a destination");
        assert!(!all_disabled.selectable(), "nor is one where everything is off");
        assert!(one_usable.selectable(), "one usable row is enough");
        // A disabled submenu is not selectable however good its contents are.
        assert!(!Item::submenu("Share", vec![Item::action("Email")]).disabled().selectable());
        // ...and it has no panel to open, either.
        assert!(Item::submenu("Share", vec![Item::action("Email")]).disabled().opens().is_none());
        assert!(one_usable.opens().is_some());
    }

    #[test]
    fn test_a_row_index_from_a_cursor_that_outlived_its_menu_does_not_panic() {
        // A menu can be replaced between a cursor's move and the next key. The clamp is one `min` and it is the difference
        // between an unreachable row and an index out of bounds.
        let items = sample();
        assert_eq!(next_selectable(&items, Some(99), 1), Some(0), "a stale index wrapped from the end");
        // **Upward from a stale index lands on the last row *before* the clamp**, not on the clamp itself: the clamp
        // produces row 5 and then the step moves off it to 4, which is disabled, so it continues to 3. I first wrote
        // `Some(5)` here, reading the clamp as the answer — the clamp is a guard against an out-of-bounds index, not a
        // destination.
        assert_eq!(next_selectable(&items, Some(99), -1), Some(3));
        // And `point_at` refuses a path whose row is gone rather than trusting it.
        let mut cursor = Cursor::default();
        assert!(!cursor.point_at(&items, &[99]));
        assert_eq!(cursor, Cursor::default(), "a refused move changed something");
    }

    #[test]
    fn test_the_lit_row_is_the_open_row_above_and_the_live_row_inside() {
        // One function rather than the caller subtracting, because this is where an off-by-one lights a row in the wrong
        // panel — two rows lit at once, which is the failure the single cursor exists to prevent.
        let mut cursor = Cursor::default();
        cursor.row = Some(1);
        assert_eq!(cursor.lit(0), Some(1));
        assert_eq!(cursor.lit(1), None, "nothing below the innermost panel");
        cursor.open = vec![3];
        cursor.row = Some(0);
        assert_eq!(cursor.lit(0), Some(3), "the row holding the panel open");
        assert_eq!(cursor.lit(1), Some(0), "the live row inside it");
        assert_eq!(cursor.lit(2), None);
    }

    #[test]
    fn test_descending_and_ascending_land_where_the_cursor_should_be() {
        let items = sample();
        let mut cursor = Cursor::default();
        // **Three steps, not two.** The first enters at row 0 and each step after it moves one selectable row: 0 → 1
        // (Open) → 3 (Share). I wrote two steps and expected row 3, counting the entry as a move.
        cursor.step(&items, 1);
        cursor.step(&items, 1);
        assert_eq!(cursor.row(), Some(1), "on Open");
        cursor.step(&items, 1);
        assert_eq!(cursor.row(), Some(3), "on Share");
        assert!(!cursor.nested());

        assert!(cursor.descend(&items), "a submenu row descends");
        assert!(cursor.nested());
        assert_eq!(cursor.open(), &[3]);
        // **It lands on the first *selectable* row inside**, not row 0 — the submenu's own first row is a real one here,
        // but the point is that the inner step uses the same rule as the outer.
        assert_eq!(cursor.row(), Some(0));
        assert_eq!(cursor.path(), Some(vec![3, 0]));

        assert!(cursor.ascend(), "and comes back to the row that opened it");
        assert_eq!(cursor.row(), Some(3));
        assert!(!cursor.nested());
        assert!(!cursor.ascend(), "at the top level, closing is the whole menu's to do");
    }

    #[test]
    fn test_descending_off_a_row_that_is_not_a_submenu_says_so() {
        // The answer is not a convenience: it is the difference between "moved into" and "moved past", and a menubar reads
        // it to decide whether `right` meant the next menu.
        let items = sample();
        let mut cursor = Cursor::default();
        cursor.step(&items, 1);
        assert_eq!(cursor.row(), Some(0), "on New, an action");
        assert!(!cursor.descend(&items));
        assert!(!cursor.nested(), "a refused descend opened something");
        // And with nothing live at all.
        let mut empty = Cursor::default();
        assert!(!empty.descend(&items));
    }

    #[test]
    fn test_pointing_at_a_submenu_row_opens_it_and_lights_nothing_inside() {
        // **Pointing at a submenu row is what opens it, and nothing inside is lit** — because the pointer is not inside it.
        // Lighting the first row there would put the highlight under a pointer that is somewhere else, and the next `down`
        // would choose a row the user never aimed at.
        let items = sample();
        let mut cursor = Cursor::default();
        assert!(cursor.point_at(&items, &[3]));
        assert_eq!(cursor.open(), &[3]);
        assert_eq!(cursor.row(), None, "a fresh panel lights nothing");
        assert_eq!(cursor.lit(0), Some(3), "the row holding it open is lit");
        assert_eq!(cursor.lit(1), None);
        assert_eq!(cursor.path(), None);

        // Pointing at a row inside closes the submenu into a plain live row.
        assert!(cursor.point_at(&items, &[3, 0]));
        assert_eq!(cursor.open(), &[3]);
        assert_eq!(cursor.row(), Some(0));
        assert_eq!(cursor.path(), Some(vec![3, 0]));
    }

    #[test]
    fn test_pointing_answers_whether_anything_changed() {
        // The pointer reports **every** move, and only a change is worth a repaint — so the answer is the repaint decision.
        let items = sample();
        let mut cursor = Cursor::default();
        assert!(cursor.point_at(&items, &[3, 0]), "the first move changed it");
        assert!(!cursor.point_at(&items, &[3, 0]), "the same move again did not");
        // **Row 1 of the submenu is its Separator**, so pointing there is refused and nothing changes; row 2 is "Email".
        // I first wrote `&[3, 1]` expecting a change, forgetting that the sample's submenu has a separator in the middle.
        assert!(!cursor.point_at(&items, &[3, 1]), "a separator inside a submenu is still a separator");
        assert!(cursor.point_at(&items, &[3, 2]), "a different row did");
    }

    #[test]
    fn test_pointing_at_a_separator_or_a_disabled_row_is_refused() {
        // A pointer resting on a separator is a pointer that is not on a row. Accepting it would leave the previously live
        // row lit while the pointer is elsewhere — the two-lights problem, arriving through the pointer instead of the
        // keyboard.
        let items = sample();
        let mut cursor = Cursor::default();
        cursor.point_at(&items, &[1]);
        assert_eq!(cursor.row(), Some(1));
        assert!(!cursor.point_at(&items, &[2]), "a separator is not a row");
        assert_eq!(cursor.row(), Some(1), "and the live row did not move");
        assert!(!cursor.point_at(&items, &[4]), "nor is a disabled row");
        assert_eq!(cursor.row(), Some(1));
        assert!(!cursor.point_at(&items, &[]), "an empty path is a dismissal's business, not a move's");
    }

    #[test]
    fn test_a_stale_chain_is_truncated_rather_than_trusted() {
        // A menu rebuilt under a cursor that named rows which are gone. `open_depth` reports how much of the chain still
        // names something open-able, and **that is what keeps a stale cursor from hanging a panel off nothing**.
        let items = sample();
        assert_eq!(open_depth(&items, &[]), 0);
        assert_eq!(open_depth(&items, &[3]), 1, "the whole chain is live");
        assert_eq!(open_depth(&items, &[3, 0]), 1, "row 0 of the submenu opens nothing, so the chain stops there");
        assert_eq!(open_depth(&items, &[1]), 0, "an action row opens nothing");
        assert_eq!(open_depth(&items, &[99]), 0, "a row that is gone");
        // And `items_at` agrees with it, which is the point of having both.
        assert!(items_at(&items, &[3]).is_some());
        assert!(items_at(&items, &[3, 0]).is_none());
        assert!(items_at(&items, &[1]).is_none());
    }

    #[test]
    fn test_clearing_returns_the_cursor_to_where_a_menu_opens() {
        let items = sample();
        let mut cursor = Cursor::default();
        cursor.point_at(&items, &[3, 0]);
        assert!(cursor.nested());
        cursor.clear();
        assert_eq!(cursor, Cursor::default());
        assert!(!cursor.nested());
        assert_eq!(cursor.path(), None);
        assert_eq!(cursor.lit(0), None);
    }

    #[test]
    fn test_the_row_builders_set_only_the_field_they_name() {
        // A builder chain is easy to get subtly wrong when the variants share field names — `with_icon` treats an action and
        // a submenu alike, and the rest touch one variant. Asserted because a builder that silently did nothing would make
        // a menu look like the caller never asked.
        let row = Item::action("Open")
            .with_description("an existing file")
            .with_icon("folder")
            .with_keystroke("cmd+o")
            .checked(true);
        match &row {
            Item::Action {
                label,
                description,
                icon,
                keystroke,
                checked,
                enabled,
            } => {
                assert_eq!(label, "Open");
                assert_eq!(description.as_deref(), Some("an existing file"));
                assert_eq!(icon.as_deref(), Some("folder"));
                assert_eq!(keystroke.as_deref(), Some("cmd+o"));
                assert!(*checked);
                assert!(*enabled);
            }
            other => panic!("a builder changed the variant: {other:?}"),
        }
        // The submenu builder takes an icon and nothing else, and a keystroke aimed at it is a no-op rather than a panic.
        let sub = Item::submenu("Share", vec![Item::action("Email")]).with_icon("share").with_keystroke("cmd+s");
        match &sub {
            Item::Submenu { icon, .. } => assert_eq!(icon.as_deref(), Some("share")),
            other => panic!("wrong variant: {other:?}"),
        }
        // A separator has no label and no builders apply.
        assert_eq!(Item::Separator.label(), None);
        assert_eq!(Item::Separator.with_icon("x").label(), None);
    }
}
