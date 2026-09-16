//! `MpColorPicker` — a grid of swatches you pick from.
//!
//! ## The colours are the caller's, and that is not a theme violation
//!
//! Every other component in this library reads its colours from `Theme::of(cx)`, and this one takes them from the caller.
//! The difference is **what the colours are**: a palette is *data* — the list of things a user may choose between —
//! whereas a plate colour is a *style*. A picker that drew its swatches from the theme would be a picker offering the
//! theme's own colours to choose from, which is a different component. So the swatches are the caller's and the **ring,
//! the border and the radius are the theme's**, and the two are kept apart on purpose: a swatch's colour may be anything,
//! while how a swatch is *drawn* is the library's.
//!
//! The same split as the A2UI `Text`'s `font_size` (the protocol's data) versus the face it is painted in (the theme's).
//!
//! ## One draw, many cells
//!
//! The grid is painted by **one** shader instance reused per swatch, which is the pattern a table's cells use: the draw
//! carries the geometry and the per-cell values, so a twenty-swatch grid is twenty draws of one shader rather than twenty
//! widget instances. That is only possible because the cells share every property except their colour and their two
//! flags — which is also why [`cell_index`] can be pure arithmetic instead of a search through hit areas.
//!
//! ## The hit test is the interesting logic, so it is the tested logic
//!
//! A point maps to a cell or to **nothing**: the gaps between cells are not swatches, and neither is the space past the
//! last colour or past the last column of a partial row. A picker that rounded a click in a gap to the nearest swatch
//! would pick a colour somebody narrowly missed — and the four cases (in a cell, in a gap, past the end, past the last
//! column) are all `None`-able, so they are functions with tests rather than arithmetic inside a draw call.
//!
//! ## What was dropped
//!
//! The v2 five-step `MpSize`. The cell is [`SWATCH`], a **component constant**, because the theme has no swatch metric and
//! a caller choosing one is choosing a radius, a gap and a ring thickness without saying so — the kind of knob this
//! port's rules forbid. If the theme ever grows a swatch metric this takes it and nothing else changes.

use makepad_widgets::*;

use crate::mp::control;

/// A swatch's side, in points, and the gap between swatches.
///
/// Constants rather than `#[live]` fields: see the module doc on why a caller does not choose these. The gap is the v2
/// widget's own proportion — a fifth of the cell, rounded — kept because it looked right and changing it while porting
/// would be a change nobody asked for.
pub const SWATCH: f64 = 22.0;
pub const GAP: f64 = 5.0;

/// How many swatches per row when the caller has not said.
pub const DEFAULT_COLUMNS: usize = 8;

/// The columns a grid uses, with **zero meaning "you choose"** rather than "one".
///
/// **And this is where porting changed the behaviour without my noticing.** The v2 widget mapped zero to eight — a
/// sensible grid — and I wrote `.max(1)`, reading "unset" as "the smallest thing that is a grid". The v2's reading was
/// right: a caller that does not choose wants a *grid*, and one column is not a grid, it is a list. The A2UI renderer is
/// what found it, because the protocol has no `columns` field at all, so **every colour picker it drew came out as a
/// single column of nine** — visible in the run as `columns=1` where the default should be in force.
///
/// A caller that genuinely wants one column says so, and one is what it gets.
pub fn columns_clamped(columns: usize) -> usize {
    if columns == 0 {
        DEFAULT_COLUMNS
    } else {
        columns
    }
}

/// How many rows a grid of `count` swatches occupies.
pub fn rows(count: usize, columns: usize) -> usize {
    let columns = columns_clamped(columns);
    count.div_ceil(columns)
}

/// The grid's width: `columns` cells and the gaps between them.
///
/// `columns - 1` gaps rather than `columns`, because a gap after the last column is a gap at the edge of the grid — which
/// is why a right-aligned grid with trailing gaps looks inset from its own container.
pub fn grid_width(columns: usize) -> f64 {
    let columns = columns_clamped(columns);
    columns as f64 * SWATCH + (columns - 1) as f64 * GAP
}

/// The grid's height for `count` swatches.
pub fn grid_height(count: usize, columns: usize) -> f64 {
    let rows = rows(count, columns);
    if rows == 0 {
        return 0.0;
    }
    rows as f64 * SWATCH + (rows - 1) as f64 * GAP
}

/// The swatch a point falls in, or `None`.
///
/// `(x, y)` is relative to the grid's top-left. Four ways to miss, and all four are `None`:
///
/// 1. **in a gap** — the space between two cells is not a cell, so a click that narrowly misses picks nothing rather than
///    the nearest swatch;
/// 2. **past the last column** of a partial row;
/// 3. **past the last swatch** — the empty cells of an incomplete final row;
/// 4. **outside the grid** — a negative coordinate, or one past the far edge.
///
/// A `NaN` or an infinity is also `None`: `NaN` compares false against everything, so a version written with `<` alone
/// would fall through to a cell rather than rejecting the point. That is the failure mode this port has already paid for
/// twice — a `NaN` that propagates into an index — so the finiteness check is explicit.
pub fn cell_index(x: f64, y: f64, count: usize, columns: usize) -> Option<usize> {
    if !x.is_finite() || !y.is_finite() || x < 0.0 || y < 0.0 {
        return None;
    }
    let columns = columns_clamped(columns);
    let stride = SWATCH + GAP;
    let column = (x / stride).floor() as usize;
    let row = (y / stride).floor() as usize;
    // In a gap rather than in a cell: the remainder past a cell's own side is the gap.
    if x - column as f64 * stride >= SWATCH || y - row as f64 * stride >= SWATCH {
        return None;
    }
    if column >= columns {
        return None;
    }
    let index = row * columns + column;
    (index < count).then_some(index)
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    /// One swatch. `bg` is the caller's colour; everything else is the theme's, written from Rust each paint.
    mod.mp.DrawMpSwatch = #(DrawMpSwatch::script_shader(vm)){
        ..mod.draw.DrawQuad

        bg: #xffffffff
        selected: 0.0
        hovered: 0.0
        radius: 5.0
        border_color: #x00000000
        ring_color: #xffffffff

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let sz = self.rect_size
            sdf.box(0.5, 0.5, sz.x - 1.0, sz.y - 1.0, self.radius)
            sdf.fill_keep(self.bg)
            // A quiet hairline, brightening under the pointer.
            sdf.stroke(mix(self.border_color, self.ring_color, self.hovered * 0.6), 1.0)
            // An inner ring marks the selection, so it cannot be confused with the hairline of a hovered neighbour.
            if (self.selected > 0.5) {
                sdf.box(2.5, 2.5, sz.x - 5.0, sz.y - 5.0, max(self.radius - 2.0, 1.0))
                sdf.stroke(self.ring_color, 2.0)
            }
            return sdf.result
        }
    }

    mod.mp.MpColorPickerBase = #(MpColorPicker::register_widget(vm))

    mod.mp.MpColorPicker = set_type_default() do mod.mp.MpColorPickerBase{
        width: Fit
        height: Fit
    }
}

/// A swatch grid.
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpSwatch {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    bg: Vec4f,
    #[live]
    selected: f32,
    #[live]
    hovered: f32,
    #[live]
    radius: f32,
    #[live]
    border_color: Vec4f,
    #[live]
    ring_color: Vec4f,
}

/// What a picker reports.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum MpColorPickerAction {
    /// A swatch was picked, carrying its colour and its index.
    ///
    /// The **index as well as the colour**, which the v2 action did not carry: two swatches may hold the same colour, and
    /// a caller that wants to mark the chosen one needs to know which it was. A colour alone is ambiguous exactly when
    /// a palette repeats itself, which palettes do.
    Picked(Vec4f, usize),
    #[default]
    None,
}

/// A grid of swatches you pick from.
#[derive(Script, ScriptHook, Widget, Animator)]
pub struct MpColorPicker {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    /// The animator the control signals need. A picker has no animation of its own, and the field is here because
    /// `control::handle` is what turns a pointer into `down`/`moved`/`hover_out` — the alternative is re-implementing the
    /// hit test this port already has, in a component that would then have two of them.
    #[apply_default]
    animator: Animator,
    #[redraw]
    #[live]
    draw_swatch: DrawMpSwatch,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    /// The colours, which are the caller's. See the module doc.
    #[rust]
    colors: Vec<Vec4f>,
    #[rust]
    columns: usize,
    #[rust]
    selected: Option<usize>,
    #[rust]
    hovered: Option<usize>,
    /// The grid's top-left, captured during the last paint, so a pointer position can be made relative to it.
    #[rust]
    origin: DVec2,
    #[rust]
    area: Area,
}

impl Widget for MpColorPicker {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let signals = control::handle(&mut self.animator, cx, event, self.area);
        // Tell whoever owns the one tooltip that this control is under the pointer; a control cannot own one itself. See
        // `mp/tooltip.rs`.
        control::emit_hover(cx, self.widget_uid(), signals);
        if signals.redraw {
            self.redraw(cx);
        }

        // **The press, from the control signals** — a gesture that carries a position, so the hit test and the value
        // come from the same place.
        if signals.down {
            let index = self.cell_at(
                signals.pointer.x - self.origin.x,
                signals.pointer.y - self.origin.y,
            );
            if let Some(index) = index {
                self.selected = Some(index);
                self.redraw(cx);
                cx.widget_action(
                    self.widget_uid(),
                    MpColorPickerAction::Picked(self.colors[index], index),
                );
            }
            return;
        }

        // **The hover, from a mouse move** — the only event that carries a position without a gesture. A finger does not
        // hover, so there is nothing here for touch to miss, and reading `MouseMove` is reading the platform's own hover
        // rather than inventing a second one from the drag signal. (The first version of this looked for
        // `Event::FingerHover` and `Event::FingerDown`, **which do not exist**: makepad reports a pointer as a `Hit`
        // against an area, not as an event variant. This port's own trap list says so, and I had written it.)
        if signals.hover_out {
            if self.hovered.is_some() {
                self.hovered = None;
                self.redraw(cx);
            }
            return;
        }
        if let Event::MouseMove(me) = event {
            let index = self.cell_at(me.abs.x - self.origin.x, me.abs.y - self.origin.y);
            if self.hovered != index {
                self.hovered = index;
                self.redraw(cx);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let (ring, border) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            (theme.paint.text, theme.paint.border)
        };
        self.draw_swatch.radius = (SWATCH * 0.22) as f32;
        self.draw_swatch.ring_color = ring;
        self.draw_swatch.border_color = border;

        // The grid's own size, declared because a self-drawing widget measures nothing on its own — the same rule the
        // step indicator's dot follows.
        let width = grid_width(self.columns);
        let height = grid_height(self.colors.len(), self.columns);
        let placed = cx.walk_turtle(Walk {
            width: Size::Fixed(width.max(self.layout.padding.left as f64 + width)),
            height: Size::Fixed(height),
            ..walk
        });
        self.origin = placed.pos;
        self.area = self.draw_swatch.area();

        let stride = SWATCH + GAP;
        for (index, color) in self.colors.iter().enumerate() {
            let column = (index % columns_clamped(self.columns)) as f64;
            let row = (index / columns_clamped(self.columns)) as f64;
            self.draw_swatch.bg = *color;
            self.draw_swatch.selected = if self.selected == Some(index) { 1.0 } else { 0.0 };
            self.draw_swatch.hovered = if self.hovered == Some(index) { 1.0 } else { 0.0 };
            self.draw_swatch.draw_abs(
                cx,
                Rect {
                    pos: self.origin + dvec2(column * stride, row * stride),
                    size: dvec2(SWATCH, SWATCH),
                },
            );
        }
        DrawStep::done()
    }
}

impl MpColorPicker {
    /// Set the palette.
    pub fn set_colors(&mut self, cx: &mut Cx, colors: Vec<Vec4f>) {
        let count = colors.len();
        self.colors = colors;
        // A selection past the end of a shorter palette is not a selection — checked here rather than at each read, so
        // there is no state in which the selected index names nothing.
        if self.selected.is_some_and(|index| index >= count) {
            self.selected = None;
        }
        if self.hovered.is_some_and(|index| index >= count) {
            self.hovered = None;
        }
        self.redraw(cx);
    }

    /// The palette.
    pub fn colors(&self) -> &[Vec4f] {
        &self.colors
    }

    /// Set how many swatches fit in a row.
    pub fn set_columns(&mut self, cx: &mut Cx, columns: usize) {
        self.columns = columns_clamped(columns);
        self.redraw(cx);
    }

    /// The columns in force, which is at least one.
    pub fn columns(&self) -> usize {
        columns_clamped(self.columns)
    }

    /// Mark a swatch selected.
    pub fn set_selected(&mut self, cx: &mut Cx, index: Option<usize>) {
        self.selected = index.filter(|index| *index < self.colors.len());
        self.redraw(cx);
    }

    /// The selected swatch's index.
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// The selected colour.
    pub fn picked_color(&self) -> Option<Vec4f> {
        self.selected.and_then(|index| self.colors.get(index).copied())
    }

    /// Which swatch a point picks, for a caller driving the widget without a pointer.
    pub fn cell_at(&self, x: f64, y: f64) -> Option<usize> {
        cell_index(x, y, self.colors.len(), self.columns)
    }
}

impl MpColorPickerRef {
    pub fn set_colors(&self, cx: &mut Cx, colors: Vec<Vec4f>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_colors(cx, colors);
        }
    }

    pub fn set_columns(&self, cx: &mut Cx, columns: usize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_columns(cx, columns);
        }
    }

    pub fn set_selected(&self, cx: &mut Cx, index: Option<usize>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_selected(cx, index);
        }
    }

    pub fn picked_color(&self) -> Option<Vec4f> {
        self.borrow().and_then(|inner| inner.picked_color())
    }

    pub fn colors(&self) -> Vec<Vec4f> {
        self.borrow().map(|inner| inner.colors.clone()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_full_row_is_columns_cells_wide_and_has_no_trailing_gap() {
        // **`columns - 1` gaps**, not `columns`: a gap after the last cell is a gap at the edge of the grid, which is why
        // a grid laid out with trailing gaps looks inset from its own container.
        assert_eq!(grid_width(1), SWATCH);
        assert_eq!(grid_width(4), SWATCH * 4.0 + GAP * 3.0);
        // Zero means "you choose", so the default is in force rather than a division by zero.
        assert_eq!(grid_width(0), grid_width(DEFAULT_COLUMNS));
    }

    #[test]
    fn test_the_number_of_rows_rounds_up_so_a_partial_row_is_a_row() {
        // `div_ceil` rather than a division: a palette of 9 in rows of 4 is three rows, and the third holds one swatch.
        // Truncating would leave the last swatch unpainted.
        assert_eq!(rows(0, 4), 0);
        assert_eq!(rows(1, 4), 1);
        assert_eq!(rows(4, 4), 1);
        assert_eq!(rows(5, 4), 2);
        assert_eq!(rows(9, 4), 3);
        assert_eq!(rows(9, 0), rows(9, DEFAULT_COLUMNS), "zero columns used the default");
        // ...and a caller that asks for one column gets one, which is a list rather than a grid — its choice to make.
        assert_eq!(rows(9, 1), 9);
        assert_eq!(grid_width(1), SWATCH);
    }

    #[test]
    fn test_the_grid_height_counts_only_the_rows_it_has() {
        assert_eq!(grid_height(0, 4), 0.0, "an empty palette has no height");
        assert_eq!(grid_height(1, 4), SWATCH);
        assert_eq!(grid_height(4, 4), SWATCH);
        assert_eq!(grid_height(5, 4), SWATCH * 2.0 + GAP);
    }

    #[test]
    fn test_a_point_inside_a_cell_picks_that_cell() {
        // Row-major: the index is `row * columns + column`, which is what makes the grid's order the palette's order.
        let columns = 4;
        assert_eq!(cell_index(0.0, 0.0, 8, columns), Some(0));
        assert_eq!(cell_index(SWATCH - 0.1, SWATCH - 0.1, 8, columns), Some(0));
        assert_eq!(cell_index(SWATCH + GAP, 0.0, 8, columns), Some(1));
        assert_eq!(cell_index(0.0, SWATCH + GAP, 8, columns), Some(4));
        assert_eq!(cell_index(SWATCH + GAP, SWATCH + GAP, 8, columns), Some(5));
    }

    #[test]
    fn test_a_point_in_a_gap_picks_nothing_rather_than_the_nearest_swatch() {
        // **The rule that makes this a picker rather than a guessing game.** A click that narrowly misses a swatch picks
        // nothing — rounding to the nearest cell would pick a colour somebody did not choose, and the gap is the only
        // place a person can aim to miss.
        let columns = 4;
        let in_gap_x = SWATCH + GAP * 0.5;
        assert_eq!(cell_index(in_gap_x, 0.0, 8, columns), None);
        let in_gap_y = SWATCH + GAP * 0.5;
        assert_eq!(cell_index(0.0, in_gap_y, 8, columns), None);
        // ...and the first point of the next cell is a cell again, so the gap's far edge is not also nothing.
        assert!(cell_index(SWATCH + GAP, 0.0, 8, columns).is_some());
    }

    #[test]
    fn test_a_point_past_the_last_swatch_picks_nothing() {
        // The empty cells of an incomplete final row. A palette of 6 in rows of 4 has two holes on the second row, and a
        // click in either must not report swatch 6 or 7 — there are none.
        let columns = 4;
        let stride = SWATCH + GAP;
        assert_eq!(cell_index(stride * 2.0, 0.0, 6, columns), Some(2));
        assert_eq!(cell_index(stride * 3.0, 0.0, 6, columns), Some(3));
        assert_eq!(cell_index(stride * 0.0, stride, 6, columns), Some(4));
        assert_eq!(cell_index(stride * 1.0, stride, 6, columns), Some(5));
        assert_eq!(cell_index(stride * 2.0, stride, 6, columns), None, "a hole was a swatch");
        assert_eq!(cell_index(stride * 3.0, stride, 6, columns), None, "a hole was a swatch");
    }

    #[test]
    fn test_a_point_past_the_last_column_or_outside_the_grid_picks_nothing() {
        // A caller's columns bound the grid: with two columns, the fifth swatch is on the **third row**, and the point
        // where a third column would be on the first row picks **nothing** — the row wrapped rather than ran on.
        let stride = SWATCH + GAP;
        assert_eq!(cell_index(stride, 0.0, 6, 2), Some(1));
        assert_eq!(
            cell_index(stride * 2.0, 0.0, 6, 2),
            None,
            "a point past the last column of a row picked something"
        );
        // **The point where a third column would be is the same point as the first column of the next row** — which is
        // exactly why the column bound has to be checked before the index is computed. My first version of this test
        // asserted the two were equal *and* that the first was `Some(2)`, which cannot both be true, and its own comment
        // said so.
        assert_ne!(
            cell_index(stride * 2.0, 0.0, 6, 2),
            cell_index(0.0, stride, 6, 2)
        );
        assert_eq!(cell_index(0.0, stride, 6, 2), Some(2));
        // Negative and far coordinates.
        assert_eq!(cell_index(-1.0, 0.0, 6, 2), None);
        assert_eq!(cell_index(0.0, -1.0, 6, 2), None);
        assert_eq!(cell_index(9_000.0, 9_000.0, 6, 2), None);
    }

    #[test]
    fn test_a_non_finite_point_picks_nothing() {
        // **The case a version written with `<` alone would get wrong**: every comparison against `NaN` is false, so a
        // `NaN` would fall through the bounds checks and land on a cell. This port has paid for a propagating `NaN`
        // twice already — in the slider and in the number input — so the finiteness check is explicit and tested.
        assert_eq!(cell_index(f64::NAN, 0.0, 6, 2), None);
        assert_eq!(cell_index(0.0, f64::NAN, 6, 2), None);
        assert_eq!(cell_index(f64::INFINITY, 0.0, 6, 2), None);
        assert_eq!(cell_index(0.0, f64::NEG_INFINITY, 6, 2), None);
    }

    #[test]
    fn test_every_swatch_is_reachable_and_the_reachable_set_is_exactly_the_palette() {
        // **The property that matters**: for any palette size and column count, every swatch has a point that picks it and
        // **no point picks anything else**. Walked by aiming at each cell's centre, which is where a person clicks.
        let stride = SWATCH + GAP;
        for count in 0..12usize {
            for columns in 1..6usize {
                let mut picked = Vec::new();
                for index in 0..count {
                    let column = index % columns;
                    let row = index / columns;
                    let x = column as f64 * stride + SWATCH * 0.5;
                    let y = row as f64 * stride + SWATCH * 0.5;
                    let hit = cell_index(x, y, count, columns);
                    assert_eq!(hit, Some(index), "count={count} columns={columns} index={index}");
                    picked.push(hit);
                }
                assert_eq!(picked.len(), count, "a swatch was unreachable");
                // And the grid's own bounds do not pick: one stride past the last column of the first row is **outside
                // the grid**, whatever the palette holds — because the row wrapped at `columns` rather than running on.
                // My first version expected `Some(columns)` when `columns < count`, which is the same point as the next
                // row's first cell and therefore the same mistake in a second place.
                assert_eq!(
                    cell_index(columns as f64 * stride, 0.0, count, columns),
                    None,
                    "count={count} columns={columns}: the point past the row's last column"
                );
            }
        }
    }

    #[test]
    fn test_the_default_is_a_grid_rather_than_a_column() {
        // **The defect the A2UI run found.** `columns` is not in the protocol, so a renderer-built picker always has the
        // default — and the default has to be a grid, because a picker showing nine swatches in one column is a list. This
        // is asserted as a *shape* rather than as the number eight: what matters is that the default wraps.
        assert!(DEFAULT_COLUMNS > 1, "the default does not wrap");
        assert_eq!(rows(DEFAULT_COLUMNS * 2, 0), 2, "the default did not wrap two rows");
        assert_eq!(rows(DEFAULT_COLUMNS, 0), 1);
        assert_eq!(rows(DEFAULT_COLUMNS + 1, 0), 2, "a partial row is a row");
        // One column is still available to a caller that wants it.
        assert_eq!(columns_clamped(0), DEFAULT_COLUMNS);
        assert_eq!(columns_clamped(1), 1);
    }

    #[test]
    fn test_a_selection_outside_a_shorter_palette_is_dropped() {
        // Checked at the door rather than at each read, so there is no state where the selected index names nothing.
        let kept = |index: Option<usize>, count: usize| index.filter(|index| *index < count);
        assert_eq!(kept(Some(3), 8), Some(3));
        assert_eq!(kept(Some(8), 8), None, "a selection past the end survived");
        assert_eq!(kept(Some(0), 0), None, "an empty palette kept a selection");
        assert_eq!(kept(None, 8), None);
    }
}
