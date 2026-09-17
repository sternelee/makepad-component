//! `MpAvatarGroup` — a row of faces that overlap, with a `+N` tail.
//!
//! ## Why this is a Rust type when the DSL already had one
//!
//! `mod.mp.MpAvatarGroup` is a `View` with **five avatars written into it by hand**, which is a fine thing for a showcase
//! page and useless for a caller with a list of names — a group whose members come from data needs the count to decide the
//! layout, and a DSL block cannot. So the static one stays where it is (it is the honest way to draw a fixed stack) and
//! this is the data-driven one, drawing the same faces through the same `MpAvatar`.
//!
//! Two groups with the same look and different jobs, which is a shape this port has met before — and the reason the two
//! are not merged is that a caller who knows its four members up front should not pay for a vector of them.
//!
//! ## The overlap is a ratio, and the comment that said so was lying
//!
//! The DSL block's comment claims its negative margin is "a ratio of the avatar's size rather than a number: `-28%` of a
//! 28pt face is 8pt, and of a 40pt face is 11". **The code writes `-8`, a number** — so at `MpAvatarLarge` the overlap
//! stayed 8pt instead of 11 and the "same object at either size" claim was simply false. The v2 widget it came from had a
//! five-row table of literals (5, 7, 8, 10, 12) with no stated provenance at all.
//!
//! [`OVERLAP_RATIO`] is that table collapsed into the one number the comment describes: 28% of the face reproduces the v2
//! table within a point at every size it covered (28 → 8, 40 → 11), so five hand-tuned constants with no source become one
//! ratio with the table recorded as its origin. **The claim is now true where it is implemented**, and the static DSL block
//! keeps its literals with its comment corrected to say so.
//!
//! ## The tail is in addition to the shown faces, not instead of one
//!
//! A limit of four with nine members shows **four faces and then `+5`** — five circles, not four. The other reading (the
//! tail replaces the fourth face) is a defensible design and is not this one; the difference is only visible when a count
//! exactly equals the limit, where this version shows no tail and the other version shows `+1` for a member who is already
//! on screen.

use makepad_widgets::*;

use makepad_theme::ControlSize;

use crate::mp::avatar::MpAvatar;

/// How many faces a group will ever draw, besides the tail.
///
/// The v2 ceiling, kept: it is a limit on **widgets**, not on meaning — a group is a glance at who is in something, and a
/// twenty-face row is a table. A caller with more members gets a larger `+N`, which is the honest summary.
pub const SLOTS: usize = 8;

/// The fraction of a face that its neighbour covers.
///
/// Collapsed from the v2 table (XSmall −5, Small −7, Medium −8, Large −10, XLarge −12): 28% of a face reproduces those
/// within a point, and unlike the table it is defined for sizes nobody tabulated. See the module doc.
pub const OVERLAP_RATIO: f64 = 0.28;

/// How much a face covers its neighbour, in points.
///
/// Rounded, because a group of faces at fractional offsets reads as ragged — the eye sees half a point of misalignment
/// between circles far more readily than between rectangles.
pub fn overlap_for(face: f64) -> f64 {
    if !face.is_finite() || face <= 0.0 {
        return 0.0;
    }
    (face * OVERLAP_RATIO).round()
}

/// The limit actually in force: **zero means "as many as there are slots"**, never zero faces.
///
/// The v2 reading, and the right one — a caller that does not set a limit wants a group, not an empty one. Clamped to
/// [`SLOTS`] at the top because the ceiling is what keeps a row finite.
pub fn effective_limit(limit: usize) -> usize {
    if limit == 0 {
        SLOTS
    } else {
        limit.min(SLOTS)
    }
}

/// How many members are drawn as faces.
pub fn shown(count: usize, limit: usize) -> usize {
    count.min(effective_limit(limit))
}

/// How many members the `+N` covers.
pub fn overflow(count: usize, limit: usize) -> usize {
    count.saturating_sub(shown(count, limit))
}

/// Whether a `+N` tail is drawn.
///
/// **Only when something would be hidden**: a tail reading `+0` is worse than no tail, and a count that exactly fills the
/// limit shows every member, so there is nothing to summarise.
pub fn shows_tail(count: usize, limit: usize, ellipsis: bool) -> bool {
    ellipsis && overflow(count, limit) > 0
}

/// The tail's text.
pub fn tail_text(overflow: usize) -> String {
    format!("+{overflow}")
}

/// How many circles the row holds, the tail included.
pub fn circles(count: usize, limit: usize, ellipsis: bool) -> usize {
    shown(count, limit) + usize::from(shows_tail(count, limit, ellipsis))
}

/// The row's width: `circles` faces overlapping all but the first.
///
/// `circles - 1` overlaps, for the same reason a grid has `columns - 1` gaps — the first face is not overlapped by
/// anything to its left, so counting its overlap would shorten the row by one face's worth of overlap.
pub fn group_width(circles: usize, face: f64, overlap: f64) -> f64 {
    if circles == 0 {
        return 0.0;
    }
    circles as f64 * (face - overlap) + overlap
}

/// Where the `index`-th circle starts, relative to the row.
///
/// Each circle is `face - overlap` past the one before it, which is what "overlapping by `overlap`" means — the only
/// arithmetic in the component and therefore a function with a test rather than an expression in a draw loop.
pub fn circle_x(index: usize, face: f64, overlap: f64) -> f64 {
    index as f64 * (face - overlap)
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    mod.mp.MpAvatarGroupBase = #(MpAvatarGroup::register_widget(vm))

    mod.mp.MpAvatarGroup = set_type_default() do mod.mp.MpAvatarGroupBase{
        width: Fit
        height: Fit
        face: 28.0
        control: mod.mpc.ControlSize.Regular
        ellipsis: true
    }

    mod.mp.MpAvatarGroupSmall = mod.mp.MpAvatarGroup{
        face: 20.0
        control: mod.mpc.ControlSize.Small
    }

    mod.mp.MpAvatarGroupLarge = mod.mp.MpAvatarGroup{
        face: 40.0
        control: mod.mpc.ControlSize.Large
    }
}

/// A row of faces that overlap, with a `+N` tail.
#[derive(Script, ScriptHook, Widget)]
pub struct MpAvatarGroup {
    #[source]
    source: ScriptObjectRef,
    /// The container's own view. **`#[deref]` rather than a bare `area` field**, so the derive finds `redraw` through it —
    /// this widget has no shader of its own to mark `#[redraw]`, because everything it draws is a face widget.
    #[deref]
    view: View,
    /// A face's side in points. The overlap is a fraction of it, so a group at any size is the same group.
    #[live]
    face: f64,
    /// The control size the faces are drawn at, for their own type and plate ring.
    #[live]
    control: ControlSize,
    /// Whether members past the limit roll into a `+N`.
    #[live]
    ellipsis: bool,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    /// The members, which are the caller's data.
    #[rust]
    names: Vec<String>,
    /// The limit in force, before [`effective_limit`] is applied.
    #[rust]
    limit: usize,
    /// One face per drawn circle, reused between paints: the widgets are expensive to make and their count changes only
    /// when the members do.
    #[rust]
    faces: Vec<MpAvatar>,
}

impl Widget for MpAvatarGroup {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // The group is a picture, not a control: it has no value and reports nothing. It reads no pointer and holds no
        // focus, so the only thing to do here is pass the event to the view the faces live on — which is what `#[deref]`
        // asks of a container.
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let overlap = overlap_for(self.face);
        let count = self.names.len();
        let circles = circles(count, self.limit, self.ellipsis);
        let width = group_width(circles, self.face, overlap);
        let placed = cx.walk_turtle(Walk {
            width: Size::Fixed(width),
            height: Size::Fixed(self.face),
            ..walk
        });

        // Make sure there are as many face widgets as circles. Never fewer: a face widget is only ever created here, so
        // the length only grows, and a group that shrinks simply draws fewer than it holds.
        while self.faces.len() < circles {
            self.faces.push(cx.cx.with_vm(MpAvatar::script_new_with_default));
        }

        let tail = shows_tail(count, self.limit, self.ellipsis);
        let faces = shown(count, self.limit);
        for index in 0..circles {
            let Some(face) = self.faces.get_mut(index) else {
                continue;
            };
            // The tail's own circle is not a person and must not look like one: it gets neutral ink and no plate, which is
            // what the `-1.0` tone asks for.
            let is_tail = tail && index == faces;
            let text = if is_tail {
                tail_text(overflow(count, self.limit))
            } else if let Some(name) = self.names.get(index) {
                // The name's own initials and plate, through the same calls the standalone avatar uses — so a face in a
                // group and the same face alone cannot disagree.
                crate::mp::avatar::initials(name)
            } else {
                String::new()
            };
            let tone = if is_tail {
                -1.0
            } else {
                self.names
                    .get(index)
                    .map(|name| crate::mp::avatar::tone_index(name) as f64)
                    .unwrap_or(-1.0)
            };
            // **`prepare`, not `set_text` + `set_presence`**: a setter redraws, and a redraw requested from inside a draw is
            // a frame that never ends. See its doc.
            face.prepare(&text, tone, self.control);
            let avatar_walk = Walk::fixed(self.face, self.face)
                .with_abs_pos(dvec2(placed.pos.x + circle_x(index, self.face, overlap), placed.pos.y));
            let _ = face.draw_walk(cx, scope, avatar_walk);
        }
        DrawStep::done()
    }
}

impl MpAvatarGroup {
    /// Fill the group with members, by display name.
    pub fn set_avatars(&mut self, cx: &mut Cx, members: &[String]) {
        self.names = members.to_vec();
        self.redraw(cx);
    }

    pub fn avatars(&self) -> &[String] {
        &self.names
    }

    /// Set the limit, with **zero meaning "as many as there are slots"** rather than none. See [`effective_limit`].
    pub fn set_limit(&mut self, cx: &mut Cx, limit: usize) {
        self.limit = limit;
        self.redraw(cx);
    }

    pub fn limit(&self) -> usize {
        effective_limit(self.limit)
    }

    pub fn set_ellipsis(&mut self, cx: &mut Cx, ellipsis: bool) {
        self.ellipsis = ellipsis;
        self.redraw(cx);
    }

    pub fn shows_ellipsis(&self) -> bool {
        self.ellipsis
    }

    /// The circles this group draws now: the shown faces plus a tail when one is due.
    pub fn circles(&self) -> usize {
        circles(self.names.len(), self.limit, self.ellipsis)
    }

    /// The row's width at the current `face`.
    pub fn width(&self) -> f64 {
        group_width(self.circles(), self.face, overlap_for(self.face))
    }

    /// How many members the tail covers.
    pub fn overflow(&self) -> usize {
        overflow(self.names.len(), self.limit)
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

    pub fn avatars(&self) -> Vec<String> {
        self.borrow().map(|inner| inner.names.clone()).unwrap_or_default()
    }

    pub fn circles(&self) -> usize {
        self.borrow().map(|inner| inner.circles()).unwrap_or(0)
    }

    /// The limit in force — **not the limit that was set**, because zero means "as many slots as there are" and a reader
    /// that reported the raw field would report zero faces for a group drawing eight.
    pub fn limit(&self) -> usize {
        self.borrow().map(|inner| inner.limit()).unwrap_or(0)
    }

    /// How many members the tail covers.
    pub fn overflow(&self) -> usize {
        self.borrow().map(|inner| inner.overflow()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_overlap_is_a_fraction_of_the_face_and_reproduces_the_table_it_replaced() {
        // **The claim the DSL comment made and the code did not.** A ratio is scale-invariant, so a group at either size is
        // the same group — which is what makes `MpAvatarGroupSmall` and `MpAvatarGroupLarge` look like one object
        // twice rather than two objects.
        assert_eq!(overlap_for(28.0), 8.0, "the v2 medium row");
        assert_eq!(overlap_for(40.0), 11.0, "the v2 large row, which the literal -8 got wrong");
        // Scale invariance, stated as a property: doubling the face doubles the overlap.
        for face in [16.0, 20.0, 24.0, 32.0] {
            let single = overlap_for(face);
            let double = overlap_for(face * 2.0);
            assert!(
                (double - single * 2.0).abs() <= 1.0,
                "face {face}: {single} doubled is not {double}"
            );
        }
    }

    #[test]
    fn test_a_face_that_is_not_a_size_has_no_overlap_rather_than_a_nonsense_one() {
        // A zero or negative face is a group nobody can see, and a `NaN` face would put a `NaN` overlap into every
        // position — the failure this port has paid for three times now.
        assert_eq!(overlap_for(0.0), 0.0);
        assert_eq!(overlap_for(-10.0), 0.0);
        assert_eq!(overlap_for(f64::NAN), 0.0);
        assert_eq!(overlap_for(f64::INFINITY), 0.0);
    }

    #[test]
    fn test_zero_means_as_many_slots_as_there_are_rather_than_none() {
        // The v2 reading. A caller that never set a limit wants a group, not an empty one — and this is the same shape as
        // the colour picker's `columns_clamped(0)`, where reading "unset" as "the smallest thing" was wrong.
        assert_eq!(effective_limit(0), SLOTS);
        assert_eq!(effective_limit(3), 3);
        assert_eq!(effective_limit(99), SLOTS, "the ceiling is what keeps a row finite");
        // ...and with no limit set, every member is a face.
        assert_eq!(shown(8, 0), 8);
        assert_eq!(shown(9, 0), SLOTS, "past the ceiling the tail takes over");
        assert_eq!(overflow(9, 0), 1);
        assert_eq!(overflow(20, 0), 12);
    }

    #[test]
    fn test_the_tail_is_in_addition_to_the_shown_faces() {
        // **The design decision, stated as a test.** Four faces and nine members is `+5` after four faces: five circles.
        // The other reading — the tail replacing a face — would show four circles and `+6`, and the difference is only
        // visible exactly at the limit, where this version shows no tail at all because nothing is hidden.
        assert_eq!(shown(9, 4), 4);
        assert_eq!(overflow(9, 4), 5);
        assert_eq!(circles(9, 4, true), 5, "four faces and a tail");
        // Exactly at the limit, every member is on screen, so a tail would be summarising nobody.
        assert_eq!(overflow(4, 4), 0);
        assert_eq!(shows_tail(4, 4, true), false, "a +0 tail would be a lie about a hidden member");
        assert_eq!(circles(4, 4, true), 4);
        // One past it, and exactly one is hidden.
        assert_eq!(overflow(5, 4), 1);
        assert_eq!(circles(5, 4, true), 5);
        assert_eq!(tail_text(overflow(5, 4)), "+1");
    }

    #[test]
    fn test_no_tail_is_drawn_when_the_caller_turned_it_off() {
        // A group with the tail disabled shows only real members. Losing the count is a caller's choice to make, and the
        // rows it produces are wider or narrower accordingly — which is why the width accounts for the tail.
        assert_eq!(circles(9, 4, false), 4);
        assert_eq!(shows_tail(9, 4, false), false);
        assert_eq!(circles(9, 4, true), 5);
        assert_ne!(circles(9, 4, false), circles(9, 4, true));
    }

    #[test]
    fn test_an_empty_group_has_no_width_and_no_circles() {
        assert_eq!(circles(0, 0, true), 0);
        assert_eq!(group_width(0, 28.0, 8.0), 0.0);
        assert_eq!(overflow(0, 4), 0);
        assert_eq!(shows_tail(0, 4, true), false);
    }

    #[test]
    fn test_the_width_counts_the_overlaps_between_circles_and_not_one_at_each_edge() {
        // `circles - 1` overlaps, the same rule as the colour picker's `columns - 1` gaps: the first circle has nothing to
        // its left, so counting its overlap would shorten the row by one overlap's worth. This is asserted against the
        // arithmetic a person would do by hand, not against the expression in the code.
        let face = 28.0;
        let overlap = overlap_for(face);
        assert_eq!(overlap, 8.0);
        assert_eq!(group_width(1, face, overlap), face, "one circle is one face wide");
        assert_eq!(group_width(2, face, overlap), face + (face - overlap));
        assert_eq!(group_width(5, face, overlap), 28.0 + 20.0 * 4.0);
    }

    #[test]
    fn test_every_circle_starts_exactly_one_face_minus_the_overlap_after_the_last() {
        let face = 28.0;
        let overlap = 8.0;
        assert_eq!(circle_x(0, face, overlap), 0.0);
        assert_eq!(circle_x(1, face, overlap), 20.0);
        assert_eq!(circle_x(2, face, overlap), 40.0);
        // **The property that ties the two functions together**: the last circle's start plus one face is the row's width.
        // If these two ever disagreed, every circle would be drawn inside a row sized for a different number of them —
        // which is the sort of defect that shows as the last face hanging outside its own background.
        for circles in 1..10usize {
            let last = circle_x(circles - 1, face, overlap);
            assert_eq!(
                last + face,
                group_width(circles, face, overlap),
                "{circles} circles: the last circle ends where the row does not"
            );
        }
    }
}
