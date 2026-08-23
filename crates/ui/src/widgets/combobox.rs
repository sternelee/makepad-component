use makepad_widgets::*;
use crate::widgets::MpInputAction;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpCombobox - editable input with filtered dropdown list
    // ============================================================

    // Option row (fixed slots inside the dropdown)
    mod.widgets.MpComboboxOptionBase = #(MpComboboxOption::register_widget(vm))
    mod.widgets.MpComboboxOption = set_type_default() do mod.widgets.MpComboboxOptionBase{
        width: Fill
        height: 30
        padding: Inset{left: 10, right: 10, top: 0, bottom: 0}
        align: Align{x: 0.0, y: 0.5}

        draw_bg +: {
            bg_color: instance(#x0000)
            bg_hover: instance(ELEMENT_HOVER)
            hover: instance(0.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let bg = mix(self.bg_color, self.bg_hover, self.hover)
                sdf.rect(0.0, 0.0, self.rect_size.x, self.rect_size.y)
                sdf.fill(bg)
                return sdf.result
            }
        }

        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{ from: {all: Forward {duration: 0.08}} apply: {draw_bg: {hover: 0.0}} }
                on: AnimatorState{ from: {all: Forward {duration: 0.08}} apply: {draw_bg: {hover: 1.0}} }
            }
        }
    }

    // Combobox container: input + expanding filtered list
    mod.widgets.MpCombobox = set_type_default() do #(MpCombobox::register_widget(vm)){
        width: Fill
        height: Fit
        flow: Down
        spacing: 4

        input := mod.widgets.MpInput{
            height: 36
        }

        list := View{
            width: Fill
            height: Fit
            flow: Down
            spacing: 2
            padding: Inset{top: 4, right: 4, bottom: 4, left: 4}
            visible: false

            show_bg: true
            draw_bg +: {
                bg_color: instance(SURFACE_CARD)
                border_color: instance(BORDER)
                radius: instance(8.0)

                pixel: fn() {
                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                    sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                    sdf.fill_keep(self.bg_color)
                    sdf.stroke(self.border_color, 1.0)
                    return sdf.result
                }
            }

            opt0 := mod.widgets.MpComboboxOption{}
            opt1 := mod.widgets.MpComboboxOption{}
            opt2 := mod.widgets.MpComboboxOption{}
            opt3 := mod.widgets.MpComboboxOption{}
            opt4 := mod.widgets.MpComboboxOption{}
            opt5 := mod.widgets.MpComboboxOption{}
            opt6 := mod.widgets.MpComboboxOption{}
            opt7 := mod.widgets.MpComboboxOption{}
        }
    }
}

pub const COMBOBOX_SLOTS: usize = 8;

#[derive(Clone, Debug, Default)]
pub enum MpComboboxAction {
    Selected(String),
    #[default]
    None,
}

#[derive(Script, ScriptHook, Widget)]
pub struct MpCombobox {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    #[rust]
    items: Vec<String>,

    #[rust]
    open: bool,
}

impl Widget for MpCombobox {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpCombobox {
    fn input_text(&self) -> String {
        self.view
            .child(id!(input))
            .as_text_input()
            .text()
    }

    fn apply_filter(&mut self, cx: &mut Cx) {
        let query = self.input_text().to_lowercase();
        let mut shown: Vec<&String> = Vec::new();

        if self.open {
            for item in &self.items {
                if query.is_empty() || item.to_lowercase().contains(&query) {
                    shown.push(item);
                    if shown.len() >= COMBOBOX_SLOTS {
                        break;
                    }
                }
            }
        }

        for i in 0..COMBOBOX_SLOTS {
            let key = [LiveId::from_str(&format!("opt{}", i))];
            let opt = self.view.mp_combobox_option(cx, &key);
            if let Some(label) = shown.get(i) {
                opt.set_visible(cx, true);
                opt.set_label(label);
            } else {
                opt.set_visible(cx, false);
            }
        }

        self.view
            .view(cx, ids!(list))
            .set_visible(cx, self.open && !shown.is_empty());
    }
}

// ============================================================
// Option row
// ============================================================

#[derive(Script, ScriptHook, Widget, Animator)]
pub struct MpComboboxOption {
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
    label: ArcStringMut,
    #[live(true)]
    visible: bool,

    #[rust]
    area: Area,
}

impl Widget for MpComboboxOption {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        if !self.visible {
            return;
        }
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
            Hit::FingerUp(fe) if fe.is_over => {
                cx.widget_action(
                    self.uid,
                    MpComboboxAction::Selected(self.label.as_ref().to_string()),
                );
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if !self.visible {
            return DrawStep::done();
        }
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_text
            .draw_walk(cx, Walk::fit(), Align::default(), self.label.as_ref());
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        DrawStep::done()
    }
}

impl MpComboboxOption {
    pub fn set_label(&mut self, text: &str) {
        self.label.as_mut_empty().push_str(text);
    }

    pub fn set_visible(&mut self, cx: &mut Cx, visible: bool) {
        if self.visible != visible {
            self.visible = visible;
            self.redraw(cx);
        }
    }
}

impl MpComboboxOptionRef {
    pub fn option_selected(&self, actions: &Actions) -> Option<String> {
        if let Some(item) = actions.find_widget_action(self.widget_uid()) {
            if let MpComboboxAction::Selected(v) = item.cast() {
                return Some(v);
            }
        }
        None
    }

    pub fn set_label(&self, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_label(text);
        }
    }

    pub fn set_visible(&self, cx: &mut Cx, visible: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_visible(cx, visible);
        }
    }
}

impl WidgetMatchEvent for MpCombobox {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        // React to typing in the input (any text-input change re-runs the filter)
        for action in actions {
            let input_action: MpInputAction = action.cast();
            if let MpInputAction::Changed(_) = input_action {
                self.open = true;
                self.apply_filter(cx);
            }
        }

        // Option click -> commit selection
        for i in 0..COMBOBOX_SLOTS {
            let key = [LiveId::from_str(&format!("opt{}", i))];
            if let Some(selected) =
                self.view.mp_combobox_option(cx, &key).option_selected(actions)
            {
                self.view
                    .text_input(cx, ids!(input))
                    .set_text(cx, &selected);
                self.open = false;
                self.apply_filter(cx);
                cx.widget_action(self.widget_uid(), MpComboboxAction::Selected(selected));
                return;
            }
        }
    }
}

impl MpCombobox {
    pub fn set_items(&mut self, cx: &mut Cx, items: Vec<String>) {
        self.items = items;
        self.apply_filter(cx);
    }

    pub fn text(&self) -> String {
        self.input_text()
    }
}

impl MpComboboxRef {
    pub fn set_items(&self, cx: &mut Cx, items: Vec<String>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_items(cx, items);
        }
    }

    pub fn text(&self) -> String {
        if let Some(inner) = self.borrow() {
            inner.text()
        } else {
            String::new()
        }
    }

    pub fn selected(&self, actions: &Actions) -> Option<String> {
        if let Some(inner) = self.borrow() {
            if let Some(action) = actions.find_widget_action(inner.widget_uid()) {
                if let MpComboboxAction::Selected(s) = action.cast() {
                    return Some(s);
                }
            }
        }
        None
    }
}
