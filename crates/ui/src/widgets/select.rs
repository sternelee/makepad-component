use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpSelect - shadcn-style select dropdown
    // ============================================================

    // Select trigger (the visible button)
    mod.widgets.MpSelectTriggerBase = #(MpSelectTrigger::register_widget(vm))
    mod.widgets.MpSelectTrigger = set_type_default() do mod.widgets.MpSelectTriggerBase{
        width: Fill
        height: 36
        padding: Inset{left: 12, right: 12, top: 0, bottom: 0}
        align: Align{x: 0.0, y: 0.5}
        flow: Right



        draw_bg +: {
            bg_color: instance(INPUT_BG)
            bg_hover: instance(ELEMENT_HOVER)
            border_color: instance(BORDER)
            focus_color: instance(ACCENT)
            radius: instance(6.0)
            has_focus: instance(0.0)
            hover: instance(0.0)
            border_width: instance(1.0)
            chevron_color: instance(TEXT_MUTED)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(self.border_width, self.border_width, self.rect_size.x - self.border_width*2.0, self.rect_size.y - self.border_width*2.0, self.radius)

                let bg = mix(self.bg_color, self.bg_hover, self.hover)
                sdf.fill(bg)

                let border_color = mix(self.border_color, self.focus_color, self.has_focus)
                let bw = mix(self.border_width, 2.0, self.has_focus)
                sdf.stroke(border_color, bw)

                return sdf.result
            }
        }

        label := Label{
            width: Fill
            height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 13.0}
                color: TEXT
            }
            text: ""
        }



        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{ from: {all: Forward {duration: 0.1}} apply: {draw_bg: {hover: 0.0}} }
                on: AnimatorState{ from: {all: Forward {duration: 0.1}} apply: {draw_bg: {hover: 1.0}} }
            }
            focus: {
                default: @off
                off: AnimatorState{ from: {all: Forward {duration: 0.15}} apply: {draw_bg: {has_focus: 0.0}} }
                on: AnimatorState{ from: {all: Forward {duration: 0.15}} apply: {draw_bg: {has_focus: 1.0}} }
            }
        }
    }

    // Select dropdown panel
    mod.widgets.MpSelectDropdownBase = #(MpSelectDropdown::register_widget(vm))
    mod.widgets.MpSelectDropdown = set_type_default() do mod.widgets.MpSelectDropdownBase{
        width: Fill
        height: Fit
        flow: Down
        visible: false






        draw_bg +: {
            bg_color: instance(SURFACE_CARD)
            border_color: instance(BORDER)
            radius: instance(8.0)
            shadow_color: instance(#x00000022)
            shadow_offset_y: instance(4.0)
            shadow_blur: instance(12.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0.0, self.shadow_offset_y, self.rect_size.x, self.rect_size.y, self.radius)
                sdf.blur = self.shadow_blur
                sdf.fill(self.shadow_color)
                sdf.blur = 0.0
                sdf.box(0.5, 0.5, self.rect_size.x-1.0, self.rect_size.y-1.0, self.radius)
                sdf.fill_keep(self.bg_color)
                sdf.stroke(self.border_color, 1.0)
                return sdf.result
            }
        }
    }

    // Select option item
    mod.widgets.MpSelectOptionBase = #(MpSelectOption::register_widget(vm))
    mod.widgets.MpSelectOption = set_type_default() do mod.widgets.MpSelectOptionBase{
        width: Fill
        height: 32
        padding: Inset{left: 12, right: 12, top: 0, bottom: 0}
        align: Align{x: 0.0, y: 0.5}



        draw_bg +: {
            bg_color: instance(#x0000)
            bg_hover: instance(ELEMENT_HOVER)
            bg_selected: instance(ELEMENT_ACTIVE)
            hover: instance(0.0)
            selected: instance(0.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let bg = mix(self.bg_color, self.bg_hover, self.hover)
                let final_bg = mix(bg, self.bg_selected, self.selected)
                sdf.rect(0.0, 0.0, self.rect_size.x, self.rect_size.y)
                sdf.fill(final_bg)
                return sdf.result
            }
        }

        label := Label{
            width: Fill
            height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 13.0}
                color: TEXT
            }
            text: ""
        }


        selected: false

        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{ from: {all: Forward {duration: 0.08}} apply: {draw_bg: {hover: 0.0}} }
                on: AnimatorState{ from: {all: Forward {duration: 0.08}} apply: {draw_bg: {hover: 1.0}} }
            }
        }
    }

    // Select (container)
    mod.widgets.MpSelect = View{
        width: Fill
        height: Fit
        flow: Overlay

        trigger := mod.widgets.MpSelectTrigger{}
        dropdown := mod.widgets.MpSelectDropdown{}
    }
}

// ============================================================
// Rust Implementation
// ============================================================

#[derive(Clone, Debug, Default)]
pub enum MpSelectAction {
    Selected(String),
    #[default]
    None,
}

#[derive(Script, ScriptHook, Widget)]
pub struct MpSelectTrigger {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
}

impl Widget for MpSelectTrigger {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        match event.hits(cx, self.view.area()) {
            Hit::FingerHoverIn(_) => {
                cx.set_cursor(MouseCursor::Hand);
            }
            Hit::FingerDown(_) => {
                cx.widget_action(self.widget_uid(), MpSelectAction::Selected("toggle".into()));
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpSelectTrigger {
    pub fn set_text(&mut self, cx: &mut Cx, text: &str) {
        self.view.label(cx, ids!(label)).set_text(cx, text);
    }
}

impl MpSelectTriggerRef {
    pub fn set_text(&self, cx: &mut Cx, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(cx, text);
        }
    }
}

#[derive(Script, ScriptHook, Widget, Animator)]
pub struct MpSelectOption {
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
    value: ArcStringMut,
    #[live]
    selected: bool,

    #[rust]
    area: Area,
}

impl Widget for MpSelectOption {
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
                    cx.widget_action(
                        uid,
                        MpSelectAction::Selected(self.value.as_ref().to_string()),
                    );
                }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_text
            .draw_walk(cx, Walk::fit(), Align::default(), self.value.as_ref());
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        DrawStep::done()
    }
}

impl MpSelectOption {
    pub fn set_selected(&mut self, cx: &mut Cx, selected: bool) {
        self.selected = selected;
        self.redraw(cx);
    }

    pub fn set_value(&mut self, text: &str) {
        self.value.as_mut_empty().push_str(text);
    }
}

impl MpSelectOptionRef {
    pub fn option_selected(&self, actions: &Actions) -> Option<String> {
        if let Some(item) = actions.find_widget_action(self.widget_uid()) {
            if let MpSelectAction::Selected(v) = item.cast() {
                return Some(v);
            }
        }
        None
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct MpSelectDropdown {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
}

impl Widget for MpSelectDropdown {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
