use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpPagination - shadcn-style page navigation
    // ============================================================

    // Pagination container (custom widget so it can propagate its size
    // to the item children declared by the app)
    mod.widgets.MpPaginationBase = #(MpPagination::register_widget(vm))
    mod.widgets.MpPagination = set_type_default() do mod.widgets.MpPaginationBase{
        width: Fit
        height: Fit
        flow: Right
        spacing: 4.0
        align: Align{y: 0.5}
    }

    // Pagination item (page number button)
    mod.widgets.MpPaginationItemBase = #(MpPaginationItem::register_widget(vm))
    mod.widgets.MpPaginationItem = set_type_default() do mod.widgets.MpPaginationItemBase{
        width: 28
        height: 28
        padding: Inset{left: 0.0, right: 0.0, top: 0.0, bottom: 0.0}
        align: Align{x: 0.5, y: 0.5}



        draw_bg +: {
            radius: instance(7.0)
            bg_color: #x0000
            bg_hover: ELEMENT_HOVER
            bg_active: (ACCENT)
            hover: instance(0.0)
            active: instance(0.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let s = min(self.rect_size.x, self.rect_size.y)
                let bg = mix(self.bg_color, self.bg_hover, self.hover)
                let final_bg = mix(bg, self.bg_active, self.active)

                sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                sdf.fill(final_bg)

                return sdf.result
            }
        }

        draw_text +: {
            text_style: theme.font_regular{font_size: 12.5}
            color: (TEXT)
            color_active: (ON_ACCENT)
            active: instance(0.0)
            get_color: fn() {
                return mix(self.color, self.color_active, self.active)
            }
        }

        text: ""
        active: false

        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_bg: {hover: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.1}}
                    apply: {draw_bg: {hover: 1.0}}
                }
            }
            active: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.2}}
                    redraw: true
                    apply: {draw_bg: {active: 0.0} draw_text: {active: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.2}}
                    redraw: true
                    apply: {draw_bg: {active: 1.0} draw_text: {active: 1.0}}
                }
            }
        }
    }

    // Pagination prev button
    mod.widgets.MpPaginationPrevBase = #(MpPaginationPrev::register_widget(vm))
    mod.widgets.MpPaginationPrev = set_type_default() do mod.widgets.MpPaginationPrevBase{
        width: 28
        height: 28
        padding: Inset{left: 0.0, right: 0.0, top: 0.0, bottom: 0.0}
        align: Align{x: 0.5, y: 0.5}



        draw_bg +: {
            radius: instance(7.0)
            bg_color: #x0000
            bg_hover: ELEMENT_HOVER
            hover: instance(0.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let s = min(self.rect_size.x, self.rect_size.y)
                let c = vec2(s * 0.5, s * 0.5)
                let bg = mix(self.bg_color, self.bg_hover, self.hover)

                sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                sdf.fill(bg)

                // Left arrow
                sdf.move_to(c.x + 4.0, c.y - 5.0)
                sdf.line_to(c.x - 4.0, c.y)
                sdf.line_to(c.x + 4.0, c.y + 5.0)
                sdf.stroke(#x64748b, 1.5)

                return sdf.result
            }
        }

        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_bg: {hover: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.1}}
                    apply: {draw_bg: {hover: 1.0}}
                }
            }
        }
    }

    // Pagination next button
    mod.widgets.MpPaginationNext = mod.widgets.MpPaginationPrev{
        draw_bg +: {
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let s = min(self.rect_size.x, self.rect_size.y)
                let c = vec2(s * 0.5, s * 0.5)
                let bg = mix(self.bg_color, self.bg_hover, self.hover)

                sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                sdf.fill(bg)

                // Right arrow
                sdf.move_to(c.x - 4.0, c.y - 5.0)
                sdf.line_to(c.x + 4.0, c.y)
                sdf.line_to(c.x - 4.0, c.y + 5.0)
                sdf.stroke(#x64748b, 1.5)

                return sdf.result
            }
        }
    }

    // Pagination ellipsis
    mod.widgets.MpPaginationEllipsis = mod.widgets.Label{
        width: 28
        height: 28
        align: Align{x: 0.5, y: 0.5}


        draw_text +: {
            text_style: theme.font_regular{font_size: 12.5}
            color: TEXT_MUTED
        }
        text: "..."
    }
}

// ============================================================
// Rust Implementation
// ============================================================

#[derive(Clone, Debug, Default)]
pub enum MpPaginationAction {
    PageSelected(usize),
    Prev,
    Next,
    #[default]
    None,
}

// MpPaginationItem - individual page number
#[derive(Script, ScriptHook, Widget, Animator)]
pub struct MpPaginationItem {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[apply_default]
    animator: Animator,

    #[redraw]
    #[live]
    draw_bg: DrawQuad,
    #[live]
    draw_text: DrawText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[live]
    text: ArcStringMut,
    #[live]
    active: bool,

    /// Five-step size driving the option row height, padding and font.
    #[live]
    size: MpSize,

    #[rust]
    area: Area,

    #[rust]
    page_num: usize,
}

impl Widget for MpPaginationItem {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let uid = self.widget_uid();

        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }

        if self.active {
            return;
        }

        match event.hits(cx, self.area) {
            Hit::FingerHoverIn(_) => {
                cx.set_cursor(MouseCursor::Hand);
                self.animator_play(cx, ids!(hover.on));
            }
            Hit::FingerHoverOut(_) => {
                self.animator_play(cx, ids!(hover.off));
            }
            Hit::FingerUp(fe)
                if fe.is_over => {
                    cx.widget_action(uid, MpPaginationAction::PageSelected(self.page_num));
                }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // Metrics from the size system (Medium = the DSL 28x28)
        let mut walk = walk;
        walk.width = Size::Fixed(pagination_box(self.size));
        walk.height = Size::Fixed(pagination_box(self.size));
        self.draw_text.text_style.font_size = self.size.font_size();

        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_text
            .draw_walk(cx, Walk::fit(), Align::default(), self.text.as_ref());
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        DrawStep::done()
    }
}

impl MpPaginationItem {
    pub fn set_page(&mut self, page: usize) {
        self.page_num = page;
    }

    pub fn set_active(&mut self, cx: &mut Cx, active: bool) {
        if self.active != active {
            self.active = active;
            self.animator_toggle(cx, active, Animate::Yes, ids!(active.on), ids!(active.off));
            self.redraw(cx);
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text.as_mut_empty().push_str(text);
    }

    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            self.redraw(cx);
        }
    }
}

impl MpPaginationItemRef {
    pub fn page_selected(&self, actions: &Actions) -> Option<usize> {
        if let Some(item) = actions.find_widget_action(self.widget_uid()) {
            if let MpPaginationAction::PageSelected(page) = item.cast() {
                return Some(page);
            }
        }
        None
    }
}

// MpPaginationPrev - prev/next button
#[derive(Script, ScriptHook, Widget, Animator)]
pub struct MpPaginationPrev {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[apply_default]
    animator: Animator,

    #[redraw]
    #[live]
    draw_bg: DrawQuad,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    /// Five-step size driving the button box size.
    #[live]
    size: MpSize,

    #[rust]
    area: Area,
}

impl Widget for MpPaginationPrev {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let uid = self.widget_uid();

        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }

        match event.hits(cx, self.area) {
            Hit::FingerHoverIn(_) => {
                cx.set_cursor(MouseCursor::Hand);
                self.animator_play(cx, ids!(hover.on));
            }
            Hit::FingerHoverOut(_) => {
                self.animator_play(cx, ids!(hover.off));
            }
            Hit::FingerUp(fe)
                if fe.is_over => {
                    cx.widget_action(uid, MpPaginationAction::Prev);
                }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let mut walk = walk;
        walk.width = Size::Fixed(pagination_box(self.size));
        walk.height = Size::Fixed(pagination_box(self.size));
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        DrawStep::done()
    }
}

impl MpPaginationPrev {
    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            self.redraw(cx);
        }
    }
}

impl MpPaginationPrevRef {
    pub fn clicked_prev(&self, actions: &Actions) -> bool {
        if let Some(item) = actions.find_widget_action(self.widget_uid()) {
            matches!(item.cast::<MpPaginationAction>(), MpPaginationAction::Prev)
        } else {
            false
        }
    }
}

// ============================================================
// MpPagination container
// ============================================================

/// Page-number box edge for a size step (Medium = the original 28).
fn pagination_box(size: MpSize) -> f64 {
    match size {
        MpSize::XSmall => 22.0,
        MpSize::Small => 24.0,
        MpSize::Medium => 28.0,
        MpSize::Large => 34.0,
        MpSize::XLarge => 40.0,
    }
}

/// Pagination container: propagates its size to the item/prev/next children
/// the app declares inside it.
#[derive(Script, ScriptHook, Widget)]
pub struct MpPagination {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    /// Five-step size propagated to the item children.
    #[live]
    size: MpSize,

    /// Last size applied to the children (avoids re-applying every draw).
    #[rust]
    applied_size: Option<MpSize>,
}

impl Widget for MpPagination {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Propagate the size to the item children once per size change
        // (child setters request redraws, so guard them).
        if self.applied_size != Some(self.size) {
            self.applied_size = Some(self.size);
            self.view.children(&mut |_id, child| {
                if let Some(mut item) = child.borrow_mut::<MpPaginationItem>() {
                    item.set_size(cx, self.size);
                } else if let Some(mut prev) = child.borrow_mut::<MpPaginationPrev>() {
                    prev.set_size(cx, self.size);
                }
            });
        }
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpPagination {
    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            // Children re-sync on the next draw_walk.
            self.applied_size = None;
            self.redraw(cx);
        }
    }
}

impl MpPaginationRef {
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
