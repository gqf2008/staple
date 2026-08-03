//! Issue documents repository.
//!
//! Text-first documents with append-only revisions, linked to issues by a
//! stable workflow key (`plan`, `design`, `notes`, ...), mirroring SPEC §7.15.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `documents` table.
#[derive(Debug, Clone)]
pub struct DocumentRecord {
    /// Document id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Title.
    pub title: Option<String>,
    /// Format (default `markdown`).
    pub format: String,
    /// Latest body.
    pub latest_body: String,
    /// Latest revision id.
    pub latest_revision_id: Option<String>,
    /// Latest revision number.
    pub latest_revision_number: i64,
    /// Created-by attribution.
    pub created_by_user_id: Option<String>,
    /// Updated-by attribution.
    pub updated_by_user_id: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
    /// ISO 8601 last update time.
    pub updated_at: String,
}

/// Input for creating an issue document.
#[derive(Debug, Clone)]
pub struct NewIssueDocument {
    /// Issue id.
    pub issue_id: String,
    /// Stable workflow key (`plan`, `design`, `notes`, ...).
    pub key: String,
    /// Title.
    pub title: Option<String>,
    /// Initial body.
    pub body: String,
    /// Creator user id.
    pub created_by_user_id: Option<String>,
}

/// Input for updating an issue document (appends a revision).
#[derive(Debug, Clone)]
pub struct UpdateIssueDocument {
    /// Issue id.
    pub issue_id: String,
    /// Stable workflow key.
    pub key: String,
    /// New body.
    pub body: String,
    /// Change summary.
    pub change_summary: Option<String>,
    /// Updater user id.
    pub updated_by_user_id: Option<String>,
}

/// Document repository errors.
#[derive(Debug, Error)]
pub enum DocumentError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The issue does not exist or belongs to another company.
    #[error("issue not found")]
    IssueNotFound,
    /// A document with this key already exists on the issue.
    #[error("document key already exists")]
    KeyExists,
    /// No document exists for the issue/key pair.
    #[error("document not found")]
    DocumentNotFound,
}

/// Document persistence contract.
#[async_trait]
pub trait DocumentRepository: Send + Sync {
    /// Creates an issue document with revision 1 and the issue link.
    ///
    /// # Errors
    ///
    /// Returns [`DocumentError`] when the issue is missing or the key is
    /// already in use.
    async fn create_issue_document(
        &self,
        input: NewIssueDocument,
    ) -> Result<DocumentRecord, DocumentError>;

    /// Appends a revision to an issue document.
    ///
    /// # Errors
    ///
    /// Returns [`DocumentError`] when the issue/key pair does not exist.
    async fn update_issue_document(
        &self,
        input: UpdateIssueDocument,
    ) -> Result<DocumentRecord, DocumentError>;

    /// Lists the documents linked to an issue.
    ///
    /// # Errors
    ///
    /// Returns [`DocumentError`] on database failure.
    async fn list_issue_documents(
        &self,
        issue_id: &str,
    ) -> Result<Vec<DocumentRecord>, DocumentError>;

    /// Fetches one document by issue id and workflow key.
    ///
    /// # Errors
    ///
    /// Returns [`DocumentError`] on database failure.
    async fn get_issue_document_by_key(
        &self,
        issue_id: &str,
        key: &str,
    ) -> Result<Option<DocumentRecord>, DocumentError>;
}

/// Turso/libSQL implementation of [`DocumentRepository`].
#[derive(Debug)]
pub struct TursoDocumentRepository {
    db: Database,
}

impl TursoDocumentRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

const DOCUMENT_COLUMNS: &str = "id, company_id, title, format, latest_body,
    latest_revision_id, latest_revision_number, created_by_agent_id, created_by_user_id,
    updated_by_agent_id, updated_by_user_id, locked_at, locked_by_agent_id,
    locked_by_user_id, created_at, updated_at";

fn row_to_document(row: &libsql::Row) -> Result<DocumentRecord, libsql::Error> {
    Ok(DocumentRecord {
        id: helpers::row_text(row, 0)?.expect("id is NOT NULL"),
        company_id: helpers::row_text(row, 1)?.expect("company_id is NOT NULL"),
        title: helpers::row_text(row, 2)?,
        format: helpers::row_text(row, 3)?.expect("format is NOT NULL"),
        latest_body: helpers::row_text(row, 4)?.expect("latest_body is NOT NULL"),
        latest_revision_id: helpers::row_text(row, 5)?,
        latest_revision_number: helpers::row_i64(row, 6)?,
        created_by_user_id: helpers::row_text(row, 8)?,
        updated_by_user_id: helpers::row_text(row, 10)?,
        created_at: helpers::row_text(row, 14)?.expect("created_at is NOT NULL"),
        updated_at: helpers::row_text(row, 15)?.expect("updated_at is NOT NULL"),
    })
}

/// Resolves the company of an issue, returning `None` when missing.
async fn issue_company(
    conn: &libsql::Connection,
    issue_id: &str,
) -> Result<Option<String>, libsql::Error> {
    helpers::row_company(conn, "issues", issue_id).await
}

/// Finds the document id linked to an issue by key.
async fn linked_document_id(
    conn: &libsql::Connection,
    issue_id: &str,
    key: &str,
) -> Result<Option<String>, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT document_id FROM issue_documents WHERE issue_id = ?1 AND key = ?2",
            libsql::params![issue_id, key],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(helpers::row_text(&row, 0)?),
        None => Ok(None),
    }
}

#[async_trait]
impl DocumentRepository for TursoDocumentRepository {
    async fn create_issue_document(
        &self,
        input: NewIssueDocument,
    ) -> Result<DocumentRecord, DocumentError> {
        let conn = crate::connection::connect(&self.db).await?;
        let tx = conn.transaction().await?;
        let Some(company_id) = issue_company(&tx, &input.issue_id).await? else {
            return Err(DocumentError::IssueNotFound);
        };
        if linked_document_id(&tx, &input.issue_id, &input.key)
            .await?
            .is_some()
        {
            return Err(DocumentError::KeyExists);
        }

        let document_id = Uuid::new_v4().to_string();
        let revision_id = Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO documents (id, company_id, title, format, latest_body,
                                    latest_revision_id, latest_revision_number,
                                    created_by_user_id, updated_by_user_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'markdown', ?4, ?5, 1, ?6, ?6,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                document_id.clone(),
                company_id.clone(),
                input.title,
                input.body.clone(),
                revision_id.clone(),
                input.created_by_user_id.clone()
            ],
        )
        .await?;
        tx.execute(
            "INSERT INTO document_revisions (id, company_id, document_id, revision_number,
                                             body, change_summary, created_at, updated_at)
             VALUES (?1, ?2, ?3, 1, ?4, NULL,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                revision_id.clone(),
                company_id.clone(),
                document_id.clone(),
                input.body.clone()
            ],
        )
        .await?;
        tx.execute(
            "INSERT INTO issue_documents (id, company_id, issue_id, document_id, key,
                                          created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                Uuid::new_v4().to_string(),
                company_id,
                input.issue_id,
                document_id.clone(),
                input.key
            ],
        )
        .await?;
        tx.commit().await?;

        let mut rows = conn
            .query(
                &format!("SELECT {DOCUMENT_COLUMNS} FROM documents WHERE id = ?1"),
                libsql::params![document_id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(row_to_document(&row)?),
            None => unreachable!("document was just inserted"),
        }
    }

    async fn update_issue_document(
        &self,
        input: UpdateIssueDocument,
    ) -> Result<DocumentRecord, DocumentError> {
        let conn = crate::connection::connect(&self.db).await?;
        let tx = conn.transaction().await?;
        let Some(company_id) = issue_company(&tx, &input.issue_id).await? else {
            return Err(DocumentError::IssueNotFound);
        };
        let Some(document_id) = linked_document_id(&tx, &input.issue_id, &input.key).await? else {
            return Err(DocumentError::DocumentNotFound);
        };

        let mut rows = tx
            .query(
                "SELECT latest_revision_number FROM documents WHERE id = ?1",
                libsql::params![document_id.clone()],
            )
            .await?;
        let row = rows.next().await?.expect("document exists");
        let revision_number = helpers::row_i64(&row, 0)? + 1;
        let revision_id = Uuid::new_v4().to_string();

        tx.execute(
            "INSERT INTO document_revisions (id, company_id, document_id, revision_number,
                                             body, change_summary, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                revision_id.clone(),
                company_id,
                document_id.clone(),
                revision_number,
                input.body.clone(),
                input.change_summary.clone()
            ],
        )
        .await?;
        tx.execute(
            "UPDATE documents
             SET latest_body = ?1, latest_revision_id = ?2, latest_revision_number = ?3,
                 updated_by_user_id = ?4, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?5",
            libsql::params![
                input.body,
                revision_id,
                revision_number,
                input.updated_by_user_id,
                document_id.clone()
            ],
        )
        .await?;
        tx.commit().await?;

        let mut rows = conn
            .query(
                &format!("SELECT {DOCUMENT_COLUMNS} FROM documents WHERE id = ?1"),
                libsql::params![document_id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(row_to_document(&row)?),
            None => unreachable!("document exists"),
        }
    }

    async fn list_issue_documents(
        &self,
        issue_id: &str,
    ) -> Result<Vec<DocumentRecord>, DocumentError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql = "SELECT d.id, d.company_id, d.title, d.format, d.latest_body,
                    d.latest_revision_id, d.latest_revision_number, d.created_by_agent_id,
                    d.created_by_user_id, d.updated_by_agent_id, d.updated_by_user_id,
                    d.locked_at, d.locked_by_agent_id, d.locked_by_user_id, d.created_at,
                    d.updated_at
             FROM issue_documents idoc
             JOIN documents d ON d.id = idoc.document_id
             WHERE idoc.issue_id = ?1
             ORDER BY idoc.created_at";
        let mut rows = conn.query(sql, libsql::params![issue_id]).await?;
        let mut documents = Vec::new();
        while let Some(row) = rows.next().await? {
            documents.push(row_to_document(&row)?);
        }
        Ok(documents)
    }

    async fn get_issue_document_by_key(
        &self,
        issue_id: &str,
        key: &str,
    ) -> Result<Option<DocumentRecord>, DocumentError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql = "SELECT d.id, d.company_id, d.title, d.format, d.latest_body,
                    d.latest_revision_id, d.latest_revision_number, d.created_by_agent_id,
                    d.created_by_user_id, d.updated_by_agent_id, d.updated_by_user_id,
                    d.locked_at, d.locked_by_agent_id, d.locked_by_user_id, d.created_at,
                    d.updated_at
             FROM issue_documents idoc
             JOIN documents d ON d.id = idoc.document_id
             WHERE idoc.issue_id = ?1 AND idoc.key = ?2";
        let mut rows = conn.query(&sql, libsql::params![issue_id, key]).await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_document(&row)?)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{Connection, migrate, open};

    async fn repo() -> (TempDir, TursoDocumentRepository, Connection) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let conn = crate::connect(&db).await.unwrap();
        let repo = TursoDocumentRepository::new(db);
        (dir, repo, conn)
    }

    async fn seed(conn: &Connection) {
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
    }

    #[tokio::test]
    async fn create_update_list_get_roundtrip() {
        let (_dir, repo, conn) = repo().await;
        seed(&conn).await;

        let created = repo
            .create_issue_document(NewIssueDocument {
                issue_id: "i1".to_owned(),
                key: "plan".to_owned(),
                title: Some("Plan".to_owned()),
                body: "# v1".to_owned(),
                created_by_user_id: Some("u1".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(created.latest_revision_number, 1);
        assert_eq!(created.latest_body, "# v1");

        // Same key again is rejected.
        let error = repo
            .create_issue_document(NewIssueDocument {
                issue_id: "i1".to_owned(),
                key: "plan".to_owned(),
                title: None,
                body: "x".to_owned(),
                created_by_user_id: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(error, DocumentError::KeyExists));

        // Update appends revision 2.
        let updated = repo
            .update_issue_document(UpdateIssueDocument {
                issue_id: "i1".to_owned(),
                key: "plan".to_owned(),
                body: "# v2".to_owned(),
                change_summary: Some("rewrite".to_owned()),
                updated_by_user_id: Some("u2".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(updated.latest_revision_number, 2);
        assert_eq!(updated.latest_body, "# v2");

        let list = repo.list_issue_documents("i1").await.unwrap();
        assert_eq!(list.len(), 1);

        let fetched = repo
            .get_issue_document_by_key("i1", "plan")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(fetched.latest_revision_number, 2);
        assert!(
            repo.get_issue_document_by_key("i1", "design")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn create_requires_issue() {
        let (_dir, repo, conn) = repo().await;
        seed(&conn).await;
        let error = repo
            .create_issue_document(NewIssueDocument {
                issue_id: "missing".to_owned(),
                key: "plan".to_owned(),
                title: None,
                body: "x".to_owned(),
                created_by_user_id: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(error, DocumentError::IssueNotFound));
    }
}
