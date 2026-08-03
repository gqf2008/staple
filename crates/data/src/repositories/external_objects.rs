//! Issue external object links with refreshable status.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `issue_external_objects` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalObjectRecord {
    /// Link id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Issue id.
    pub issue_id: String,
    /// Kind (e.g. `github_pr`).
    pub kind: String,
    /// External id.
    pub external_id: String,
    /// URL.
    pub url: Option<String>,
    /// Status.
    pub status: String,
    /// ISO 8601 last sync time.
    pub last_synced_at: Option<String>,
    /// Metadata JSON.
    pub metadata: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// Input for linking an external object.
#[derive(Debug, Clone)]
pub struct NewExternalObject {
    /// Issue id.
    pub issue_id: String,
    /// Kind.
    pub kind: String,
    /// External id.
    pub external_id: String,
    /// URL.
    pub url: Option<String>,
    /// Metadata JSON.
    pub metadata: Option<String>,
}

/// External object repository errors.
#[derive(Debug, Error)]
pub enum ExternalObjectError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The issue does not exist.
    #[error("issue not found")]
    IssueNotFound,
    /// The link already exists.
    #[error("external object link already exists")]
    AlreadyExists,
}

/// External object persistence contract.
#[async_trait]
pub trait ExternalObjectRepository: Send + Sync {
    /// Creates a link.
    ///
    /// # Errors
    ///
    /// Returns [`ExternalObjectError`] on invalid references or duplicates.
    async fn create(
        &self,
        input: NewExternalObject,
    ) -> Result<ExternalObjectRecord, ExternalObjectError>;

    /// Lists links for an issue.
    ///
    /// # Errors
    ///
    /// Returns [`ExternalObjectError`] on database failure.
    async fn list_for_issue(
        &self,
        issue_id: &str,
    ) -> Result<Vec<ExternalObjectRecord>, ExternalObjectError>;

    /// Refreshes a link's status and sync time.
    ///
    /// # Errors
    ///
    /// Returns [`ExternalObjectError`] on database failure.
    async fn refresh(
        &self,
        id: &str,
        status: &str,
    ) -> Result<Option<ExternalObjectRecord>, ExternalObjectError>;
}

/// Turso/libSQL implementation of [`ExternalObjectRepository`].
#[derive(Debug)]
pub struct TursoExternalObjectRepository {
    db: Database,
}

impl TursoExternalObjectRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ExternalObjectRepository for TursoExternalObjectRepository {
    async fn create(
        &self,
        input: NewExternalObject,
    ) -> Result<ExternalObjectRecord, ExternalObjectError> {
        let conn = crate::connection::connect(&self.db).await?;
        let Some(company_id) = helpers::row_company(&conn, "issues", &input.issue_id).await? else {
            return Err(ExternalObjectError::IssueNotFound);
        };
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO issue_external_objects (id, company_id, issue_id, kind, external_id,
                                                     url, status, metadata, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending', ?7,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    company_id,
                    input.issue_id,
                    input.kind,
                    input.external_id,
                    input.url,
                    input.metadata
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        "SELECT id, company_id, issue_id, kind, external_id, url, status,
                                last_synced_at, metadata, created_at
                         FROM issue_external_objects WHERE id = ?1",
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("link was just inserted");
                Ok(row_to_object(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(ExternalObjectError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_for_issue(
        &self,
        issue_id: &str,
    ) -> Result<Vec<ExternalObjectRecord>, ExternalObjectError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, issue_id, kind, external_id, url, status,
                        last_synced_at, metadata, created_at
                 FROM issue_external_objects WHERE issue_id = ?1 ORDER BY created_at",
                libsql::params![issue_id],
            )
            .await?;
        let mut objects = Vec::new();
        while let Some(row) = rows.next().await? {
            objects.push(row_to_object(&row)?);
        }
        Ok(objects)
    }

    async fn refresh(
        &self,
        id: &str,
        status: &str,
    ) -> Result<Option<ExternalObjectRecord>, ExternalObjectError> {
        let conn = crate::connection::connect(&self.db).await?;
        let updated = conn
            .execute(
                "UPDATE issue_external_objects
                 SET status = ?1, last_synced_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE id = ?2",
                libsql::params![status, id],
            )
            .await?;
        if updated == 0 {
            return Ok(None);
        }
        let mut rows = conn
            .query(
                "SELECT id, company_id, issue_id, kind, external_id, url, status,
                        last_synced_at, metadata, created_at
                 FROM issue_external_objects WHERE id = ?1",
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("link exists");
        Ok(Some(row_to_object(&row)?))
    }
}

fn row_to_object(row: &libsql::Row) -> Result<ExternalObjectRecord, libsql::Error> {
    Ok(ExternalObjectRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        issue_id: helpers::row_text(row, 2)?.expect("issue_id"),
        kind: helpers::row_text(row, 3)?.expect("kind"),
        external_id: helpers::row_text(row, 4)?.expect("external_id"),
        url: helpers::row_text(row, 5)?,
        status: helpers::row_text(row, 6)?.expect("status"),
        last_synced_at: helpers::row_text(row, 7)?,
        metadata: helpers::row_text(row, 8)?,
        created_at: helpers::row_text(row, 9)?.expect("created_at"),
    })
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoExternalObjectRepository) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let conn = crate::connect(&db).await.unwrap();
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO issues (id, company_id, title, issue_number, identifier)
             VALUES ('i1', 'c1', 'T', 1, 'ALPHA-1')",
            (),
        )
        .await
        .unwrap();
        let repo = TursoExternalObjectRepository::new(db);
        (dir, repo)
    }

    #[tokio::test]
    async fn create_list_refresh_roundtrip() {
        let (_dir, repo) = repo().await;
        let created = repo
            .create(NewExternalObject {
                issue_id: "i1".to_owned(),
                kind: "github_pr".to_owned(),
                external_id: "123".to_owned(),
                url: Some("https://github.com/x/y/pull/123".to_owned()),
                metadata: None,
            })
            .await
            .unwrap();
        assert_eq!(created.status, "pending");

        let error = repo
            .create(NewExternalObject {
                issue_id: "i1".to_owned(),
                kind: "github_pr".to_owned(),
                external_id: "123".to_owned(),
                url: None,
                metadata: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(error, ExternalObjectError::AlreadyExists));

        let refreshed = repo.refresh(&created.id, "merged").await.unwrap().unwrap();
        assert_eq!(refreshed.status, "merged");
        assert!(refreshed.last_synced_at.is_some());

        let list = repo.list_for_issue("i1").await.unwrap();
        assert_eq!(list.len(), 1);
    }
}
