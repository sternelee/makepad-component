use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpSearchableList - filterable item list (simplified gpui
    // searchable_list): search input + programmatic string rows.
    // Rows are SEARCH_LIST_SLOTS pre-built overlay rows toggled via
    // set_visible (description_list pattern).
    // ============================================================

    // One row: overlay highlight layers + label. Clicking selects,
    // hovering brightens (layers toggled from Rust).
    mod.widgets.MpSearchRow = mod.widgets.View{
        width: Fill
        height: 30.0
        flow: Overlay
        align: Align{y: 0.5}

        row_highlight := mod.widgets.SolidView{
            width: Fill
            height: Fill
            margin: Inset{left: 2.0, right: 2.0}
            visible: false
            draw_bg +: {
                radius: instance(4.0)
                color: instance(ELEMENT_HOVER)
            }
        }

        row_selected := mod.widgets.SolidView{
            width: Fill
            height: Fill
            margin: Inset{left: 2.0, right: 2.0}
            visible: false
            draw_bg +: {
                radius: instance(4.0)
                color: instance(ACCENT_MUTED)
            }
        }

        row_label := Label{
            width: Fill
            height: Fit
            margin: Inset{left: 12.0, right: 10.0}
            draw_text +: {
                text_style: theme.font_regular{font_size: 13.0}
                color: TEXT
            }
            text: ""
        }
    }

    mod.widgets.MpSearchableList = set_type_default() do #(MpSearchableList::register_widget(vm)){
        width: Fill
        height: Fit
        flow: Down
        padding: Inset{left: 8.0, right: 8.0, top: 8.0, bottom: 8.0}
        spacing: 6.0

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

        search := mod.widgets.MpInputSearch{}

        rows := View{
            width: Fill
            height: Fit
            flow: Down
            spacing: 1.0

            row0 := mod.widgets.MpSearchRow{}
            row1 := mod.widgets.MpSearchRow{}
            row2 := mod.widgets.MpSearchRow{}
            row3 := mod.widgets.MpSearchRow{}
            row4 := mod.widgets.MpSearchRow{}
            row5 := mod.widgets.MpSearchRow{}
            row6 := mod.widgets.MpSearchRow{}
            row7 := mod.widgets.MpSearchRow{}
            row8 := mod.widgets.MpSearchRow{}
            row9 := mod.widgets.MpSearchRow{}
        }

        // Shown when more items match than there are slots
        more_label := Label{
            width: Fit
            height: Fit
            margin: Inset{left: 12.0}
            visible: false
            draw_text +: {
                text_style: theme.font_regular{font_size: 11.0}
                color: TEXT_FAINT
            }
            text: ""
        }
    }
}

pub const SEARCH_LIST_SLOTS: usize = 10;

#[derive(Clone, Debug, Default)]
pub enum MpSearchableListAction {
    /// An item row was picked; carries the item text.
    Selected(String),
    #[default]
    None,
}

/// The list is a registered widget; rows are View aliases reached by id.
#[derive(Script, ScriptHook, Widget)]
pub struct MpSearchableList {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    /// Full item set (unfiltered).
    #[rust]
    items: Vec<String>,
    /// Current search query (lowercased for matching).
    #[rust]
    query: String,
    /// Filtered items currently shown (at most SEARCH_LIST_SLOTS).
    #[rust]
    filtered: Vec<String>,
    /// Total items matching the query (may exceed the slot count).
    #[rust]
    filtered_total: usize,
    /// Index into `filtered` of the picked row.
    #[rust]
    selected: Option<usize>,
    /// Index into `filtered` of the hovered row.
    #[rust]
    hovered: Option<usize>,
    /// Hit areas captured during the last draw pass.
    #[rust]
    row_areas: Vec<(Area, usize)>,

    /// Five-step size driving row height and fonts.
    #[live]
    size: MpSize,

    /// Last size applied to the rows (avoids re-applying every draw).
    #[rust]
    applied_size: Option<MpSize>,
}

impl Widget for MpSearchableList {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        // Row hit testing (table header_areas pattern): rows are plain
        // View aliases, so match finger hits against their captured areas.
        for (area, idx) in self.row_areas.clone().iter() {
            match event.hits(cx, *area) {
                Hit::FingerHoverIn(_) => {
                    if self.hovered != Some(*idx) {
                        self.hovered = Some(*idx);
                        cx.set_cursor(MouseCursor::Hand);
                        self.sync_row_layers(cx);
                    }
                }
                Hit::FingerHoverOut(_) => {
                    if self.hovered == Some(*idx) {
                        self.hovered = None;
                        cx.set_cursor(MouseCursor::Default);
                        self.sync_row_layers(cx);
                    }
                }
                Hit::FingerDown(_) => {
                    self.selected = Some(*idx);
                    let text = self.filtered.get(*idx).cloned().unwrap_or_default();
                    cx.widget_action(
                        self.widget_uid(),
                        MpSearchableListAction::Selected(text),
                    );
                    self.sync_row_layers(cx);
                    self.redraw(cx);
                }
                _ => {}
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.applied_size != Some(self.size) {
            self.applied_size = Some(self.size);
            let row_h = match self.size {
                MpSize::XSmall => 24.0,
                MpSize::Small => 27.0,
                MpSize::Medium => 30.0,
                MpSize::Large => 36.0,
                MpSize::XLarge => 42.0,
            };
            let font = self.size.font_size();
            for i in 0..SEARCH_LIST_SLOTS {
                let row_id = LiveId::from_str(&format!("row{}", i));
                if let Some(mut row) = self.view.view(cx, &[row_id]).borrow_mut() {
                    row.walk.height = Size::Fixed(row_h);
                }
                if let Some(mut label) = self
                    .view
                    .label(cx, &[row_id, LiveId::from_str("row_label")])
                    .borrow_mut()
                {
                    label.draw_text.text_style.font_size = font;
                }
            }
        }

        // Capture row hit areas for this frame's event handling.
        self.view.draw_walk(cx, scope, walk);
        // Areas must be read AFTER the draw: views re-emit their draw calls
        // every pass, so pre-draw areas point at dead draw calls.
        self.row_areas.clear();
        for i in 0..SEARCH_LIST_SLOTS {
            let row_id = LiveId::from_str(&format!("row{}", i));
            if let Some(row) = self.view.view(cx, &[row_id]).borrow() {
                if row.visible() {
                    self.row_areas.push((row.area(), i));
                }
            }
        }

        DrawStep::done()
    }
}

impl WidgetMatchEvent for MpSearchableList {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        // Search box text changed -> re-filter and re-render rows.
        if self
            .view
            .text_input(cx, &[id!(search), id!(input)])
            .changed(actions)
            .is_some()
        {
            let text = self.view.text_input(cx, &[id!(search), id!(input)]).text();
            self.query = text.to_lowercase();
            self.selected = None;
            self.apply_filter(cx);
        }
    }
}

impl MpSearchableList {
    /// Re-filter `items` against `query` and sync the row slots.
    fn apply_filter(&mut self, cx: &mut Cx) {
        self.filtered_total = if self.query.is_empty() {
            self.items.len()
        } else {
            self.items
                .iter()
                .filter(|it| it.to_lowercase().contains(&self.query))
                .count()
        };
        self.filtered = self
            .items
            .iter()
            .filter(|it| self.query.is_empty() || it.to_lowercase().contains(&self.query))
            .take(SEARCH_LIST_SLOTS)
            .cloned()
            .collect();
        self.hovered = None;
        self.sync_rows(cx);
    }

    /// Write row labels / visibility / highlight layers.
    fn sync_rows(&mut self, cx: &mut Cx) {
        let shown = self.filtered.len();
        for i in 0..SEARCH_LIST_SLOTS {
            let row_id = LiveId::from_str(&format!("row{}", i));
            let row = self.view.view(cx, &[row_id]);
            if i < shown {
                if let Some(mut label) = self
                    .view
                    .label(cx, &[row_id, LiveId::from_str("row_label")])
                    .borrow_mut()
                {
                    label.set_text(cx, &self.filtered[i].clone());
                }
                row.set_visible(cx, true);
            } else {
                row.set_visible(cx, false);
            }
        }

        // "+N more" hint when matches exceed the slot count
        let hidden = self.filtered_total.saturating_sub(shown);
        if let Some(mut more) = self.view.label(cx, &[id!(more_label)]).borrow_mut() {
            more.set_visible(cx, hidden > 0);
            if hidden > 0 {
                more.set_text(cx, &format!("+{} more", hidden));
            }
        }
        self.sync_row_layers(cx);
        self.redraw(cx);
    }

    /// Toggle hover / selected highlight layers per row.
    fn sync_row_layers(&mut self, cx: &mut Cx) {
        for i in 0..SEARCH_LIST_SLOTS {
            let row_id = LiveId::from_str(&format!("row{}", i));
            let is_selected = self.selected == Some(i);
            let is_hovered = self.hovered == Some(i);
            self.view
                .view(cx, &[row_id, LiveId::from_str("row_selected")])
                .set_visible(cx, is_selected);
            self.view
                .view(cx, &[row_id, LiveId::from_str("row_highlight")])
                .set_visible(cx, is_hovered && !is_selected);
        }
        self.redraw(cx);
    }
}

impl MpSearchableList {
    /// Replace the item set (selection is cleared).
    pub fn set_items(&mut self, cx: &mut Cx, items: Vec<String>) {
        self.items = items;
        self.selected = None;
        self.apply_filter(cx);
    }

    pub fn set_query(&mut self, cx: &mut Cx, query: &str) {
        self.view
            .text_input(cx, &[id!(search), id!(input)])
            .set_text(cx, query);
        self.query = query.to_lowercase();
        self.selected = None;
        self.apply_filter(cx);
    }

    pub fn query(&self) -> String {
        self.query.clone()
    }

    pub fn selected_text(&self) -> Option<String> {
        self.selected.and_then(|i| self.filtered.get(i).cloned())
    }

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

impl MpSearchableListRef {
    /// Replace the item set (selection is cleared).
    pub fn set_items(&self, cx: &mut Cx, items: Vec<String>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_items(cx, items);
        }
    }

    pub fn set_query(&self, cx: &mut Cx, query: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_query(cx, query);
        }
    }

    pub fn query(&self) -> String {
        if let Some(inner) = self.borrow() {
            inner.query()
        } else {
            String::new()
        }
    }

    /// Item picked since the last action snapshot.
    pub fn selected(&self, actions: &Actions) -> Option<String> {
        if let Some(item) = actions.find_widget_action(self.widget_uid()) {
            if let MpSearchableListAction::Selected(text) = item.cast() {
                return Some(text);
            }
        }
        None
    }

    pub fn selected_text(&self) -> Option<String> {
        if let Some(inner) = self.borrow() {
            inner.selected_text()
        } else {
            None
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
