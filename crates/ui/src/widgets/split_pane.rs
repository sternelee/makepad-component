use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpSplitPane - two panes with a draggable vertical divider
    // ============================================================

    mod.widgets.MpSplitPane = set_type_default() do #(MpSplitPane::register_widget(vm)){
        width: Fill
        height: Fill
        flow: Right
        spacing: 0

        left := View{
            width: 200.0
            height: Fill
            flow: Down
        }

        divider := View{
            width: 5.0
            height: Fill
            flow: Down
            cursor: ColResize

            show_bg: true
            draw_bg +: {
                bg_color: instance(ELEMENT_HOVER)
                bg_hover: instance(TEXT_FAINT)
                hover: instance(0.0)

                pixel: fn() {
                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                    let c = mix(self.bg_color, self.bg_hover, self.hover)
                    sdf.rect(0.0, 0.0, self.rect_size.x, self.rect_size.y)
                    sdf.fill(c)
                    return sdf.result
                }
            }

            animator: Animator{
                hover: {
                    default: @off
                    off: AnimatorState{ from: {all: Forward {duration: 0.1}} apply: {draw_bg: {hover: 0.0}} }
                    on: AnimatorState{ from: {all: Forward {duration: 0.1}} apply: {draw_bg: {hover: 1.0}} }
                }
            }
        }

        right := View{
            width: Fill
            height: Fill
            flow: Down
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum MpSplitPaneAction {
    Resized(f32),
    #[default]
    None,
}

#[derive(Script, ScriptHook, Widget)]
pub struct MpSplitPane {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    #[live(200.0)]
    left_width: f32,

    #[rust]
    dragging: bool,

    #[rust]
    start_x: f32,

    #[rust]
    start_width: f32,
}

impl Widget for MpSplitPane {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.handle_divider(cx, event);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpSplitPane {
    fn handle_divider(&mut self, cx: &mut Cx, event: &Event) {
        let divider = self.view.view(cx, ids!(divider));
        match event.hits(cx, divider.area()) {
            Hit::FingerHoverIn(_) => {
                if let Some(mut dv) = divider.borrow_mut() {
                    dv.cursor = Some(MouseCursor::ColResize);
                }
            }
            Hit::FingerHoverOut(_) => {
                if let Some(mut dv) = divider.borrow_mut() {
                    dv.cursor = None;
                }
            }
            Hit::FingerDown(fe) => {
                self.dragging = true;
                self.start_x = fe.abs.x as f32;
                self.start_width = self.left_width;
            }
            Hit::FingerMove(fe) => {
                if self.dragging {
                    let delta = fe.abs.x as f32 - self.start_x;
                    let new_width = (self.start_width + delta).max(40.0);
                    if (new_width - self.left_width).abs() > 0.5 {
                        self.left_width = new_width;
                        if let Some(mut lv) = self.view.view(cx, ids!(left)).borrow_mut() {
                            lv.walk.width = Size::Fixed(new_width as f64);
                        }
                        cx.widget_action(
                            self.widget_uid(),
                            MpSplitPaneAction::Resized(new_width),
                        );
                        cx.redraw_all();
                    }
                }
            }
            Hit::FingerUp(_) => {
                self.dragging = false;
            }
            _ => {}
        }
    }
}

impl MpSplitPaneRef {
    pub fn set_left_width(&self, cx: &mut Cx, width: f32) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.left_width = width.max(40.0);
            if let Some(mut lv) = inner.view.view(cx, ids!(left)).borrow_mut() {
                lv.walk.width = Size::Fixed(inner.left_width as f64);
            }
            cx.redraw_all();
        }
    }

    pub fn left_width(&self) -> f32 {
        if let Some(inner) = self.borrow() {
            inner.left_width
        } else {
            200.0
        }
    }

    pub fn resized(&self, actions: &Actions) -> Option<f32> {
        if let Some(inner) = self.borrow() {
            if let Some(action) = actions.find_widget_action(inner.widget_uid()) {
                if let MpSplitPaneAction::Resized(w) = action.cast() {
                    return Some(w);
                }
            }
        }
        None
    }
}
