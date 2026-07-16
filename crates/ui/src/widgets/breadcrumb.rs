use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp_theme.*

    // ============================================================
    // MpBreadcrumb - Navigation breadcrumb trail
    // Inspired by shadcn/ui Breadcrumb
    // ============================================================

    mod.widgets.MpBreadcrumb = mod.widgets.View{
        width: Fit
        height: Fit
        flow: Right
        spacing: 4.0
        align: Align{y: 0.5}
    }

    // Breadcrumb item (clickable) — registered widget inherits full widget capabilities
    mod.widgets.MpBreadcrumbItemBase = #(MpBreadcrumbItem::register_widget(vm))
    mod.widgets.MpBreadcrumbItem = set_type_default() do mod.widgets.MpBreadcrumbItemBase{
        width: Fit
        height: Fit

        draw_text +: {
            text_style: theme.font_regular{font_size: 13.0}
            color: #x3B82F6
        }

        text: ""
        active: false

        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_text: {color: #x3B82F6}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_text: {color: #x2563EB}}
                }
            }
        }
    }

    // Breadcrumb separator — keep as View, no draw_text needed
    mod.widgets.MpBreadcrumbSeparator = mod.widgets.Label{
        width: Fit
        height: Fit
        margin: Inset{left: 4.0, right: 4.0}
        draw_text +: {
            text_style: theme.font_regular{font_size: 13.0}
            color: #x94A3B8
        }
        text: "/"
    }

    // Active breadcrumb (last item, not clickable)
    mod.widgets.MpBreadcrumbActive = set_type_default() do mod.widgets.MpBreadcrumbItemBase{
        draw_text +: {
            color: #x1D1D1F
        }
    }
}

// ============================================================
// Rust Implementation
// ============================================================

#[derive(Script, ScriptHook, Widget, Animator)]
pub struct MpBreadcrumbItem {
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
}

#[derive(Clone, Debug, Default)]
pub enum MpBreadcrumbAction {
    Clicked,
    #[default]
    None,
}

impl Widget for MpBreadcrumbItem {
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
                    cx.widget_action(uid, MpBreadcrumbAction::Clicked);
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

impl MpBreadcrumbItem {
    pub fn clicked(&self, actions: &Actions) -> bool {
        if let Some(action) = actions.find_widget_action(self.widget_uid()) {
            matches!(action.cast(), MpBreadcrumbAction::Clicked)
        } else {
            false
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text.as_mut_empty().push_str(text);
    }

    pub fn set_active(&mut self, cx: &mut Cx, active: bool) {
        self.active = active;
        self.redraw(cx);
    }
}

impl MpBreadcrumbItemRef {
    pub fn clicked(&self, actions: &Actions) -> bool {
        if let Some(inner) = self.borrow() {
            inner.clicked(actions)
        } else {
            false
        }
    }

    pub fn set_text(&self, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(text);
        }
    }

    pub fn set_active(&self, cx: &mut Cx, active: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_active(cx, active);
        }
    }
}
