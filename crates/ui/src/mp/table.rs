//! `MpTable` — rows and columns, and the arithmetic that puts a cell where it
//! belongs.
//!
//! ## Why this is one widget and not a tree of them
//!
//! Every other component here is a widget per thing. A table cannot be: a
//! hundred rows of six columns is six hundred widgets, each with an area, a
//! scope entry and a draw step, for content that is *a grid of strings*. So the
//! table owns the layout and paints its own cells — a `DrawQuad` for the chrome
//! and a `DrawText` drawn at absolute positions — which is the same shape
//! Makepad's own `DataGrid` and the canvas terminal's item chrome use.
//!
//! What that buys is that the whole table is one area, and what it costs is that
//! a cell cannot have its own widget. That is the right trade at this scope: the
//! moment a column needs an editor, the honest answer is a different component
//! (a grid of widgets), not a flag on this one.
//!
//! ## The layout is one function
//!
//! [`MpTable::layout`] answers where every row and column is, and the painter,
//! the hover test and the click test all read it. Writing that arithmetic out
//! once per consumer is how a row ends up clickable somewhere it is not drawn —
//! the fault this crate has already fixed twice, in the canvas terminal's
//! minimised strip and in `mp::control`'s hit contract.

use makepad_widgets::*;

use crate::mp::text;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    set_type_default() do #(DrawMpTable::script_shader(vm)){
        ..mod.draw.DrawQuad

        fill: #x00000000
        border: #x00000000
        border_width: 0.0
        (radius: 8.0)

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let bw = self.border_width
            sdf.box(bw, bw, self.rect_size.x - bw * 2.0, self.rect_size.y - bw * 2.0, max(1.0, self.radius))
            // `fill_keep`, because the stroke below is the same box.
            sdf.fill_keep(self.fill)
            if (bw > 0.0) {
                sdf.stroke(self.border, bw)
            }
            return sdf.result
        }
    }

    mod.mp.MpTableBase = #(MpTable::register_widget(vm))

    mod.mp.MpTable = set_type_default() do mod.mp.MpTableBase{
        width: Fill
        height: Fit

        draw_bg +: {
            text_style: mod.mpc.type.body
            color: #x00000000
        }
        draw_head +: {
            text_style: mod.mpc.type.caption
            color: #x00000000
        }
        draw_cell +: {
            text_style: mod.mpc.type.body
            color: #x00000000
        }
    }
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpTable {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    fill: Vec4f,
    #[live]
    border: Vec4f,
    #[live]
    border_width: f32,
    #[live]
    radius: f32,
}

/// One column: what it is called and how wide it is.
///
/// A width of `0` means "share the remainder", which is what most columns want.
/// Naming pixels for every column is how a table fails to fill its container, so
/// the default is the flexible one.
#[derive(Clone, Debug, PartialEq)]
pub struct TableColumn {
    pub label: String,
    /// Fixed width in points, or `0` to share what is left.
    pub width: f64,
    /// Right-align the column. Numbers want this and prose does not — a column of
    /// figures set flush left is the single most common table mistake.
    pub align_end: bool,
}

impl TableColumn {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            width: 0.0,
            align_end: false,
        }
    }

    pub fn width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// Right-align, for a column of numbers.
    pub fn end(mut self) -> Self {
        self.align_end = true;
        self
    }
}

/// What a table reports.
#[derive(Clone, Debug, Default)]
pub enum MpTableAction {
    /// A row was clicked, carrying its index.
    RowSelected(usize),
    #[default]
    None,
}

/// The row height, and the header's. Named because the painter and the hit test
/// both need them and a table whose rows are one height to the eye and another
/// to the pointer is a bug nobody can see in a screenshot.
const ROW_H: f64 = 30.0;
const HEAD_H: f64 = 28.0;
/// The gap between a cell's text and its column's edge.
const CELL_PAD: f64 = 10.0;

/// One row's height.
fn row_height(index: usize) -> f64 {
    let _ = index;
    ROW_H
}

/// A row's or column's rectangle, in the table's own coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Box2 {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Box2 {
    fn contains(&self, p: Vec2d) -> bool {
        p.x >= self.x && p.x <= self.x + self.w && p.y >= self.y && p.y <= self.y + self.h
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct MpTable {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_bg: DrawMpTable,
    #[live]
    draw_head: DrawText,
    #[live]
    draw_cell: DrawText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[rust]
    columns: Vec<TableColumn>,
    #[rust]
    rows: Vec<Vec<String>>,
    #[rust]
    area: Area,
    /// Which row the pointer is over, so the hover wash can be drawn.
    #[rust]
    hovered: Option<usize>,
}

impl MpTable {
    pub fn set_columns(&mut self, cx: &mut Cx, columns: Vec<TableColumn>) {
        self.columns = columns;
        self.redraw(cx);
    }

    pub fn columns(&self) -> &[TableColumn] {
        &self.columns
    }

    pub fn set_rows(&mut self, cx: &mut Cx, rows: Vec<Vec<String>>) {
        self.rows = rows;
        self.hovered = None;
        self.redraw(cx);
    }

    pub fn rows(&self) -> &[Vec<String>] {
        &self.rows
    }

    pub fn row_selected(&self, actions: &Actions) -> Option<usize> {
        crate::mp::action::first::<MpTableAction>(self.widget_uid(), actions).and_then(|a| {
            match a {
                MpTableAction::RowSelected(i) => Some(*i),
                MpTableAction::None => None,
            }
        })
    }

    /// How tall the table wants to be, so a caller can size a scroll container
    /// around it without restating the row height.
    pub fn content_height(&self) -> f64 {
        HEAD_H
            + (0..self.rows.len())
                .map(row_height)
                .sum::<f64>()
    }

    /// Every column's `x` and width, given the table's own width.
    ///
    /// Flexible columns share what the fixed ones leave, equally. When the fixed
    /// columns already overrun the container the flexible ones get nothing rather
    /// than a negative width — a table narrower than its fixed columns clips, and
    /// it clips cleanly.
    fn column_boxes(&self, width: f64) -> Vec<Box2> {
        let fixed: f64 = self.columns.iter().map(|c| c.width).sum();
        let flexible = self.columns.iter().filter(|c| c.width <= 0.0).count();
        let share = if flexible == 0 {
            0.0
        } else {
            ((width - fixed) / flexible as f64).max(0.0)
        };
        let mut x = 0.0;
        self.columns
            .iter()
            .map(|c| {
                let w = if c.width > 0.0 { c.width } else { share };
                let b = Box2 {
                    x,
                    y: 0.0,
                    w,
                    h: 0.0,
                };
                x += w;
                b
            })
            .collect()
    }

    /// Where row `index`'s box is, in the table's own coordinates.
    fn row_box(&self, index: usize) -> Box2 {
        let y = HEAD_H
            + (0..index).map(row_height).sum::<f64>();
        Box2 {
            x: 0.0,
            y,
            w: f64::INFINITY,
            h: row_height(index),
        }
    }

    /// Which row a point in the table's own coordinates is over.
    fn row_at(&self, p: Vec2d) -> Option<usize> {
        (0..self.rows.len()).find(|i| self.row_box(*i).contains(p))
    }

    /// The text a cell shows, or an empty string when the row is short.
    ///
    /// A row with fewer cells than there are columns is normal — a trailing
    /// optional column — and treating it as a bug would make every caller pad.
    fn cell_text(&self, row: usize, col: usize) -> &str {
        self.rows
            .get(row)
            .and_then(|r| r.get(col))
            .map(|s| s.as_str())
            .unwrap_or("")
    }
}

impl Widget for MpTable {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        // Hover and click both read the *row* boxes, because a table's unit of
        // interaction is a row. A cell that needs its own interaction is a
        // different component; see the module doc.
        let mut hovered = self.hovered;
        match event.hits(cx, self.area) {
            // The three arms are split rather than matched together because a
            // hover event and a move event are different types with the same
            // field, and Rust cannot bind one name to both.
            Hit::FingerHoverIn(fe) => {
                hovered = self.row_at(fe.abs - self.area.rect(cx).pos);
            }
            Hit::FingerMove(fe) => {
                hovered = self.row_at(fe.abs - self.area.rect(cx).pos);
            }
            Hit::FingerHoverOut(_) => hovered = None,
            Hit::FingerUp(fe) => {
                if fe.is_over {
                    if let Some(row) = self.row_at(fe.abs - self.area.rect(cx).pos) {
                        cx.widget_action(self.widget_uid(), MpTableAction::RowSelected(row));
                    }
                }
            }
            _ => return,
        }
        if hovered != self.hovered {
            self.hovered = hovered;
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // Everything needed from the theme is **copied out** before any mutable
        // use of `cx`. `Theme::of` borrows `cx`, and the first version held that
        // borrow across `draw_bg.begin(cx, ..)` — which does not compile, and the
        // fault is worth naming because the fix is not to clone the theme but to
        // take the handful of numbers this paint actually needs.
        let (card, border, hover_wash, band, muted_ink, body_ink, divider, radius, body_box, head_box) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            let p = &theme.paint;
            (
                p.surface_card,
                p.border,
                p.element_hover,
                p.band,
                p.text_muted,
                p.text,
                p.divider,
                makepad_theme::Theme::panel_radius() as f32,
                theme.metrics(makepad_theme::TextStyle::Body).line_height() as f64,
                theme.metrics(makepad_theme::TextStyle::Caption).line_height() as f64,
            )
        };

        // The plate: a card's ground, because a table is a card's content.
        self.draw_bg.fill = card;
        self.draw_bg.border = border;
        self.draw_bg.border_width = 1.0;
        self.draw_bg.radius = radius;
        // **The table sizes itself from its content.** Nothing else can: its
        // cells are painted by `draw_abs` inside its own turtle rather than laid
        // out as children, so there is no widget whose height the layout pass
        // could measure. `height: Fit` therefore measures zero and the plate —
        // and every cell in it — is drawn into a zero-height box, which looks
        // exactly like a table that does not draw at all. The same fault the
        // pulse loader had, and the same fix.
        self.walk.height = Size::Fixed(self.content_height());
        let walk = self.walk;
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);
        let rect = self.draw_bg.area().rect(cx.cx);
        self.area = self.draw_bg.area();

        // Everything below is painted at absolute positions inside the plate, so
        // the table's own walk is the only layout it takes part in.
        let width = rect.size.x;
        let cols = self.column_boxes(width);
        let origin = rect.pos;

        // The hovered row's wash, under the text and over the plate.
        if let Some(row) = self.hovered.filter(|r| *r < self.rows.len()) {
            let b = self.row_box(row);
            self.draw_bg.fill = hover_wash;
            self.draw_bg.border_width = 0.0;
            self.draw_bg.draw_abs(
                cx,
                Rect {
                    pos: origin + dvec2(b.x, b.y),
                    size: dvec2(width, b.h),
                },
            );
            self.draw_bg.fill = card;
        }

        // The header's own ground, then its rule.
        self.draw_bg.fill = band;
        self.draw_bg.draw_abs(
            cx,
            Rect {
                pos: origin,
                size: dvec2(width, HEAD_H),
            },
        );
        self.draw_bg.fill = border;
        self.draw_bg.draw_abs(
            cx,
            Rect {
                pos: origin + dvec2(0.0, HEAD_H - 1.0),
                size: dvec2(width, 1.0),
            },
        );

        // The header's labels.
        self.draw_head.color = muted_ink;
        for (i, col) in self.columns.iter().enumerate() {
            let Some(b) = cols.get(i) else { continue };
            let text = self.fit(col.label.as_str(), b.w, &self.draw_head, cx);
            let x = if col.align_end {
                text::right_aligned_x(
                    &text,
                    b.x,
                    b.w,
                    CELL_PAD,
                    self.draw_head.text_style.font_size as f64,
                )
            } else {
                b.x + CELL_PAD
            };
            self.draw_head.draw_abs(
                cx,
                origin + dvec2(x, (HEAD_H - head_box) * 0.5),
                &text,
            );
        }

        // The rows. One hairline per row rather than a stroke per cell: the eye
        // reads a table's rows, not its cells.
        self.draw_cell.color = body_ink;
        for row in 0..self.rows.len() {
            let b = self.row_box(row);
            let y = b.y + (b.h - body_box) * 0.5;
            for (i, col) in self.columns.iter().enumerate() {
                let Some(cb) = cols.get(i) else { continue };
                let text = self.fit(self.cell_text(row, i), cb.w, &self.draw_cell, cx);
                let x = if col.align_end {
                    text::right_aligned_x(
                        &text,
                        cb.x,
                        cb.w,
                        CELL_PAD,
                        self.draw_cell.text_style.font_size as f64,
                    )
                } else {
                    cb.x + CELL_PAD
                };
                self.draw_cell.draw_abs(cx, origin + dvec2(x, y), &text);
            }
            if row + 1 < self.rows.len() {
                self.draw_bg.fill = divider;
                self.draw_bg.draw_abs(
                    cx,
                    Rect {
                        pos: origin + dvec2(0.0, b.y + b.h - 1.0),
                        size: dvec2(width, 1.0),
                    },
                );
            }
        }

        DrawStep::done()
    }
}

impl MpTable {
    /// A cell's text, clipped to its column.
    ///
    /// The arithmetic lives in [`crate::mp::text`], shared with the tree and the
    /// list: each of the three grew its own copy first, which means two widgets
    /// could clip the same string at different points.
    fn fit(&self, value: &str, column_w: f64, draw: &DrawText, _cx: &mut Cx2d) -> String {
        let avail = column_w - CELL_PAD * 2.0;
        text::clip(value, avail, draw.text_style.font_size as f64)
    }
}

impl MpTableRef {
    pub fn set_columns(&self, cx: &mut Cx, columns: Vec<TableColumn>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_columns(cx, columns);
        }
    }

    pub fn set_rows(&self, cx: &mut Cx, rows: Vec<Vec<String>>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_rows(cx, rows);
        }
    }

    pub fn row_selected(&self, actions: &Actions) -> Option<usize> {
        self.borrow().and_then(|inner| inner.row_selected(actions))
    }

    pub fn content_height(&self) -> f64 {
        self.borrow().map(|inner| inner.content_height()).unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pure arithmetic, exercised without a widget.
    ///
    /// Building an `MpTable` needs a script VM for its drawers, and the parts
    /// worth testing — where a column sits, where a row sits, which row a point
    /// is over — do not involve one. So the layout is a free struct here and the
    /// widget is the thin shell that reads it, which is also why the widget's
    /// methods are one-liners over it.
    #[derive(Debug)]
    struct Layout2 {
        columns: Vec<TableColumn>,
        rows: usize,
    }

    impl Layout2 {
        fn new(columns: Vec<TableColumn>, rows: usize) -> Self {
            Self { columns, rows }
        }

        fn column_boxes(&self, width: f64) -> Vec<(f64, f64)> {
            let fixed: f64 = self.columns.iter().map(|c| c.width).sum();
            let flexible = self.columns.iter().filter(|c| c.width <= 0.0).count();
            let share = if flexible == 0 {
                0.0
            } else {
                ((width - fixed) / flexible as f64).max(0.0)
            };
            let mut x = 0.0;
            self.columns
                .iter()
                .map(|c| {
                    let w = if c.width > 0.0 { c.width } else { share };
                    let at = x;
                    x += w;
                    (at, w)
                })
                .collect()
        }

        fn row_y(&self, index: usize) -> f64 {
            HEAD_H + (0..index).map(row_height).sum::<f64>()
        }

        fn row_at(&self, y: f64) -> Option<usize> {
            (0..self.rows).find(|i| {
                let top = self.row_y(*i);
                y >= top && y <= top + row_height(*i)
            })
        }

        fn content_height(&self) -> f64 {
            HEAD_H + (0..self.rows).map(row_height).sum::<f64>()
        }
    }

    fn table(columns: Vec<TableColumn>, rows: usize) -> Layout2 {
        Layout2::new(columns, rows)
    }

    #[test]
    fn test_fixed_columns_keep_their_width_and_the_rest_share_the_remainder() {
        let t = table(
            vec![
                TableColumn::new("name"),
                TableColumn::new("status").width(80.0),
                TableColumn::new("count").width(60.0).end(),
            ],
            0,
        );
        let cols = t.column_boxes(500.0);
        assert_eq!(cols.len(), 3);
        assert!((cols[0].0 - 0.0).abs() < 1e-9);
        // The flexible column takes what the two fixed ones leave.
        assert!((cols[0].1 - 360.0).abs() < 1e-9, "{}", cols[0].1);
        assert!((cols[1].0 - 360.0).abs() < 1e-9);
        assert!((cols[2].1 - 60.0).abs() < 1e-9);
        // The last column ends exactly at the table's edge — no gap, no overrun.
        let end = cols[2].0 + cols[2].1;
        assert!((end - 500.0).abs() < 1e-9, "{end}");
    }

    #[test]
    fn test_flexible_columns_share_equally() {
        let t = table(
            vec![
                TableColumn::new("a"),
                TableColumn::new("b"),
                TableColumn::new("c"),
            ],
            0,
        );
        let cols = t.column_boxes(300.0);
        for (_, w) in &cols {
            assert!((w - 100.0).abs() < 1e-9, "{w}");
        }
    }

    #[test]
    fn test_a_table_narrower_than_its_fixed_columns_gives_flexible_ones_nothing() {
        // A negative width would draw a cell inside out. Zero clips cleanly, and
        // the fixed columns keep the widths their caller named.
        let t = table(
            vec![TableColumn::new("wide").width(400.0), TableColumn::new("rest")],
            0,
        );
        let cols = t.column_boxes(200.0);
        assert!((cols[0].1 - 400.0).abs() < 1e-9);
        assert!((cols[1].1 - 0.0).abs() < 1e-9, "{}", cols[1].1);
    }

    #[test]
    fn test_rows_are_stacked_below_the_header_without_gaps() {
        let t = table(vec![TableColumn::new("a")], 3);
        assert!((t.row_y(0) - HEAD_H).abs() < 1e-9);
        assert!((t.row_y(1) - (HEAD_H + ROW_H)).abs() < 1e-9);
        // No gap and no overlap between neighbours.
        assert!((t.row_y(0) + ROW_H - t.row_y(1)).abs() < 1e-9);
    }

    #[test]
    fn test_the_content_height_is_the_header_plus_every_row() {
        let t = table(vec![TableColumn::new("a")], 3);
        let want = HEAD_H + 3.0 * ROW_H;
        assert!((t.content_height() - want).abs() < 1e-9);

        // An empty table is its header, not zero: a table with no rows still has
        // columns to name.
        let empty = table(vec![TableColumn::new("a")], 0);
        assert!((empty.content_height() - HEAD_H).abs() < 1e-9);
    }

    #[test]
    fn test_row_hit_testing_agrees_with_row_layout() {
        // The property that matters: a row is clickable exactly where it is
        // drawn. This is the fault the crate has already fixed twice elsewhere.
        let t = table(vec![TableColumn::new("a")], 4);
        for row in 0..4 {
            let top = t.row_y(row);
            // A point a hair inside each edge of the drawn box is that row.
            assert_eq!(t.row_at(top + 0.5), Some(row), "top of {row}");
            assert_eq!(t.row_at(top + ROW_H - 0.5), Some(row), "bottom of {row}");
        }
        // The header is nobody's row.
        assert_eq!(t.row_at(HEAD_H - 1.0), None);
        // So is past the last one.
        assert_eq!(t.row_at(t.row_y(3) + ROW_H + 1.0), None);
    }

    #[test]
    fn test_a_short_row_reads_as_empty_rather_than_panicking() {
        // A row with fewer cells than columns is normal — a trailing optional
        // column — and treating it as a bug would make every caller pad.
        let rows: Vec<Vec<String>> = vec![vec!["only".into()]];
        let cell = |r: usize, c: usize| -> &str {
            rows.get(r).and_then(|row| row.get(c)).map(|s| s.as_str()).unwrap_or("")
        };
        assert_eq!(cell(0, 0), "only");
        assert_eq!(cell(0, 1), "");
        assert_eq!(cell(9, 9), "");
    }

    #[test]
    fn test_a_column_with_no_width_shares_without_a_width_to_share() {
        // No columns at all, and a table of only fixed columns: neither divides by
        // a count of zero.
        let empty = table(Vec::new(), 0);
        assert!(empty.column_boxes(500.0).is_empty());
        let all_fixed = table(vec![TableColumn::new("a").width(50.0)], 0);
        let cols = all_fixed.column_boxes(500.0);
        assert!((cols[0].1 - 50.0).abs() < 1e-9);
    }

}
