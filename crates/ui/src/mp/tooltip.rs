//! `MpTooltip` — a plate that draws above everything, and the two things that
//! turned out to be load-bearing about it.
//!
//! ## The overlay
//!
//! A tooltip has to paint *over* whatever follows it in the tree. The v2 set
//! solved that by managing an overlay pass by hand and resolving z-order with
//! geometry hit-tests — an approach that produced the two hardest bugs in
//! `docs/WIDGETS_PROGRESS_CN.md`: a select whose dropdown was painted and then
//! covered by a following section, and a sheet whose close button never received
//! its hit.
//!
//! Makepad's mechanism is a [`DrawList2d`] bracketed with `begin_overlay_reuse`:
//! the draw list is composited after the normal tree, so a tooltip is above a
//! dialog that is above a card by the framework's own ordering rather than by
//! where the author put it. Getting this right once gives the whole overlay
//! family (popover, menu, select, combobox) a pattern to follow.
//!
//! ## Two things it took to make it draw
//!
//! Both were invisible until a screenshot was taken, and both are recorded here
//! because the second one is a *design* constraint, not a bug:
//!
//! 1. **A zero-sized overlay has nothing to composite.** The first version
//!    declared `width: 0, height: 0` on the theory that a tooltip should not
//!    take space — and the plate never appeared at all. It is `Fill`/`Fill` in
//!    an `Overlay` flow, which is also how Makepad's own `Tooltip` is declared.
//!    The consequence is the usage rule below: it must be a child of an
//!    `Overlay`-flow parent, beside the content it covers rather than inside the
//!    content's column.
//! 2. **The plate must be `Fit`-sized, so it cannot live on this widget.** The
//!    second version put the plate on the widget's own `draw_bg` — which is
//!    `Fill`/`Fill`, so it painted a full-window rectangle over the entire
//!    application. `draw_bg` is here only to carry the turtle the overlay pass
//!    needs, and it paints **nothing**; the plate is the `content` child, which
//!    is `Fit` and shrink-wraps the label. Makepad's own `Tooltip` does exactly
//!    this, with a `pixel: fn()` that returns transparent.
//!
//! ```text
//! SomeParent{ flow: Overlay }      // the widget is a sibling of the content
//!   content_column := View{ ... }  // what the tooltip covers
//!   tip := mod.mp.MpTooltip{}      // Fill/Fill, draws after, plate is content
//! ```
//!
//! ## One tooltip per overlay region, and why a wrapper cannot exist
//!
//! The gallery's Overlay page detects its hover in the *app* and does not work.
//! The obvious fix is a wrapper — an `MpTooltipArea` that holds its trigger and
//! its own tooltip — and **that was built, tested, and cannot work.** Recording
//! it here because the reason is structural and would otherwise be re-attempted:
//!
//! An overlay draw list clips to **its widget's rectangle**. A tooltip's plate is
//! positioned at an absolute `pos` that lies *outside* a trigger-sized box, so in
//! a `Fit`-sized wrapper the plate is clipped away. This widget works because it
//! is `Fill`/`Fill` inside a `Fill`/`Fill` `Overlay` parent: its rectangle is the
//! whole overlay region, and the plate is inside it.
//!
//! What was observed, precisely:
//!
//! - `Fill`/`Fill` inside the page's `Overlay` flow, opened by `show_for` —
//!   **draws**, over a following sibling, anchored to a trigger's `Area`.
//! - the same tooltip nested inside a `Fit` wrapper that owns its trigger —
//!   **does not draw**, whether opened by a real hover or by calling `show()`
//!   directly on it. The direct call is what isolated the fault to the draw path
//!   rather than to the hover, since a synthetic pointer cannot be trusted here
//!   (see below).
//!
//! So the constraint is a *usage* rule with teeth: **one tooltip per overlay
//! region, `Fill`/`Fill`, and triggers cause it to be shown rather than owning
//! one.** A wrapper is not an option, and neither is a tooltip per control.
//!
//! ## The hover, and what this tool can and cannot check
//!
//! Hovering belongs in the trigger — the widget that receives `Hit::FingerHoverIn`
//! in its own `handle_event`, which is exactly what
//! [`control::Signals::hover_in`] already computes — but a trigger cannot *own* a
//! tooltip, so what it must do is signal one. That is a hover action from the
//! control, handled where the one shared tooltip lives.
//!
//! That work is not done. Note also that a synthetic pointer warp does **not**
//! produce a hover event in this app, so the hover path cannot be verified from a
//! capture script at all: the evidence is that a ghost button under the warped
//! pointer shows none of its hover wash. `GALLERY_TOOLTIP=1` exists for that
//! reason — it exercises the plate and the anchoring without a pointer.
//!
//! [`control::Signals::hover_in`]: crate::mp::control::Signals
//!
//! ## The plate//! ## The plate
//!
//! A tooltip is the one surface in the library that is *inverted*: a `solid`
//! plate carrying `on_solid` ink. That is what the palette's inverted pair is
//! for, and it is why a tooltip reads as an overlay rather than as another card —
//! it is the only thing on screen painted the other way round.

use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    mod.mp.MpTooltipBase = #(MpTooltip::register_widget(vm))

    mod.mp.MpTooltip = set_type_default() do mod.mp.MpTooltipBase{
        // Fill/Fill in an `Overlay` flow: a zero-sized overlay has nothing to
        // composite, which is why the first version drew nothing at all.
        width: Fill
        height: Fill
        flow: Overlay
        align: Align{x: 0.0, y: 0.0}

        // Carries the turtle the overlay pass needs, and paints **nothing**.
        // The plate cannot be drawn here: this widget is Fill-sized, so a plate
        // on it is a full-window rectangle — which is precisely what the second
        // version drew, a near-white sheet over the whole application.
        draw_bg +: {
            pixel: fn() {
                return vec4(0.0, 0.0, 0.0, 0.0)
            }
        }

        // The plate. `Fit`, so it shrink-wraps the label, and inverted: the
        // palette's `solid`/`on_solid` pair is what a tooltip is for.
        //
        // The colours come from the DSL here rather than from Rust, because only
        // a child can be `Fit`-sized. That puts the plate on the container path —
        // token references copied at apply time, refreshed by
        // `Theme::install`'s `request_script_reapply` — which is the same trade
        // `mp/surface.rs` makes for every surface in the library.
        content := RoundedView{
            width: Fit
            height: Fit
            padding: Inset{left: 8, right: 8, top: 5, bottom: 5}

            draw_bg +: {
                color: instance(solid)
                border_size: instance(1.0)
                border_color: instance(border_strong)
                border_radius: instance(6.0)
            }

            label := Label{
                width: Fit
                height: Fit
                draw_text +: {
                    text_style: caption
                    color: on_solid
                }
                text: "Tooltip"
            }
        }
    }
}

/// A plate that draws above the widget tree, anchoring at a screen position.
// No `ScriptHook` derive: it is implemented by hand to create the draw list.
// No `#[uid]`: the derive takes the uid from the `#[deref]` field, and a widget
// with a deref view cannot also declare one.
#[derive(Script, Widget)]
pub struct MpTooltip {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    /// The overlay's own draw list. `Some` always, set at construction.
    ///
    /// A `rust` field rather than a `live` one because a `DrawList2d` is created
    /// from the script VM (`DrawList2d::script_new`) rather than applied from the
    /// DSL.
    #[rust]
    draw_list: Option<DrawList2d>,

    /// The turtle holder the overlay pass needs. Paints nothing; see the module
    /// doc for why the plate cannot be drawn here.
    #[live]
    draw_bg: DrawQuad,

    #[rust]
    opened: bool,
    /// Where the plate's top-left corner goes, in screen coordinates.
    #[rust]
    pos: Vec2d,
    /// How far from the anchor the plate sits. A tooltip that touched its
    /// trigger would read as part of it.
    #[rust]
    offset: Vec2d,
}

impl ScriptHook for MpTooltip {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        self.draw_list = Some(DrawList2d::script_new(vm));
        // The gap between the trigger and the plate. Vertical only: the anchor
        // [`MpTooltipRef::show_for`] passes is the trigger's *bottom* edge, so a
        // horizontal offset would push the plate sideways over whatever sits
        // beside the trigger rather than under it.
        self.offset = dvec2(0.0, 6.0);
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

impl MpTooltip {
    /// The gap between the anchor and the plate.
    pub fn offset(&self) -> Vec2d {
        self.offset
    }

    pub fn set_offset(&mut self, cx: &mut Cx, offset: Vec2d) {
        self.offset = offset;
        self.redraw(cx);
    }

    pub fn is_opened(&self) -> bool {
        self.opened
    }

    pub fn set_text(&mut self, cx: &mut Cx, text: &str) {
        self.view.label(cx, ids!(label)).set_text(cx, text);
        self.redraw(cx);
    }

    /// Show the plate with its top-left at `anchor` plus this tooltip's offset.
    ///
    /// `anchor` is a *corner to hang from*, which is why
    /// [`MpTooltipRef::show_for`] hands over a trigger's bottom edge rather than
    /// its top-left: a caller that knows where a widget is has not decided where
    /// the plate goes, and one place has to.
    ///
    /// The offset is applied here rather than by the caller because a caller that
    /// has a widget's rectangle has an *anchor*, not a corner — it knows where
    /// the thing is, not where the tooltip should go. One place decides that, and
    /// a caller that disagrees calls [`MpTooltip::set_offset`].
    pub fn show_at(&mut self, cx: &mut Cx, anchor: Vec2d, text: &str) {
        self.set_text(cx, text);
        self.pos = anchor + self.offset;
        self.show(cx);
    }

    pub fn show(&mut self, cx: &mut Cx) {
        if self.opened {
            return;
        }
        self.opened = true;
        self.redraw_overlay(cx);
    }

    pub fn hide(&mut self, cx: &mut Cx) {
        if !self.opened {
            return;
        }
        self.opened = false;
        self.redraw_overlay(cx);
    }

    /// Redraw both the overlay's draw list and the widget's own area.
    ///
    /// Both, and not just one: the widget's `area` and the view's draw list are
    /// only set up once `draw_walk_all` has run with the tooltip open, so the
    /// first `show` would otherwise redraw nothing and the plate would be
    /// invisible until something else happened to cause a repaint.
    fn redraw_overlay(&mut self, cx: &mut Cx) {
        if let Some(draw_list) = &self.draw_list {
            draw_list.redraw(cx);
        }
        self.redraw(cx);
    }
}

impl Widget for MpTooltip {
    fn visit_cancel(&self, visit: &mut dyn FnMut(LiveId, WidgetRef)) -> bool {
        // An open tooltip cancels events for its children, so a click on the
        // plate does not fall through to whatever is underneath it.
        self.opened && self.cancel_children_impl(visit)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if !self.opened {
            return;
        }
        self.view.widget(cx, ids!(content)).handle_event(cx, event, scope);
        // Any real interaction dismisses it. Raw events rather than a hit test,
        // deliberately: a tooltip must not change how hits resolve for anything
        // else, and the only thing it needs to know is *that* something happened.
        match event {
            Event::BackPressed { .. }
            | Event::MouseDown(_)
            | Event::MouseUp(_)
            | Event::Scroll(_) => self.hide(cx),
            Event::TouchUpdate(tu) => {
                if tu
                    .touches
                    .iter()
                    .any(|t| matches!(t.state, event::TouchState::Start))
                {
                    self.hide(cx);
                }
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, _walk: Walk) -> DrawStep {
        let Some(draw_list) = self.draw_list.as_mut() else {
            return DrawStep::done();
        };
        draw_list.begin_overlay_reuse(cx);

        let size = cx.current_pass_size();
        cx.begin_root_turtle(size, self.view.layout);
        self.draw_bg.begin(cx, self.view.walk, self.view.layout);

        if self.opened {
            let walk = self.view.walk(cx).with_abs_pos(self.pos);
            self.view.draw_walk_all(cx, scope, walk);
        }

        self.draw_bg.end(cx);
        cx.end_pass_sized_turtle();
        self.draw_list.as_mut().unwrap().end(cx);
        DrawStep::done()
    }
}

impl MpTooltipRef {
    pub fn is_opened(&self) -> bool {
        self.borrow().is_some_and(|inner| inner.is_opened())
    }

    pub fn set_text(&self, cx: &mut Cx, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(cx, text);
        }
    }

    pub fn set_offset(&self, cx: &mut Cx, offset: Vec2d) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_offset(cx, offset);
        }
    }

    /// Show the plate anchored at a widget's own rectangle.
    ///
    /// The shape a call site actually wants: it has the trigger's `Area`, and
    /// where on that trigger to hang the plate is this widget's business.
    ///
    /// **An `Area` is empty until its widget has been laid out**, so a caller
    /// anchoring during startup has to wait for a non-empty `area.rect(cx)`
    /// before calling this — otherwise the anchor is `(0, 0)` and the plate
    /// appears in the window's corner, which looks like the overlay drawing in
    /// the wrong place rather than like a caller racing layout. A hover call site
    /// is never affected, because a hover is by construction after a layout.
    pub fn show_for(&self, cx: &mut Cx, anchor: Area, text: &str) {
        let rect = anchor.rect(cx);
        // The trigger's bottom-left: the plate hangs under the trigger rather
        // than over it, and under the trigger's leading edge rather than
        // straddling its centre — the latter needs the plate's own width, which
        // is not known until it has been laid out once.
        let anchor = dvec2(rect.pos.x, rect.pos.y + rect.size.y);
        if let Some(mut inner) = self.borrow_mut() {
            inner.show_at(cx, anchor, text);
        }
    }

    pub fn show_at(&self, cx: &mut Cx, anchor: Vec2d, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.show_at(cx, anchor, text);
        }
    }

    pub fn show(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.show(cx);
        }
    }

    pub fn hide(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.hide(cx);
        }
    }
}
