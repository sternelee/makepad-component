//! `MpFocus` — `tab` and `shift-tab` between controls.
//!
//! ## What Makepad does not have
//!
//! Makepad has a **single** key-focus `Area` on `Cx` (`set_key_focus` /
//! `has_key_focus`) and no traversal at all. This module turns traversal on: focusable
//! controls call [`register`] from their `draw_walk`, and the app root calls
//! [`handle_key`] from its `handle_event`.
//!
//! ## Order is paint order
//!
//! The registry is keyed by the widget's stable [`WidgetUid`] and holds entries in
//! registration order, which is tree order, which is the order controls are painted in.
//! Nothing is numbered by hand, and **inserting a control in the middle of a form does not
//! renumber the rest** — which is the failure mode that makes HTML `tabindex` a liability.
//!
//! Each control re-registers its fresh `Area` every frame, because a Makepad area carries a
//! per-redraw id; the registry keyed by uid means the map never grows past the number of
//! distinct controls.
//!
//! ## The parameter that makes a stale entry impossible
//!
//! [`register`] takes `disabled` rather than leaving the caller to skip the call, and the
//! reason is a bug this module was written to fix. The port's first version put the guard at
//! the call site — `if !self.disabled { focus::register(..) }` — and both call sites honoured
//! it. What that misses is the control that **becomes** disabled: it stops calling
//! `register`, so nothing removes it, and its `Area` is still valid because the control is
//! still drawn. So it stays in the tab order and **tab lands on a control the reader cannot
//! use**.
//!
//! Taking the flag means a disabled control is **removed** from the order when it draws
//! instead of merely not being added, so the stale entry cannot exist. The distinction is
//! the same one [`crate::mp::list`] makes about a value that is no longer in its list:
//! dropping says "not available", and not-adding says nothing about what was there before.
//!
//! ## A surface can claim `tab`
//!
//! [`claim_tab`] stands this traversal down, for a surface where `tab` belongs to something
//! else — a document that nests a list, an editor that inserts indentation. The claim
//! **persists until released** rather than being per-event, because it describes who owns the
//! key while that surface has focus, not one keystroke.
//!
//! ## Using it
//!
//! ```ignore
//! // in the app's root Widget::handle_event:
//! focus::handle_key(cx, event);
//! ```

use makepad_widgets::*;

/// The tab order — control `WidgetUid` → its most recent `Area`.
#[derive(Default)]
pub struct FocusRegistry {
    entries: Vec<(WidgetUid, Area)>,
    /// Whether a surface has taken `tab` for itself.
    claimed: bool,
}

impl FocusRegistry {
    /// Insert a control, or overwrite its slot. Returns `true` when the slot is new.
    pub fn insert(&mut self, uid: WidgetUid, area: Area) -> bool {
        if let Some(position) = self.position_of_uid(uid) {
            self.entries[position].1 = area;
            return false;
        }
        self.entries.push((uid, area));
        true
    }

    /// Take a control out of the order.
    ///
    /// The half the call-site guard could not express: a control that is drawn but not
    /// usable has to be **removed**, because nothing else will remove it.
    pub fn remove(&mut self, uid: WidgetUid) -> bool {
        let before = self.entries.len();
        self.entries.retain(|(u, _)| *u != uid);
        self.entries.len() != before
    }

    /// Whether the order contains this control. For tests and for a caller asking "will tab
    /// reach this".
    pub fn contains(&self, uid: WidgetUid) -> bool {
        self.position_of_uid(uid).is_some()
    }

    fn position_of_uid(&self, uid: WidgetUid) -> Option<usize> {
        self.entries.iter().position(|(u, _)| *u == uid)
    }

    /// The uids in tab (registration) order.
    pub fn uid_order(&self) -> Vec<WidgetUid> {
        self.entries.iter().map(|(u, _)| *u).collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Register a control's area under its stable uid into the tab order.
///
/// Paint order is traversal order: Makepad walks children in tree order, so a control drawn
/// earlier is tabbed into earlier. A control that is hidden does not `draw_walk` at all, so it
/// stops re-registering and its stale entry is dropped by the validity sweep; a control that
/// is **disabled** still draws, which is why `disabled` is a parameter here rather than a
/// guard at the call site. See the module doc.
pub fn register(cx: &mut Cx2d, uid: WidgetUid, area: Area, disabled: bool) {
    let registry = cx.global::<FocusRegistry>();
    if disabled || area.is_empty() {
        // Removed rather than skipped, so a control that became disabled leaves the order
        // at once instead of keeping the entry it had while it was enabled.
        registry.remove(uid);
        return;
    }
    registry.insert(uid, area);
}

/// Take `tab` away from this traversal until [`release_tab`].
///
/// For a surface where `tab` means something else — a document that nests a list, an editor
/// that inserts indentation. Persists rather than being per-event, because it describes who
/// owns the key while that surface holds focus.
pub fn claim_tab(cx: &mut Cx) {
    cx.global::<FocusRegistry>().claimed = true;
}

/// Give `tab` back to this traversal.
pub fn release_tab(cx: &mut Cx) {
    cx.global::<FocusRegistry>().claimed = false;
}

/// Whether a surface currently owns `tab`.
pub fn tab_claimed(cx: &mut Cx) -> bool {
    cx.global::<FocusRegistry>().claimed
}

/// The registry's areas in tab order, with slots whose area has gone stale dropped.
///
/// A control that was removed, or that became hidden, stops re-registering; its `Area` stops
/// being valid; and dropping it here is what keeps `tab` from landing on a control that is no
/// longer on screen.
fn current_order(cx: &mut Cx) -> Vec<Area> {
    // Snapshotted out first so `is_valid` (which borrows `cx` immutably) can be called
    // without fighting the mutable borrow on the global. One pass, not one pass per entry:
    // the first version collected the valid uids into a `Vec` and then re-scanned the entries
    // for each of them.
    let snapshot: Vec<(WidgetUid, Area)> = cx.global::<FocusRegistry>().entries.clone();
    let valid: Vec<(WidgetUid, Area)> = snapshot
        .into_iter()
        .filter(|(_, area)| area.is_valid(cx))
        .collect();
    let registry = cx.global::<FocusRegistry>();
    registry.entries = valid;
    registry.entries.iter().map(|(_, area)| *area).collect()
}

/// Index of the slot to focus after `current`, wrapping.
///
/// A `current` that is not in the order (nothing focused yet) resolves to the **last** index,
/// so a plain `tab` from nothing starts at the first control.
fn next_index(order_len: usize, current: Option<usize>) -> usize {
    if order_len == 0 {
        return 0;
    }
    let index = current.unwrap_or(order_len.saturating_sub(1));
    (index + 1) % order_len
}

/// Index of the slot to focus before `current`, wrapping.
///
/// A `current` that is not in the order resolves to `0`, so `shift-tab` from nothing starts at
/// the last control.
fn prev_index(order_len: usize, current: Option<usize>) -> usize {
    if order_len == 0 {
        return 0;
    }
    let index = current.unwrap_or(0);
    (index + order_len - 1) % order_len
}

/// Move focus to the next control, wrapping. `None` when there is nothing to focus.
pub fn focus_next(cx: &mut Cx) -> Option<Area> {
    let order = current_order(cx);
    if order.is_empty() {
        return None;
    }
    let current = cx.key_focus();
    let position = order.iter().position(|area| *area == current);
    let next = order[next_index(order.len(), position)];
    cx.set_key_focus(next);
    cx.redraw_all();
    Some(next)
}

/// Move focus to the previous control, wrapping. `None` when there is nothing to focus.
pub fn focus_prev(cx: &mut Cx) -> Option<Area> {
    let order = current_order(cx);
    if order.is_empty() {
        return None;
    }
    let current = cx.key_focus();
    let position = order.iter().position(|area| *area == current);
    let prev = order[prev_index(order.len(), position)];
    cx.set_key_focus(prev);
    cx.redraw_all();
    Some(prev)
}

/// Handle a `tab` / `shift-tab` key event for the whole window. Call from the app's root
/// `handle_event`. Returns `true` when the event was consumed.
///
/// Returns `false` — consuming nothing — while a surface has [claimed](claim_tab) `tab`, so a
/// document that nests a list keeps its indentation key.
pub fn handle_key(cx: &mut Cx, event: &Event) -> bool {
    if let Event::KeyDown(ke) = event {
        if ke.key_code == KeyCode::Tab {
            if tab_claimed(cx) {
                return false;
            }
            if !ke.is_repeat {
                if ke.modifiers.shift {
                    focus_prev(cx);
                } else {
                    focus_next(cx);
                }
            }
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uid(n: u64) -> WidgetUid {
        WidgetUid(n)
    }

    #[test]
    fn test_insert_is_keyed_by_uid_so_the_map_does_not_grow() {
        // A control re-registers its fresh area every frame; without the uid key the map
        // would grow once per frame per control.
        let mut registry = FocusRegistry::default();
        assert!(registry.insert(uid(1), Area::Empty));
        registry.insert(uid(2), Area::Empty);
        assert!(!registry.insert(uid(1), Area::Empty), "an existing slot");
        assert_eq!(registry.uid_order(), vec![uid(1), uid(2)]);
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_re_registering_never_reorders() {
        // Tab order is paint order, so a control that repaints does not move in the order.
        let mut registry = FocusRegistry::default();
        registry.insert(uid(3), Area::Empty);
        registry.insert(uid(1), Area::Empty);
        registry.insert(uid(2), Area::Empty);
        assert_eq!(registry.uid_order(), vec![uid(3), uid(1), uid(2)]);
        registry.insert(uid(1), Area::Empty);
        assert_eq!(registry.uid_order(), vec![uid(3), uid(1), uid(2)]);
    }

    #[test]
    fn test_removing_a_control_takes_it_out_of_the_order() {
        let mut registry = FocusRegistry::default();
        registry.insert(uid(1), Area::Empty);
        registry.insert(uid(2), Area::Empty);
        assert!(registry.remove(uid(1)));
        assert_eq!(registry.uid_order(), vec![uid(2)]);
        assert!(!registry.remove(uid(1)), "removing twice is not a change");
        assert!(!registry.contains(uid(1)));
        assert!(registry.contains(uid(2)));
    }

    #[test]
    fn test_a_control_that_becomes_disabled_leaves_the_order() {
        // **The bug this module was written to fix.** A control that becomes disabled still
        // draws, so a call-site guard (`if !disabled { register() }`) leaves the entry it had
        // while it was enabled — and its `Area` is still valid, so the validity sweep does
        // not drop it either. Tab then lands on a control the reader cannot use.
        //
        // Reaching `register` is what this models: the guard is *inside* it now.
        let mut registry = FocusRegistry::default();
        registry.insert(uid(7), Area::Empty);
        assert!(registry.contains(uid(7)));
        // It draws again, this time disabled. The entry must go.
        registry.remove(uid(7));
        assert!(
            !registry.contains(uid(7)),
            "a disabled control stayed in the tab order"
        );
        assert!(registry.is_empty());
    }

    #[test]
    fn test_the_two_orders_of_a_control_drawing_enabled_and_disabled_agree() {
        // The property behind the test above, stated as a comparison: whether a control
        // becomes disabled before or after it was ever enabled, the order must be the same.
        let mut was_enabled = FocusRegistry::default();
        was_enabled.insert(uid(1), Area::Empty);
        was_enabled.insert(uid(2), Area::Empty);
        was_enabled.remove(uid(2)); // now disabled

        let mut never_enabled = FocusRegistry::default();
        never_enabled.insert(uid(1), Area::Empty);

        assert_eq!(was_enabled.uid_order(), never_enabled.uid_order());
    }

    #[test]
    fn test_a_claim_stands_the_traversal_down() {
        // Bezel's `CLAIMS_TAB`, as a flag: `tab` belongs to a nested surface while it has
        // focus, so this traversal consumes nothing and the key falls through.
        let mut registry = FocusRegistry::default();
        registry.insert(uid(1), Area::Empty);
        assert!(!registry.claimed);
        registry.claimed = true;
        assert!(registry.claimed);
        registry.claimed = false;
        assert!(!registry.claimed);
    }

    #[test]
    fn test_next_and_prev_wrap_around() {
        let next = |current: Option<usize>| next_index(3, current);
        let prev = |current: Option<usize>| prev_index(3, current);
        assert_eq!(next(Some(0)), 1);
        assert_eq!(next(Some(2)), 0, "past the last wraps to the first");
        assert_eq!(prev(Some(1)), 0);
        assert_eq!(prev(Some(0)), 2, "before the first wraps to the last");
        // Nothing focused: `tab` starts at the first and `shift-tab` at the last.
        assert_eq!(next(None), 0);
        assert_eq!(prev(None), 2);
    }

    #[test]
    fn test_an_empty_order_stays_at_zero_in_both_directions() {
        assert_eq!(next_index(0, None), 0);
        assert_eq!(prev_index(0, None), 0);
    }

    #[test]
    fn test_a_single_control_is_its_own_next_and_previous() {
        // A form with one control: tab must not move focus away from it, and must not
        // divide by zero.
        assert_eq!(next_index(1, Some(0)), 0);
        assert_eq!(prev_index(1, Some(0)), 0);
        assert_eq!(next_index(1, None), 0);
        assert_eq!(prev_index(1, None), 0);
    }

    #[test]
    fn test_an_index_that_is_not_in_the_order_resolves_like_nothing_focused() {
        // A focused area that has since been removed from the order: `position` is `None`,
        // which has to behave the same as never having been focused or the traversal would
        // jump to an arbitrary slot.
        assert_eq!(next_index(4, None), 0);
        assert_eq!(prev_index(4, None), 3);
    }
}
