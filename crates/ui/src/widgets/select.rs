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
        height: 32
        padding: Inset{left: 10, right: 10, top: 0, bottom: 0}
        align: Align{x: 0.0, y: 0.5}
        flow: Right
        spacing: 8.0



        draw_bg +: {
            bg_color: instance(INPUT_BG)
            bg_hover: instance(ELEMENT_HOVER)
            border_color: instance(BORDER)
            focus_color: instance(ACCENT)
            radius: instance(8.0)
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
            highlighted: instance(0.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let bg = mix(self.bg_color, self.bg_hover, max(self.hover, self.highlighted))
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
    mod.widgets.MpSelect = set_type_default() do #(MpSelect::register_widget(vm)){
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

    #[rust]
    focused: bool,
}

impl Widget for MpSelectTrigger {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        // Sync focus state from Cx
        let has_focus = cx.has_key_focus(self.view.area());
        if has_focus != self.focused {
            self.focused = has_focus;
            self.view
                .animator_toggle(cx, has_focus, Animate::Yes, ids!(focus.on), ids!(focus.off));
        }

        match event.hits(cx, self.view.area()) {
            Hit::FingerHoverIn(_) => {
                cx.set_cursor(MouseCursor::Hand);
                self.view.animator_play(cx, ids!(hover.on));
            }
            Hit::FingerHoverOut(_) => {
                cx.set_cursor(MouseCursor::Default);
                self.view.animator_play(cx, ids!(hover.off));
            }
            Hit::FingerDown(_) => {
                cx.set_key_focus(self.view.area());
                cx.widget_action(self.widget_uid(), MpSelectAction::Selected("toggle".into()));
            }
            _ => {}
        }

        // Keyboard activation (Enter/Space) when focused
        if self.focused {
            if let Event::KeyDown(ke) = event {
                if (ke.key_code == KeyCode::ReturnKey || ke.key_code == KeyCode::Space)
                    && !ke.is_repeat
                {
                    cx.widget_action(self.widget_uid(), MpSelectAction::Selected("toggle".into()));
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let step = self.view.draw_walk(cx, scope, walk);
        crate::widgets::focus::register(cx, self.widget_uid(), self.view.area());
        step
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
    #[live]
    highlighted: bool,

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

    pub fn set_highlighted(&mut self, cx: &mut Cx, highlighted: bool) {
        if self.highlighted != highlighted {
            self.highlighted = highlighted;
            self.redraw(cx);
        }
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

    pub fn set_highlighted(&self, cx: &mut Cx, highlighted: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_highlighted(cx, highlighted);
        }
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

// ============================================================
// MpSelect — trigger + dropdown container with keyboard navigation
// ============================================================

#[derive(Script, ScriptHook, Widget)]
pub struct MpSelect {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    /// Whether the dropdown is open.
    #[rust]
    open: bool,

    /// Keyboard-highlighted option index (None = no keyboard highlight).
    #[rust]
    highlighted: Option<usize>,
}

impl Widget for MpSelect {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Keyboard navigation while the dropdown is open (before child dispatch).
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
                            self.set_open(cx, false);
                            return;
                        }
                        KeyCode::ReturnKey | KeyCode::Space => {
                            if let Some(idx) = self.highlighted {
                                if let Some(value) = self.option_value(cx, idx) {
                                    self.commit(cx, &value);
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

impl WidgetMatchEvent for MpSelect {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        // Trigger toggle (mouse down or Enter/Space on the focused trigger).
        if let Some(action) = actions.find_widget_action(self.trigger_uid(cx)) {
            if let MpSelectAction::Selected(v) = action.cast() {
                if v == "toggle" {
                    self.toggle(cx);
                }
            }
        }

        // Option click -> commit.
        let count = self.option_count();
        for i in 0..count {
            if let Some(value) = self.option_selected_at(cx, i, actions) {
                self.commit(cx, &value);
                return;
            }
        }
    }
}

impl MpSelect {
    fn trigger_uid(&self, cx: &mut Cx) -> WidgetUid {
        self.view.mp_select_trigger(cx, ids!(trigger)).widget_uid()
    }

    /// Collect the dropdown's `MpSelectOption` child refs (in tree order).
    fn option_refs(&self) -> Vec<WidgetRef> {
        let mut opts = Vec::new();
        let dropdown = self.view.child(id!(dropdown));
        dropdown.children(&mut |_id, child| {
            if child.borrow::<MpSelectOption>().is_some() {
                opts.push(child);
            }
        });
        opts
    }

    /// Number of option children in the dropdown.
    fn option_count(&self) -> usize {
        self.option_refs().len()
    }

    /// The value of option `idx` (by child index in the dropdown view).
    fn option_value(&self, cx: &mut Cx, idx: usize) -> Option<String> {
        let child = self.option_refs().into_iter().nth(idx)?;
        child
            .borrow::<MpSelectOption>()
            .map(|o| o.value.as_ref().to_string())
    }

    fn option_selected_at(&self, cx: &mut Cx, idx: usize, actions: &Actions) -> Option<String> {
        let child = self.option_refs().into_iter().nth(idx)?;
        let uid = child.widget_uid();
        if let Some(item) = actions.find_widget_action(uid) {
            if let MpSelectAction::Selected(v) = item.cast() {
                return Some(v);
            }
        }
        None
    }

    /// Move the keyboard highlight by `delta` (wraps around).
    fn move_highlight(&mut self, cx: &mut Cx, delta: isize) {
        let count = self.option_count();
        if count == 0 {
            self.highlighted = None;
            return;
        }
        let next = match self.highlighted {
            Some(cur) => ((cur as isize + delta).rem_euclid(count as isize)) as usize,
            None if delta > 0 => 0,
            None => count - 1,
        };
        self.highlighted = Some(next);
        self.apply_highlight(cx);
    }

    fn apply_highlight(&mut self, cx: &mut Cx) {
        let opts = self.option_refs();
        for (i, child) in opts.into_iter().enumerate() {
            let on = self.highlighted == Some(i);
            if let Some(mut opt) = child.borrow_mut::<MpSelectOption>() {
                opt.set_highlighted(cx, on);
            }
        }
    }

    fn set_open(&mut self, cx: &mut Cx, open: bool) {
        self.open = open;
        if !open {
            self.highlighted = None;
        }
        self.view.view(cx, ids!(dropdown)).set_visible(cx, open);
        self.redraw(cx);
    }

    fn toggle(&mut self, cx: &mut Cx) {
        self.set_open(cx, !self.open);
    }

    fn commit(&mut self, cx: &mut Cx, value: &str) {
        self.view
            .mp_select_trigger(cx, ids!(trigger))
            .set_text(cx, value);
        self.set_open(cx, false);
        cx.widget_action(
            self.widget_uid(),
            MpSelectAction::Selected(value.to_string()),
        );
    }
}

impl MpSelectRef {
    pub fn set_open(&self, cx: &mut Cx, open: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_open(cx, open);
        }
    }

    pub fn selected(&self, actions: &Actions) -> Option<String> {
        if let Some(inner) = self.borrow() {
            if let Some(action) = actions.find_widget_action(inner.widget_uid()) {
                if let MpSelectAction::Selected(s) = action.cast() {
                    return Some(s);
                }
            }
        }
        None
    }
}
