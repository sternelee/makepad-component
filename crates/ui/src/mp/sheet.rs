//! `MpSheet` — a panel pinned to an edge, which travels in rather than growing in place.
//!
//! ## The argument is an *extent*, not a width
//!
//! bezel makes this point by naming it: a sheet spans the edge it is pinned to and takes `extent` **across** it, so on
//! [`SheetSide::Bottom`] the extent is a height. A parameter called `width` would be wrong for one of the three sides, and the
//! kind of wrong that a caller discovers by measuring.
//!
//! ## It travels, so its motion is not a dialog's
//!
//! A dialog grows in place; a sheet crosses the window from an edge. That is a different motion, and reading a travel at a
//! grow's duration arrives before the eye has followed it — which is why `makepad_motion` gained `SHEET_IN` rather than this
//! component borrowing `DIALOG_IN`, and why the catalog asserts the sheet's entry is the slower one.
//!
//! ## Only the free corners are rounded, and makepad has no per-corner radius
//!
//! A panel attached to an edge is part of the frame on that edge: rounding its pinned corners shows the page behind through
//! notches that were never notches. A sheet on the left therefore rounds its **right** corners only, and makepad's
//! `border_radius` rounds all four.
//!
//! So the plate is drawn **past the pinned edge by its own radius** ([`plate_overshoot`]), which puts the two corners that
//! must stay square outside the panel's rect where the clip removes them. The arithmetic is a function with a test, because
//! "which way does it overshoot" is exactly the sort of thing that is right for one side and wrong for another.
//!
//! ## Dismissal is the caller's, and it is asked about the panel's rect
//!
//! [`is_outside`] answers whether a point is off the panel. A click there is a dismissal, and the caller decides what that
//! means — the same division every surface in this library keeps: the widget reports, the application acts.

use makepad_widgets::*;

/// Which edge a sheet is pinned to.
///
/// bezel's three, and its note on the third: `Bottom` is the shape a phone puts a picker or a share list in, and what a narrow
/// window wants instead of a side panel that leaves no room for the page behind it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SheetSide {
    #[default]
    Left,
    Right,
    Bottom,
}

/// A sheet's progress, held between 0 (fully off-screen) and 1 (seated).
///
/// A caller animating one passes whatever its animator produced; a `NaN` is treated as seated rather than as off-screen,
/// because a sheet that failed to arrive is worse than one that arrived without an animation.
pub fn clamp_progress(t: f64) -> f64 {
    if t.is_nan() {
        return 1.0;
    }
    t.clamp(0.0, 1.0)
}

/// How far the panel is set back from its seat at progress `t`.
///
/// **Negative**, and that is the point: at `t = 0` the inset is `-extent`, which puts the panel entirely outside the window on
/// the edge it travels from. At `t = 1` it is zero and the panel is seated.
pub fn seat_inset(extent: f64, t: f64) -> f64 {
    let extent = if extent.is_finite() { extent.max(0.0) } else { 0.0 };
    extent * (clamp_progress(t) - 1.0)
}

/// The panel's rect at progress `t`, for a viewport of `viewport`.
pub fn panel_rect(side: SheetSide, extent: f64, viewport: DVec2, t: f64) -> Rect {
    let extent = if extent.is_finite() && extent > 0.0 { extent } else { 0.0 };
    let width = if viewport.x.is_finite() { viewport.x.max(0.0) } else { 0.0 };
    let height = if viewport.y.is_finite() { viewport.y.max(0.0) } else { 0.0 };
    let inset = seat_inset(extent, t);
    match side {
        // Spans the edge's whole length, and `extent` across it.
        SheetSide::Left => Rect {
            pos: dvec2(inset, 0.0),
            size: dvec2(extent, height),
        },
        SheetSide::Right => Rect {
            pos: dvec2(width - extent - inset, 0.0),
            size: dvec2(extent, height),
        },
        SheetSide::Bottom => Rect {
            // Downwards from the bottom edge, so a negative inset pushes it off the bottom of the window.
            pos: dvec2(0.0, height - extent - inset),
            size: dvec2(width, extent),
        },
    }
}

/// Which corners of the panel are **free** — the ones that round — as `[top-left, top-right, bottom-right, bottom-left]`.
///
/// A panel pinned to an edge is part of the frame there, so the corners on the pinned side stay square: rounding them shows the
/// page behind through notches that were never notches. This is the rule as data, so the shader's overshoot can be checked
/// against it rather than agreeing with it by coincidence.
pub fn free_corners(side: SheetSide) -> [bool; 4] {
    match side {
        // Pinned on the left, so the corners away from it are free.
        SheetSide::Left => [false, true, true, false],
        SheetSide::Right => [true, false, false, true],
        // Pinned at the bottom, so the top two are free.
        SheetSide::Bottom => [true, true, false, false],
    }
}

/// How far past the panel's own rect the plate is drawn, as `(dx, dy)` for its origin plus the same added to its size.
///
/// **The trick that stands in for a per-corner radius, which makepad has not got.** Drawing the rounded plate `radius` further
/// out on the pinned edge puts the two corners that must stay square outside the panel, where the clip removes them — leaving
/// a plate whose pinned corners are square and whose free ones are round.
pub fn plate_overshoot(side: SheetSide, radius: f64) -> (f64, f64) {
    let radius = if radius.is_finite() && radius > 0.0 { radius } else { 0.0 };
    match side {
        SheetSide::Left => (-radius, 0.0),
        SheetSide::Right => (radius, 0.0),
        // Pinned at the bottom, so the plate is drawn **down** past the panel's bottom edge — the top corners are the free ones.
        SheetSide::Bottom => (0.0, radius),
    }
}

/// Whether a point is outside the panel, and therefore a dismissal.
///
/// Asked about the **panel's rect** rather than the whole surface: a click inside is the content's, and a sheet that dismissed
/// on any click would close as soon as its own body was used.
pub fn is_outside(point: DVec2, panel: Rect) -> bool {
    if !point.x.is_finite() || !point.y.is_finite() {
        return false;
    }
    point.x < panel.pos.x
        || point.x >= panel.pos.x + panel.size.x
        || point.y < panel.pos.y
        || point.y >= panel.pos.y + panel.size.y
}

/// What a sheet reports.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum MpSheetAction {
    /// A press outside the panel: the caller decides what dismissal means.
    Dismissed,
    #[default]
    None,
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    /// The panel's plate. `shift_x`/`shift_y` are the overshoot: the plate is drawn past the pinned edge so its corners
    /// there fall outside the clip.
    mod.mp.DrawMpSheet = #(DrawMpSheet::script_shader(vm)){
        ..mod.draw.DrawQuad

        radius: 12.0
        shift_x: 0.0
        shift_y: 0.0
        plate: #x00000000
        border_color: #x00000000

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let sz = self.rect_size
            sdf.box(
                self.shift_x,
                self.shift_y,
                sz.x,
                sz.y,
                self.radius
            )
            sdf.fill_keep(self.plate)
            sdf.stroke(self.border_color, 1.0)
            return sdf.result
        }
    }

    mod.mp.MpSheetBase = #(MpSheet::register_widget(vm))

    mod.mp.MpSheet = set_type_default() do mod.mp.MpSheetBase{
        width: Fill
        height: Fill

        /// The panel's content, laid out inside it.
        content := View{
            width: Fill
            height: Fill
            flow: Down
        }

        animator: Animator{
            // **The catalog's sheet motion, not a dialog's.** A sheet travels and a dialog grows: reading a travel at a grow's
            // duration arrives before the eye has followed it, which is why `makepad_motion` has its own entry.
            seat: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward{duration: mod.motion.sheet_in.duration}}
                    redraw: true
                    // **`draw_bg.progress`, not a bare `progress`** — the animator applies to a property path, and the shader is
                    // this widget's own `#[live]` field, so its instance is the one Rust can read back. The same form
                    // `mp/control.rs` uses for its hover and press tracks.
                    apply: {draw_bg: {progress: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward{duration: mod.motion.sheet_in.duration}}
                    redraw: true
                    apply: {draw_bg: {progress: 1.0}}
                }
            }
        }
    }
}

/// The panel's plate.
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpSheet {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    radius: f32,
    /// The panel's progress, 0 off-screen and 1 seated. **On the shader, because that is the property path the animator can
    /// drive** — and because a `#[live]` instance field on a widget's own shader is readable from Rust, unlike a child DSL
    /// view's.
    #[live]
    progress: f32,
    #[live]
    shift_x: f32,
    #[live]
    shift_y: f32,
    #[live]
    plate: Vec4f,
    #[live]
    border_color: Vec4f,
}

/// A panel pinned to an edge.
#[derive(Script, ScriptHook, Widget, Animator)]
pub struct MpSheet {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[apply_default]
    animator: Animator,
    #[live]
    draw_bg: DrawMpSheet,
    /// Which edge it is pinned to.
    #[live]
    side: f64,
    /// How far it reaches across that edge — **a width on the sides and a height on the bottom**, which is why it is an extent.
    #[live]
    extent: f64,
    #[rust]
    open: bool,
    #[rust]
    origin: DVec2,
    #[rust]
    area: Area,
}

impl MpSheet {
    fn side(&self) -> SheetSide {
        match self.side.round() as i64 {
            1 => SheetSide::Right,
            2 => SheetSide::Bottom,
            _ => SheetSide::Left,
        }
    }

    /// Show or hide it, animating through the catalog's sheet motion.
    pub fn set_open(&mut self, cx: &mut Cx, open: bool) -> bool {
        if self.open == open {
            return false;
        }
        self.open = open;
        self.view.set_visible(cx, open);
        self.animator_play(cx, if open { ids!(seat.on) } else { ids!(seat.off) });
        self.redraw(cx);
        true
    }

    pub fn open(&mut self, cx: &mut Cx) -> bool {
        self.set_open(cx, true)
    }

    pub fn close(&mut self, cx: &mut Cx) -> bool {
        self.set_open(cx, false)
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn side_now(&self) -> SheetSide {
        self.side()
    }

    /// The panel's rect in screen coordinates, at the current progress.
    pub fn panel(&self, cx: &mut Cx) -> Rect {
        let viewport = self.area.rect(cx).size;
        let rect = panel_rect(self.side(), self.extent, viewport, self.draw_bg.progress as f64);
        Rect {
            pos: self.origin + rect.pos,
            size: rect.size,
        }
    }
}

impl Widget for MpSheet {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if !self.open {
            return;
        }
        self.view.handle_event(cx, event, scope);
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
        let panel = self.panel(cx);
        if let Hit::FingerUp(fe) = event.hits(cx, self.area) {
            if fe.is_over && is_outside(fe.abs, panel) {
                cx.widget_action(self.widget_uid(), MpSheetAction::Dismissed);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if !self.open && self.draw_bg.progress >= 1.0 {
            return DrawStep::done();
        }
        let (plate, border) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            (theme.paint.surface_card, theme.paint.border)
        };
        let placed = cx.walk_turtle(walk);
        self.origin = placed.pos;
        self.area = self.draw_bg.area();
        let panel = panel_rect(self.side(), self.extent, placed.size, self.draw_bg.progress as f64);
        // **The plate is drawn past the pinned edge**, so the corners that must stay square fall outside the panel's rect and
        // the clip removes them. That is how a panel gets two rounded corners out of a shader that rounds four.
        let (dx, dy) = plate_overshoot(self.side(), 12.0);
        let plate_rect = Rect {
            pos: placed.pos + panel.pos + dvec2(dx, dy),
            size: panel.size + dvec2(-dx * 2.0, -dy * 2.0),
        };
        self.draw_bg.radius = 12.0;
        self.draw_bg.shift_x = dx as f32;
        self.draw_bg.shift_y = dy as f32;
        self.draw_bg.plate = plate;
        self.draw_bg.border_color = border;
        self.draw_bg.draw_abs(cx, plate_rect);

        let content = self.view.view(cx.cx, ids!(content));
        let _ = content.draw_walk(
            cx,
            scope,
            Walk::fixed(panel.size.x, panel.size.y).with_abs_pos(placed.pos + panel.pos),
        );
        DrawStep::done()
    }
}

impl MpSheetRef {
    pub fn set_open(&self, cx: &mut Cx, open: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_open(cx, open);
        }
    }

    pub fn open(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.open(cx);
        }
    }

    pub fn close(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.close(cx);
        }
    }

    pub fn is_open(&self) -> bool {
        self.borrow().map(|inner| inner.is_open()).unwrap_or(false)
    }

    /// Whether the sheet reported a dismissal, for a caller reading an event batch.
    pub fn dismissed(&self, actions: &Actions) -> bool {
        actions
            .find_widget_action(self.widget_uid())
            .is_some_and(|action| matches!(action.cast::<MpSheetAction>(), MpSheetAction::Dismissed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEWPORT: DVec2 = DVec2 { x: 1000.0, y: 800.0 };

    #[test]
    fn test_the_panel_travels_from_outside_its_edge_to_seated() {
        // At `t = 0` the panel is **entirely** outside the window on the edge it travels from; at `t = 1` it is seated. The
        // sign is the whole of it: an inset of `+extent` would put a left-pinned sheet *inside* the page at rest.
        assert_eq!(seat_inset(320.0, 1.0), 0.0, "seated");
        assert_eq!(seat_inset(320.0, 0.0), -320.0, "not off-screen");
        assert_eq!(seat_inset(320.0, 0.5), -160.0, "halfway");
        // A panel at rest is exactly its own extent off the edge.
        let off = panel_rect(SheetSide::Left, 320.0, VIEWPORT, 0.0);
        assert_eq!(off.pos.x + off.size.x, 0.0, "the off-screen panel still shows");
        // ...and seated it is at the edge, spanning the viewport's length.
        let seated = panel_rect(SheetSide::Left, 320.0, VIEWPORT, 1.0);
        assert_eq!(seated.pos.x, 0.0);
        assert_eq!(seated.size, dvec2(320.0, 800.0));
        // A right-pinned sheet travels the other way.
        let right_off = panel_rect(SheetSide::Right, 320.0, VIEWPORT, 0.0);
        assert_eq!(right_off.pos.x, 1000.0, "the right panel is not off the right edge");
        let right_seated = panel_rect(SheetSide::Right, 320.0, VIEWPORT, 1.0);
        assert_eq!(right_seated.pos.x + right_seated.size.x, 1000.0);
        // And a bottom one spans the width and travels down.
        let bottom_seated = panel_rect(SheetSide::Bottom, 300.0, VIEWPORT, 1.0);
        assert_eq!(bottom_seated.pos, dvec2(0.0, 500.0));
        assert_eq!(bottom_seated.size, dvec2(1000.0, 300.0));
        let bottom_off = panel_rect(SheetSide::Bottom, 300.0, VIEWPORT, 0.0);
        assert_eq!(bottom_off.pos.y, 800.0, "the bottom sheet is not off the bottom edge");
    }

    #[test]
    fn test_the_extent_is_across_the_edge_which_is_why_it_is_not_called_a_width() {
        // On the two sides the extent is a width and on the bottom it is a **height** — one argument meaning the same thing
        // on all three, which a parameter named `width` would be lying about.
        let left = panel_rect(SheetSide::Left, 300.0, VIEWPORT, 1.0);
        assert_eq!(left.size.x, 300.0);
        assert_ne!(left.size.y, 300.0, "a left sheet is not 300 tall");
        let bottom = panel_rect(SheetSide::Bottom, 300.0, VIEWPORT, 1.0);
        assert_eq!(bottom.size.y, 300.0);
        assert_ne!(bottom.size.x, 300.0, "a bottom sheet is not 300 wide");
        // The panel always spans the edge it is pinned to.
        assert_eq!(left.size.y, VIEWPORT.y);
        assert_eq!(bottom.size.x, VIEWPORT.x);
    }

    #[test]
    fn test_only_the_corners_away_from_the_pinned_edge_are_free() {
        // A panel pinned to an edge is part of the frame there: rounding its pinned corners shows the page behind through
        // notches that were never notches. The rule as data, so the plate's overshoot can be checked against it.
        assert_eq!(free_corners(SheetSide::Left), [false, true, true, false]);
        assert_eq!(free_corners(SheetSide::Right), [true, false, false, true]);
        assert_eq!(free_corners(SheetSide::Bottom), [true, true, false, false]);
        // Exactly two corners are free on every side, which is the shape of an edge-pinned panel.
        for side in [SheetSide::Left, SheetSide::Right, SheetSide::Bottom] {
            assert_eq!(free_corners(side).iter().filter(|free| **free).count(), 2, "{side:?}");
            // And the two that are free are adjacent — a panel with opposite corners rounded would look like a mint.
            let free = free_corners(side);
            assert!(!(free[0] && free[2]), "{side:?} rounds opposite corners");
            assert!(!(free[1] && free[3]), "{side:?} rounds opposite corners");
        }
    }

    #[test]
    fn test_the_plate_is_drawn_past_the_pinned_edge_so_its_corners_there_are_square() {
        // **The trick that stands in for a per-corner radius, which makepad has not got.** Drawing the plate further out on
        // the pinned edge puts the corners that must stay square outside the panel, where the clip removes them.
        let radius = 12.0;
        // Pinned on the left: the plate is drawn leftwards, so its left corners are outside.
        assert_eq!(plate_overshoot(SheetSide::Left, radius), (-radius, 0.0));
        assert_eq!(plate_overshoot(SheetSide::Right, radius), (radius, 0.0));
        // Pinned at the bottom: the plate is drawn **down**, because the bottom corners are the pinned ones.
        assert_eq!(plate_overshoot(SheetSide::Bottom, radius), (0.0, radius));
        // The direction agrees with which corners are free: the overshoot is always on the pinned side, and the free corners
        // are always the others.
        for side in [SheetSide::Left, SheetSide::Right, SheetSide::Bottom] {
            let (dx, dy) = plate_overshoot(side, radius);
            let free = free_corners(side);
            if dx < 0.0 {
                assert!(!free[0] && !free[3], "a leftwards overshoot leaves a free left corner: {side:?}");
            }
            if dx > 0.0 {
                assert!(!free[1] && !free[2], "a rightwards overshoot leaves a free right corner: {side:?}");
            }
            if dy > 0.0 {
                assert!(!free[2] && !free[3], "a downwards overshoot leaves a free bottom corner: {side:?}");
            }
        }
        // A nonsense radius overshoots by nothing rather than by a `NaN`.
        assert_eq!(plate_overshoot(SheetSide::Left, f64::NAN), (0.0, 0.0));
        assert_eq!(plate_overshoot(SheetSide::Left, -5.0), (0.0, 0.0));
    }

    #[test]
    fn test_progress_is_held_between_off_screen_and_seated() {
        // A caller passes whatever its animator produced. Out of range is clamped — an overshoot would put the panel past its
        // own seat and show the window's edge through the gap — and a `NaN` is **seated**, because a sheet that failed to
        // arrive is worse than one that arrived without an animation.
        assert_eq!(clamp_progress(0.0), 0.0);
        assert_eq!(clamp_progress(1.0), 1.0);
        assert_eq!(clamp_progress(0.5), 0.5);
        assert_eq!(clamp_progress(-2.0), 0.0);
        assert_eq!(clamp_progress(9.0), 1.0);
        assert_eq!(clamp_progress(f64::NAN), 1.0, "a failed animation left the panel off-screen");
        // And the geometry follows, so a nonsense progress cannot push a panel somewhere unreachable.
        let wild = panel_rect(SheetSide::Left, 320.0, VIEWPORT, f64::NAN);
        assert_eq!(wild.pos.x, 0.0);
    }

    #[test]
    fn test_dismissal_is_asked_about_the_panel_and_not_the_surface() {
        // **A click inside the panel is the content's.** A sheet that dismissed on any click would close as soon as its own
        // body was used — the same reason `MpDialog` takes its hit on the backdrop's own area.
        let panel = Rect {
            pos: dvec2(0.0, 0.0),
            size: dvec2(320.0, 800.0),
        };
        assert!(!is_outside(dvec2(10.0, 10.0), panel), "inside the panel counts as a dismissal");
        assert!(!is_outside(dvec2(319.9, 799.9), panel));
        assert!(is_outside(dvec2(320.0, 400.0), panel), "the first point past the panel");
        assert!(is_outside(dvec2(500.0, 400.0), panel));
        assert!(is_outside(dvec2(-1.0, 400.0), panel));
        assert!(is_outside(dvec2(100.0, -1.0), panel));
        // A non-finite point is not a dismissal: it is not a click anywhere.
        assert!(!is_outside(dvec2(f64::NAN, 10.0), panel));
        assert!(!is_outside(dvec2(10.0, f64::INFINITY), panel));
    }

    #[test]
    fn test_every_panel_rect_is_inside_or_beside_the_viewport_but_never_astray() {
        // The property that ties the three sides together: seated, a panel touches the viewport's far edge exactly; off-screen,
        // it is exactly one extent beyond its own edge; and at any progress it is somewhere between the two.
        for side in [SheetSide::Left, SheetSide::Right, SheetSide::Bottom] {
            let seated = panel_rect(side, 300.0, VIEWPORT, 1.0);
            let off = panel_rect(side, 300.0, VIEWPORT, 0.0);
            match side {
                SheetSide::Left => {
                    assert_eq!(seated.pos.x, 0.0);
                    assert_eq!(off.pos.x + off.size.x, 0.0);
                }
                SheetSide::Right => {
                    assert_eq!(seated.pos.x + seated.size.x, VIEWPORT.x);
                    assert_eq!(off.pos.x, VIEWPORT.x);
                }
                SheetSide::Bottom => {
                    assert_eq!(seated.pos.y + seated.size.y, VIEWPORT.y);
                    assert_eq!(off.pos.y, VIEWPORT.y);
                }
            }
            // At any progress the panel's size is its extent across and the viewport's length along.
            for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
                let rect = panel_rect(side, 300.0, VIEWPORT, t);
                assert_eq!(rect.size, panel_rect(side, 300.0, VIEWPORT, 1.0).size);
                assert!(rect.pos.x.is_finite() && rect.pos.y.is_finite());
            }
        }
        // A nonsense extent or viewport lays out a panel of no size rather than a `NaN`.
        let bad = panel_rect(SheetSide::Left, f64::NAN, dvec2(f64::NAN, 0.0), 1.0);
        assert_eq!(bad.size, dvec2(0.0, 0.0));
    }
}
