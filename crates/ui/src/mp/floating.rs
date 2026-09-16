//! `MpFloating` — a panel that floats over a page and is dragged around it.
//!
//! A meter, an inspector, a detached preview: content the reader positions rather than the
//! layout does.
//!
//! ## The drag has to be heard on a layer larger than the thing being dragged
//!
//! That is the whole reason this is a component and not a `View` with a drag handler. A panel
//! that listens for the drag **on itself** stalls the moment the pointer outruns a frame: the
//! pointer lands outside a box-sized hitbox, the panel stops hearing moves, and the box is
//! stranded behind the cursor. So the widget lays a **full-size layer** over its container and
//! places the box inside it — the gesture is heard on the layer, which the pointer cannot
//! outrun, and only the box moves. `mp/scroll.rs`'s thumb does the same thing against its
//! track, for the same reason.
//!
//! ## The grab offset is what makes a panel draggable at all
//!
//! A drag that positions the panel's top-left at the pointer **jumps** the moment it starts:
//! grab a panel by its title bar and it snaps so that the pointer is at its corner. So the
//! press records where *inside the panel* it landed, and every move keeps that same point
//! under the pointer. It is also why nothing is clamped: a panel dragged half off the window
//! stays there, and because the grab offset is preserved it can always be dragged back —
//! clamping would fight the reader for no benefit.
//!
//! ## The threshold is the difference between a drag and a shaky click
//!
//! A press that moves a pixel is a click with a hand tremor in it, and a panel that moves on
//! one is a panel that shifts every time its title bar is clicked. So a press becomes a drag
//! only after the pointer has travelled [`Floating::THRESHOLD`] from where it went down.
//!
//! The measurement is from the **press**, not from the previous move: slow drift across many
//! small moves has to accumulate to a drag, because that is what a reader doing it slowly
//! expects — measuring per-move would let a pointer crawl across the panel forever without
//! ever starting the gesture.

use makepad_widgets::*;

/// How far the pointer must travel from the press before a press becomes a drag.
///
/// Chosen, and the order of gpui's own `DRAG_THRESHOLD`, which bezel's `floating.rs` adopts
/// rather than picking its own. The exact number matters much less than there being one: at
/// zero, a panel moves every time its title bar is clicked, which reads as the panel being
/// loose.
pub const DRAG_THRESHOLD: f64 = 3.0;

/// The position and gesture state of a floating panel.
///
/// Pure: it knows where the panel is and what the pointer is doing, and nothing about how
/// either is drawn. That is what makes the four rules above testable with no window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Floating {
    /// The panel's top-left, in the layer's coordinates.
    pos: Vec2d,
    /// Where inside the panel the pointer grabbed it. The point every move keeps under the
    /// pointer.
    grab: Option<Vec2d>,
    /// Where the press landed, for measuring the threshold.
    press_at: Option<Vec2d>,
    /// Whether the press has travelled far enough to be a drag.
    dragging: bool,
}

/// The layer's top-left, which is where a panel with no remembered position belongs.
///
/// Exists because a host needs its state fields constructible before the first event, the same
/// reason `History` has one — and because `(0, 0)` is the one position that means *nothing has
/// been decided yet* rather than a place someone put the panel.
impl Default for Floating {
    fn default() -> Self {
        Self::new(dvec2(0.0, 0.0))
    }
}

impl Floating {
    pub fn new(pos: Vec2d) -> Self {
        Self {
            pos,
            grab: None,
            press_at: None,
            dragging: false,
        }
    }

    pub fn pos(&self) -> Vec2d {
        self.pos
    }

    /// Whether a press is down on this panel.
    pub fn is_held(&self) -> bool {
        self.grab.is_some()
    }

    /// Whether the press has become a drag.
    pub fn is_dragging(&self) -> bool {
        self.dragging
    }

    /// Put the panel somewhere, without a gesture — a restore, a re-centre, a test.
    pub fn set_pos(&mut self, pos: Vec2d) {
        self.pos = pos;
    }

    /// A press at `at`, which is inside the panel's box.
    ///
    /// Records the grab offset — where inside the panel the pointer landed — so the panel
    /// does not jump when the drag starts.
    pub fn press(&mut self, at: Vec2d) {
        self.grab = Some(at - self.pos);
        self.press_at = Some(at);
        self.dragging = false;
    }

    /// The pointer moved to `at`. Returns whether the panel moved.
    ///
    /// Does nothing when no press is down: a pointer passing **over** a panel must not drag
    /// it, which is the difference between a panel and a magnet.
    pub fn move_to(&mut self, at: Vec2d) -> bool {
        let Some(grab) = self.grab else {
            return false;
        };
        if !self.dragging {
            let Some(from) = self.press_at else {
                return false;
            };
            // From the **press**, so slow drift accumulates. See the module doc.
            if (at - from).length() < DRAG_THRESHOLD {
                return false;
            }
            self.dragging = true;
        }
        let moved = at - grab;
        if moved == self.pos {
            return false;
        }
        self.pos = moved;
        true
    }

    /// The pointer was released. The panel keeps where it is.
    pub fn release(&mut self) {
        self.grab = None;
        self.press_at = None;
        self.dragging = false;
    }
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    /// A panel the reader can drag: a raised box with a grip along its top edge.
    ///
    /// The grip is a **separate strip** rather than the whole panel, because a panel whose
    /// every pixel starts a drag cannot hold a control — the first click on a button inside it
    /// would move the panel instead.
    ///
    /// ## The press is heard on the box; the moves are heard on the window
    ///
    /// A press has to be **inside** the panel, or a click anywhere on the page would start
    /// dragging it. But a move must be heard on something larger than the box, or the gesture
    /// stalls the first time the pointer outruns a frame. So [`MpFloating::handle_event`]
    /// takes the press and [`MpFloating::drag`] takes the move and the release, and the app
    /// forwards the second from its root:
    ///
    /// ```ignore
    /// fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
    ///     // heard on the whole window, because the panel is the thing being dragged and
    ///     // cannot also be the thing listening
    ///     self.ui.mp_floating(cx, ids!(inspector)).drag(cx, event);
    ///     self.match_event(cx, event);
    ///     self.ui.handle_event(cx, event, &mut Scope::empty());
    /// }
    /// ```
    mod.mp.MpFloatingBase = #(MpFloating::register_widget(vm))

    mod.mp.MpFloating = set_type_default() do mod.mp.MpFloatingBase{
        width: Fit
        height: Fit
        flow: Down
        spacing: 0

        draw_bg +: {
            color: surface_dialog
            border_color: border
            border_size: 1.0
            border_radius: 8.0
        }

        floating_grip := mod.mp.Row{
            width: Fill
            height: 28
            spacing: 6
            padding: Inset{left: 10.0, right: 10.0}
            align: Align{x: 0.0, y: 0.5}
            draw_bg +: {
                color: surface_raised
                border_color: divider
                border_size: 1.0
                border_radius: 8.0
            }
            floating_title := Label{
                width: Fit
                height: Fit
                draw_text +: {text_style: caption, color: text_muted}
                text: "Panel"
            }
        }
        floating_content := mod.mp.Column{
            width: Fill
            height: Fit
            spacing: 10
            padding: Inset{left: 12.0, right: 12.0, top: 12.0, bottom: 12.0}
        }
    }

    /// The layer a floating panel is dragged inside: full size, and above the content.
    mod.mp.MpFloatingLayer = View{
        width: Fill
        height: Fill
        flow: Overlay
        align: Align{x: 0.0, y: 0.0}
    }
}

/// A panel the reader can drag.
///
/// `#[deref] view` rather than a hand-drawn body, the same shape `MpPopover` uses: the panel's
/// grip and content are the DSL's, and this widget owns only **where it is** and **what the
/// pointer is doing to it**.
#[derive(Script, Widget)]
pub struct MpFloating {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    /// The position and the gesture. `#[rust]` rather than `#[live]`, because an app mutates
    /// it through a setter — the rule `mp/pagination.rs` states and this port has paid for
    /// twice.
    #[rust]
    floating: Floating,
    #[rust]
    area: Area,
}

/// Empty for the same reason `MpSegmented`'s is: no animator to seat, nothing to place.
impl ScriptHook for MpFloating {
    fn on_after_new(&mut self, _vm: &mut ScriptVm) {}
}

impl Widget for MpFloating {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // **The press, and only the press.** A move has to be heard on something larger than
        // the box, so it arrives through `drag` instead — see the DSL's note. Taking the press
        // here is what keeps a click elsewhere on the page from starting a drag.
        // `event.hits` rather than a finger event: Makepad reports fingers as `Hit`s from a
        // hit test against an area, not as variants of `Event` — so the press is taken here
        // because this area is the box, and the move is taken by `drag` because that area is
        // the window.
        if let Hit::FingerDown(fe) = event.hits(cx, self.area) {
            self.floating.press(fe.abs);
            cx.set_key_focus(self.area);
            self.redraw(cx);
        }
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Placed at the floating position rather than in the flow: a panel the reader has
        // dragged is not laid out by its container.
        let walk = walk.with_abs_pos(self.floating.pos());
        self.view.draw_walk_all(cx, scope, walk);
        self.area = self.view.area();
        DrawStep::done()
    }
}

impl MpFloating {
    /// The panel's top-left, in the layer's coordinates.
    pub fn pos(&self) -> Vec2d {
        self.floating.pos()
    }

    pub fn is_dragging(&self) -> bool {
        self.floating.is_dragging()
    }

    /// Put the panel somewhere without a gesture.
    pub fn set_pos(&mut self, cx: &mut Cx, pos: Vec2d) {
        self.floating.set_pos(pos);
        self.redraw(cx);
    }

    /// Take the move and the release of a drag, hit-tested against `layer`.
    ///
    /// The half of the gesture the panel cannot hear about itself: a pointer that outruns a
    /// frame lands outside the box, so the move has to be heard on something larger. `layer` is
    /// the app's to name — the window root, a page, a scroll view — which is what makes the
    /// requirement explicit rather than a comment about a widget that draws an invisible
    /// full-size box and hopes.
    pub fn drag(&mut self, cx: &mut Cx, event: &Event, layer: Area) -> bool {
        if !self.floating.is_held() {
            return false;
        }
        match event.hits(cx, layer) {
            Hit::FingerMove(fe) => {
                if self.floating.move_to(fe.abs) {
                    self.redraw(cx);
                    return true;
                }
                false
            }
            Hit::FingerUp(_) => {
                self.floating.release();
                self.redraw(cx);
                true
            }
            _ => false,
        }
    }
}

impl MpFloatingRef {
    pub fn pos(&self) -> Option<Vec2d> {
        self.borrow().map(|inner| inner.pos())
    }

    pub fn is_dragging(&self) -> bool {
        self.borrow().is_some_and(|inner| inner.is_dragging())
    }

    pub fn set_pos(&self, cx: &mut Cx, pos: Vec2d) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_pos(cx, pos);
        }
    }

    /// Forward a move or a release, hit-tested against the layer the app names.
    pub fn drag(&self, cx: &mut Cx, event: &Event, layer: Area) -> bool {
        if let Some(mut inner) = self.borrow_mut() {
            return inner.drag(cx, event, layer);
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f64, y: f64) -> Vec2d {
        dvec2(x, y)
    }

    fn panel() -> Floating {
        Floating::new(at(100.0, 100.0))
    }

    #[test]
    fn test_a_press_alone_moves_nothing() {
        let mut panel = panel();
        panel.press(at(120.0, 110.0));
        assert!(panel.is_held());
        assert!(!panel.is_dragging());
        assert_eq!(panel.pos(), at(100.0, 100.0));
    }

    #[test]
    fn test_a_move_with_no_press_does_nothing() {
        // A pointer passing over a panel must not drag it. This is the difference between a
        // panel and a magnet.
        let mut panel = panel();
        assert!(!panel.move_to(at(500.0, 500.0)));
        assert_eq!(panel.pos(), at(100.0, 100.0));
        assert!(!panel.is_dragging());
    }

    #[test]
    fn test_a_press_that_barely_moves_is_a_click_not_a_drag() {
        // A hand tremor on a title bar must not shift the panel.
        let mut panel = panel();
        panel.press(at(120.0, 110.0));
        assert!(!panel.move_to(at(121.0, 110.5)));
        assert!(!panel.move_to(at(122.0, 111.0)));
        assert!(!panel.is_dragging());
        assert_eq!(panel.pos(), at(100.0, 100.0));
    }

    #[test]
    fn test_the_threshold_is_measured_from_the_press_so_slow_drift_becomes_a_drag() {
        // Every individual move is under the threshold and the total is not. Measuring
        // per-move would let a pointer crawl across the panel forever without ever starting
        // the gesture — which is exactly how a reader drags something slowly.
        let mut panel = panel();
        panel.press(at(100.0, 100.0));
        let mut moved = false;
        for step in 1..=10 {
            moved |= panel.move_to(at(100.0 + step as f64, 100.0));
        }
        assert!(moved, "slow drift never started the drag");
        assert!(panel.is_dragging());
    }

    #[test]
    fn test_the_panel_keeps_the_grabbed_point_under_the_pointer() {
        // **The rule that makes a panel draggable.** The press landed 20 across and 10 down
        // inside the panel, so the panel's top-left is always the pointer minus that offset —
        // never the pointer itself, which would snap the panel so the pointer held its corner.
        let mut panel = panel();
        panel.press(at(120.0, 110.0));
        assert!(panel.move_to(at(320.0, 210.0)));
        assert_eq!(panel.pos(), at(300.0, 200.0), "the panel jumped on grab");
        // And it holds for every subsequent move.
        panel.move_to(at(1000.0, 40.0));
        assert_eq!(panel.pos(), at(980.0, 30.0));
        // Stated as the invariant rather than as values.
        for (x, y) in [(500.0, 500.0), (12.0, 900.0), (-40.0, -40.0)] {
            panel.move_to(at(x, y));
            assert_eq!(panel.pos() + at(20.0, 10.0), at(x, y));
        }
    }

    #[test]
    fn test_the_panel_is_not_clamped_to_anything() {
        // Deliberate, and bezel's decision as well: a panel dragged half off the window stays
        // there. The grab offset is preserved, so it can always be dragged back — clamping
        // would fight the reader for no benefit.
        let mut panel = panel();
        panel.press(at(100.0, 100.0));
        panel.move_to(at(-500.0, -500.0));
        assert_eq!(panel.pos(), at(-500.0, -500.0));
        panel.move_to(at(-400.0, -400.0));
        assert_eq!(panel.pos(), at(-400.0, -400.0), "it can be dragged back");
    }

    #[test]
    fn test_release_ends_both_the_hold_and_the_drag() {
        let mut panel = panel();
        panel.press(at(100.0, 100.0));
        panel.move_to(at(200.0, 200.0));
        assert!(panel.is_held() && panel.is_dragging());
        panel.release();
        assert!(!panel.is_held());
        assert!(!panel.is_dragging());
        assert_eq!(panel.pos(), at(200.0, 200.0), "the panel keeps where it is");
        // And a move after release does nothing.
        assert!(!panel.move_to(at(900.0, 900.0)));
        assert_eq!(panel.pos(), at(200.0, 200.0));
    }

    #[test]
    fn test_a_new_press_after_a_release_grabs_afresh() {
        // The offset must be recomputed, not carried over: the second grab is at a different
        // place inside the panel, and a stale offset would make the panel jump by the
        // difference.
        let mut panel = panel();
        panel.press(at(140.0, 100.0));
        panel.move_to(at(240.0, 100.0));
        assert_eq!(panel.pos(), at(200.0, 100.0));
        panel.release();
        // Now grab it near its top-left instead.
        panel.press(at(205.0, 102.0));
        panel.move_to(at(305.0, 202.0));
        assert_eq!(
            panel.pos(),
            at(300.0, 200.0),
            "the second grab used the first grab's offset"
        );
    }

    #[test]
    fn test_a_move_that_does_not_change_the_position_reports_no_move() {
        // A caller repainting only on `true` must not repaint a panel that did not move, which
        // is what a pointer jittering by a fraction of a pixel produces.
        let mut panel = panel();
        panel.press(at(100.0, 100.0));
        panel.move_to(at(200.0, 200.0));
        assert!(!panel.move_to(at(200.0, 200.0)));
        assert!(panel.move_to(at(201.0, 200.0)));
    }

    #[test]
    fn test_the_threshold_is_exactly_where_it_says_it_is() {
        // The boundary, because a threshold that behaves differently from its own doc is worse
        // than no threshold: a reader would tune against the doc.
        let mut short = panel();
        short.press(at(0.0, 0.0));
        assert!(
            !short.move_to(at(DRAG_THRESHOLD - 0.5, 0.0)),
            "a move under the threshold started a drag"
        );
        let mut exact = panel();
        exact.press(at(0.0, 0.0));
        assert!(
            exact.move_to(at(DRAG_THRESHOLD, 0.0)),
            "a move at the threshold did not start a drag"
        );
    }

    #[test]
    fn test_set_pos_places_the_panel_without_a_gesture() {
        let mut panel = panel();
        panel.set_pos(at(42.0, 42.0));
        assert_eq!(panel.pos(), at(42.0, 42.0));
        assert!(!panel.is_held() && !panel.is_dragging());
    }
}
