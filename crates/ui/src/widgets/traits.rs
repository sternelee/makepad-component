//! Capability traits (gpui-component `component_traits.rs` port).
//!
//! Small contracts every control can honour so app code can toggle
//! state uniformly across widget kinds instead of per-widget setters.

use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

/// Widgets with an enabled/disabled state.
pub trait Disableable {
    fn is_disabled(&self) -> bool;
    fn set_disabled(&mut self, cx: &mut Cx, disabled: bool);
}

/// Widgets with a boolean checked/selected state.
pub trait Selectable {
    fn is_selected(&self) -> bool;
    fn set_selected(&mut self, cx: &mut Cx, selected: bool);
}

/// Widgets honouring the five-step size system.
pub trait Sizable {
    fn size(&self) -> MpSize;
    fn set_size(&mut self, cx: &mut Cx, size: MpSize);
}

/// Widgets with a collapsed/expanded state (gpui `Collapsible`).
pub trait Collapsible {
    fn is_collapsed(&self) -> bool;
    fn set_collapsed(&mut self, cx: &mut Cx, collapsed: bool);
}

// ---------- MpButton ----------

impl Disableable for crate::widgets::button::MpButton {
    fn is_disabled(&self) -> bool {
        crate::widgets::button::MpButton::is_disabled(self)
    }
    fn set_disabled(&mut self, cx: &mut Cx, disabled: bool) {
        crate::widgets::button::MpButton::set_disabled(self, cx, disabled);
    }
}

impl Sizable for crate::widgets::button::MpButton {
    fn size(&self) -> MpSize {
        crate::widgets::button::MpButton::size(self)
    }
    fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        crate::widgets::button::MpButton::set_size(self, cx, size);
    }
}

// ---------- MpSwitch ----------

impl Disableable for crate::widgets::switch::MpSwitch {
    fn is_disabled(&self) -> bool {
        crate::widgets::switch::MpSwitch::is_disabled(self)
    }
    fn set_disabled(&mut self, cx: &mut Cx, disabled: bool) {
        crate::widgets::switch::MpSwitch::set_disabled(self, cx, disabled);
    }
}

impl Selectable for crate::widgets::switch::MpSwitch {
    fn is_selected(&self) -> bool {
        crate::widgets::switch::MpSwitch::is_on(self)
    }
    fn set_selected(&mut self, cx: &mut Cx, selected: bool) {
        crate::widgets::switch::MpSwitch::set_on(self, cx, selected);
    }
}

impl Sizable for crate::widgets::switch::MpSwitch {
    fn size(&self) -> MpSize {
        crate::widgets::switch::MpSwitch::size(self)
    }
    fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        crate::widgets::switch::MpSwitch::set_size(self, cx, size);
    }
}

// ---------- MpCheckbox ----------

impl Disableable for crate::widgets::checkbox::MpCheckbox {
    fn is_disabled(&self) -> bool {
        crate::widgets::checkbox::MpCheckbox::is_disabled(self)
    }
    fn set_disabled(&mut self, cx: &mut Cx, disabled: bool) {
        crate::widgets::checkbox::MpCheckbox::set_disabled(self, cx, disabled);
    }
}

impl Selectable for crate::widgets::checkbox::MpCheckbox {
    fn is_selected(&self) -> bool {
        crate::widgets::checkbox::MpCheckbox::is_checked(self)
    }
    fn set_selected(&mut self, cx: &mut Cx, selected: bool) {
        crate::widgets::checkbox::MpCheckbox::set_checked(self, cx, selected);
    }
}

impl Sizable for crate::widgets::checkbox::MpCheckbox {
    fn size(&self) -> MpSize {
        crate::widgets::checkbox::MpCheckbox::size(self)
    }
    fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        crate::widgets::checkbox::MpCheckbox::set_size(self, cx, size);
    }
}

// ---------- MpRadio ----------

impl Disableable for crate::widgets::radio::MpRadio {
    fn is_disabled(&self) -> bool {
        crate::widgets::radio::MpRadio::is_disabled(self)
    }
    fn set_disabled(&mut self, cx: &mut Cx, disabled: bool) {
        crate::widgets::radio::MpRadio::set_disabled(self, cx, disabled);
    }
}

impl Selectable for crate::widgets::radio::MpRadio {
    fn is_selected(&self) -> bool {
        crate::widgets::radio::MpRadio::is_checked(self)
    }
    fn set_selected(&mut self, cx: &mut Cx, selected: bool) {
        crate::widgets::radio::MpRadio::set_checked(self, cx, selected);
    }
}

impl Sizable for crate::widgets::radio::MpRadio {
    fn size(&self) -> MpSize {
        crate::widgets::radio::MpRadio::size(self)
    }
    fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        crate::widgets::radio::MpRadio::set_size(self, cx, size);
    }
}

// ---------- MpToggle ----------

impl Disableable for crate::widgets::toggle::MpToggle {
    fn is_disabled(&self) -> bool {
        crate::widgets::toggle::MpToggle::is_disabled(self)
    }
    fn set_disabled(&mut self, cx: &mut Cx, disabled: bool) {
        crate::widgets::toggle::MpToggle::set_disabled(self, cx, disabled);
    }
}

impl Selectable for crate::widgets::toggle::MpToggle {
    fn is_selected(&self) -> bool {
        crate::widgets::toggle::MpToggle::is_active(self)
    }
    fn set_selected(&mut self, cx: &mut Cx, selected: bool) {
        crate::widgets::toggle::MpToggle::set_active(self, cx, selected);
    }
}

impl Sizable for crate::widgets::toggle::MpToggle {
    fn size(&self) -> MpSize {
        crate::widgets::toggle::MpToggle::size(self)
    }
    fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        crate::widgets::toggle::MpToggle::set_size(self, cx, size);
    }
}

// ---------- MpSlider ----------

impl Disableable for crate::widgets::slider::MpSlider {
    fn is_disabled(&self) -> bool {
        crate::widgets::slider::MpSlider::is_disabled(self)
    }
    fn set_disabled(&mut self, cx: &mut Cx, disabled: bool) {
        crate::widgets::slider::MpSlider::set_disabled(self, cx, disabled);
    }
}

impl Sizable for crate::widgets::slider::MpSlider {
    fn size(&self) -> MpSize {
        crate::widgets::slider::MpSlider::size(self)
    }
    fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        crate::widgets::slider::MpSlider::set_size(self, cx, size);
    }
}

// ---------- MpTextArea ----------

impl Sizable for crate::widgets::textarea::MpTextArea {
    fn size(&self) -> MpSize {
        crate::widgets::textarea::MpTextArea::size(self)
    }
    fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        crate::widgets::textarea::MpTextArea::set_size(self, cx, size);
    }
}

// ---------- MpTab ----------

impl Selectable for crate::widgets::tab::MpTab {
    fn is_selected(&self) -> bool {
        crate::widgets::tab::MpTab::is_selected(self)
    }
    fn set_selected(&mut self, cx: &mut Cx, selected: bool) {
        crate::widgets::tab::MpTab::set_selected(self, cx, selected);
    }
}

impl Sizable for crate::widgets::tab::MpTab {
    fn size(&self) -> MpSize {
        crate::widgets::tab::MpTab::size(self)
    }
    fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        crate::widgets::tab::MpTab::set_size(self, cx, size);
    }
}

// ---------- MpStepper ----------

impl Sizable for crate::widgets::stepper::MpStepper {
    fn size(&self) -> MpSize {
        crate::widgets::stepper::MpStepper::size(self)
    }
    fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        crate::widgets::stepper::MpStepper::set_size(self, cx, size);
    }
}

// ---------- MpToggleGroup ----------

impl Sizable for crate::widgets::toggle_group::MpToggleGroup {
    fn size(&self) -> MpSize {
        crate::widgets::toggle_group::MpToggleGroup::size(self)
    }
    fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        crate::widgets::toggle_group::MpToggleGroup::set_size(self, cx, size);
    }
}

// ---------- MpAvatar ----------

impl Sizable for crate::widgets::avatar::MpAvatar {
    fn size(&self) -> MpSize {
        crate::widgets::avatar::MpAvatar::size(self)
    }
    fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        crate::widgets::avatar::MpAvatar::set_size(self, cx, size);
    }
}

// ---------- MpChip ----------

impl Sizable for crate::widgets::chip::MpChip {
    fn size(&self) -> MpSize {
        crate::widgets::chip::MpChip::size(self)
    }
    fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        crate::widgets::chip::MpChip::set_size(self, cx, size);
    }
}

// ---------- MpKbd ----------

impl Sizable for crate::widgets::kbd::MpKbd {
    fn size(&self) -> MpSize {
        crate::widgets::kbd::MpKbd::size(self)
    }
    fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        crate::widgets::kbd::MpKbd::set_size(self, cx, size);
    }
}

// ---------- MpSelect ----------

impl Sizable for crate::widgets::select::MpSelect {
    fn size(&self) -> MpSize {
        crate::widgets::select::MpSelect::size(self)
    }
    fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        crate::widgets::select::MpSelect::set_size(self, cx, size);
    }
}

// ---------- MpPagination ----------

impl Sizable for crate::widgets::pagination::MpPagination {
    fn size(&self) -> MpSize {
        crate::widgets::pagination::MpPagination::size(self)
    }
    fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        crate::widgets::pagination::MpPagination::set_size(self, cx, size);
    }
}

// ---------- MpCollapsibleTrigger ----------

impl Collapsible for crate::widgets::collapsible::MpCollapsibleTrigger {
    fn is_collapsed(&self) -> bool {
        !crate::widgets::collapsible::MpCollapsibleTrigger::is_expanded(self)
    }
    fn set_collapsed(&mut self, cx: &mut Cx, collapsed: bool) {
        crate::widgets::collapsible::MpCollapsibleTrigger::set_expanded(self, cx, !collapsed);
    }
}
