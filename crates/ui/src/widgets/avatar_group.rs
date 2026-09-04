use makepad_widgets::*;

use crate::widgets::avatar::{MpAvatarRef, MpAvatarWidgetRefExt};
use crate::widgets::sizing::MpSize;

/// Convenience: borrow an MpAvatar slot as its Ref (mutable view access).
fn avatar_slot(view: &mut makepad_widgets::View, cx: &mut Cx, slot: LiveId) -> MpAvatarRef {
    view.child(slot).as_mp_avatar()
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpAvatarGroup - overlapping avatar stack (gpui AvatarGroup).
    // Fixed slots with a baked color rotation (default/primary/success/
    // danger/warning ...); overlap = 30% of the avatar diameter.
    // ============================================================

    mod.widgets.MpAvatarGroupBase = #(MpAvatarGroup::register_widget(vm))
    mod.widgets.MpAvatarGroup = set_type_default() do mod.widgets.MpAvatarGroupBase{
        width: Fit
        height: Fit

        flow: Right
        align: Align{y: 0.5}

        // Fixed avatar slots with a color rotation baked into the DSL
        avatar0 := mod.widgets.MpAvatar{}
        avatar1 := mod.widgets.MpAvatarPrimary{}
        avatar2 := mod.widgets.MpAvatarSuccess{}
        avatar3 := mod.widgets.MpAvatarDanger{}
        avatar4 := mod.widgets.MpAvatarWarning{}
        avatar5 := mod.widgets.MpAvatar{}
        avatar6 := mod.widgets.MpAvatarPrimary{}
        avatar7 := mod.widgets.MpAvatarSuccess{}

        // "+N" tail avatar (muted)
        more := mod.widgets.MpAvatar{
            visible: false
            label +: { text: "+" }
            draw_bg +: {
                bg_color: ELEMENT_ACTIVE
            }
        }
    }
}

pub const AVATAR_GROUP_SLOTS: usize = 8;

/// Overlapping avatar stack with a size-consistent "+N" tail when the
/// member count exceeds the slot limit.
#[derive(Script, ScriptHook, Widget)]
pub struct MpAvatarGroup {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    /// Max avatars painted before the tail takes over.
    #[rust]
    limit: usize,
    /// Whether a "+N" tail shows when members exceed `limit`.
    #[rust]
    ellipsis: bool,

    /// Five-step size propagated to every avatar slot.
    #[live]
    size: MpSize,

    /// Last size applied (avoids re-applying every draw).
    #[rust]
    applied_size: Option<MpSize>,
}

impl Widget for MpAvatarGroup {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Size: re-apply the negative overlap when the size step changes.
        if self.applied_size != Some(self.size) {
            self.applied_size = Some(self.size);
            self.apply_overlap(cx);
        }
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpAvatarGroup {
    /// Negative left margin for every avatar except the first in flow
    /// order (later siblings paint over earlier ones).
    fn overlap_for(&self) -> f64 {
        match self.size {
            MpSize::XSmall => -5.0,
            MpSize::Small => -7.0,
            MpSize::Medium => -8.0,
            MpSize::Large => -10.0,
            MpSize::XLarge => -12.0,
        }
    }

    fn apply_overlap(&mut self, cx: &mut Cx) {
        let overlap = self.overlap_for();
        for i in 0..AVATAR_GROUP_SLOTS {
            let slot_id = LiveId::from_str(&format!("avatar{}", i));
            let slot_ref = self.view.child(slot_id).as_mp_avatar();
            slot_ref.set_size(cx, self.size);
            slot_ref.set_overlap_margin(cx, if i == 0 { 0.0 } else { overlap });
        }
        let more_ref = self.view.child(id!(more)).as_mp_avatar();
        more_ref.set_size(cx, self.size);
        more_ref.set_overlap_margin(cx, overlap);
    }
}

impl MpAvatarGroup {
    /// Fill the group with initial-text members; members beyond `limit`
    /// roll into the "+N" tail when `ellipsis` is enabled.
    pub fn set_avatars(&mut self, cx: &mut Cx, members: &[String]) {
        let limit = if self.limit == 0 {
            AVATAR_GROUP_SLOTS
        } else {
            self.limit.min(AVATAR_GROUP_SLOTS)
        };
        let shown = members.len().min(limit);

        for i in 0..AVATAR_GROUP_SLOTS {
            let slot_id = LiveId::from_str(&format!("avatar{}", i));
            let slot = self.view.child(slot_id).as_mp_avatar();
            if i < shown {
                slot.set_visible(cx, true);
                slot.set_text(cx, &members[i]);
            } else {
                slot.set_visible(cx, false);
            }
        }

        let rest = members.len().saturating_sub(shown);
        let more_ref = self.view.child(id!(more)).as_mp_avatar();
        if self.ellipsis && rest > 0 {
            more_ref.set_text(cx, &format!("+{}", rest));
            more_ref.set_visible(cx, true);
        } else {
            more_ref.set_visible(cx, false);
        }
        self.apply_overlap(cx);
        self.view.redraw(cx);
    }

    pub fn set_limit(&mut self, cx: &mut Cx, limit: usize) {
        self.limit = limit;
        self.view.redraw(cx);
    }

    pub fn set_ellipsis(&mut self, cx: &mut Cx, ellipsis: bool) {
        self.ellipsis = ellipsis;
        self.view.redraw(cx);
    }

    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            self.applied_size = None;
            self.redraw(cx);
        }
    }
}

impl MpAvatarGroupRef {
    pub fn set_avatars(&self, cx: &mut Cx, members: &[String]) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_avatars(cx, members);
        }
    }

    pub fn set_limit(&self, cx: &mut Cx, limit: usize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_limit(cx, limit);
        }
    }

    pub fn set_ellipsis(&self, cx: &mut Cx, ellipsis: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_ellipsis(cx, ellipsis);
        }
    }

    pub fn size(&self) -> MpSize {
        if let Some(inner) = self.borrow() {
            inner.size()
        } else {
            MpSize::default()
        }
    }

    pub fn set_size(&self, cx: &mut Cx, size: MpSize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_size(cx, size);
        }
    }
}
