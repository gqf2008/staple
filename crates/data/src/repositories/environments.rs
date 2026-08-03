//! Environments repository (global environment pool, upstream-compatible).

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `environments` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentRecord {
    /// Environment id.
    pub id: String,
    /// Name (unique).
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Driver (`local` by default; only one `local` allowed).
    pub driver: String,
    /// Status.
    pub status: String,
    /// Config JSON.
    pub config: String,
    /// Env vars JSON.
    pub env_vars: String,
    /// Metadata JSON.
    pub metadata: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// Input for creating an environment.
#[derive(Debug, Clone)]
pub struct NewEnvironment {
    /// Name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Driver.
    pub driver: String,
    /// Config JSON.
    pub config: Option<String>,
}

/// Environment repository errors.
#[derive(Debug, Error)]
pub enum EnvironmentError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The environment name is already taken.
    #[error("environment already exists")]
    AlreadyExists,
    /// A `local` environment already exists (only one allowed).
    #[error("a local environment already exists")]
    LocalAlreadyExists,
}

/// Environment persistence contract.
#[async_trait]
pub trait EnvironmentRepository: Send + Sync {
    /// Creates an environment.
    ///
    /// # Errors
    ///
    /// Returns [`EnvironmentError`] on duplicate names or duplicate local
    /// drivers.
    async fn create(&self, input: NewEnvironment) -> Result<EnvironmentRecord, EnvironmentError>;

    /// Lists environments.
    ///
    /// # Errors
    ///
    /// Returns [`EnvironmentError`] on database failure.
    async fn list(&self) -> Result<Vec<EnvironmentRecord>, EnvironmentError>;

    /// Fetches one environment by id.
    ///
    /// # Errors
    ///
    /// Returns [`EnvironmentError`] on database failure.
    async fn get(&self, id: &str) -> Result<Option<EnvironmentRecord>, EnvironmentError>;

    /// Ensures a `local` environment exists (the upstream default), creating
    /// it when missing.
    ///
    /// # Errors
    ///
    /// Returns [`EnvironmentError`] on database failure.
    async fn ensure_local(&self) -> Result<EnvironmentRecord, EnvironmentError>;
}

/// Turso/libSQL implementation of [`EnvironmentRepository`].
#[derive(Debug)]
pub struct TursoEnvironmentRepository {
    db: Database,
}

impl TursoEnvironmentRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

const COLUMNS: &str =
    "id, name, description, driver, status, config, env_vars, metadata, created_at";

fn row_to_environment(row: &libsql::Row) -> Result<EnvironmentRecord, libsql::Error> {
    Ok(EnvironmentRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        name: helpers::row_text(row, 1)?.expect("name"),
        description: helpers::row_text(row, 2)?,
        driver: helpers::row_text(row, 3)?.expect("driver"),
        status: helpers::row_text(row, 4)?.expect("status"),
        config: helpers::row_text(row, 5)?.expect("config"),
        env_vars: helpers::row_text(row, 6)?.expect("env_vars"),
        metadata: helpers::row_text(row, 7)?,
        created_at: helpers::row_text(row, 8)?.expect("created_at"),
    })
}

#[async_trait]
impl EnvironmentRepository for TursoEnvironmentRepository {
    async fn create(&self, input: NewEnvironment) -> Result<EnvironmentRecord, EnvironmentError> {
        let conn = crate::connection::connect(&self.db).await?;
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO environments (id, name, description, driver, status, config,
                                          env_vars, metadata, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 'active', ?5, '{}', NULL,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.name,
                    input.description,
                    input.driver,
                    input.config.unwrap_or_else(|| "{}".to_owned())
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!("SELECT {COLUMNS} FROM environments WHERE id = ?1"),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("environment was just inserted");
                Ok(row_to_environment(&row)?)
            }
            Err(error) => {
                let message = error.to_string();
                if message.contains("UNIQUE constraint failed") {
                    if message.contains("environments.name") {
                        return Err(EnvironmentError::AlreadyExists);
                    }
                    // Partial unique index on `driver` (only one `local`).
                    return Err(EnvironmentError::LocalAlreadyExists);
                }
                Err(error.into())
            }
        }
    }

    async fn list(&self) -> Result<Vec<EnvironmentRecord>, EnvironmentError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!("SELECT {COLUMNS} FROM environments ORDER BY name"),
                (),
            )
            .await?;
        let mut environments = Vec::new();
        while let Some(row) = rows.next().await? {
            environments.push(row_to_environment(&row)?);
        }
        Ok(environments)
    }

    async fn get(&self, id: &str) -> Result<Option<EnvironmentRecord>, EnvironmentError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!("SELECT {COLUMNS} FROM environments WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_environment(&row)?)),
            None => Ok(None),
        }
    }

    async fn ensure_local(&self) -> Result<EnvironmentRecord, EnvironmentError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id FROM environments WHERE driver = 'local' LIMIT 1",
                (),
            )
            .await?;
        if let Some(row) = rows.next().await? {
            let id = helpers::row_text(&row, 0)?.expect("id");
            return Ok(self.get(&id).await?.expect("environment exists"));
        }
        self.create(NewEnvironment {
            name: "local".to_owned(),
            description: Some("Local default environment".to_owned()),
            driver: "local".to_owned(),
            config: None,
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    #[tokio::test]
    async fn create_list_get_and_local_ensure() {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let repo = TursoEnvironmentRepository::new(db);

        let created = repo
            .create(NewEnvironment {
                name: "prod".to_owned(),
                description: None,
                driver: "remote".to_owned(),
                config: None,
            })
            .await
            .unwrap();
        assert_eq!(created.status, "active");

        // Duplicate name rejected.
        let error = repo
            .create(NewEnvironment {
                name: "prod".to_owned(),
                description: None,
                driver: "remote".to_owned(),
                config: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(error, EnvironmentError::AlreadyExists));

        // Ensure local creates it once and is idempotent.
        let local = repo.ensure_local().await.unwrap();
        assert_eq!(local.driver, "local");
        let local2 = repo.ensure_local().await.unwrap();
        assert_eq!(local2.id, local.id);

        // Second local rejected.
        let error = repo
            .create(NewEnvironment {
                name: "other-local".to_owned(),
                description: None,
                driver: "local".to_owned(),
                config: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(error, EnvironmentError::LocalAlreadyExists));

        let list = repo.list().await.unwrap();
        assert_eq!(list.len(), 2);
    }
}
