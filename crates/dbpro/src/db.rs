// DbPro — database driver layer.
//
// Connection management, the three built-in drivers (SQLite, MySQL,
// PostgreSQL) and the background worker threads that post `DbAction`s
// back to the Makepad UI thread via `Cx::post_action`.
//
// Design notes:
// - One `DbConn` per connection config, stored by the UI in
//   `HashMap<u64, Arc<Mutex<DbConn>>>`. Worker threads lock the shared
//   connection while a query runs (one query at a time per connection).
// - All cell values are rendered to strings on the worker thread so the
//   UI thread never touches driver types.
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde::{Deserialize, Serialize};

use makepad_widgets::Cx;

// Queryable trait gives Conn::query_iter / exec_iter.
use mysql::prelude::*;

// ---------------------------------------------------------------------------
// Connection config + persistence
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DbKind {
    #[default]
    Sqlite,
    Mysql,
    Postgres,
}

impl DbKind {
    pub fn label(&self) -> &'static str {
        match self {
            DbKind::Sqlite => "SQLite",
            DbKind::Mysql => "MySQL",
            DbKind::Postgres => "PostgreSQL",
        }
    }

    pub fn default_port(&self) -> u16 {
        match self {
            DbKind::Sqlite => 0,
            DbKind::Mysql => 3306,
            DbKind::Postgres => 5432,
        }
    }

    /// The kinds in the order a chooser shows them.
    ///
    /// **The order is a decision, and it lives here** so that the segmented control's segments and the index it reports
    /// cannot come from two lists that disagree — which would make choosing "MySQL" open a Postgres connection.
    pub const ALL: [DbKind; 3] = [DbKind::Sqlite, DbKind::Mysql, DbKind::Postgres];

    /// This kind's position in [`DbKind::ALL`].
    pub fn index(&self) -> usize {
        match self {
            DbKind::Sqlite => 0,
            DbKind::Mysql => 1,
            DbKind::Postgres => 2,
        }
    }

    /// The kind at a position, for a chooser that reports one.
    pub fn from_index(index: usize) -> Option<Self> {
        DbKind::ALL.get(index).copied()
    }

    pub fn from_label(s: &str) -> Option<Self> {
        match s {
            "SQLite" => Some(DbKind::Sqlite),
            "MySQL" => Some(DbKind::Mysql),
            "PostgreSQL" => Some(DbKind::Postgres),
            _ => None,
        }
    }
}

/// A stored connection profile. Passwords are never persisted to disk.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub id: u64,
    pub name: String,
    pub kind: DbKind,
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub port: u16,
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub database: String,
    /// SQLite file path.
    #[serde(default)]
    pub path: String,
    /// Optional sidebar folder name ("" = ungrouped).
    #[serde(default)]
    pub group: String,
    /// Experimental SSH tunnel (key-based auth via the `ssh` binary).
    #[serde(default)]
    pub ssh_host: String,
    #[serde(default)]
    pub ssh_user: String,
    #[serde(default)]
    pub ssh_port: u16,
}

impl ConnectionConfig {
    pub fn display_host(&self) -> String {
        match self.kind {
            DbKind::Sqlite => self.path.clone(),
            _ => format!("{}:{}/{}", self.host, self.port, self.database),
        }
    }
}

/// Persisted connection library (passwords intentionally excluded).
fn config_dir() -> std::path::PathBuf {
    let base = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    std::path::Path::new(&base).join(".dbpro")
}

fn config_path() -> std::path::PathBuf {
    config_dir().join("connections.json")
}

pub fn load_configs() -> Option<Vec<ConnectionConfig>> {
    let text = std::fs::read_to_string(config_path()).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn save_configs(configs: &[ConnectionConfig]) {
    let dir = config_dir();
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    if let Ok(json) = serde_json::to_string_pretty(configs) {
        let _ = std::fs::write(config_path(), json);
    }
}

// ---------------------------------------------------------------------------
// Result models
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TableKind {
    Table,
    View,
}

#[derive(Clone, Debug)]
pub struct TableInfo {
    pub name: String,
    pub kind: TableKind,
}

/// A foreign-key relationship: `column` references `ref_table.ref_col`.
#[derive(Clone, Debug, PartialEq)]
pub struct FkInfo {
    pub column: String,
    pub ref_table: String,
    pub ref_col: String,
}

/// An index on a table.
#[derive(Clone, Debug, PartialEq)]
pub struct IndexInfo {
    pub name: String,
    /// Comma-joined column list in index order.
    pub columns: String,
    pub unique: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub is_key: bool,
}

/// Result of running a page fetch or an arbitrary SQL statement.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct QueryResult {
    pub columns: Vec<ColumnInfo>,
    pub rows: Vec<Vec<String>>,
    pub total_rows: Option<u64>,
    pub rows_affected: Option<u64>,
    pub truncated: bool,
    pub elapsed_ms: f64,
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// The unified connection enum
// ---------------------------------------------------------------------------

pub enum DbConn {
    Sqlite(rusqlite::Connection),
    Mysql(mysql::Conn),
    Postgres(postgres::Client),
}

impl std::fmt::Debug for DbConn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DbConn({})", self.describe_host())
    }
}

/// Max rows fetched for an arbitrary SQL statement's result grid.
pub const SQL_MAX_ROWS: usize = 1000;
/// Rows fetched per data-grid page.
pub const PAGE_SIZE: usize = 200;

/// (columns, rows, total_matching, elapsed_ms) for one grid page.
pub type PageResult = Result<(Vec<ColumnInfo>, Vec<Vec<String>>, u64, f64), String>;

impl DbConn {
    pub fn connect(cfg: &ConnectionConfig, password: &str) -> Result<Self, String> {
        match cfg.kind {
            DbKind::Sqlite => {
                if cfg.path.is_empty() {
                    return Err("SQLite requires a database file path".into());
                }
                let conn = rusqlite::Connection::open(&cfg.path)
                    .map_err(|e| format!("SQLite open failed: {}", e))?;
                Ok(DbConn::Sqlite(conn))
            }
            DbKind::Mysql => {
                let opts = mysql::OptsBuilder::new()
                    .ip_or_hostname(if cfg.host.is_empty() {
                        None
                    } else {
                        Some(cfg.host.as_str())
                    })
                    .tcp_port(if cfg.port == 0 { 3306 } else { cfg.port })
                    .user(if cfg.user.is_empty() {
                        None
                    } else {
                        Some(cfg.user.as_str())
                    })
                    .pass(if password.is_empty() {
                        None
                    } else {
                        Some(password)
                    })
                    .db_name(if cfg.database.is_empty() {
                        None
                    } else {
                        Some(cfg.database.as_str())
                    });
                let conn =
                    mysql::Conn::new(opts).map_err(|e| format!("MySQL connect failed: {}", e))?;
                Ok(DbConn::Mysql(conn))
            }
            DbKind::Postgres => {
                let url = format!(
                    "host={} port={} user={} password={} dbname={}",
                    if cfg.host.is_empty() {
                        "localhost"
                    } else {
                        &cfg.host
                    },
                    if cfg.port == 0 { 5432 } else { cfg.port },
                    if cfg.user.is_empty() {
                        "postgres"
                    } else {
                        &cfg.user
                    },
                    password,
                    if cfg.database.is_empty() {
                        "postgres"
                    } else {
                        &cfg.database
                    },
                );
                let conn = postgres::Client::connect(&url, postgres::NoTls)
                    .map_err(|e| format!("PostgreSQL connect failed: {}", e))?;
                Ok(DbConn::Postgres(conn))
            }
        }
    }

    pub fn describe_host(&self) -> &'static str {
        match self {
            DbConn::Sqlite(_) => "SQLite",
            DbConn::Mysql(_) => "MySQL",
            DbConn::Postgres(_) => "PostgreSQL",
        }
    }

    // -- schema ------------------------------------------------------------

    pub fn list_tables(&mut self) -> Result<Vec<TableInfo>, String> {
        match self {
            DbConn::Sqlite(conn) => {
                let mut out = Vec::new();
                let mut stmt = conn
                    .prepare(
                        "SELECT name, type FROM sqlite_master \
                         WHERE type IN ('table','view') AND name NOT LIKE 'sqlite_%' \
                         ORDER BY type DESC, name",
                    )
                    .map_err(|e| e.to_string())?;
                // type DESC: 'view' > 'table' → tables first? No: DESC puts
                // 'view' first, so order manually below instead.
                let rows = stmt
                    .query_map([], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                    })
                    .map_err(|e| e.to_string())?;
                for r in rows {
                    let (name, ty) = r.map_err(|e| e.to_string())?;
                    out.push((
                        name,
                        if ty == "view" {
                            TableKind::View
                        } else {
                            TableKind::Table
                        },
                    ));
                }
                out.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
                Ok(out
                    .into_iter()
                    .map(|(name, kind)| TableInfo { name, kind })
                    .collect())
            }
            DbConn::Mysql(conn) => {
                let mut out = Vec::new();
                let result = conn
                    .query_iter("SHOW FULL TABLES")
                    .map_err(|e| e.to_string())?;
                for row in result {
                    let row = row.map_err(|e| e.to_string())?;
                    let name: String = row.get(0).ok_or("bad row")?;
                    let ty: String = row.get(1).unwrap_or_else(|| "BASE TABLE".into());
                    out.push((
                        name,
                        if ty.ends_with("VIEW") {
                            TableKind::View
                        } else {
                            TableKind::Table
                        },
                    ));
                }
                out.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
                Ok(out
                    .into_iter()
                    .map(|(name, kind)| TableInfo { name, kind })
                    .collect())
            }
            DbConn::Postgres(client) => {
                let rows = client
                    .query(
                        "SELECT table_name, table_type FROM information_schema.tables \
                         WHERE table_schema = current_schema() \
                           AND table_type IN ('BASE TABLE','VIEW') ORDER BY 2, 1",
                        &[],
                    )
                    .map_err(|e| e.to_string())?;
                Ok(rows
                    .iter()
                    .map(|r| TableInfo {
                        name: r.get::<_, String>(0),
                        kind: if r.get::<_, String>(1) == "VIEW" {
                            TableKind::View
                        } else {
                            TableKind::Table
                        },
                    })
                    .collect())
            }
        }
    }

    pub fn describe_table(&mut self, table: &str) -> Result<Vec<ColumnInfo>, String> {
        match self {
            DbConn::Sqlite(conn) => {
                let sql = format!("PRAGMA table_info({})", quote_ident_sqlite(table));
                let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
                let rows = stmt
                    .query_map([], |row| {
                        Ok(ColumnInfo {
                            name: row.get::<_, String>(1)?,
                            data_type: {
                                let t: String = row.get(2)?;
                                if t.is_empty() {
                                    "any".into()
                                } else {
                                    t.to_lowercase()
                                }
                            },
                            nullable: row.get::<_, i64>(3)? == 0,
                            is_key: row.get::<_, i64>(5)? > 0,
                        })
                    })
                    .map_err(|e| e.to_string())?;
                rows.collect::<Result<Vec<_>, _>>()
                    .map_err(|e| e.to_string())
            }
            DbConn::Mysql(conn) => {
                let sql = format!("SHOW COLUMNS FROM {}", quote_ident_mysql(table));
                let result = conn.query_iter(&sql).map_err(|e| e.to_string())?;
                let mut out = Vec::new();
                for row in result {
                    let row = row.map_err(|e| e.to_string())?;
                    out.push(ColumnInfo {
                        name: row.get(0).unwrap_or_default(),
                        data_type: row
                            .get::<String, _>(1)
                            .unwrap_or_default()
                            .split(['(', ' '])
                            .next()
                            .unwrap_or("")
                            .to_string(),
                        nullable: row.get::<String, _>(2).unwrap_or_default() == "YES",
                        is_key: row
                            .get::<String, _>(3)
                            .map(|k| !k.is_empty())
                            .unwrap_or(false),
                    });
                }
                Ok(out)
            }
            DbConn::Postgres(client) => {
                let rows = client
                    .query(
                        "SELECT column_name, data_type, is_nullable \
                         FROM information_schema.columns \
                         WHERE table_schema = current_schema() AND table_name = $1 \
                         ORDER BY ordinal_position",
                        &[&table],
                    )
                    .map_err(|e| e.to_string())?;
                let pk = client
                    .query(
                        "SELECT kcu.column_name \
                         FROM information_schema.table_constraints tc \
                         JOIN information_schema.key_column_usage kcu \
                           ON tc.constraint_name = kcu.constraint_name \
                         WHERE tc.table_name = $1 AND tc.constraint_type = 'PRIMARY KEY'",
                        &[&table],
                    )
                    .map_err(|e| e.to_string())?;
                let pk_names: Vec<String> = pk.iter().map(|r| r.get::<_, String>(0)).collect();
                Ok(rows
                    .iter()
                    .map(|r| {
                        let name: String = r.get(0);
                        ColumnInfo {
                            is_key: pk_names.contains(&name),
                            name,
                            data_type: r.get::<_, String>(1),
                            nullable: r.get::<_, String>(2) == "YES",
                        }
                    })
                    .collect())
            }
        }
    }

    // -- data grid ----------------------------------------------------------

    /// Fetch one page of `table`, with optional server-side filter and sort.
    /// Returns (columns, rows, total_matching_rows, elapsed_ms).
    pub fn query_page(
        &mut self,
        table: &str,
        page: usize,
        search: Option<&str>,
        order_col: Option<&str>,
        order_asc: bool,
    ) -> PageResult {
        let started = Instant::now();
        let cols = self.describe_table(table)?;

        // Build the WHERE clause for the search filter over the first
        // 10 columns (server-side LIKE / ILIKE). The user's term doubles as
        // the LIKE pattern (so `%` / `_` work as wildcards), wrapped in %…%.
        let mut where_sql = String::new();
        let search_term: String = search
            .filter(|s| !s.is_empty())
            .map(|s| format!("%{}%", s))
            .unwrap_or_default();
        if !search_term.is_empty() && !cols.is_empty() {
            let mut parts = Vec::new();
            for c in cols.iter().take(10) {
                match self {
                    DbConn::Postgres(_) => parts.push(format!(
                        "CAST({} AS TEXT) ILIKE '%{}%'",
                        quote_ident_pg(&c.name),
                        search_term.trim_matches('%').replace('\'', "''")
                    )),
                    DbConn::Mysql(_) => {
                        parts.push(format!("{} LIKE ?", quote_ident_mysql(&c.name)))
                    }
                    DbConn::Sqlite(_) => {
                        parts.push(format!("{} LIKE ?", quote_ident_sqlite(&c.name)))
                    }
                }
            }
            where_sql = format!(" WHERE {}", parts.join(" OR "));
        }

        // ORDER BY: explicit sort column, else rowid for SQLite tables.
        let mut order_sql = String::new();
        if let Some(col) = order_col {
            let dir = if order_asc { "ASC" } else { "DESC" };
            match self {
                DbConn::Mysql(_) => {
                    order_sql = format!(" ORDER BY {} {}", quote_ident_mysql(col), dir)
                }
                DbConn::Postgres(_) => {
                    order_sql = format!(" ORDER BY {} {}", quote_ident_pg(col), dir)
                }
                DbConn::Sqlite(_) => {
                    order_sql = format!(" ORDER BY {} {}", quote_ident_sqlite(col), dir)
                }
            }
        } else if let DbConn::Sqlite(conn) = self {
            if !table_is_view_sqlite(conn, table) {
                order_sql = " ORDER BY rowid".to_string();
            }
        }

        let offset = page * PAGE_SIZE;
        match self {
            DbConn::Sqlite(conn) => {
                // Total count
                let count_sql = format!(
                    "SELECT COUNT(*) FROM {}{}",
                    quote_ident_sqlite(table),
                    where_sql
                );
                let total: u64 = if search_term.is_empty() {
                    conn.query_row(&count_sql, [], |r| r.get::<_, i64>(0))
                } else {
                    let n_params = where_sql.matches('?').count();
                    let params: Vec<String> = vec![search_term.clone(); n_params];
                    conn.query_row(&count_sql, rusqlite::params_from_iter(params.iter()), |r| {
                        r.get::<_, i64>(0)
                    })
                }
                .map_err(|e| e.to_string())? as u64;

                let sql = format!(
                    "SELECT * FROM {}{}{} LIMIT {} OFFSET {}",
                    quote_ident_sqlite(table),
                    where_sql,
                    order_sql,
                    PAGE_SIZE,
                    offset
                );
                let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
                let mut rows = Vec::new();
                if search_term.is_empty() {
                    let mut qr = stmt.query([]).map_err(|e| e.to_string())?;
                    while let Some(row) = qr.next().map_err(|e| e.to_string())? {
                        rows.push(row_values_sqlite(row)?);
                    }
                } else {
                    let n_params = where_sql.matches('?').count();
                    let params: Vec<String> = vec![search_term.clone(); n_params];
                    let mut qr = stmt
                        .query(rusqlite::params_from_iter(params.iter()))
                        .map_err(|e| e.to_string())?;
                    while let Some(row) = qr.next().map_err(|e| e.to_string())? {
                        rows.push(row_values_sqlite(row)?);
                    }
                }
                Ok((cols, rows, total, started.elapsed().as_secs_f64() * 1000.0))
            }
            DbConn::Mysql(conn) => {
                let count_sql = format!(
                    "SELECT COUNT(*) FROM {}{}",
                    quote_ident_mysql(table),
                    where_sql
                );
                let total: u64 = mysql_count(
                    conn,
                    &count_sql,
                    if search_term.is_empty() {
                        None
                    } else {
                        Some(&search_term)
                    },
                )?;

                let sql = format!(
                    "SELECT * FROM {}{}{} LIMIT {} OFFSET {}",
                    quote_ident_mysql(table),
                    where_sql,
                    order_sql,
                    PAGE_SIZE,
                    offset
                );
                let mut rows = Vec::new();
                if search_term.is_empty() {
                    let result = conn.query_iter(&sql).map_err(|e| e.to_string())?;
                    for row in result {
                        rows.push(row_values_mysql(&row.map_err(|e| e.to_string())?));
                    }
                } else {
                    let n_params = where_sql.matches('?').count();
                    let result = conn
                        .exec_iter(
                            &sql,
                            mysql::Params::Positional(vec![search_term.clone().into(); n_params]),
                        )
                        .map_err(|e| e.to_string())?;
                    for row in result {
                        rows.push(row_values_mysql(&row.map_err(|e| e.to_string())?));
                    }
                }
                Ok((cols, rows, total, started.elapsed().as_secs_f64() * 1000.0))
            }
            DbConn::Postgres(client) => {
                let count_sql = format!(
                    "SELECT COUNT(*) FROM {}{}",
                    quote_ident_pg(table),
                    where_sql
                );
                // COUNT(*) is an int8 — a typed single-row query is safe here.
                let total: i64 = client
                    .query_one(&count_sql, &[])
                    .map_err(|e| e.to_string())?
                    .get(0);

                // Page fetch goes through simple_query: values arrive as
                // text (no binary type decoding) and the search term is
                // inlined as an escaped string literal (simple protocol has
                // no parameter binding).
                let sql = format!(
                    "SELECT * FROM {}{}{} LIMIT {} OFFSET {}",
                    quote_ident_pg(table),
                    where_sql,
                    order_sql,
                    PAGE_SIZE,
                    offset
                );
                let messages = client.simple_query(&sql).map_err(|e| e.to_string())?;
                let mut rows = Vec::new();
                for msg in &messages {
                    if let postgres::SimpleQueryMessage::Row(row) = msg {
                        rows.push(
                            (0..row.len())
                                .map(|i| {
                                    row.get(i)
                                        .map(|s| s.to_string())
                                        .unwrap_or_else(|| "NULL".into())
                                })
                                .collect(),
                        );
                    }
                }
                Ok((
                    cols,
                    rows,
                    total as u64,
                    started.elapsed().as_secs_f64() * 1000.0,
                ))
            }
        }
    }

    // -- helpers ---------------------------------------------------------------

    /// `SELECT COUNT(*)` for a table (quoted per driver).
    pub fn table_count(&mut self, table: &str) -> Result<u64, String> {
        match self {
            DbConn::Sqlite(conn) => conn
                .query_row(
                    &format!("SELECT COUNT(*) FROM {}", quote_ident_sqlite(table)),
                    [],
                    |r| r.get::<_, i64>(0),
                )
                .map(|v| v as u64)
                .map_err(|e| e.to_string()),
            DbConn::Mysql(conn) => mysql_count(
                conn,
                &format!("SELECT COUNT(*) FROM {}", quote_ident_mysql(table)),
                None,
            ),
            DbConn::Postgres(client) => client
                .query_one(
                    &format!("SELECT COUNT(*) FROM {}", quote_ident_pg(table)),
                    &[],
                )
                .map(|row| row.get::<_, i64>(0) as u64)
                .map_err(|e| e.to_string()),
        }
    }

    // -- foreign keys ----------------------------------------------------------

    /// Foreign keys of a table: (column → ref_table.ref_col).
    pub fn foreign_keys(&mut self, table: &str) -> Result<Vec<FkInfo>, String> {
        match self {
            DbConn::Sqlite(conn) => {
                let sql = format!("PRAGMA foreign_key_list({})", quote_ident_sqlite(table));
                let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
                let rows = stmt
                    .query_map([], |row| {
                        Ok(FkInfo {
                            column: row.get::<_, String>(3)?,
                            ref_table: row.get::<_, String>(2)?,
                            ref_col: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                        })
                    })
                    .map_err(|e| e.to_string())?;
                let mut out: Vec<FkInfo> = rows
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| e.to_string())?;
                out.retain(|f| !f.ref_col.is_empty());
                Ok(out)
            }
            DbConn::Mysql(conn) => {
                let sql = format!(
                    "SELECT COLUMN_NAME, REFERENCED_TABLE_NAME, REFERENCED_COLUMN_NAME \
                     FROM information_schema.KEY_COLUMN_USAGE \
                     WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = '{}' \
                       AND REFERENCED_TABLE_NAME IS NOT NULL",
                    table.replace('\'', "''")
                );
                let result = conn.query_iter(&sql).map_err(|e| e.to_string())?;
                let mut out = Vec::new();
                for row in result {
                    let row = row.map_err(|e| e.to_string())?;
                    out.push(FkInfo {
                        column: row.get(0).unwrap_or_default(),
                        ref_table: row.get(1).unwrap_or_default(),
                        ref_col: row.get(2).unwrap_or_default(),
                    });
                }
                Ok(out)
            }
            DbConn::Postgres(client) => {
                let rows = client
                    .query(
                        "SELECT kcu.column_name, ccu.table_name, ccu.column_name \
                         FROM information_schema.table_constraints tc \
                         JOIN information_schema.key_column_usage kcu \
                           ON tc.constraint_name = kcu.constraint_name \
                          AND tc.table_schema = kcu.table_schema \
                         JOIN information_schema.constraint_column_usage ccu \
                           ON ccu.constraint_name = tc.constraint_name \
                          AND ccu.table_schema = tc.table_schema \
                         WHERE tc.constraint_type = 'FOREIGN KEY' AND tc.table_name = $1",
                        &[&table],
                    )
                    .map_err(|e| e.to_string())?;
                Ok(rows
                    .iter()
                    .map(|r| FkInfo {
                        column: r.get(0),
                        ref_table: r.get(1),
                        ref_col: r.get(2),
                    })
                    .collect())
            }
        }
    }

    /// Pick a human-friendly display column from the referenced table.
    fn fk_display_column(&mut self, ref_table: &str, ref_col: &str) -> String {
        const PREFERRED: [&str; 5] = ["name", "title", "label", "email", "username"];
        if let Ok(cols) = self.describe_table(ref_table) {
            for p in PREFERRED {
                if let Some(c) = cols.iter().find(|c| c.name.to_lowercase() == p) {
                    return c.name.clone();
                }
            }
            if let Some(c) = cols.iter().find(|c| c.name != ref_col) {
                return c.name.clone();
            }
        }
        ref_col.to_string()
    }

    /// Fetch up to `limit` (key, display) pairs for a dropdown.
    pub fn fk_options(
        &mut self,
        fk: &FkInfo,
        limit: usize,
    ) -> Result<Vec<(String, String)>, String> {
        let display = self.fk_display_column(&fk.ref_table, &fk.ref_col);
        let cols = format!(
            "{}, {}",
            quote_col_by_kind_for(self, &fk.ref_col),
            quote_col_by_kind_for(self, &display)
        );
        let sql = format!(
            "SELECT {} FROM {} ORDER BY 1 LIMIT {}",
            cols,
            quote_col_by_kind_for(self, &fk.ref_table),
            limit
        );
        let pairs = match self {
            DbConn::Sqlite(conn) => {
                let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
                let mut qr = stmt.query([]).map_err(|e| e.to_string())?;
                let mut out: Vec<(String, String)> = Vec::new();
                while let Some(row) = qr.next().map_err(|e| e.to_string())? {
                    let k = sqlite_value_text(row, 0);
                    let d = sqlite_value_text(row, 1);
                    out.push((k, d));
                }
                out
            }
            DbConn::Mysql(conn) => {
                let mut out = Vec::new();
                for row in conn.query_iter(&sql).map_err(|e| e.to_string())? {
                    let row = row.map_err(|e| e.to_string())?;
                    out.push((
                        row.get::<String, _>(0).unwrap_or_default(),
                        row.get::<String, _>(1).unwrap_or_default(),
                    ));
                }
                out
            }
            DbConn::Postgres(client) => {
                let messages = client.simple_query(&sql).map_err(|e| e.to_string())?;
                let mut out = Vec::new();
                for msg in &messages {
                    if let postgres::SimpleQueryMessage::Row(row) = msg {
                        out.push((
                            row.get(0).unwrap_or("").to_string(),
                            row.get(1).unwrap_or("NULL").to_string(),
                        ));
                    }
                }
                out
            }
        };
        Ok(pairs
            .into_iter()
            .map(|(k, d)| {
                if d == "NULL" || d == k {
                    (k.clone(), k)
                } else {
                    (k.clone(), format!("{} — {}", k, d))
                }
            })
            .collect())
    }

    // -- indexes ---------------------------------------------------------------

    /// Indexes of a table with their columns and uniqueness.
    pub fn table_indexes(&mut self, table: &str) -> Result<Vec<IndexInfo>, String> {
        match self {
            DbConn::Sqlite(conn) => {
                let list_sql = format!("PRAGMA index_list({})", quote_ident_sqlite(table));
                let mut stmt = conn.prepare(&list_sql).map_err(|e| e.to_string())?;
                let entries: Vec<(String, bool)> = {
                    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
                    let mut out = Vec::new();
                    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
                        // columns: seq(0), name(1), unique(2), origin(3), partial(4)
                        out.push((
                            row.get::<_, String>(1).map_err(|e| e.to_string())?,
                            row.get::<_, i64>(2).map_err(|e| e.to_string())? != 0,
                        ));
                    }
                    out
                };
                let mut out = Vec::new();
                for (name, unique) in entries {
                    let cols_sql = format!("PRAGMA index_info({})", quote_ident_sqlite(&name));
                    let mut stmt = conn.prepare(&cols_sql).map_err(|e| e.to_string())?;
                    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
                    let mut cols: Vec<String> = Vec::new();
                    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
                        cols.push(row.get::<_, String>(2).map_err(|e| e.to_string())?);
                    }
                    out.push(IndexInfo {
                        name,
                        columns: cols.join(", "),
                        unique,
                    });
                }
                Ok(out)
            }
            DbConn::Mysql(conn) => {
                let sql = format!("SHOW INDEX FROM {}", quote_ident_mysql(table));
                let result = conn.query_iter(&sql).map_err(|e| e.to_string())?;
                // group rows by key name, preserving column order
                let mut order: Vec<String> = Vec::new();
                let mut map: HashMap<String, (Vec<(u32, String)>, bool)> = HashMap::new();
                for row in result {
                    let row = row.map_err(|e| e.to_string())?;
                    let key: String = row.get(2).unwrap_or_default();
                    let seq: u32 = row.get(3).unwrap_or(1);
                    let col: String = row.get(4).unwrap_or_default();
                    let non_unique: bool = row.get(1).unwrap_or(1u8) != 0;
                    let entry = map.entry(key.clone()).or_insert_with(|| {
                        order.push(key.clone());
                        (Vec::new(), !non_unique)
                    });
                    entry.0.push((seq, col));
                }
                Ok(order
                    .into_iter()
                    .map(|name| {
                        let (mut cols, unique) = map.remove(&name).unwrap();
                        cols.sort_by_key(|(seq, _)| *seq);
                        IndexInfo {
                            name,
                            columns: cols
                                .into_iter()
                                .map(|(_, c)| c)
                                .collect::<Vec<_>>()
                                .join(", "),
                            unique,
                        }
                    })
                    .collect())
            }
            DbConn::Postgres(client) => {
                let rows = client
                    .query(
                        "SELECT indexname, indexdef FROM pg_indexes WHERE tablename = $1",
                        &[&table],
                    )
                    .map_err(|e| e.to_string())?;
                Ok(rows
                    .iter()
                    .map(|r| {
                        let name: String = r.get(0);
                        let def: String = r.get(1);
                        let unique = def.to_uppercase().contains("UNIQUE");
                        // extract the column list inside the first parens pair
                        let columns = def
                            .split('(')
                            .nth(1)
                            .and_then(|rest| rest.split(')').next())
                            .unwrap_or("")
                            .to_string();
                        IndexInfo {
                            name,
                            columns,
                            unique,
                        }
                    })
                    .collect())
            }
        }
    }

    // -- DDL preview ---------------------------------------------------------

    /// CREATE statement for a table/view (SQLite and MySQL only; PG has no
    /// single-statement equivalent without pg_dump).
    pub fn fetch_ddl(&mut self, table: &str) -> Result<String, String> {
        match self {
            DbConn::Sqlite(conn) => conn
                .query_row(
                    "SELECT COALESCE(sql, '') FROM sqlite_master WHERE name = ?1",
                    [table],
                    |r| r.get::<_, String>(0),
                )
                .map_err(|e| e.to_string()),
            DbConn::Mysql(conn) => {
                let sql = format!("SHOW CREATE TABLE {}", quote_ident_mysql(table));
                let mut result = conn.query_iter(&sql).map_err(|e| e.to_string())?;
                match result.next() {
                    Some(Ok(row)) => Ok(row
                        .get::<String, _>(1)
                        .unwrap_or_else(|| "-- empty DDL".into())),
                    Some(Err(e)) => Err(e.to_string()),
                    None => Err("SHOW CREATE TABLE returned no rows".into()),
                }
            }
            DbConn::Postgres(_) => {
                Err("DDL preview is not supported for PostgreSQL tables — use pg_dump".into())
            }
        }
    }

    // -- arbitrary SQL -------------------------------------------------------

    pub fn run_sql(&mut self, sql: &str) -> Result<QueryResult, String> {
        let started = Instant::now();
        let sql_trim = sql.trim();
        if sql_trim.is_empty() {
            return Err("Empty statement".into());
        }
        match self {
            DbConn::Sqlite(conn) => {
                let mut stmt = conn.prepare(sql_trim).map_err(|e| e.to_string())?;
                let col_count = stmt.column_count();
                let mut result = QueryResult {
                    elapsed_ms: 0.0,
                    ..Default::default()
                };
                if col_count == 0 {
                    let affected = stmt.execute([]).map_err(|e| e.to_string())?;
                    result.rows_affected = Some(affected as u64);
                } else {
                    let names: Vec<String> = stmt
                        .column_names()
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect();
                    result.columns = names
                        .into_iter()
                        .map(|name| ColumnInfo {
                            name,
                            data_type: String::new(),
                            nullable: true,
                            is_key: false,
                        })
                        .collect();
                    let mut rows_out = Vec::new();
                    let mut qr = stmt.query([]).map_err(|e| e.to_string())?;
                    while let Some(row) = qr.next().map_err(|e| e.to_string())? {
                        if rows_out.len() >= SQL_MAX_ROWS {
                            result.truncated = true;
                            break;
                        }
                        rows_out.push(row_values_sqlite(row)?);
                    }
                    result.rows = rows_out;
                    result.total_rows = Some(result.rows.len() as u64);
                }
                result.elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
                Ok(result)
            }
            DbConn::Mysql(conn) => {
                let mut result = QueryResult {
                    elapsed_ms: 0.0,
                    ..Default::default()
                };
                let mut query_result = conn.query_iter(sql_trim).map_err(|e| e.to_string())?;
                let col_count = query_result.columns().as_ref().len();
                if col_count == 0 {
                    result.rows_affected = Some(query_result.affected_rows());
                } else {
                    let names: Vec<String> = query_result
                        .columns()
                        .as_ref()
                        .iter()
                        .map(|c| c.name_str().to_string())
                        .collect();
                    result.columns = names
                        .into_iter()
                        .map(|name| ColumnInfo {
                            name,
                            data_type: String::new(),
                            nullable: true,
                            is_key: false,
                        })
                        .collect();
                    let mut rows_out = Vec::new();
                    for row in query_result.by_ref() {
                        if rows_out.len() >= SQL_MAX_ROWS {
                            result.truncated = true;
                            break;
                        }
                        rows_out.push(row_values_mysql(&row.map_err(|e| e.to_string())?));
                    }
                    result.rows = rows_out;
                    result.total_rows = Some(result.rows.len() as u64);
                }
                result.elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
                Ok(result)
            }
            DbConn::Postgres(client) => {
                // simple_query: works for both SELECT and DDL/DML, returns
                // text values directly, and tolerates multiple statements.
                let messages = client.simple_query(sql_trim).map_err(|e| e.to_string())?;
                let mut result = QueryResult {
                    elapsed_ms: 0.0,
                    ..Default::default()
                };
                let mut current_cols: Option<Vec<String>> = None;
                for msg in &messages {
                    match msg {
                        postgres::SimpleQueryMessage::Row(row) => {
                            let names: Vec<String> =
                                row.columns().iter().map(|c| c.name().to_string()).collect();
                            let ncols = names.len();
                            if current_cols.is_none() {
                                current_cols = Some(names);
                            }
                            // Collect rows of the first result set only.
                            if result.columns.len() == ncols || result.columns.is_empty() {
                                if result.rows.len() < SQL_MAX_ROWS {
                                    let vals: Vec<String> = (0..row.len())
                                        .map(|i| {
                                            row.get(i)
                                                .map(|s| s.to_string())
                                                .unwrap_or_else(|| "NULL".into())
                                        })
                                        .collect();
                                    result.rows.push(vals);
                                } else {
                                    result.truncated = true;
                                }
                            }
                        }
                        postgres::SimpleQueryMessage::CommandComplete(rows)
                            if result.columns.is_empty() =>
                        {
                            result.rows_affected = Some(*rows);
                        }
                        _ => {}
                    }
                }
                if let Some(names) = current_cols {
                    result.columns = names
                        .into_iter()
                        .map(|name| ColumnInfo {
                            name,
                            data_type: String::new(),
                            nullable: true,
                            is_key: false,
                        })
                        .collect();
                    result.total_rows = Some(result.rows.len() as u64);
                }
                result.elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
                Ok(result)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Value rendering helpers
// ---------------------------------------------------------------------------

/// Read one cell as text regardless of its storage class.
fn sqlite_value_text(row: &rusqlite::Row, i: usize) -> String {
    match row.get_ref(i) {
        Ok(rusqlite::types::ValueRef::Null) => "NULL".to_string(),
        Ok(rusqlite::types::ValueRef::Integer(v)) => v.to_string(),
        Ok(rusqlite::types::ValueRef::Real(v)) => format_float(v),
        Ok(rusqlite::types::ValueRef::Text(t)) => String::from_utf8_lossy(t).into_owned(),
        Ok(rusqlite::types::ValueRef::Blob(b)) => format!("<blob {} B>", b.len()),
        Err(_) => String::new(),
    }
}

fn row_values_sqlite(row: &rusqlite::Row) -> Result<Vec<String>, String> {
    let n = row.as_ref().column_count();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let v = row.get_ref(i).map_err(|e| e.to_string())?;
        out.push(match v {
            rusqlite::types::ValueRef::Null => "NULL".to_string(),
            rusqlite::types::ValueRef::Integer(i) => i.to_string(),
            rusqlite::types::ValueRef::Real(f) => format_float(f),
            rusqlite::types::ValueRef::Text(t) => String::from_utf8_lossy(t).into_owned(),
            rusqlite::types::ValueRef::Blob(b) => {
                format!("<blob {} B>", b.len())
            }
        });
    }
    Ok(out)
}

fn row_values_mysql(row: &mysql::Row) -> Vec<String> {
    let n = row.len();
    (0..n)
        .map(|i| match row.as_ref(i) {
            Some(mysql::Value::NULL) => "NULL".to_string(),
            Some(mysql::Value::Bytes(b)) => String::from_utf8_lossy(b).into_owned(),
            Some(mysql::Value::Int(v)) => v.to_string(),
            Some(mysql::Value::UInt(v)) => v.to_string(),
            Some(mysql::Value::Float(v)) => format_float(*v as f64),
            Some(mysql::Value::Double(v)) => format_float(*v),
            Some(mysql::Value::Date(y, m, d, h, mi, s, _us)) => {
                format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, m, d, h, mi, s)
            }
            Some(mysql::Value::Time(neg, d, h, m, s, _us)) => {
                let sign = if *neg { "-" } else { "" };
                format!("{}{:02}:{:02}:{:02}", sign, d * 24 + *h as u32, m, s)
            }
            _ => String::new(),
        })
        .collect()
}

/// SELECT COUNT(*) helper for MySQL (optional bound search term).
fn mysql_count(conn: &mut mysql::Conn, sql: &str, term: Option<&String>) -> Result<u64, String> {
    if let Some(t) = term {
        let n = sql.matches('?').count();
        let mut result = conn
            .exec_iter(sql, mysql::Params::Positional(vec![t.clone().into(); n]))
            .map_err(|e| e.to_string())?;
        return match result.next() {
            Some(Ok(row)) => Ok(row.get::<u64, _>(0).unwrap_or(0)),
            Some(Err(e)) => Err(e.to_string()),
            None => Err("COUNT query returned no rows".into()),
        };
    }
    let mut result = conn.query_iter(sql).map_err(|e| e.to_string())?;
    match result.next() {
        Some(Ok(row)) => Ok(row.get::<u64, _>(0).unwrap_or(0)),
        Some(Err(e)) => Err(e.to_string()),
        None => Err("COUNT query returned no rows".into()),
    }
}

fn format_float(f: f64) -> String {
    if f == f.trunc() && f.abs() < 1e15 {
        format!("{}", f as i64)
    } else {
        let s = format!("{}", f);
        s
    }
}

fn table_is_view_sqlite(conn: &rusqlite::Connection, table: &str) -> bool {
    conn.query_row(
        "SELECT type FROM sqlite_master WHERE name = ?1",
        [table],
        |r| r.get::<_, String>(0),
    )
    .map(|t| t == "view")
    .unwrap_or(false)
}

// Identifier quoting --------------------------------------------------------

fn quote_ident_sqlite(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

fn quote_ident_mysql(name: &str) -> String {
    format!("`{}`", name.replace('`', "``"))
}

fn quote_ident_pg(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

// ---------------------------------------------------------------------------
// Write-back SQL builders (cell edit / row delete)
// ---------------------------------------------------------------------------

/// Escape a value as a SQL string literal for the given driver.
/// MySQL additionally escapes backslashes (they are escape chars by default).
pub fn sql_literal(kind: DbKind, value: &str) -> String {
    match kind {
        DbKind::Mysql => format!("'{}'", value.replace('\\', "\\\\").replace('\'', "''")),
        _ => format!("'{}'", value.replace('\'', "''")),
    }
}

fn quote_col_by_kind_for(conn: &DbConn, name: &str) -> String {
    match conn {
        DbConn::Sqlite(_) => quote_ident_sqlite(name),
        DbConn::Mysql(_) => quote_ident_mysql(name),
        DbConn::Postgres(_) => quote_ident_pg(name),
    }
}

fn quote_col(kind: DbKind, name: &str) -> String {
    match kind {
        DbKind::Mysql => quote_ident_mysql(name),
        DbKind::Postgres => quote_ident_pg(name),
        DbKind::Sqlite => quote_ident_sqlite(name),
    }
}

/// WHERE clause matching the table's primary key from the cached row.
/// Returns None when the row has no usable key.
pub fn pk_where(kind: DbKind, pk: &[(String, String)]) -> Option<String> {
    if pk.is_empty() {
        return None;
    }
    Some(
        pk.iter()
            .map(|(k, v)| format!("{} = {}", quote_col(kind, k), sql_literal(kind, v)))
            .collect::<Vec<_>>()
            .join(" AND "),
    )
}

/// UPDATE one row's column. `new_value = None` sets NULL.
pub fn build_update(
    kind: DbKind,
    table: &str,
    column: &str,
    new_value: Option<&str>,
    pk: &[(String, String)],
) -> Option<String> {
    let wh = pk_where(kind, pk)?;
    let set = match new_value {
        Some(v) => format!("{} = {}", quote_col(kind, column), sql_literal(kind, v)),
        None => format!("{} = NULL", quote_col(kind, column)),
    };
    Some(format!(
        "UPDATE {} SET {} WHERE {}",
        quote_col(kind, table),
        set,
        wh
    ))
}

/// DELETE one row by primary key.
pub fn build_delete(kind: DbKind, table: &str, pk: &[(String, String)]) -> Option<String> {
    let wh = pk_where(kind, pk)?;
    Some(format!(
        "DELETE FROM {} WHERE {}",
        quote_col(kind, table),
        wh
    ))
}

/// CSV field escaping (RFC 4180).
pub fn csv_cell(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// INSERT a blank row, letting every column fall back to its default.
/// MySQL has no DEFAULT VALUES clause — `() VALUES ()` is equivalent.
pub fn build_insert_default(kind: DbKind, table: &str) -> String {
    match kind {
        DbKind::Mysql => format!("INSERT INTO {} () VALUES ()", quote_col(kind, table)),
        _ => format!("INSERT INTO {} DEFAULT VALUES", quote_col(kind, table)),
    }
}

/// Parse CSV text (RFC 4180): quoted fields, escaped quotes (`""`),
/// commas and newlines inside quotes, `\r\n` line endings.
/// The first row is the caller's concern (header); nothing is skipped here.
pub fn parse_csv(text: &str) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut pending_quote = false; // saw a closing quote (field was quoted)
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    in_quotes = false;
                    pending_quote = true;
                }
            } else {
                field.push(c);
            }
        } else {
            match c {
                '"' if field.is_empty() => {
                    in_quotes = true;
                    pending_quote = false;
                }
                '"' => {
                    // stray quote inside an unquoted field — keep it
                    field.push(c);
                    let _ = pending_quote;
                }
                ',' => {
                    row.push(std::mem::take(&mut field));
                    pending_quote = false;
                }
                '\r' => {
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    row.push(std::mem::take(&mut field));
                    rows.push(std::mem::take(&mut row));
                    pending_quote = false;
                }
                '\n' => {
                    row.push(std::mem::take(&mut field));
                    rows.push(std::mem::take(&mut row));
                    pending_quote = false;
                }
                _ => {
                    field.push(c);
                    pending_quote = false;
                }
            }
        }
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    rows
}

/// Build a multi-row `INSERT INTO table (cols) VALUES (...), ...` mapping
/// CSV header names to table columns (exact, then case-insensitive).
/// Returns the SQL and the number of data rows.
pub fn build_insert_rows(
    kind: DbKind,
    table: &str,
    table_columns: &[ColumnInfo],
    header: &[String],
    rows: &[Vec<String>],
) -> Result<(String, usize), String> {
    if header.is_empty() {
        return Err("CSV has no header row".into());
    }
    // map each header cell to a table column index
    let mut col_map: Vec<Option<usize>> = Vec::with_capacity(header.len());
    let mut unknown: Vec<String> = Vec::new();
    for h in header {
        let name = h.trim();
        let exact = table_columns.iter().position(|c| c.name == name);
        let ci = exact.or_else(|| {
            table_columns
                .iter()
                .position(|c| c.name.to_lowercase() == name.to_lowercase())
        });
        match ci {
            Some(i) => col_map.push(Some(i)),
            None => {
                unknown.push(name.to_string());
                col_map.push(None);
            }
        }
    }
    if let Some(first) = unknown.first() {
        return Err(format!(
            "CSV column '{}' does not exist in table '{}' (remove it from the CSV header)",
            first, table
        ));
    }
    if col_map.iter().all(|c| c.is_none()) {
        return Err("CSV header matches no table columns".into());
    }

    let used: Vec<String> = col_map
        .iter()
        .enumerate()
        .filter(|(_, m)| m.is_some())
        .map(|(i, _)| quote_col(kind, header[i].trim()))
        .collect();
    let used_cols = used.join(", ");

    let mut tuples: Vec<String> = Vec::new();
    let mut count = 0usize;
    for row in rows {
        if row.iter().all(|c| c.is_empty()) {
            continue; // skip blank lines
        }
        let mut vals: Vec<String> = Vec::with_capacity(col_map.len());
        // values are positional in CSV order; col_map only decides which
        // table column each lands in (via the INSERT column list)
        for (i, _) in col_map.iter().enumerate() {
            let v = row.get(i).map(|s| s.as_str()).unwrap_or("");
            if v.is_empty() || v == "NULL" {
                vals.push("NULL".to_string());
            } else {
                vals.push(sql_literal(kind, v));
            }
        }
        tuples.push(format!("({})", vals.join(", ")));
        count += 1;
    }
    if count == 0 {
        return Err("CSV has no data rows".into());
    }
    Ok((
        format!(
            "INSERT INTO {} ({}) VALUES {}",
            quote_col(kind, table),
            used_cols,
            tuples.join(", ")
        ),
        count,
    ))
}

fn is_int_type(t: &str) -> bool {
    let t = t.to_lowercase();
    t.contains("int") || t.contains("serial")
}

/// INSERT a copy of `row`. Integer primary keys are omitted so the engine
/// assigns a fresh key (SQLite rowid alias / AUTO_INCREMENT / serial).
pub fn build_insert_copy(
    kind: DbKind,
    table: &str,
    columns: &[ColumnInfo],
    row: &[String],
) -> Option<String> {
    let mut cols: Vec<String> = Vec::new();
    let mut vals: Vec<String> = Vec::new();
    for (i, c) in columns.iter().enumerate() {
        let Some(v) = row.get(i) else { continue };
        if c.is_key && is_int_type(&c.data_type) {
            continue;
        }
        cols.push(quote_col(kind, &c.name));
        if v == "NULL" {
            vals.push("NULL".to_string());
        } else {
            vals.push(sql_literal(kind, v));
        }
    }
    if cols.is_empty() {
        return None;
    }
    Some(format!(
        "INSERT INTO {} ({}) VALUES ({})",
        quote_col(kind, table),
        cols.join(", "),
        vals.join(", ")
    ))
}

// ---------------------------------------------------------------------------
// Actions posted back to the UI thread
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum DbAction {
    Connected {
        config: ConnectionConfig,
        conn: Arc<Mutex<DbConn>>,
        tables: Vec<TableInfo>,
    },
    ConnectFailed {
        config_id: u64,
        error: String,
    },
    TableRows {
        config_id: u64,
        table: String,
        page: usize,
        search: String,
        result: PageResult,
    },
    SqlDone {
        request_key: String,
        result: Result<QueryResult, String>,
    },
    TestResult {
        ok: bool,
        message: String,
    },
    /// Full-table CSV export finished.
    ExportDone {
        path: String,
        rows: u64,
        error: Option<String>,
    },
    /// Row counts for a schema (background COUNT sweep).
    TableCounts {
        config_id: u64,
        counts: Vec<(String, u64)>,
    },
    /// Full-table export progress (rows written so far).
    ExportProgress {
        rows: u64,
    },
    /// Table list refreshed for an already-open connection.
    SchemaRefreshed {
        config_id: u64,
        result: Result<Vec<TableInfo>, String>,
    },
    /// CSV import finished.
    ImportDone {
        rows: usize,
        error: Option<String>,
    },
    /// Foreign keys of a table (background metadata sweep).
    FksFetched {
        config_id: u64,
        table: String,
        result: Result<Vec<FkInfo>, String>,
    },
    /// Dropdown options for an FK cell being edited.
    FkOptionsFetched {
        options: Vec<(String, String)>,
    },
    /// Index list fetched for the Structure view.
    TableIndexes {
        config_id: u64,
        table: String,
        result: Result<Vec<IndexInfo>, String>,
    },
    /// CREATE statement fetched for the Structure view.
    DdlFetched {
        config_id: u64,
        table: String,
        result: Result<String, String>,
    },
}

/// Connect in the background; on success hand the live connection + table
/// list back to the UI thread.
/// Live SSH tunnel processes keyed by config id. Killed and replaced on
/// reconnect; the OS reaps them when the app exits.
static TUNNELS: std::sync::LazyLock<Mutex<HashMap<u64, std::process::Child>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Bring up (or reuse) an `ssh -N -L` tunnel for `cfg`. Returns the local
/// host:port the database connection should use. Key-based auth only
/// (BatchMode) — password prompts would block the worker thread.
fn ensure_tunnel(cfg: &ConnectionConfig) -> Result<(String, u16), String> {
    use std::net::TcpStream;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    // replace any previous tunnel for this config
    let mut t = TUNNELS.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(mut old) = t.remove(&cfg.id) {
        let _ = old.kill();
        let _ = old.wait();
    }
    // pick a free local port
    let local_port = {
        let s = std::net::TcpListener::bind(("127.0.0.1", 0)).map_err(|e| e.to_string())?;
        s.local_addr().map_err(|e| e.to_string())?.port()
    };
    let user = if cfg.ssh_user.is_empty() {
        String::new()
    } else {
        format!("{}@", cfg.ssh_user)
    };
    let mut child = Command::new("ssh")
        .args([
            "-N",
            "-L",
            &format!("127.0.0.1:{}:{}:{}", local_port, cfg.host, cfg.port),
            "-p",
            &cfg.ssh_port.to_string(),
            "-o",
            "BatchMode=yes",
            "-o",
            "ExitOnForwardFailure=yes",
            "-o",
            "StrictHostKeyChecking=accept-new",
            &format!("{}{}", user, cfg.ssh_host),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("failed to launch ssh: {}", e))?;
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if TcpStream::connect(("127.0.0.1", local_port)).is_ok() {
            TUNNELS
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .insert(cfg.id, child);
            return Ok(("127.0.0.1".to_string(), local_port));
        }
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!(
                "ssh exited early ({}) — check key auth / host / port",
                status
            ));
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    let _ = child.kill();
    Err("ssh tunnel timeout — port never opened".into())
}

/// Connect in the background; on success hand the live connection + table
/// list back to the UI thread.
pub fn spawn_connect(cfg: ConnectionConfig, password: String) {
    std::thread::spawn(move || {
        let mut cfg = cfg;
        if !cfg.ssh_host.is_empty() {
            match ensure_tunnel(&cfg) {
                Ok((host, port)) => {
                    cfg.host = host;
                    cfg.port = port;
                }
                Err(e) => {
                    Cx::post_action(DbAction::ConnectFailed {
                        config_id: cfg.id,
                        error: format!("SSH tunnel: {}", e),
                    });
                    return;
                }
            }
        }
        match DbConn::connect(&cfg, &password) {
            Ok(mut conn) => match conn.list_tables() {
                Ok(tables) => {
                    Cx::post_action(DbAction::Connected {
                        config: cfg,
                        conn: Arc::new(Mutex::new(conn)),
                        tables,
                    });
                }
                Err(e) => {
                    Cx::post_action(DbAction::ConnectFailed {
                        config_id: cfg.id,
                        error: format!("Connected, but listing tables failed: {}", e),
                    });
                }
            },
            Err(e) => {
                Cx::post_action(DbAction::ConnectFailed {
                    config_id: cfg.id,
                    error: e,
                });
            }
        }
    });
}

/// Test a connection in the background (dialog “Test” button).
pub fn spawn_test_connection(cfg: ConnectionConfig, password: String) {
    std::thread::spawn(move || {
        let mut cfg = cfg;
        if !cfg.ssh_host.is_empty() {
            if let Err(e) = ensure_tunnel(&cfg) {
                Cx::post_action(DbAction::TestResult {
                    ok: false,
                    message: format!("SSH tunnel: {}", e),
                });
                return;
            }
            cfg.host = "127.0.0.1".into();
        }
        match DbConn::connect(&cfg, &password) {
            Ok(mut conn) => {
                let server = conn.describe_host().to_string();
                let tables = conn.list_tables().map(|t| t.len()).unwrap_or(0);
                Cx::post_action(DbAction::TestResult {
                    ok: true,
                    message: format!("✓ Connected to {} · {} objects", server, tables),
                });
            }
            Err(e) => {
                Cx::post_action(DbAction::TestResult {
                    ok: false,
                    message: e,
                });
            }
        }
    });
}

/// Fetch one page of `table` on the shared connection in the background.
pub fn spawn_query_page(
    conn: Arc<Mutex<DbConn>>,
    config_id: u64,
    table: String,
    page: usize,
    search: String,
    order_col: Option<String>,
    order_asc: bool,
) {
    std::thread::spawn(move || {
        let mut guard = match conn.lock() {
            Ok(g) => g,
            Err(_) => {
                Cx::post_action(DbAction::TableRows {
                    config_id,
                    table,
                    page,
                    search,
                    result: Err("Connection mutex poisoned".into()),
                });
                return;
            }
        };
        let result = guard.query_page(
            &table,
            page,
            if search.is_empty() {
                None
            } else {
                Some(&search)
            },
            order_col.as_deref(),
            order_asc,
        );
        Cx::post_action(DbAction::TableRows {
            config_id,
            table,
            page,
            search,
            result,
        });
    });
}

/// Run arbitrary SQL on the shared connection in the background.
pub fn spawn_run_sql(conn: Arc<Mutex<DbConn>>, request_key: String, sql: String) {
    std::thread::spawn(move || {
        let result = match conn.lock() {
            Ok(mut guard) => guard.run_sql(&sql),
            Err(_) => Err("Connection mutex poisoned".into()),
        };
        Cx::post_action(DbAction::SqlDone {
            request_key,
            result,
        });
    });
}

// ---------------------------------------------------------------------------
// Demo database (first run): a small SQLite store so the app is usable
// immediately.
// ---------------------------------------------------------------------------

/// Background COUNT(*) sweep for a schema (capped at 200 tables so huge
/// schemas don't hammer the server). Posts one batched action at the end.
pub fn spawn_count_tables(conn: Arc<Mutex<DbConn>>, config_id: u64, tables: Vec<String>) {
    std::thread::spawn(move || {
        let mut guard = match conn.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let mut counts: Vec<(String, u64)> = Vec::new();
        for table in tables.iter().take(200) {
            if let Ok(n) = guard.table_count(table) {
                counts.push((table.clone(), n));
            }
        }
        Cx::post_action(DbAction::TableCounts { config_id, counts });
    });
}

/// Refresh the table list for an open connection.
pub fn spawn_list_tables(conn: Arc<Mutex<DbConn>>, config_id: u64) {
    std::thread::spawn(move || {
        let result = match conn.lock() {
            Ok(mut guard) => guard.list_tables(),
            Err(_) => Err("Connection mutex poisoned".into()),
        };
        Cx::post_action(DbAction::SchemaRefreshed { config_id, result });
    });
}

/// Set to true to request cancellation of a running full-table export.
pub static EXPORT_CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Stream a whole table to a CSV file, one page at a time.
pub fn spawn_export_table(conn: Arc<Mutex<DbConn>>, table: String, path: String) {
    std::thread::spawn(move || {
        use std::sync::atomic::Ordering;
        EXPORT_CANCEL.store(false, Ordering::Relaxed);
        use std::io::Write;
        let mut guard = match conn.lock() {
            Ok(g) => g,
            Err(_) => {
                Cx::post_action(DbAction::ExportDone {
                    path,
                    rows: 0,
                    error: Some("Connection mutex poisoned".into()),
                });
                return;
            }
        };
        let mut file = match std::fs::File::create(&path) {
            Ok(f) => f,
            Err(e) => {
                Cx::post_action(DbAction::ExportDone {
                    path,
                    rows: 0,
                    error: Some(e.to_string()),
                });
                return;
            }
        };
        let mut total: u64 = 0;
        let mut page = 0usize;
        let mut header_written = false;
        loop {
            match guard.query_page(&table, page, None, None, true) {
                Ok((columns, rows, _, _)) => {
                    if !header_written {
                        let header: Vec<String> =
                            columns.iter().map(|c| csv_cell(&c.name)).collect();
                        let _ = writeln!(file, "{}", header.join(","));
                        header_written = true;
                    }
                    if rows.is_empty() {
                        break;
                    }
                    for row in &rows {
                        let line: Vec<String> = row.iter().map(|c| csv_cell(c)).collect();
                        let _ = writeln!(file, "{}", line.join(","));
                        total += 1;
                    }
                    Cx::post_action(DbAction::ExportProgress { rows: total });
                    if EXPORT_CANCEL.load(Ordering::Relaxed) {
                        let _ = std::fs::remove_file(&path);
                        Cx::post_action(DbAction::ExportDone {
                            path,
                            rows: total,
                            error: Some("cancelled by user".into()),
                        });
                        return;
                    }
                    if rows.len() < PAGE_SIZE {
                        break;
                    }
                    page += 1;
                }
                Err(e) => {
                    let _ = std::fs::remove_file(&path);
                    Cx::post_action(DbAction::ExportDone {
                        path,
                        rows: total,
                        error: Some(e),
                    });
                    return;
                }
            }
        }
        Cx::post_action(DbAction::ExportDone {
            path,
            rows: total,
            error: None,
        });
    });
}

/// Import a CSV file into `table`: parse, map headers to columns, insert
/// all rows in one multi-row statement (atomic per engine).
pub fn spawn_import_csv(conn: Arc<Mutex<DbConn>>, table: String, path: String) {
    std::thread::spawn(move || {
        let done = |rows: usize, error: Option<String>| {
            Cx::post_action(DbAction::ImportDone { rows, error });
        };
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                done(0, Some(format!("read {}: {}", path, e)));
                return;
            }
        };
        let mut csv_rows = parse_csv(&text);
        if csv_rows.len() < 2 {
            done(
                0,
                Some("CSV needs a header row plus at least one data row".into()),
            );
            return;
        }
        let header = csv_rows.remove(0);

        let mut guard = match conn.lock() {
            Ok(g) => g,
            Err(_) => {
                done(0, Some("Connection mutex poisoned".into()));
                return;
            }
        };
        let columns = match guard.describe_table(&table) {
            Ok(c) => c,
            Err(e) => {
                done(0, Some(format!("describe {}: {}", table, e)));
                return;
            }
        };
        let kind = match &*guard {
            DbConn::Sqlite(_) => DbKind::Sqlite,
            DbConn::Mysql(_) => DbKind::Mysql,
            DbConn::Postgres(_) => DbKind::Postgres,
        };
        let (sql, count) = match build_insert_rows(kind, &table, &columns, &header, &csv_rows) {
            Ok(x) => x,
            Err(e) => {
                done(0, Some(e));
                return;
            }
        };
        match guard.run_sql(&sql) {
            Ok(r) => done(count, r.error.clone()),
            Err(e) => done(0, Some(e)),
        }
    });
}

/// Fetch foreign keys of a table in the background.
pub fn spawn_fetch_fks(conn: Arc<Mutex<DbConn>>, config_id: u64, table: String) {
    std::thread::spawn(move || {
        let result = match conn.lock() {
            Ok(mut guard) => guard.foreign_keys(&table),
            Err(_) => Err("Connection mutex poisoned".into()),
        };
        Cx::post_action(DbAction::FksFetched {
            config_id,
            table,
            result,
        });
    });
}

/// Fetch dropdown options for an FK column being edited.
pub fn spawn_fetch_fk_options(conn: Arc<Mutex<DbConn>>, fk: FkInfo) {
    std::thread::spawn(move || {
        let options = match conn.lock() {
            Ok(mut guard) => guard.fk_options(&fk, 500).unwrap_or_default(),
            Err(_) => Vec::new(),
        };
        Cx::post_action(DbAction::FkOptionsFetched { options });
    });
}

/// Fetch index metadata for the Structure view.
pub fn spawn_fetch_indexes(conn: Arc<Mutex<DbConn>>, config_id: u64, table: String) {
    std::thread::spawn(move || {
        let result = match conn.lock() {
            Ok(mut guard) => guard.table_indexes(&table),
            Err(_) => Err("Connection mutex poisoned".into()),
        };
        Cx::post_action(DbAction::TableIndexes {
            config_id,
            table,
            result,
        });
    });
}

/// Fetch the CREATE statement for the Structure view.
pub fn spawn_fetch_ddl(conn: Arc<Mutex<DbConn>>, config_id: u64, table: String) {
    std::thread::spawn(move || {
        let result = match conn.lock() {
            Ok(mut guard) => guard.fetch_ddl(&table),
            Err(_) => Err("Connection mutex poisoned".into()),
        };
        Cx::post_action(DbAction::DdlFetched {
            config_id,
            table,
            result,
        });
    });
}

/// Query history persistence (~/.dbpro/history.json).
pub fn load_history() -> Vec<String> {
    let base = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    std::fs::read_to_string(std::path::Path::new(&base).join(".dbpro/history.json"))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save_history(history: &[String]) {
    let base = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir = std::path::Path::new(&base).join(".dbpro");
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    if let Ok(json) = serde_json::to_string(history) {
        let _ = std::fs::write(dir.join("history.json"), json);
    }
}

pub fn demo_db_path() -> String {
    "dbpro-demo.db".to_string()
}

/// Create + seed the demo SQLite database if it doesn't exist yet.
pub fn ensure_demo_db(path: &str) {
    if std::path::Path::new(path).exists() {
        return;
    }
    let Ok(conn) = rusqlite::Connection::open(path) else {
        return;
    };
    let _ = conn.execute_batch(
        r#"
        CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            age INTEGER,
            city TEXT,
            balance REAL,
            created_at TEXT
        );
        CREATE TABLE products (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            price REAL NOT NULL,
            stock INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL REFERENCES users(id),
            product_id INTEGER NOT NULL REFERENCES products(id),
            quantity INTEGER NOT NULL,
            total REAL NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            ordered_at TEXT NOT NULL
        );
        CREATE TABLE settings (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL DEFAULT 'setting',
            value TEXT,
            updated_at TEXT
        );
        CREATE VIEW order_summary AS
            SELECT o.id, u.name AS customer, p.name AS product,
                   o.quantity, o.total, o.status
            FROM orders o
            JOIN users u ON u.id = o.user_id
            JOIN products p ON p.id = o.product_id;
        "#,
    );

    let cities = [
        "Shanghai", "Beijing", "Shenzhen", "Hangzhou", "Chengdu", "Berlin", "Tokyo", "Seattle",
    ];
    let mut stmt = conn
        .prepare("INSERT INTO users (name, email, age, city, balance, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")
        .expect("seed users");
    for i in 1..=48 {
        let city = cities[i % cities.len()];
        let name = format!("user_{:03}", i);
        let _ = stmt.execute(rusqlite::params![
            name,
            format!("{}@example.com", name),
            18 + (i * 7) % 50,
            city,
            (i as f64 * 37.5) % 5000.0,
            format!("2026-{:02}-{:02}", 1 + i % 12, 1 + i % 28),
        ]);
    }

    let categories = ["keyboard", "mouse", "monitor", "desk", "chair", "cable"];
    let mut stmt = conn
        .prepare("INSERT INTO products (name, category, price, stock) VALUES (?1, ?2, ?3, ?4)")
        .expect("seed products");
    for i in 1..=36 {
        let cat = categories[i % categories.len()];
        let _ = stmt.execute(rusqlite::params![
            format!("{} mk{}", cat, i % 9),
            cat,
            9.9 + (i as f64 * 11.3) % 900.0,
            (i * 13) % 240,
        ]);
    }

    let mut stmt = conn
        .prepare("INSERT INTO settings (name, value, updated_at) VALUES (?1, ?2, ?3)")
        .expect("seed settings");
    let _ = stmt.execute(rusqlite::params!["theme", "dark", "2026-09-15"]);
    let _ = stmt.execute(rusqlite::params!["font_size", "13", "2026-09-15"]);
    let _ = stmt.execute(rusqlite::params!["autocomplete", "on", "2026-09-14"]);

    let statuses = ["pending", "paid", "shipped", "delivered", "cancelled"];
    let mut stmt = conn
        .prepare("INSERT INTO orders (user_id, product_id, quantity, total, status, ordered_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")
        .expect("seed orders");
    for i in 1..=120 {
        let uid = 1 + (i * 11) % 48;
        let pid = 1 + (i * 5) % 36;
        let qty = 1 + i % 4;
        let total = qty as f64 * (9.9 + (pid as f64 * 11.3) % 900.0);
        let _ = stmt.execute(rusqlite::params![
            uid,
            pid,
            qty,
            total,
            statuses[i % statuses.len()],
            format!("2026-{:02}-{:02}", 1 + i % 12, 1 + i % 28),
        ]);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db(tag: &str) -> String {
        let dir = std::env::temp_dir().join(format!("dbpro-test-{}-{}", tag, std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("t.db").to_string_lossy().into_owned()
    }

    #[test]
    fn test_demo_seed_and_sqlite_roundtrip() {
        let path = temp_db("roundtrip");
        ensure_demo_db(&path);
        assert!(std::path::Path::new(&path).exists());

        let cfg = ConnectionConfig {
            id: 1,
            name: "t".into(),
            kind: DbKind::Sqlite,
            host: String::new(),
            port: 0,
            user: String::new(),
            database: String::new(),
            path: path.clone(),
            group: String::new(),
            ssh_host: String::new(),
            ssh_user: String::new(),
            ssh_port: 22,
        };
        let mut conn = DbConn::connect(&cfg, "").unwrap();

        // Schema listing: tables + view, tables first
        let tables = conn.list_tables().unwrap();
        assert_eq!(
            tables.first().map(|t| t.kind.clone()),
            Some(TableKind::Table)
        );
        assert!(tables
            .iter()
            .any(|t| t.name == "users" && t.kind == TableKind::Table));
        assert!(tables
            .iter()
            .any(|t| t.name == "order_summary" && t.kind == TableKind::View));

        // Page fetch
        let (cols, rows, total, _ms) = conn.query_page("users", 0, None, None, true).unwrap();
        assert_eq!(total, 48);
        assert_eq!(rows.len(), 48);
        assert!(cols.iter().any(|c| c.name == "email"));
        assert_eq!(cols[0].name, "id");

        // Second page is empty (only 48 rows)
        let (_, rows2, _, _) = conn.query_page("users", 1, None, None, true).unwrap();
        assert!(rows2.is_empty());

        // Server-side search
        let (_, _, total, _) = conn
            .query_page("users", 0, Some("user_00"), None, true)
            .unwrap();
        assert!(total > 0 && total < 48);

        // Server-side sort (numeric, descending)
        let (_, rows3, _, _) = conn
            .query_page("users", 0, None, Some("age"), false)
            .unwrap();
        let age_idx = cols.iter().position(|c| c.name == "age").unwrap();
        let ages: Vec<i64> = rows3
            .iter()
            .map(|r| r[age_idx].parse::<i64>().unwrap())
            .collect();
        let mut sorted = ages.clone();
        sorted.sort_unstable();
        sorted.reverse();
        assert_eq!(ages, sorted);

        // Arbitrary SQL: SELECT
        let r = conn.run_sql("SELECT id, name FROM users LIMIT 5").unwrap();
        assert_eq!(r.rows.len(), 5);
        assert_eq!(r.columns.len(), 2);
        assert!(!r.truncated);

        // Arbitrary SQL: DDL / DML
        conn.run_sql("CREATE TABLE zz_test(a INTEGER)").unwrap();
        let r = conn
            .run_sql("INSERT INTO zz_test (a) VALUES (1), (2), (3)")
            .unwrap();
        assert_eq!(r.rows_affected, Some(3));
        conn.run_sql("DROP TABLE zz_test").unwrap();

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_writeback_roundtrip() {
        let path = temp_db("writeback");
        ensure_demo_db(&path);
        let cfg = ConnectionConfig {
            id: 1,
            name: "t".into(),
            kind: DbKind::Sqlite,
            host: String::new(),
            port: 0,
            group: String::new(),
            ssh_host: String::new(),
            ssh_user: String::new(),
            ssh_port: 22,
            user: String::new(),
            database: String::new(),
            path: path.clone(),
        };
        let mut conn = DbConn::connect(&cfg, "").unwrap();

        // UPDATE via build_update (the exact path the UI takes)
        let sql = build_update(
            DbKind::Sqlite,
            "users",
            "city",
            Some("Kyoto"),
            &[("id".to_string(), "1".to_string())],
        )
        .unwrap();
        conn.run_sql(&sql).unwrap();
        let (_, rows, _, _) = conn.query_page("users", 0, None, None, true).unwrap();
        let row1 = rows.iter().find(|r| r[0] == "1").unwrap();
        assert_eq!(row1[4], "Kyoto");

        // set NULL (empty editor value)
        let sql = build_update(
            DbKind::Sqlite,
            "users",
            "city",
            None,
            &[("id".to_string(), "1".to_string())],
        )
        .unwrap();
        conn.run_sql(&sql).unwrap();
        let (_, rows, _, _) = conn.query_page("users", 0, None, None, true).unwrap();
        let row1 = rows.iter().find(|r| r[0] == "1").unwrap();
        assert_eq!(row1[4], "NULL");

        // INSERT DEFAULT VALUES on the settings table (all columns nullable
        // or defaulted) — the exact path the ＋ Row button takes
        let sql = build_insert_default(DbKind::Sqlite, "settings");
        assert_eq!(sql, "INSERT INTO \"settings\" DEFAULT VALUES");
        conn.run_sql(&sql).unwrap();
        let (_, _, total, _) = conn.query_page("settings", 0, None, None, true).unwrap();
        assert_eq!(total, 4);
        // blank insert fails on users (NOT NULL name without default)
        let sql = build_insert_default(DbKind::Sqlite, "users");
        assert!(conn.run_sql(&sql).is_err());

        // table_count helper
        assert_eq!(conn.table_count("users").unwrap(), 48);

        // foreign keys of orders: user_id → users.id, product_id → products.id
        let fks = conn.foreign_keys("orders").unwrap();
        assert!(fks.contains(&FkInfo {
            column: "user_id".into(),
            ref_table: "users".into(),
            ref_col: "id".into()
        }));
        // views have no FKs
        assert!(conn.foreign_keys("order_summary").unwrap().is_empty());
        // options: display column heuristic picks `name`, pairs are (key, "key — name")
        let fk = fks.iter().find(|f| f.column == "user_id").unwrap().clone();
        let options = conn.fk_options(&fk, 500).unwrap();
        assert_eq!(options.len(), 48);
        assert!(options[0].1.contains("—"), "got: {:?}", options[0]);

        // indexes: INTEGER PRIMARY KEY (rowid alias) creates no separate
        // index, but a UNIQUE constraint does (sqlite_autoindex_*)
        let indexes = conn.table_indexes("users").unwrap();
        assert!(indexes.is_empty(), "got: {:?}", indexes);
        conn.run_sql("CREATE TABLE tagged (id INTEGER PRIMARY KEY, tag TEXT UNIQUE)")
            .unwrap();
        let indexes = conn.table_indexes("tagged").unwrap();
        assert!(
            indexes.iter().any(|ix| ix.unique && ix.columns == "tag"),
            "got: {:?}",
            indexes
        );
        // PRAGMA on a missing table yields an empty list, not an error
        assert!(conn.table_indexes("missing_table").unwrap().is_empty());

        // DDL fetch (SQLite)
        let ddl = conn.fetch_ddl("users").unwrap();
        assert!(ddl.contains("CREATE TABLE users"), "got: {ddl}");
        assert!(conn.fetch_ddl("missing_table").is_err());

        // DELETE by key (orders is not referenced by anything — deleting a
        // user would trip the FK constraint, which is correct behavior)
        let sql = build_delete(
            DbKind::Sqlite,
            "orders",
            &[("id".to_string(), "120".to_string())],
        )
        .unwrap();
        conn.run_sql(&sql).unwrap();
        let (_, _, total, _) = conn.query_page("orders", 0, None, None, true).unwrap();
        assert_eq!(total, 119);

        // FK constraint surfaces as an error for referenced rows
        let sql = build_delete(
            DbKind::Sqlite,
            "users",
            &[("id".to_string(), "48".to_string())],
        )
        .unwrap();
        assert!(conn.run_sql(&sql).is_err());

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_parse_csv() {
        // plain
        assert_eq!(
            parse_csv("a,b\nc,d\n"),
            vec![vec!["a", "b"], vec!["c", "d"]]
        );
        // quoted with comma, escaped quotes, newline inside quotes, \r\n
        let parsed =
            parse_csv("name,note\r\n\"Smith, J.\",\"said \"\"hi\"\"\"\n\"multi\nline\",2\n");
        assert_eq!(parsed[0], vec!["name", "note"]);
        assert_eq!(parsed[1], vec!["Smith, J.", "said \"hi\""]);
        assert_eq!(parsed[2], vec!["multi\nline", "2"]);
        // no trailing newline
        assert_eq!(parse_csv("1,2"), vec![vec!["1", "2"]]);
        // empty fields
        assert_eq!(parse_csv("a,,c\n"), vec![vec!["a", "", "c"]]);
    }

    #[test]
    fn test_build_insert_rows_mapping() {
        let cols = vec![
            ColumnInfo {
                name: "id".into(),
                data_type: "integer".into(),
                nullable: true,
                is_key: true,
            },
            ColumnInfo {
                name: "name".into(),
                data_type: "text".into(),
                nullable: false,
                is_key: false,
            },
            ColumnInfo {
                name: "age".into(),
                data_type: "integer".into(),
                nullable: true,
                is_key: false,
            },
        ];
        // header order differs from table order; case-insensitive match
        let header = vec!["NAME".to_string(), "Age".to_string()];
        let rows = vec![
            vec!["Ann".to_string(), "30".to_string()],
            vec!["O'Brien".to_string(), String::new()],
        ];
        let (sql, count) =
            build_insert_rows(DbKind::Sqlite, "users", &cols, &header, &rows).unwrap();
        assert_eq!(count, 2);
        // column names are echoed as written in the CSV header
        assert!(
            sql.starts_with("INSERT INTO \"users\" (\"NAME\", \"Age\") VALUES"),
            "got: {sql}"
        );
        assert!(sql.contains("O''Brien"), "got: {sql}");
        assert!(sql.contains(", NULL)"), "got: {sql}");
        // unknown column → error
        let bad = vec!["nope".to_string()];
        assert!(build_insert_rows(DbKind::Sqlite, "users", &cols, &bad, &rows).is_err());
        // empty header → error
        assert!(build_insert_rows(DbKind::Sqlite, "users", &cols, &[], &rows).is_err());
    }

    #[test]
    fn test_insert_copy_roundtrip() {
        let path = temp_db("insertcopy");
        ensure_demo_db(&path);
        let cfg = ConnectionConfig {
            id: 1,
            name: "t".into(),
            kind: DbKind::Sqlite,
            host: String::new(),
            port: 0,
            user: String::new(),
            database: String::new(),
            path: path.clone(),
            group: String::new(),
            ssh_host: String::new(),
            ssh_user: String::new(),
            ssh_port: 22,
        };
        let mut conn = DbConn::connect(&cfg, "").unwrap();

        let (cols, rows, total_before, _) = conn.query_page("users", 0, None, None, true).unwrap();
        assert!(!rows.is_empty());

        // duplicate row 0 — the integer PK is omitted, so this must succeed
        let sql = build_insert_copy(DbKind::Sqlite, "users", &cols, &rows[0]).unwrap();
        assert!(!sql.contains("INSERT INTO \"users\" (\"id\""), "got: {sql}");
        conn.run_sql(&sql).unwrap();

        let (_, _, total_after, _) = conn.query_page("users", 0, None, None, true).unwrap();
        assert_eq!(total_after, total_before + 1);

        // the copy carries the same name but a fresh id
        let (_, rows2, _, _) = conn.query_page("users", 0, None, None, true).unwrap();
        let name_count = rows2.iter().filter(|r| r[1] == rows[0][1]).count();
        assert!(name_count >= 2);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_sql_builders() {
        let pk = vec![("id".to_string(), "7".to_string())];

        let u = build_update(DbKind::Sqlite, "users", "name", Some("O'Brien"), &pk).unwrap();
        assert_eq!(
            u,
            "UPDATE \"users\" SET \"name\" = 'O''Brien' WHERE \"id\" = '7'"
        );

        let n = build_update(DbKind::Sqlite, "users", "age", None, &pk).unwrap();
        assert!(n.contains("\"age\" = NULL"));

        // MySQL escapes backslashes too
        let m = build_update(DbKind::Mysql, "orders", "memo", Some("a\\b'c"), &pk).unwrap();
        assert!(m.contains("'a\\\\b''c'"), "got: {m}");

        let d = build_delete(DbKind::Postgres, "orders", &pk).unwrap();
        assert_eq!(d, "DELETE FROM \"orders\" WHERE \"id\" = '7'");

        // composite key joins with AND
        let pk2 = vec![
            ("a".to_string(), "1".to_string()),
            ("b".to_string(), "2".to_string()),
        ];
        let u2 = build_update(DbKind::Postgres, "t", "c", Some("x"), &pk2).unwrap();
        assert!(u2.contains("\"a\" = '1' AND \"b\" = '2'"));

        // no key → no SQL (read-only)
        assert!(build_update(DbKind::Sqlite, "t", "c", Some("x"), &[]).is_none());
        assert!(build_delete(DbKind::Sqlite, "t", &[]).is_none());
    }

    #[test]
    fn test_connection_persistence_roundtrip() {
        let dir = std::env::temp_dir().join(format!("dbpro-cfg-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let cfgs = vec![ConnectionConfig {
            id: 7,
            name: "demo".into(),
            kind: DbKind::Sqlite,
            host: String::new(),
            port: 0,
            user: String::new(),
            database: String::new(),
            path: "/tmp/x.db".into(),
            group: String::new(),
            ssh_host: String::new(),
            ssh_user: String::new(),
            ssh_port: 22,
        }];
        let json = serde_json::to_string_pretty(&cfgs).unwrap();
        let p = dir.join("connections.json");
        std::fs::write(&p, json).unwrap();
        let text = std::fs::read_to_string(&p).unwrap();
        let loaded: Vec<ConnectionConfig> = serde_json::from_str(&text).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "demo");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
