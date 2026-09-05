use crate::widgets::sizing::MpSize;
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

    /// Five-step size driving the trigger height and label font.
    #[live]
    size: MpSize,

    /// Last size applied to the label (avoids re-applying every draw).
    #[rust]
    applied_size: Option<MpSize>,
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
        // The trigger extent scales with the size system (Medium = the DSL 32).
        let mut walk = walk;
        walk.height = Size::Fixed(self.size.min_height());

        if self.applied_size != Some(self.size) {
            self.applied_size = Some(self.size);
            if let Some(mut label) = self.view.label(cx, ids!(label)).borrow_mut() {
                label.draw_text.text_style.font_size = self.size.font_size();
            }
        }

        let step = self.view.draw_walk(cx, scope, walk);
        crate::widgets::focus::register(cx, self.widget_uid(), self.view.area());
        step
    }
}

impl MpSelectTrigger {
    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            self.applied_size = None;
            self.redraw(cx);
        }
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

    /// Five-step size driving the option row height, padding and font.
    #[live]
    size: MpSize,

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
        // Metrics from the size system (Medium = the DSL 32)
        let mut walk = walk;
        walk.height = Size::Fixed(self.size.min_height());
        self.layout.padding = Inset {
            left: self.size.padding_h(),
            right: self.size.padding_h(),
            top: 0.0,
            bottom: 0.0,
        };
        self.draw_text.text_style.font_size = self.size.font_size();

        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_text
            .draw_walk(cx, Walk::fit(), Align::default(), self.value.as_ref());
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        DrawStep::done()
    }
}

impl MpSelectOption {
    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            self.redraw(cx);
        }
    }

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

    /// Set only while the panel is being drawn into the overlay draw list —
    /// in-flow draws (inside the MpSelect root) always skip, so the panel
    /// renders exactly once per frame and only when open.
    #[rust]
    draw_enabled: bool,
}

impl Widget for MpSelectDropdown {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if !self.draw_enabled {
            return DrawStep::done();
        }
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

    /// Five-step size propagated to the trigger and option rows.
    #[live]
    size: MpSize,

    /// Last size applied to the children (avoids re-applying every draw).
    #[rust]
    applied_size: Option<MpSize>,

    /// Overlay draw list the open dropdown renders into (above sibling
    /// content — in-flow it would be painted over by later widgets).
    #[rust]
    draw_list: Option<DrawList2d>,

    /// Laid-out panel size from the last overlay frame (drives the
    /// bottom-overflow flip; zero until first measured).
    #[rust]
    panel_size: DVec2,
}

impl Widget for MpSelect {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Click outside the trigger and dropdown closes the panel (same
        // geometric approach as makepad's own DropDown).
        if self.open {
            if let Event::MouseDown(e) = event {
                let trig = self.view.widget(cx, ids!(trigger)).area().rect(cx);
                let dd = self.view.widget(cx, ids!(dropdown)).area().rect(cx);
                let dd_open = dd.size.x > 0.0 && dd.size.y > 0.0;
                let inside_trigger = trig.contains(e.abs);
                let inside_dropdown = dd_open && dd.contains(e.abs);
                if !inside_trigger && !inside_dropdown {
                    self.set_open(cx, false);
                    return;
                }
            }
        }

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
                        KeyCode::Home => {
                            self.set_highlight_index(cx, 0);
                            return;
                        }
                        KeyCode::End => {
                            let count = self.option_count();
                            if count > 0 {
                                self.set_highlight_index(cx, count - 1);
                            }
                            return;
                        }
                        KeyCode::PageDown => {
                            self.move_highlight(cx, 5);
                            return;
                        }
                        KeyCode::PageUp => {
                            self.move_highlight(cx, -5);
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
        // Propagate the size to the trigger and option rows once per size
        // change (child setters request redraws, so guard them).
        if self.applied_size != Some(self.size) {
            self.applied_size = Some(self.size);
            if let Some(mut trigger) = self.view.mp_select_trigger(cx, ids!(trigger)).borrow_mut() {
                trigger.set_size(cx, self.size);
            }
            for child in self.option_refs() {
                if let Some(mut opt) = child.borrow_mut::<MpSelectOption>() {
                    opt.set_size(cx, self.size);
                }
            }
        }

        // The dropdown never renders in the document flow (later siblings
        // would paint over it); when open it is drawn into an overlay draw
        // list anchored under the trigger, like MpTooltip's popup.
        let step = self.view.draw_walk(cx, scope, walk);
        if self.open {
            self.draw_dropdown_overlay(cx, scope);
        }
        step
    }
}

impl MpSelect {
    /// Draw the open dropdown into an overlay draw list, anchored below the
    /// trigger at the trigger's width (MpTooltip::draw_popup_overlay pattern).
    /// Like shadcn, the panel flips above the trigger when it would overflow
    /// the bottom of the window. Panel height is measured on a first
    /// off-screen frame (Fit height is unknown before layout).
    fn draw_dropdown_overlay(&mut self, cx: &mut Cx2d, scope: &mut Scope) {
        let dd = self.view.widget(cx, ids!(dropdown));

        if self.draw_list.is_none() {
            self.draw_list = Some(DrawList2d::new(cx));
        }
        let draw_list = self.draw_list.as_mut().unwrap();
        draw_list.begin_overlay_reuse(cx);

        let pass_size = cx.current_pass_size();
        cx.begin_root_turtle(pass_size, Layout::flow_overlay());

        let trig = self.view.widget(cx, ids!(trigger)).area().rect(cx);
        let measured = self.panel_size.y > 0.5;
        let below_edge = trig.pos.y + trig.size.y;
        let pos = if !measured {
            // First frame: draw off-screen to measure the panel size.
            DVec2 { x: -10000.0, y: -10000.0 }
        } else if below_edge + self.panel_size.y > pass_size.y
            && trig.pos.y - self.panel_size.y >= 0.0
        {
            // Would overflow the bottom and fits above: flip up.
            DVec2 {
                x: trig.pos.x,
                y: trig.pos.y - self.panel_size.y,
            }
        } else {
            DVec2 {
                x: trig.pos.x,
                y: below_edge,
            }
        };
        let mut walk = dd.walk(cx);
        walk.abs_pos = Some(pos);
        walk.width = Size::Fixed(trig.size.x.max(120.0));
        walk.margin = Inset::default();

        // Enable drawing only for this overlay pass (see MpSelectDropdown).
        if let Some(mut inner) = dd.borrow_mut::<MpSelectDropdown>() {
            inner.draw_enabled = true;
        }
        let _ = dd.draw_walk(cx, scope, walk);
        if let Some(mut inner) = dd.borrow_mut::<MpSelectDropdown>() {
            inner.draw_enabled = false;
        }

        cx.end_pass_sized_turtle();
        draw_list.end(cx);

        // Track the laid-out panel size; request a full redraw when it first
        // becomes known (or changes) so the panel lands on the right anchor.
        let new_size = dd.area().rect(cx).size;
        if (new_size.x - self.panel_size.x).abs() > 0.5
            || (new_size.y - self.panel_size.y).abs() > 0.5
        {
            self.panel_size = new_size;
            cx.redraw_all();
        }
    }
}

impl WidgetMatchEvent for MpSelect {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        // Trigger toggle (mouse down or Enter/Space on the focused trigger).
        for item in actions.iter().filter_map(|a| a.downcast_ref::<WidgetAction>()) {
            if item.widget_uid != self.trigger_uid(cx) {
                continue;
            }
            if let Some(MpSelectAction::Selected(v)) = item.action.downcast_ref::<MpSelectAction>() {
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
    fn option_value(&self, _cx: &mut Cx, idx: usize) -> Option<String> {
        let child = self.option_refs().into_iter().nth(idx)?;
        child
            .borrow::<MpSelectOption>()
            .map(|o| o.value.as_ref().to_string())
    }

    fn option_selected_at(&self, _cx: &mut Cx, idx: usize, actions: &Actions) -> Option<String> {
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

    /// Set the keyboard highlight to an absolute index (clamped).
    fn set_highlight_index(&mut self, cx: &mut Cx, idx: usize) {
        let count = self.option_count();
        if count == 0 {
            self.highlighted = None;
            return;
        }
        self.highlighted = Some(idx.min(count - 1));
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
        // The panel's on-screen life is handled in draw_walk: when open it
        // renders into the overlay draw list (draw_dropdown_overlay), never
        // in the document flow.
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

    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            // Trigger and options re-sync on the next draw_walk.
            self.applied_size = None;
            self.redraw(cx);
        }
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

    pub fn size(&self) -> MpSize {
        if let Some(inner) = self.borrow() {
            inner.size()
        } else {
            MpSize::default()
        }
    }

    pub fn set_size(&self, cx: &mut Cx, size: MpSize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_size(cx, size);
        }
    }
}
