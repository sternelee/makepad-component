use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp_theme.*

    // ============================================================
    // MpPagination - shadcn-style page navigation
    // ============================================================

    // Pagination container
    mod.widgets.MpPagination = mod.widgets.View{
        width: Fit
        height: Fit
        flow: Right
        spacing: 2.0
        align: Align{y: 0.5}
    }

    // Pagination item (page number button)
    mod.widgets.MpPaginationItemBase = #(MpPaginationItem::register_widget(vm))
    mod.widgets.MpPaginationItem = set_type_default() do mod.widgets.MpPaginationItemBase{
        width: 36
        height: 36
        padding: Inset{left: 0.0, right: 0.0, top: 0.0, bottom: 0.0}
        align: Align{x: 0.5, y: 0.5}
        

        
        draw_bg +: {
            radius: instance(6.0)
            bg_color: #x0000
            bg_hover: #xf1f5f9
            bg_active: (PRIMARY)
            hover: instance(0.0)
            active: instance(0.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let s = min(self.rect_size.x, self.rect_size.y)
                let bg = mix(self.bg_color, self.bg_hover, self.hover)
                let final_bg = mix(bg, self.bg_active, self.active)

                sdf.circle(s * 0.5, s * 0.5, s * 0.5)
                sdf.fill(final_bg)

                return sdf.result
            }
        }

        draw_text +: {
            text_style: theme.font_regular{font_size: 13.0}
            color: (FOREGROUND)
            color_active: (PRIMARY_FOREGROUND)
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
        width: 36
        height: 36
        padding: Inset{left: 0.0, right: 0.0, top: 0.0, bottom: 0.0}
        align: Align{x: 0.5, y: 0.5}
        

        
        draw_bg +: {
            radius: instance(6.0)
            bg_color: #x0000
            bg_hover: #xf1f5f9
            hover: instance(0.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let s = min(self.rect_size.x, self.rect_size.y)
                let c = vec2(s * 0.5, s * 0.5)
                let bg = mix(self.bg_color, self.bg_hover, self.hover)

                sdf.circle(c.x, c.y, s * 0.5)
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

                sdf.circle(c.x, c.y, s * 0.5)
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
        width: 36
        height: 36
        align: Align{x: 0.5, y: 0.5}
        

        draw_text +: {
            text_style: theme.font_regular{font_size: 13.0}
            color: MUTED_FOREGROUND
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
            Hit::FingerUp(fe) => {
                if fe.is_over {
                    cx.widget_action(uid, MpPaginationAction::PageSelected(self.page_num));
                }
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
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
            Hit::FingerUp(fe) => {
                if fe.is_over {
                    cx.widget_action(uid, MpPaginationAction::Prev);
                }
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        DrawStep::done()
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
