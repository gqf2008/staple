//! Issue relations (blockers) repository.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `issue_relations` table (`type = 'blocks'`).
#[derive(Debug, Clone)]
pub struct IssueRelationRecord {
    /// Relation id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Blocking issue id (the source).
    pub issue_id: String,
    /// Blocked issue id (the target).
    pub related_issue_id: String,
    /// Relation type (`blocks`).
    pub r#type: String,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// Input for adding a blocker relation.
#[derive(Debug, Clone)]
pub struct NewIssueRelation {
    /// Blocking issue id.
    pub issue_id: String,
    /// Blocked issue id.
    pub related_issue_id: String,
}

/// Relation repository errors.
#[derive(Debug, Error)]
pub enum IssueRelationError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// One of the issues does not exist or belongs to another company.
    #[error("issue not found")]
    IssueNotFound,
    /// The relation already exists.
    #[error("relation already exists")]
    AlreadyExists,
}

/// Issue relation persistence contract.
#[async_trait]
pub trait IssueRelationRepository: Send + Sync {
    /// Adds a blocker relation (`issue_id` blocks `related_issue_id`).
    ///
    /// # Errors
    ///
    /// Returns [`IssueRelationError`] when an issue is missing or the edge
    /// already exists.
    async fn add_blocker(
        &self,
        input: NewIssueRelation,
    ) -> Result<IssueRelationRecord, IssueRelationError>;

    /// Lists the blockers of an issue (issues that block it).
    ///
    /// # Errors
    ///
    /// Returns [`IssueRelationError`] on database failure.
    async fn list_blockers(
        &self,
        issue_id: &str,
    ) -> Result<Vec<IssueRelationRecord>, IssueRelationError>;

    /// Removes a blocker relation.
    ///
    /// # Errors
    ///
    /// Returns [`IssueRelationError`] on database failure.
    async fn remove_blocker(
        &self,
        id: &str,
    ) -> Result<Option<IssueRelationRecord>, IssueRelationError>;
}

/// Turso/libSQL implementation of [`IssueRelationRepository`].
#[derive(Debug)]
pub struct TursoIssueRelationRepository {
    db: Database,
}

impl TursoIssueRelationRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

const RELATION_COLUMNS: &str = "id, company_id, issue_id, related_issue_id, type, created_at";

fn row_to_relation(row: &libsql::Row) -> Result<IssueRelationRecord, libsql::Error> {
    Ok(IssueRelationRecord {
        id: helpers::row_text(row, 0)?.expect("id is NOT NULL"),
        company_id: helpers::row_text(row, 1)?.expect("company_id is NOT NULL"),
        issue_id: helpers::row_text(row, 2)?.expect("issue_id is NOT NULL"),
        related_issue_id: helpers::row_text(row, 3)?.expect("related_issue_id is NOT NULL"),
        r#type: helpers::row_text(row, 4)?.expect("type is NOT NULL"),
        created_at: helpers::row_text(row, 5)?.expect("created_at is NOT NULL"),
    })
}

#[async_trait]
impl IssueRelationRepository for TursoIssueRelationRepository {
    async fn add_blocker(
        &self,
        input: NewIssueRelation,
    ) -> Result<IssueRelationRecord, IssueRelationError> {
        let conn = crate::connection::connect(&self.db).await?;
        let Some(company_id) = helpers::row_company(&conn, "issues", &input.issue_id).await? else {
            return Err(IssueRelationError::IssueNotFound);
        };
        if !helpers::row_belongs_to_company(&conn, "issues", &input.related_issue_id, &company_id)
            .await?
        {
            return Err(IssueRelationError::IssueNotFound);
        }

        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO issue_relations (id, company_id, issue_id, related_issue_id,
                                              type, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 'blocks',
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    company_id,
                    input.issue_id,
                    input.related_issue_id
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!("SELECT {RELATION_COLUMNS} FROM issue_relations WHERE id = ?1"),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("relation was just inserted");
                Ok(row_to_relation(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(IssueRelationError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_blockers(
        &self,
        issue_id: &str,
    ) -> Result<Vec<IssueRelationRecord>, IssueRelationError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql = format!(
            "SELECT {RELATION_COLUMNS} FROM issue_relations WHERE related_issue_id = ?1 ORDER BY created_at"
        );
        let mut rows = conn.query(&sql, libsql::params![issue_id]).await?;
        let mut relations = Vec::new();
        while let Some(row) = rows.next().await? {
            relations.push(row_to_relation(&row)?);
        }
        Ok(relations)
    }

    async fn remove_blocker(
        &self,
        id: &str,
    ) -> Result<Option<IssueRelationRecord>, IssueRelationError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!("SELECT {RELATION_COLUMNS} FROM issue_relations WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let relation = row_to_relation(&row)?;
        conn.execute(
            "DELETE FROM issue_relations WHERE id = ?1",
            libsql::params![id],
        )
        .await?;
        Ok(Some(relation))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{Connection, migrate, open};

    async fn repo() -> (TempDir, TursoIssueRelationRepository, Connection) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let conn = crate::connect(&db).await.unwrap();
        let repo = TursoIssueRelationRepository::new(db);
        (dir, repo, conn)
    }

    #[tokio::test]
    async fn add_list_remove_roundtrip() {
        let (_dir, repo, conn) = repo().await;
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO issues (id, company_id, title, issue_number, identifier)
             VALUES ('i1', 'c1', 'blocking', 1, 'ALPHA-1'),
                    ('i2', 'c1', 'blocked', 2, 'ALPHA-2')",
            (),
        )
        .await
        .unwrap();

        let relation = repo
            .add_blocker(NewIssueRelation {
                issue_id: "i1".to_owned(),
                related_issue_id: "i2".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(relation.r#type, "blocks");

        // Duplicate rejected.
        let error = repo
            .add_blocker(NewIssueRelation {
                issue_id: "i1".to_owned(),
                related_issue_id: "i2".to_owned(),
            })
            .await
            .unwrap_err();
        assert!(matches!(error, IssueRelationError::AlreadyExists));

        let blockers = repo.list_blockers("i2").await.unwrap();
        assert_eq!(blockers.len(), 1);
        assert_eq!(blockers[0].issue_id, "i1");

        let removed = repo.remove_blocker(&relation.id).await.unwrap().unwrap();
        assert_eq!(removed.id, relation.id);
        assert!(repo.list_blockers("i2").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn add_requires_same_company_issues() {
        let (_dir, repo, conn) = repo().await;
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024), ('c2', 'Beta', 'BETA', 1024)",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO issues (id, company_id, title, issue_number, identifier)
             VALUES ('i1', 'c1', 'a', 1, 'ALPHA-1'), ('i2', 'c2', 'b', 1, 'BETA-1')",
            (),
        )
        .await
        .unwrap();
        let error = repo
            .add_blocker(NewIssueRelation {
                issue_id: "i1".to_owned(),
                related_issue_id: "i2".to_owned(),
            })
            .await
            .unwrap_err();
        assert!(matches!(error, IssueRelationError::IssueNotFound));
    }
}
