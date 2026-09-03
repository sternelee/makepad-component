use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpChip - small removable tag/pill
    // ============================================================

    mod.widgets.MpChip = set_type_default() do #(MpChip::register_widget(vm)){
        width: Fit
        height: 24.0
        flow: Right
        spacing: 2
        align: Align{x: 0.0, y: 0.5}

        // Chips are tag-like: default to the Small step (the original look)
        size: MpSize.Small

        draw_bg +: {
            bg_color: instance(SURFACE_RAISED)
            bg_hover: instance(ELEMENT_HOVER)
            border_color: instance(BORDER)
            radius: instance(6.0)
            hover: instance(0.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                let bg = mix(self.bg_color, self.bg_hover, self.hover)
                sdf.fill_keep(bg)
                sdf.stroke(self.border_color, 1.0)
                return sdf.result
            }
        }

        label := Label{
            width: Fit
            height: Fit
            padding: Inset{left: 8.0, right: 4.0}
            draw_text +: {
                text_style: theme.font_regular{font_size: 12.0}
                color: TEXT
            }
            text: ""
        }

        remove := mod.widgets.MpButtonGhost{
            width: 18
            height: 18
            text: "×"
            draw_text +: { text_style: theme.font_regular{font_size: 10.0} }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum MpChipAction {
    Removed(String),
    #[default]
    None,
}

#[derive(Script, Widget)]
pub struct MpChip {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    #[live]
    text: ArcStringMut,

    /// Five-step size driving the chip height and label font. Defaults to
    /// Small in the DSL (chips are tag-like elements).
    #[live]
    size: MpSize,

    /// Last size applied to the label (avoids re-applying every draw).
    #[rust]
    applied_size: Option<MpSize>,
}

/// Chip height for a size step (slimmer than controls: tag-like element).
fn chip_height(size: MpSize) -> f64 {
    match size {
        MpSize::XSmall => 20.0,
        MpSize::Small => 24.0,
        MpSize::Medium => 28.0,
        MpSize::Large => 34.0,
        MpSize::XLarge => 40.0,
    }
}

impl ScriptHook for MpChip {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        // Sync the DSL `text:` into the label child (set_text only runs for
        // runtime updates, so DSL-declared chips would otherwise show an
        // empty label).
        let text = self.text.as_ref().to_string();
        vm.with_cx_mut(|cx| {
            self.view.label(cx, ids!(label)).set_text(cx, &text);
        });
    }
}

impl Widget for MpChip {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let mut walk = walk;
        walk.height = Size::Fixed(chip_height(self.size));

        if self.applied_size != Some(self.size) {
            self.applied_size = Some(self.size);
            if let Some(mut label) = self.view.label(cx, ids!(label)).borrow_mut() {
                label.draw_text.text_style.font_size = self.size.font_size();
            }
            // The remove affordance stays a constant 18x18 ✕ (icons don't
            // need to scale with the tag).
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for MpChip {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        if self.view.button(cx, ids!(remove)).clicked(actions) {
            let label = self.text.as_ref().to_string();
            cx.widget_action(self.widget_uid(), MpChipAction::Removed(label));
        }
    }
}

impl MpChip {
    pub fn set_text(&mut self, cx: &mut Cx, text: &str) {
        self.text.as_mut_empty().push_str(text);
        self.view
            .label(cx, ids!(label))
            .set_text(cx, text);
    }

    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            // Label re-syncs on the next draw_walk.
            self.applied_size = None;
            self.redraw(cx);
        }
    }
}

impl MpChipRef {
    pub fn set_text(&self, cx: &mut Cx, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(cx, text);
        }
    }

    pub fn removed(&self, actions: &Actions) -> Option<String> {
        if let Some(inner) = self.borrow() {
            if let Some(action) = actions.find_widget_action(inner.widget_uid()) {
                if let MpChipAction::Removed(s) = action.cast() {
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
