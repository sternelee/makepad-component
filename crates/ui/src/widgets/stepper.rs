use makepad_widgets::*;

use crate::widgets::button::MpButtonWidgetRefExt;
use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpStepper - numeric stepper (- value +)
    // ============================================================

    mod.widgets.MpStepper = set_type_default() do #(MpStepper::register_widget(vm)){
        width: Fit
        height: 32
        flow: Right
        spacing: 8
        align: Align{x: 0.0, y: 0.5}

        dec := mod.widgets.MpButtonGhost{
            width: 36
            height: Fill
            text: "-"
        }

        value_label := Label{
            width: 56
            height: Fit
            align: Align{x: 0.5, y: 0.5}
            draw_text +: {
                text_style: theme.font_regular{font_size: 13.0}
                color: TEXT
            }
            text: "0"
        }

        inc := mod.widgets.MpButtonGhost{
            width: 36
            height: Fill
            text: "+"
        }
    }

    // Size variants (metrics propagate to the -/+ buttons and value label)
    mod.widgets.MpStepperSmall = mod.widgets.MpStepper{
        size: MpSize.Small
    }

    mod.widgets.MpStepperLarge = mod.widgets.MpStepper{
        size: MpSize.Large
    }
}

#[derive(Clone, Debug, Default)]
pub enum MpStepperAction {
    Changed(f64),
    #[default]
    None,
}

#[derive(Script, Widget)]
pub struct MpStepper {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    #[live(0.0)]
    value: f64,

    #[live(1.0)]
    step: f64,

    #[live(0.0)]
    min: f64,

    #[live(100.0)]
    max: f64,

    #[live(0usize)]
    precision: usize,

    /// Five-step size system driving the stepper height, -/+ buttons and
    /// value label font.
    #[live]
    size: MpSize,

    /// Last size applied to the children (avoid re-applying every draw).
    #[rust]
    applied_size: Option<MpSize>,
}

impl Widget for MpStepper {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let size = self.size;

        // The control extent scales with the size system (Medium = the DSL 32).
        let mut walk = walk;
        walk.height = Size::Fixed(size.min_height());

        // Propagate the size to the child buttons and value label once per
        // size change (child setters may request redraws, so guard them).
        if self.applied_size != Some(size) {
            self.applied_size = Some(size);
            self.view.child(id!(dec)).as_mp_button().set_size(cx, size);
            self.view.child(id!(inc)).as_mp_button().set_size(cx, size);
            if let Some(mut label) = self.view.label(cx, ids!(value_label)).borrow_mut() {
                label.draw_text.text_style.font_size = size.font_size();
            }
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl ScriptHook for MpStepper {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        let value = self.value;
        let precision = self.precision;
        vm.with_cx_mut(|cx| {
            self.view
                .label(cx, ids!(value_label))
                .set_text(cx, &format!("{:.*}", precision, value));
        });
    }
}

impl MpStepper {
    fn format_value(&self) -> String {
        format!("{:.*}", self.precision, self.value)
    }

    pub fn set_value(&mut self, cx: &mut Cx, value: f64) {
        let clamped = value.clamp(self.min, self.max);
        if (clamped - self.value).abs() < f64::EPSILON {
            return;
        }
        self.value = clamped;
        self.sync_label(cx);
        cx.widget_action(self.widget_uid(), MpStepperAction::Changed(self.value));
    }

    fn sync_label(&mut self, cx: &mut Cx) {
        self.view
            .label(cx, ids!(value_label))
            .set_text(cx, &self.format_value());
        self.redraw(cx);
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            // Children re-sync on the next draw_walk.
            self.applied_size = None;
            self.redraw(cx);
        }
    }
}

impl WidgetMatchEvent for MpStepper {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        if self.view.button(cx, ids!(dec)).clicked(actions) {
            self.set_value(cx, self.value - self.step);
        }
        if self.view.button(cx, ids!(inc)).clicked(actions) {
            self.set_value(cx, self.value + self.step);
        }
    }
}

impl MpStepperRef {
    pub fn set_value(&self, cx: &mut Cx, value: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_value(cx, value);
        }
    }

    pub fn value(&self) -> f64 {
        if let Some(inner) = self.borrow() {
            inner.value()
        } else {
            0.0
        }
    }

    pub fn changed(&self, actions: &Actions) -> Option<f64> {
        if let Some(inner) = self.borrow() {
            if let Some(action) = actions.find_widget_action(inner.widget_uid()) {
                if let MpStepperAction::Changed(v) = action.cast() {
                    return Some(v);
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
