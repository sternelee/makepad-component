use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpChip - small removable tag/pill
    // ============================================================

    mod.widgets.MpChip = set_type_default() do #(MpChip::register_widget(vm)){
        width: Fit
        height: 24.0
        flow: Right
        spacing: 2
        align: Align{x: 0.0, y: 0.5}

        draw_bg +: {
            bg_color: instance(SURFACE_RAISED)
            border_color: instance(BORDER)
            radius: instance(6.0)
            hover: instance(0.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                let bg = mix(self.bg_color, ELEMENT_HOVER, self.hover)
                sdf.fill_keep(bg)
                sdf.stroke(self.border_color, 1.0)
                return sdf.result
            }
        }

        label := Label{
            width: Fit
            height: Fit
            padding: Inset{left: 8.0, right: 4.0}
            draw_text +: {
                text_style: theme.font_regular{font_size: 12.0}
                color: TEXT
            }
            text: ""
        }

        remove := mod.widgets.MpButtonGhost{
            width: 18
            height: 18
            text: "✕"
            draw_text +: { text_style: theme.font_regular{font_size: 10.0} }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum MpChipAction {
    Removed(String),
    #[default]
    None,
}

#[derive(Script, ScriptHook, Widget)]
pub struct MpChip {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    #[live]
    text: ArcStringMut,
}

impl Widget for MpChip {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for MpChip {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        if self.view.button(cx, ids!(remove)).clicked(actions) {
            let label = self.text.as_ref().to_string();
            cx.widget_action(self.widget_uid(), MpChipAction::Removed(label));
        }
    }
}

impl MpChip {
    pub fn set_text(&mut self, cx: &mut Cx, text: &str) {
        self.text.as_mut_empty().push_str(text);
        self.view
            .label(cx, ids!(label))
            .set_text(cx, text);
    }
}

impl MpChipRef {
    pub fn set_text(&self, cx: &mut Cx, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(cx, text);
        }
    }

    pub fn removed(&self, actions: &Actions) -> Option<String> {
        if let Some(inner) = self.borrow() {
            if let Some(action) = actions.find_widget_action(inner.widget_uid()) {
                if let MpChipAction::Removed(s) = action.cast() {
                    return Some(s);
                }
            }
        }
        None
    }
}
