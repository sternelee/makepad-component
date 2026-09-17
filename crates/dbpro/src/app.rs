// DbPro — main application.
//
// Layout (TablePro-inspired):
//
//   ┌──────────────────────────────────────────────────────────┐
//   │ ⚡ DbPro  ● status        [＋ Query] [＋ Connection]      │
//   ├───────────────┬──────────────────────────────────────────┤
//   │ CONNECTIONS ⚙ │ [users ×] [Query 1 ×]              [+]   │
//   │ ◉ Demo(SQLite)│ ┌────────────────────────────────────┐  │
//   │  users        │ │ [filter…] [◀ 1/1 ▶] [Reload]         │  │
//   │  products     │ │ # │ id │ name │ …  (paged grid)     │  │
//   │  order_sum…   │ └────────────────────────────────────┘  │
//   ├───────────────┴──────────────────────────────────────────┤
//   │ SQLite · dbpro-demo.db           Makepad · Rust          │
//   └──────────────────────────────────────────────────────────┘
//
// Every tab carries its own state (page / search / sort / cached rows for
// table tabs, SQL + last result for query tabs) and is bound to the
// connection it was opened from, so multiple connections can be browsed
// side by side.
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use makepad_widgets::*;

// **Every import is explicit, because v3's `mp/mod.rs` does not re-export.** The v2 glob (`widgets::*`) worked
// because `widgets/mod.rs` did `pub use button::*` for each module, which is what brought the generated
// `*WidgetRefExt` traits into scope. Naming them here is the price of the v3 layout, and it is a small one.
use makepad_component::mp::button::MpButtonWidgetRefExt;
use makepad_component::mp::dialog::MpDialogWidgetRefExt;
use makepad_component::mp::editor::MpEditorWidgetRefExt;
use makepad_component::mp::segmented::MpSegmentedWidgetRefExt;
use makepad_component::mp::table::{MpTableWidgetRefExt, TableColumn};
use makepad_component::mp::tree::{MpTreeWidgetRefExt, TreeItem};

use crate::db::{
    self, ColumnInfo, ConnectionConfig, DbAction, DbConn, DbKind, FkInfo, IndexInfo, QueryResult,
    TableInfo, TableKind,
};
use crate::grid::*;
use crate::tab_bar::{DbTabBarWidgetRefExt, TabDef};

// Type aliases keep the token-based Script derive parser happy
// (it cannot parse commas nested inside generic arguments).
type ConnMap = HashMap<u64, Arc<Mutex<DbConn>>>;
type TableMap = HashMap<u64, Vec<TableInfo>>;
type CountMap = HashMap<u64, HashMap<String, u64>>;

// ---------------------------------------------------------------------------
// Tab model
// ---------------------------------------------------------------------------

/// A table browser tab. Owns its paging / search / sort state and the last
/// fetched page (so switching tabs back and forth is instant).
#[derive(Clone, Debug, PartialEq)]
struct TableTab {
    config_id: u64,
    table: String,
    page: usize,
    search: String,
    order_col: Option<String>,
    order_asc: bool,
    columns: Vec<ColumnInfo>,
    rows: Vec<Vec<String>>,
    total: u64,
    loaded: bool,
    pending: bool,
    show_struct: bool,
    error: Option<String>,
    elapsed_ms: f64,
    ddl: Option<String>,
    pending_ddl: bool,
    fks: Vec<FkInfo>,
    fks_loaded: bool,
    indexes: Option<Vec<IndexInfo>>,
    pending_indexes: bool,
}

/// A SQL editor tab. Remembers its statement and last result.
#[derive(Clone, Debug, PartialEq)]
struct QueryTab {
    config_id: u64,
    sql: String,
    result: Option<QueryResult>,
    pending: bool,
}

#[derive(Clone, Debug, PartialEq)]
enum TabKind {
    Table(TableTab),
    Query(QueryTab),
}

#[derive(Clone, Debug)]
struct TabState {
    id: u64,
    kind: TabKind,
}

/// Which connection an active tab points at (0 = none).
const NO_CONN: u64 = u64::MAX;

/// Sidebar row → what it points at (parallel to the flat tree model).
#[derive(Clone, Debug, PartialEq)]
enum SideItem {
    Conn(u64),
    Object(u64, String),
    Nothing,
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

app_main!(App);

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,

    #[rust]
    next_config_id: u64,

    #[rust]
    next_tab_id: u64,

    #[rust]
    configs: Vec<ConnectionConfig>,

    #[rust]
    conns: ConnMap,

    #[rust]
    tables: TableMap,

    /// Row counts per (config id, table), filled by a background sweep.
    #[rust]
    counts: CountMap,

    #[rust]
    active_config: Option<u64>,

    #[rust]
    dialog_password: String,

    #[rust]
    dialog_editing: Option<u64>,

    #[rust]
    dialog_kind: DbKind,

    #[rust]
    tabs: Vec<TabState>,

    #[rust]
    active_tab: Option<usize>,

    #[rust]
    sidebar_filter: String,

    #[rust]
    tree_items: Vec<TreeItem>,

    #[rust]
    sidebar_items: Vec<SideItem>,

    #[rust]
    grid_shared: Arc<Mutex<GridShared>>,

    /// Row the user last clicked in the data grid (for row ops).
    #[rust]
    last_grid_row: Option<usize>,

    /// Pending row delete awaiting confirmation: (sql, description)
    #[rust]
    pending_delete: Option<(String, String)>,

    /// Table the import dialog targets.
    #[rust]
    import_table: String,

    /// FK cell edit waiting for its dropdown options: (row, col).
    #[rust]
    pending_fk_edit: Option<(usize, usize)>,

    /// Raw FK values parallel to the dropdown labels.
    #[rust]
    fk_commit_values: Vec<String>,

    /// A full-table export is currently running.
    #[rust]
    export_running: bool,

    #[rust]
    search_timer: Timer,

    #[rust]
    history: Vec<String>,

    #[rust]
    history_idx: usize,
}

// Status dot colors (packed #RRGGBBAA for the script heap).
const DOT_IDLE: u32 = 0x5a5a64ff;
const DOT_OK: u32 = 0x4caf6eff;
const DOT_BUSY: u32 = 0xe0b04fff;
const DOT_ERR: u32 = 0xe05f65ff;

#[derive(Clone, Copy)]
enum Dot {
    Idle,
    Ok,
    Busy,
    Err,
}

impl Dot {
    fn color(self) -> u32 {
        match self {
            Dot::Idle => DOT_IDLE,
            Dot::Ok => DOT_OK,
            Dot::Busy => DOT_BUSY,
            Dot::Err => DOT_ERR,
        }
    }
}

/// Which kind of tab is active (borrow-free snapshot for dispatch).
#[derive(Clone, Copy, PartialEq)]
enum ActiveKind {
    Table,
    Query,
    None,
}

/// Build the sidebar tree model (pure — unit-testable). Groups with a
/// non-empty `group` field become `📁` folders; ungrouped connections sit
/// at the root. Tables can be narrowed by `filter` (lowercase substring).
fn build_sidebar_model(
    configs: &[ConnectionConfig],
    tables: &HashMap<u64, Vec<TableInfo>>,
    counts: &HashMap<u64, HashMap<String, u64>>,
    connected: &std::collections::HashSet<u64>,
    filter: &str,
) -> (Vec<TreeItem>, Vec<SideItem>) {
    let filter_lower = filter.to_lowercase();
    let mut items: Vec<TreeItem> = Vec::new();
    let mut side: Vec<SideItem> = Vec::new();

    #[allow(clippy::too_many_arguments)]
    fn push_conn(
        items: &mut Vec<TreeItem>,
        side: &mut Vec<SideItem>,
        cfg_id: u64,
        name: &str,
        connected: bool,
        tables: Option<&Vec<TableInfo>>,
        counts: Option<&HashMap<String, u64>>,
        filter_lower: &str,
        depth: usize,
    ) {
        let icon = if connected { "◉" } else { "○" };
        items.push(TreeItem::new(&format!("{} {}", icon, name), depth));
        side.push(SideItem::Conn(cfg_id));
        let before = side.len();
        if let Some(tables) = tables {
            for t in tables {
                if !filter_lower.is_empty() && !t.name.to_lowercase().contains(filter_lower) {
                    continue;
                }
                let count = counts
                    .and_then(|m| m.get(&t.name))
                    .map(|n| format!(" ({})", n))
                    .unwrap_or_default();
                // avoid exotic glyphs (tofu in the bundled font)
                let label = match t.kind {
                    TableKind::Table => t.name.clone(),
                    TableKind::View => format!("{} (view)", t.name),
                };
                items.push(TreeItem::new(&format!("{}{}", label, count), depth + 1));
                side.push(SideItem::Object(cfg_id, t.name.clone()));
            }
        }
        if connected && side.len() == before {
            let hint = if filter_lower.is_empty() {
                "(no tables)"
            } else {
                "(no match)"
            };
            items.push(TreeItem::new(hint, depth + 1));
            side.push(SideItem::Nothing);
        }
    }

    // grouped connections first, folders in first-seen order
    let mut groups_done: Vec<String> = Vec::new();
    for cfg in configs {
        let group = cfg.group.trim().to_string();
        if group.is_empty() {
            continue;
        }
        if !groups_done.contains(&group) {
            groups_done.push(group.clone());
            items.push(TreeItem::new(&format!("📁 {}", group), 0));
            side.push(SideItem::Nothing);
        }
        push_conn(
            &mut items,
            &mut side,
            cfg.id,
            &cfg.name,
            connected.contains(&cfg.id),
            tables.get(&cfg.id),
            counts.get(&cfg.id),
            &filter_lower,
            1,
        );
    }
    // ungrouped connections at the root
    for cfg in configs {
        if !cfg.group.trim().is_empty() {
            continue;
        }
        push_conn(
            &mut items,
            &mut side,
            cfg.id,
            &cfg.name,
            connected.contains(&cfg.id),
            tables.get(&cfg.id),
            counts.get(&cfg.id),
            &filter_lower,
            0,
        );
    }
    (items, side)
}

impl App {
    fn active_kind(&self) -> ActiveKind {
        match self.active_tab_ref() {
            Some(t) => match &t.kind {
                TabKind::Table(_) => ActiveKind::Table,
                TabKind::Query(_) => ActiveKind::Query,
            },
            None => ActiveKind::None,
        }
    }

    fn active_tab_ref(&self) -> Option<&TabState> {
        self.active_tab.and_then(|i| self.tabs.get(i))
    }

    fn push_status(&mut self, cx: &mut Cx, text: &str) {
        self.ui.label(cx, ids!(status_label)).set_text(cx, text);
    }

    /// Update the toolbar status dot color through the script heap.
    fn set_dot(&mut self, cx: &mut Cx, dot: Dot) {
        let color = ScriptValue::from_color(dot.color());
        cx.with_vm(|vm| {
            let m = vm.module(id!(db_state));
            let t = NoTrap;
            vm.bx.heap.set_value(m, id!(status_color).into(), color, t);
        });
        let mut dot_view = self.ui.view(cx, ids!(status_dot));
        script_apply_eval!(cx, dot_view, { draw_bg +: { color: mod.db_state.status_color } });
        self.ui.redraw(cx);
    }

    /// Bottom-bar description of the active connection + window title.
    fn update_conn_label(&mut self, cx: &mut Cx) {
        let cfg = self
            .active_config
            .and_then(|id| self.configs.iter().find(|c| c.id == id))
            .cloned();
        let text = match &cfg {
            Some(c) => format!("{} · {}", c.kind.label(), c.display_host()),
            None => "—".to_string(),
        };
        self.ui.label(cx, ids!(conn_label)).set_text(cx, &text);
        let title = match &cfg {
            Some(c) => format!("DbPro — {}", c.name),
            None => "DbPro — Database GUI".to_string(),
        };
        self.ui.window(cx, ids!(main_window)).set_title(cx, &title);
    }

    // -- sidebar -------------------------------------------------------------

    fn rebuild_tree(&mut self, cx: &mut Cx) {
        let connected: std::collections::HashSet<u64> = self.conns.keys().copied().collect();
        let (items, side) = build_sidebar_model(
            &self.configs,
            &self.tables,
            &self.counts,
            &connected,
            &self.sidebar_filter,
        );
        let conn_indices: Vec<usize> = side
            .iter()
            .enumerate()
            .filter(|(_, s)| matches!(s, SideItem::Conn(_)))
            .map(|(i, _)| i)
            .collect();
        self.tree_items = items;
        self.sidebar_items = side;
        let tree = self.ui.mp_tree(cx, ids!(sidebar_tree));
        tree.set_items(cx, self.tree_items.clone());
        // only depth-0 branches auto-expand; grouped connections sit at
        // depth 1, so open them explicitly
        for idx in conn_indices {
            tree.set_collapsed(cx, idx, false);
        }
    }

    // -- tab bar --------------------------------------------------------------

    fn tab_title(&self, tab: &TabState) -> String {
        match &tab.kind {
            TabKind::Table(t) => t.table.clone(),
            // "Query N": N counts query tabs created before this one;
            // with several connections, disambiguate with the name
            TabKind::Query(q) => {
                let n = 1 + self
                    .tabs
                    .iter()
                    .filter(|t| matches!(t.kind, TabKind::Query(_)) && t.id < tab.id)
                    .count();
                if self.conns.len() > 1 {
                    if let Some(cfg) = self.configs.iter().find(|c| c.id == q.config_id) {
                        return format!("Query {} · {}", n, cfg.name);
                    }
                }
                format!("Query {}", n)
            }
        }
    }

    fn sync_tab_bar(&mut self, cx: &mut Cx) {
        let defs: Vec<TabDef> = self
            .tabs
            .iter()
            .map(|t| TabDef::new(&self.tab_title(t), true))
            .collect();
        self.ui
            .db_tab_bar(cx, ids!(tab_bar))
            .set_tabs(cx, defs, self.active_tab);
    }

    // -- tab lifecycle ---------------------------------------------------------

    fn open_table_tab(&mut self, cx: &mut Cx, config_id: u64, table: &str) {
        if let Some(pos) = self.tabs.iter().position(|t| {
            matches!(&t.kind, TabKind::Table(tt) if tt.config_id == config_id && tt.table == table)
        }) {
            self.activate_tab(cx, Some(pos));
            return;
        }
        self.next_tab_id += 1;
        self.tabs.push(TabState {
            id: self.next_tab_id,
            kind: TabKind::Table(TableTab {
                config_id,
                table: table.to_string(),
                page: 0,
                search: String::new(),
                order_col: None,
                order_asc: true,
                columns: Vec::new(),
                rows: Vec::new(),
                total: 0,
                loaded: false,
                pending: false,
                show_struct: false,
                error: None,
                elapsed_ms: 0.0,
                ddl: None,
                pending_ddl: false,
                fks: Vec::new(),
                fks_loaded: false,
                indexes: None,
                pending_indexes: false,
            }),
        });
        self.activate_tab(cx, Some(self.tabs.len() - 1));
    }

    fn open_query_tab(&mut self, cx: &mut Cx) {
        self.next_tab_id += 1;
        let config_id = self
            .active_config
            .or_else(|| self.configs.first().map(|c| c.id))
            .unwrap_or(NO_CONN);
        self.tabs.push(TabState {
            id: self.next_tab_id,
            kind: TabKind::Query(QueryTab {
                config_id,
                sql: String::new(),
                result: None,
                pending: false,
            }),
        });
        self.activate_tab(cx, Some(self.tabs.len() - 1));
    }

    /// Save the editor text into the active query tab (call before the
    /// active tab changes or when running).
    fn sync_editor_into_tab(&mut self, cx: &mut Cx) {
        if let Some(pos) = self.active_tab {
            let text = self.ui.text_input(cx, ids!(sql_editor)).text();
            if let TabKind::Query(q) = &mut self.tabs[pos].kind {
                q.sql = text;
            }
        }
    }

    /// Switch the active tab (None = empty state). `pos` must be valid.
    fn activate_tab(&mut self, cx: &mut Cx, pos: Option<usize>) {
        self.active_tab = pos;
        self.sync_tab_bar(cx);
        match pos {
            None => {
                self.ui.view(cx, ids!(table_view)).set_visible(cx, false);
                self.ui.view(cx, ids!(query_view)).set_visible(cx, false);
                self.ui.view(cx, ids!(empty_view)).set_visible(cx, true);
                self.push_status(cx, "Open a table from the sidebar");
            }
            Some(i) => {
                // the tab drives the "active" connection (bottom bar, ⚙ edit)
                let tab_cfg = match &self.tabs[i].kind {
                    TabKind::Table(t) => t.config_id,
                    TabKind::Query(q) => q.config_id,
                };
                if tab_cfg != NO_CONN && self.active_config != Some(tab_cfg) {
                    self.active_config = Some(tab_cfg);
                    self.update_conn_label(cx);
                }
                let tab = self.tabs[i].clone();
                match tab.kind {
                    TabKind::Table(mut t) => self.render_table_tab(cx, &mut t, true),
                    TabKind::Query(q) => self.render_query_tab(cx, &q, true),
                }
                // lazily fetch data for unloaded table tabs
                let needs_load = match &self.tabs[i].kind {
                    TabKind::Table(t) => !t.loaded && !t.pending,
                    _ => false,
                };
                if needs_load {
                    self.load_active_table(cx);
                }
            }
        }
        self.ui.redraw(cx);
    }

    // -- rendering: table tab ---------------------------------------------------

    /// Full render of a table tab. `fresh` = also reset widgets that would
    /// otherwise disturb typing (search input).
    fn render_table_tab(&mut self, cx: &mut Cx, t: &mut TableTab, fresh: bool) {
        self.ui.view(cx, ids!(table_view)).set_visible(cx, true);
        self.ui.view(cx, ids!(query_view)).set_visible(cx, false);
        self.ui.view(cx, ids!(empty_view)).set_visible(cx, false);

        // mode toggle state
        self.apply_mode_button_style(cx, t.show_struct);
        self.ui
            .view(cx, ids!(data_view))
            .set_visible(cx, !t.show_struct);
        self.ui
            .view(cx, ids!(struct_view))
            .set_visible(cx, t.show_struct);

        if fresh {
            self.ui
                .text_input(cx, ids!(search_input))
                .set_text(cx, &t.search);
        }

        if t.show_struct {
            self.render_struct(cx, t);
            return;
        }

        self.push_grid_snapshot(cx, t);

        self.update_pager(cx, t);
        match &t.error {
            Some(e) => {
                self.set_dot(cx, Dot::Err);
                self.push_status(cx, &format!("⚠ {}", e));
            }
            None => {
                if t.pending {
                    self.set_dot(cx, Dot::Busy);
                    self.push_status(cx, &format!("Loading {}…", t.table));
                } else if t.loaded {
                    self.set_dot(cx, Dot::Ok);
                    let pages = (t.total as usize).div_ceil(db::PAGE_SIZE).max(1);
                    self.push_status(
                        cx,
                        &format!(
                            "{} · {} rows · page {}/{} · {:.1} ms",
                            t.table,
                            t.total,
                            (t.page + 1).min(pages),
                            pages,
                            t.elapsed_ms
                        ),
                    );
                }
            }
        }
    }

    fn render_struct(&mut self, cx: &mut Cx, t: &mut TableTab) {
        let defs = vec![
            TableColumn::new("Column").width(200.0),
            TableColumn::new("Type").width(150.0),
            TableColumn::new("Nullable").width(90.0),
            TableColumn::new("Key").width(70.0),
        ];
        let rows: Vec<Vec<String>> = t
            .columns
            .iter()
            .map(|c| {
                vec![
                    c.name.clone(),
                    c.data_type.clone(),
                    if c.nullable { "YES" } else { "NO" }.to_string(),
                    if c.is_key { "PK" } else { "—" }.to_string(),
                ]
            })
            .collect();
        let grid = self.ui.mp_table(cx, ids!(struct_table));
        grid.set_columns(cx, defs);
        grid.set_rows(cx, rows);
        let hint = if t.columns.is_empty() {
            "Structure unavailable — load the table first".to_string()
        } else {
            format!("{} · {} columns", t.table, t.columns.len())
        };
        self.ui.label(cx, ids!(struct_hint)).set_text(cx, &hint);

        // indexes: fetch once per tab on first view
        if t.indexes.is_none() && !t.pending_indexes && !t.columns.is_empty() {
            if let Some(conn) = self.conns.get(&t.config_id).cloned() {
                t.pending_indexes = true;
                db::spawn_fetch_indexes(conn, t.config_id, t.table.clone());
            }
        }
        let idx_defs = vec![
            TableColumn::new("Index").width(170.0),
            TableColumn::new("Columns").width(230.0),
            TableColumn::new("Unique").width(70.0),
        ];
        let idx_rows: Vec<Vec<String>> = match &t.indexes {
            Some(list) => list
                .iter()
                .map(|ix| {
                    vec![
                        ix.name.clone(),
                        ix.columns.clone(),
                        if ix.unique { "UNIQUE" } else { "" }.to_string(),
                    ]
                })
                .collect(),
            None => Vec::new(),
        };
        let idx_table = self.ui.mp_table(cx, ids!(indexes_table));
        idx_table.set_columns(cx, idx_defs);
        idx_table.set_rows(cx, idx_rows);

        // DDL preview: fetch once per tab, then reuse
        let ddl_text = match &t.ddl {
            Some(ddl) => ddl.clone(),
            None => {
                if !t.pending_ddl && !t.columns.is_empty() {
                    if let Some(conn) = self.conns.get(&t.config_id).cloned() {
                        t.pending_ddl = true;
                        self.ui
                            .mp_editor(cx, ids!(struct_ddl))
                            .set_source(cx, "-- loading DDL…");
                        db::spawn_fetch_ddl(conn, t.config_id, t.table.clone());
                    }
                }
                String::new()
            }
        };
        self.ui
            .mp_editor(cx, ids!(struct_ddl))
            .set_source(cx, &ddl_text);
        self.set_dot(cx, if t.error.is_some() { Dot::Err } else { Dot::Ok });
    }

    /// Style the Data/Structure mode buttons: the ACTIVE one becomes a
    /// filled Secondary plate, the inactive one a Ghost. (draw_walk rewrites
    /// text/bg colors from the variant style every frame, so per-instance
    /// color overrides would be clobbered — switching the variant is the
    /// reliable way.)
    /// Show the matching Data/Structure button pair (deterministic — no
    /// runtime style mutation).
    fn apply_mode_button_style(&self, cx: &mut Cx, show_struct: bool) {
        // MpButton has no working set_visible — toggle the wrapping Views
        self.ui
            .view(cx, ids!(mode_data_on_wrap))
            .set_visible(cx, !show_struct);
        self.ui
            .view(cx, ids!(mode_struct_off_wrap))
            .set_visible(cx, !show_struct);
        self.ui
            .view(cx, ids!(mode_data_off_wrap))
            .set_visible(cx, show_struct);
        self.ui
            .view(cx, ids!(mode_struct_on_wrap))
            .set_visible(cx, show_struct);
    }

    /// Push the active table page into the editable grid.
    fn push_grid_snapshot(&mut self, cx: &mut Cx, t: &TableTab) {
        let sort = t.order_col.as_ref().and_then(|name| {
            t.columns
                .iter()
                .position(|c| &c.name == name)
                .map(|i| (i, t.order_asc))
        });
        let numeric_cols: Vec<bool> = t
            .columns
            .iter()
            .map(|c| {
                let d = c.data_type.to_lowercase();
                [
                    "int", "float", "double", "real", "numeric", "decimal", "number", "serial",
                ]
                .iter()
                .any(|k| d.contains(k))
            })
            .collect();
        self.ui.db_grid_host(cx, ids!(db_grid)).push_snapshot(
            cx,
            GridShared {
                col_labels: t.columns.iter().map(|c| c.name.clone()).collect(),
                numeric_cols,
                rows: t.rows.clone(),
                sort,
                editable: t.columns.iter().any(|c| c.is_key),
                ..Default::default()
            },
        );
    }

    // -- cell editing / row ops ---------------------------------------------

    fn start_cell_edit(&mut self, cx: &mut Cx, row: usize, col: usize, replace: Option<String>) {
        let Some(pos) = self.active_tab else { return };
        let t = match &self.tabs[pos].kind {
            TabKind::Table(t) => t.clone(),
            _ => return,
        };
        if !t.columns.iter().any(|c| c.is_key) {
            self.set_dot(cx, Dot::Err);
            self.push_status(
                cx,
                "⚠ Editing needs a primary key — views and PK-less tables are read-only",
            );
            return;
        }
        // FK column → dropdown of referenced values (async fetch, then edit)
        if let Some(col_name) = t.columns.get(col).map(|c| c.name.clone()) {
            if let Some(fk) = t.fks.iter().find(|f| f.column == col_name).cloned() {
                self.pending_fk_edit = Some((row, col));
                self.push_status(cx, &format!("Loading {} options…", fk.ref_table));
                if let Some(conn) = self.conns.get(&t.config_id).cloned() {
                    db::spawn_fetch_fk_options(conn, fk);
                }
                self.ui.redraw(cx);
                return;
            }
        }
        let seed = match replace {
            Some(s) => Some(s),
            None => {
                let cur = t
                    .rows
                    .get(row)
                    .and_then(|r| r.get(col))
                    .cloned()
                    .unwrap_or_default();
                Some(if cur == "NULL" { String::new() } else { cur })
            }
        };
        self.ui
            .db_grid_host(cx, ids!(db_grid))
            .start_edit(cx, row, col, seed);
    }

    fn commit_cell_edit(&mut self, cx: &mut Cx, row: usize, col: usize, text: &str) {
        self.ui.db_grid_host(cx, ids!(db_grid)).end_edit(cx);
        // hand keyboard focus back to the grid so navigation keeps working
        let grid_area = self.ui.data_grid(cx, ids!(db_grid.grid)).area();
        cx.set_key_focus(grid_area);
        let Some(pos) = self.active_tab else { return };
        let t = match &self.tabs[pos].kind {
            TabKind::Table(t) => t.clone(),
            _ => return,
        };
        let tab_id = self.tabs[pos].id;
        let Some(ci) = t.columns.get(col) else { return };
        // primary key values from the cached row
        let mut pk: Vec<(String, String)> = Vec::new();
        for (i, c) in t.columns.iter().enumerate() {
            if c.is_key {
                if let Some(v) = t.rows.get(row).and_then(|r| r.get(i)) {
                    pk.push((c.name.clone(), v.clone()));
                }
            }
        }
        if pk.is_empty() {
            self.set_dot(cx, Dot::Err);
            self.push_status(cx, "⚠ No primary key — row is read-only");
            return;
        }
        let is_null = text.is_empty() || text == "NULL";
        if is_null && !ci.nullable {
            self.set_dot(cx, Dot::Err);
            self.push_status(cx, &format!("⚠ Column {} is NOT NULL", ci.name));
            return;
        }
        let new_value: Option<&str> = if is_null { None } else { Some(text) };
        let Some(sql) = db::build_update(
            cfg_kind(self, t.config_id),
            &t.table,
            &ci.name,
            new_value,
            &pk,
        ) else {
            return;
        };
        // optimistic local update, then verify on the server
        let display = new_value
            .map(|s| s.to_string())
            .unwrap_or_else(|| "NULL".into());
        if let TabKind::Table(t) = &mut self.tabs[pos].kind {
            if let Some(r) = t.rows.get_mut(row) {
                if let Some(c) = r.get_mut(col) {
                    *c = display;
                }
            }
        }
        let t2 = match &self.tabs[pos].kind {
            TabKind::Table(t) => t.clone(),
            _ => return,
        };
        self.push_grid_snapshot(cx, &t2);
        self.set_dot(cx, Dot::Busy);
        self.push_status(cx, "Updating…");
        if let Some(conn) = self.conns.get(&t2.config_id).cloned() {
            db::spawn_run_sql(conn, format!("u{}", tab_id), sql);
        }
        self.ui.redraw(cx);
    }

    fn delete_selected_row(&mut self, cx: &mut Cx) {
        let Some(row) = self.last_grid_row else {
            self.push_status(cx, "Select a row first (click a row, then － Row)");
            return;
        };
        let Some(pos) = self.active_tab else { return };
        let t = match &self.tabs[pos].kind {
            TabKind::Table(t) => t.clone(),
            _ => return,
        };
        let mut pk: Vec<(String, String)> = Vec::new();
        for (i, c) in t.columns.iter().enumerate() {
            if c.is_key {
                if let Some(v) = t.rows.get(row).and_then(|r| r.get(i)) {
                    pk.push((c.name.clone(), v.clone()));
                }
            }
        }
        let Some(sql) = db::build_delete(cfg_kind(self, t.config_id), &t.table, &pk) else {
            self.set_dot(cx, Dot::Err);
            self.push_status(cx, "⚠ No primary key — row is read-only");
            return;
        };
        // confirm before destroying data
        let desc = pk
            .iter()
            .map(|(k, v)| format!("{} = {}", k, v))
            .collect::<Vec<_>>()
            .join(", ");
        self.pending_delete = Some((sql, desc.clone()));
        self.ui
            .label(cx, ids!(confirm_msg))
            .set_text(cx, &format!("Delete row from {} where {}?", t.table, desc));
        self.ui.mp_dialog(cx, ids!(confirm_dialog)).open(cx);
        self.ui.redraw(cx);
    }

    fn run_pending_delete(&mut self, cx: &mut Cx) {
        let Some(pos) = self.active_tab else { return };
        let t = match &self.tabs[pos].kind {
            TabKind::Table(t) => t.clone(),
            _ => return,
        };
        let tab_id = self.tabs[pos].id;
        let Some((sql, desc)) = self.pending_delete.take() else {
            return;
        };
        self.set_dot(cx, Dot::Busy);
        self.push_status(cx, &format!("Deleting row ({})…", desc));
        if let Some(conn) = self.conns.get(&t.config_id).cloned() {
            db::spawn_run_sql(conn, format!("d{}", tab_id), sql);
        }
        self.ui.redraw(cx);
    }

    fn insert_default_row(&mut self, cx: &mut Cx) {
        let Some(pos) = self.active_tab else { return };
        let t = match &self.tabs[pos].kind {
            TabKind::Table(t) => t.clone(),
            _ => return,
        };
        let tab_id = self.tabs[pos].id;
        if !t.columns.iter().any(|c| c.is_key) {
            self.set_dot(cx, Dot::Err);
            self.push_status(
                cx,
                "⚠ Adding rows needs a primary key — this table is read-only",
            );
            return;
        }
        let sql = db::build_insert_default(cfg_kind(self, t.config_id), &t.table);
        self.set_dot(cx, Dot::Busy);
        self.push_status(cx, &format!("Adding row to {}…", t.table));
        if let Some(conn) = self.conns.get(&t.config_id).cloned() {
            db::spawn_run_sql(conn, format!("n{}", tab_id), sql);
        }
        self.ui.redraw(cx);
    }

    /// Open the row-detail dialog: every column of the selected row.
    fn show_row_detail_dialog(&mut self, cx: &mut Cx) {
        let Some(row) = self.last_grid_row else {
            self.push_status(cx, "Select a row first (click a row, then Detail)");
            return;
        };
        let detail = match self.active_tab_ref() {
            Some(t) => match &t.kind {
                TabKind::Table(t) => {
                    let num = t.page * db::PAGE_SIZE + row + 1;
                    match t.rows.get(row) {
                        Some(values) => {
                            let mut out = format!("Row {} of {}\n\n", num, t.table);
                            for (i, c) in t.columns.iter().enumerate() {
                                let v = values.get(i).map(|s| s.as_str()).unwrap_or("NULL");
                                out.push_str(&format!("{}: {}\n", c.name, v));
                            }
                            Some(out)
                        }
                        None => None,
                    }
                }
                _ => None,
            },
            None => None,
        };
        if let Some(text) = detail {
            self.ui.mp_editor(cx, ids!(detail_text)).set_source(cx, &text);
            self.ui.mp_dialog(cx, ids!(detail_dialog)).open(cx);
            self.ui.redraw(cx);
        }
    }

    fn duplicate_selected_row(&mut self, cx: &mut Cx) {
        let Some(row) = self.last_grid_row else {
            self.push_status(cx, "Select a row first (click a row, then ⧉ Duplicate)");
            return;
        };
        let Some(pos) = self.active_tab else { return };
        let t = match &self.tabs[pos].kind {
            TabKind::Table(t) => t.clone(),
            _ => return,
        };
        let tab_id = self.tabs[pos].id;
        let Some(row_vals) = t.rows.get(row).cloned() else {
            return;
        };
        let Some(sql) =
            db::build_insert_copy(cfg_kind(self, t.config_id), &t.table, &t.columns, &row_vals)
        else {
            self.set_dot(cx, Dot::Err);
            self.push_status(cx, "⚠ Nothing to insert (no columns)");
            return;
        };
        self.set_dot(cx, Dot::Busy);
        self.push_status(cx, "Duplicating row…");
        if let Some(conn) = self.conns.get(&t.config_id).cloned() {
            db::spawn_run_sql(conn, format!("i{}", tab_id), sql);
        }
        self.ui.redraw(cx);
    }

    fn export_whole_table(&mut self, cx: &mut Cx) {
        let Some(pos) = self.active_tab else { return };
        let (config_id, table) = match &self.tabs[pos].kind {
            TabKind::Table(t) => (t.config_id, t.table.clone()),
            _ => {
                self.push_status(cx, "⬇ All exports the full table — open a table first");
                return;
            }
        };
        let Some(conn) = self.conns.get(&config_id).cloned() else {
            self.push_status(cx, "⚠ Connection is not open");
            return;
        };
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let path = format!("{}/dbpro-export-{}-all-{}.csv", home, table, compact_ts());
        if self.export_running {
            // second click while exporting → cancel
            db::EXPORT_CANCEL.store(true, std::sync::atomic::Ordering::Relaxed);
            self.push_status(cx, "Cancelling export…");
            self.ui.redraw(cx);
            return;
        }
        self.export_running = true;
        self.set_dot(cx, Dot::Busy);
        self.push_status(
            cx,
            &format!(
                "Exporting {} (all pages)… — click ⬇ All again to stop",
                table
            ),
        );
        db::spawn_export_table(conn, table, path);
        self.ui.redraw(cx);
    }

    fn refresh_schema(&mut self, cx: &mut Cx) {
        let Some(config_id) = self.active_config else {
            return;
        };
        let Some(conn) = self.conns.get(&config_id).cloned() else {
            self.push_status(cx, "⚠ Connection is not open");
            return;
        };
        self.push_status(cx, "Refreshing schema…");
        db::spawn_list_tables(conn, config_id);
        self.ui.redraw(cx);
    }

    fn show_row_detail(&mut self, cx: &mut Cx, row: usize, col: usize) {
        let detail = match self.active_tab_ref() {
            Some(t) => match &t.kind {
                TabKind::Table(t) => {
                    let num = t.page * db::PAGE_SIZE + row + 1;
                    match (
                        t.columns.get(col).map(|c| c.name.clone()),
                        t.rows.get(row).and_then(|r| r.get(col)).cloned(),
                    ) {
                        (Some(cn), Some(cv)) => Some((num, cn, cv)),
                        _ => None,
                    }
                }
                _ => None,
            },
            None => None,
        };
        if let Some((num, cn, cv)) = detail {
            self.push_status(cx, &format!("Row {} · {} = {}", num, cn, cv));
        }
    }

    fn update_pager(&mut self, cx: &mut Cx, t: &TableTab) {
        let pages = (t.total as usize).div_ceil(db::PAGE_SIZE).max(1);
        let label = format!(
            "Page {}/{} · {} rows",
            (t.page + 1).min(pages),
            pages,
            t.total
        );
        self.ui.label(cx, ids!(pager_label)).set_text(cx, &label);
        self.ui
            .mp_button(cx, ids!(page_prev))
            .set_disabled(cx, t.page == 0);
        self.ui
            .mp_button(cx, ids!(page_next))
            .set_disabled(cx, t.page + 1 >= pages);
    }

    // -- rendering: query tab ----------------------------------------------------

    fn render_query_tab(&mut self, cx: &mut Cx, q: &QueryTab, fresh: bool) {
        self.ui.view(cx, ids!(table_view)).set_visible(cx, false);
        self.ui.view(cx, ids!(query_view)).set_visible(cx, true);
        self.ui.view(cx, ids!(empty_view)).set_visible(cx, false);
        if fresh {
            self.ui
                .text_input(cx, ids!(sql_editor))
                .set_text(cx, &q.sql);
        }
        match &q.result {
            Some(r) => self.render_query_result(cx, r, q.pending),
            None => {
                if q.pending {
                    self.set_dot(cx, Dot::Busy);
                    self.push_status(cx, "Running…");
                } else {
                    self.set_dot(cx, Dot::Ok);
                    self.push_status(cx, "Ready — ⌘/Ctrl+Enter to run");
                }
            }
        }
    }

    fn render_query_result(&mut self, cx: &mut Cx, r: &QueryResult, pending: bool) {
        if pending {
            self.set_dot(cx, Dot::Busy);
            self.push_status(cx, "Running…");
            return;
        }
        if let Some(err) = &r.error {
            self.set_dot(cx, Dot::Err);
            self.push_status(cx, &format!("⚠ {}", err));
            return;
        }
        self.set_dot(cx, Dot::Ok);
        let summary = if let Some(affected) = r.rows_affected {
            format!("OK · {} rows affected · {:.1} ms", affected, r.elapsed_ms)
        } else {
            format!(
                "{} rows · {:.1} ms{}",
                r.rows.len(),
                r.elapsed_ms,
                if r.truncated { " (truncated)" } else { "" }
            )
        };
        self.push_status(cx, &summary);
        if !r.columns.is_empty() {
            let defs: Vec<TableColumn> = r
                .columns
                .iter()
                .map(|c| {
                    let w = (16.0 + c.name.chars().count() as f64 * 7.2).clamp(90.0, 260.0);
                    TableColumn::new(&c.name).width(w)
                })
                .collect();
            let grid = self.ui.mp_table(cx, ids!(query_grid));
            grid.set_columns(cx, defs);
            grid.set_rows(cx, r.rows.clone());
        }
    }

    // -- data loading ---------------------------------------------------------------

    fn load_active_table(&mut self, cx: &mut Cx) {
        let Some(pos) = self.active_tab else { return };
        let (config_id, table, page, search, order_col, order_asc) = match &self.tabs[pos].kind {
            TabKind::Table(t) => (
                t.config_id,
                t.table.clone(),
                t.page,
                t.search.clone(),
                t.order_col.clone(),
                t.order_asc,
            ),
            _ => return,
        };
        let Some(conn) = self.conns.get(&config_id).cloned() else {
            let err = "Connection is not open — click the connection in the sidebar";
            if let TabKind::Table(t) = &mut self.tabs[pos].kind {
                t.error = Some(err.to_string());
            }
            let mut t = match &self.tabs[pos].kind {
                TabKind::Table(t) => t.clone(),
                _ => return,
            };
            self.render_table_tab(cx, &mut t, false);
            return;
        };
        if let TabKind::Table(t) = &mut self.tabs[pos].kind {
            t.pending = true;
            t.error = None;
        }
        self.set_dot(cx, Dot::Busy);
        self.push_status(cx, &format!("Loading {}…", table));
        db::spawn_query_page(conn, config_id, table, page, search, order_col, order_asc);
        self.ui.redraw(cx);
    }

    fn apply_table_result(
        &mut self,
        cx: &mut Cx,
        config_id: u64,
        table: &str,
        page: usize,
        search: &str,
        result: db::PageResult,
    ) {
        // find the tab this response belongs to; drop stale responses
        // (the user paged / searched on in the meantime)
        let pos = self.tabs.iter().position(|t| match &t.kind {
            TabKind::Table(tt) => {
                tt.config_id == config_id
                    && tt.table == table
                    && tt.page == page
                    && tt.search == search
            }
            _ => false,
        });
        let Some(pos) = pos else { return };
        match result {
            Ok((columns, rows, total, ms)) => {
                if let TabKind::Table(t) = &mut self.tabs[pos].kind {
                    t.columns = columns;
                    t.rows = rows;
                    t.total = total;
                    t.loaded = true;
                    t.pending = false;
                    t.error = None;
                    t.elapsed_ms = ms;
                }
            }
            Err(e) => {
                if let TabKind::Table(t) = &mut self.tabs[pos].kind {
                    t.pending = false;
                    t.error = Some(e);
                }
            }
        }
        // fetch FK metadata once per table (powers dropdown editing)
        if let TabKind::Table(t) = &self.tabs[pos].kind {
            if t.loaded && !t.fks_loaded {
                let conn = self.conns.get(&t.config_id).cloned();
                if let Some(conn) = conn {
                    let cfg_id = t.config_id;
                    let table = t.table.clone();
                    db::spawn_fetch_fks(conn, cfg_id, table);
                }
            }
        }
        if self.active_tab == Some(pos) {
            let mut t = match &self.tabs[pos].kind {
                TabKind::Table(t) => t.clone(),
                _ => return,
            };
            self.render_table_tab(cx, &mut t, false);
            self.ui.redraw(cx);
        }
    }

    // -- query execution ---------------------------------------------------------------

    fn run_active_query(&mut self, cx: &mut Cx) {
        let Some(pos) = self.active_tab else { return };
        self.sync_editor_into_tab(cx);
        let (tab_id, config_id, sql) = match &self.tabs[pos].kind {
            TabKind::Query(q) => (self.tabs[pos].id, q.config_id, q.sql.clone()),
            _ => return,
        };
        if sql.trim().is_empty() {
            self.push_status(cx, "Nothing to run — the editor is empty");
            return;
        }
        let Some(conn) = self.conns.get(&config_id).cloned() else {
            self.set_dot(cx, Dot::Err);
            self.push_status(cx, "⚠ The connection for this query tab is not open");
            return;
        };
        // history (dedupe consecutive duplicates), persisted across sessions
        if self.history.last().map(|s| s.as_str()) != Some(sql.as_str()) {
            self.history.push(sql.clone());
        }
        if self.history.len() > 100 {
            let excess = self.history.len() - 100;
            self.history.drain(0..excess);
        }
        self.history_idx = self.history.len();
        db::save_history(&self.history);
        if let TabKind::Query(q) = &mut self.tabs[pos].kind {
            q.pending = true;
        }
        self.set_dot(cx, Dot::Busy);
        self.push_status(cx, "Running…");
        let key = format!("q{}", tab_id);
        db::spawn_run_sql(conn, key, sql);
        self.ui.redraw(cx);
    }

    fn apply_sql_result(&mut self, cx: &mut Cx, key: &str, result: Result<QueryResult, String>) {
        // cell-edit write-back: optimistic value already applied; on failure
        // reload the page to restore server truth
        if let Some(_tab_id) = key.strip_prefix('u').and_then(|s| s.parse::<u64>().ok()) {
            match &result {
                Ok(r) if r.error.is_none() => match r.rows_affected {
                    Some(0) => {
                        self.set_dot(cx, Dot::Err);
                        self.push_status(cx, "⚠ Update matched 0 rows (stale key?)");
                        self.load_active_table(cx);
                    }
                    _ => {
                        self.set_dot(cx, Dot::Ok);
                        self.push_status(cx, &format!("✓ Updated · {:.1} ms", r.elapsed_ms));
                    }
                },
                other => {
                    let msg = match other {
                        Ok(r) => r.error.clone().unwrap_or_else(|| "unknown error".into()),
                        Err(e) => e.clone(),
                    };
                    self.set_dot(cx, Dot::Err);
                    self.push_status(cx, &format!("⚠ Update failed: {}", msg));
                    self.load_active_table(cx);
                }
            }
            self.ui.redraw(cx);
            return;
        }
        // blank-row insert: jump to the last page where the new row landed
        if let Some(_tab_id) = key.strip_prefix('n').and_then(|s| s.parse::<u64>().ok()) {
            match &result {
                Ok(r) if r.error.is_none() => {
                    self.set_dot(cx, Dot::Ok);
                    self.push_status(cx, &format!("✓ Row added · {:.1} ms", r.elapsed_ms));
                    let mut jumped = false;
                    if let Some(pos) = self.active_tab {
                        if let TabKind::Table(t) = &mut self.tabs[pos].kind {
                            let pages = ((t.total + 1) as usize).div_ceil(db::PAGE_SIZE).max(1);
                            t.page = pages - 1;
                            jumped = true;
                        }
                    }
                    if jumped {
                        self.load_active_table(cx);
                    }
                }
                other => {
                    let msg = match other {
                        Ok(r) => r.error.clone().unwrap_or_else(|| "unknown error".into()),
                        Err(e) => e.clone(),
                    };
                    self.set_dot(cx, Dot::Err);
                    self.push_status(
                        cx,
                        &format!("⚠ Add row failed: {} — try ⧉ Duplicate instead", msg),
                    );
                }
            }
            self.ui.redraw(cx);
            return;
        }
        // duplicate row: jump to the last page where the copy landed
        if let Some(_tab_id) = key.strip_prefix('i').and_then(|s| s.parse::<u64>().ok()) {
            match &result {
                Ok(r) if r.error.is_none() => {
                    self.set_dot(cx, Dot::Ok);
                    self.push_status(cx, &format!("✓ Row duplicated · {:.1} ms", r.elapsed_ms));
                    let mut jumped = false;
                    if let Some(pos) = self.active_tab {
                        if let TabKind::Table(t) = &mut self.tabs[pos].kind {
                            let pages = ((t.total + 1) as usize).div_ceil(db::PAGE_SIZE).max(1);
                            t.page = pages - 1;
                            jumped = true;
                        }
                    }
                    if jumped {
                        self.load_active_table(cx);
                    }
                }
                other => {
                    let msg = match other {
                        Ok(r) => r.error.clone().unwrap_or_else(|| "unknown error".into()),
                        Err(e) => e.clone(),
                    };
                    self.set_dot(cx, Dot::Err);
                    self.push_status(cx, &format!("⚠ Duplicate failed: {}", msg));
                }
            }
            self.ui.redraw(cx);
            return;
        }
        // row delete: refetch the page on success
        if let Some(_tab_id) = key.strip_prefix('d').and_then(|s| s.parse::<u64>().ok()) {
            match &result {
                Ok(r) if r.error.is_none() => {
                    self.set_dot(cx, Dot::Ok);
                    self.push_status(cx, &format!("✓ Row deleted · {:.1} ms", r.elapsed_ms));
                    self.load_active_table(cx);
                }
                other => {
                    let msg = match other {
                        Ok(r) => r.error.clone().unwrap_or_else(|| "unknown error".into()),
                        Err(e) => e.clone(),
                    };
                    self.set_dot(cx, Dot::Err);
                    self.push_status(cx, &format!("⚠ Delete failed: {}", msg));
                }
            }
            self.ui.redraw(cx);
            return;
        }
        let Some(tab_id) = key.strip_prefix('q').and_then(|s| s.parse::<u64>().ok()) else {
            return;
        };
        let pos = self
            .tabs
            .iter()
            .position(|t| t.id == tab_id && matches!(t.kind, TabKind::Query(_)));
        let Some(pos) = pos else { return };
        if let TabKind::Query(q) = &mut self.tabs[pos].kind {
            q.pending = false;
            q.result = Some(match result {
                Ok(r) => r,
                Err(e) => QueryResult {
                    error: Some(e),
                    ..Default::default()
                },
            });
        }
        if self.active_tab == Some(pos) {
            let q = match &self.tabs[pos].kind {
                TabKind::Query(q) => q.clone(),
                _ => return,
            };
            self.render_query_tab(cx, &q, false);
            self.ui.redraw(cx);
        }
    }

    // -- history navigation -----------------------------------------------------------

    fn history_nav(&mut self, cx: &mut Cx, older: bool) {
        if self.history.is_empty() {
            return;
        }
        if older {
            self.history_idx = self.history_idx.saturating_sub(1);
        } else {
            self.history_idx = (self.history_idx + 1).min(self.history.len());
        }
        let text = if self.history_idx < self.history.len() {
            self.history[self.history_idx].clone()
        } else {
            String::new()
        };
        self.ui.text_input(cx, ids!(sql_editor)).set_text(cx, &text);
        let pos = if self.history_idx < self.history.len() {
            self.history_idx + 1
        } else {
            self.history.len()
        };
        self.push_status(cx, &format!("History {}/{}", pos, self.history.len()));
    }

    // -- CSV export ---------------------------------------------------------------------

    fn export_csv(&mut self, cx: &mut Cx) {
        let Some(pos) = self.active_tab else { return };
        // gather (borrow-free) before touching the UI
        let data = match &self.tabs[pos].kind {
            TabKind::Table(t) => Some((
                t.table.clone(),
                t.columns.iter().map(|c| c.name.clone()).collect::<Vec<_>>(),
                t.rows.clone(),
            )),
            TabKind::Query(q) => {
                let tab_id = self.tabs[pos].id;
                q.result.as_ref().filter(|r| !r.rows.is_empty()).map(|r| {
                    (
                        format!("query-{}", tab_id),
                        r.columns.iter().map(|c| c.name.clone()).collect::<Vec<_>>(),
                        r.rows.clone(),
                    )
                })
            }
        };
        let Some((name, header, rows)) = data else {
            self.push_status(cx, "Nothing to export — run a query or load a table first");
            return;
        };
        if rows.is_empty() {
            self.push_status(cx, "Nothing to export — no rows");
            return;
        }
        let mut csv = String::new();
        csv.push_str(
            &header
                .iter()
                .map(|h| db::csv_cell(h))
                .collect::<Vec<_>>()
                .join(","),
        );
        csv.push('\n');
        for row in &rows {
            csv.push_str(
                &row.iter()
                    .map(|c| db::csv_cell(c))
                    .collect::<Vec<_>>()
                    .join(","),
            );
            csv.push('\n');
        }
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let path = format!("{}/dbpro-export-{}-{}.csv", home, name, compact_ts());
        match std::fs::write(&path, csv) {
            Ok(_) => {
                self.push_status(cx, &format!("✓ Exported {} rows → {}", rows.len(), path));
            }
            Err(e) => {
                self.set_dot(cx, Dot::Err);
                self.push_status(cx, &format!("⚠ Export failed: {}", e));
            }
        }
    }

    // -- connection dialog -----------------------------------------------------------------

    fn open_connect_dialog(&mut self, cx: &mut Cx, editing: Option<u64>) {
        self.dialog_editing = editing;
        self.dialog_password.clear();

        self.dialog_kind = DbKind::Sqlite;
        self.ui.mp_segmented(cx, ids!(dlg_kind)).set_segments(
            cx,
            DbKind::ALL.iter().map(|kind| kind.label().to_string()).collect(),
        );
        self.set_dialog_kind_label(cx, DbKind::Sqlite);

        if let Some(id) = editing {
            let cfg = self.configs.iter().find(|c| c.id == id).cloned();
            if let Some(cfg) = cfg {
                self.ui
                    .text_input(cx, ids!(dlg_name))
                    .set_text(cx, &cfg.name);
                self.dialog_kind = cfg.kind;
                self.set_dialog_kind_label(cx, cfg.kind);
                match cfg.kind {
                    DbKind::Sqlite => {
                        self.ui
                            .text_input(cx, ids!(dlg_path))
                            .set_text(cx, &cfg.path);
                    }
                    _ => {
                        self.ui
                            .text_input(cx, ids!(dlg_host))
                            .set_text(cx, &cfg.host);
                        self.ui
                            .text_input(cx, ids!(dlg_port))
                            .set_text(cx, &cfg.port.to_string());
                        self.ui
                            .text_input(cx, ids!(dlg_user))
                            .set_text(cx, &cfg.user);
                        self.ui
                            .text_input(cx, ids!(dlg_database))
                            .set_text(cx, &cfg.database);
                    }
                }
                self.ui
                    .text_input(cx, ids!(dlg_group))
                    .set_text(cx, &cfg.group);
                self.ui
                    .label(cx, ids!(dlg_title))
                    .set_text(cx, "Edit Connection");
            }
        } else {
            self.ui.text_input(cx, ids!(dlg_name)).set_text(cx, "");
            self.ui.text_input(cx, ids!(dlg_path)).set_text(cx, "");
            self.ui.text_input(cx, ids!(dlg_host)).set_text(cx, "");
            self.ui.text_input(cx, ids!(dlg_port)).set_text(cx, "");
            self.ui.text_input(cx, ids!(dlg_user)).set_text(cx, "");
            self.ui.text_input(cx, ids!(dlg_database)).set_text(cx, "");
            self.ui.text_input(cx, ids!(dlg_group)).set_text(cx, "");
            self.ui
                .label(cx, ids!(dlg_title))
                .set_text(cx, "New Connection");
        }
        // SSH tunnel fields (persisted, optional)
        let ssh = editing
            .and_then(|id| self.configs.iter().find(|c| c.id == id))
            .cloned();
        let (sh, su, sp) = match &ssh {
            Some(c) => (
                c.ssh_host.clone(),
                c.ssh_user.clone(),
                if c.ssh_port == 0 {
                    "22".to_string()
                } else {
                    c.ssh_port.to_string()
                },
            ),
            None => (String::new(), String::new(), "22".to_string()),
        };
        self.ui.text_input(cx, ids!(dlg_ssh_host)).set_text(cx, &sh);
        self.ui.text_input(cx, ids!(dlg_ssh_user)).set_text(cx, &su);
        self.ui.text_input(cx, ids!(dlg_ssh_port)).set_text(cx, &sp);
        self.ui
            .text_input(cx, ids!(dlg_password.input))
            .set_text(cx, "");
        let connected = editing
            .map(|id| self.conns.contains_key(&id))
            .unwrap_or(false);
        self.ui
            .widget(cx, ids!(dlg_disconnect))
            .set_visible(cx, connected);
        self.ui
            .widget(cx, ids!(dlg_delete))
            .set_visible(cx, editing.is_some());
        self.ui.label(cx, ids!(dlg_status)).set_text(cx, "");
        self.update_dialog_fields(cx);
        self.ui.mp_dialog(cx, ids!(connect_dialog)).open(cx);
        self.ui.redraw(cx);
    }

    fn set_dialog_kind_label(&self, cx: &mut Cx, kind: DbKind) {
        // The segmented control is highlighted by index rather than given text: its segments are set once from
        // `DbKind::ALL`, so "which kind" is a position and `label()` stays the word that is *stored*.
        self.ui
            .mp_segmented(cx, ids!(dlg_kind))
            .set_active(cx, Some(kind.index()));
    }

    fn update_dialog_fields(&mut self, cx: &mut Cx) {
        let is_sqlite = self.dialog_kind == DbKind::Sqlite;
        for id in [
            ids!(dlg_host_row),
            ids!(dlg_port_row),
            ids!(dlg_user_row),
            ids!(dlg_password_row),
            ids!(dlg_database_row),
        ] {
            self.ui.view(cx, id).set_visible(cx, !is_sqlite);
        }
        self.ui
            .view(cx, ids!(dlg_path_row))
            .set_visible(cx, is_sqlite);
        self.ui.redraw(cx);
    }

    fn dialog_connect(&mut self, cx: &mut Cx, test_only: bool) {
        let kind = self.dialog_kind;
        let name = self.ui.text_input(cx, ids!(dlg_name)).text();
        let name = if name.is_empty() {
            kind.label().to_string()
        } else {
            name
        };
        let group = self.ui.text_input(cx, ids!(dlg_group)).text();
        let path = self.ui.text_input(cx, ids!(dlg_path)).text();
        let host = self.ui.text_input(cx, ids!(dlg_host)).text();
        let port = self
            .ui
            .text_input(cx, ids!(dlg_port))
            .text()
            .parse::<u16>()
            .unwrap_or(kind.default_port());
        let user = self.ui.text_input(cx, ids!(dlg_user)).text();
        let database = self.ui.text_input(cx, ids!(dlg_database)).text();
        let password = self.ui.text_input(cx, ids!(dlg_password.input)).text();
        let ssh_host = self.ui.text_input(cx, ids!(dlg_ssh_host)).text();
        let ssh_user = self.ui.text_input(cx, ids!(dlg_ssh_user)).text();
        let ssh_port = self
            .ui
            .text_input(cx, ids!(dlg_ssh_port))
            .text()
            .parse::<u16>()
            .unwrap_or(22);

        if kind == DbKind::Sqlite && path.is_empty() {
            self.ui
                .label(cx, ids!(dlg_status))
                .set_text(cx, "⚠ Path required");
            return;
        }
        if kind != DbKind::Sqlite && host.is_empty() {
            self.ui
                .label(cx, ids!(dlg_status))
                .set_text(cx, "⚠ Host required");
            return;
        }

        let id = self.dialog_editing.unwrap_or_else(|| {
            let id = self.next_config_id;
            self.next_config_id += 1;
            id
        });
        let cfg = ConnectionConfig {
            id,
            name,
            kind,
            host,
            port,
            user,
            database,
            path,
            group,
            ssh_host,
            ssh_user,
            ssh_port,
        };
        if let Some(pos) = self.configs.iter().position(|c| c.id == id) {
            self.configs[pos] = cfg.clone();
        } else {
            self.configs.push(cfg.clone());
        }
        db::save_configs(&self.configs);
        self.rebuild_tree(cx);

        if test_only {
            self.ui
                .label(cx, ids!(dlg_status))
                .set_text(cx, "⏳ Testing…");
            db::spawn_test_connection(cfg, password);
        } else {
            self.ui.mp_dialog(cx, ids!(connect_dialog)).close(cx);
            self.push_status(cx, &format!("Connecting to {}…", cfg.name));
            self.set_dot(cx, Dot::Busy);
            self.ui.redraw(cx);
            db::spawn_connect(cfg, password);
        }
    }

    fn delete_connection(&mut self, cx: &mut Cx, id: u64) {
        self.configs.retain(|c| c.id != id);
        self.conns.remove(&id);
        self.tables.remove(&id);
        // close tabs that belonged to this connection
        let mut i = 0;
        while i < self.tabs.len() {
            let belongs = match &self.tabs[i].kind {
                TabKind::Table(t) => t.config_id == id,
                TabKind::Query(q) => q.config_id == id,
            };
            if belongs {
                self.tabs.remove(i);
            } else {
                i += 1;
            }
        }
        if self.tabs.is_empty() {
            self.activate_tab(cx, None);
        } else {
            let active = self.active_tab.map(|a| a.min(self.tabs.len() - 1));
            self.activate_tab(cx, active);
        }
        if self.active_config == Some(id) {
            self.active_config = self.configs.first().map(|c| c.id);
            self.update_conn_label(cx);
        }
        db::save_configs(&self.configs);
        self.rebuild_tree(cx);
        self.ui.redraw(cx);
    }

    /// Close the live connection for the active config. Tabs stay open but
    /// any refresh will report the connection as closed.
    fn disconnect_active(&mut self, cx: &mut Cx) {
        let Some(id) = self.active_config else { return };
        let name = self
            .configs
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.name.clone());
        if self.conns.remove(&id).is_some() {
            self.rebuild_tree(cx);
            self.set_dot(cx, Dot::Idle);
            if let Some(name) = name {
                self.push_status(cx, &format!("Disconnected from {}", name));
            }
            self.update_conn_label(cx);
        } else {
            self.push_status(cx, "Connection was not open");
        }
        self.ui.redraw(cx);
    }

    fn connect_saved(&mut self, cx: &mut Cx, id: u64) {
        let cfg = self.configs.iter().find(|c| c.id == id).cloned();
        if let Some(cfg) = cfg {
            if self.conns.contains_key(&id) {
                self.push_status(cx, &format!("Active: {}", cfg.name));
                return;
            }
            self.push_status(cx, &format!("Connecting to {}…", cfg.name));
            self.set_dot(cx, Dot::Busy);
            self.ui.redraw(cx);
            db::spawn_connect(cfg, String::new());
        }
    }

    // -- sidebar interaction ---------------------------------------------------------

    fn handle_tree_click(&mut self, cx: &mut Cx, idx: usize) {
        let Some(item) = self.sidebar_items.get(idx).cloned() else {
            return;
        };
        match item {
            SideItem::Conn(id) => {
                self.active_config = Some(id);
                self.update_conn_label(cx);
                if self.conns.contains_key(&id) {
                    let name = self
                        .configs
                        .iter()
                        .find(|c| c.id == id)
                        .map(|c| c.name.clone());
                    if let Some(name) = name {
                        self.push_status(cx, &format!("Active: {}", name));
                        self.set_dot(cx, Dot::Ok);
                    }
                } else {
                    self.connect_saved(cx, id);
                }
            }
            SideItem::Object(config_id, table) => {
                self.active_config = Some(config_id);
                self.update_conn_label(cx);
                self.open_table_tab(cx, config_id, &table);
            }
            SideItem::Nothing => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Grid helpers
// ---------------------------------------------------------------------------

/// Connection kind for a config id (borrow-free helper for edit builders).
fn cfg_kind(app: &App, config_id: u64) -> DbKind {
    app.configs
        .iter()
        .find(|c| c.id == config_id)
        .map(|c| c.kind)
        .unwrap_or(DbKind::Sqlite)
}

/// Compact timestamp for export filenames (no chrono dependency).
fn compact_ts() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{:x}", now)
}

// ---------------------------------------------------------------------------
// Event loop
// ---------------------------------------------------------------------------

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        makepad_widgets::script_mod(vm);
        makepad_component::script_mod(vm);
        crate::tab_bar::script_mod(vm);
        crate::grid::script_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        if let Event::Startup = event {
            self.configs = db::load_configs().unwrap_or_default();
            self.next_config_id = self.configs.iter().map(|c| c.id + 1).max().unwrap_or(1);

            if self.configs.is_empty() {
                let path = db::demo_db_path();
                db::ensure_demo_db(&path);
                self.configs.push(ConnectionConfig {
                    id: 0,
                    name: "Demo (SQLite)".to_string(),
                    kind: DbKind::Sqlite,
                    host: String::new(),
                    port: 0,
                    user: String::new(),
                    database: String::new(),
                    path,
                    group: String::new(),
                    ssh_host: String::new(),
                    ssh_user: String::new(),
                    ssh_port: 22,
                });
                self.next_config_id = 1;
                db::save_configs(&self.configs);
            }
            if let Some(mut host) = self.ui.widget(cx, ids!(db_grid)).borrow_mut::<DbGridHost>() {
                host.set_shared(self.grid_shared.clone());
            }
            self.history = db::load_history();
            self.history_idx = self.history.len();
            self.apply_mode_button_style(cx, false);
            self.rebuild_tree(cx);
            self.set_dot(cx, Dot::Idle);
            self.update_conn_label(cx);
            if let Some(first) = self.configs.first().cloned() {
                self.active_config = Some(first.id);
                db::spawn_connect(first, String::new());
            }
        }

        // actions posted from worker threads
        if let Event::Actions(actions) = event {
            for action in actions {
                if let Some(db_action) = action.downcast_ref::<DbAction>() {
                    match db_action {
                        DbAction::Connected {
                            config,
                            conn,
                            tables,
                        } => {
                            self.conns.insert(config.id, conn.clone());
                            self.tables.insert(config.id, tables.clone());
                            self.active_config = Some(config.id);
                            self.rebuild_tree(cx);
                            self.update_conn_label(cx);
                            self.set_dot(cx, Dot::Ok);
                            self.push_status(
                                cx,
                                &format!(
                                    "✓ Connected to {} · {} objects",
                                    config.name,
                                    tables.len()
                                ),
                            );
                            if self.tabs.is_empty() {
                                if let Some(first) = tables
                                    .iter()
                                    .find(|t| t.kind == TableKind::Table)
                                    .or_else(|| tables.first())
                                {
                                    let name = first.name.clone();
                                    let cfg_id = config.id;
                                    self.open_table_tab(cx, cfg_id, &name);
                                }
                            }
                            // background COUNT sweep for sidebar labels
                            if let Some(conn) = self.conns.get(&config.id).cloned() {
                                let names: Vec<String> =
                                    tables.iter().map(|t| t.name.clone()).collect();
                                db::spawn_count_tables(conn, config.id, names);
                            }
                            self.ui.redraw(cx);
                        }
                        DbAction::ConnectFailed { error, .. } => {
                            self.set_dot(cx, Dot::Err);
                            self.push_status(cx, &format!("✗ {}", error));
                            self.ui.redraw(cx);
                        }
                        DbAction::TableRows {
                            config_id,
                            table,
                            page,
                            search,
                            result,
                        } => {
                            self.apply_table_result(
                                cx,
                                *config_id,
                                table,
                                *page,
                                search,
                                result.clone(),
                            );
                        }
                        DbAction::SqlDone {
                            request_key,
                            result,
                        } => {
                            self.apply_sql_result(cx, request_key, result.clone());
                        }
                        DbAction::TableCounts { config_id, counts } => {
                            let map = counts
                                .iter()
                                .map(|(k, v)| (k.clone(), *v))
                                .collect::<HashMap<String, u64>>();
                            self.counts.insert(*config_id, map);
                            self.rebuild_tree(cx);
                            self.ui.redraw(cx);
                        }
                        DbAction::TableIndexes {
                            config_id,
                            table,
                            result,
                        } => {
                            let pos = self.tabs.iter().position(|t| match &t.kind {
                                TabKind::Table(tt) => {
                                    tt.config_id == *config_id && tt.table == *table
                                }
                                _ => false,
                            });
                            if let Some(pos) = pos {
                                if let TabKind::Table(t) = &mut self.tabs[pos].kind {
                                    t.pending_indexes = false;
                                    t.indexes = Some(result.clone().unwrap_or_default());
                                }
                                if self.active_tab == Some(pos) {
                                    let mut t = match &self.tabs[pos].kind {
                                        TabKind::Table(t) => t.clone(),
                                        _ => return,
                                    };
                                    if t.show_struct {
                                        self.render_struct(cx, &mut t);
                                        self.ui.redraw(cx);
                                    }
                                }
                            }
                        }
                        DbAction::ImportDone { rows, error } => {
                            match error {
                                None => {
                                    self.set_dot(cx, Dot::Ok);
                                    self.push_status(cx, &format!("✓ Imported {} rows", rows));
                                    // jump to the last page where new rows landed
                                    let mut jumped = false;
                                    if let Some(pos) = self.active_tab {
                                        if let TabKind::Table(t) = &mut self.tabs[pos].kind {
                                            let pages = ((t.total as usize + *rows)
                                                .div_ceil(db::PAGE_SIZE))
                                            .div_ceil(db::PAGE_SIZE)
                                            .max(1);
                                            t.page = pages - 1;
                                            jumped = true;
                                        }
                                    }
                                    if jumped {
                                        self.load_active_table(cx);
                                    }
                                }
                                Some(e) => {
                                    self.set_dot(cx, Dot::Err);
                                    self.push_status(cx, &format!("⚠ Import failed: {}", e));
                                }
                            }
                            self.ui.redraw(cx);
                        }
                        DbAction::ExportProgress { rows } => {
                            self.push_status(cx, &format!("Exporting… {} rows", rows));
                        }
                        DbAction::ExportDone { path, rows, error } => {
                            self.export_running = false;
                            match error {
                                None => {
                                    self.set_dot(cx, Dot::Ok);
                                    self.push_status(
                                        cx,
                                        &format!("✓ Exported {} rows → {}", rows, path),
                                    );
                                }
                                Some(e) => {
                                    if e.contains("cancelled") {
                                        self.set_dot(cx, Dot::Ok);
                                        self.push_status(
                                            cx,
                                            "■ Export cancelled — partial file removed",
                                        );
                                    } else {
                                        self.set_dot(cx, Dot::Err);
                                        self.push_status(cx, &format!("⚠ Export failed: {}", e));
                                    }
                                }
                            }
                            self.ui.redraw(cx);
                        }
                        DbAction::SchemaRefreshed { config_id, result } => {
                            match result {
                                Ok(tables) => {
                                    self.tables.insert(*config_id, tables.clone());
                                    self.push_status(
                                        cx,
                                        &format!("✓ Schema refreshed · {} objects", tables.len()),
                                    );
                                    if let Some(conn) = self.conns.get(config_id).cloned() {
                                        let names: Vec<String> =
                                            tables.iter().map(|t| t.name.clone()).collect();
                                        db::spawn_count_tables(conn, *config_id, names);
                                    }
                                }
                                Err(e) => {
                                    self.push_status(cx, &format!("⚠ Refresh failed: {}", e));
                                }
                            }
                            self.rebuild_tree(cx);
                            self.ui.redraw(cx);
                        }
                        DbAction::DdlFetched {
                            config_id,
                            table,
                            result,
                        } => {
                            let pos = self.tabs.iter().position(|t| match &t.kind {
                                TabKind::Table(tt) => {
                                    tt.config_id == *config_id && tt.table == *table
                                }
                                _ => false,
                            });
                            if let Some(pos) = pos {
                                if let TabKind::Table(t) = &mut self.tabs[pos].kind {
                                    t.pending_ddl = false;
                                    t.ddl = Some(match result {
                                        Ok(ddl) => ddl.clone(),
                                        Err(e) => format!("-- {}", e),
                                    });
                                }
                                if self.active_tab == Some(pos) {
                                    let mut t = match &self.tabs[pos].kind {
                                        TabKind::Table(t) => t.clone(),
                                        _ => return,
                                    };
                                    if t.show_struct {
                                        self.render_struct(cx, &mut t);
                                        self.ui.redraw(cx);
                                    }
                                }
                            }
                        }
                        DbAction::FksFetched {
                            config_id,
                            table,
                            result,
                        } => {
                            let pos = self.tabs.iter().position(|t| match &t.kind {
                                TabKind::Table(tt) => {
                                    tt.config_id == *config_id && tt.table == *table
                                }
                                _ => false,
                            });
                            if let Some(pos) = pos {
                                if let TabKind::Table(t) = &mut self.tabs[pos].kind {
                                    t.fks_loaded = true;
                                    t.fks = result.clone().unwrap_or_default();
                                }
                            }
                        }
                        DbAction::FkOptionsFetched { options } => {
                            if let Some((row, col)) = self.pending_fk_edit.take() {
                                if options.is_empty() {
                                    self.set_dot(cx, Dot::Err);
                                    self.push_status(cx, "⚠ No options for this foreign key");
                                } else {
                                    let labels: Vec<String> =
                                        options.iter().map(|(_, d)| d.clone()).collect();
                                    self.fk_commit_values =
                                        options.iter().map(|(v, _)| v.clone()).collect();
                                    self.ui.db_grid_host(cx, ids!(db_grid)).start_fk_edit(
                                        cx,
                                        row,
                                        col,
                                        labels,
                                        self.fk_commit_values.clone(),
                                    );
                                }
                            }
                            self.ui.redraw(cx);
                        }
                        DbAction::TestResult { ok, message } => {
                            let prefix = if *ok { "✓" } else { "✗" };
                            self.ui
                                .label(cx, ids!(dlg_status))
                                .set_text(cx, &format!("{} {}", prefix, message));
                            self.ui.redraw(cx);
                        }
                    }
                }
            }
        }

        // debounced search: fire when the timer elapses
        if self.search_timer.is_event(event).is_some() {
            let text = self.ui.text_input(cx, ids!(search_input)).text();
            let mut do_load = false;
            if let Some(pos) = self.active_tab {
                let changed = match &self.tabs[pos].kind {
                    TabKind::Table(t) => t.search != text,
                    _ => false,
                };
                if changed {
                    if let TabKind::Table(t) = &mut self.tabs[pos].kind {
                        t.search = text;
                        t.page = 0;
                    }
                    do_load = true;
                }
            }
            if do_load {
                self.load_active_table(cx);
            }
        }

        // keyboard shortcuts
        if let Event::KeyDown(ke) = event {
            if !ke.is_repeat {
                let primary = ke.modifiers.is_primary();
                let kind = self.active_kind();
                match ke.key_code {
                    KeyCode::KeyR if primary => match kind {
                        ActiveKind::Table => self.load_active_table(cx),
                        ActiveKind::Query => self.run_active_query(cx),
                        ActiveKind::None => {}
                    },
                    KeyCode::ArrowUp if primary && kind == ActiveKind::Query => {
                        self.history_nav(cx, true);
                    }
                    KeyCode::ArrowDown if primary && kind == ActiveKind::Query => {
                        self.history_nav(cx, false);
                    }
                    KeyCode::Escape if self.ui.mp_dialog(cx, ids!(connect_dialog)).is_open() => {
                        self.ui.mp_dialog(cx, ids!(connect_dialog)).close(cx);
                    }
                    KeyCode::Escape if self.ui.mp_dialog(cx, ids!(confirm_dialog)).is_open() => {
                        self.ui.mp_dialog(cx, ids!(confirm_dialog)).close(cx);
                        self.pending_delete = None;
                    }
                    _ => {}
                }
            }
        }

        // UI actions
        let actions = cx.capture_actions(|cx| {
            self.ui.handle_event(cx, event, &mut Scope::empty());
        });
        self.handle_actions(cx, &actions);
    }
}

impl App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        // ── sidebar ──
        if let Some(idx) = self
            .ui
            .mp_tree(cx, ids!(sidebar_tree))
            .row_selected(actions)
        {
            self.handle_tree_click(cx, idx);
        }
        if self.ui.mp_button(cx, ids!(new_conn_btn)).clicked(actions) {
            self.open_connect_dialog(cx, None);
        }
        if self.ui.mp_button(cx, ids!(edit_conn_btn)).clicked(actions) {
            let editing = self.active_config;
            self.open_connect_dialog(cx, editing);
        }
        if self.ui.mp_button(cx, ids!(disconnect_btn)).clicked(actions) {
            self.disconnect_active(cx);
        }

        // ── tab bar ──
        let tab_bar = self.ui.db_tab_bar(cx, ids!(tab_bar));
        if let Some(idx) = tab_bar.selected(actions) {
            if idx < self.tabs.len() && self.active_tab != Some(idx) {
                self.sync_editor_into_tab(cx);
                self.activate_tab(cx, Some(idx));
            }
        }
        if let Some(idx) = tab_bar.closed(actions) {
            if idx < self.tabs.len() {
                let was_active = self.active_tab == Some(idx);
                self.tabs.remove(idx);
                if self.tabs.is_empty() {
                    self.activate_tab(cx, None);
                } else if was_active {
                    let next = idx.min(self.tabs.len() - 1);
                    self.activate_tab(cx, Some(next));
                } else if let Some(a) = self.active_tab {
                    if a > idx {
                        self.activate_tab(cx, Some(a - 1));
                    } else {
                        self.sync_tab_bar(cx);
                    }
                }
                self.ui.redraw(cx);
            }
        }
        if tab_bar.add_clicked(actions) {
            self.open_query_tab(cx);
        }

        // ── toolbar ──
        if self.ui.mp_button(cx, ids!(new_query_btn)).clicked(actions) {
            self.open_query_tab(cx);
        }

        // ── sidebar filter ──
        if let Some(text) = self
            .ui
            .text_input(cx, ids!(sidebar_filter_input))
            .changed(actions)
        {
            let lowered = text.to_lowercase();
            if self.sidebar_filter != lowered {
                self.sidebar_filter = lowered;
                self.rebuild_tree(cx);
                self.ui.redraw(cx);
            }
        }

        // ── table toolbar ──
        // search input: restart the debounce timer on every keystroke
        if self
            .ui
            .text_input(cx, ids!(search_input))
            .changed(actions)
            .is_some()
        {
            self.search_timer = cx.start_timeout(0.4);
        }
        // paging
        let prev_clicked = self.ui.mp_button(cx, ids!(page_prev)).clicked(actions);
        let next_clicked = self.ui.mp_button(cx, ids!(page_next)).clicked(actions);
        if prev_clicked || next_clicked {
            let delta: isize = if prev_clicked { -1 } else { 1 };
            let mut do_load = false;
            if let Some(pos) = self.active_tab {
                if let TabKind::Table(t) = &mut self.tabs[pos].kind {
                    let pages = (t.total as usize).div_ceil(db::PAGE_SIZE).max(1);
                    let new_page = t.page as isize + delta;
                    if new_page >= 0 && (new_page as usize) < pages {
                        t.page = new_page as usize;
                        do_load = true;
                    }
                }
            }
            if do_load {
                self.load_active_table(cx);
            }
        }
        if self.ui.mp_button(cx, ids!(refresh_btn)).clicked(actions)
            && self.active_kind() == ActiveKind::Table
        {
            self.load_active_table(cx);
        }
        if self
            .ui
            .mp_button(cx, ids!(grid_export_btn))
            .clicked(actions)
        {
            self.export_csv(cx);
        }
        if self.ui.mp_button(cx, ids!(detail_btn)).clicked(actions) {
            self.show_row_detail_dialog(cx);
        }
        if self.ui.mp_button(cx, ids!(detail_close)).clicked(actions) {
            self.ui.mp_dialog(cx, ids!(detail_dialog)).close(cx);
        }
        if self.ui.mp_button(cx, ids!(dup_row_btn)).clicked(actions) {
            self.duplicate_selected_row(cx);
        }
        if self.ui.mp_button(cx, ids!(add_row_btn)).clicked(actions) {
            self.insert_default_row(cx);
        }
        if self.ui.mp_button(cx, ids!(export_all_btn)).clicked(actions) {
            self.export_whole_table(cx);
        }
        if self
            .ui
            .mp_button(cx, ids!(refresh_schema_btn))
            .clicked(actions)
        {
            self.refresh_schema(cx);
        }

        // Data / Structure mode toggle
        let data_clicked = self.ui.mp_button(cx, ids!(mode_data_on)).clicked(actions)
            || self.ui.mp_button(cx, ids!(mode_data_off)).clicked(actions);
        let struct_clicked = self.ui.mp_button(cx, ids!(mode_struct_on)).clicked(actions)
            || self
                .ui
                .mp_button(cx, ids!(mode_struct_off))
                .clicked(actions);
        if data_clicked || struct_clicked {
            let want_struct = struct_clicked;
            let mut render: Option<TableTab> = None;
            if let Some(pos) = self.active_tab {
                if let TabKind::Table(t) = &mut self.tabs[pos].kind {
                    if t.show_struct != want_struct {
                        t.show_struct = want_struct;
                        render = Some(t.clone());
                    }
                }
            }
            if let Some(mut t) = render {
                self.render_table_tab(cx, &mut t, false);
                self.ui.redraw(cx);
            }
        }

        // ── data grid (DataGrid): editing, sort, selection ──
        let grid_ref = self.ui.data_grid(cx, ids!(db_grid.grid));
        for action in grid_ref.actions(actions) {
            match action {
                DataGridAction::EditCell { row, col, replace } => {
                    self.start_cell_edit(cx, row, col, replace);
                }
                DataGridAction::CellDoubleClicked { row, col } => {
                    self.start_cell_edit(cx, row, col, None);
                }
                DataGridAction::CellClicked { row, col, .. } => {
                    // clicking anywhere while editing commits the live editor
                    let host = self.ui.db_grid_host(cx, ids!(db_grid));
                    if let Some((er, ec)) = host.editing_cell() {
                        let text = host.live_editor_text(cx).unwrap_or_default();
                        self.commit_cell_edit(cx, er, ec, &text);
                    }
                    self.last_grid_row = Some(row);
                    self.show_row_detail(cx, row, col);
                }
                DataGridAction::HeaderClicked { col, .. } => {
                    let mut do_load = false;
                    if let Some(pos) = self.active_tab {
                        if let TabKind::Table(t) = &mut self.tabs[pos].kind {
                            let name = t.columns.get(col).map(|c| c.name.clone());
                            if let Some(name) = name {
                                if t.order_col.as_ref() == Some(&name) {
                                    if t.order_asc {
                                        t.order_asc = false;
                                    } else {
                                        t.order_col = None;
                                    }
                                } else {
                                    t.order_col = Some(name);
                                    t.order_asc = true;
                                }
                                t.page = 0;
                                do_load = true;
                            }
                        }
                    }
                    if do_load {
                        self.load_active_table(cx);
                    }
                }
                DataGridAction::ClearCells => {
                    // Delete on a single selected cell sets it to NULL
                    let sel = self.ui.data_grid(cx, ids!(db_grid.grid)).selection();
                    if let Some(sel) = sel {
                        if sel.kind == GridSelectKind::Cells && sel.anchor == sel.head {
                            let (row, col) = sel.anchor;
                            self.commit_cell_edit(cx, row, col, "");
                        } else {
                            self.push_status(cx, "Select a single cell to set NULL");
                        }
                    }
                }
                DataGridAction::SelectionChanged {
                    selection: Some(sel),
                } => {
                    let (r0, r1) = sel.row_range();
                    if r0 == r1 {
                        self.last_grid_row = Some(r0);
                    }
                }
                _ => {}
            }
        }
        // per-cell editor commit / cancel
        for (_row, _col, widget) in grid_ref.cell_widgets_with_actions(actions) {
            let ti = widget.as_text_input();
            let dd = widget.as_drop_down();
            if let Some(idx) = dd.selected(actions) {
                // FK dropdown pick
                let host = self.ui.db_grid_host(cx, ids!(db_grid));
                if let Some((er, ec)) = host.editing_cell() {
                    let value = host.fk_value_at(idx);
                    self.commit_cell_edit(cx, er, ec, value.as_deref().unwrap_or(""));
                }
            } else if let Some((text, _mods)) = ti.returned(actions) {
                let host = self.ui.db_grid_host(cx, ids!(db_grid));
                if let Some((er, ec)) = host.editing_cell() {
                    self.commit_cell_edit(cx, er, ec, &text);
                }
            } else if ti.escaped(actions) {
                self.ui.db_grid_host(cx, ids!(db_grid)).end_edit(cx);
                let grid_area = self.ui.data_grid(cx, ids!(db_grid.grid)).area();
                cx.set_key_focus(grid_area);
                self.push_status(cx, "Edit cancelled");
            }
        }
        // ── row delete ──
        if self.ui.mp_button(cx, ids!(del_row_btn)).clicked(actions) {
            self.delete_selected_row(cx);
        }
        // ── csv import ──
        if self.ui.mp_button(cx, ids!(import_csv_btn)).clicked(actions) {
            let table = match self.active_tab_ref().map(|t| &t.kind) {
                Some(TabKind::Table(t)) => t.table.clone(),
                _ => String::new(),
            };
            if table.is_empty() {
                self.push_status(cx, "⬆ CSV imports into the open table");
            } else {
                self.import_table = table;
                if let Some(cfg) = self
                    .active_config
                    .and_then(|id| self.configs.iter().find(|c| c.id == id))
                {
                    self.ui
                        .label(cx, ids!(import_title))
                        .set_text(cx, &format!("Import CSV → {}", cfg.name));
                }
                self.ui.text_input(cx, ids!(import_path)).set_text(cx, "");
                self.ui.mp_dialog(cx, ids!(import_dialog)).open(cx);
                self.ui.redraw(cx);
            }
        }
        if self.ui.mp_button(cx, ids!(import_cancel)).clicked(actions) {
            self.ui.mp_dialog(cx, ids!(import_dialog)).close(cx);
        }
        if self.ui.mp_button(cx, ids!(import_go)).clicked(actions) {
            let table = self.import_table.clone();
            let path = self.ui.text_input(cx, ids!(import_path)).text();
            if table.is_empty() {
                return;
            }
            if path.is_empty() {
                self.ui
                    .label(cx, ids!(import_status))
                    .set_text(cx, "⚠ File path required");
                return;
            }
            self.ui.mp_dialog(cx, ids!(import_dialog)).close(cx);
            let Some(pos) = self.active_tab else { return };
            let config_id = match &self.tabs[pos].kind {
                TabKind::Table(t) => t.config_id,
                _ => return,
            };
            let Some(conn) = self.conns.get(&config_id).cloned() else {
                self.push_status(cx, "⚠ Connection is not open");
                return;
            };
            self.set_dot(cx, Dot::Busy);
            self.push_status(cx, &format!("Importing {} → {}…", path, table));
            db::spawn_import_csv(conn, table, path);
            self.ui.redraw(cx);
        }

        // ── query editor ──
        if self.ui.mp_button(cx, ids!(run_btn)).clicked(actions) {
            self.run_active_query(cx);
        }
        // ⌘/Ctrl+Enter inside the editor emits `Returned`
        if let Some((text, mods)) = self.ui.text_input(cx, ids!(sql_editor)).returned(actions) {
            if mods.is_primary()
                && self.active_kind() == ActiveKind::Query
                && !text.trim().is_empty()
            {
                self.run_active_query(cx);
            }
        }
        for (sample_id, sql) in [
            (ids!(sample_select), "SELECT * FROM users LIMIT 10;"),
            (
                ids!(sample_join),
                "SELECT o.id, u.name, p.name, o.quantity, o.total\nFROM orders o\nJOIN users u ON u.id = o.user_id\nJOIN products p ON p.id = o.product_id\nLIMIT 20;",
            ),
            (
                ids!(sample_agg),
                "SELECT city, COUNT(*) AS users, ROUND(AVG(balance), 2) AS avg_balance\nFROM users\nGROUP BY city\nORDER BY users DESC;",
            ),
        ] {
            if self.ui.mp_button(cx, sample_id).clicked(actions) {
                self.ui.text_input(cx, ids!(sql_editor)).set_text(cx, sql);
                if let Some(pos) = self.active_tab {
                    if let TabKind::Query(q) = &mut self.tabs[pos].kind {
                        q.sql = sql.to_string();
                    }
                }
                self.run_active_query(cx);
            }
        }
        if self
            .ui
            .mp_button(cx, ids!(query_export_btn))
            .clicked(actions)
        {
            self.export_csv(cx);
        }

        // ── connect dialog ──
        // **A segmented control rather than the v2 select.** Three mutually exclusive kinds is what a segmented control
        // is for, and it needs no floating panel — which matters because v3's select trigger is a DSL alias of makepad's
        // `RoundedView` with no Rust type, and because the v2 select declared its options as DSL children where the v3
        // library takes them as data. The *index* it reports is mapped through `DbKind::ALL`, one list, so the segments
        // and the mapping cannot disagree.
        if let Some(index) = self.ui.mp_segmented(cx, ids!(dlg_kind)).selected(actions) {
            if let Some(kind) = DbKind::from_index(index) {
                self.dialog_kind = kind;
                self.update_dialog_fields(cx);
            }
        }
        if self.ui.mp_button(cx, ids!(dlg_cancel)).clicked(actions) {
            self.ui.mp_dialog(cx, ids!(connect_dialog)).close(cx);
        }
        if self.ui.mp_button(cx, ids!(dlg_test)).clicked(actions) {
            self.dialog_connect(cx, true);
        }
        if self.ui.mp_button(cx, ids!(dlg_connect)).clicked(actions) {
            self.dialog_connect(cx, false);
        }
        if self.ui.mp_button(cx, ids!(dlg_delete)).clicked(actions) {
            if let Some(id) = self.dialog_editing {
                self.ui.mp_dialog(cx, ids!(connect_dialog)).close(cx);
                self.delete_connection(cx, id);
            }
        }
        // ── disconnect from within the edit dialog ──
        if self.ui.mp_button(cx, ids!(dlg_disconnect)).clicked(actions) {
            if let Some(id) = self.dialog_editing {
                self.active_config = Some(id);
                self.ui.mp_dialog(cx, ids!(connect_dialog)).close(cx);
                self.disconnect_active(cx);
            }
        }
        // ── delete confirmation dialog ──
        if self.ui.mp_button(cx, ids!(confirm_cancel)).clicked(actions) {
            self.ui.mp_dialog(cx, ids!(confirm_dialog)).close(cx);
            self.pending_delete = None;
        }
        if self.ui.mp_button(cx, ids!(confirm_ok)).clicked(actions) {
            self.ui.mp_dialog(cx, ids!(confirm_dialog)).close(cx);
            self.run_pending_delete(cx);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod sidebar_tests {
    use super::*;

    fn cfg(id: u64, name: &str, group: &str) -> ConnectionConfig {
        ConnectionConfig {
            id,
            name: name.into(),
            kind: DbKind::Sqlite,
            host: String::new(),
            port: 0,
            user: String::new(),
            database: String::new(),
            path: format!("{}.db", name),
            group: group.into(),
            ssh_host: String::new(),
            ssh_user: String::new(),
            ssh_port: 22,
        }
    }

    fn table(name: &str) -> TableInfo {
        TableInfo {
            name: name.into(),
            kind: TableKind::Table,
        }
    }

    #[test]
    fn test_sidebar_ungrouped() {
        let configs = vec![cfg(1, "A", ""), cfg(2, "B", "")];
        let mut tables = HashMap::new();
        tables.insert(1, vec![table("users")]);
        let connected: std::collections::HashSet<u64> = [1u64].into_iter().collect();
        let empty = HashMap::new();
        let (items, side) = build_sidebar_model(&configs, &tables, &empty, &connected, "");
        assert_eq!(items[0].label, "◉ A");
        assert_eq!(items[1].label, "users");
        assert_eq!(items[2].label, "○ B");
        assert_eq!(side[0], SideItem::Conn(1));
        assert_eq!(side[1], SideItem::Object(1, "users".into()));
        assert_eq!(side[2], SideItem::Conn(2));
    }

    #[test]
    fn test_sidebar_grouping_counts_and_filter() {
        let configs = vec![cfg(1, "prod", "Work"), cfg(2, "local", "")];
        let mut tables = HashMap::new();
        tables.insert(1, vec![table("users"), table("orders")]);
        let mut counts: HashMap<u64, HashMap<String, u64>> = HashMap::new();
        let mut m: HashMap<String, u64> = HashMap::new();
        m.insert("users".to_string(), 48u64);
        counts.insert(1, m);
        let connected: std::collections::HashSet<u64> = std::collections::HashSet::new();

        let (items, side) = build_sidebar_model(&configs, &tables, &counts, &connected, "");
        assert_eq!(items[0].label, "📁 Work");
        assert_eq!(items[1].label, "○ prod");
        assert_eq!(items[2].label, "users (48)");
        assert_eq!(items[3].label, "orders");
        assert!(items.iter().any(|i| i.label == "○ local"));
        assert_eq!(side[0], SideItem::Nothing);
        assert_eq!(side[1], SideItem::Conn(1));

        // filter narrows tables but keeps connections
        let (items2, _) = build_sidebar_model(&configs, &tables, &counts, &connected, "USE");
        assert!(items2.iter().any(|i| i.label.contains("users")));
        assert!(!items2.iter().any(|i| i.label.contains("orders")));
        assert!(items2.iter().any(|i| i.label == "○ local"));
    }

    #[test]
    fn test_sidebar_empty_and_no_tables() {
        let (items, side) = build_sidebar_model(
            &[],
            &HashMap::new(),
            &HashMap::new(),
            &std::collections::HashSet::new(),
            "",
        );
        assert!(items.is_empty() && side.is_empty());

        let configs = vec![cfg(3, "empty", "")];
        let connected: std::collections::HashSet<u64> = [3u64].into_iter().collect();
        let (items, _) =
            build_sidebar_model(&configs, &HashMap::new(), &HashMap::new(), &connected, "");
        // connected root with an empty schema shows the hint
        assert_eq!(items[0].label, "◉ empty");
        assert_eq!(items[1].label, "(no tables)");
    }
}

// ---------------------------------------------------------------------------
// UI definition (Makepad 2.0 script_mod DSL)
// ---------------------------------------------------------------------------

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*
    use mod.theme.*

    mod.db_theme = {
        bg: #x141417ff
        sidebar: #x191a1fff
        panel: #x1e1f25ff
        border: #x2a2b33ff
        text: #xe2e2e6ff
        text_muted: #x8a8a92ff
        text_faint: #x5a5a64ff
        accent: #x4f8cc9ff
        accent_dim: #x2c4a6bff
        ok_c: #x4caf6eff
        err_c: #xe05f65ff
        warn_c: #xe0b04fff
    }

    // Runtime-mutable UI state (updated from Rust via the script heap).
    mod.db_state = {
        status_color: #x5a5a64ff
    }

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                show_bg: true
                width: Fill
                height: Fill
                window.inner_size: vec2(1360 860)
                window.title: "DbPro — Database GUI"

                body := mod.widgets.SolidView{
                    width: Fill
                    height: Fill
                    flow: Down
                    spacing: 0
                    draw_bg +: {
                        color: mod.db_theme.bg
                        pixel: fn() {
                            return self.color
                        }
                    }

                    // ── Toolbar ──
                    toolbar := View{
                        width: Fill
                        height: 46
                        flow: Right
                        spacing: 8
                        align: Align{x: 0.0, y: 0.5}
                        padding: Inset{left: 14, right: 14, top: 0, bottom: 0}

                        show_bg: true
                        draw_bg +: {
                            color: mod.db_theme.panel
                            pixel: fn() { return self.color }
                        }

                        app_title := Label{
                            text: "⚡ DbPro"
                            draw_text +: {
                                text_style: theme.font_bold{font_size: 15.0}
                                color: mod.db_theme.text
                            }
                        }

                        status_dot := View{
                            width: 9
                            height: 9
                            margin: Inset{left: 8}
                            show_bg: true
                            draw_bg +: {
                                color: mod.db_state.status_color
                                pixel: fn() {
                                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                                    sdf.circle(self.rect_size.x * 0.5, self.rect_size.y * 0.5, self.rect_size.x * 0.5)
                                    sdf.fill(self.color)
                                    return sdf.result
                                }
                            }
                        }

                        status_label := Label{
                            text: "Starting…"
                            draw_text +: {
                                text_style: theme.font_regular{font_size: 12.0}
                                color: mod.db_theme.text_muted
                            }
                        }

                        View{width: Fill, height: 1}

                        new_query_btn := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost, text: "＋ Query" }
                        new_conn_btn := mod.mp.MpButton{style: mod.mp.ButtonStyle.Prominent, text: "＋ Connection" }
                    }

                    // hairline under toolbar
                    toolbar_divider := View{
                        width: Fill
                        height: 1
                        show_bg: true
                        draw_bg +: {
                            color: mod.db_theme.border
                            pixel: fn() { return self.color }
                        }
                    }

                    // ── Main split: sidebar / content ──
                    main_split := mod.mp.MpSplitPane{
                        width: Fill
                        height: Fill

                        left := View{
                            width: 264
                            height: Fill
                            flow: Down
                            show_bg: true
                            draw_bg +: {
                                color: mod.db_theme.sidebar
                                pixel: fn() { return self.color }
                            }

                            sidebar_header := View{
                                width: Fill
                                height: 34
                                flow: Right
                                align: Align{x: 0.0, y: 0.5}
                                padding: Inset{left: 14, right: 8}

                                sidebar_title := Label{
                                    text: "CONNECTIONS"
                                    draw_text +: {
                                        text_style: theme.font_bold{font_size: 10.5}
                                        color: mod.db_theme.text_faint
                                    }
                                }
                                View{width: Fill, height: 1}
                                refresh_schema_btn := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost,
                                    text: "Sync"
                                    draw_text +: {
                                        text_style: theme.font_regular{font_size: 13.0}
                                        color: mod.db_theme.text_faint
                                    }
                                }
                                edit_conn_btn := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost,
                                    text: "⚙"
                                    draw_text +: {
                                        text_style: theme.font_regular{font_size: 13.0}
                                        color: mod.db_theme.text_faint
                                    }
                                }
                            }

                            sidebar_filter := View{
                                width: Fill
                                height: Fit
                                padding: Inset{left: 10, right: 10, top: 0, bottom: 6}

                                sidebar_filter_input := mod.mp.MpTextInput{
                                    width: Fill
                                    height: Fit
                                    empty_text: "Filter tables…"
                                }
                            }

                            sidebar_scroll := ScrollYView{
                                width: Fill
                                height: Fill

                                sidebar_tree := mod.mp.MpTree{
                                    width: Fill
                                    height: Fit
                                }
                            }
                        }

                        right := View{
                            width: Fill
                            height: Fill
                            flow: Down

                            // ── Document tabs ──
                            tab_bar := mod.widgets.DbTabBar{
                                width: Fill
                                height: 38
                            }

                            tabs_divider := View{
                                width: Fill
                                height: 1
                                show_bg: true
                                draw_bg +: {
                                    color: mod.db_theme.border
                                    pixel: fn() { return self.color }
                                }
                            }

                            // ── Content ──
                            content := View{
                                width: Fill
                                height: Fill
                                flow: Down
                                padding: Inset{left: 12, right: 12, top: 10, bottom: 10}
                                spacing: 8

                                // ── table browser view ──
                                table_view := View{
                                    width: Fill
                                    height: Fill
                                    flow: Down
                                    spacing: 8
                                    visible: false

                                    table_toolbar := View{
                                        width: Fill
                                        height: Fit
                                        flow: Down
                                        spacing: 6

                                        // row 1: mode + filter + paging + reload
                                        toolbar_row1 := View{
                                            width: Fill
                                            height: Fit
                                            flow: Right
                                            spacing: 8
                                            align: Align{x: 0.0, y: 0.5}

                                            mode_data_on_wrap := View{
                                                width: Fit, height: Fit
                                                mode_data_on := mod.mp.MpButton{style: mod.mp.ButtonStyle.Default, text: "◉ Data" }
                                            }
                                            mode_struct_off_wrap := View{
                                                width: Fit, height: Fit
                                                mode_struct_off := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost, text: "○ Struct" }
                                            }
                                            mode_data_off_wrap := View{
                                                width: Fit, height: Fit
                                                visible: false
                                                mode_data_off := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost, text: "◉ Data" }
                                            }
                                            mode_struct_on_wrap := View{
                                                width: Fit, height: Fit
                                                visible: false
                                                mode_struct_on := mod.mp.MpButton{style: mod.mp.ButtonStyle.Default, text: "◉ Struct" }
                                            }

                                            View{width: 8, height: 1}

                                            search_input := mod.mp.MpTextInput{
                                                width: 240
                                                height: Fit
                                                empty_text: "Filter rows…"
                                            }

                                            page_prev := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost,
                                                width: 30
                                                text: "◀"
                                            }
                                            pager_label := Label{
                                                text: "Page 1/1 · 0 rows"
                                                margin: Inset{left: 4, right: 4}
                                                draw_text +: {
                                                    text_style: theme.font_regular{font_size: 12.0}
                                                    color: mod.db_theme.text_muted
                                                }
                                            }
                                            page_next := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost,
                                                width: 30
                                                text: "▶"
                                            }

                                            View{width: Fill, height: 1}

                                            refresh_btn := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost, text: "Reload" }
                                        }

                                        // row 2: row ops + import/export
                                        toolbar_row2 := View{
                                            width: Fill
                                            height: Fit
                                            flow: Right
                                            spacing: 8
                                            align: Align{x: 0.0, y: 0.5}

                                            add_row_btn := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost, text: "＋ Row" }
                                            del_row_btn := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost, text: "－ Row" }
                                            dup_row_btn := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost, text: "⧉ Duplicate" }
                                            detail_btn := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost, text: "Detail" }

                                            View{width: Fill, height: 1}

                                            import_csv_btn := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost, text: "⬆ Import" }
                                            grid_export_btn := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost, text: "⬇ CSV" }
                                            export_all_btn := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost, text: "⬇ All" }
                                        }
                                    }

                                    // data grid (paged rows, editable via
                                    // the `Editor` cell template below)
                                    data_view := View{
                                        width: Fill
                                        height: Fill
                                        flow: Down
                                        visible: true

                                        db_grid := mod.widgets.DbGridHost{
                                            width: Fill
                                            height: Fill

                                            grid := DataGrid{
                                                width: Fill
                                                height: Fill
                                                default_row_height: 27.0
                                                col_header_height: 30.0
                                                row_header_width: 44.0
                                                show_row_headers: true
                                                zebra_stripes: true
                                                cell_pad_x: 8.0

                                                color_bg: mod.db_theme.bg
                                                color_cell: mod.db_theme.bg
                                                color_cell_alt: #x1c1d23ff
                                                color_text: mod.db_theme.text
                                                color_header: mod.db_theme.panel
                                                color_header_active: mod.db_theme.accent_dim
                                                color_header_text: mod.db_theme.text_muted
                                                color_selection: #x4f8cc92e
                                                color_selection_border: mod.db_theme.accent
                                                color_drag_marker: mod.db_theme.accent
                                                color_resize_guide: mod.db_theme.accent

                                                draw_text +: {
                                                    text_style: theme.font_regular{font_size: 12.0}
                                                    color: mod.db_theme.text
                                                }

                                                // inline cell editor template
                                                Editor := TextInput{
                                                    width: Fill
                                                    height: Fill
                                                    padding: Inset{left: 6, right: 4, top: 2, bottom: 2}
                                                    draw_bg +: {
                                                        border_size: uniform(2.0)
                                                        border_radius: uniform(0.0)
                                                        color: (INPUT_BG)
                                                        color_hover: (INPUT_BG)
                                                        color_focus: (INPUT_BG)
                                                        color_down: (INPUT_BG)
                                                        color_empty: (INPUT_BG)
                                                        border_color: (ACCENT)
                                                        border_color_hover: (ACCENT)
                                                        border_color_focus: (ACCENT)
                                                        border_color_down: (ACCENT)
                                                        border_color_empty: (ACCENT)
                                                    }
                                                    draw_text +: {
                                                        text_style: theme.font_regular{font_size: 12.0}
                                                        color: TEXT
                                                        color_empty: TEXT
                                                        color_hover: TEXT
                                                        color_focus: TEXT
                                                        color_down: TEXT
                                                    }
                                                }

                                                // FK dropdown editor template
                                                FkEditor := mod.widgets.DropDownFlat{
                                                    width: Fill
                                                    height: Fill
                                                    draw_text +: {
                                                        text_style: theme.font_regular{font_size: 12.0}
                                                        color: mod.db_theme.text
                                                    }
                                                    draw_bg +: {
                                                        color: (INPUT_BG)
                                                        color_hover: (INPUT_BG)
                                                        color_focus: (INPUT_BG)
                                                        color_down: (INPUT_BG)
                                                        border_color: (ACCENT)
                                                        border_color_hover: (ACCENT)
                                                        border_color_focus: (ACCENT)
                                                        border_color_down: (ACCENT)
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // structure view (columns)
                                    struct_view := View{
                                        width: Fill
                                        height: Fill
                                        flow: Down
                                        spacing: 6
                                        visible: false

                                        struct_hint := Label{
                                            width: Fill
                                            text: "—"
                                            draw_text +: {
                                                text_style: theme.font_regular{font_size: 12.0}
                                                color: mod.db_theme.text_muted
                                            }
                                        }

                                        struct_scroll := mod.mp.MpScrollBoth{
                                            width: Fill
                                            height: Fill

                                            struct_table := mod.mp.MpTable{
                                                width: Fit
                                                height: Fit
                                            }
                                        }

                                        idx_caption := Label{
                                            width: Fill
                                            text: "INDEXES"
                                            draw_text +: {
                                                text_style: theme.font_bold{font_size: 10.5}
                                                color: mod.db_theme.text_faint
                                            }
                                        }

                                        idx_scroll := ScrollYView{
                                            width: Fill
                                            height: 110

                                            indexes_table := mod.mp.MpTable{
                                                width: Fit
                                                height: Fit
                                            }
                                        }

                                        struct_ddl_caption := Label{
                                            width: Fill
                                            text: "DDL"
                                            draw_text +: {
                                                text_style: theme.font_bold{font_size: 10.5}
                                                color: mod.db_theme.text_faint
                                            }
                                        }

                                        // display-only text (MpTextArea paints,
                                        // it is not an editor)
                                        // The editor takes its plate and its type from the theme: the v2 text area's
                                        // colour and size overrides were a call site choosing a style, which is the thing
                                        // this library's vocabulary refuses, and they no longer name fields that exist.
                                        struct_ddl := mod.mp.MpEditor{
                                            width: Fill
                                            height: 130
                                        }
                                    }
                                }

                                // ── query editor view ──
                                query_view := View{
                                    width: Fill
                                    height: Fill
                                    flow: Down
                                    spacing: 8
                                    visible: false

                                    query_toolbar := View{
                                        width: Fill
                                        height: Fit
                                        flow: Right
                                        spacing: 8
                                        align: Align{x: 0.0, y: 0.5}

                                        run_btn := mod.mp.MpButton{style: mod.mp.ButtonStyle.Prominent, text: "▶ Run" }
                                        run_hint := Label{
                                            text: "⌘/Ctrl+Enter · ⌘↑/↓ history · ⌘R re-run"
                                            draw_text +: {
                                                text_style: theme.font_regular{font_size: 11.0}
                                                color: mod.db_theme.text_faint
                                            }
                                        }

                                        View{width: 12, height: 1}

                                        sample_select := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost, text: "SELECT *" }
                                        sample_join := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost, text: "JOIN sample" }
                                        sample_agg := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost, text: "GROUP BY sample" }

                                        View{width: Fill, height: 1}

                                        query_export_btn := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost, text: "⬇ CSV" }
                                    }

                                    // Real multiline text editor (MpTextArea is
                                    // display-only). ⌘/Ctrl+Enter emits `Returned`
                                    // instead of inserting a newline.
                                    sql_editor := mod.widgets.TextInput{
                                        width: Fill
                                        height: 150
                                        is_multiline: true
                                        empty_text: "-- Write SQL here, then ⌘/Ctrl+Enter to run"

                                        draw_bg +: {
                                            color: (INPUT_BG)
                                            border_color: (BORDER)
                                            border_color_focus: (CARET)
                                        }
                                        draw_text +: {
                                            text_style: theme.font_regular{font_size: 13.0}
                                            color: TEXT
                                            color_empty: TEXT_FAINT
                                        }
                                    }

                                    query_grid_scroll := mod.mp.MpScrollBoth{
                                        width: Fill
                                        height: Fill

                                        query_grid := mod.mp.MpTable{
                                            width: Fit
                                            height: Fit
                                        }
                                    }
                                }

                                // ── empty state ──
                                empty_view := View{
                                    width: Fill
                                    height: Fill
                                    align: Align{x: 0.5, y: 0.5}
                                    flow: Down
                                    spacing: 10
                                    visible: true

                                    empty_title := Label{
                                        text: "Open a table from the sidebar"
                                        draw_text +: {
                                            text_style: theme.font_bold{font_size: 16.0}
                                            color: mod.db_theme.text_muted
                                        }
                                    }
                                    empty_hint := Label{
                                        text: "◉ click a connection to connect · ＋ Connection adds a server · ＋ Query opens the SQL editor"
                                        draw_text +: {
                                            text_style: theme.font_regular{font_size: 12.0}
                                            color: mod.db_theme.text_faint
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // ── Bottom status bar ──
                    bottom_bar := View{
                        width: Fill
                        height: 24
                        flow: Right
                        align: Align{x: 0.0, y: 0.5}
                        padding: Inset{left: 14, right: 14, top: 0, bottom: 0}
                        spacing: 8
                        show_bg: true
                        draw_bg +: {
                            color: mod.db_theme.panel
                            pixel: fn() { return self.color }
                        }

                        conn_label := Label{
                            text: "—"
                            draw_text +: {
                                text_style: theme.font_regular{font_size: 11.0}
                                color: mod.db_theme.text_faint
                            }
                        }
                        View{width: Fill, height: 1}
                        brand_label := Label{
                            text: "Makepad · Rust · SQLite / MySQL / PostgreSQL"
                            draw_text +: {
                                text_style: theme.font_regular{font_size: 11.0}
                                color: mod.db_theme.text_faint
                            }
                        }
                    }

                    // ── Connection dialog (overlay) ──
                    connect_dialog := mod.mp.MpDialog{
                        content +: {
                            dialog +: {
                                header +: {
                                    title +: {
                                        dlg_title := Label{
                                            text: "New Connection"
                                            draw_text +: { color: mod.db_theme.text }
                                        }
                                    }
                                }
                                body +: {
                                    form := View{
                                        width: Fill
                                        height: Fit
                                        flow: Down
                                        spacing: 8

                                        dlg_kind_row := View{
                                            width: Fill, height: Fit
                                            flow: Right
                                            spacing: 8
                                            align: Align{y: 0.5}
                                            kind_caption := Label{
                                                text: "Type"
                                                width: 84
                                                draw_text +: {
                                                    text_style: theme.font_regular{font_size: 12.0}
                                                    color: mod.db_theme.text_muted
                                                }
                                            }
                                            dlg_kind := mod.mp.MpSegmented{
                                                width: Fill
                                            }
                                        }

                                        dlg_name_row := View{
                                            width: Fill, height: Fit
                                            flow: Right
                                            spacing: 8
                                            align: Align{y: 0.5}
                                            name_caption := Label{
                                                text: "Name"
                                                width: 84
                                                draw_text +: {
                                                    text_style: theme.font_regular{font_size: 12.0}
                                                    color: mod.db_theme.text_muted
                                                }
                                            }
                                            dlg_name := mod.mp.MpTextInput{
                                                width: Fill
                                                height: Fit
                                                empty_text: "Display name (optional)"
                                            }
                                        }

                                        dlg_group_row := View{
                                            width: Fill, height: Fit
                                            flow: Right
                                            spacing: 8
                                            align: Align{y: 0.5}
                                            group_caption := Label{
                                                text: "Group"
                                                width: 84
                                                draw_text +: {
                                                    text_style: theme.font_regular{font_size: 12.0}
                                                    color: mod.db_theme.text_muted
                                                }
                                            }
                                            dlg_group := mod.mp.MpTextInput{
                                                width: Fill
                                                height: Fit
                                                empty_text: "Sidebar folder (optional)"
                                            }
                                        }

                                        dlg_path_row := View{
                                            width: Fill, height: Fit
                                            flow: Right
                                            spacing: 8
                                            align: Align{y: 0.5}
                                            path_caption := Label{
                                                text: "File path"
                                                width: 84
                                                draw_text +: {
                                                    text_style: theme.font_regular{font_size: 12.0}
                                                    color: mod.db_theme.text_muted
                                                }
                                            }
                                            dlg_path := mod.mp.MpTextInput{
                                                width: Fill
                                                height: Fit
                                                empty_text: "/path/to/database.db"
                                            }
                                        }

                                        dlg_host_row := View{
                                            width: Fill, height: Fit
                                            flow: Right
                                            spacing: 8
                                            align: Align{y: 0.5}
                                            visible: false
                                            host_caption := Label{
                                                text: "Host"
                                                width: 84
                                                draw_text +: {
                                                    text_style: theme.font_regular{font_size: 12.0}
                                                    color: mod.db_theme.text_muted
                                                }
                                            }
                                            dlg_host := mod.mp.MpTextInput{
                                                width: Fill
                                                height: Fit
                                                empty_text: "localhost"
                                            }
                                        }

                                        dlg_port_row := View{
                                            width: Fill, height: Fit
                                            flow: Right
                                            spacing: 8
                                            align: Align{y: 0.5}
                                            visible: false
                                            port_caption := Label{
                                                text: "Port"
                                                width: 84
                                                draw_text +: {
                                                    text_style: theme.font_regular{font_size: 12.0}
                                                    color: mod.db_theme.text_muted
                                                }
                                            }
                                            dlg_port := mod.mp.MpTextInput{
                                                width: Fill
                                                height: Fit
                                                empty_text: "3306"
                                            }
                                        }

                                        dlg_user_row := View{
                                            width: Fill, height: Fit
                                            flow: Right
                                            spacing: 8
                                            align: Align{y: 0.5}
                                            visible: false
                                            user_caption := Label{
                                                text: "User"
                                                width: 84
                                                draw_text +: {
                                                    text_style: theme.font_regular{font_size: 12.0}
                                                    color: mod.db_theme.text_muted
                                                }
                                            }
                                            dlg_user := mod.mp.MpTextInput{
                                                width: Fill
                                                height: Fit
                                                empty_text: "username"
                                            }
                                        }

                                        dlg_password_row := View{
                                            width: Fill, height: Fit
                                            flow: Right
                                            spacing: 8
                                            align: Align{y: 0.5}
                                            visible: false
                                            password_caption := Label{
                                                text: "Password"
                                                width: 84
                                                draw_text +: {
                                                    text_style: theme.font_regular{font_size: 12.0}
                                                    color: mod.db_theme.text_muted
                                                }
                                            }
                                            dlg_password := mod.mp.MpTextInput{
                                                width: Fill
                                                height: Fit
                                            }
                                        }

                                        dlg_database_row := View{
                                            width: Fill, height: Fit
                                            flow: Right
                                            spacing: 8
                                            align: Align{y: 0.5}
                                            visible: false
                                            database_caption := Label{
                                                text: "Database"
                                                width: 84
                                                draw_text +: {
                                                    text_style: theme.font_regular{font_size: 12.0}
                                                    color: mod.db_theme.text_muted
                                                }
                                            }
                                            dlg_database := mod.mp.MpTextInput{
                                                width: Fill
                                                height: Fit
                                                empty_text: "database name"
                                            }
                                        }

                                        dlg_ssh_divider := View{
                                            width: Fill, height: 1
                                            margin: Inset{top: 4, bottom: 4}
                                            show_bg: true
                                            draw_bg +: {
                                                color: mod.db_theme.border
                                                pixel: fn() { return self.color }
                                            }
                                        }

                                        dlg_ssh_host_row := View{
                                            width: Fill, height: Fit
                                            flow: Right
                                            spacing: 8
                                            align: Align{y: 0.5}
                                            ssh_host_caption := Label{
                                                text: "SSH Host"
                                                width: 84
                                                draw_text +: {
                                                    text_style: theme.font_regular{font_size: 12.0}
                                                    color: mod.db_theme.text_muted
                                                }
                                            }
                                            dlg_ssh_host := mod.mp.MpTextInput{
                                                width: Fill
                                                height: Fit
                                                empty_text: "optional — jump host for tunnel (key auth)"
                                            }
                                        }

                                        dlg_ssh_user_row := View{
                                            width: Fill, height: Fit
                                            flow: Right
                                            spacing: 8
                                            align: Align{y: 0.5}
                                            ssh_user_caption := Label{
                                                text: "SSH User"
                                                width: 84
                                                draw_text +: {
                                                    text_style: theme.font_regular{font_size: 12.0}
                                                    color: mod.db_theme.text_muted
                                                }
                                            }
                                            dlg_ssh_user := mod.mp.MpTextInput{
                                                width: Fill
                                                height: Fit
                                                empty_text: "username on the jump host"
                                            }
                                        }

                                        dlg_ssh_port_row := View{
                                            width: Fill, height: Fit
                                            flow: Right
                                            spacing: 8
                                            align: Align{y: 0.5}
                                            ssh_port_caption := Label{
                                                text: "SSH Port"
                                                width: 84
                                                draw_text +: {
                                                    text_style: theme.font_regular{font_size: 12.0}
                                                    color: mod.db_theme.text_muted
                                                }
                                            }
                                            dlg_ssh_port := mod.mp.MpTextInput{
                                                width: Fill
                                                height: Fit
                                                empty_text: "22"
                                            }
                                        }

                                        dlg_status := Label{
                                            width: Fill
                                            text: ""
                                            draw_text +: {
                                                text_style: theme.font_regular{font_size: 12.0}
                                                color: mod.db_theme.warn_c
                                            }
                                        }
                                    }
                                }
                                footer +: {
                                    dlg_disconnect := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost, text: "Disconnect" }
                                    dlg_delete := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost, text: "Delete" }
                                    dlg_test := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost, text: "Test" }
                                    dlg_cancel := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost, text: "Cancel" }
                                    dlg_connect := mod.mp.MpButton{style: mod.mp.ButtonStyle.Prominent, text: "Connect" }
                                }
                            }
                        }
                    }

                    // ── delete confirmation dialog ──
                    confirm_dialog := mod.mp.MpDialog{
                        content +: {
                            dialog +: {
                                header +: {
                                    title +: {
                                        confirm_title := Label{
                                            text: "Delete Row"
                                            draw_text +: { color: mod.db_theme.err_c }
                                        }
                                    }
                                }
                                body +: {
                                    confirm_msg := Label{
                                        width: Fill
                                        text: ""
                                        draw_text +: {
                                            text_style: theme.font_regular{font_size: 13.0}
                                            color: mod.db_theme.text
                                        }
                                    }
                                }
                                footer +: {
                                    confirm_cancel := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost, text: "Cancel" }
                                    confirm_ok := mod.mp.MpButton{style: mod.mp.ButtonStyle.Prominent, text: "Delete" }
                                }
                            }
                        }
                    }

                    // ── row detail dialog ──
                    detail_dialog := mod.mp.MpDialog{
                        content +: {
                            dialog +: {
                                header +: {
                                    title +: {
                                        detail_title := Label{
                                            text: "Row Detail"
                                            draw_text +: { color: mod.db_theme.text }
                                        }
                                    }
                                }
                                body +: {
                                    detail_text := mod.mp.MpEditor{
                                        width: Fill
                                        height: 400
                                    }
                                }
                                footer +: {
                                    detail_close := mod.mp.MpButton{style: mod.mp.ButtonStyle.Prominent, text: "Close" }
                                }
                            }
                        }
                    }

                    // ── CSV import dialog ──
                    import_dialog := mod.mp.MpDialog{
                        content +: {
                            dialog +: {
                                header +: {
                                    title +: {
                                        import_title := Label{
                                            text: "Import CSV"
                                            draw_text +: { color: mod.db_theme.text }
                                        }
                                    }
                                }
                                body +: {
                                    View{
                                        width: Fill, height: Fit
                                        flow: Right
                                        spacing: 8
                                        align: Align{y: 0.5}
                                        import_path_caption := Label{
                                            text: "File path"
                                            width: 84
                                            draw_text +: {
                                                text_style: theme.font_regular{font_size: 12.0}
                                                color: mod.db_theme.text_muted
                                            }
                                        }
                                        import_path := mod.mp.MpTextInput{
                                            width: Fill
                                            height: Fit
                                            empty_text: "/path/to/data.csv (header row = column names)"
                                        }
                                    }
                                    import_status := Label{
                                        width: Fill
                                        text: "Columns are matched by header name; empty cells become NULL."
                                        draw_text +: {
                                            text_style: theme.font_regular{font_size: 12.0}
                                            color: mod.db_theme.text_faint
                                        }
                                    }
                                }
                                footer +: {
                                    import_cancel := mod.mp.MpButton{style: mod.mp.ButtonStyle.Ghost, text: "Cancel" }
                                    import_go := mod.mp.MpButton{style: mod.mp.ButtonStyle.Prominent, text: "Import" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
