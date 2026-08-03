//! SQL migrations: up/down, idempotent, versioned.
//!
//! Migrations live in `migrations/NNNN_name/up.sql` and
//! `migrations/NNNN_name/down.sql`. Applied versions are tracked in the
//! `schema_migrations` table, so running `migrate` repeatedly is a no-op once
//! the schema is current.

use std::{
    fs,
    path::{Path, PathBuf},
};

use libsql::{Connection, Database};
use thiserror::Error;

/// One versioned migration.
#[derive(Debug, Clone)]
pub struct Migration {
    /// Sortable version number, derived from the `NNNN` directory prefix.
    pub version: i64,
    /// Human-readable migration name.
    pub name: String,
    /// Forward SQL.
    pub up: String,
    /// Reverse SQL.
    pub down: String,
}

/// Migration errors.
#[derive(Debug, Error)]
pub enum MigrateError {
    /// The migrations directory does not exist.
    #[error("migrations directory not found: {0}")]
    MissingDir(PathBuf),
    /// A migration directory name does not match `NNNN_name`.
    #[error("invalid migration directory name: {0}")]
    InvalidName(String),
    /// A migration is missing its `up.sql` or `down.sql`.
    #[error("migration {0} is missing {1}")]
    MissingFile(String, &'static str),
    /// A migration file could not be read.
    #[error("failed to read migration file {0}: {1}")]
    ReadFile(PathBuf, #[source] std::io::Error),
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// A migration failed while applying.
    #[error("migration {0} failed: {1}")]
    Apply(String, String),
}

/// Loads and sorts the migrations under `dir`.
///
/// # Errors
///
/// Returns [`MigrateError`] when the directory is missing, a name is
/// malformed, or a file cannot be read.
pub fn load_migrations(dir: impl AsRef<Path>) -> Result<Vec<Migration>, MigrateError> {
    let dir = dir.as_ref();
    let entries = fs::read_dir(dir).map_err(|_| MigrateError::MissingDir(dir.to_path_buf()))?;

    let mut migrations = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| MigrateError::ReadFile(dir.to_path_buf(), error))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let (prefix, name) = name
            .split_once('_')
            .ok_or_else(|| MigrateError::InvalidName(name.clone()))?;
        let version = prefix.parse::<i64>().map_err(|_| {
            MigrateError::InvalidName(entry.file_name().to_string_lossy().into_owned())
        })?;

        let up_path = path.join("up.sql");
        let down_path = path.join("down.sql");
        let up = fs::read_to_string(&up_path)
            .map_err(|error| MigrateError::ReadFile(up_path.clone(), error))?;
        let down = fs::read_to_string(&down_path)
            .map_err(|error| MigrateError::ReadFile(down_path.clone(), error))?;

        migrations.push(Migration {
            version,
            name: name.to_owned(),
            up,
            down,
        });
    }

    migrations.sort_by_key(|migration| migration.version);
    Ok(migrations)
}

/// Applies all pending migrations, creating the `schema_migrations` table
/// first. Idempotent: already-applied versions are skipped.
///
/// # Errors
///
/// Returns [`MigrateError`] on any database or file failure. Each migration
/// runs in its own transaction and rolls back on failure.
pub async fn migrate(db: &Database) -> Result<Vec<String>, MigrateError> {
    let conn = db.connect()?;
    migrate_conn(&conn).await
}

/// Like [`migrate`], but reuses an existing connection.
///
/// # Errors
///
/// See [`migrate`].
pub async fn migrate_conn(conn: &Connection) -> Result<Vec<String>, MigrateError> {
    conn.execute("PRAGMA foreign_keys = ON", ()).await?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
        )",
        (),
    )
    .await?;

    let applied = applied_versions(conn).await?;
    let migrations = load_migrations(Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations"))?;

    let mut applied_now = Vec::new();
    for migration in migrations.iter().filter(|m| !applied.contains(&m.version)) {
        let tx = conn.transaction().await?;
        tx.execute_batch(&migration.up)
            .await
            .map_err(|error| MigrateError::Apply(migration.name.clone(), error.to_string()))?;
        tx.execute(
            "INSERT INTO schema_migrations (version, name) VALUES (?1, ?2)",
            (migration.version, migration.name.clone()),
        )
        .await?;
        tx.commit().await?;
        tracing::info!(version = migration.version, name = %migration.name, "applied migration");
        applied_now.push(migration.name.clone());
    }
    Ok(applied_now)
}

/// Rolls back applied migrations down to (but not including) `to_version`.
/// Returns the names of the migrations that were rolled back.
///
/// # Errors
///
/// Returns [`MigrateError`] on any database or file failure. Each migration
/// runs in its own transaction and rolls back on failure.
pub async fn migrate_down(db: &Database, to_version: i64) -> Result<Vec<String>, MigrateError> {
    let conn = db.connect()?;
    conn.execute("PRAGMA foreign_keys = ON", ()).await?;

    let applied = applied_versions(&conn).await?;
    let migrations = load_migrations(Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations"))?;
    let mut rolled_back = Vec::new();

    for migration in migrations
        .iter()
        .filter(|m| m.version > to_version && applied.contains(&m.version))
        .rev()
    {
        let tx = conn.transaction().await?;
        tx.execute_batch(&migration.down)
            .await
            .map_err(|error| MigrateError::Apply(migration.name.clone(), error.to_string()))?;
        tx.execute(
            "DELETE FROM schema_migrations WHERE version = ?1",
            libsql::params![migration.version],
        )
        .await?;
        tx.commit().await?;
        tracing::info!(version = migration.version, name = %migration.name, "rolled back migration");
        rolled_back.push(migration.name.clone());
    }
    Ok(rolled_back)
}

async fn applied_versions(conn: &Connection) -> Result<Vec<i64>, MigrateError> {
    let mut rows = conn
        .query("SELECT version FROM schema_migrations ORDER BY version", ())
        .await?;
    let mut versions = Vec::new();
    while let Some(row) = rows.next().await? {
        versions.push(row.get(0)?);
    }
    Ok(versions)
}
