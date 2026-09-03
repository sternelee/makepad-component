use crate::widgets::MpInputAction;
use crate::widgets::sizing::MpSize;
use makepad_widgets::*;

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
            highlighted: instance(0.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let bg = mix(self.bg_color, self.bg_hover, max(self.hover, self.highlighted))
                sdf.rect(0.0, 0.0, self.rect_size.x, self.rect_size.y)
                sdf.fill(bg)
                return sdf.result
            }
        }

        draw_text +: {
            text_style: theme.font_regular{font_size: 13.0}
            color: TEXT
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

        empty_label := Label{
            width: Fill
            height: Fit
            padding: Inset{left: 10, right: 10, top: 8, bottom: 8}
            draw_text +: {
                text_style: theme.font_regular{font_size: 13.0}
                color: TEXT_MUTED
            }
            text: "No matches"
            visible: false
        }
    }

    // Small variant: tighter trigger + option rows
    mod.widgets.MpComboboxSmall = mod.widgets.MpCombobox{
        input: {height: 28}
        list: {
            opt0: {size: MpSize.Small}
            opt1: {size: MpSize.Small}
            opt2: {size: MpSize.Small}
            opt3: {size: MpSize.Small}
            opt4: {size: MpSize.Small}
            opt5: {size: MpSize.Small}
            opt6: {size: MpSize.Small}
            opt7: {size: MpSize.Small}
        }
    }

    // Large variant: taller trigger + option rows
    mod.widgets.MpComboboxLarge = mod.widgets.MpCombobox{
        input: {height: 42}
        list: {
            opt0: {size: MpSize.Large}
            opt1: {size: MpSize.Large}
            opt2: {size: MpSize.Large}
            opt3: {size: MpSize.Large}
            opt4: {size: MpSize.Large}
            opt5: {size: MpSize.Large}
            opt6: {size: MpSize.Large}
            opt7: {size: MpSize.Large}
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

    /// Keyboard-highlighted option index (None = no keyboard highlight).
    #[rust]
    highlighted: Option<usize>,
}

impl Widget for MpCombobox {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Keyboard navigation while the list is open (before child dispatch, so
        // arrow keys move the highlight even while the input holds key focus).
        if self.open {
            if let Event::KeyDown(ke) = event {
                if !ke.is_repeat {
                    match ke.key_code {
                        KeyCode::ArrowDown => {
                            self.move_highlight(cx, 1);
                            return;
                        }
                        KeyCode::ArrowUp => {
                            self.move_highlight(cx, -1);
                            return;
                        }
                        KeyCode::Escape => {
                            self.open = false;
                            self.apply_filter(cx);
                            self.redraw(cx);
                            return;
                        }
                        KeyCode::ReturnKey | KeyCode::Space => {
                            if let Some(idx) = self.highlighted {
                                if let Some(label) = self.shown_labels().get(idx) {
                                    self.commit(cx, label.clone());
                                    return;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpCombobox {
    fn input_text(&self) -> String {
        self.view.child(id!(input)).as_text_input().text()
    }

    /// The filtered labels currently visible (in slot order).
    fn shown_labels(&self) -> Vec<String> {
        let query = self.input_text().to_lowercase();
        let mut shown = Vec::new();
        if self.open {
            for item in &self.items {
                if query.is_empty() || item.to_lowercase().contains(&query) {
                    shown.push(item.clone());
                    if shown.len() >= COMBOBOX_SLOTS {
                        break;
                    }
                }
            }
        }
        shown
    }

    /// Move the keyboard highlight by `delta` (wraps around), clamping to the
    /// visible option count.
    fn move_highlight(&mut self, cx: &mut Cx, delta: isize) {
        let count = self.shown_labels().len();
        if count == 0 {
            self.highlighted = None;
            return;
        }
        let next = match self.highlighted {
            Some(cur) => {
                let signed = (cur as isize + delta).rem_euclid(count as isize);
                signed as usize
            }
            None if delta > 0 => 0,
            None => count - 1,
        };
        self.highlighted = Some(next);
        self.redraw(cx);
    }

    /// Apply the highlight state to the visible option slots.
    fn apply_highlight(&mut self, cx: &mut Cx) {
        let count = self.shown_labels().len();
        for i in 0..COMBOBOX_SLOTS {
            let key = [LiveId::from_str(&format!("opt{}", i))];
            let on = self.highlighted == Some(i) && i < count;
            self.view
                .mp_combobox_option(cx, &key)
                .set_highlighted(cx, on);
        }
    }

    fn commit(&mut self, cx: &mut Cx, label: String) {
        self.view.text_input(cx, ids!(input)).set_text(cx, &label);
        self.open = false;
        self.highlighted = None;
        self.apply_filter(cx);
        cx.widget_action(self.widget_uid(), MpComboboxAction::Selected(label));
    }

    fn apply_filter(&mut self, cx: &mut Cx) {
        let shown = self.shown_labels();

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

        let any_match = self.open && !shown.is_empty();
        self.view.view(cx, ids!(list)).set_visible(cx, any_match);
        self.view
            .label(cx, ids!(empty_label))
            .set_visible(cx, self.open && !any_match);

        // Keep the keyboard highlight within range and paint it.
        if let Some(h) = self.highlighted {
            if h >= shown.len() {
                self.highlighted = None;
            }
        }
        self.apply_highlight(cx);
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
    #[live]
    highlighted: bool,

    /// Five-step size metrics (row height, padding, font) for this option.
    #[live]
    size: MpSize,

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
        // Metrics from the size system (row height + padding + font)
        let mut walk = walk;
        walk.height = Size::Fixed(self.size.min_height());
        self.layout.padding = Inset {
            left: self.size.padding_h() * 0.85,
            right: self.size.padding_h() * 0.85,
            top: 0.0,
            bottom: 0.0,
        };
        self.draw_text.text_style.font_size = self.size.font_size();
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

    pub fn set_highlighted(&mut self, cx: &mut Cx, highlighted: bool) {
        if self.highlighted != highlighted {
            self.highlighted = highlighted;
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

    pub fn set_highlighted(&self, cx: &mut Cx, highlighted: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_highlighted(cx, highlighted);
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
            if let Some(selected) = self
                .view
                .mp_combobox_option(cx, &key)
                .option_selected(actions)
            {
                self.commit(cx, selected);
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
