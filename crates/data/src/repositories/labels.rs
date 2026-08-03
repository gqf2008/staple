//! Labels repository: company labels and issue-label links.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `labels` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelRecord {
    /// Label id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Name (unique per company).
    pub name: String,
    /// Color.
    pub color: String,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// A label attached to an issue.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueLabelRecord {
    /// Issue id.
    pub issue_id: String,
    /// Label id.
    pub label_id: String,
    /// Owning company id.
    pub company_id: String,
}

/// Input for creating a label.
#[derive(Debug, Clone)]
pub struct NewLabel {
    /// Owning company id.
    pub company_id: String,
    /// Name.
    pub name: String,
    /// Color.
    pub color: String,
}

/// Label repository errors.
#[derive(Debug, Error)]
pub enum LabelError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The company does not exist.
    #[error("company not found")]
    CompanyNotFound,
    /// The label name is already taken in this company.
    #[error("label already exists")]
    AlreadyExists,
    /// The label or issue does not exist in this company.
    #[error("label or issue not found")]
    NotFound,
    /// The label is already attached to the issue.
    #[error("label already attached")]
    AlreadyAttached,
}

/// Label persistence contract.
#[async_trait]
pub trait LabelRepository: Send + Sync {
    /// Creates a label.
    ///
    /// # Errors
    ///
    /// Returns [`LabelError`] on invalid references or duplicates.
    async fn create(&self, input: NewLabel) -> Result<LabelRecord, LabelError>;

    /// Lists labels for a company.
    ///
    /// # Errors
    ///
    /// Returns [`LabelError`] on database failure.
    async fn list(&self, company_id: &str) -> Result<Vec<LabelRecord>, LabelError>;

    /// Attaches a label to an issue (same company enforced; the company is
    /// resolved from the issue).
    ///
    /// # Errors
    ///
    /// Returns [`LabelError`] on invalid references or duplicates.
    async fn attach(&self, issue_id: &str, label_id: &str) -> Result<IssueLabelRecord, LabelError>;

    /// Lists labels attached to an issue.
    ///
    /// # Errors
    ///
    /// Returns [`LabelError`] on database failure.
    async fn list_for_issue(&self, issue_id: &str) -> Result<Vec<LabelRecord>, LabelError>;

    /// Detaches a label from an issue.
    ///
    /// # Errors
    ///
    /// Returns [`LabelError`] on database failure.
    async fn detach(&self, issue_id: &str, label_id: &str) -> Result<(), LabelError>;
}

/// Turso/libSQL implementation of [`LabelRepository`].
#[derive(Debug)]
pub struct TursoLabelRepository {
    db: Database,
}

impl TursoLabelRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl LabelRepository for TursoLabelRepository {
    async fn create(&self, input: NewLabel) -> Result<LabelRecord, LabelError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(LabelError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO labels (id, company_id, name, color, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![id.clone(), input.company_id, input.name, input.color],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        "SELECT id, company_id, name, color, created_at FROM labels WHERE id = ?1",
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("label was just inserted");
                Ok(LabelRecord {
                    id: helpers::row_text(&row, 0)?.expect("id"),
                    company_id: helpers::row_text(&row, 1)?.expect("company_id"),
                    name: helpers::row_text(&row, 2)?.expect("name"),
                    color: helpers::row_text(&row, 3)?.expect("color"),
                    created_at: helpers::row_text(&row, 4)?.expect("created_at"),
                })
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(LabelError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list(&self, company_id: &str) -> Result<Vec<LabelRecord>, LabelError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, name, color, created_at FROM labels
                 WHERE company_id = ?1 ORDER BY name",
                libsql::params![company_id],
            )
            .await?;
        let mut labels = Vec::new();
        while let Some(row) = rows.next().await? {
            labels.push(LabelRecord {
                id: helpers::row_text(&row, 0)?.expect("id"),
                company_id: helpers::row_text(&row, 1)?.expect("company_id"),
                name: helpers::row_text(&row, 2)?.expect("name"),
                color: helpers::row_text(&row, 3)?.expect("color"),
                created_at: helpers::row_text(&row, 4)?.expect("created_at"),
            });
        }
        Ok(labels)
    }

    async fn attach(&self, issue_id: &str, label_id: &str) -> Result<IssueLabelRecord, LabelError> {
        let conn = crate::connection::connect(&self.db).await?;
        let Some(company_id) = helpers::row_company(&conn, "issues", issue_id).await? else {
            return Err(LabelError::NotFound);
        };
        if !helpers::row_belongs_to_company(&conn, "labels", label_id, &company_id).await? {
            return Err(LabelError::NotFound);
        }
        let result = conn
            .execute(
                "INSERT INTO issue_labels (issue_id, label_id, company_id, created_at)
                 VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![issue_id, label_id, company_id.clone()],
            )
            .await;
        match result {
            Ok(_) => Ok(IssueLabelRecord {
                issue_id: issue_id.to_owned(),
                label_id: label_id.to_owned(),
                company_id: company_id.to_owned(),
            }),
            Err(error)
                if error.to_string().contains("UNIQUE constraint failed")
                    || error.to_string().contains("PRIMARY KEY") =>
            {
                Err(LabelError::AlreadyAttached)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_for_issue(&self, issue_id: &str) -> Result<Vec<LabelRecord>, LabelError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT l.id, l.company_id, l.name, l.color, l.created_at
                 FROM issue_labels il JOIN labels l ON l.id = il.label_id
                 WHERE il.issue_id = ?1 ORDER BY l.name",
                libsql::params![issue_id],
            )
            .await?;
        let mut labels = Vec::new();
        while let Some(row) = rows.next().await? {
            labels.push(LabelRecord {
                id: helpers::row_text(&row, 0)?.expect("id"),
                company_id: helpers::row_text(&row, 1)?.expect("company_id"),
                name: helpers::row_text(&row, 2)?.expect("name"),
                color: helpers::row_text(&row, 3)?.expect("color"),
                created_at: helpers::row_text(&row, 4)?.expect("created_at"),
            });
        }
        Ok(labels)
    }

    async fn detach(&self, issue_id: &str, label_id: &str) -> Result<(), LabelError> {
        let conn = crate::connection::connect(&self.db).await?;
        conn.execute(
            "DELETE FROM issue_labels WHERE issue_id = ?1 AND label_id = ?2",
            libsql::params![issue_id, label_id],
        )
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    #[tokio::test]
    async fn label_lifecycle() {
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
        let repo = TursoLabelRepository::new(db);

        let label = repo
            .create(NewLabel {
                company_id: "c1".to_owned(),
                name: "bug".to_owned(),
                color: "#dc2626".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(label.name, "bug");

        let error = repo
            .create(NewLabel {
                company_id: "c1".to_owned(),
                name: "bug".to_owned(),
                color: "#000000".to_owned(),
            })
            .await
            .unwrap_err();
        assert!(matches!(error, LabelError::AlreadyExists));

        repo.attach("i1", &label.id).await.unwrap();
        let error = repo.attach("i1", &label.id).await.unwrap_err();
        assert!(matches!(error, LabelError::AlreadyAttached));

        let labels = repo.list_for_issue("i1").await.unwrap();
        assert_eq!(labels.len(), 1);

        repo.detach("i1", &label.id).await.unwrap();
        assert!(repo.list_for_issue("i1").await.unwrap().is_empty());

        // Missing label rejected.
        let error = repo.attach("i1", "missing").await.unwrap_err();
        assert!(matches!(error, LabelError::NotFound));
    }
}
