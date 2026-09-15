# DbPro

A [TablePro](https://github.com/TableProApp/TablePro)-inspired, cross-platform
database GUI built with **Makepad + Rust**. No Electron, no web view — pure
GPU-rendered UI via the Makepad 2.0 `script_mod!` API.

![status](https://img.shields.io/badge/status-MVP-blue)

## Features

- **Multi-database**: SQLite (bundled), MySQL, PostgreSQL — built in, no plugins
- **Connection library**: saved profiles in `~/.dbpro/connections.json`
  (passwords are kept in memory only, never persisted), connect dialog with
  Test / Connect / Edit / Delete; optional **Group** field organizes
  connections into `📁` folders in the sidebar
- **SSH tunnels (experimental)**: fill SSH Host / User / Port in the connect
  dialog and DbPro brings up an `ssh -N -L` tunnel automatically, then
  connects the database through it. Uses the `ssh` binary with key-based
  auth (BatchMode) — set up agent forwarding or `~/.ssh/config` for hosts
  that need it
- **Explorer sidebar**: connections → tables / views tree; `◉` = connected,
  `○` = click to connect; chevron folds a schema; clicking a table opens it;
  row counts are fetched in a background sweep and shown next to each table;
  a filter box narrows long table lists
- **Document tabs with per-tab state**: every table tab remembers its own
  page / filter / sort / cached rows (switching tabs is instant, no refetch);
  query tabs each remember their SQL; tabs are **bound to the connection they
  were opened from**, so you can browse several servers side by side
- **Editable data grid** (makepad `DataGrid`): server-side paging
  (200 rows/page), server-side `ORDER BY` via column-header click
  (indicator survives reloads), server-side text filter across the first
  10 columns (0.4 s debounce), row-number headers, zebra stripes, cell
  selection with keyboard navigation, resizable columns,
  content-adaptive column widths, **numeric columns right-aligned with
  cleaned-up float display** (raw values preserved for edit/export)
- **Copy to clipboard**: select cells in the grid and press `⌘/Ctrl+C` —
  TSV output pastes straight into a spreadsheet
- **CSV import**: `⬆ CSV` opens a dialog (file path) and imports a CSV into
  the open table — header names are matched to columns
  (case-insensitive), empty cells become `NULL`, and everything is written
  in one atomic multi-row `INSERT`; the view jumps to the last page where
  the new rows landed
- **Safe deletes**: `－ Row` asks for confirmation (showing the exact
  `DELETE` statement) before destroying data
- **FK-aware editing**: foreign-key columns (e.g. `orders.user_id` →
  `users.id`) open a dropdown of referenced values rendered as
  `id — name` instead of a bare text editor
- **Write-back**: double-click (or F2 / type over) a cell to edit it inline;
  `Enter` commits (`UPDATE … WHERE pk`), `Esc` cancels, empty value sets
  `NULL` for nullable columns; `－ Row` deletes the selected row;
  `⧉ Duplicate` copies the selected row (integer PKs are omitted so the
  engine assigns a fresh key, then the view jumps to the last page where the
  copy landed); `＋ Row` inserts a blank row via `INSERT … DEFAULT VALUES`
  and jumps to it — columns fall back to their defaults, tables with
  NOT-NULL-without-default columns report the driver error. Optimistic
  local update + server verification; editing requires a primary key, so
  views and PK-less tables stay read-only (FK violations surface as errors)
- **Structure view**: per-table `Data | Structure` toggle showing column
  name / type / nullability / primary key, the table's **indexes**, and the
  CREATE statement (SQLite / MySQL; PG reports that pg_dump is needed)
- **SQL editor**: real multiline editor (makepad `TextInput`), `⌘/Ctrl+Enter`
  to run, result grid (up to 1000 rows), `rows affected` for DML/DDL, sample
  queries, query history cycled with `⌘↑` / `⌘↓` and persisted across
  sessions (`~/.dbpro/history.json`, last 100)
- **CSV export**: `⬇ CSV` exports the current grid page or the last query
  result; `⬇ All` streams the **whole table** to disk page by page in a
  worker thread with live progress in the status bar
  (`~/dbpro-export-…-all-<ts>.csv`); click `⬇ All` again to cancel
- **Status at a glance**: toolbar dot (gray idle · green connected · amber
  busy · red error), bottom bar shows the active connection, window title
  follows it; `⏏` disconnects the active connection, `⌕ Detail` opens the
  selected row with every column in a readable dialog
- **Schema refresh + filter**: `⟳` in the sidebar re-reads the table list
  for the active connection without reconnecting; the filter box narrows
  the tree to matching tables (handy on hundred-table schemas)
- **First-run demo database**: a seeded SQLite file (`dbpro-demo.db`) with
  users / products / orders / settings + a view, auto-connected on launch

## Keyboard shortcuts

| Keys | Action |
| --- | --- |
| `⌘/Ctrl + Enter` | run the SQL in the editor |
| `⌘/Ctrl + R` | refresh the current table page / re-run the query |
| `⌘/Ctrl + ↑` / `↓` | cycle query history |
| `Enter` / `Esc` (in cell editor) | commit / cancel the cell edit |
| `F2` / double-click (on a cell) | start inline editing |
| `Delete` / `Backspace` (on a cell) | set the cell to `NULL` |
| `⟳` (sidebar) / `⧉ Duplicate` / `⬇ All` | refresh schema · duplicate row · export full table |
| `Esc` | close the connection dialog |

## Run

```bash
cargo run -p dbpro
```

Development progress / iteration log: [`../../docs/DBPRO_PROGRESS_CN.md`](../../docs/DBPRO_PROGRESS_CN.md).

On first launch DbPro creates and connects to a demo SQLite database so you
can immediately browse data and run queries.

### Connecting to a server

1. Click **＋ Connection**
2. Pick the type (SQLite / MySQL / PostgreSQL)
3. Fill host / port / user / password / database (or the file path for SQLite)
4. **Test** to verify, then **Connect**

Saved profiles survive restarts; the password is requested per session.
**⚙** in the sidebar header edits the active connection.

## Architecture

```
crates/dbpro/
├── Cargo.toml
└── src/
    ├── main.rs      entry point (app_main!)
    ├── lib.rs       module wiring
    ├── db.rs        driver layer:
    │                - ConnectionConfig (+ JSON persistence, no passwords)
    │                - DbConn enum: rusqlite / mysql / postgres (sync drivers)
    │                - schema listing (tables before views), describe, paged
    │                  queries, arbitrary SQL
    │                - worker threads post DbAction via Cx::post_action
    │                - first-run demo seeding (ensure_demo_db)
    ├── tab_bar.rs   DbTabBar — dynamic tab strip widget (paint + Areas +
    │                widget actions, MpTable-style)
    ├── grid.rs      DbGridHost — hosts makepad's DataGrid, feeds it the
    │                current page and hosts the inline cell editor
    │                (GridShared snapshot pushed in by the App)
    └── app.rs       App: script_mod! UI tree + logic. Each tab owns its
                     state (TableTab: page/search/order/columns/rows/…;
                     QueryTab: sql/result) and its connection binding.
```

### Threading model

All database I/O runs on `std::thread` workers. Connections live in
`HashMap<u64, Arc<Mutex<DbConn>>>`; a worker locks the shared connection for
the duration of one query and reports back with `Cx::post_action(DbAction::…)`
— the same pattern used by `gemini-talker`. Responses carry the request's
`(connection, table, page, search)` so the UI drops stale replies (the user
paged or re-searched on in the meantime).

### Value rendering

- `NULL` → `NULL`
- floats → trimmed decimal text
- blobs → `<blob N B>`
- PostgreSQL pages are fetched through `simple_query` (text protocol), so all
  types render without binary decoding; the search term is escaped and
  inlined as a string literal
- SQLite/MySQL filters use bound parameters
- UPDATE/DELETE statements are built with per-driver escaping (MySQL also
  escapes backslashes); primary keys come from the cached row

## Tests

```bash
cargo test -p dbpro
```

Covers the demo seeding + full SQLite round trip (schema listing with
tables-before-views ordering, paging, search, sorting, arbitrary SQL,
DDL/DML row counts) and connection-profile persistence.

## Known MVP limitations

- No INSERT-from-blank form yet (UPDATE / DELETE / duplicate-row are supported)
- SSH tunnels are experimental: key-based auth only, no host-key prompts
- PostgreSQL `simple_query` pages have no per-column type metadata
- Filter matches the first 10 columns only
- CSV export covers the current page / last query result, not the whole table
