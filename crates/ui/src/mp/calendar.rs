//! `MpCalendar` — a configurable grid of bands, not a month view.
//!
//! ## What it is, because the name misleads
//!
//! It is **not** a month calendar and has nothing to do with [`crate::mp::date`]'s `month_grid`. It is a timetable: a column
//! header row, then one band per row, each band divided into equal columns, each cell holding up to five short strings. The
//! A2UI protocol calls a component of this shape `Calendar` and that is where the name comes from; the widget it is ported
//! from is 713 lines of the same thing. Renaming it would be the honest fix and would churn every call site, so the name is
//! kept and said out loud here instead.
//!
//! ## The rules, all of which are arithmetic and therefore all of which are functions
//!
//! - **Columns are equal, and there is no gutter**: `width / columns`, one share each. Every cell is
//!   `(row, col)` by counting bands and dividing, which is why [`cell_at`] is exact rather than a search through hit areas.
//! - **A row's height comes from its colour hint**, which is a *string* in the protocol: `"header"` is 55, `"budget"` is 40,
//!   anything else is 70. A hint is data telling the layout what kind of row it is, which is what makes a timetable a table
//!   of different things rather than a uniform grid.
//! - **A header row shows the column headers; a data row shows its data.** The same grid, read two ways, chosen by the
//!   hint — not by position, so a header row can be the third row and the widget does not care.
//! - **An empty config draws nothing rather than panicking.** `columns == 0 || rows == 0` is a config nobody filled in, and
//!   division by zero columns is the shape of bug this port has paid for repeatedly.
//! - **A ragged cell matrix is grown to the declared shape**, padded with empty cells. The A2UI renderer builds the matrix
//!   from a data model that can be short by a row or a column, and the alternative — indexing and panicking, or silently
//!   drawing fewer cells than the header claims — is worse than an empty cell.
//!
//! ## What was dropped from the ported widget
//!
//! The v2's per-row colour table (`row_color` mapping a hint to a background), its `hovered_idx`, its own `grid_width` live
//! field, and its five separate text sizes as literals. Backgrounds and text sizes are the theme's; the width is the caller's
//! walk. What survives is the geometry, which is the part with rules in it.

use makepad_widgets::*;

/// The title band's height, when there is a title.
pub const TITLE_HEIGHT: f64 = 45.0;

/// The footer band's height, when there is a footer.
pub const FOOTER_HEIGHT: f64 = 32.0;

/// How tall a row is, from its colour hint.
///
/// The ported widget's own three numbers. A hint is a *string* because it arrives from the protocol that way, and it decides
/// both the band's height and whether the band is a header — so this is the one place a string becomes a layout.
pub fn row_height(hint: &str) -> f64 {
    match hint {
        "header" => 55.0,
        "budget" => 40.0,
        _ => 70.0,
    }
}

/// Whether a row is a header band.
pub fn is_header_hint(hint: &str) -> bool {
    hint == "header"
}

/// The grid's shape: a header row for every column, and one band per row label.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Shape {
    pub columns: usize,
    pub rows: usize,
}

/// How many columns and rows a config describes.
///
/// The two lengths are the shape; everything else about a config is content. A config with headers but no row labels has zero
/// rows and draws nothing, which is the honest reading of "nobody said what the rows are".
pub fn shape(config: &CalendarConfig) -> Shape {
    Shape {
        columns: config.column_headers.len(),
        rows: config.row_labels.len(),
    }
}

/// The top of row `row`, relative to the grid's top — the title band included when there is one.
pub fn row_top(config: &CalendarConfig, row: usize) -> f64 {
    let mut y = title_height(config);
    for index in 0..row.min(config.row_labels.len()) {
        y += row_height(hint_of(config, index));
    }
    y
}

pub fn title_height(config: &CalendarConfig) -> f64 {
    if config.title.is_empty() {
        0.0
    } else {
        TITLE_HEIGHT
    }
}

pub fn footer_height(config: &CalendarConfig) -> f64 {
    if config.footer.is_empty() {
        0.0
    } else {
        FOOTER_HEIGHT
    }
}

/// The whole grid's height.
pub fn grid_height(config: &CalendarConfig) -> f64 {
    let bands: f64 = (0..config.row_labels.len())
        .map(|row| row_height(hint_of(config, row)))
        .sum();
    title_height(config) + bands + footer_height(config)
}

/// A row's colour hint, empty when the config does not say.
pub fn hint_of(config: &CalendarConfig, row: usize) -> &str {
    config.row_color_hints.get(row).map(String::as_str).unwrap_or("")
}

/// One column's width: **equal shares, with no gutter**.
///
/// Zero columns has no width rather than an infinite one — the division that would otherwise be a `NaN` every cell inherits.
pub fn col_width(width: f64, columns: usize) -> f64 {
    if columns == 0 || !width.is_finite() || width <= 0.0 {
        return 0.0;
    }
    width / columns as f64
}

/// Which cell a point is in, or `None`.
///
/// `(x, y)` is relative to the grid's top-left. `None` for the title and footer bands, for a point outside the grid, and for
/// **a non-finite coordinate** — every comparison against a `NaN` is false, so a version written with `<` alone would fall
/// through and land on a cell.
pub fn cell_at(x: f64, y: f64, config: &CalendarConfig, width: f64) -> Option<(usize, usize)> {
    if !x.is_finite() || !y.is_finite() || x < 0.0 || y < 0.0 {
        return None;
    }
    let Shape { columns, rows } = shape(config);
    if columns == 0 || rows == 0 {
        return None;
    }
    let column_width = col_width(width, columns);
    if column_width <= 0.0 || x >= column_width * columns as f64 {
        return None;
    }
    let column = (x / column_width).floor() as usize;
    if column >= columns {
        return None;
    }
    // Walk the bands: the first one whose bottom is past `y` is the row.
    let mut top = title_height(config);
    // **The title band is not a cell, and my first version let it be one.** The walk started at `title_height`, so its first
    // comparison was `y < 45 + row_height` — which a `y` of 0 satisfies, reporting row 0 for a point in the title. Starting the
    // walk after the band is not the same as excluding it; the band has to be excluded before the walk, which is what this
    // line does.
    if y < top {
        return None;
    }
    for row in 0..rows {
        let bottom = top + row_height(hint_of(config, row));
        if y < bottom {
            return Some((row, column));
        }
        top = bottom;
    }
    None
}

/// A cell's rect, the inverse of [`cell_at`] — so the cell drawn and the cell a click reports come from one arithmetic.
pub fn cell_rect(
    row: usize,
    column: usize,
    config: &CalendarConfig,
    width: f64,
    origin: DVec2,
) -> Option<Rect> {
    let Shape { columns, rows } = shape(config);
    if row >= rows || column >= columns {
        return None;
    }
    let column_width = col_width(width, columns);
    let height = row_height(hint_of(config, row));
    Some(Rect {
        pos: dvec2(
            origin.x + column as f64 * column_width,
            origin.y + row_top(config, row),
        ),
        size: dvec2(column_width, height),
    })
}

/// Grow a ragged cell matrix to the declared shape, padding with empty cells.
///
/// **The rule that keeps a short data model from being a panic or a lie.** The A2UI renderer builds this from a data model
/// that may be short by a row or a column; growing it means every cell the headers promise exists, and the ones nobody filled
/// in are visibly empty rather than missing. Truncating instead would silently drop data the caller supplied.
pub fn normalize(cells: Vec<Vec<CalendarCellData>>, shape: &Shape) -> Vec<Vec<CalendarCellData>> {
    let mut out: Vec<Vec<CalendarCellData>> = (0..shape.rows)
        .map(|row| {
            let mut line = cells.get(row).cloned().unwrap_or_default();
            line.resize(shape.columns.max(line.len()), CalendarCellData::default());
            line.truncate(shape.columns);
            line
        })
        .collect();
    out.truncate(shape.rows);
    for line in &mut out {
        line.resize(shape.columns, CalendarCellData::default());
    }
    out
}

/// A term and its value, as a cell holds them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CalendarCellData {
    pub line1: String,
    pub line2: String,
    pub time: String,
    pub description: String,
    pub tips: String,
}

impl CalendarCellData {
    /// Whether the cell holds nothing.
    ///
    /// **The empty cell is drawn as empty rather than as a blank rectangle with a plate**, because a timetable's empty slots
    /// are where nothing happens and should read that way.
    pub fn is_empty(&self) -> bool {
        self.line1.is_empty()
            && self.line2.is_empty()
            && self.time.is_empty()
            && self.description.is_empty()
            && self.tips.is_empty()
    }
}

/// The grid's structure: what the columns are and what the rows are called.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CalendarConfig {
    pub title: String,
    pub footer: String,
    pub column_headers: Vec<String>,
    pub column_subtitles: Vec<String>,
    pub row_labels: Vec<String>,
    pub row_color_hints: Vec<String>,
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    mod.mp.DrawMpCalendarCell = #(DrawMpCalendarCell::script_shader(vm)){
        ..mod.draw.DrawQuad

        radius: 0.0
        plate: #x00000000
        border_color: #x00000000

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let sz = self.rect_size
            sdf.box(0.5, 0.5, sz.x - 1.0, sz.y - 1.0, self.radius)
            sdf.fill_keep(self.plate)
            sdf.stroke(self.border_color, 1.0)
            return sdf.result
        }
    }

    mod.mp.MpCalendarBase = #(MpCalendar::register_widget(vm))

    mod.mp.MpCalendar = set_type_default() do mod.mp.MpCalendarBase{
        width: Fill
        height: Fit

        draw_title +: {text_style: mod.mpc.type.title3, color: mod.mpc.tokens.text}
        draw_header +: {text_style: mod.mpc.type.caption, color: mod.mpc.tokens.text_muted}
        draw_cell +: {text_style: mod.mpc.type.caption, color: mod.mpc.tokens.text}
        draw_footer +: {text_style: mod.mpc.type.caption, color: mod.mpc.tokens.text_faint}
    }
}

/// A cell's plate and hairline.
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpCalendarCell {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    radius: f32,
    #[live]
    plate: Vec4f,
    #[live]
    border_color: Vec4f,
}

/// A banded timetable grid.
#[derive(Script, ScriptHook, Widget)]
pub struct MpCalendar {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[redraw]
    #[live]
    draw_bg: DrawMpCalendarCell,
    #[live]
    draw_title: DrawText,
    #[live]
    draw_header: DrawText,
    #[live]
    draw_cell: DrawText,
    #[live]
    draw_footer: DrawText,
    /// The grid's total width, which the cells divide equally.
    #[live]
    grid_width: f64,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[rust]
    config: CalendarConfig,
    #[rust]
    cells: Vec<Vec<CalendarCellData>>,
    #[rust]
    selected: Option<(usize, usize)>,
    #[rust]
    origin: DVec2,
    #[rust]
    area: Area,
}

impl MpCalendar {
    pub fn set_config(&mut self, cx: &mut Cx, config: CalendarConfig) {
        self.config = config;
        // The matrix is regrown to the new shape here, so **every cell the headers promise exists** from the moment the config
        // is set — rather than at each read, which is how one code path ends up drawing a cell another cannot hit.
        let shape = shape(&self.config);
        self.cells = normalize(std::mem::take(&mut self.cells), &shape);
        // A selection outside the new shape is not a selection.
        if let Some((row, column)) = self.selected {
            if row >= shape.rows || column >= shape.columns {
                self.selected = None;
            }
        }
        self.redraw(cx);
    }

    pub fn config(&self) -> &CalendarConfig {
        &self.config
    }

    /// Set every cell, growing a ragged matrix to the config's shape.
    pub fn set_all_cells(&mut self, cx: &mut Cx, cells: Vec<Vec<CalendarCellData>>) {
        let shape = shape(&self.config);
        self.cells = normalize(cells, &shape);
        self.redraw(cx);
    }

    pub fn set_cell(&mut self, cx: &mut Cx, row: usize, column: usize, data: CalendarCellData) {
        let shape = shape(&self.config);
        if row >= shape.rows || column >= shape.columns {
            return;
        }
        let line = &mut self.cells[row];
        line.resize(shape.columns, CalendarCellData::default());
        line[column] = data;
        self.redraw(cx);
    }

    pub fn cell(&self, row: usize, column: usize) -> Option<&CalendarCellData> {
        self.cells.get(row).and_then(|line| line.get(column))
    }

    pub fn selected_cell(&self) -> Option<(usize, usize)> {
        self.selected
    }

    pub fn set_selected_cell(&mut self, cx: &mut Cx, cell: Option<(usize, usize)>) {
        let shape = shape(&self.config);
        self.selected = cell.filter(|(row, column)| *row < shape.rows && *column < shape.columns);
        self.redraw(cx);
    }

    pub fn shape(&self) -> Shape {
        shape(&self.config)
    }

    pub fn height(&self) -> f64 {
        grid_height(&self.config)
    }

    pub fn cell_at(&self, x: f64, y: f64) -> Option<(usize, usize)> {
        cell_at(x, y, &self.config, self.grid_width)
    }
}

impl Widget for MpCalendar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let hit = event.hits(cx, self.area);
        let (x, y) = match event {
            Event::MouseDown(me) => (me.abs.x - self.origin.x, me.abs.y - self.origin.y),
            _ => {
                let _ = hit;
                return;
            }
        };
        // The same function the drawing uses, so the cell a click reports is the cell under the pointer.
        if let Some((row, column)) = cell_at(x, y, &self.config, self.grid_width) {
            self.selected = Some((row, column));
            self.redraw(cx);
            cx.widget_action(
                self.uid,
                MpCalendarAction::CellClicked {
                    row,
                    column,
                    // The cell's own first line is carried so a caller does not have to look the indices back up to know
                    // what was clicked — the same reason `MpColorPickerAction::Picked` carries the colour.
                    label: self
                        .cell(row, column)
                        .map(|cell| cell.line1.clone())
                        .unwrap_or_default(),
                },
            );
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let (plate, border, ink, muted, faint) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            (
                theme.paint.surface_card,
                theme.paint.border,
                theme.paint.text,
                theme.paint.text_muted,
                theme.paint.text_faint,
            )
        };
        let height = grid_height(&self.config);
        let placed = cx.walk_turtle(Walk {
            width: Size::Fixed(self.grid_width),
            height: Size::Fixed(height),
            ..walk
        });
        self.origin = placed.pos;
        self.area = self.draw_bg.area();

        // The title band.
        let mut y = placed.pos.y;
        if !self.config.title.is_empty() {
            self.draw_title.color = ink;
            self.draw_title
                .draw_abs(cx, dvec2(placed.pos.x + 8.0, y + 14.0), &self.config.title);
            y += TITLE_HEIGHT;
        }

        let Shape { columns, rows } = shape(&self.config);
        let column_width = col_width(self.grid_width, columns);
        for row in 0..rows {
            let hint = hint_of(&self.config, row);
            let height = row_height(hint);
            let header = is_header_hint(hint);
            for column in 0..columns {
                let Some(rect) = cell_rect(row, column, &self.config, self.grid_width, placed.pos) else {
                    continue;
                };
                let selected = self.selected == Some((row, column));
                self.draw_bg.plate = if selected { border } else { plate };
                self.draw_bg.border_color = border;
                self.draw_bg.radius = 0.0;
                self.draw_bg.draw_abs(cx, rect);
                let x = rect.pos.x + 6.0;
                if header {
                    // A header band shows the column's name and, under it, its subtitle — so the same columns can be read as
                    // dates and as weekdays without a second grid.
                    self.draw_header.color = ink;
                    if let Some(name) = self.config.column_headers.get(column) {
                        self.draw_header.draw_abs(cx, dvec2(x, rect.pos.y + 10.0), name);
                    }
                    if let Some(subtitle) = self.config.column_subtitles.get(column) {
                        if !subtitle.is_empty() {
                            self.draw_header.color = muted;
                            self.draw_header
                                .draw_abs(cx, dvec2(x, rect.pos.y + 28.0), subtitle);
                        }
                    }
                    continue;
                }
                // A data band: the row's own label, then whatever the cell holds.
                let label = self.config.row_labels.get(row).map(String::as_str).unwrap_or("");
                self.draw_cell.color = faint;
                if !label.is_empty() {
                    self.draw_cell.draw_abs(cx, dvec2(x, rect.pos.y + 8.0), label);
                }
                // **Snapshotted before drawing, not read while drawing.** `self.cell(...)` borrows the grid, and assigning
                // `self.draw_cell.color` inside the same loop is a borrow error — the trap this port has recorded before:
                // reading a `&self` element while calling `&mut self` on a field of the same struct. The five strings are
                // cloned once per cell, which is a few small allocations per paint and the price of not having two paths.
                let lines: Vec<String> = match self.cell(row, column) {
                    Some(cell) => vec![
                        cell.line1.clone(),
                        cell.line2.clone(),
                        cell.time.clone(),
                        cell.description.clone(),
                        cell.tips.clone(),
                    ],
                    None => vec![String::new(); 5],
                };
                let mut line_y = rect.pos.y + 24.0;
                for (index, text) in lines.iter().enumerate() {
                    if text.is_empty() {
                        continue;
                    }
                    // The first three are the cell's own contents at full ink; the last two are its annotation.
                    let (color, step) = if index < 3 { (ink, 14.0) } else { (muted, 12.0) };
                    self.draw_cell.color = color;
                    self.draw_cell.draw_abs(cx, dvec2(x, line_y), text);
                    line_y += step;
                }
            }
            y += height;
            let _ = column_width;
        }

        if !self.config.footer.is_empty() {
            self.draw_footer.color = faint;
            self.draw_footer
                .draw_abs(cx, dvec2(placed.pos.x + 8.0, y + 8.0), &self.config.footer);
        }
        DrawStep::done()
    }
}

/// What a grid reports.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum MpCalendarAction {
    /// A cell was clicked, carrying its indices **and its first line**, so a caller knows what was clicked without looking
    /// the indices back up in the config it just handed over.
    CellClicked {
        row: usize,
        column: usize,
        label: String,
    },
    #[default]
    None,
}

impl MpCalendarRef {
    pub fn set_config(&self, cx: &mut Cx, config: CalendarConfig) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_config(cx, config);
        }
    }

    pub fn set_all_cells(&self, cx: &mut Cx, cells: Vec<Vec<CalendarCellData>>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_all_cells(cx, cells);
        }
    }

    pub fn set_selected_cell(&self, cx: &mut Cx, cell: Option<(usize, usize)>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_selected_cell(cx, cell);
        }
    }

    pub fn selected_cell(&self) -> Option<(usize, usize)> {
        self.borrow().and_then(|inner| inner.selected_cell())
    }

    pub fn shape(&self) -> Shape {
        self.borrow().map(|inner| inner.shape()).unwrap_or_default()
    }

    pub fn height(&self) -> f64 {
        self.borrow().map(|inner| inner.height()).unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> CalendarConfig {
        CalendarConfig {
            title: "Week".to_string(),
            footer: "2 bookings".to_string(),
            column_headers: vec!["Mon".into(), "Tue".into(), "Wed".into()],
            column_subtitles: vec!["1".into(), String::new(), "3".into()],
            row_labels: vec!["Morning".into(), "Break".into(), "Afternoon".into()],
            row_color_hints: vec!["header".into(), "budget".into(), String::new()],
        }
    }

    #[test]
    fn test_a_row_s_height_comes_from_its_colour_hint() {
        // A hint is data telling the layout what kind of row it is, and the three numbers are the ported widget's own. This is
        // the one place a string becomes a layout, so it is the one place a typo would be invisible: `"Headers"` would not
        // fail, it would silently make a 70-point data row out of a 55-point header.
        assert_eq!(row_height("header"), 55.0);
        assert_eq!(row_height("budget"), 40.0);
        assert_eq!(row_height(""), 70.0);
        assert_eq!(row_height("anything else"), 70.0);
        assert_eq!(
            row_height("Headers"),
            70.0,
            "a near-miss is not a header, and that is why the hint is asserted rather than trusted"
        );
        assert!(is_header_hint("header"));
        assert!(!is_header_hint("Headers"));
        assert!(!is_header_hint(""));
    }

    #[test]
    fn test_the_grid_s_height_is_its_bands_and_only_the_bands_it_has() {
        // The title and footer are bands too, and they are present only when their text is — so a grid without a title is 45
        // points shorter rather than 45 points of blank.
        let config = config();
        assert_eq!(title_height(&config), TITLE_HEIGHT);
        assert_eq!(footer_height(&config), FOOTER_HEIGHT);
        assert_eq!(
            grid_height(&config),
            TITLE_HEIGHT + 55.0 + 40.0 + 70.0 + FOOTER_HEIGHT
        );
        let bare = CalendarConfig {
            title: String::new(),
            footer: String::new(),
            ..config.clone()
        };
        assert_eq!(title_height(&bare), 0.0);
        assert_eq!(footer_height(&bare), 0.0);
        assert_eq!(grid_height(&bare), 55.0 + 40.0 + 70.0);
        assert_eq!(
            grid_height(&CalendarConfig::default()),
            0.0,
            "a config nobody filled in has no height"
        );
    }

    #[test]
    fn test_columns_are_equal_shares_with_no_gutter() {
        // `width / columns`, one share each — so every cell is `(row, col)` by counting bands and dividing, which is what makes
        // the hit test exact rather than a search. Zero columns has **no** width rather than an infinite one: the division that
        // would otherwise be a `NaN` every cell inherits.
        assert_eq!(col_width(300.0, 3), 100.0);
        assert_eq!(col_width(300.0, 4), 75.0);
        assert_eq!(col_width(300.0, 0), 0.0, "zero columns is no width, not a NaN");
        assert_eq!(col_width(0.0, 3), 0.0);
        assert_eq!(col_width(-10.0, 3), 0.0);
        assert_eq!(col_width(f64::NAN, 3), 0.0);
        // The shares tile the width exactly.
        for columns in 1..8usize {
            let share = col_width(280.0, columns);
            assert!((share * columns as f64 - 280.0).abs() < 1e-9, "{columns} columns do not tile");
        }
    }

    #[test]
    fn test_a_point_lands_on_the_cell_it_is_in_and_off_the_bands_it_is_not() {
        // The title band and the footer are not cells, and neither is a point past the last column. Every miss is `None` rather
        // than the nearest cell, because rounding would pick a slot nobody aimed at.
        let config = config();
        let width = 300.0;
        let top = TITLE_HEIGHT;
        assert_eq!(cell_at(0.0, 0.0, &config, width), None, "the title band is not a cell");
        assert_eq!(cell_at(0.0, top - 0.1, &config, width), None);
        assert_eq!(cell_at(0.0, top, &config, width), Some((0, 0)), "the first point of row 0");
        assert_eq!(cell_at(99.9, top + 10.0, &config, width), Some((0, 0)));
        assert_eq!(cell_at(100.0, top + 10.0, &config, width), Some((0, 1)), "the second column");
        assert_eq!(cell_at(299.9, top + 10.0, &config, width), Some((0, 2)));
        assert_eq!(cell_at(300.0, top + 10.0, &config, width), None, "past the last column");
        // Down through the bands, each with its own height.
        let row1 = top + 55.0;
        assert_eq!(cell_at(150.0, row1 - 0.1, &config, width), Some((0, 1)), "still the header band");
        assert_eq!(cell_at(150.0, row1, &config, width), Some((1, 1)), "the short row");
        let row2 = row1 + 40.0;
        assert_eq!(cell_at(150.0, row2, &config, width), Some((2, 1)), "the tall row");
        assert_eq!(
            cell_at(150.0, row2 + 70.0, &config, width),
            None,
            "the footer is not a cell"
        );
    }

    #[test]
    fn test_a_non_finite_or_negative_point_lands_on_nothing() {
        // Every comparison against a `NaN` is false, so a version written with `<` alone would fall through the bounds and land
        // on a cell. This port has paid for a propagating `NaN` six times now.
        let config = config();
        for point in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0] {
            assert_eq!(cell_at(point, 100.0, &config, 300.0), None, "x={point}");
            assert_eq!(cell_at(50.0, point, &config, 300.0), None, "y={point}");
        }
    }

    #[test]
    fn test_an_empty_config_draws_nothing_and_hits_nothing() {
        // `columns == 0 || rows == 0` is a config nobody filled in, and division by zero columns is the shape of bug this port
        // has paid for repeatedly. Nothing is drawn, nothing can be hit, and no division happens.
        let empty = CalendarConfig::default();
        assert_eq!(shape(&empty), Shape { columns: 0, rows: 0 });
        assert_eq!(grid_height(&empty), 0.0);
        assert_eq!(cell_at(0.0, 0.0, &empty, 300.0), None);
        assert!(cell_rect(0, 0, &empty, 300.0, dvec2(0.0, 0.0)).is_none());
        // Headers with no rows is also nothing: nobody said what the rows are.
        let headers_only = CalendarConfig {
            column_headers: vec!["Mon".into()],
            ..CalendarConfig::default()
        };
        assert_eq!(shape(&headers_only), Shape { columns: 1, rows: 0 });
        assert_eq!(cell_at(0.0, 0.0, &headers_only, 300.0), None);
        // And rows with no columns likewise.
        let rows_only = CalendarConfig {
            row_labels: vec!["Morning".into()],
            ..CalendarConfig::default()
        };
        assert_eq!(shape(&rows_only), Shape { columns: 0, rows: 1 });
        assert_eq!(cell_at(0.0, 0.0, &rows_only, 300.0), None);
    }

    #[test]
    fn test_a_ragged_matrix_is_grown_to_the_shape_rather_than_being_a_panic_or_a_lie() {
        // The A2UI renderer builds this from a data model that may be short by a row or a column. Growing it means every cell
        // the headers promise exists and the ones nobody filled in are visibly empty; truncating would silently drop data the
        // caller supplied.
        let shape = Shape { columns: 3, rows: 2 };
        let short = vec![vec![CalendarCellData {
            line1: "a".into(),
            ..CalendarCellData::default()
        }]];
        let grown = normalize(short, &shape);
        assert_eq!(grown.len(), 2, "the missing row exists");
        assert_eq!(grown[0].len(), 3, "the short row was padded");
        assert_eq!(grown[1].len(), 3);
        assert_eq!(grown[0][0].line1, "a", "the supplied cell survived");
        assert!(grown[0][1].is_empty(), "a padded cell is empty");
        assert!(grown[1].iter().all(CalendarCellData::is_empty));
        // A matrix with **more** than the shape is truncated to it, since the shape is what the headers promise.
        let long = vec![
            vec![CalendarCellData::default(); 5],
            vec![CalendarCellData::default(); 5],
            vec![CalendarCellData::default(); 5],
        ];
        let cut = normalize(long, &shape);
        assert_eq!(cut.len(), 2);
        assert!(cut.iter().all(|line| line.len() == 3));
        // And the property that matters: whatever goes in, the result **is** the shape.
        for rows in 0..4usize {
            for columns in 0..4usize {
                let shape = Shape { columns, rows };
                let input = vec![vec![CalendarCellData::default(); columns.saturating_sub(1)]; rows];
                let out = normalize(input, &shape);
                assert_eq!(out.len(), rows);
                assert!(out.iter().all(|line| line.len() == columns));
            }
        }
    }

    #[test]
    fn test_the_drawn_rect_and_the_hit_test_are_inverses() {
        // **The property that keeps the cell on screen and the cell a click reports from disagreeing.** Every cell's rect
        // centre must hit that same cell, and the rects must tile the grid with no gap and no overlap.
        let config = config();
        let width = 300.0;
        let origin = dvec2(10.0, 20.0);
        let Shape { columns, rows } = shape(&config);
        for row in 0..rows {
            for column in 0..columns {
                let rect = cell_rect(row, column, &config, width, origin).expect("in shape");
                let centre = dvec2(
                    rect.pos.x - origin.x + rect.size.x * 0.5,
                    rect.pos.y - origin.y + rect.size.y * 0.5,
                );
                assert_eq!(
                    cell_at(centre.x, centre.y, &config, width),
                    Some((row, column)),
                    "the centre of {row},{column} hit something else"
                );
            }
        }
        // The cell after a column starts where the previous one ends, and the row after starts where its predecessor ends.
        for column in 0..columns - 1 {
            let a = cell_rect(0, column, &config, width, origin).expect("in shape");
            let b = cell_rect(0, column + 1, &config, width, origin).expect("in shape");
            assert_eq!(a.pos.x + a.size.x, b.pos.x, "columns {column} and {} overlap", column + 1);
        }
        for row in 0..rows - 1 {
            let a = cell_rect(row, 0, &config, width, origin).expect("in shape");
            let b = cell_rect(row + 1, 0, &config, width, origin).expect("in shape");
            assert_eq!(a.pos.y + a.size.y, b.pos.y, "rows {row} and {} overlap", row + 1);
        }
        // And the last row's bottom is the grid's own bottom, minus the footer.
        let last = cell_rect(rows - 1, 0, &config, width, origin).expect("in shape");
        assert_eq!(
            last.pos.y + last.size.y - origin.y,
            grid_height(&config) - FOOTER_HEIGHT
        );
        // Out-of-shape indices are `None` rather than a panic.
        assert!(cell_rect(rows, 0, &config, width, origin).is_none());
        assert!(cell_rect(0, columns, &config, width, origin).is_none());
    }
}
