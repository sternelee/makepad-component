use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpOptionCard - selectable option card (title + meta line)
    // ============================================================

    mod.widgets.MpOptionCard = set_type_default() do #(MpOptionCard::register_widget(vm)){
        width: Fill
        height: Fit
        padding: Inset{left: 12.0, right: 12.0, top: 10.0, bottom: 10.0}

        cursor: MouseCursor.Hand

        draw_bg +: {
            bg_color: instance(INPUT_BG)
            bg_selected: instance(SURFACE_CARD)
            border_color: instance(BORDER_STRONG)
            border_selected: instance(SOLID)
            border_width: instance(1.0)
            selected: instance(0.0)
            hovered: instance(0.0)
            radius: instance(8.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(
                    self.border_width,
                    self.border_width,
                    self.rect_size.x - self.border_width * 2.0,
                    self.rect_size.y - self.border_width * 2.0,
                    self.radius
                )
                let bg = mix(self.bg_color, self.bg_selected, self.selected)
                let bg2 = mix(bg, self.bg_selected, self.hovered * 0.5)
                sdf.fill(bg2)
                let bc = mix(self.border_color, self.border_selected, self.selected)
                sdf.stroke(bc, self.border_width)
                return sdf.result
            }
        }

        draw_title +: {
            text_style: theme.font_regular{font_size: 13.0}
            color: TEXT_MUTED
            color_active: instance(TEXT)
            selected: instance(0.0)
            get_color: fn() {
                return mix(self.color, self.color_active, self.selected)
            }
        }

        draw_meta +: {
            text_style: theme.font_regular{font_size: 12.0}
            color: TEXT_FAINT
        }

        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{ from: {all: Forward {duration: 0.1}} apply: {draw_bg: {hovered: 0.0}} }
                on: AnimatorState{ from: {all: Forward {duration: 0.1}} apply: {draw_bg: {hovered: 1.0}} }
            }
            selected_state: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.1}}
                    apply: {draw_bg: {selected: 0.0} draw_title: {selected: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.1}}
                    apply: {draw_bg: {selected: 1.0} draw_title: {selected: 1.0}}
                }
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum MpOptionCardAction {
    Clicked,
    #[default]
    None,
}

#[derive(Script, ScriptHook, Widget, Animator)]
pub struct MpOptionCard {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_bg: DrawQuad,
    #[live]
    draw_title: DrawText,
    #[live]
    draw_meta: DrawText,
    #[animator]
    animator: Animator,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[live]
    text: ArcStringMut,
    #[live]
    meta_text: ArcStringMut,

    #[rust]
    area: Area,
}

impl Widget for MpOptionCard {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
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
            Hit::FingerUp(fe) if fe.is_over && fe.was_tap() => {
                cx.widget_action(self.uid, MpOptionCardAction::Clicked);
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_bg.draw_walk(cx, walk);
        let title = self.text.as_ref().to_string();
        self.draw_title.draw_walk(cx, Walk::fit(), Align::default(), &title);
        let meta = self.meta_text.as_ref().to_string();
        if !meta.is_empty() {
            self.draw_meta.draw_walk(cx, Walk::fit(), Align::default(), &meta);
        }
        self.area = self.draw_bg.area();
        DrawStep::done()
    }
}

impl MpOptionCard {
    pub fn set_title(&mut self, cx: &mut Cx, title: &str) {
        self.text.as_mut_empty().push_str(title);
        self.redraw(cx);
    }

    pub fn set_meta(&mut self, cx: &mut Cx, meta: &str) {
        self.meta_text.as_mut_empty().push_str(meta);
        self.redraw(cx);
    }

    pub fn set_selected(&mut self, cx: &mut Cx, selected: bool) {
        if selected {
            self.animator_play(cx, ids!(selected_state.on));
        } else {
            self.animator_play(cx, ids!(selected_state.off));
        }
    }

    pub fn is_selected(&self, cx: &Cx) -> bool {
        self.animator_in_state(cx, ids!(selected_state.on))
    }
}

impl MpOptionCardRef {
    pub fn set_title(&self, cx: &mut Cx, title: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_title(cx, title);
        }
    }

    pub fn set_meta(&self, cx: &mut Cx, meta: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_meta(cx, meta);
        }
    }

    pub fn set_selected(&self, cx: &mut Cx, selected: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_selected(cx, selected);
        }
    }

    pub fn clicked(&self, actions: &Actions) -> bool {
        if let Some(inner) = self.borrow() {
            if let Some(action) = actions.find_widget_action(inner.widget_uid()) {
                return matches!(
                    action.cast::<MpOptionCardAction>(),
                    MpOptionCardAction::Clicked
                );
            }
        }
        false
    }
}
