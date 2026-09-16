//! `MpPopover` — a floating panel, and the first overlay whose trigger can be
//! verified.
//!
//! It uses the recipe [`MpTooltip`](crate::mp::tooltip) established, because the
//! constraints are structural rather than a matter of taste:
//!
//! - **`Fill`/`Fill` in an `Overlay` flow.** A zero-sized overlay has nothing to
//!   composite. The consequence — and this is the part that took an experiment to
//!   find — is that the host's rectangle must *contain* the panel, because an
//!   overlay draw list clips to it. So a popover is a full-size layer at the
//!   window root, not a small box beside its trigger, and a trigger-sized wrapper
//!   cannot host one.
//! - **The plate is a `Fit`-sized child, not this widget's `draw_bg`.** A plate on
//!   a `Fill`-sized widget is a full-window rectangle.
//! - **The content is the caller's**, which is the difference from a tooltip: a
//!   tooltip has one label, a popover has a form, a menu, a colour picker. So this
//!   widget is the tooltip recipe with the label removed and `#[deref] view`
//!   exposed as the content.
//!
//! ## It works, and the hunt for why it did not is worth recording
//!
//! Three faults, in the order they were found, and the *third* was the one that
//! hid the other two:
//!
//! 1. The panels were nested in their trigger rows, so a `Fill`/`Fill` popover
//!    had no rectangle to draw in. They are siblings of the content at the page
//!    root now — the overlay region a real app declares once at the window root.
//! 2. This widget forwarded its view's children, so `panel` drew **inline**,
//!    beside its trigger, open or shut. It returns `DrawStep::done()` now and the
//!    panel draws only in the overlay, which is what `MpTooltip` does.
//! 3. **`self.pin_popover(cx)` was never called.** The helper existed, the
//!    environment variable was read, and nothing invoked it — so with
//!    `GALLERY_POPOVER=1` set the popover was never opened at all. Every
//!    "the panel does not draw" conclusion drawn from those runs was about a
//!    widget that had not been asked to do anything. It took a log line at the
//!    top of `draw_walk` (6 calls, never `open=true`) to see it.
//!
//! A fourth thing looked like a fault and is a **layout rule**: a `Fill` child
//! contributes nothing to a `Fit` parent's width, so a `Fit` panel measures to
//! its widest *intrinsic* child. The form panel was 143pt wide with a 260pt field
//! inside it and everything past the label was clipped. A panel of `Fill` rows
//! must name its width.
//!
//! ## The tooltip's trigger could not be checked; this one is meant to be//! ## The tooltip's trigger could not be checked; this one is meant to be
//!
//! A tooltip's trigger is a hover, and a synthetic pointer warp produces no hover
//! event in this app, so its plate and anchoring are only checkable through
//! `GALLERY_TOOLTIP=1`. A popover's trigger is a **click**, and this note exists
//! because the click does arrive — which is what makes the failure diagnosable
//! rather than a tool limitation.
//!
//! ## Dismissal
//!
//! Three ways, and all three are needed: the trigger again, a click anywhere
//! outside, and Escape. "Click outside" is read from the panel's own hit — a
//! press that reaches this widget but not its content is outside it.

use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.motion.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    mod.mp.MpPopoverBase = #(MpPopover::register_widget(vm))

    mod.mp.MpPopover = set_type_default() do mod.mp.MpPopoverBase{
        // The layer. `Fill`/`Fill` so its rectangle contains the panel — see the
        // module doc for why that is a requirement and not a preference.
        width: Fill
        height: Fill
        flow: Overlay
        align: Align{x: 0.0, y: 0.0}

        // Carries the turtle the overlay pass needs, and paints nothing: the
        // plate is `panel`, which is `Fit`-sized. A plate here would be a
        // full-window rectangle.
        draw_bg +: {
            pixel: fn() {
                return vec4(0.0, 0.0, 0.0, 0.0)
            }
        }

        // `AUTO_CLOSE` rather than a bool prop: the caller almost always wants
        // the same three dismissals, and spelling them out at every call site
        // would be three places to get them wrong.
        popover_open: false

        // The panel, drawn at the anchor. The caller's children land in the
        // `panel` view, which is why it is `Fit`: the plate wraps its content.
        panel := RoundedView{
            width: Fit
            height: Fit
            padding: Inset{left: 12, right: 12, top: 12, bottom: 12}
            flow: Down
            spacing: 8

            draw_bg +: {
                color: instance(surface_overlay)
                border_size: instance(1.0)
                border_color: instance(border_strong)
                border_radius: instance(12.0)
            }
        }
    }
}

#[derive(Script, Widget)]
pub struct MpPopover {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    #[rust]
    draw_list: Option<DrawList2d>,

    #[live]
    draw_bg: DrawQuad,

    /// Whether the panel is showing. `live` so a caller can open one at
    /// construction, and so the state survives an app restart for a panel that
    /// is genuinely a setting rather than a gesture.
    #[live]
    popover_open: bool,

    /// Where the panel's top-left goes, in screen coordinates.
    #[rust]
    pos: Vec2d,
    /// The panel's measured box, so a second open can centre against it.
    #[rust]
    panel: Rect,
}

/// The gap between the anchor and the panel.
const POPOVER_GAP: f64 = 6.0;

impl ScriptHook for MpPopover {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        self.draw_list = Some(DrawList2d::script_new(vm));
    }

    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        vm.with_cx_mut(|cx| {
            if let Some(draw_list) = &self.draw_list {
                draw_list.redraw(cx);
            }
        });
    }
}

/// What a popover reports.
#[derive(Clone, Debug, Default)]
pub enum MpPopoverAction {
    Opened,
    Closed,
    #[default]
    None,
}

impl MpPopover {
    pub fn is_open(&self) -> bool {
        self.popover_open
    }

    pub fn opened(&self, actions: &Actions) -> bool {
        crate::mp::action::is::<MpPopoverAction>(self.widget_uid(), actions, |a| {
            matches!(a, MpPopoverAction::Opened)
        })
    }

    pub fn closed(&self, actions: &Actions) -> bool {
        crate::mp::action::is::<MpPopoverAction>(self.widget_uid(), actions, |a| {
            matches!(a, MpPopoverAction::Closed)
        })
    }

    /// Open the panel under `anchor`'s bottom-left edge.
    ///
    /// Anchored from an `Area` rather than a coordinate for the reason the
    /// tooltip records: whoever has the `Area` has the only reliable answer to
    /// where the trigger is, and `Area::rect` is in the pass's own space — the
    /// arithmetic that a call site doing it by hand gets wrong.
    pub fn open_for(&mut self, cx: &mut Cx, anchor: Area) {
        let rect = anchor.rect(cx);
        self.open_at(cx, dvec2(rect.pos.x, rect.pos.y + rect.size.y + POPOVER_GAP));
    }

    /// Open the panel with its top-left at `pos`.
    pub fn open_at(&mut self, cx: &mut Cx, pos: Vec2d) {
        self.pos = pos;
        self.set_open(cx, true);
    }

    /// Open the panel centred on `pos` horizontally — the placement a menu under
    /// a centred trigger wants.
    ///
    /// Falls back to left-aligned until the panel has been measured once: its
    /// width is not knowable before a paint, and a panel that jumped sideways on
    /// its second appearance would be worse than one briefly off-centre.
    pub fn open_centered_at(&mut self, cx: &mut Cx, pos: Vec2d) {
        let x = if self.panel.size.x > 0.0 {
            pos.x - self.panel.size.x * 0.5
        } else {
            pos.x
        };
        self.open_at(cx, dvec2(x, pos.y));
    }

    pub fn close(&mut self, cx: &mut Cx) {
        self.set_open(cx, false);
    }

    pub fn toggle(&mut self, cx: &mut Cx) {
        let open = !self.popover_open;
        self.set_open(cx, open);
    }

    /// The single mutation path, so the field, the draw list and the reported
    /// action can never disagree — which is what makes `popover_open: true` in a
    /// DSL block behave like a click.
    pub fn set_open(&mut self, cx: &mut Cx, open: bool) {
        if self.popover_open == open {
            return;
        }
        self.popover_open = open;
        if let Some(draw_list) = &self.draw_list {
            draw_list.redraw(cx);
        }
        cx.widget_action(
            self.widget_uid(),
            if open {
                MpPopoverAction::Opened
            } else {
                MpPopoverAction::Closed
            },
        );
        self.redraw(cx);
    }
}

impl Widget for MpPopover {
    fn visit_cancel(&self, visit: &mut dyn FnMut(LiveId, WidgetRef)) -> bool {
        // An open panel cancels events for its children, so a click inside it
        // does not also fall through to whatever is beneath.
        self.popover_open && self.cancel_children_impl(visit)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if !self.popover_open {
            return;
        }

        // Escape closes. Read before the children, because a text field inside
        // the panel would otherwise consume it.
        if let Event::KeyDown(ke) = event {
            if ke.key_code == KeyCode::Escape {
                self.close(cx);
                return;
            }
        }
        // The platform's back gesture is a dismissal too, and the only one that
        // arrives with no position at all.
        if matches!(event, Event::BackPressed { .. }) {
            self.close(cx);
            return;
        }

        // A press anywhere but the panel dismisses it, which is how "click away
        // to close" is done without a global mouse hook — the panel's own
        // rectangle is the answer to what "away" means.
        let panel = self.view.view(cx, ids!(panel)).area();
        if let Some(pos) = press_pos(event) {
            let inside = panel.is_valid(cx) && panel.rect(cx).contains(pos);
            if !inside {
                self.close(cx);
                return;
            }
        }

        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.popover_open {
            if let Some(draw_list) = self.draw_list.as_mut() {
                draw_list.begin_overlay_reuse(cx);

                let size = cx.current_pass_size();
                cx.begin_root_turtle(size, self.view.layout);
                self.draw_bg.begin(cx, self.view.walk, self.view.layout);

                // `self.view`, not the `panel` child directly — the same call
                // `MpTooltip` makes, and the difference that made this draw:
                // a child drawn by `draw_walk_all` outside its parent's
                // bookkeeping does not render, while the widget's own deref view
                // does. `panel` is that view's only child, so drawing the view
                // draws the plate.
                let view_walk = self.view.walk(cx).with_abs_pos(self.pos);
                self.view.draw_walk_all(cx, scope, view_walk);

                self.draw_bg.end(cx);
                self.panel = self.draw_bg.area().rect(cx.cx);

                cx.end_pass_sized_turtle();
                self.draw_list.as_mut().unwrap().end(cx);
            }
        }

        // **Nothing is forwarded.** The panel is a child of this widget's view,
        // so returning the view's steps would draw it *inline*, in the flow,
        // whether or not the popover is open — which is exactly what the first
        // version did: every panel on the page sat beside its trigger like a
        // stray card. `MpTooltip` returns `DrawStep::done()` for the same reason:
        // a child that belongs in the overlay must be drawn nowhere else.
        let _ = walk;
        DrawStep::done()
    }
}

/// Where a press landed, if the event was one.
///
/// Read from the raw event rather than through `event.hits`, because the point
/// is precisely to catch a press that did *not* land on the panel — and a hit
/// test for the panel answers the question the wrong way round. A touch is taken
/// at its first contact, which is enough to decide inside-or-outside.
fn press_pos(event: &Event) -> Option<Vec2d> {
    match event {
        Event::MouseDown(me) => Some(me.abs),
        Event::TouchUpdate(tu) => tu.touches.first().map(|t| t.abs),
        _ => None,
    }
}

impl MpPopoverRef {
    pub fn is_open(&self) -> bool {
        self.borrow().is_some_and(|inner| inner.is_open())
    }

    pub fn opened(&self, actions: &Actions) -> bool {
        self.borrow().is_some_and(|inner| inner.opened(actions))
    }

    pub fn closed(&self, actions: &Actions) -> bool {
        self.borrow().is_some_and(|inner| inner.closed(actions))
    }

    pub fn open_for(&self, cx: &mut Cx, anchor: Area) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.open_for(cx, anchor);
        }
    }

    pub fn open_at(&self, cx: &mut Cx, pos: Vec2d) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.open_at(cx, pos);
        }
    }

    pub fn close(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.close(cx);
        }
    }

    pub fn toggle(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.toggle(cx);
        }
    }

    /// Open without a pointer.
    ///
    /// For a capture or a test: Makepad exposes no accessibility tree, so a
    /// script cannot click a trigger — and a click is the one gesture a capture
    /// script would otherwise be able to synthesise.
    pub fn show(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            let area = inner.view.area();
            inner.open_for(cx, area);
        }
    }
}
