use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpDescriptionList - key/value detail list (gpui DescriptionList)
    // ============================================================

    // One key/value row: content row + bottom hairline (toggled via
    // set_visible from the list, no runtime instance writes needed)
    mod.widgets.MpDescriptionRow = mod.widgets.View{
        width: Fill
        height: Fit
        flow: Down

        content := View{
            width: Fill
            height: Fit
            padding: Inset{top: 6.0, bottom: 6.0}
            flow: Right
            spacing: 12.0
            align: Align{y: 0.0}

            desc_label := Label{
                width: 120.0
                height: Fit
                draw_text +: {
                    text_style: theme.font_regular{font_size: 13.0}
                    color: TEXT_MUTED
                }
                text: ""
            }

            desc_value := Label{
                width: Fill
                height: Fit
                draw_text +: {
                    text_style: theme.font_regular{font_size: 13.0}
                    color: TEXT
                }
                text: ""
            }
        }

        row_divider := View{
            width: Fill
            height: 0.5

            show_bg: true
            draw_bg +: {
                bg_color: instance(BORDER)
            }
        }
    }

    // Bordered container wrapping the rows
    mod.widgets.MpDescriptionList = set_type_default() do #(MpDescriptionList::register_widget(vm)){
        width: Fill
        height: Fit
        flow: Down
        padding: Inset{left: 16.0, right: 16.0, top: 8.0, bottom: 8.0}

        show_bg: true
        draw_bg +: {
            bg_color: instance(SURFACE_CARD)
            border_color: instance(BORDER)
            border_width: instance(1.0)
            radius: instance(8.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(
                    0.5,
                    0.5,
                    self.rect_size.x - 1.0,
                    self.rect_size.y - 1.0,
                    self.radius
                )
                sdf.fill_keep(self.bg_color)
                sdf.stroke(self.border_color, self.border_width)
                return sdf.result
            }
        }

        row0 := mod.widgets.MpDescriptionRow{}
        row1 := mod.widgets.MpDescriptionRow{}
        row2 := mod.widgets.MpDescriptionRow{}
        row3 := mod.widgets.MpDescriptionRow{}
        row4 := mod.widgets.MpDescriptionRow{}
        row5 := mod.widgets.MpDescriptionRow{}
        row6 := mod.widgets.MpDescriptionRow{}
        row7 := mod.widgets.MpDescriptionRow{}
    }
}

pub const DESCRIPTION_LIST_SLOTS: usize = 8;

#[derive(Clone, Debug, Default)]
pub struct MpDescriptionItem {
    pub label: String,
    pub value: String,
}

impl MpDescriptionItem {
    pub fn new(label: &str, value: &str) -> Self {
        Self {
            label: label.to_string(),
            value: value.to_string(),
        }
    }
}

/// The list is a registered widget; rows are View aliases reached by id.
#[derive(Script, ScriptHook, Widget)]
pub struct MpDescriptionList {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    /// Hide the bordered container chrome (plain key/value stack).
    #[live(true)]
    bordered: bool,

    /// Five-step size driving container padding and row fonts.
    #[live]
    size: MpSize,

    /// Last size applied to the rows (avoids re-applying every draw).
    #[rust]
    applied_size: Option<MpSize>,
}

impl Widget for MpDescriptionList {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Metrics from the size system (Medium = the DSL 16px padding)
        let pad = self.size.padding_h() + 4.0;
        self.view.layout.padding = Inset {
            left: pad,
            right: pad,
            top: pad * 0.5,
            bottom: pad * 0.5,
        };

        if self.applied_size != Some(self.size) {
            self.applied_size = Some(self.size);
            let font = self.size.font_size();
            let label_w = 120.0 * (font / 13.0) as f64;
            for i in 0..DESCRIPTION_LIST_SLOTS {
                let label_id = [
                    LiveId::from_str(&format!("row{}", i)),
                    LiveId::from_str("content"),
                    LiveId::from_str("desc_label"),
                ];
                let value_id = [
                    LiveId::from_str(&format!("row{}", i)),
                    LiveId::from_str("content"),
                    LiveId::from_str("desc_value"),
                ];
                if let Some(mut row) = self
                    .view
                    .view(cx, &[LiveId::from_str(&format!("row{}", i)), LiveId::from_str("content")])
                    .borrow_mut()
                {
                    row.layout.padding = Inset {
                        top: self.size.padding_v() * 0.5,
                        bottom: self.size.padding_v() * 0.5,
                        left: 0.0,
                        right: 0.0,
                    };
                }
                if let Some(mut dl) = self.view.label(cx, &label_id).borrow_mut() {
                    dl.draw_text.text_style.font_size = font;
                    dl.walk.width = Size::Fixed(label_w);
                }
                if let Some(mut dv) = self.view.label(cx, &value_id).borrow_mut() {
                    dv.draw_text.text_style.font_size = font;
                }
            }
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpDescriptionList {
    /// Fill the list with `items`; extra slots are hidden. The hairline is
    /// shown between rows and hidden after the last visible one.
    pub fn set_items(&mut self, cx: &mut Cx, items: &[MpDescriptionItem]) {
        let shown = items.len().min(DESCRIPTION_LIST_SLOTS);
        for i in 0..DESCRIPTION_LIST_SLOTS {
            let row_id = [LiveId::from_str(&format!("row{}", i))];
            let row = self.view.view(cx, &row_id);
            if i < shown {
                row.set_visible(cx, true);
                let label_id = [
                    LiveId::from_str(&format!("row{}", i)),
                    LiveId::from_str("content"),
                    LiveId::from_str("desc_label"),
                ];
                let value_id = [
                    LiveId::from_str(&format!("row{}", i)),
                    LiveId::from_str("content"),
                    LiveId::from_str("desc_value"),
                ];
                let divider_id = [
                    LiveId::from_str(&format!("row{}", i)),
                    LiveId::from_str("row_divider"),
                ];
                self.view.label(cx, &label_id).set_text(cx, &items[i].label);
                self.view.label(cx, &value_id).set_text(cx, &items[i].value);
                row.view(cx, &divider_id).set_visible(cx, i + 1 < shown);
            } else {
                row.set_visible(cx, false);
            }
        }
        self.redraw(cx);
    }

    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            // Rows re-sync on the next draw_walk.
            self.applied_size = None;
            self.redraw(cx);
        }
    }
}

impl MpDescriptionListRef {
    pub fn set_items(&self, cx: &mut Cx, items: &[MpDescriptionItem]) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_items(cx, items);
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
}
