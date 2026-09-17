//! `MpSegmented` — one of a few adjacent choices, with a plate marking which.
//!
//! ## Why this exists, and why it is a widget rather than a composition
//!
//! The Bars page needed a `Source | Split | Preview` control and there was none, so
//! it used three ghost buttons with the active one promoted, and said in the page
//! that this was an interim. This module is that interim replaced.
//!
//! It cannot be a composition the way `MpPopover` is. A segmented control's whole
//! visual identity is that the segments are **adjacent** — one rounded track with a
//! plate inside it — and three independent buttons have three rounded outlines with
//! gaps between them. So the widget owns the track and the plate.
//!
//! ## The arithmetic is the part worth testing
//!
//! Everything here is either a label or a rectangle. The rectangles are the part
//! that can be wrong invisibly:
//!
//! - **Which slot is at a position** — the hit test. A segmented control's segments
//!   are not child widgets, so the slot under the pointer is arithmetic on the box
//!   rather than a hit test on a child. Off by one and the reader selects their
//!   neighbour, which is the kind of fault that reads as "the mouse is broken".
//! - **Where a slot is** — the plate. Division by zero when there are no segments,
//!   and segments that must **tile** the box: the first starts at zero, the last
//!   ends at the width, and nothing is left over between them.
//!
//! Both are pure functions of `(index, count, width)`, so both are tested with no
//! window at all. The widget is then the part that reads them.
//!
//! ## Equal slots, not content-sized ones
//!
//! Every slot is the width of the **widest** label plus its padding, so all the
//! slots are equal. That is what makes [`slot_span`] arithmetic on the box instead
//! of a measurement of the children — and it is the canonical form, an iOS-style
//! segmented control. A content-sized variant would need each segment's width
//! measured during the draw pass and the plate drawn afterwards, which is a
//! different widget rather than a flag on this one.
//!
//! ## The plate jumps rather than slides
//!
//! A Makepad animator animates towards values declared in the DSL's `AnimatorState`s,
//! and this plate's target is a *dynamic* index. A slide therefore needs a clock and
//! a lerp rather than an animator state, which is a different design; the plate is
//! drawn at its destination. Recorded rather than hidden, because the motion is the
//! one thing a reader who knows segmented controls will notice is absent.

use makepad_widgets::*;

use crate::mp::text;

/// The label's breathing room inside its slot, on each side.
///
/// Air on each side of a slot's label, so a segment reads as a target rather than as text.
///
/// ## It carried a bug for three attempts, and the fourth was to stop carrying it
///
/// The Bars toolbar's first render drew `Preview` as **`Previ`**, and there were three attempts to fix it with
/// a *better estimate*: this pad raised from 14 to 16 to absorb the error, a glyph correction in `mp/text.rs`
/// for symbols, and then the DPI factor that estimate had always been missing. Each was an improvement and none
/// was the answer — a screenshot after the third still showed `Previ`, because the estimate for that particular
/// word was still under.
///
/// The labels are **measured** now (`text::measured_width`, which is `DrawText::layout` plus the layout-pixel to
/// point conversion), so this is air and nothing else: 14 points of it, which is where it started before it was
/// asked to hide an error.
///
/// ## Why it was clipped, which took eight attempts
///
/// The panel painted ~193 while its walk said 276.3, and the third label was invisible. The cause is in the
/// **parent**, not here, and it is a trap in `Fit`:
///
/// > `View::walk_from_previous_size` resolves a `Fit` width from **that view's own previous area**
/// > (`view_size`, written at the end of `View::draw_walk`), **not from its children's content.**
///
/// So a `Fit` container that was first given less than its child needs is a **self-reinforcing fixed point**: a
/// small box, so it clips, so its area is small, so it asks for the small box again. And **everything a child
/// draws is clipped to that box** — `draw_abs` as much as `begin` — so nothing the control drew could escape.
/// Giving the *container* an explicit width breaks the loop, which is what the Bars page now does, and it is
/// documented in `mp/bars.rs` where the slots are declared.
///
/// The eight attempts are worth keeping because **seven of them were in the wrong file**:
///
/// | attempt | result |
/// |---|---|
/// | a larger pad; a glyph correction; the missing DPI factor | no change — the estimate was never the problem |
/// | replacing the estimate with `DrawText::layout`'s own number | geometry **more** correct, paint unchanged |
/// | deciding the width in `set_segments` rather than while drawing | kept, because it is right; no change here |
/// | `width: 400` **on this control** | the **labels** moved, the **panel** did not |
/// | two `Fill` siblings in the bar | they **split** the space — real, and now in `mp/bars.rs` |
/// | reading `walk_from_previous_size` | found the mechanism; then a `Fixed` width **on the container** |
///
/// The last row is the fix. The lesson is the shape of the search: a widget whose geometry is provably correct
/// while its paint is not has a **parent** problem, and the measurement is the wrong place to look — which is
/// where seven attempts went.
const SLOT_PAD_X: f64 = 14.0;

/// How far the plate is inset from its slot.
///
/// Chosen: the plate has to be visible *inside* the track with the track showing
/// around it, or a selected segment reads as a filled cell rather than as a pill
/// that has moved. Two points is enough to show the track's fill; more and the plate
/// stops being aligned with its label.
const PLATE_INSET: f64 = 2.0;

/// Which segment's slot contains `x`, measured from the control's left edge.
///
/// `None` for a box with no segments, and `None` for a position outside it — so a
/// click that lands on the track's rounded corner or a pixel past the right edge
/// selects nothing rather than the nearest segment.
pub fn slot_at(x: f64, count: usize, width: f64) -> Option<usize> {
    // `width.is_finite()` is load-bearing and was **missing** in the first version:
    // the guard checked the position for NaN but not the box. A NaN width passes
    // `width <= 0.0` (every comparison against NaN is false) and passes
    // `x >= width`, so the whole guard is skipped, `slot_w` becomes NaN, and
    // `NaN.floor() as usize` **saturates to 0** — so a control whose layout has not
    // settled yet reported a hit on slot 0 for any pointer position, including one
    // outside it. The test for it was written before the guard was.
    if count == 0
        || !width.is_finite()
        || width <= 0.0
        || !x.is_finite()
        || x < 0.0
        || x >= width
    {
        return None;
    }
    let slot_w = width / count as f64;
    if slot_w <= 0.0 {
        return None;
    }
    // `min` because a width that does not divide evenly can put the last slot's
    // upper edge a rounding step below `width`, and `floor` of a value just under
    // `count` is already `count` itself.
    Some((((x / slot_w).floor() as usize)).min(count - 1))
}

/// The left edge and width of segment `index`'s slot, in a box `width` wide.
///
/// `None` for a box with no segments or a width that is not positive, and `None` for
/// an index past the last slot — a stale index should draw **no** plate rather than a
/// plate in the wrong place. That is the same rule [`crate::mp::palette::remap`]
/// exists for, applied to geometry.
pub fn slot_span(index: usize, count: usize, width: f64) -> Option<(f64, f64)> {
    if count == 0 || width <= 0.0 || !width.is_finite() || index >= count {
        return None;
    }
    let slot_w = width / count as f64;
    if slot_w <= 0.0 {
        return None;
    }
    // `index * width / count` rather than `index * slot_w`, so that the last slot's
    // right edge comes out as close to `width` as the arithmetic allows.
    let left = index as f64 * width / count as f64;
    let right = (index + 1) as f64 * width / count as f64;
    Some((left, right - left))
}

/// Clamp a remembered selection to a segment list of `count` entries.
///
/// A segmented control whose list shrinks — "Split" removed while it was selected —
/// would otherwise hold an index that draws no plate and reports a selection for a
/// segment that no longer exists.
pub fn valid(active: Option<usize>, count: usize) -> Option<usize> {
    match active {
        Some(index) if index < count => Some(index),
        _ => None,
    }
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    /// The track and the plate.
    ///
    /// One shader drawn twice, exactly as `mp/list.rs` and `mp/pagination.rs` do: the
    /// track is the widget's own rectangle and the plate is a `draw_abs` inside it.
    /// A second shader would be a second copy of `sdf.box`.
    set_type_default() do #(DrawMpSegmented::script_shader(vm)){
        ..mod.draw.DrawQuad

        fill: #x00000000
        border: #x00000000
        border_width: 0.0
        radius: 10.0

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let bw = self.border_width
            sdf.box(bw, bw, self.rect_size.x - bw * 2.0, self.rect_size.y - bw * 2.0, max(1.0, self.radius))
            sdf.fill_keep(self.fill)
            if (bw > 0.0) {
                sdf.stroke(self.border, bw)
            }
            return sdf.result
        }
    }

    mod.mp.MpSegmentedBase = #(MpSegmented::register_widget(vm))

    mod.mp.MpSegmented = set_type_default() do mod.mp.MpSegmentedBase{
        // `Fit`, so the control is as wide as its segments need. A caller that wants
        // it to span a toolbar sets `width: Fill` and the slots divide whatever box
        // it lands in.
        width: Fit
        height: mod.mpc.layout.control.regular.height

        draw_label +: {
            text_style: mod.mpc.type.body
            color: #x00000000
        }
    }

    mod.mp.MpSegmentedSmall = mod.mp.MpSegmented{
        height: mod.mpc.layout.control.small.height
        draw_label +: {text_style: mod.mpc.type.caption}
    }
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpSegmented {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    fill: Vec4f,
    #[live]
    border: Vec4f,
    #[live]
    border_width: f32,
    #[live]
    radius: f32,
}

/// What a segmented control reports.
#[derive(Clone, Debug, Default)]
pub enum MpSegmentedAction {
    /// The selection changed to this index. Emitted only on a **change**, so a
    /// caller can treat it as "the user picked this" without filtering — clicking
    /// the segment already selected reports nothing, the same rule
    /// [`crate::mp::radio::MpRadio::select`] follows.
    Selected(usize),
    #[default]
    None,
}

#[derive(Script, Widget)]
pub struct MpSegmented {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_bg: DrawMpSegmented,
    #[live]
    draw_label: DrawText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    // Segments come from Rust, like `MpList`'s rows: a list of labels is not
    // expressible as a DSL literal, and it is the same decision the list made.
    #[rust]
    segments: Vec<String>,
    /// The selection, clamped to the segment list on every write.
    #[rust]
    active: Option<usize>,
    #[rust]
    hovered: Option<usize>,
    #[rust]
    area: Area,
    /// This widget's own last drawn size, for resolving a `Fit` width.
    ///
    /// **`View::walk_from_previous_size` does this for a `View`, and a custom `Widget` has to do it for itself.**
    /// Makepad resolves a child's `Fit` dimension from what it measured *last* time — that is how a `Fit` view
    /// converges, because its content-driven size cannot be known before it draws. A widget that is not a `View`
    /// gets `None` for that and is therefore allocated **nothing**, so everything it draws is clipped to an empty
    /// cell. That was this control: a panel of ~193 where its walk said 276.3, with the third label invisible, and
    /// six attempts went after the *measurement* rather than after the allocation.
    #[rust]
    last_size: Option<Vec2d>,
}

impl MpSegmented {
    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    pub fn active(&self) -> Option<usize> {
        self.active
    }

    /// Whether this control reported a change in this batch, and to what.
    pub fn selected(&self, actions: &Actions) -> Option<usize> {
        crate::mp::action::first::<MpSegmentedAction>(self.widget_uid(), actions).and_then(|a| {
            match a {
                MpSegmentedAction::Selected(index) => Some(*index),
                MpSegmentedAction::None => None,
            }
        })
    }

    /// Replace the segments.
    ///
    /// The selection is **clamped silently** when the list shrinks past it: an index
    /// that no longer exists is not a choice the reader made, so reporting it as a
    /// selection would be a lie about what happened. A caller that needs to know
    /// reads [`MpSegmented::active`] after calling this, which is the same contract
    /// `MpList::set_items` has.
    pub fn set_segments(&mut self, cx: &mut Cx, segments: Vec<String>) {
        self.segments = segments;
        self.active = valid(self.active, self.segments.len());
        self.hovered = None;
        // **The width is decided here, on the way in, and not in `draw_walk`.**
        //
        // A parent lays its children out by reading their **declared** `walk`, and only then calls their
        // `draw_walk`. So a widget that sets its own width *while drawing* is a frame too late every frame: the
        // parent allocated nothing, and everything the child drew — including its `draw_abs` markers and
        // labels — was clipped to that. A segmented control painted two of its three labels, and three attempts
        // to fix it went after the *measurement* rather than after **when** the measurement was applied.
        //
        // `set_segments` has a `Cx`, which is what measuring a label needs, and it runs between frames. So the
        // size is known before the next layout pass and the parent reads a `Fixed` walk.
        if !self.segments.is_empty() {
            let width = self.content_width(cx);
            self.walk.width = Size::Fixed(width);
        }
        self.redraw(cx);
    }

    /// Choose a segment, without reporting.
    pub fn set_active(&mut self, cx: &mut Cx, active: Option<usize>) {
        let active = valid(active, self.segments.len());
        if self.active == active {
            return;
        }
        self.active = active;
        self.redraw(cx);
    }

    /// Choose a segment as the user would, reporting a change.
    pub fn select(&mut self, cx: &mut Cx, index: usize) {
        if index >= self.segments.len() || self.active == Some(index) {
            return;
        }
        self.active = Some(index);
        cx.widget_action(self.widget_uid(), MpSegmentedAction::Selected(index));
        self.redraw(cx);
    }

    /// Move the selection, wrapping.
    pub fn step(&mut self, cx: &mut Cx, delta: i32) {
        let count = self.segments.len();
        if count == 0 {
            return;
        }
        let current = self.active.unwrap_or(0) as i32;
        let moved = (current + delta).rem_euclid(count as i32) as usize;
        self.select(cx, moved);
    }

    /// The slot width one segment gets, measured from the drawn box.
    fn slot_width(&self, width: f64) -> f64 {
        if self.segments.is_empty() {
            0.0
        } else {
            width / self.segments.len() as f64
        }
    }

    /// How wide one label paints, **measured** rather than estimated.
    ///
    /// `DrawText::layout` lays the string out and returns the size, so this is the same number the renderer will
    /// use — no estimator, no error margin and no direction to get wrong.
    ///
    /// **This replaces [`crate::mp::text::width`] in this widget's geometry**, and it is the fix for a fault
    /// that took three attempts: the segmented control's last label was clipped (`Preview` drawn as `Previ`) and
    /// each attempt was a better *estimate* — a bigger pad, a glyph correction, then the missing DPI factor —
    /// when the answer was to stop estimating. The estimator stays in `text.rs` for **clipping**, where an error
    /// in either direction is invisible.
    fn label_width(&self, cx: &mut Cx, label: &str) -> f64 {
        text::measured_width(&self.draw_label, cx, label)
    }

    /// How wide the control wants to be: every slot the width of the widest label, **measured**.
    fn content_width(&mut self, cx: &mut Cx) -> f64 {
        if self.segments.is_empty() {
            return 0.0;
        }
        let mut widest = 0.0f64;
        for index in 0..self.segments.len() {
            let width = self.label_width(cx, &self.segments[index]);
            widest = widest.max(width);
        }
        (widest + SLOT_PAD_X * 2.0) * self.segments.len() as f64
    }

    /// Which slot a position in the control's own coordinates falls in.
    ///
    /// Reads the width back from the drawn `Area` rather than from
    /// `content_width`, because the two differ whenever a caller set `width: Fill` —
    /// and the hit test has to describe the box that was **drawn**, not the one the
    /// widget would have chosen. Before the first layout the `Area` is empty, which
    /// makes the width zero and every position a miss, which is right: nothing has
    /// been drawn to click on yet.
    fn slot_at_local(&self, cx: &Cx, x: f64) -> Option<usize> {
        let width = self.area.rect(cx).size.x;
        slot_at(x, self.segments.len(), width)
    }
}

/// Empty on purpose, and the emptiness is the interesting part.
///
/// `Widget` derive requires `ScriptHook`, and every other control in this crate
/// needs its body to **seat an animator at its initial value** — a checkbox built
/// `checked` has to `cut` rather than `play`, or it animates in from off on the first
/// frame. This widget has no animator, because its plate is drawn rather than
/// animated (see the module doc on why it jumps), so there is nothing to seat and
/// nothing to place. A hook that did something here would be doing something it
/// should not.
impl ScriptHook for MpSegmented {
    fn on_after_new(&mut self, _vm: &mut ScriptVm) {}
}

impl Widget for MpSegmented {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        // Split rather than matched together, following `mp/list.rs`: a hover event
        // and a move event are different types with the same field.
        match event.hits(cx, self.area) {
            Hit::FingerHoverIn(fe) => {
                cx.set_cursor(MouseCursor::Hand);
                self.hovered = self.slot_at_local(cx, fe.abs.x - self.area.rect(cx).pos.x);
                self.redraw(cx);
            }
            Hit::FingerMove(fe) => {
                let hovered = self.slot_at_local(cx, fe.abs.x - self.area.rect(cx).pos.x);
                if hovered != self.hovered {
                    self.hovered = hovered;
                    self.redraw(cx);
                }
            }
            Hit::FingerHoverOut(_) => {
                if self.hovered.is_some() {
                    self.hovered = None;
                    self.redraw(cx);
                }
            }
            Hit::FingerUp(fe) => {
                // `is_over`, so a press that slid off the control does not select
                // whatever it happened to be over on release.
                if fe.is_over {
                    if let Some(index) = self.slot_at_local(cx, fe.abs.x - self.area.rect(cx).pos.x) {
                        self.select(cx, index);
                    }
                }
            }
            _ => {
                // Arrows, for the reason every other control in the library has
                // them: a segmented control is a choice, and a choice has to be
                // reachable without a pointer.
                if cx.has_key_focus(self.area) {
                    if let Event::KeyDown(ke) = event {
                        if !ke.is_repeat {
                            match ke.key_code {
                                KeyCode::ArrowRight | KeyCode::ArrowDown => self.step(cx, 1),
                                KeyCode::ArrowLeft | KeyCode::ArrowUp => self.step(cx, -1),
                                KeyCode::ReturnKey | KeyCode::Space => {
                                    if let Some(index) = self.active {
                                        self.select(cx, index);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // Copied out before any mutable use of `cx`; see the note in `mp/table.rs`.
        let (track, plate, hover_wash, border, strong, muted, radius) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            let p = &theme.paint;
            (
                p.input_bg,
                p.surface_raised,
                p.element_hover,
                p.border,
                p.text,
                p.text_muted,
                makepad_theme::Theme::control_radius() as f32,
            )
        };

        // The font size is **read, not written**. `MpSegmented` declares
        // `text_style: mod.mpc.type.body` and `MpSegmentedSmall` overrides it to
        // `caption`; the first version of this function assigned the Body size over
        // the top of whatever the DSL had chosen, so the small variant drew at the
        // body size and the width estimate disagreed with the paint. Reading it back
        // means the variant's own type is respected and the measurement and the
        // draw cannot disagree — the same rule `mp/pagination.rs` states about
        // setting the drawn size from the rung the arithmetic uses.
        let font = self.draw_label.text_style.font_size as f64;

        self.draw_label.color = strong;

        // States its own size, because every part of it is drawn by `draw_abs`
        // inside its own turtle and nothing else can measure it. See `mp/table.rs`.
        // `Fit { .. }` rather than `Fit`: it is a struct variant carrying a weight,
        // a basis and bounds. A caller that asked for `Fill` keeps it, and the slots
        // then divide whatever box the control lands in — which is the case the hit
        // test reads back from the drawn `Area` rather than from `content_width`.
        // **A `Fit` width is resolved from this widget's own last drawn size**, the way
        // `View::walk_from_previous_size` resolves it for a `View`. On the first frame there is no previous size,
        // so the measured content width is used and the frame after that converges on what was actually drawn.
        let measured = self.content_width(cx.cx);
        let walk = match walk.width {
            Size::Fit { .. } => Walk {
                width: Size::Fixed(self.last_size.map_or(measured, |size| {
                    if size.x > 0.0 {
                        size.x
                    } else {
                        measured
                    }
                })),
                ..walk
            },
            // `Fixed` and `Fill` are already known before drawing, so they are kept live — including the one
            // `set_segments` declared.
            _ => walk,
        };
        self.walk = walk;

        self.draw_bg.fill = track;
        self.draw_bg.border = border;
        self.draw_bg.border_width = 1.0;
        self.draw_bg.radius = radius;
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);

        let rect = self.draw_bg.area().rect(cx.cx);
        // Remember what was actually drawn, for the next frame's `Fit` resolution above.
        if rect.size.x > 0.0 {
            self.last_size = Some(rect.size);
        }
        self.area = self.draw_bg.area();
        let count = self.segments.len();
        let line_box = font * 1.2;

        for (index, label) in self.segments.iter().enumerate() {
            let Some((left, width)) = slot_span(index, count, rect.size.x) else {
                continue;
            };
            // Centred by the **measured** width, so the label is centred by the same number that sized its slot.
            let text_w = self.label_width(cx.cx, label);
            // The selected label reads at full strength and the rest recede, the
            // same rule the checkbox and the radio follow, so a control is
            // scannable without reading every word.
            self.draw_label.color = if self.active == Some(index) {
                strong
            } else {
                muted
            };
            self.draw_label.draw_abs(
                cx,
                dvec2(
                    rect.pos.x + left + (width - text_w) * 0.5,
                    // `rect.pos.y`, not a bare y: the plate above needed the same,
                    // and `mp/pagination.rs` learned it the hard way — a y without
                    // the origin draws at the top of the *window*, which reads
                    // exactly like "it does not draw at all".
                    rect.pos.y + (rect.size.y - line_box) * 0.5,
                ),
                label,
            );
        }

        DrawStep::done()
    }
}

impl MpSegmentedRef {
    pub fn active(&self) -> Option<usize> {
        self.borrow().and_then(|inner| inner.active())
    }

    pub fn selected(&self, actions: &Actions) -> Option<usize> {
        self.borrow().and_then(|inner| inner.selected(actions))
    }

    pub fn set_segments(&self, cx: &mut Cx, segments: Vec<String>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_segments(cx, segments);
        }
    }

    pub fn set_active(&self, cx: &mut Cx, active: Option<usize>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_active(cx, active);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_box_with_no_segments_has_no_slots() {
        // The division-by-zero case, from both directions.
        assert_eq!(slot_at(0.0, 0, 100.0), None);
        assert_eq!(slot_at(50.0, 0, 100.0), None);
        assert_eq!(slot_span(0, 0, 100.0), None);
    }

    #[test]
    fn test_a_zero_width_box_has_no_slots() {
        assert_eq!(slot_at(0.0, 3, 0.0), None);
        assert_eq!(slot_span(0, 3, 0.0), None);
        assert_eq!(slot_span(0, 3, -10.0), None);
    }

    #[test]
    fn test_slots_tile_the_box_with_nothing_left_over() {
        // The property that makes the plate line up: the first slot starts at zero,
        // each slot starts where the last ended, and the last ends at the width.
        let width = 300.0;
        let count = 3;
        let mut x = 0.0;
        for index in 0..count {
            let (left, slot) = slot_span(index, count, width).unwrap();
            assert!(
                (left - x).abs() < 1e-9,
                "slot {index} starts at {left}, expected {x}"
            );
            x = left + slot;
        }
        assert!(
            (x - width).abs() < 1e-9,
            "the slots end at {x}, the box is {width}"
        );
    }

    #[test]
    fn test_slots_tile_a_width_that_does_not_divide_evenly() {
        // 100 across 3 is 33.33…; the tiling property has to survive it, or the
        // plate and the hit test disagree at the right edge by a fraction of a point
        // and the fault shows up only on widths that are not round numbers.
        let width = 100.0;
        let count = 3;
        let mut x = 0.0;
        for index in 0..count {
            let (left, slot) = slot_span(index, count, width).unwrap();
            assert!((left - x).abs() < 1e-9);
            x = left + slot;
        }
        assert!((x - width).abs() < 1e-9);
    }

    #[test]
    fn test_a_slot_past_the_last_one_has_no_span() {
        // A stale index must draw **no** plate rather than a plate in the wrong
        // place — the same rule `palette::remap` follows for a stale cursor.
        assert_eq!(slot_span(3, 3, 300.0), None);
        assert_eq!(slot_span(99, 3, 300.0), None);
        assert!(slot_span(2, 3, 300.0).is_some());
    }

    #[test]
    fn test_the_hit_test_finds_the_slot_a_point_is_in() {
        let (count, width) = (3usize, 300.0);
        assert_eq!(slot_at(0.0, count, width), Some(0));
        assert_eq!(slot_at(99.9, count, width), Some(0));
        assert_eq!(slot_at(100.0, count, width), Some(1), "a boundary is the upper slot");
        assert_eq!(slot_at(199.9, count, width), Some(1));
        assert_eq!(slot_at(200.0, count, width), Some(2));
        assert_eq!(slot_at(299.9, count, width), Some(2));
    }

    #[test]
    fn test_the_hit_test_refuses_a_point_outside_the_box() {
        // A click on the track's rounded corner or a pixel past the right edge
        // selects nothing rather than the nearest segment.
        let (count, width) = (3usize, 300.0);
        assert_eq!(slot_at(-0.1, count, width), None);
        assert_eq!(slot_at(-50.0, count, width), None);
        assert_eq!(slot_at(300.0, count, width), None);
        assert_eq!(slot_at(301.0, count, width), None);
    }

    #[test]
    fn test_the_hit_test_and_the_span_agree_at_every_boundary() {
        // The two functions are the plate and the pointer, and they have to describe
        // the same slots. This is the test that would catch one of them being
        // changed without the other.
        let (count, width) = (4usize, 250.0);
        for index in 0..count {
            let (left, slot) = slot_span(index, count, width).unwrap();
            // The first pixel of the slot, and the last pixel before the next one.
            assert_eq!(slot_at(left, count, width), Some(index));
            assert_eq!(slot_at(left + slot * 0.5, count, width), Some(index));
            if index + 1 < count {
                let next_left = slot_span(index + 1, count, width).unwrap().0;
                assert_eq!(
                    slot_at(next_left - 1e-6, count, width),
                    Some(index),
                    "just below the next slot's edge is still this one"
                );
                assert_eq!(slot_at(next_left, count, width), Some(index + 1));
            }
        }
    }

    #[test]
    fn test_a_nan_or_infinite_position_selects_nothing() {
        // A pointer position that has not been computed yet arrives as a NaN, and a
        // NaN compared against bounds is false, so a naive bounds check would let it
        // through and `floor` of a NaN is a nonsense index.
        assert_eq!(slot_at(f64::NAN, 3, 300.0), None);
        assert_eq!(slot_at(f64::INFINITY, 3, 300.0), None);
        assert_eq!(slot_at(0.0, 3, f64::NAN), None);
    }

    #[test]
    fn test_a_selection_is_clamped_when_the_segment_list_shrinks() {
        // "Split" removed while it was selected: an index of 2 in a two-segment
        // control draws no plate and reports a selection that is not there.
        assert_eq!(valid(Some(2), 2), None);
        assert_eq!(valid(Some(1), 2), Some(1));
        assert_eq!(valid(Some(0), 2), Some(0));
        assert_eq!(valid(Some(0), 0), None);
        assert_eq!(valid(None, 2), None);
    }

    #[test]
    fn test_a_single_segment_fills_the_box() {
        let (left, width) = slot_span(0, 1, 240.0).unwrap();
        assert_eq!(left, 0.0);
        assert_eq!(width, 240.0);
        assert_eq!(slot_at(120.0, 1, 240.0), Some(0));
        assert_eq!(slot_at(240.0, 1, 240.0), None);
    }
}
