//! Keyboard focus traversal — `tab` / `shift-tab` between controls.
//!
//! A port of gpui-bezel's `focus.rs` to the Makepad 2.0 `script_mod!` world.
//! gpui has focus machinery and nothing on by default; Makepad has a single
//! key-focus `Area` on `Cx` (`set_key_focus` / `has_key_focus`) and no
//! traversal at all. This module turns it on:
//!
//! - Focusable controls call [`register`] from their `draw_walk`, in paint
//!   order. The registry is a `Cx` global keyed by the widget's stable
//!   `WidgetUid`, so each control occupies exactly one slot and re-registers
//!   its fresh `Area` every frame (Makepad areas carry a per-redraw id).
//! - The app root calls [`handle_key`] from its own `handle_event`; it moves
//!   focus to the next / previous registered control on `tab` / `shift-tab`
//!   and wraps around. If nothing is focused it focuses the first (or last for
//!   shift-tab) control.
//!
//! ```ignore
//! // in the app's root Widget::handle_event, before/after children:
//! focus::handle_key(cx, event);
//! ```

use makepad_widgets::*;

/// The tab order — control `WidgetUid` → its most recent `Area`.
///
/// Registered by [`register`] during `draw_walk`. Keyed by uid so the map
/// never grows past the number of distinct controls: every frame each control
/// overwrites its own slot with the area it just drew.
#[derive(Default)]
pub struct FocusRegistry {
    entries: Vec<(WidgetUid, Area)>,
}

impl FocusRegistry {
    /// Insert (or overwrite, keyed by uid) a control's area. Returns `true`
    /// when the slot is new, `false` when it replaced an existing entry.
    pub fn insert(&mut self, uid: WidgetUid, area: Area) -> bool {
        if let Some(idx) = self.position_of_uid(uid) {
            self.entries[idx].1 = area;
            return false;
        }
        self.entries.push((uid, area));
        true
    }

    fn position_of_uid(&self, uid: WidgetUid) -> Option<usize> {
        self.entries.iter().position(|(u, _)| *u == uid)
    }

    /// The uids in tab (registration) order. Exposed for tests.
    #[cfg(test)]
    fn uid_order(&self) -> Vec<WidgetUid> {
        self.entries.iter().map(|(u, _)| *u).collect()
    }
}

/// Register a control's area under its stable uid into the tab order.
///
/// Paint order is traversal order — Makepad walks children in tree order, so a
/// control drawn earlier is tabbed into earlier. Hidden or disabled controls
/// must skip this call (a hidden control does not `draw_walk` at all, so it
/// naturally never (re-)registers).
pub fn register(cx: &mut Cx2d, uid: WidgetUid, area: Area) {
    if area.is_empty() {
        return;
    }
    cx.global::<FocusRegistry>().insert(uid, area);
}

/// The registry's areas in tab order, snapshot out of the global. Slots whose
/// area has gone stale (the control was removed or became invisible, so it
/// stopped re-registering) are dropped here so `tab` never lands on a control
/// that is no longer on screen.
fn current_order(cx: &mut Cx) -> Vec<Area> {
    // Snapshot (uid, area) out first so we can call `is_valid` (which borrows
    // `cx` immutably) without fighting the mutable borrow on the global.
    let snapshot: Vec<(WidgetUid, Area)> = {
        let reg = cx.global::<FocusRegistry>();
        reg.entries.clone()
    };
    let valid_uids: Vec<WidgetUid> = snapshot
        .iter()
        .filter(|(_, a)| a.is_valid(cx))
        .map(|(u, _)| *u)
        .collect();
    let reg = cx.global::<FocusRegistry>();
    reg.entries.retain(|(u, _)| valid_uids.contains(u));
    reg.entries.iter().map(|(_, area)| *area).collect()
}

/// Index of the slot to focus after `current`, wrapping. A `current` that is
/// not in `order` (e.g. nothing focused yet) resolves to `order.len()-1`, so a
/// plain `tab` from nothing starts at the first control.
fn next_index(order_len: usize, current: Option<usize>) -> usize {
    if order_len == 0 {
        return 0;
    }
    let idx = current.unwrap_or(order_len.saturating_sub(1));
    (idx + 1) % order_len
}

/// Index of the slot to focus before `current`, wrapping. A `current` that is
/// not in `order` (nothing focused) resolves to `0`, so `shift-tab` from
/// nothing starts at the last control.
fn prev_index(order_len: usize, current: Option<usize>) -> usize {
    if order_len == 0 {
        return 0;
    }
    let idx = current.unwrap_or(0);
    (idx + order_len - 1) % order_len
}

/// Move keyboard focus to the next control, wrapping around. Returns the
/// focused area, or `None` when there is nothing to focus.
pub fn focus_next(cx: &mut Cx) -> Option<Area> {
    let order = current_order(cx);
    if order.is_empty() {
        return None;
    }
    let cur = cx.key_focus();
    let current = order.iter().position(|a| *a == cur);
    let next = order[next_index(order.len(), current)];
    cx.set_key_focus(next);
    cx.redraw_all();
    Some(next)
}

/// Move keyboard focus to the previous control, wrapping around. Shift-Tab.
pub fn focus_prev(cx: &mut Cx) -> Option<Area> {
    let order = current_order(cx);
    if order.is_empty() {
        return None;
    }
    let cur = cx.key_focus();
    let current = order.iter().position(|a| *a == cur);
    let prev = order[prev_index(order.len(), current)];
    cx.set_key_focus(prev);
    cx.redraw_all();
    Some(prev)
}

/// Handle a `tab` / `shift-tab` key event for the whole window. Call from the
/// app's root `handle_event`. Returns `true` when the event was consumed.
///
/// Optional — the moves above are public, so an app that wants different keys
/// binds them itself:
///
/// ```ignore
/// if let Event::KeyDown(ke) = event {
///     if ke.key_code == KeyCode::Tab && !ke.is_repeat {
///         if ke.modifiers.shift { focus::focus_prev(cx); }
///         else { focus::focus_next(cx); }
///     }
/// }
/// ```
pub fn handle_key(cx: &mut Cx, event: &Event) -> bool {
    if let Event::KeyDown(ke) = event {
        if ke.key_code == KeyCode::Tab {
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
    fn insert_keyed_by_uid_replaces_not_grows() {
        let mut reg = FocusRegistry::default();
        assert!(reg.insert(uid(1), Area::Empty)); // new slot
        reg.insert(uid(2), Area::Empty);
        assert!(!reg.insert(uid(1), Area::Empty)); // existing slot -> replaced

        assert_eq!(reg.uid_order(), vec![uid(1), uid(2)]);
    }

    #[test]
    fn insert_preserves_paint_order_and_replaces_same_uid() {
        let mut reg = FocusRegistry::default();
        reg.insert(uid(3), Area::Empty);
        reg.insert(uid(1), Area::Empty);
        reg.insert(uid(2), Area::Empty);
        assert_eq!(reg.uid_order(), vec![uid(3), uid(1), uid(2)]);

        // Re-registering existing uid never reorders it.
        reg.insert(uid(1), Area::Empty);
        assert_eq!(reg.uid_order(), vec![uid(3), uid(1), uid(2)]);
        assert_eq!(reg.position_of_uid(uid(1)), Some(1));
        assert_eq!(reg.position_of_uid(uid(99)), None);
    }

    #[test]
    fn next_and_prev_wrap_around() {
        // 3 controls: 0 1 2
        let next = |c: Option<usize>| next_index(3, c);
        let prev = |c: Option<usize>| prev_index(3, c);

        // From control 0, next is 1; from 2, next wraps to 0.
        assert_eq!(next(Some(0)), 1);
        assert_eq!(next(Some(2)), 0);
        // From control 1, prev is 0; from 0, prev wraps to 2.
        assert_eq!(prev(Some(1)), 0);
        assert_eq!(prev(Some(0)), 2);
        // Nothing focused: next starts at first (index 0), prev starts at last.
        assert_eq!(next(None), 0);
        assert_eq!(prev(None), 2);
    }

    #[test]
    fn next_and_prev_on_empty_order_stay_at_zero() {
        assert_eq!(next_index(0, None), 0);
        assert_eq!(prev_index(0, None), 0);
    }
}
