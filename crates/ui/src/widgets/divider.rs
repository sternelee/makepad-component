use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp_theme.*

    // Horizontal Divider
    mod.widgets.MpDivider = mod.widgets.SolidView{
        width: Fill
        height: 1
        show_bg: true
        draw_bg +: { color: BORDER }
    }

    // Vertical Divider
    mod.widgets.MpDividerVertical = mod.widgets.SolidView{
        width: 1
        height: Fill
        show_bg: true
        draw_bg +: { color: BORDER }
    }

    // Divider with margin
    mod.widgets.MpDividerWithMargin = mod.widgets.View{
        width: Fill
        height: Fit
        margin: Inset{top: 16.0, bottom: 16.0}

        SolidView{
            width: Fill
            height: 1
            show_bg: true
            draw_bg +: { color: BORDER }
        }
    }

    // Divider with label in center
    mod.widgets.MpDividerWithLabelBase = #(MpDividerWithLabel::register_widget(vm))
    mod.widgets.MpDividerWithLabel = set_type_default() do mod.widgets.MpDividerWithLabelBase{
        width: Fill
        height: Fit
        flow: Right
        align: Align{y: 0.5}

        text: ""

        left_line := SolidView{
            width: Fill
            height: 1
            show_bg: true
            draw_bg +: { color: BORDER }
        }

        label := Label{
            width: Fit
            margin: Inset{left: 12.0, right: 12.0}
            draw_text +: {
                text_style: theme.font_regular{font_size: 12.0}
                color: MUTED_FOREGROUND
            }
            text: ""
        }

        right_line := SolidView{
            width: Fill
            height: 1
            show_bg: true
            draw_bg +: { color: BORDER }
        }
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct MpDividerWithLabel {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    #[live]
    text: ArcStringMut,
}

impl Widget for MpDividerWithLabel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if !self.text.as_ref().is_empty() {
            self.view
                .widget(cx, ids!(label))
                .set_text(cx, self.text.as_ref());
        }
        self.view.draw_walk(cx, scope, walk)
    }
}
