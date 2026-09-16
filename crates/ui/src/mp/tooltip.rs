//! `MpTooltip` — a plate that draws above everything, and the overlay mechanism
//! that is supposed to make that true.
//!
//! # ⚠️ Implemented, compiles, and does not draw
//!
//! Stated first because it is the state: the widget registers, the app runs with
//! zero `[E]` lines, `show_for` is reachable and sets `opened` — and **the plate
//! never appears**. Neither a hover over a trigger nor a startup call anchored to
//! a laid-out trigger produced a visible tooltip.
//!
//! What is ruled out: `handle_startup` was the first suspect, and the call was
//! moved to the first `handle_event` (after layout, so `area().rect()` is real),
//! which did not change the outcome. So the fault is in `draw_walk` / the draw
//! list rather than in the timing of `show`.
//!
//! Where to look next, in order of suspicion:
//!
//! 1. `begin_root_turtle(size, self.view.layout)` inside an overlay draw list.
//!    Makepad's own `Tooltip` calls it with `self.view.layout`, and this does too,
//!    but its `draw_bg.begin` uses `self.view.walk` — a `Walk` with `abs_pos`
//!    unset. If the root turtle is at the pass origin rather than at `pos`, the
//!    plate is drawn off-screen or at zero size and nothing is visible.
//! 2. Whether `#[deref] view` plus a manually-owned `DrawList2d` needs a
//!    `visit_cancel`/`draw_walk_all` arrangement this widget does not have. The
//!    deref field means the *view's* draw list may be the one being composited,
//!    leaving this one unused.
//! 3. `end_pass_sized_turtle` versus `end_root_turtle` — the pair must match the
//!    `begin_root_turtle`, and a mismatch that does not error could still yield an
//!    empty pass.
//!
//! The rest of this file is left in place because the *mechanism* is right — a
//! `DrawList2d` bracketed with `begin_overlay_reuse` is how Makepad orders
//! overlays, and the v2 alternative (geometry hit-testing) is what produced the
//! two hardest bugs in `docs/WIDGETS_PROGRESS_CN.md`. What is wrong is the
//! plumbing, not the approach, and the next session should diagnose rather than
//! rewrite.
//!
//! ## Why the approach is right even though it does not run
//!
//! A tooltip has to paint *over* whatever follows it in the tree. The v2 set
//! solved that by making the tooltip's owner draw the popup in an overlay pass
//! and by hit-testing geometry to decide what was on top — a workaround that
//! produced the two hardest bugs recorded in `docs/WIDGETS_PROGRESS_CN.md`: a
//! select whose dropdown was painted and then covered by a following section,
//! and a sheet whose close button never received its hit.
//!
//! Makepad has the mechanism: a widget owns a [`DrawList2d`] and brackets its
//! popup drawing in `begin_overlay_reuse`. The draw list is composited after the
//! normal tree, so a tooltip is above a dialog that is above a card — by the
//! framework's own ordering rather than by where the author happened to put it.
//! Getting this right once means the whole overlay family (popover, menu,
//! select, combobox) has a pattern to follow.
//!
//! ## The plate
//!
//! A tooltip is the one surface in the library that is *inverted*: a `solid`
//! plate carrying `on_solid` ink. That is what the palette's inverted pair is
//! for, and it is why a tooltip reads as an overlay rather than as another card
//! — it is the only thing on screen painted the other way round.

use makepad_widgets::*;

use makepad_theme::ControlSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    set_type_default() do #(DrawMpTooltip::script_shader(vm)){
        ..mod.draw.DrawQuad

        // Resolved from `Theme::of(cx)` every paint — this widget owns its
        // shader, so it takes the leaf path rather than the DSL-token one.
        fill: #x00000000
        border: #x00000000
        border_width: 1.0
        radius: 6.0

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let bw = self.border_width
            sdf.box(bw, bw, self.rect_size.x - bw * 2.0, self.rect_size.y - bw * 2.0, max(1.0, self.radius))
            // `fill_keep` because the stroke below is the *same* box — the one
            // place keeping the shape is what you want.
            sdf.fill_keep(self.fill)
            if (bw > 0.0) {
                sdf.stroke(self.border, bw)
            }
            return sdf.result
        }
    }

    mod.mp.MpTooltipBase = #(MpTooltip::register_widget(vm))

    mod.mp.MpTooltip = set_type_default() do mod.mp.MpTooltipBase{
        // The widget itself occupies nothing: it is a root-anchored overlay whose
        // plate appears at `pos`. A tooltip that took space in the layout would
        // push its own trigger around.
        width: 0
        height: 0

        content := View{
            width: Fit
            height: Fit
            padding: Inset{left: 8, right: 8, top: 5, bottom: 5}

            label := Label{
                width: Fit
                height: Fit
                draw_text +: {
                    text_style: mod.mpc.type.caption
                    color: #x00000000
                }
                text: "Tooltip"
            }
        }
    }
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpTooltip {
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

    #[live]
    draw_bg: DrawMpTooltip,

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
        // A tooltip hangs below and to the right of its anchor by default: enough
        // to clear the pointer's own hotspot, which is what a tooltip should not
        // sit under.
        self.offset = dvec2(12.0, 18.0);
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

        let theme = makepad_theme::Theme::of(cx.cx);
        let p = &theme.paint;
        // The inverted pair: the one plate in the library painted the other way
        // round, which is what makes a tooltip read as an overlay rather than as
        // another card.
        self.draw_bg.fill = p.solid;
        self.draw_bg.border = p.border_strong;
        self.draw_bg.border_width = 1.0;
        self.draw_bg.radius = ControlSize::Small.radius() as f32;

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
    pub fn show_for(&self, cx: &mut Cx, anchor: Area, text: &str) {
        let anchor = anchor.rect(cx).pos;
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
