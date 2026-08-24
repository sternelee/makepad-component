use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpRating - interactive star rating (0..5)
    // ============================================================

    mod.widgets.MpRating = set_type_default() do #(MpRating::register_widget(vm)){
        width: Fit
        height: 24.0

        star_color: #xEAB308
        star_color_empty: TEXT_FAINT

        draw_bg +: {
            color: #x0000
        }

        draw_text0 +: {
            text_style: theme.font_regular{font_size: 20.0}
            color: TEXT_FAINT
        }
        draw_text1 +: {
            text_style: theme.font_regular{font_size: 20.0}
            color: TEXT_FAINT
        }
        draw_text2 +: {
            text_style: theme.font_regular{font_size: 20.0}
            color: TEXT_FAINT
        }
        draw_text3 +: {
            text_style: theme.font_regular{font_size: 20.0}
            color: TEXT_FAINT
        }
        draw_text4 +: {
            text_style: theme.font_regular{font_size: 20.0}
            color: TEXT_FAINT
        }
    }
}

pub const RATING_MAX: usize = 5;

#[derive(Clone, Debug, Default)]
pub enum MpRatingAction {
    Changed(usize),
    #[default]
    None,
}

#[derive(Script, ScriptHook, Widget)]
pub struct MpRating {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_bg: DrawQuad,
    #[live]
    draw_text0: DrawText,
    #[live]
    draw_text1: DrawText,
    #[live]
    draw_text2: DrawText,
    #[live]
    draw_text3: DrawText,
    #[live]
    draw_text4: DrawText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[live(0usize)]
    value: usize,
    #[live]
    star_color: Vec4f,
    #[live]
    star_color_empty: Vec4f,

    #[rust]
    area: Area,
    #[rust]
    star_rects: [Rect; RATING_MAX],
}

impl Widget for MpRating {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event.hits(cx, self.area) {
            Hit::FingerUp(fe) if fe.is_over => {
                let p = fe.abs;
                for i in 0..RATING_MAX {
                    if self.star_rects[i].contains(p) {
                        self.set_value(cx, i + 1);
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_bg.begin(cx, walk, self.layout);

        let star_w = 24.0;
        for i in 0..RATING_MAX {
            let text = if i < self.value {
                self.star_color
            } else {
                self.star_color_empty
            };
            let dt = match i {
                0 => &mut self.draw_text0,
                1 => &mut self.draw_text1,
                2 => &mut self.draw_text2,
                3 => &mut self.draw_text3,
                _ => &mut self.draw_text4,
            };
            dt.color = text;
            let rect = dt.draw_walk(
                cx,
                Walk::fixed(star_w, 24.0),
                Align::default(),
                "★",
            );
            self.star_rects[i] = rect;
        }

        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        DrawStep::done()
    }
}

impl MpRating {
    pub fn set_value(&mut self, cx: &mut Cx, value: usize) {
        let v = value.min(RATING_MAX);
        if self.value != v {
            self.value = v;
            self.redraw(cx);
            cx.widget_action(self.uid, MpRatingAction::Changed(v));
        }
    }

    pub fn value(&self) -> usize {
        self.value
    }
}

impl MpRatingRef {
    pub fn set_value(&self, cx: &mut Cx, value: usize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_value(cx, value);
        }
    }

    pub fn value(&self) -> usize {
        if let Some(inner) = self.borrow() {
            inner.value()
        } else {
            0
        }
    }

    pub fn changed(&self, actions: &Actions) -> Option<usize> {
        if let Some(inner) = self.borrow() {
            if let Some(action) = actions.find_widget_action(inner.widget_uid()) {
                if let MpRatingAction::Changed(v) = action.cast() {
                    return Some(v);
                }
            }
        }
        None
    }
}
