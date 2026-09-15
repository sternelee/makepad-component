// DbPro — editable data-grid host.
//
// Wraps makepad's `DataGrid` (which owns scrolling, selection, header
// clicks, column resize and keyboard navigation) and feeds it the current
// table page. The App pushes a `GridShared` snapshot in; the host draws
// cells and hosts the inline `Editor` (a TextInput template declared in the
// DSL) for cell editing.
//
// Draw contract (same as makepad's mpsheets app): the host's `draw_walk`
// repeatedly steps the child view; whenever the DataGrid yields a step, the
// host drives `next_cell` and paints values — or the live editor widget for
// the cell being edited.
use std::sync::{Arc, Mutex};

use makepad_widgets::*;

/// Snapshot of the grid contents the App pushes before each redraw.
#[derive(Clone, Debug, Default)]
pub struct GridShared {
    pub col_labels: Vec<String>,
    /// Right-align flags per column (numeric columns).
    pub numeric_cols: Vec<bool>,
    pub rows: Vec<Vec<String>>,
    /// (data column, ascending)
    pub sort: Option<(usize, bool)>,
    /// Cell currently being edited.
    pub editing: Option<(usize, usize)>,
    /// Seed text for the editor, consumed on the next draw of that cell.
    pub edit_seed: Option<String>,
    /// FK dropdown: display labels (parallel to values) — non-empty means
    /// the editing cell renders an MpDropdown instead of a text editor.
    pub fk_labels: Vec<String>,
    /// FK dropdown: the raw values committed on selection.
    pub fk_values: Vec<String>,
    /// False when the table has no primary key (views / PK-less tables).
    pub editable: bool,
    /// Bumped on every push; resets selection + re-applies labels.
    pub version: u64,
}

/// NULL-cell text color (matches db_theme.text_faint).
const COLOR_NULL: f32 = 0x5a as f32 / 255.0;

/// Display formatting for numeric cells: trim trailing zeros and round
/// runaway float precision (>4 decimals) for readable grids. Raw values are
/// preserved for editing/export.
fn fmt_numeric_display(s: &str) -> String {
    let Ok(v) = s.parse::<f64>() else {
        return s.to_string();
    };
    if !s.contains('.') {
        return s.to_string(); // integers stay verbatim
    }
    let decimals = s.split('.').next_back().map(|d| d.len()).unwrap_or(0);
    let mut t = if decimals > 4 {
        format!("{:.4}", v)
    } else {
        s.to_string()
    };
    while t.ends_with('0') {
        t.pop();
    }
    if t.ends_with('.') {
        t.pop();
    }
    t
}

#[derive(Script, ScriptHook, Widget)]
pub struct DbGridHost {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    #[rust]
    shared: Arc<Mutex<GridShared>>,

    #[rust]
    last_version: u64,

    #[rust]
    applied_sort: Option<(usize, bool)>,

    #[rust]
    copy_provider_set: bool,

    /// Content-based column widths (computed when the data version changes).
    #[rust]
    content_widths: Vec<f64>,

    #[rust]
    last_host_width: f64,
}

impl DbGridHost {
    /// Copy a snapshot into the host's shared model. Call once at startup,
    /// before the first draw. Content is copied (not the Arc) so closures
    /// capturing `self.shared` stay valid.
    pub fn set_shared(&mut self, shared: Arc<Mutex<GridShared>>) {
        if let (Ok(mut dst), Ok(src)) = (self.shared.lock(), shared.lock()) {
            *dst = src.clone();
        }
    }
}

/// Apply column widths: content width plus an even share of any leftover
/// pane width, so the grid fills the pane edge to edge.
fn apply_col_widths(content_widths: &[f64], g: &mut DataGrid, host_width: f64) {
    let avail = (host_width - 44.0).max(0.0); // minus row header
    if content_widths.is_empty() {
        return;
    }
    let total: f64 = content_widths.iter().sum();
    if avail > total {
        let extra = (avail - total) / content_widths.len() as f64;
        for (i, w) in content_widths.iter().enumerate() {
            g.set_col_width(i, w + extra);
        }
    } else {
        for (i, w) in content_widths.iter().enumerate() {
            g.set_col_width(i, *w);
        }
    }
}

impl Widget for DbGridHost {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let shared = self.shared.clone();
        let Ok(mut guard) = shared.lock() else {
            return self.view.draw_walk(cx, scope, walk);
        };
        // parent turtle width = the pane width available to the grid
        let host_width = cx.turtle().rect().size.x;

        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            let grid = step.as_data_grid();

            // ⌘C copies the selection as TSV (paste-ready for spreadsheets)
            if !self.copy_provider_set {
                self.copy_provider_set = true;
                let shared = self.shared.clone();
                grid.set_copy_provider(Box::new(move |sel| {
                    let Ok(guard) = shared.lock() else {
                        return String::new();
                    };
                    let (r0, r1) = sel.row_range();
                    let (c0, c1) = sel.col_range();
                    let mut out = String::new();
                    for r in r0..=r1 {
                        let Some(row) = guard.rows.get(r) else {
                            continue;
                        };
                        let cells: Vec<String> = (c0..=c1)
                            .map(|c| row.get(c).cloned().unwrap_or_default())
                            .collect();
                        out.push_str(&cells.join("\t"));
                        out.push('\n');
                    }
                    out
                }));
            }

            let g = &mut *grid.borrow_mut().unwrap();
            g.set_grid_size(guard.rows.len(), guard.col_labels.len());
            // recompute content widths on data change, and re-apply when the
            // pane width changes — the final (stretched) widths are applied
            // in the same pass, so there is no content-width flash frame
            let resized = host_width != self.last_host_width;
            if guard.version != self.last_version || resized {
                self.last_version = guard.version;
                self.last_host_width = host_width;
                g.set_col_labels(guard.col_labels.clone());
                g.set_selection(cx, None);
                self.content_widths = guard
                    .col_labels
                    .iter()
                    .enumerate()
                    .map(|(i, label)| {
                        let mut max_len = label.chars().count();
                        for row in guard.rows.iter().take(60) {
                            if let Some(cell) = row.get(i) {
                                max_len = max_len.max(cell.chars().count().min(24));
                            }
                        }
                        (16.0 + max_len as f64 * 8.0).clamp(64.0, 300.0)
                    })
                    .collect();
                apply_col_widths(&self.content_widths, g, host_width);
            }
            if self.applied_sort != guard.sort {
                self.applied_sort = guard.sort;
                g.set_sort_indicator(guard.sort);
            }

            while let Some(cell) = g.next_cell(cx) {
                // the cell being edited hosts the live editor: an FK
                // dropdown when options are loaded, else a TextInput
                if guard.editing == Some((cell.row, cell.col)) {
                    if !guard.fk_labels.is_empty() {
                        if let Some(item) = g.item(cx, cell.row, cell.col, live_id!(FkEditor)) {
                            item.as_drop_down().set_labels(cx, guard.fk_labels.clone());
                            g.draw_item(cx, &cell, &item, None);
                        }
                        continue;
                    }
                    if let Some(item) = g.item(cx, cell.row, cell.col, live_id!(Editor)) {
                        let seed = guard.edit_seed.take();
                        if let Some(seed) = &seed {
                            item.as_text_input().set_text(cx, seed);
                        }
                        g.draw_item(cx, &cell, &item, None);
                        if seed.is_some() {
                            item.as_text_input().take_key_focus(cx);
                        }
                    }
                    continue;
                }
                let raw = guard
                    .rows
                    .get(cell.row)
                    .and_then(|r| r.get(cell.col))
                    .cloned()
                    .unwrap_or_default();
                let is_null = raw == "NULL";
                let numeric = guard.numeric_cols.get(cell.col).copied().unwrap_or(false);
                let text = if numeric && !is_null {
                    fmt_numeric_display(&raw)
                } else {
                    raw.clone()
                };
                let mut style = CellStyle::default();
                if is_null {
                    style.color = Some(Vec4f {
                        x: COLOR_NULL,
                        y: COLOR_NULL,
                        z: COLOR_NULL,
                        w: 1.0,
                    });
                }
                if numeric && !is_null {
                    style.align = 1.0; // right-align numbers
                }
                g.cell_text_styled(cx, &cell, &text, style);
            }
        }
        DrawStep::done()
    }
}

// ---------------------------------------------------------------------------
// Convenience accessors used by the App
// ---------------------------------------------------------------------------

impl DbGridHostRef {
    /// Push a new snapshot into the grid and repaint.
    pub fn push_snapshot(&self, cx: &mut Cx, snapshot: GridShared) {
        if let Some(mut inner) = self.borrow_mut() {
            if let Ok(mut guard) = inner.shared.lock() {
                guard.col_labels = snapshot.col_labels;
                guard.rows = snapshot.rows;
                guard.sort = snapshot.sort;
                guard.editable = snapshot.editable;
                guard.editing = snapshot.editing;
                guard.edit_seed = snapshot.edit_seed;
                guard.fk_labels.clear();
                guard.fk_values.clear();
                guard.version += 1;
            }
            inner.redraw(cx);
        }
    }

    /// Begin inline editing of a cell. `seed` replaces the current value in
    /// the editor (e.g. typed-over text); None edits the existing value.
    pub fn start_edit(&self, cx: &mut Cx, row: usize, col: usize, seed: Option<String>) {
        if let Some(mut inner) = self.borrow_mut() {
            if let Ok(mut guard) = inner.shared.lock() {
                guard.editing = Some((row, col));
                guard.fk_labels.clear();
                guard.fk_values.clear();
                guard.edit_seed = Some(seed.unwrap_or_default());
            }
            inner.redraw(cx);
        }
    }

    /// Begin editing of an FK cell with a dropdown of options.
    pub fn start_fk_edit(
        &self,
        cx: &mut Cx,
        row: usize,
        col: usize,
        labels: Vec<String>,
        values: Vec<String>,
    ) {
        if let Some(mut inner) = self.borrow_mut() {
            if let Ok(mut guard) = inner.shared.lock() {
                guard.editing = Some((row, col));
                guard.edit_seed = None;
                guard.fk_labels = labels;
                guard.fk_values = values;
            }
            inner.redraw(cx);
        }
    }

    /// The committed value for dropdown option `idx`, if still valid.
    pub fn fk_value_at(&self, idx: usize) -> Option<String> {
        let inner = self.borrow()?;
        let guard = inner.shared.lock().ok()?;
        guard.fk_values.get(idx).cloned()
    }

    /// Current text of the live editor, if editing.
    pub fn live_editor_text(&self, cx: &Cx) -> Option<String> {
        let inner = self.borrow()?;
        let guard = inner.shared.lock().ok()?;
        let (row, col) = guard.editing?;
        let (_, widget) = inner.view.data_grid(cx, ids!(grid)).get_item(row, col)?;
        Some(widget.as_text_input().text())
    }

    /// Stop editing (discard the editor), optionally returning key focus to
    /// the grid for keyboard navigation.
    pub fn end_edit(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            if let Ok(mut guard) = inner.shared.lock() {
                guard.editing = None;
                guard.edit_seed = None;
                guard.fk_labels.clear();
                guard.fk_values.clear();
            }
            inner.redraw(cx);
        }
    }

    /// The cell currently being edited, if any.
    pub fn editing_cell(&self) -> Option<(usize, usize)> {
        let inner = self.borrow()?;
        let guard = inner.shared.lock().ok()?;
        guard.editing
    }
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    mod.widgets.DbGridHost = #(DbGridHost::register_widget(vm)){
        width: Fill
        height: Fill
        flow: Down
    }
}
