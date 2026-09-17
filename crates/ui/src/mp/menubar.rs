//! `Menubar` — the in-window bar: a strip of titles that drop menus.
//!
//! ## What makes it a bar rather than a row of dropdowns
//!
//! **One menu being open changes what the others do.** Sliding the pointer onto a sibling title switches to it with no
//! click, and `left`/`right` cross between menus without leaving the keyboard. With nothing open, hovering does nothing at
//! all — a bar that dropped a menu at the mere passage of the mouse would be unusable, and that asymmetry
//! ([`Bar::hover_switch`]) is the whole difference between this and a row of buttons each holding its own popup.
//!
//! ## At most one menu is open, and the type says so
//!
//! [`Bar::open`] is one `Option<usize>`, not one popup per title. That is not a memory saving: it makes "exactly one menu
//! can be open" an invariant of the state rather than something every caller has to maintain, and it makes switching
//! between menus a single assignment with no way to leave two down.
//!
//! ## A fresh menu opens with nothing highlighted
//!
//! [`.open`] clears the cursor whenever the menu changes. Without that, the cursor's row number — which is a number into
//! *the menu it was in* — would point at whatever now sits at that index in a different menu, and the highlight would be on
//! a row nobody selected. The same for a submenu chain: a chain into one menu's rows names nothing in another's.
//!
//! ## `right` on a submenu row descends, and that is not a detail
//!
//! [`Bar::go_deeper`] tries the cursor's `descend` first and only crosses to the next menu if there was nothing to open.
//! A submenu row that swallowed `right` without opening would be **a dead key on the one row that has somewhere to go**.
//! Symmetrically `left` ascends a level before it crosses ([`Bar::go_shallower`]).
//!
//! ## What is not here
//!
//! The strip of titles and the anchored panel. `mp/menu_card.rs` has the panel; what remains for a widget is the row of
//! titles above it and the anchoring between them, neither of which is in this file — this is the state machine, and the
//! state machine is the half with the rules in it.

use makepad_widgets::*;

use crate::mp::menu::{items_at, Cursor, Item};

/// A title and the menu it drops.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Menu {
    pub title: String,
    pub items: Vec<Item>,
}

impl Menu {
    pub fn new(title: impl Into<String>, items: Vec<Item>) -> Self {
        Self {
            title: title.into(),
            items,
        }
    }

    /// The item a path names, where `path` is a row index per level, outermost first.
    ///
    /// This is what turns the path a [`MpMenubarAction`] carries back into the row the caller acts on — the bar reports an
    /// index and leaves dispatch with the caller, so a caller that wanted the item should not have to walk the same chain
    /// a second time and get it wrong.
    pub fn at(&self, path: &[usize]) -> Option<&Item> {
        let (row, above) = path.split_last()?;
        // `items_at` walks the chain above, which is exactly the part that must resolve to open-able submenus.
        let Some(level) = items_at(&self.items, above) else {
            return None;
        };
        level.get(*row)
    }

    /// Whether any row in the whole menu can be landed on.
    pub fn selectable(&self) -> bool {
        self.items.iter().any(Item::selectable)
    }
}

/// What a bar did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing happened — the key was not the bar's, or there was nowhere for it to go.
    None,
    /// The bar's own state changed: a menu opened, closed, or the cursor moved. Worth a repaint and nobody's business
    /// otherwise.
    Changed,
    /// A row was chosen. **Dispatch stays with the caller**, which is why this carries an index path rather than an action.
    Chose(usize, Vec<usize>),
}

impl Outcome {
    /// Whether anything happened at all.
    pub fn happened(&self) -> bool {
        !matches!(self, Outcome::None)
    }
}

/// The bar's state: which menu is down, and where the keyboard and pointer are inside it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Bar {
    menus: Vec<Menu>,
    /// Which title is down. One field for the whole bar rather than one popup each — see the module doc.
    open: Option<usize>,
    /// Where the keyboard and the pointer both are inside the open menu, and which of its submenus are down.
    cursor: Cursor,
}

impl Bar {
    pub fn new(menus: Vec<Menu>) -> Self {
        Self {
            menus,
            open: None,
            cursor: Cursor::default(),
        }
    }

    pub fn menus(&self) -> &[Menu] {
        &self.menus
    }

    /// Which menu is down, if any.
    pub fn open(&self) -> Option<usize> {
        self.open
    }

    pub fn is_open(&self) -> bool {
        self.open.is_some()
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    /// The open menu's rows, for a card to paint.
    pub fn open_items(&self) -> Option<&[Item]> {
        self.open.and_then(|menu| self.menus.get(menu)).map(|menu| menu.items.as_slice())
    }

    /// Open a menu, or close it if it is already the open one — what a click on a title does.
    ///
    /// A click on the open title closes the bar, which is the only way the pointer can dismiss it without choosing
    /// anything; a click on a sibling switches, which is the same thing as hovering there once the click has landed.
    pub fn toggle(&mut self, menu: usize) -> Outcome {
        if menu >= self.menus.len() {
            return Outcome::None;
        }
        if self.open == Some(menu) {
            return self.close();
        }
        self.show(menu)
    }

    /// Open `menu`, clearing the cursor.
    ///
    /// **Two defects were here and one test found the first.** It read
    /// `self.open != Some(menu) || !self.cursor_clear()`, which is wrong twice over:
    ///
    /// 1. `cursor_clear` answers *whether the cursor had anything*, so the negation made "the cursor was already clear"
    ///    count as a change — every no-op `show` reported a repaint, and crossing a single-menu bar never answered
    ///    `None`;
    /// 2. **and worse, `||` short-circuits.** When the menu really did switch, `cursor_clear()` was never called at all,
    ///    so the one place the cursor absolutely must be cleared — a switch, where its row number points into the menu it
    ///    came from — was the one place it could be skipped.
    ///
    /// The clear is now unconditional and the outcome is computed from two locals. A condition that must have a side
    /// effect does not belong on the right of a short-circuit.
    fn show(&mut self, menu: usize) -> Outcome {
        let switched = self.open != Some(menu);
        let had_cursor = self.cursor_clear();
        self.open = Some(menu);
        if switched || had_cursor {
            Outcome::Changed
        } else {
            Outcome::None
        }
    }

    fn cursor_clear(&mut self) -> bool {
        let had = self.cursor != Cursor::default();
        self.cursor.clear();
        had
    }

    /// **The rule that makes a bar a bar**: with one menu already down, the pointer crossing a sibling title opens it.
    ///
    /// With none down, hovering does nothing — a menubar that dropped a menu at the mere passage of the mouse would be
    /// unusable. Hovering the already-open title does nothing either, so the pointer cannot close what it is resting on.
    pub fn hover_switch(&mut self, menu: usize) -> Outcome {
        if menu >= self.menus.len() {
            return Outcome::None;
        }
        if self.is_open() && self.open != Some(menu) {
            return self.show(menu);
        }
        Outcome::None
    }

    /// Close the bar.
    pub fn close(&mut self) -> Outcome {
        let changed = self.open.is_some() || self.cursor != Cursor::default();
        self.open = None;
        // **Cleared before anything could paint, not after.** A submenu paints on its own layer, so a chain left behind
        // would hang there over the menu dissolving under it.
        self.cursor.clear();
        if changed {
            Outcome::Changed
        } else {
            Outcome::None
        }
    }

    /// `left`/`right` between titles: one menu down at a time, wrapping, with the cursor cleared.
    ///
    /// The cursor is cleared for the reason in the module doc: its row number is an index into **the menu it was in**, and
    /// a fresh menu has whatever now sits at that index.
    pub fn step_menu(&mut self, delta: isize) -> Outcome {
        let Some(menu) = self.open else {
            return Outcome::None;
        };
        let count = self.menus.len() as isize;
        if count == 0 {
            return Outcome::None;
        }
        self.show(((menu as isize + delta).rem_euclid(count)) as usize)
    }

    /// `up`/`down` within the open menu.
    pub fn step_item(&mut self, delta: isize) -> Outcome {
        let Some(menu) = self.open else {
            return Outcome::None;
        };
        let Some(items) = self.menus.get(menu) else {
            return Outcome::None;
        };
        let before = self.cursor.clone();
        let items = items.items.clone();
        self.cursor.step(&items, delta);
        if self.cursor != before {
            Outcome::Changed
        } else {
            Outcome::None
        }
    }

    /// `right`: into the submenu under the cursor if there is one, else across to the next menu.
    ///
    /// See the module doc on why the descend comes first.
    pub fn go_deeper(&mut self) -> Outcome {
        let Some(menu) = self.open else {
            return Outcome::None;
        };
        let Some(items) = self.menus.get(menu) else {
            return Outcome::None;
        };
        let items = items.items.clone();
        if self.cursor.descend(&items) {
            Outcome::Changed
        } else {
            self.step_menu(1)
        }
    }

    /// `left`: out of the innermost submenu, else back to the previous menu.
    pub fn go_shallower(&mut self) -> Outcome {
        if self.cursor.ascend() {
            Outcome::Changed
        } else {
            self.step_menu(-1)
        }
    }

    /// `enter`.
    ///
    /// Three answers, and each is the one the row deserves: a submenu row's `enter` **opens it, the way `right` does**;
    /// an action row is chosen; and with the bar closed, `enter` drops the first menu — so the key means "act on this
    /// control" either way, which is what makes a bar reachable by keyboard at all.
    pub fn confirm(&mut self) -> Outcome {
        match (self.open, self.cursor.path()) {
            (Some(menu), Some(path)) => {
                let Some(items) = self.menus.get(menu).map(|menu| menu.items.clone()) else {
                    return Outcome::None;
                };
                if self.cursor.descend(&items) {
                    Outcome::Changed
                } else {
                    self.choose(menu, path)
                }
            }
            (None, _) if !self.menus.is_empty() => self.show(0),
            _ => Outcome::None,
        }
    }

    /// The pointer or the keyboard chose a row in `menu`.
    pub fn choose(&mut self, menu: usize, path: Vec<usize>) -> Outcome {
        let chosen = Outcome::Chose(menu, path);
        self.close();
        chosen
    }

    /// `escape`: one level at a time, the bar itself last.
    ///
    /// So an `escape` with a submenu down closes the submenu and leaves you in the menu — the key means "back out" rather
    /// than "cancel everything", which is what a person pressing it expects.
    pub fn dismiss(&mut self) -> Outcome {
        if self.cursor.ascend() {
            Outcome::Changed
        } else {
            self.close()
        }
    }

    /// What the pointer did to the open menu.
    pub fn hit(&mut self, hit: &MpMenubarHit) -> Outcome {
        let Some(menu) = self.open else {
            return Outcome::None;
        };
        match hit {
            MpMenubarHit::Point(path) => {
                let Some(items) = self.menus.get(menu).map(|menu| menu.items.clone()) else {
                    return Outcome::None;
                };
                if self.cursor.point_at(&items, path) {
                    Outcome::Changed
                } else {
                    Outcome::None
                }
            }
            MpMenubarHit::Choose(path) => self.choose(menu, path.clone()),
            MpMenubarHit::Dismiss => self.close(),
        }
    }
}

/// What the pointer did, as the bar receives it from the card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MpMenubarHit {
    Point(Vec<usize>),
    Choose(Vec<usize>),
    Dismiss,
}

/// What the bar reports to the application.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum MpMenubarAction {
    /// A row was chosen. `path` is a row index per level, outermost first — one entry for a top-level row, two for a row in
    /// a submenu. [`Menu::at`] turns it back into the item.
    Selected { menu: usize, path: Vec<usize> },
    #[default]
    None,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn menus() -> Vec<Menu> {
        vec![
            Menu::new(
                "File",
                vec![
                    Item::action("New Window"),
                    Item::submenu("Open Recent", vec![Item::action("notes.md")]),
                    Item::Separator,
                    Item::action("Close").disabled(),
                ],
            ),
            Menu::new("Edit", vec![Item::action("Undo"), Item::action("Redo")]),
        ]
    }

    fn bar() -> Bar {
        Bar::new(menus())
    }

    /// Step into the open menu and onto its submenu row.
    ///
    /// **Two steps, and this helper exists because I wrote one step three separate times.** `step` entering a menu lands on
    /// the edge the direction comes from — row 0 going down — so it takes a second step to arrive at row 1. Written out at
    /// each call site that rule was forgotten over and over; written once it cannot be.
    fn onto_submenu_row(bar: &mut Bar) {
        bar.step_item(1);
        bar.step_item(1);
        assert_eq!(bar.cursor().row(), Some(1), "the helper did not arrive on the submenu row");
    }

    #[test]
    fn test_hovering_a_sibling_title_switches_only_while_a_menu_is_down() {
        // **The rule that makes a bar a bar, and the asymmetry in it.** Open first, then hover a sibling: that switches
        // with no click. Hover with nothing open: nothing at all, because a bar that dropped a menu at the mere passage of
        // the mouse would be unusable.
        let mut bar = bar();
        assert_eq!(bar.hover_switch(1), Outcome::None, "nothing was open");
        assert!(!bar.is_open());
        assert_eq!(bar.toggle(0), Outcome::Changed);
        assert_eq!(bar.hover_switch(1), Outcome::Changed, "a sibling takes over");
        assert_eq!(bar.open(), Some(1));
        // The already-open title is not a switch, so the pointer cannot close what it is resting on.
        assert_eq!(bar.hover_switch(1), Outcome::None);
        assert_eq!(bar.open(), Some(1));
        // And a title that is not there is not a switch either.
        assert_eq!(bar.hover_switch(99), Outcome::None);
        assert_eq!(bar.open(), Some(1));
    }

    #[test]
    fn test_a_click_on_the_open_title_closes_it_and_a_click_on_a_sibling_switches() {
        // The pointer's only way to dismiss the bar without choosing anything.
        let mut bar = bar();
        assert_eq!(bar.toggle(0), Outcome::Changed);
        assert_eq!(bar.toggle(0), Outcome::Changed, "clicking the open title closes it");
        assert!(!bar.is_open());
        assert_eq!(bar.toggle(0), Outcome::Changed);
        assert_eq!(bar.toggle(1), Outcome::Changed, "a sibling switches in one click");
        assert_eq!(bar.open(), Some(1));
        assert_eq!(bar.toggle(99), Outcome::None, "a title that is not there");
    }

    #[test]
    fn test_a_fresh_menu_opens_with_nothing_highlighted() {
        // **The stale-row rule.** A cursor's row number is an index into *the menu it was in*, so carrying it across a
        // switch would leave the highlight on whatever now sits at that index in a different menu — a row nobody selected.
        // This is asserted across every way the menu can change.
        let mut bar = bar();
        bar.toggle(0);
        onto_submenu_row(&mut bar);
        bar.go_deeper();
        assert!(bar.cursor().nested(), "in its submenu");

        // Crossing with the keyboard.
        assert_eq!(bar.step_menu(1), Outcome::Changed);
        assert_eq!(bar.open(), Some(1));
        assert_eq!(bar.cursor(), &Cursor::default(), "the chain survived the switch");

        // Switching by pointer, and **back to a menu that has a submenu** — Edit is two plain actions, so descending there
        // would cross instead. (Two steps, because the first enters at row 0; I got this wrong three times in this file.)
        bar.hover_switch(0);
        assert_eq!(bar.cursor(), &Cursor::default(), "the chain survived the switch");
        onto_submenu_row(&mut bar);
        bar.go_deeper();
        assert!(bar.cursor().nested(), "in its submenu");

        // Closing.
        bar.step_item(1);
        bar.close();
        assert_eq!(bar.cursor(), &Cursor::default());
    }

    #[test]
    fn test_left_and_right_cross_between_titles_and_wrap() {
        // "Without leaving the keyboard" — and wrapping, so one key held down walks the whole bar.
        let mut bar = bar();
        assert_eq!(bar.step_menu(1), Outcome::None, "nothing is open to cross from");
        bar.toggle(0);
        assert_eq!(bar.step_menu(1), Outcome::Changed);
        assert_eq!(bar.open(), Some(1));
        assert_eq!(bar.step_menu(1), Outcome::Changed);
        assert_eq!(bar.open(), Some(0), "past the last title wraps to the first");
        assert_eq!(bar.step_menu(-1), Outcome::Changed);
        assert_eq!(bar.open(), Some(1), "and back the other way");
        // One menu, so crossing stays put — and says nothing happened.
        let mut single = Bar::new(vec![Menu::new("File", vec![Item::action("New")])]);
        single.toggle(0);
        assert_eq!(single.step_menu(1), Outcome::None);
        assert_eq!(single.open(), Some(0));
    }

    #[test]
    fn test_right_descends_into_a_submenu_before_it_crosses() {
        // **Not a detail**: a submenu row that swallowed `right` without opening would be a dead key on the one row that
        // has somewhere to go.
        let mut bar = bar();
        bar.toggle(0);
        onto_submenu_row(&mut bar);
        assert_eq!(bar.go_deeper(), Outcome::Changed);
        assert!(bar.cursor().nested(), "it went in");
        assert_eq!(bar.open(), Some(0), "and stayed in the same menu");
        // From a row that opens nothing, `right` crosses. **`dismiss` lands back on the submenu row**, so the cursor has to
        // move to an action row first — going deeper again from there would descend, which is what the row above asserts.
        bar.dismiss();
        assert!(!bar.cursor().nested());
        assert_eq!(bar.cursor().row(), Some(1), "dismiss returned to the submenu row");
        bar.step_item(-1);
        assert_eq!(bar.cursor().row(), Some(0), "on New Window, an action");
        assert_eq!(bar.go_deeper(), Outcome::Changed);
        assert_eq!(bar.open(), Some(1), "from an action row it crosses");
    }

    #[test]
    fn test_left_leaves_the_innermost_submenu_before_it_crosses() {
        // The mirror of `go_deeper`, and where a menubar's `left` differs from a plain "previous menu".
        let mut bar = bar();
        bar.toggle(0);
        onto_submenu_row(&mut bar);
        bar.go_deeper();
        assert!(bar.cursor().nested());
        assert_eq!(bar.go_shallower(), Outcome::Changed);
        assert!(!bar.cursor().nested(), "it came out of the submenu");
        assert_eq!(bar.open(), Some(0), "and is still in the same menu");
        // Now it crosses.
        assert_eq!(bar.go_shallower(), Outcome::Changed);
        assert_eq!(bar.open(), Some(1), "out of the submenu, then across");
    }

    #[test]
    fn test_escape_closes_one_level_at_a_time_and_the_bar_last() {
        // "Back out" rather than "cancel everything", which is what a person pressing it expects.
        let mut bar = bar();
        bar.toggle(0);
        bar.step_item(1);
        bar.step_item(1);
        bar.go_deeper();
        assert!(bar.cursor().nested(), "in the submenu");
        assert_eq!(bar.dismiss(), Outcome::Changed);
        assert!(!bar.cursor().nested(), "the submenu closed");
        assert!(bar.is_open(), "and the bar did not");
        assert_eq!(bar.dismiss(), Outcome::Changed);
        assert!(!bar.is_open(), "the bar closed");
        assert_eq!(bar.dismiss(), Outcome::None, "and there was nothing left to close");
    }

    #[test]
    fn test_enter_on_a_closed_bar_drops_the_first_menu_so_the_key_always_means_act_on_this() {
        let mut bar = bar();
        assert_eq!(bar.confirm(), Outcome::Changed);
        assert_eq!(bar.open(), Some(0), "the first menu, not the last focused one");
        assert_eq!(bar.cursor().row(), None, "opened with nothing highlighted");
        // On an empty bar there is nothing to drop.
        let mut empty = Bar::new(vec![]);
        assert_eq!(empty.confirm(), Outcome::None);
    }

    #[test]
    fn test_enter_opens_a_submenu_row_and_chooses_an_action_row() {
        // `enter` and `right` agree about a submenu row; only an action row is a choice.
        let mut bar = bar();
        bar.toggle(0);
        onto_submenu_row(&mut bar);
        assert_eq!(bar.confirm(), Outcome::Changed, "a submenu row opens");
        assert!(bar.cursor().nested());
        assert_eq!(bar.open(), Some(0), "and the bar stayed open");
        // Now choose the row inside it.
        assert_eq!(bar.cursor().row(), Some(0));
        assert_eq!(bar.confirm(), Outcome::Chose(0, vec![1, 0]));
        assert!(!bar.is_open(), "choosing closes the bar");
    }

    #[test]
    fn test_choosing_reports_the_path_and_leaves_dispatch_alone() {
        // The index path is the report; **what the row means is the caller's**. `Menu::at` is what turns it back into the
        // item, so a caller does not walk the chain a second time and get it wrong.
        let mut bar = bar();
        bar.toggle(0);
        bar.step_item(1);
        assert_eq!(bar.confirm(), Outcome::Chose(0, vec![0]));
        let menus = menus();
        let item = menus[0].at(&[0]).expect("the path names a row");
        assert_eq!(item.label(), Some("New Window"));
        let nested = menus[0].at(&[1, 0]).expect("a two-level path");
        assert_eq!(nested.label(), Some("notes.md"));
        // A path that names nothing, in every way it can fail.
        assert!(menus[0].at(&[]).is_none(), "an empty path names no row");
        assert!(menus[0].at(&[99]).is_none(), "a row that is gone");
        assert!(menus[0].at(&[0, 0]).is_none(), "walking into an action row that opens nothing");
        assert!(menus[0].at(&[2, 0]).is_none(), "walking into a separator");
    }

    #[test]
    fn test_a_disabled_row_cannot_be_landed_on_so_it_can_never_be_chosen() {
        // The sample's last row is disabled, and the cursor steps over it — which is what keeps `enter` from reporting a row
        // whose whole meaning is "you cannot do this".
        let mut bar = bar();
        bar.toggle(0);
        // **Only stepping.** My first version called `confirm` in the loop too, which *chooses* whichever row the cursor
        // reached — and choosing closes the bar, so every later step had no menu to move in and the test was asserting
        // about a closed bar.
        let mut landed = Vec::new();
        for _ in 0..8 {
            bar.step_item(1);
            let row = bar.cursor().row();
            if let Some(row) = row {
                assert_ne!(row, 3, "the cursor landed on the disabled row");
                landed.push(row);
            }
        }
        // And it did walk the whole menu, so the assertion above was not vacuous.
        assert!(landed.contains(&0) && landed.contains(&1), "the walk did not cover the menu: {landed:?}");
        // `enter` from a walked-to row can never report the disabled one — which is the claim, rather than which row the
        // walk happened to stop on. (My first version asserted `Chose(0, vec![0])`, forgetting that a walk of eight steps
        // does not end where it started, and that a confirm on the submenu row descends instead of choosing.)
        if let Outcome::Chose(menu, path) = bar.confirm() {
            assert_eq!(menu, 0);
            assert_ne!(path, vec![3], "enter reported the disabled row");
        }
    }

    #[test]
    fn test_the_pointer_reports_through_the_same_cursor_the_keyboard_moves() {
        // One cursor for both devices — see `mp/menu.rs`. Feeding a `Hit` moves the same field `step_item` moves, so the
        // two cannot disagree about which row a submenu hangs off.
        let mut bar = bar();
        bar.toggle(0);
        assert_eq!(bar.hit(&MpMenubarHit::Point(vec![0])), Outcome::Changed);
        assert_eq!(bar.cursor().row(), Some(0), "an action row is live");
        // Pointing at the same row again is not a change.
        assert_eq!(bar.hit(&MpMenubarHit::Point(vec![0])), Outcome::None);
        // A separator is not a row.
        assert_eq!(bar.hit(&MpMenubarHit::Point(vec![2])), Outcome::None);
        assert_eq!(bar.cursor().row(), Some(0), "and the live row did not move");
        // **Row 1 is the submenu row, and pointing at it opens it and lights nothing inside** — the pointer is not in the
        // fresh panel, and lighting its first row would put the highlight somewhere the pointer is not.
        assert_eq!(bar.hit(&MpMenubarHit::Point(vec![1])), Outcome::Changed);
        assert!(bar.cursor().nested());
        assert_eq!(bar.cursor().row(), None);
        assert_eq!(bar.hit(&MpMenubarHit::Point(vec![1, 0])), Outcome::Changed);
        assert_eq!(bar.cursor().row(), Some(0), "and a row inside is live");
        // A choose reports and closes; a dismiss only closes.
        assert_eq!(
            bar.hit(&MpMenubarHit::Choose(vec![1, 0])),
            Outcome::Chose(0, vec![1, 0])
        );
        assert!(!bar.is_open());
        bar.toggle(0);
        assert_eq!(bar.hit(&MpMenubarHit::Dismiss), Outcome::Changed);
        assert!(!bar.is_open());
        // With nothing open, a hit is nobody's.
        assert_eq!(bar.hit(&MpMenubarHit::Point(vec![0])), Outcome::None);
    }

    #[test]
    fn test_stepping_in_a_menu_with_nothing_selectable_moves_nowhere() {
        // The bound in `mp/menu.rs`'s `next_selectable` seen from above: a menu of separators and disabled rows has nowhere
        // to land, and the bar says so rather than pretending it moved.
        let mut bar = Bar::new(vec![Menu::new(
            "Empty",
            vec![Item::Separator, Item::action("Off").disabled()],
        )]);
        bar.toggle(0);
        assert_eq!(bar.step_item(1), Outcome::None);
        assert_eq!(bar.cursor().row(), None);
        // And `enter` with nothing highlighted does nothing rather than choosing row 0 by accident.
        assert_eq!(bar.confirm(), Outcome::None);
        assert!(bar.is_open(), "and the bar is still open, waiting");
        assert!(!bar.menus()[0].selectable());
    }

    #[test]
    fn test_at_most_one_menu_is_open_whatever_is_pressed() {
        // **The invariant the type carries.** Every mutation is reached and the count is checked after each: if a second
        // popup were possible it would need a second field, and this is the test that would fail first.
        let mut bar = bar();
        let open_count = |bar: &Bar| bar.open().iter().count();
        for action in [
            Box::new(|b: &mut Bar| { let _ = b.toggle(0); }) as Box<dyn Fn(&mut Bar)>,
            Box::new(|b: &mut Bar| { let _ = b.confirm(); }),
            Box::new(|b: &mut Bar| { let _ = b.step_item(1); }),
            Box::new(|b: &mut Bar| { let _ = b.go_deeper(); }),
            Box::new(|b: &mut Bar| { let _ = b.step_menu(1); }),
            Box::new(|b: &mut Bar| { let _ = b.hover_switch(0); }),
            Box::new(|b: &mut Bar| { let _ = b.go_shallower(); }),
            Box::new(|b: &mut Bar| { let _ = b.dismiss(); }),
            Box::new(|b: &mut Bar| { let _ = b.hit(&MpMenubarHit::Dismiss); }),
            Box::new(|b: &mut Bar| { let _ = b.toggle(1); }),
        ] {
            action(&mut bar);
            assert!(open_count(&bar) <= 1, "more than one menu is open");
        }
    }
}
