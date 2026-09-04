use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpNumberInput - numeric field with step up/down buttons
    // (gpui NumberInput simplified: text field + two-step cluster).
    // Size variants are DSL-level (MpInputSmall/Large precedent): the
    // inner TextInput's draw_text is private, so runtime MpSize cannot
    // rescale its font.
    // ============================================================

    mod.widgets.MpNumberInputBase = #(MpNumberInput::register_widget(vm))
    mod.widgets.MpNumberInput = set_type_default() do mod.widgets.MpNumberInputBase{
        width: Fit
        height: Fit

        flow: Right
        align: Align{y: 0.5}
        spacing: 4.0

        // Numeric text field (fills remaining width after the step cluster)
        input := mod.widgets.MpInputNumeric{
            width: Fill
            height: Fit
            empty_text: "0"
        }

        // Compact step buttons stacked vertically
        step_cluster := View{
            width: 22.0
            height: Fit
            flow: Down
            spacing: 2.0

            step_up := mod.widgets.MpButtonGhost{
                width: Fill
                height: Fit
                padding: Inset{left: 4.0, right: 4.0, top: 1.0, bottom: 1.0}
                text: "▲"
                draw_text +: {
                    text_style: theme.font_regular{font_size: 9.0}
                    color: TEXT_FAINT
                }
            }

            step_down := mod.widgets.MpButtonGhost{
                width: Fill
                height: Fit
                padding: Inset{left: 4.0, right: 4.0, top: 1.0, bottom: 1.0}
                text: "▼"
                draw_text +: {
                    text_style: theme.font_regular{font_size: 9.0}
                    color: TEXT_FAINT
                }
            }
        }
    }

    // Small variant: compact input + step cluster
    mod.widgets.MpNumberInputSmall = set_type_default() do mod.widgets.MpNumberInputBase{
        width: Fit
        height: Fit

        flow: Right
        align: Align{y: 0.5}
        spacing: 3.0

        input := mod.widgets.MpInputSmall{
            width: Fill
            height: Fit
            empty_text: "0"
            is_numeric_only: true
        }

        step_cluster := View{
            width: 18.0
            height: Fit
            flow: Down
            spacing: 1.0

            step_up := mod.widgets.MpButtonGhost{
                width: Fill
                height: Fit
                padding: Inset{left: 3.0, right: 3.0, top: 0.0, bottom: 0.0}
                text: "▲"
                draw_text +: {
                    text_style: theme.font_regular{font_size: 8.0}
                    color: TEXT_FAINT
                }
            }

            step_down := mod.widgets.MpButtonGhost{
                width: Fill
                height: Fit
                padding: Inset{left: 3.0, right: 3.0, top: 0.0, bottom: 0.0}
                text: "▼"
                draw_text +: {
                    text_style: theme.font_regular{font_size: 8.0}
                    color: TEXT_FAINT
                }
            }
        }
    }

    // Large variant: roomier input + step cluster
    mod.widgets.MpNumberInputLarge = set_type_default() do mod.widgets.MpNumberInputBase{
        width: Fit
        height: Fit

        flow: Right
        align: Align{y: 0.5}
        spacing: 6.0

        input := mod.widgets.MpInputLarge{
            width: Fill
            height: Fit
            empty_text: "0"
            is_numeric_only: true
        }

        step_cluster := View{
            width: 26.0
            height: Fit
            flow: Down
            spacing: 3.0

            step_up := mod.widgets.MpButtonGhost{
                width: Fill
                height: Fit
                padding: Inset{left: 5.0, right: 5.0, top: 2.0, bottom: 2.0}
                text: "▲"
                draw_text +: {
                    text_style: theme.font_regular{font_size: 11.0}
                    color: TEXT_FAINT
                }
            }

            step_down := mod.widgets.MpButtonGhost{
                width: Fill
                height: Fit
                padding: Inset{left: 5.0, right: 5.0, top: 2.0, bottom: 2.0}
                text: "▼"
                draw_text +: {
                    text_style: theme.font_regular{font_size: 11.0}
                    color: TEXT_FAINT
                }
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum MpNumberInputAction {
    /// The value changed (button step or committed edit).
    Changed(f64),
    #[default]
    None,
}

/// Numeric input: text field + step cluster. The field accepts typed
/// numbers; the buttons step by `step` within `[min, max]`.
#[derive(Script, ScriptHook, Widget)]
pub struct MpNumberInput {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    /// Current numeric value.
    #[rust]
    value: f64,
    /// Step increment applied by the buttons.
    #[rust]
    step: f64,
    /// Inclusive bounds.
    #[rust]
    min: f64,
    #[rust]
    max: f64,
    /// Decimal places for formatting (0 = integer).
    #[rust]
    decimals: usize,
    /// Set the text field programmatically without re-parsing.
    #[rust]
    syncing: bool,
}

impl MpNumberInput {
    fn clamp(&self, v: f64) -> f64 {
        clamp_number(v, self.min, self.max)
    }

    fn format(&self, v: f64) -> String {
        format_number(v, self.decimals)
    }

    /// Write `value` into the text field (optionally emitting an action).
    fn commit(&mut self, cx: &mut Cx, emit: bool) {
        self.syncing = true;
        self.view
            .text_input(cx, &[id!(input)])
            .set_text(cx, &self.format(self.value));
        self.syncing = false;
        if emit {
            cx.widget_action(
                self.widget_uid(),
                MpNumberInputAction::Changed(self.value),
            );
        }
        self.view.redraw(cx);
    }

    /// Step by `delta`, snapping to the step grid, then clamp.
    fn step_by(&mut self, cx: &mut Cx, delta: f64) {
        let snapped = snap_step(self.clamp(self.value + delta), self.step);
        self.value = self.clamp(snapped);
        self.commit(cx, true);
    }
}

impl Widget for MpNumberInput {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for MpNumberInput {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        // Step buttons
        let up = self.view.button(cx, &[id!(step_up)]).clicked(actions);
        let down = self.view.button(cx, &[id!(step_down)]).clicked(actions);
        if up {
            self.step_by(cx, self.step);
        }
        if down {
            self.step_by(cx, -self.step);
        }
        if up || down {
            return;
        }

        // Typed text: parse on change; clamp + rewrite when out of bounds.
        // Programmatic set_text (commit) must not re-parse its own output.
        if self.syncing {
            return;
        }
        if let Some(action) = self.view.text_input(cx, &[id!(input)]).changed(actions) {
            if let Some(parsed) = action.trim().parse::<f64>().ok() {
                let clamped = self.clamp(parsed);
                if (clamped - parsed).abs() > 1e-9 {
                    // Out of bounds: rewrite the field to the clamped value
                    self.value = clamped;
                    self.commit(cx, false);
                } else {
                    self.value = clamped;
                    cx.widget_action(
                        self.widget_uid(),
                        MpNumberInputAction::Changed(self.value),
                    );
                    self.view.redraw(cx);
                }
            }
        }
    }
}

impl MpNumberInput {
    /// Configure bounds and step; `decimals` controls formatting
    /// (0 = integer display).
    pub fn set_bounds(&mut self, cx: &mut Cx, min: f64, max: f64, step: f64, decimals: usize) {
        self.min = min;
        self.max = max;
        self.step = if step > 0.0 { step } else { 1.0 };
        self.decimals = decimals;
        self.set_value(cx, self.value);
    }

    pub fn set_value(&mut self, cx: &mut Cx, value: f64) {
        self.value = self.clamp(value);
        self.commit(cx, false);
    }

    pub fn value(&self) -> f64 {
        self.value
    }
}

impl MpNumberInputRef {
    pub fn set_bounds(&self, cx: &mut Cx, min: f64, max: f64, step: f64, decimals: usize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_bounds(cx, min, max, step, decimals);
        }
    }

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

    /// Value changed since the last action snapshot (step or typed edit).
    pub fn changed(&self, actions: &Actions) -> Option<f64> {
        if let Some(item) = actions.find_widget_action(self.widget_uid()) {
            if let MpNumberInputAction::Changed(v) = item.cast() {
                return Some(v);
            }
        }
        None
    }
}


/// Pure formatting helper (integer display when `decimals == 0`).
pub fn format_number(v: f64, decimals: usize) -> String {
    if decimals == 0 {
        format!("{}", v.round() as i64)
    } else {
        format!("{:.*}", decimals, v)
    }
}

/// Pure clamp helper.
pub fn clamp_number(v: f64, min: f64, max: f64) -> f64 {
    v.clamp(min, max)
}

/// Pure step-grid snap: rounds `v` onto the nearest `step` multiple.
pub fn snap_step(v: f64, step: f64) -> f64 {
    (v / step).round() * step
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_respects_decimals() {
        assert_eq!(format_number(41.7, 0), "42");
        assert_eq!(format_number(1.25, 2), "1.25");
        assert_eq!(format_number(2.0, 2), "2.00");
    }

    #[test]
    fn clamp_and_snap_bounds() {
        assert_eq!(clamp_number(150.0, 0.0, 100.0), 100.0);
        assert_eq!(clamp_number(-5.0, 0.0, 100.0), 0.0);
        assert_eq!(clamp_number(42.0, 0.0, 100.0), 42.0);
        assert_eq!(snap_step(12.0, 5.0), 10.0);
        assert_eq!(snap_step(13.0, 5.0), 15.0);
        assert_eq!(snap_step(2.6, 0.5), 2.5);
    }
}
