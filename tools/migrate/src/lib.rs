//! Postgres → Turso row-level snapshot migration library.
//!
//! See the binary help for commands; the functions here are also used by the
//! integration tests.

use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result, bail};
use serde_json::{Map, Value};

/// Tables in FK-safe order (parents before children).
pub const TABLE_ORDER: &[&str] = &[
    "companies",
    "agents",
    "agent_api_keys",
    "goals",
    "projects",
    "issues",
    "issue_comments",
    "heartbeat_runs",
    "cost_events",
    "approvals",
    "activity_log",
    "project_memberships",
    "agent_memberships",
    "company_secrets",
    "company_secret_versions",
    "assets",
    "issue_attachments",
    "documents",
    "document_revisions",
    "issue_documents",
    "issue_relations",
    "issue_work_products",
    "decision_queues",
    "decision_queue_items",
    "decision_triage",
    "issue_external_objects",
    "company_skills",
];

/// A snapshot: table name → rows (each row is a column-name → value map).
type Snapshot = HashMap<String, Vec<Map<String, Value>>>;

pub fn load_snapshot(path: &PathBuf) -> Result<Snapshot> {
    let raw =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&raw).context("parsing snapshot JSON")
}

/// Opens a local libsql database or errors for remote URLs.
pub async fn open_local(path: &str) -> Result<libsql::Database> {
    if path.starts_with("postgres://")
        || path.starts_with("libsql://")
        || path.starts_with("https://")
    {
        bail!("this command needs a local database path, got {path}");
    }
    libsql::Builder::new_local(path)
        .build()
        .await
        .map_err(|error| anyhow::anyhow!("cannot open {path}: {error}"))
}

/// Exports all tables in FK-safe order.
pub async fn export(source: &str) -> Result<Snapshot> {
    let db = open_local(source).await?;
    let conn = db.connect()?;
    let mut snapshot = Snapshot::new();
    for table in TABLE_ORDER {
        let sql = format!("SELECT * FROM {table} ORDER BY id");
        let mut rows = conn.query(&sql, ()).await?;
        let mut table_rows = Vec::new();
        while let Some(row) = rows.next().await? {
            let mut map = Map::new();
            for idx in 0..row.column_count() {
                let name = row.column_name(idx).unwrap_or_default().to_owned();
                let value = row.get_value(idx)?;
                let json = match value {
                    libsql::Value::Null => Value::Null,
                    libsql::Value::Integer(i) => Value::Number(i.into()),
                    libsql::Value::Real(f) => serde_json::Number::from_f64(f)
                        .map(Value::Number)
                        .unwrap_or(Value::Null),
                    libsql::Value::Text(t) => Value::String(t),
                    libsql::Value::Blob(b) => {
                        Value::String(String::from_utf8_lossy(&b).into_owned())
                    }
                };
                map.insert(name, json);
            }
            table_rows.push(map);
        }
        snapshot.insert((*table).to_owned(), table_rows);
    }
    Ok(snapshot)
}

/// Exports all tables from a Postgres database into the same snapshot shape
/// as [`export`] (row-level, FK-safe order). Runs on a blocking thread.
pub async fn export_postgres(source: &str) -> Result<Snapshot> {
    let url = source.to_owned();
    tokio::task::spawn_blocking(move || -> Result<Snapshot> {
        let mut client =
            postgres::Client::connect(&url, postgres::NoTls).context("connecting to Postgres")?;
        let mut snapshot = Snapshot::new();
        for table in TABLE_ORDER {
            let rows = client
                .query(&format!("SELECT * FROM {table} ORDER BY id"), &[])
                .with_context(|| format!("selecting from {table}"))?;
            let mut table_rows = Vec::new();
            for row in &rows {
                let mut map = Map::new();
                for (idx, column) in row.columns().iter().enumerate() {
                    map.insert(
                        column.name().to_owned(),
                        pg_row_value(row, idx, column.type_().name()),
                    );
                }
                table_rows.push(map);
            }
            snapshot.insert((*table).to_owned(), table_rows);
        }
        Ok(snapshot)
    })
    .await
    .context("Postgres export task panicked")?
}

/// Converts one Postgres cell into the snapshot JSON representation.
fn pg_row_value(row: &postgres::Row, idx: usize, type_name: &str) -> Value {
    match type_name {
        "bool" => match row.try_get::<_, bool>(idx) {
            Ok(value) => Value::Bool(value),
            Err(_) => Value::Null,
        },
        "int2" | "int4" | "int8" | "oid" => match row.try_get::<_, i64>(idx) {
            Ok(value) => Value::Number(value.into()),
            Err(_) => Value::Null,
        },
        "float4" | "float8" => match row.try_get::<_, f64>(idx) {
            Ok(value) => serde_json::Number::from_f64(value)
                .map(Value::Number)
                .unwrap_or(Value::Null),
            Err(_) => Value::Null,
        },
        "numeric" => match row.try_get::<_, String>(idx) {
            Ok(value) => Value::String(value),
            Err(_) => match row.try_get::<_, f64>(idx) {
                Ok(value) => serde_json::Number::from_f64(value)
                    .map(Value::Number)
                    .unwrap_or(Value::Null),
                Err(_) => Value::Null,
            },
        },
        "json" | "jsonb" => match row.try_get::<_, serde_json::Value>(idx) {
            Ok(value) => value,
            Err(_) => match row.try_get::<_, String>(idx) {
                Ok(value) => Value::String(value),
                Err(_) => Value::Null,
            },
        },
        "bytea" => match row.try_get::<_, Vec<u8>>(idx) {
            Ok(value) => Value::String(String::from_utf8_lossy(&value).into_owned()),
            Err(_) => Value::Null,
        },
        "uuid" => match row.try_get::<_, uuid::Uuid>(idx) {
            Ok(value) => Value::String(value.to_string()),
            Err(_) => Value::Null,
        },
        _ => match row.try_get::<_, String>(idx) {
            Ok(value) => Value::String(value),
            Err(_) => Value::Null,
        },
    }
}

/// Imports a snapshot into a Turso database, running migrations first.
pub async fn import(target: &str, snapshot: &Snapshot) -> Result<HashMap<String, usize>> {
    let db = open_local(target).await?;
    // Run the data-layer migrations (schema + indexes + FKs).
    let migrations = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("crates/data/migrations");
    let conn = db.connect()?;
    conn.execute("PRAGMA foreign_keys = ON", ()).await?;
    apply_migrations(&conn, &migrations).await?;

    let mut counts = HashMap::new();
    for table in TABLE_ORDER {
        // Only insert columns the target schema actually defines; snapshot
        // columns that are schema drift are skipped so a richer source (for
        // example Postgres with newer upstream columns) can still import.
        // Column info: name -> (notnull, has_default). Used to skip NULL
        // values for columns the target schema can default itself.
        let mut target_columns = std::collections::HashMap::<String, (bool, bool)>::new();
        let mut info = conn
            .query(&format!("PRAGMA table_info({table})"), ())
            .await?;
        while let Some(row) = info.next().await? {
            let notnull: bool = row.get::<i64>(3)? != 0;
            let dflt: Option<String> = row.get(4)?;
            target_columns.insert(row.get::<String>(1)?, (notnull, dflt.is_some()));
        }
        let rows = snapshot.get(*table).cloned().unwrap_or_default();
        for row in &rows {
            let columns: Vec<&String> = row
                .keys()
                .filter(|column| {
                    target_columns
                        .get(column.as_str())
                        .map(|(notnull, has_default)| {
                            // Keep the column unless it is NULL and the target
                            // schema can supply its own default.
                            let is_null = row.get(*column).is_none() || row[*column].is_null();
                            !(is_null && (*has_default || *notnull))
                        })
                        .unwrap_or(false)
                })
                .collect();
            let placeholders: Vec<String> = (1..=columns.len()).map(|i| format!("?{i}")).collect();
            let sql = format!(
                "INSERT INTO {table} ({}) VALUES ({})",
                columns
                    .iter()
                    .map(|c| c.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                placeholders.join(", ")
            );
            let params: Vec<libsql::Value> = columns
                .iter()
                .map(|column| json_to_value(&row[*column]))
                .collect();
            conn.execute(&sql, params)
                .await
                .with_context(|| format!("inserting into {table} (row id {:?})", row.get("id")))?;
        }
        counts.insert(table.to_string(), rows.len());
    }
    Ok(counts)
}

/// Converts a JSON value into a libsql bind value (bool → 0/1, numbers →
/// integers when whole).
pub fn json_to_value(value: &Value) -> libsql::Value {
    match value {
        Value::Null => libsql::Value::Null,
        Value::Bool(b) => libsql::Value::Integer(i64::from(*b)),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                libsql::Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                libsql::Value::Real(f)
            } else {
                libsql::Value::Null
            }
        }
        Value::String(s) => libsql::Value::Text(s.clone()),
        Value::Array(_) | Value::Object(_) => libsql::Value::Text(value.to_string()),
    }
}

/// Applies every `NNNN_name/up.sql` migration under `dir`.
pub async fn apply_migrations(conn: &libsql::Connection, dir: &PathBuf) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
        )",
        (),
    )
    .await?;
    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .collect();
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        let version = name
            .split('_')
            .next()
            .and_then(|prefix| prefix.parse::<i64>().ok())
            .unwrap_or(0);
        let mut rows = conn
            .query(
                "SELECT 1 FROM schema_migrations WHERE version = ?1",
                libsql::params![version],
            )
            .await?;
        if rows.next().await?.is_some() {
            continue;
        }
        let up = std::fs::read_to_string(entry.path().join("up.sql"))?;
        conn.execute_batch(&up).await?;
        conn.execute(
            "INSERT INTO schema_migrations (version, name) VALUES (?1, ?2)",
            libsql::params![version, name],
        )
        .await?;
    }
    Ok(())
}

/// Compares snapshot row counts against the database.
pub async fn verify(target: &str, snapshot: &Snapshot) -> Result<()> {
    let db = open_local(target).await?;
    let conn = db.connect()?;
    let mut mismatches = Vec::new();
    for table in TABLE_ORDER {
        let expected = snapshot.get(*table).map(Vec::len).unwrap_or(0);
        let mut rows = conn
            .query(&format!("SELECT COUNT(*) FROM {table}"), ())
            .await?;
        let actual = match rows.next().await? {
            Some(row) => row.get::<i64>(0)?,
            None => 0,
        };
        if actual as usize != expected {
            mismatches.push(format!("{table}: expected {expected}, got {actual}"));
        } else {
            println!("{table}: {actual} rows ✓");
        }
    }
    if mismatches.is_empty() {
        println!("verify OK");
    } else {
        bail!("row count mismatches: {}", mismatches.join("; "));
    }
    Ok(())
}
