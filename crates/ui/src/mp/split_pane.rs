//! `MpSplitPane` — two panes and the divider you drag between them.
//!
//! ## The whole component is one number, so the number is a function
//!
//! A split pane holds **one** piece of state: how wide the leading pane is. Everything else follows from it — the divider is
//! at that width, the trailing pane gets what is left — so the interesting logic is the arithmetic of a drag, and it is
//! extracted rather than written inside an event handler:
//!
//! - [`clamp_left`] keeps **both** panes usable. A drag may not shrink the leading pane below [`MIN_PANE`] nor push the
//!   divider so far right that the trailing one is below it, and the divider's own width is part of that budget — a pane
//!   whose width is measured to the divider's centre would let the divider hang off the window's edge.
//! - [`left_after_drag`] applies a pointer movement to a starting width. **It is a function of the starting pair, not of the
//!   previous frame**: a drag accumulated frame by frame drifts once a clamp bites, because the frames that were clamped are
//!   lost from the running sum and the pane never returns to where the pointer is. Measured from the press, a clamped drag
//!   comes straight back.
//! - A drag on a window narrower than the two minimums cannot satisfy both, and [`clamp_left`] answers the **smaller** pane
//!   rather than a negative one — a window that small is a degenerate layout, not a place to draw a pane backwards.
//!
//! ## The divider is a hit region wider than it looks
//!
//! [`DIVIDER`] is what is *drawn*; [`DIVIDER_HIT`] is what is *grabbed*, and the gap between them is the difference between a
//! divider that is easy to hit and one that is a hairline you have to aim at. Both are constants because a caller choosing
//! them is choosing a hit target's tolerance, which is not a styling decision.
//!
//! ## What is dropped
//!
//! The v2's `f32` widths (geometry here is `f64` throughout, like every other widget in this port) and its
//! `resized(&Actions) -> Option<f32>` accessor, which read the action back out of a batch; the v3 reports
//! `MpSplitPaneAction::Resized` and a caller that wants the accessor can call [`crate::mp::split_pane::resized`].

use makepad_widgets::*;

/// The divider's drawn width.
pub const DIVIDER: f64 = 6.0;

/// The divider's grab width.
///
/// Wider than it is drawn: a 6-point divider is easy to see and hard to hit, and the difference between the two is what makes
/// it draggable without aiming.
pub const DIVIDER_HIT: f64 = 12.0;

/// The narrowest either pane may become.
pub const MIN_PANE: f64 = 80.0;

/// The leading pane's width as it is dragged to, clamped so both panes stay usable.
///
/// A window narrower than `2 * MIN_PANE + DIVIDER` cannot satisfy both minimums, and the answer is the **smaller** pane rather
/// than a negative one: a window that small is a degenerate layout, and the honest thing to draw is two narrow panes rather
/// than one pane running backwards off the window.
pub fn clamp_left(desired: f64, width: f64) -> f64 {
    if !width.is_finite() || width <= 0.0 {
        return 0.0;
    }
    // **Three cases, and my first version collapsed two of them.** `NaN` is *unknown*, so the safe answer is the minimum —
    // it keeps the other pane usable. But `+inf` is not unknown: it means "as wide as possible", and `f64::clamp` already
    // takes it to the maximum, so special-casing it to the minimum would silently collapse a pane a caller asked to
    // maximise. Only `NaN` needs handling.
    let requested = if desired.is_nan() { MIN_PANE } else { desired };
    // The divider's width is part of the budget: a pane measured to the divider's centre would let the divider hang off the
    // window's edge.
    let most = width - MIN_PANE - DIVIDER;
    if most < MIN_PANE {
        // No width satisfies both minimums, so the largest pane that leaves the trailing one something is the answer.
        return most.max(0.0);
    }
    requested.clamp(MIN_PANE, most)
}

/// The leading pane's width after dragging the divider from `start_x` to `now_x`.
///
/// **Measured from where the drag began, not from the previous frame.** A frame-by-frame accumulation drifts as soon as a
/// clamp bites: the frames spent against the limit are discarded from the sum, so dragging past the minimum and back leaves
/// the pane short of where the pointer is. From the press, the clamp is a lossless limit — go too far and come back, and the
/// pane is where it would have been.
pub fn left_after_drag(start_left: f64, start_x: f64, now_x: f64, width: f64) -> f64 {
    if !start_x.is_finite() || !now_x.is_finite() {
        return clamp_left(start_left, width);
    }
    clamp_left(start_left + (now_x - start_x), width)
}

/// What the trailing pane gets.
///
/// Clamped at zero rather than allowed negative, which is the same degenerate case `clamp_left` handles.
pub fn right_width(width: f64, left: f64) -> f64 {
    if !width.is_finite() || width <= 0.0 {
        return 0.0;
    }
    (width - left - DIVIDER).max(0.0)
}

/// The divider's rect.
pub fn divider_rect(_width: f64, left: f64, origin: DVec2, height: f64) -> Rect {
    Rect {
        pos: dvec2(origin.x + left, origin.y),
        size: dvec2(DIVIDER, height.max(0.0)),
    }
}

/// Whether an `x` is on the divider, for taking hold of it.
///
/// `x` is relative to the split's leading edge. The region is centred on the divider and [`DIVIDER_HIT`] wide, so a press a
/// few points to either side takes hold of it — and a press far away does not.
pub fn on_divider(x: f64, width: f64, left: f64) -> bool {
    if !x.is_finite() || x < 0.0 {
        return false;
    }
    let centre = left + DIVIDER * 0.5;
    (x - centre).abs() <= DIVIDER_HIT * 0.5 && x < width.max(0.0)
}

/// The slot ids a split's tree is addressed by — the same three the widget it replaces used.
pub const LEFT: LiveId = live_id!(left);
pub const DIVIDER_ID: LiveId = live_id!(divider);
pub const RIGHT: LiveId = live_id!(right);

/// What a split reports.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum MpSplitPaneAction {
    /// The divider was dragged to this leading width.
    Resized(f64),
    #[default]
    None,
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    mod.mp.DrawMpSplitDivider = #(DrawMpSplitDivider::script_shader(vm)){
        ..mod.draw.DrawQuad

        radius: 0.0
        plate: #x00000000
        hover_color: #x00000000
        live: 0.0

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let sz = self.rect_size
            sdf.box(0.0, 0.0, sz.x, sz.y, self.radius)
            sdf.fill_keep(mix(self.plate, self.hover_color, self.live))
            return sdf.result
        }
    }

    mod.mp.MpSplitPaneBase = #(MpSplitPane::register_widget(vm))

    mod.mp.MpSplitPane = set_type_default() do mod.mp.MpSplitPaneBase{
        width: Fill
        height: Fill

        left_width: 264.0
    }
}

/// The divider's plate.
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpSplitDivider {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    radius: f32,
    #[live]
    plate: Vec4f,
    #[live]
    hover_color: Vec4f,
    #[live]
    live: f32,
}

/// Two panes and the divider between them.
#[derive(Script, ScriptHook, Widget, Animator)]
pub struct MpSplitPane {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    /// The animator the control signals need.
    #[apply_default]
    animator: Animator,
    #[redraw]
    #[live]
    draw_divider: DrawMpSplitDivider,
    /// The leading pane's width. `#[live]` because it is the caller's declared starting width — and the DSL's value is
    /// applied only as a start, so a drag is not undone by a theme change: the dragged width lives in [`MpSplitPane::left`].
    #[live]
    left_width: f64,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[rust]
    left: f64,
    /// Where the pointer was when the drag began, and the width it began at — **a pair**, because the drag is measured from
    /// the press rather than accumulated. See [`left_after_drag`].
    #[rust]
    drag: Option<(DVec2, f64)>,
    #[rust]
    hovered: bool,
    #[rust]
    origin: DVec2,
    #[rust]
    area: Area,
}

impl MpSplitPane {
    /// The leading pane's width.
    pub fn left_width_now(&self, width: f64) -> f64 {
        clamp_left(self.left, width)
    }

    pub fn set_left_width(&mut self, cx: &mut Cx, width: f64, total: f64) {
        let clamped = clamp_left(width, total);
        if clamped != self.left {
            self.left = clamped;
            self.redraw(cx);
        }
    }

    pub fn is_dragging(&self) -> bool {
        self.drag.is_some()
    }
}

impl Widget for MpSplitPane {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let signals = crate::mp::control::handle(&mut self.animator, cx, event, self.area);
        if signals.redraw {
            self.redraw(cx);
        }
        let total = self.area.rect(cx).size.x;
        if signals.down {
            let x = signals.pointer.x - self.origin.x;
            // **The grab region, not the drawn width.** A press on the divider takes hold; a press elsewhere is the panes'
            // business, so it is passed on rather than swallowed.
            if on_divider(x, total, self.left_width_now(total)) {
                self.drag = Some((dvec2(signals.pointer.x, signals.pointer.y), self.left_width_now(total)));
                return;
            }
            self.view_handle(cx, event, scope);
            return;
        }
        if signals.moved {
            if let Some((start, start_left)) = self.drag {
                let next = left_after_drag(start_left, start.x, signals.pointer.x, total);
                if next != self.left {
                    self.left = next;
                    self.redraw(cx);
                    cx.widget_action(self.uid, MpSplitPaneAction::Resized(next));
                }
                return;
            }
            self.view_handle(cx, event, scope);
            return;
        }
        if signals.up {
            self.drag = None;
            return;
        }
        // Hover, from a bare move — the only event that carries a position without a gesture.
        if let Event::MouseMove(me) = event {
            let hovered = on_divider(me.abs.x - self.origin.x, total, self.left_width_now(total));
            if hovered != self.hovered {
                self.hovered = hovered;
                self.redraw(cx);
            }
        }
        self.view_handle(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (plate, hover_color) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            (theme.paint.divider, theme.paint.element_hover)
        };
        let placed = cx.walk_turtle(walk);
        self.origin = placed.pos;
        self.area = self.draw_divider.area();
        let total = placed.size.x;
        // The DSL's declared starting width is applied on the first paint, so a caller's `left_width:` means what it says and
        // a later drag is not overwritten by a re-apply.
        if self.left <= 0.0 && self.left_width > 0.0 {
            self.left = clamp_left(self.left_width, total);
        }
        let left = self.left_width_now(total);
        let height = placed.size.y;

        // **The three children drawn at computed positions**, because a split's whole job is to decide widths — the panes
        // cannot be laid out by a flow, since the flow would use their declared widths instead of the dragged one.
        for (id, x, w) in [
            (LEFT, 0.0, left),
            (RIGHT, left + DIVIDER, right_width(total, left)),
        ] {
            let child = self.view(cx.cx, &[id]);
            let _ = child.draw_walk(
                cx,
                scope,
                Walk::fixed(w, height).with_abs_pos(dvec2(placed.pos.x + x, placed.pos.y)),
            );
        }
        let rect = divider_rect(total, left, placed.pos, height);
        self.draw_divider.plate = plate;
        self.draw_divider.hover_color = hover_color;
        self.draw_divider.live = if self.hovered || self.is_dragging() { 1.0 } else { 0.0 };
        self.draw_divider.radius = 0.0;
        self.draw_divider.draw_abs(cx, rect);
        DrawStep::done()
    }
}

impl MpSplitPane {
    /// Pass an event on to the two panes.
    ///
    /// **This was an empty function for one revision, and an empty function is the worst kind of stub**: it compiles, it
    /// reports nothing, and the panes simply stop receiving events — a scroll view inside a split would not scroll and the
    /// only symptom would be a component that quietly does not work. It is written out here rather than left as a placeholder
    /// because "does nothing" is not a smaller version of "passes the event on"; it is a different behaviour.
    ///
    /// Only the two panes: the divider is this widget's own, and it is handled above before this is reached.
    fn view_handle(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        for id in [LEFT, RIGHT] {
            self.view(cx, &[id]).handle_event(cx, event, scope);
        }
    }
}

impl MpSplitPaneRef {
    pub fn set_left_width(&self, cx: &mut Cx, width: f64, total: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_left_width(cx, width, total);
        }
    }

    pub fn is_dragging(&self) -> bool {
        self.borrow().map(|inner| inner.is_dragging()).unwrap_or(false)
    }

    /// The width a `Resized` action carries, for a caller reading an event batch.
    pub fn resized(&self, actions: &Actions) -> Option<f64> {
        actions
            .find_widget_action(self.widget_uid())
            .and_then(|action| match action.cast::<MpSplitPaneAction>() {
                MpSplitPaneAction::Resized(width) => Some(width),
                MpSplitPaneAction::None => None,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_both_panes_keep_a_usable_width_and_the_divider_is_part_of_the_budget() {
        // A drag may not shrink the leading pane below `MIN_PANE` nor push the divider so far right that the trailing one is
        // below it — and the divider's own width is inside that budget, because a pane measured to the divider's *centre*
        // would let the divider hang off the window's edge.
        let width = 1000.0;
        assert_eq!(clamp_left(300.0, width), 300.0);
        assert_eq!(clamp_left(0.0, width), MIN_PANE, "the leading pane collapsed");
        assert_eq!(clamp_left(-50.0, width), MIN_PANE);
        // The widest the leading pane may be leaves exactly MIN_PANE plus the divider on the other side.
        assert_eq!(clamp_left(10_000.0, width), width - MIN_PANE - DIVIDER);
        assert_eq!(
            right_width(width, clamp_left(10_000.0, width)),
            MIN_PANE,
            "the trailing pane was squeezed below its minimum"
        );
        // ...and the same at the other end.
        assert_eq!(right_width(width, clamp_left(0.0, width)), width - MIN_PANE - DIVIDER);
    }

    #[test]
    fn test_a_window_too_narrow_for_both_minimums_gives_two_narrow_panes_not_a_negative_one() {
        // A window smaller than `2 * MIN_PANE + DIVIDER` cannot satisfy both minimums. The answer is the **smaller** pane
        // rather than a negative one: a degenerate layout should draw two narrow panes, not one running backwards off the
        // window.
        let narrow = MIN_PANE + 10.0;
        let left = clamp_left(MIN_PANE, narrow);
        assert!(left >= 0.0, "a negative pane");
        assert!(left <= narrow, "a pane wider than its window");
        assert!(right_width(narrow, left) >= 0.0);
        // The absurd cases are zero rather than a `NaN` every rect would inherit.
        assert_eq!(clamp_left(100.0, 0.0), 0.0);
        assert_eq!(clamp_left(100.0, -10.0), 0.0);
        assert_eq!(clamp_left(100.0, f64::NAN), 0.0);
        assert_eq!(clamp_left(100.0, f64::INFINITY), 0.0);
        assert_eq!(clamp_left(f64::NAN, 500.0), MIN_PANE, "a `NaN` requested width is unknown, so the minimum");
        // **`+inf` is not `NaN`.** It means "as wide as possible", which is the *maximum* — and my first version treated
        // the two alike, silently collapsing a pane a caller asked to maximise.
        assert_eq!(clamp_left(f64::INFINITY, 500.0), 500.0 - MIN_PANE - DIVIDER);
        assert_eq!(clamp_left(f64::NEG_INFINITY, 500.0), MIN_PANE, "-inf is the other end");
        assert_ne!(clamp_left(f64::INFINITY, 500.0), clamp_left(f64::NAN, 500.0));
        assert_eq!(right_width(f64::NAN, 100.0), 0.0);
    }

    #[test]
    fn test_a_drag_is_measured_from_the_press_so_a_clamped_drag_comes_back() {
        // **The rule that keeps a drag from drifting.** Accumulating a delta frame by frame loses the frames spent against a
        // limit, so dragging past the minimum and back leaves the pane short of where the pointer is. Measured from the
        // press, the clamp is a lossless limit.
        let width = 1000.0;
        let (start_left, start_x) = (300.0, 300.0);
        assert_eq!(left_after_drag(start_left, start_x, 400.0, width), 400.0, "a plain move");
        assert_eq!(left_after_drag(start_left, start_x, 200.0, width), 200.0, "backwards");
        // Far past the minimum, and then back to a position inside the range: the pane is where the pointer is, not short of
        // it by the amount the clamp swallowed.
        let _slammed = left_after_drag(start_left, start_x, -500.0, width);
        assert_eq!(_slammed, MIN_PANE);
        assert_eq!(
            left_after_drag(start_left, start_x, 320.0, width),
            320.0,
            "the drag did not come back — it is being accumulated"
        );
        // ...and the same at the far end.
        assert_eq!(left_after_drag(start_left, start_x, 9_000.0, width), width - MIN_PANE - DIVIDER);
        assert_eq!(left_after_drag(start_left, start_x, 350.0, width), 350.0);
        // A non-finite pointer leaves the pane where it was rather than moving it somewhere unreachable.
        assert_eq!(left_after_drag(start_left, start_x, f64::NAN, width), clamp_left(start_left, width));
        assert_eq!(left_after_drag(start_left, f64::NAN, 400.0, width), clamp_left(start_left, width));
    }

    #[test]
    fn test_the_divider_is_grabbed_wider_than_it_is_drawn() {
        // A 6-point divider is easy to see and hard to hit, and the difference between the two constants is what makes it
        // draggable without aiming. The grab region is centred on the drawn divider, so it extends to either side.
        let (width, left) = (1000.0, 300.0);
        let centre = left + DIVIDER * 0.5;
        assert!(on_divider(centre, width, left), "the divider itself");
        assert!(on_divider(centre - DIVIDER_HIT * 0.5, width, left), "the left edge of the grab region");
        assert!(on_divider(centre + DIVIDER_HIT * 0.5, width, left), "the right edge");
        assert!(!on_divider(centre - DIVIDER_HIT * 0.5 - 0.1, width, left), "just past it");
        assert!(!on_divider(centre + DIVIDER_HIT * 0.5 + 0.1, width, left));
        assert!(DIVIDER_HIT > DIVIDER, "the grab region is not wider than the divider");
        // A press on a pane is not a press on the divider, which is what lets the panes keep their own events.
        assert!(!on_divider(10.0, width, left));
        assert!(!on_divider(900.0, width, left));
        // Non-finite and negative points are not on it.
        assert!(!on_divider(f64::NAN, width, left));
        assert!(!on_divider(-1.0, width, left));
        assert!(!on_divider(centre, width, f64::NAN), "a `NaN` divider is nowhere");
    }

    #[test]
    fn test_the_three_bands_tile_the_split_exactly() {
        // **The property that keeps the drawing from overlapping or leaving a gap**: the leading pane, the divider and the
        // trailing pane add up to the whole width, at every width limit.
        for width in [200.0f64, 400.0, 1000.0] {
            for requested in [-100.0f64, 0.0, 150.0, 300.0, 800.0, 5000.0] {
                let left = clamp_left(requested, width);
                let right = right_width(width, left);
                assert!(
                    (left + DIVIDER + right - width).abs() < 1e-9,
                    "width {width}, requested {requested}: {left} + {DIVIDER} + {right} is not {width}"
                );
                assert!(left >= 0.0 && right >= 0.0);
            }
        }
        // And the divider's rect starts where the leading pane ends.
        let origin = dvec2(20.0, 40.0);
        let rect = divider_rect(1000.0, 300.0, origin, 500.0);
        assert_eq!(rect.pos.x, origin.x + 300.0);
        assert_eq!(rect.pos.y, origin.y);
        assert_eq!(rect.size.x, DIVIDER);
        assert_eq!(rect.pos.x + rect.size.x, origin.x + 300.0 + DIVIDER);
    }

    #[test]
    fn test_defaults_are_the_ones_the_widget_it_replaces_used() {
        // The numbers a caller's layout was tuned against, kept: a port that changed the divider's width or the grab
        // tolerance would move every divider in the app for no reason anybody could name.
        assert_eq!(DIVIDER, 6.0);
        assert_eq!(MIN_PANE, 80.0);
        assert!(DIVIDER_HIT >= DIVIDER * 2.0, "the grab region should be generously wider");
        assert_eq!(
            SOURCE_SLOTS,
            ["left", "divider", "right"],
            "the slot ids a caller writes into changed"
        );
        assert_eq!(LEFT, live_id!(left));
        assert_eq!(DIVIDER_ID, live_id!(divider));
        assert_eq!(RIGHT, live_id!(right));
    }

    const SOURCE_SLOTS: [&str; 3] = ["left", "divider", "right"];
}
