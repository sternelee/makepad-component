//! Compatibility forwarders to [`crate::mp::focus`].
//!
//! The traversal moved to `mp/focus.rs` when the **v3** controls needed it: they were
//! registering into a v2 module, and the v3 API needs a `disabled` parameter the v2 shape
//! does not have — see that module on the stale-entry bug the parameter fixes.
//!
//! This module stays because the v2 widgets still call it, and because the apps that drive
//! them (component-zoo, a2ui-demo) call `handle_key` here. Everything forwards to
//! **one registry**, which is the part that matters: two registries would mean `tab` walking
//! half the controls on a page that has both, and the gallery has exactly such pages while
//! the port is in progress.
//!
//! ## A known limitation, recorded rather than repaired
//!
//! [`register`] forwards with `disabled: false`, so the v2 widgets keep their own
//! `if !self.disabled` guards — and therefore keep the bug `mp/focus.rs` fixes: a v2 control
//! that *becomes* disabled stays in the tab order while its area is still valid. Fixing it
//! means passing the flag at seven v2 call sites, and those seven widgets are scheduled for
//! deletion by phase 6; repairing them would be work spent on code the plan already removes.
//! The new API does not have the fault, and every v3 control uses it.
//!
//! Deleted with the v2 widgets.

use makepad_widgets::*;

pub use crate::mp::focus::FocusRegistry;

/// Register a control, in the v2 shape: the caller has already checked `disabled`.
pub fn register(cx: &mut Cx2d, uid: WidgetUid, area: Area) {
    crate::mp::focus::register(cx, uid, area, false);
}

/// See [`crate::mp::focus::focus_next`].
pub fn focus_next(cx: &mut Cx) -> Option<Area> {
    crate::mp::focus::focus_next(cx)
}

/// See [`crate::mp::focus::focus_prev`].
pub fn focus_prev(cx: &mut Cx) -> Option<Area> {
    crate::mp::focus::focus_prev(cx)
}

/// See [`crate::mp::focus::handle_key`].
pub fn handle_key(cx: &mut Cx, event: &Event) -> bool {
    crate::mp::focus::handle_key(cx, event)
}
