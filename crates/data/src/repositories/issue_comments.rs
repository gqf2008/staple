//! Issue comments repository.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `issue_comments` table.
#[derive(Debug, Clone)]
pub struct IssueCommentRecord {
    /// Comment id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Issue id.
    pub issue_id: String,
    /// Author agent id.
    pub author_agent_id: Option<String>,
    /// Author user id.
    pub author_user_id: Option<String>,
    /// Comment body.
    pub body: String,
    /// ISO 8601 creation time.
    pub created_at: String,
    /// ISO 8601 last update time.
    pub updated_at: String,
}

/// Input for adding a comment.
#[derive(Debug, Clone)]
pub struct NewIssueComment {
    /// Issue id.
    pub issue_id: String,
    /// Author agent id.
    pub author_agent_id: Option<String>,
    /// Author user id.
    pub author_user_id: Option<String>,
    /// Comment body (non-empty).
    pub body: String,
}

/// Comment repository errors.
#[derive(Debug, Error)]
pub enum IssueCommentError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The issue does not exist or belongs to another company.
    #[error("issue not found")]
    IssueNotFound,
    /// The author agent does not belong to the issue's company.
    #[error("author agent belongs to a different company")]
    AuthorInDifferentCompany,
}

/// Comment persistence contract.
#[async_trait]
pub trait IssueCommentRepository: Send + Sync {
    /// Adds a comment to an issue.
    ///
    /// # Errors
    ///
    /// Returns [`IssueCommentError`] when the issue is missing.
    async fn create(&self, input: NewIssueComment)
    -> Result<IssueCommentRecord, IssueCommentError>;

    /// Lists all comments of an issue.
    ///
    /// # Errors
    ///
    /// Returns [`IssueCommentError`] on database failure.
    async fn list(&self, issue_id: &str) -> Result<Vec<IssueCommentRecord>, IssueCommentError>;

    /// Fetches one comment by id.
    ///
    /// # Errors
    ///
    /// Returns [`IssueCommentError`] on database failure.
    async fn get(&self, id: &str) -> Result<Option<IssueCommentRecord>, IssueCommentError>;

    /// Deletes a comment, returning the deleted row.
    ///
    /// # Errors
    ///
    /// Returns [`IssueCommentError`] on database failure.
    async fn delete(&self, id: &str) -> Result<Option<IssueCommentRecord>, IssueCommentError>;
}

/// Turso/libSQL implementation of [`IssueCommentRepository`].
#[derive(Debug)]
pub struct TursoIssueCommentRepository {
    db: Database,
}

impl TursoIssueCommentRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

const COMMENT_COLUMNS: &str = "id, company_id, issue_id, author_agent_id, author_user_id,
    body, created_at, updated_at";

fn row_to_comment(row: &libsql::Row) -> Result<IssueCommentRecord, libsql::Error> {
    Ok(IssueCommentRecord {
        id: helpers::row_text(row, 0)?.expect("id is NOT NULL"),
        company_id: helpers::row_text(row, 1)?.expect("company_id is NOT NULL"),
        issue_id: helpers::row_text(row, 2)?.expect("issue_id is NOT NULL"),
        author_agent_id: helpers::row_text(row, 3)?,
        author_user_id: helpers::row_text(row, 4)?,
        body: helpers::row_text(row, 5)?.expect("body is NOT NULL"),
        created_at: helpers::row_text(row, 6)?.expect("created_at is NOT NULL"),
        updated_at: helpers::row_text(row, 7)?.expect("updated_at is NOT NULL"),
    })
}

#[async_trait]
impl IssueCommentRepository for TursoIssueCommentRepository {
    async fn create(
        &self,
        input: NewIssueComment,
    ) -> Result<IssueCommentRecord, IssueCommentError> {
        let conn = crate::connection::connect(&self.db).await?;
        let Some(company_id) = helpers::row_company(&conn, "issues", &input.issue_id).await? else {
            return Err(IssueCommentError::IssueNotFound);
        };
        if let Some(author_agent_id) = &input.author_agent_id
            && !helpers::row_belongs_to_company(&conn, "agents", author_agent_id, &company_id)
                .await?
        {
            return Err(IssueCommentError::AuthorInDifferentCompany);
        }

        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO issue_comments (id, company_id, issue_id, author_agent_id,
                                         author_user_id, body, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                company_id,
                input.issue_id,
                input.author_agent_id,
                input.author_user_id,
                input.body
            ],
        )
        .await?;
        Ok(self.get(&id).await?.expect("comment was just inserted"))
    }

    async fn list(&self, issue_id: &str) -> Result<Vec<IssueCommentRecord>, IssueCommentError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql = format!(
            "SELECT {COMMENT_COLUMNS} FROM issue_comments WHERE issue_id = ?1 ORDER BY created_at"
        );
        let mut rows = conn.query(&sql, libsql::params![issue_id]).await?;
        let mut comments = Vec::new();
        while let Some(row) = rows.next().await? {
            comments.push(row_to_comment(&row)?);
        }
        Ok(comments)
    }

    async fn get(&self, id: &str) -> Result<Option<IssueCommentRecord>, IssueCommentError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql = format!("SELECT {COMMENT_COLUMNS} FROM issue_comments WHERE id = ?1");
        let mut rows = conn.query(&sql, libsql::params![id]).await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_comment(&row)?)),
            None => Ok(None),
        }
    }

    async fn delete(&self, id: &str) -> Result<Option<IssueCommentRecord>, IssueCommentError> {
        let conn = crate::connection::connect(&self.db).await?;
        let comment = self.get(id).await?;
        let Some(comment) = comment else {
            return Ok(None);
        };
        conn.execute(
            "DELETE FROM issue_comments WHERE id = ?1",
            libsql::params![id],
        )
        .await?;
        Ok(Some(comment))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{Connection, migrate, open};

    async fn repo() -> (TempDir, TursoIssueCommentRepository, Connection) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let conn = crate::connect(&db).await.unwrap();
        let repo = TursoIssueCommentRepository::new(db);
        (dir, repo, conn)
    }

    #[tokio::test]
    async fn create_list_get_delete_roundtrip() {
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
             VALUES ('i1', 'c1', 'T', 1, 'ALPHA-1')",
            (),
        )
        .await
        .unwrap();

        let created = repo
            .create(NewIssueComment {
                issue_id: "i1".to_owned(),
                author_agent_id: None,
                author_user_id: Some("u1".to_owned()),
                body: "hello".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(created.body, "hello");

        let list = repo.list("i1").await.unwrap();
        assert_eq!(list.len(), 1);

        let fetched = repo.get(&created.id).await.unwrap().unwrap();
        assert_eq!(fetched.author_user_id.as_deref(), Some("u1"));

        let deleted = repo.delete(&created.id).await.unwrap().unwrap();
        assert_eq!(deleted.id, created.id);
    }

    #[tokio::test]
    async fn create_requires_issue_in_company() {
        let (_dir, repo, conn) = repo().await;
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
            (),
        )
        .await
        .unwrap();
        let error = repo
            .create(NewIssueComment {
                issue_id: "missing".to_owned(),
                author_agent_id: None,
                author_user_id: None,
                body: "x".to_owned(),
            })
            .await
            .unwrap_err();
        assert!(matches!(error, IssueCommentError::IssueNotFound));
    }
}
