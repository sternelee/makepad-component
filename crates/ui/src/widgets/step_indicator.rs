use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpStepIndicator - step-by-step progress (gpui Stepper)
    // States: passed (filled check) / active (accent) / future (muted)
    // driven by per-item animators; connectors by child visibility.
    // ============================================================

    // One step: numbered circle + title
    mod.widgets.MpStepItemBase = #(MpStepItem::register_widget(vm))
    mod.widgets.MpStepItem = set_type_default() do mod.widgets.MpStepItemBase{
        width: Fit
        height: Fit
        flow: Down
        spacing: 6.0

        // Palette baked for Rust-side label color logic
        c_text: TEXT
        c_text_muted: TEXT_MUTED
        c_text_faint: TEXT_FAINT

        step_circle := View{
            width: 28.0
            height: 28.0
            align: Align{x: 0.5, y: 0.5}

            show_bg: true
            draw_bg +: {
                // states driven by the passed/active animators
                passed: instance(0.0)
                active: instance(0.0)
                c_solid: instance(SOLID)
                c_accent: instance(ACCENT)
                c_border: instance(BORDER)

                pixel: fn() {
                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                    let r = self.rect_size.x * 0.5
                    sdf.circle(r, r, r)
                    let bg = mix(#x0000, self.c_solid, self.passed)
                    let bg = mix(bg, self.c_accent, self.active)
                    let border = mix(self.c_border, bg, max(self.passed, self.active))
                    sdf.fill(bg)
                    sdf.stroke(border, 1.0)
                    return sdf.result
                }
            }

            step_label := Label{
                width: Fit
                height: Fit
                draw_text +: {
                    text_style: theme.font_bold{font_size: 13.0}
                    color: TEXT_MUTED
                }
                text: "1"
            }

            step_check := Label{
                width: Fit
                height: Fit
                visible: false
                draw_text +: {
                    text_style: theme.font_bold{font_size: 13.0}
                    color: ON_SOLID
                }
                text: "✓"
            }
        }

        step_title := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 13.0}
                color: TEXT_MUTED
            }
            text: ""
        }

        animator: Animator{
            passed: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {step_circle: {draw_bg: {passed: 0.0}}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {step_circle: {draw_bg: {passed: 1.0}}}
                }
            }
            active: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    redraw: true
                    apply: {step_circle: {draw_bg: {active: 0.0}}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    redraw: true
                    apply: {step_circle: {draw_bg: {active: 1.0}}}
                }
            }
        }
    }

    // Connector line between steps: two stacked lines, the checked one
    // toggled via visibility (no runtime instance writes)
    mod.widgets.MpStepConnector = mod.widgets.View{
        width: Fill
        height: 2.0
        flow: Overlay

        track := View{
            width: Fill
            height: Fill

            show_bg: true
            draw_bg +: {
                bg_color: instance(BORDER)
            }
        }

        checked_line := View{
            width: Fill
            height: Fill
            visible: false

            show_bg: true
            draw_bg +: {
                bg_color: instance(SOLID)
            }
        }
    }

    // Container: 8 slots, each slot = connector + item (connector0 hidden)
    mod.widgets.MpStepIndicator = set_type_default() do #(MpStepIndicator::register_widget(vm)){
        width: Fill
        height: Fit
        flow: Right
        spacing: 8.0

        slot0 := View{
            width: Fill, height: Fit, flow: Right, spacing: 8.0
            connector0 := mod.widgets.MpStepConnector{visible: false}
            item0 := mod.widgets.MpStepItem{}
        }
        slot1 := View{
            width: Fill, height: Fit, flow: Right, spacing: 8.0
            connector1 := mod.widgets.MpStepConnector{}
            item1 := mod.widgets.MpStepItem{}
        }
        slot2 := View{
            width: Fill, height: Fit, flow: Right, spacing: 8.0
            connector2 := mod.widgets.MpStepConnector{}
            item2 := mod.widgets.MpStepItem{}
        }
        slot3 := View{
            width: Fill, height: Fit, flow: Right, spacing: 8.0
            connector3 := mod.widgets.MpStepConnector{}
            item3 := mod.widgets.MpStepItem{}
        }
        slot4 := View{
            width: Fill, height: Fit, flow: Right, spacing: 8.0
            connector4 := mod.widgets.MpStepConnector{}
            item4 := mod.widgets.MpStepItem{}
        }
        slot5 := View{
            width: Fill, height: Fit, flow: Right, spacing: 8.0
            connector5 := mod.widgets.MpStepConnector{}
            item5 := mod.widgets.MpStepItem{}
        }
        slot6 := View{
            width: Fill, height: Fit, flow: Right, spacing: 8.0
            connector6 := mod.widgets.MpStepConnector{}
            item6 := mod.widgets.MpStepItem{}
        }
        slot7 := View{
            width: Fill, height: Fit, flow: Right, spacing: 8.0
            connector7 := mod.widgets.MpStepConnector{}
            item7 := mod.widgets.MpStepItem{}
        }
    }
}

pub const STEP_INDICATOR_SLOTS: usize = 8;

#[derive(Clone, Debug, Default)]
pub enum MpStepIndicatorAction {
    Selected(usize),
    #[default]
    None,
}

#[derive(Script, ScriptHook, Widget, Animator)]
pub struct MpStepItem {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[apply_default]
    animator: Animator,

    /// Palette baked from the theme for Rust-side label color logic.
    #[live]
    c_text: Vec4f,
    #[live]
    c_text_muted: Vec4f,
    #[live]
    c_text_faint: Vec4f,

    /// Five-step size driving the circle diameter and fonts.
    #[live]
    size: MpSize,
}

impl Widget for MpStepItem {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Metrics from the size system (circle grows with icon_size)
        let diameter = self.size.icon_size() + 14.0;
        if let Some(mut circle) = self.view.view(cx, ids!(step_circle)).borrow_mut() {
            circle.walk.width = Size::Fixed(diameter);
            circle.walk.height = Size::Fixed(diameter);
        }
        if let Some(mut num) = self.view.label(cx, ids!(step_circle.step_label)).borrow_mut() {
            num.draw_text.text_style.font_size = self.size.font_size();
        }
        if let Some(mut check) = self.view.label(cx, ids!(step_circle.step_check)).borrow_mut() {
            check.draw_text.text_style.font_size = self.size.font_size();
        }
        if let Some(mut title) = self.view.label(cx, ids!(step_title)).borrow_mut() {
            title.draw_text.text_style.font_size = self.size.font_size();
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpStepItem {
    pub fn set_states(&mut self, cx: &mut Cx, passed: bool, active: bool) {
        self.animator_toggle(cx, passed, Animate::Yes, ids!(passed.on), ids!(passed.off));
        self.animator_toggle(cx, active, Animate::Yes, ids!(active.on), ids!(active.off));
        self.redraw(cx);
    }
}

impl MpStepItemRef {
    pub fn set_states(&self, cx: &mut Cx, passed: bool, active: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_states(cx, passed, active);
        }
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct MpStepIndicator {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    /// Index of the active step (0-based). Steps before it are passed.
    #[live(0usize)]
    step: usize,

    /// Gray out and ignore clicks.
    #[live(false)]
    disabled: bool,

    /// Five-step size propagated to the items and connectors.
    #[live]
    size: MpSize,

    /// Titles assigned via `set_items`, mirrored into the slots.
    #[rust]
    titles: Vec<String>,

    /// Last state applied to the slots (avoids re-applying every draw).
    #[rust]
    applied: Option<(usize, bool, MpSize, usize)>,
}

impl Widget for MpStepIndicator {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if !self.disabled {
            // Finger-up on the bar: jump to the hit step (context-menu style
            // abs-rect containment; the bar covers all item areas).
            match event.hits(cx, self.view.area()) {
                Hit::FingerUp(fe) if fe.is_over && fe.was_tap() => {
                    for i in 0..self.titles.len().min(STEP_INDICATOR_SLOTS) {
                        let item_path = [
                            LiveId::from_str(&format!("slot{}", i)),
                            LiveId::from_str(&format!("item{}", i)),
                        ];
                        let item = self.view.mp_step_item(cx, &item_path);
                        if item.area().rect(cx).contains(fe.abs) {
                            self.set_step(cx, i);
                            cx.widget_action(self.widget_uid(), MpStepIndicatorAction::Selected(i));
                            return;
                        }
                    }
                }
                _ => {}
            }
        }

        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let state = (self.step, self.disabled, self.size, self.titles.len());
        if self.applied != Some(state) {
            self.applied = Some(state);
            self.apply_state(cx);
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpStepIndicator {
    /// Sync slot visibility, titles, step states and connector fills.
    fn apply_state(&mut self, cx: &mut Cx) {
        let shown = self.titles.len().min(STEP_INDICATOR_SLOTS);
        for i in 0..STEP_INDICATOR_SLOTS {
            let slot_path = [LiveId::from_str(&format!("slot{}", i))];
            let item_path = [
                LiveId::from_str(&format!("slot{}", i)),
                LiveId::from_str(&format!("item{}", i)),
            ];
            let connector_path = [
                LiveId::from_str(&format!("slot{}", i)),
                LiveId::from_str(&format!("connector{}", i)),
            ];
            let checked_path = [
                LiveId::from_str(&format!("slot{}", i)),
                LiveId::from_str(&format!("connector{}", i)),
                LiveId::from_str("checked_line"),
            ];
            let num_path = [
                LiveId::from_str(&format!("slot{}", i)),
                LiveId::from_str(&format!("item{}", i)),
                LiveId::from_str("step_circle"),
                LiveId::from_str("step_label"),
            ];
            let check_path = [
                LiveId::from_str(&format!("slot{}", i)),
                LiveId::from_str(&format!("item{}", i)),
                LiveId::from_str("step_circle"),
                LiveId::from_str("step_check"),
            ];
            let title_path = [
                LiveId::from_str(&format!("slot{}", i)),
                LiveId::from_str(&format!("item{}", i)),
                LiveId::from_str("step_title"),
            ];

            let visible = i < shown;
            let slot = self.view.view(cx, &slot_path);
            slot.set_visible(cx, visible);
            if !visible {
                continue;
            }

            // connector0 sits before the first step; hide it.
            if i == 0 {
                self.view.view(cx, &connector_path).set_visible(cx, false);
            }

            let passed = i < self.step;
            let active = i == self.step;

            // Item states via its animators + palette
            let item = self.view.mp_step_item(cx, &item_path);
            let colors = item
                .borrow()
                .map(|inner| (inner.c_text, inner.c_text_muted, inner.c_text_faint))
                .unwrap_or_else(|| {
                    let gray = Vec4f { x: 0.5, y: 0.5, z: 0.5, w: 1.0 };
                    (gray, gray, gray)
                });
            item.set_states(cx, passed, active);

            // Number vs check label
            self.view.label(cx, &num_path).set_text(cx, &format!("{}", i + 1));
            self.view.label(cx, &check_path).set_visible(cx, passed);

            // Title color follows the state
            if let Some(mut title_label) = self.view.label(cx, &title_path).borrow_mut() {
                title_label.set_text(cx, self.titles.get(i).map(|s| s.as_str()).unwrap_or(""));
                title_label.draw_text.color = if self.disabled {
                    colors.2
                } else if active || passed {
                    colors.0
                } else {
                    colors.1
                };
            }

            // Connector fill: solid when the step before it is passed
            self.view.view(cx, &checked_path).set_visible(cx, passed);
        }
    }

    /// Set the step titles (up to 8).
    pub fn set_items(&mut self, cx: &mut Cx, titles: Vec<String>) {
        self.titles = titles;
        self.apply_state(cx);
        self.redraw(cx);
    }

    /// Jump to a step.
    pub fn set_step(&mut self, cx: &mut Cx, step: usize) {
        let clamped = step.min(self.titles.len().saturating_sub(1));
        if self.step != clamped {
            self.step = clamped;
            self.apply_state(cx);
            self.redraw(cx);
        }
    }

    pub fn step(&self) -> usize {
        self.step
    }

    pub fn set_disabled(&mut self, cx: &mut Cx, disabled: bool) {
        if self.disabled != disabled {
            self.disabled = disabled;
            self.apply_state(cx);
            self.redraw(cx);
        }
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            self.apply_state(cx);
            self.redraw(cx);
        }
    }
}

impl MpStepIndicatorRef {
    pub fn set_items(&self, cx: &mut Cx, titles: Vec<String>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_items(cx, titles);
        }
    }

    pub fn set_step(&self, cx: &mut Cx, step: usize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_step(cx, step);
        }
    }

    pub fn step(&self) -> usize {
        if let Some(inner) = self.borrow() {
            inner.step()
        } else {
            0
        }
    }

    pub fn set_disabled(&self, cx: &mut Cx, disabled: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_disabled(cx, disabled);
        }
    }

    pub fn is_disabled(&self) -> bool {
        if let Some(inner) = self.borrow() {
            inner.is_disabled()
        } else {
            false
        }
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

    pub fn selected(&self, actions: &Actions) -> Option<usize> {
        if let Some(inner) = self.borrow() {
            if let Some(action) = actions.find_widget_action(inner.widget_uid()) {
                if let MpStepIndicatorAction::Selected(i) = action.cast() {
                    return Some(i);
                }
            }
        }
        None
    }
}
