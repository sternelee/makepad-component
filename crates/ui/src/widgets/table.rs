use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    mod.widgets.MpTableBase = #(MpTable::register_widget(vm))

    // Register the row shader class before the widget defaults reference it
    set_type_default() do #(DrawTableRow::script_shader(vm)){
        ..mod.draw.DrawQuad

        hovered: 0.0
        selected: 0.0
        base_color: #x00000000
        hover_color: ELEMENT_HOVER
        selected_color: ELEMENT_ACTIVE

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let base = mix(self.base_color, self.hover_color, self.hovered)
            let c = mix(base, self.selected_color, self.selected)
            sdf.rect(0.0, 0.0, self.rect_size.x, self.rect_size.y)
            sdf.fill(c)
            return sdf.result
        }
    }

    mod.widgets.MpTable = set_type_default() do mod.widgets.MpTableBase{
        width: Fill
        height: Fill

        draw_bg +: {
            color: #x00000000
        }

        // Header plate sits on the content surface
        draw_header +: {
            color: SURFACE
        }

        draw_header_text +: {
            text_style: theme.font_regular{font_size: 11.5}
            color: TEXT_MUTED
        }

        draw_cell_text +: {
            text_style: theme.font_regular{font_size: 12.5}
            color: TEXT
        }

        draw_divider +: {
            color: BORDER
        }
    }
}

/// One column configuration: header label, fixed width px, sortability.
#[derive(Clone, Debug, Default)]
pub struct TableColumn {
    pub label: String,
    pub width: f64,
    pub sortable: bool,
}

impl TableColumn {
    pub fn new(label: &str, width: f64, sortable: bool) -> Self {
        Self {
            label: label.to_string(),
            width,
            sortable,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
    #[default]
    None,
}

#[derive(Clone, Debug, Default)]
pub enum MpTableAction {
    RowSelected(usize),
    Sorted(usize, SortDirection),
    #[default]
    None,
}

/// Row paint shader: mixes base / hover / selected washes.
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawTableRow {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    hovered: f32,
    #[live]
    selected: f32,
    #[live]
    base_color: Vec4f,
    #[live]
    hover_color: Vec4f,
    #[live]
    selected_color: Vec4f,
}

/// Bezel-style data table: header plate, hairline dividers, hover wash,
/// click-to-sort headers. Rows are painted manually over a flat model.
#[derive(Script, ScriptHook, Widget)]
pub struct MpTable {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_bg: DrawQuad,

    #[redraw]
    #[live]
    draw_header: DrawQuad,

    #[live]
    draw_header_text: DrawText,

    #[redraw]
    #[live]
    draw_row: DrawTableRow,

    #[live]
    draw_cell_text: DrawText,

    #[redraw]
    #[live]
    draw_divider: DrawColor,

    #[walk]
    walk: Walk,

    #[layout]
    layout: Layout,

    #[live(32.0)]
    row_height: f64,

    #[live(36.0)]
    header_height: f64,

    /// Five-step size driving row/header heights and fonts. The default
    /// Medium defers to the DSL heights (32/36); other steps override.
    #[live]
    size: MpSize,

    // Data (set by caller)
    #[rust]
    columns: Vec<TableColumn>,

    #[rust]
    rows: Vec<Vec<String>>,

    // Sort state
    #[rust]
    sort_col: Option<usize>,

    #[rust]
    sort_asc: bool,

    // Interaction state
    #[rust]
    selected_row: Option<usize>,

    #[rust]
    hovered_row: Option<usize>,

    #[rust]
    row_areas: Vec<(Area, usize)>,

    #[rust]
    header_areas: Vec<(Area, usize)>,

    #[rust]
    area: Area,
}

impl MpTable {
    pub fn set_columns(&mut self, columns: Vec<TableColumn>) {
        self.columns = columns;
        self.sort_col = None;
    }

    pub fn set_rows(&mut self, cx: &mut Cx, rows: Vec<Vec<String>>) {
        self.rows = rows;
        self.selected_row = None;
        self.redraw(cx);
    }

    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            self.redraw(cx);
        }
    }

    /// Indices of rows in current display (sort) order.
    fn display_order(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.rows.len()).collect();
        if let Some(col) = self.sort_col {
            let asc = self.sort_asc;
            order.sort_by(|&a, &b| {
                let ka = self.rows[a].get(col).map(|s| s.as_str()).unwrap_or("");
                let kb = self.rows[b].get(col).map(|s| s.as_str()).unwrap_or("");
                let ord = match (ka.parse::<f64>(), kb.parse::<f64>()) {
                    (Ok(va), Ok(vb)) => va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal),
                    _ => ka.cmp(kb),
                };
                if asc {
                    ord
                } else {
                    ord.reverse()
                }
            });
        }
        order
    }

    /// Returns the row index if a row was selected in this action batch.
    pub fn row_selected(&self, actions: &Actions) -> Option<usize> {
        if let Some(action) = actions.find_widget_action(self.widget_uid()) {
            if let MpTableAction::RowSelected(i) = action.cast() {
                return Some(i);
            }
        }
        None
    }

    /// Returns the sorted column and direction if a header was clicked.
    pub fn sorted_column(&self, actions: &Actions) -> Option<(usize, SortDirection)> {
        if let Some(action) = actions.find_widget_action(self.widget_uid()) {
            if let MpTableAction::Sorted(col, dir) = action.cast() {
                return Some((col, dir));
            }
        }
        None
    }

    fn total_width(&self) -> f64 {
        self.columns.iter().map(|c| c.width).sum::<f64>().max(120.0)
    }
}

impl Widget for MpTable {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let mut needs_redraw = false;

        for (area, row_idx) in self.row_areas.iter() {
            match event.hits(cx, *area) {
                Hit::FingerHoverIn(_) => {
                    if self.hovered_row != Some(*row_idx) {
                        self.hovered_row = Some(*row_idx);
                        cx.set_cursor(MouseCursor::Hand);
                        needs_redraw = true;
                    }
                }
                Hit::FingerHoverOut(_) => {
                    if self.hovered_row == Some(*row_idx) {
                        self.hovered_row = None;
                        cx.set_cursor(MouseCursor::Default);
                        needs_redraw = true;
                    }
                }
                Hit::FingerDown(_) => {
                    self.selected_row = Some(*row_idx);
                    cx.widget_action(self.widget_uid(), MpTableAction::RowSelected(*row_idx));
                    needs_redraw = true;
                }
                _ => {}
            }
        }

        for (area, col_idx) in self.header_areas.iter() {
            if !self.columns.get(*col_idx).map(|c| c.sortable).unwrap_or(false) {
                continue;
            }
            if let Hit::FingerUp(fe) = event.hits(cx, *area) {
                if fe.is_over && fe.was_tap() {
                    let (new_col, new_dir) = if self.sort_col == Some(*col_idx) {
                        if self.sort_asc {
                            self.sort_asc = false;
                            (*col_idx, SortDirection::Descending)
                        } else {
                            self.sort_col = None;
                            (*col_idx, SortDirection::None)
                        }
                    } else {
                        self.sort_col = Some(*col_idx);
                        self.sort_asc = true;
                        (*col_idx, SortDirection::Ascending)
                    };
                    cx.widget_action(self.widget_uid(), MpTableAction::Sorted(new_col, new_dir));
                    needs_redraw = true;
                }
            }
        }

        if needs_redraw {
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // Size system: Medium defers to the DSL heights/fonts; other steps
        // override row/header heights and text sizes.
        if self.size != MpSize::Medium {
            let (row_h, header_h) = match self.size {
                MpSize::XSmall => (26.0, 30.0),
                MpSize::Small => (29.0, 33.0),
                MpSize::Large => (38.0, 42.0),
                MpSize::XLarge => (44.0, 48.0),
                MpSize::Medium => (32.0, 36.0),
            };
            self.row_height = row_h;
            self.header_height = header_h;
            let cell_font = match self.size {
                MpSize::XSmall => 10.5,
                MpSize::Small => 11.5,
                MpSize::Large => 13.5,
                MpSize::XLarge => 15.5,
                MpSize::Medium => 12.5,
            };
            self.draw_header_text.text_style.font_size = cell_font;
            self.draw_cell_text.text_style.font_size = cell_font;
        }

        let total_width = self.total_width();
        let order = self.display_order();
        let num_cols = self.columns.len();

        self.row_areas.clear();
        self.header_areas.clear();

        self.draw_bg.begin(cx, walk, self.layout);

        // Outer container
        cx.begin_turtle(
            Walk::new(Size::Fixed(total_width), Size::fit()),
            Layout {
                flow: Flow::Down,
                spacing: 0.0,
                ..Layout::default()
            },
        );

        // === Header row ===
        cx.begin_turtle(
            Walk::new(Size::Fixed(total_width), Size::Fixed(self.header_height)),
            Layout {
                flow: Flow::right(),
                spacing: 0.0,
                ..Layout::default()
            },
        );
        for col_idx in 0..num_cols {
            let col_width = self.columns[col_idx].width;
            self.draw_header.begin(
                cx,
                Walk::new(Size::Fixed(col_width), Size::Fixed(self.header_height)),
                Layout {
                    align: Align { x: 0.0, y: 0.5 },
                    padding: Inset {
                        left: 12.0,
                        right: 8.0,
                        ..Default::default()
                    },
                    ..Layout::default()
                },
            );
            let label = if self.sort_col == Some(col_idx) {
                format!(
                    "{} {}",
                    self.columns[col_idx].label,
                    if self.sort_asc { "↑" } else { "↓" }
                )
            } else {
                self.columns[col_idx].label.clone()
            };
            self.draw_header_text
                .draw_walk(cx, Walk::fit(), Align::default(), &label);
            self.draw_header.end(cx);
            self.header_areas.push((self.draw_header.area(), col_idx));
        }
        cx.end_turtle();

        // Hairline under the header
        self.draw_divider.draw_walk(
            cx,
            Walk {
                width: Size::Fixed(total_width),
                height: Size::Fixed(1.0),
                ..Walk::default()
            },
        );

        // === Data rows ===
        for row_idx in order.iter() {
            let row = &self.rows[*row_idx];
            let is_hovered = self.hovered_row == Some(*row_idx);
            let is_selected = self.selected_row == Some(*row_idx);

            self.draw_row.hovered = if is_hovered { 1.0 } else { 0.0 };
            self.draw_row.selected = if is_selected { 1.0 } else { 0.0 };

            // Row container: cells laid out horizontally
            cx.begin_turtle(
                Walk::new(Size::Fixed(total_width), Size::Fixed(self.row_height)),
                Layout {
                    flow: Flow::right(),
                    spacing: 0.0,
                    ..Layout::default()
                },
            );

            let mut row_first_area = Area::default();
            for col_idx in 0..num_cols {
                let col_width = self.columns[col_idx].width;
                self.draw_row.begin(
                    cx,
                    Walk::new(Size::Fixed(col_width), Size::Fixed(self.row_height)),
                    Layout {
                        align: Align { x: 0.0, y: 0.5 },
                        padding: Inset {
                            left: 12.0,
                            right: 8.0,
                            ..Default::default()
                        },
                        ..Layout::default()
                    },
                );

                let text = row.get(col_idx).map(|s| s.as_str()).unwrap_or("");
                self.draw_cell_text
                    .draw_walk(cx, Walk::fit(), Align::default(), text);

                self.draw_row.end(cx);

                if col_idx == 0 {
                    row_first_area = self.draw_row.area();
                }
            }
            cx.end_turtle();

            // Hit-test proxy: first cell of the row
            self.row_areas.push((row_first_area, *row_idx));

            // Hairline divider between rows
            self.draw_divider.draw_walk(
                cx,
                Walk {
                    width: Size::Fixed(total_width),
                    height: Size::Fixed(1.0),
                    margin: Inset {
                        top: -1.0,
                        ..Default::default()
                    },
                    ..Walk::default()
                },
            );
        }

        cx.end_turtle();
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        DrawStep::done()
    }
}

impl MpTableRef {
    pub fn set_columns(&self, columns: Vec<TableColumn>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_columns(columns);
        }
    }

    pub fn set_rows(&self, cx: &mut Cx, rows: Vec<Vec<String>>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_rows(cx, rows);
        }
    }

    pub fn row_selected(&self, actions: &Actions) -> Option<usize> {
        if let Some(inner) = self.borrow() {
            inner.row_selected(actions)
        } else {
            None
        }
    }

    pub fn sorted_column(&self, actions: &Actions) -> Option<(usize, SortDirection)> {
        if let Some(inner) = self.borrow() {
            inner.sorted_column(actions)
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
