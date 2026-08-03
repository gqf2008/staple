//! Turso/libSQL connection layer.
//!
//! Dev runs against an embedded file database by default (no `TURSO_URL`).
//! Production uses `TURSO_URL` plus `TURSO_AUTH_TOKEN`.

use std::{env, path::PathBuf};

use libsql::{Builder, Connection, Database};
use thiserror::Error;

const DEFAULT_LOCAL_PATH: &str = "data/staple.db";

/// Database configuration.
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// Remote Turso/libSQL URL (`libsql://...` or `https://...`). `None`
    /// selects the embedded file database.
    pub url: Option<String>,
    /// Bearer token for the remote database.
    pub auth_token: Option<String>,
    /// Local file path for the embedded database.
    pub local_path: PathBuf,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            url: None,
            auth_token: None,
            local_path: PathBuf::from(DEFAULT_LOCAL_PATH),
        }
    }
}

impl DbConfig {
    /// Loads configuration from the environment.
    ///
    /// - `TURSO_URL` / `TURSO_AUTH_TOKEN`: remote database
    /// - `STAPLE_DB_PATH`: embedded database file (default `data/staple.db`)
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            url: env::var("TURSO_URL").ok().filter(|value| !value.is_empty()),
            auth_token: env::var("TURSO_AUTH_TOKEN")
                .ok()
                .filter(|value| !value.is_empty()),
            local_path: env::var("STAPLE_DB_PATH")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from(DEFAULT_LOCAL_PATH)),
        }
    }

    /// Embedded local database at `path`.
    #[must_use]
    pub fn local(path: impl Into<PathBuf>) -> Self {
        Self {
            url: None,
            auth_token: None,
            local_path: path.into(),
        }
    }
}

/// Data-layer errors.
#[derive(Debug, Error)]
pub enum DataError {
    /// The database could not be opened.
    #[error("failed to open database: {0}")]
    Open(#[from] libsql::Error),
    /// The embedded database directory could not be created.
    #[error("failed to create database directory: {0}")]
    CreateDir(#[source] std::io::Error),
}

/// Opens the database described by `config`.
///
/// # Errors
///
/// Returns [`DataError`] when the database cannot be opened or the local
/// database directory cannot be created.
pub async fn open(config: &DbConfig) -> Result<Database, DataError> {
    match &config.url {
        Some(url) => {
            let token = config.auth_token.clone().unwrap_or_default();
            Ok(Builder::new_remote(url.clone(), token).build().await?)
        }
        None => {
            if let Some(parent) = config.local_path.parent()
                && !parent.as_os_str().is_empty()
            {
                std::fs::create_dir_all(parent).map_err(DataError::CreateDir)?;
            }
            Ok(Builder::new_local(&config.local_path).build().await?)
        }
    }
}

/// Opens a connection with `PRAGMA foreign_keys = ON`.
///
/// SQLite disables foreign keys by default; every connection must enable them
/// so the company-boundary constraints in the schema are enforced.
///
/// # Errors
///
/// Returns [`DataError`] when the connection cannot be opened or the pragma
/// cannot be applied.
pub async fn connect(db: &Database) -> Result<Connection, DataError> {
    let conn = db.connect()?;
    // SQLite returns SQLITE_BUSY immediately by default when another
    // connection holds a conflicting lock; wait briefly instead so concurrent
    // heartbeat transactions serialize cleanly.
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    conn.execute("PRAGMA foreign_keys = ON", ()).await?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_embedded() {
        let config = DbConfig::default();
        assert!(config.url.is_none());
        assert_eq!(config.local_path, PathBuf::from("data/staple.db"));
    }
}
