//! Cases repository (upstream cases.ts).

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `cases` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseRecord {
    /// Case id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Project id.
    pub project_id: Option<String>,
    /// Per-company case number.
    pub case_number: i64,
    /// Identifier.
    pub identifier: String,
    /// Case type.
    pub case_type: String,
    /// Type-scoped key.
    pub key: Option<String>,
    /// Title.
    pub title: String,
    /// Summary.
    pub summary: Option<String>,
    /// Status.
    pub status: String,
    /// Fields JSON.
    pub fields: serde_json::Value,
    /// Parent case id.
    pub parent_case_id: Option<String>,
    /// Creating agent id.
    pub created_by_agent_id: Option<String>,
    /// Creating user id.
    pub created_by_user_id: Option<String>,
    /// ISO 8601 completion.
    pub completed_at: Option<String>,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// Input for creating a case.
#[derive(Debug, Clone)]
pub struct NewCase {
    /// Owning company id.
    pub company_id: String,
    /// Project id.
    pub project_id: Option<String>,
    /// Case type.
    pub case_type: String,
    /// Type-scoped key.
    pub key: Option<String>,
    /// Title.
    pub title: String,
    /// Summary.
    pub summary: Option<String>,
    /// Fields JSON.
    pub fields: Option<serde_json::Value>,
    /// Parent case id.
    pub parent_case_id: Option<String>,
    /// Creating agent id.
    pub created_by_agent_id: Option<String>,
    /// Creating user id.
    pub created_by_user_id: Option<String>,
}

/// Input for updating a case.
#[derive(Debug, Clone)]
pub struct CasePatch {
    /// New title.
    pub title: Option<String>,
    /// New summary (`null` clears).
    pub summary: Option<Option<String>>,
    /// New fields.
    pub fields: Option<serde_json::Value>,
    /// New parent case (`null` clears).
    pub parent_case_id: Option<Option<String>>,
}

/// Case repository errors.
#[derive(Debug, Error)]
pub enum CaseError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The company does not exist.
    #[error("company not found")]
    CompanyNotFound,
    /// A referenced record is missing or in another company.
    #[error("reference not found")]
    ReferenceNotFound,
    /// The case does not exist.
    #[error("case not found")]
    CaseNotFound,
    /// The status transition is invalid.
    #[error("invalid status transition: {0} -> {1}")]
    InvalidStatusTransition(String, String),
}

/// Allowed forward transitions (draft → in_progress → in_review →
/// approved/done; any non-terminal → cancelled).
#[must_use]
pub fn allowed_case_transition(from: &str, to: &str) -> bool {
    if to == "cancelled" && from != "cancelled" && from != "done" {
        return true;
    }
    matches!(
        (from, to),
        ("draft", "in_progress")
            | ("in_progress", "in_review" | "done" | "draft")
            | ("in_review", "approved" | "done" | "in_progress")
            | ("approved", "done" | "in_review")
    )
}

/// Case persistence contract.
#[async_trait]
pub trait CaseRepository: Send + Sync {
    /// Creates a case, assigning the next per-company case number and an
    /// identifier derived from the company prefix.
    ///
    /// # Errors
    ///
    /// Returns [`CaseError`] on invalid references or duplicates.
    async fn create(&self, input: NewCase) -> Result<CaseRecord, CaseError>;

    /// Lists cases for a company (optionally filtered by project).
    ///
    /// # Errors
    ///
    /// Returns [`CaseError`] on database failure.
    async fn list(
        &self,
        company_id: &str,
        project_id: Option<&str>,
    ) -> Result<Vec<CaseRecord>, CaseError>;

    /// Gets one case (company-scoped).
    ///
    /// # Errors
    ///
    /// Returns [`CaseError`] on database failure.
    async fn get(&self, company_id: &str, id: &str) -> Result<Option<CaseRecord>, CaseError>;

    /// Resolves the owning company of a case.
    ///
    /// # Errors
    ///
    /// Returns [`CaseError`] on database failure.
    async fn company_of(&self, id: &str) -> Result<Option<String>, CaseError>;

    /// Applies a partial update.
    ///
    /// # Errors
    ///
    /// Returns [`CaseError`] on invalid references.
    async fn update(
        &self,
        company_id: &str,
        id: &str,
        patch: CasePatch,
    ) -> Result<Option<CaseRecord>, CaseError>;

    /// Moves a case to a new status (forward state machine).
    ///
    /// # Errors
    ///
    /// Returns [`CaseError`] on invalid transitions.
    async fn set_status(
        &self,
        company_id: &str,
        id: &str,
        status: &str,
    ) -> Result<Option<CaseRecord>, CaseError>;

    /// Deletes a case (company-scoped).
    ///
    /// # Errors
    ///
    /// Returns [`CaseError`] on database failure.
    async fn delete(&self, company_id: &str, id: &str) -> Result<Option<CaseRecord>, CaseError>;
}

/// Turso/libSQL implementation of [`CaseRepository`].
#[derive(Debug)]
pub struct TursoCaseRepository {
    db: Database,
}

impl TursoCaseRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

fn row_to_case(row: &libsql::Row) -> Result<CaseRecord, libsql::Error> {
    Ok(CaseRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        project_id: helpers::row_text(row, 2)?,
        case_number: helpers::row_i64(row, 3)?,
        identifier: helpers::row_text(row, 4)?.expect("identifier"),
        case_type: helpers::row_text(row, 5)?.expect("case_type"),
        key: helpers::row_text(row, 6)?,
        title: helpers::row_text(row, 7)?.expect("title"),
        summary: helpers::row_text(row, 8)?,
        status: helpers::row_text(row, 9)?.expect("status"),
        fields: helpers::row_text(row, 10)?
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or(serde_json::Value::Null),
        parent_case_id: helpers::row_text(row, 11)?,
        created_by_agent_id: helpers::row_text(row, 12)?,
        created_by_user_id: helpers::row_text(row, 13)?,
        completed_at: helpers::row_text(row, 14)?,
        created_at: helpers::row_text(row, 15)?.expect("created_at"),
    })
}

const CASE_COLUMNS: &str = "id, company_id, project_id, case_number, identifier, case_type, key,
    title, summary, status, fields, parent_case_id, created_by_agent_id, created_by_user_id,
    completed_at, created_at";

#[async_trait]
impl CaseRepository for TursoCaseRepository {
    async fn create(&self, input: NewCase) -> Result<CaseRecord, CaseError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(CaseError::CompanyNotFound);
        }
        if let Some(project_id) = &input.project_id
            && !helpers::row_belongs_to_company(&conn, "projects", project_id, &input.company_id)
                .await?
        {
            return Err(CaseError::ReferenceNotFound);
        }
        if let Some(parent_id) = &input.parent_case_id
            && !helpers::row_belongs_to_company(&conn, "cases", parent_id, &input.company_id)
                .await?
        {
            return Err(CaseError::ReferenceNotFound);
        }
        // Next per-company case number.
        let mut rows = conn
            .query(
                "SELECT COALESCE(MAX(case_number), 0) + 1 FROM cases WHERE company_id = ?1",
                libsql::params![input.company_id.clone()],
            )
            .await?;
        let case_number = helpers::row_i64(&rows.next().await?.expect("row"), 0)?;
        // Identifier derived from the company issue prefix.
        let mut prefix_rows = conn
            .query(
                "SELECT issue_prefix FROM companies WHERE id = ?1",
                libsql::params![input.company_id.clone()],
            )
            .await?;
        let prefix =
            helpers::row_text(&prefix_rows.next().await?.expect("row"), 0)?.unwrap_or_default();
        let identifier = format!("{prefix}-CASE-{case_number}");
        let id = Uuid::new_v4().to_string();
        let fields = input
            .fields
            .map(|value| value.to_string())
            .unwrap_or_else(|| "{}".to_owned());
        let result = conn
            .execute(
                "INSERT INTO cases
                   (id, company_id, project_id, case_number, identifier, case_type, key,
                    title, summary, status, fields, parent_case_id, created_by_agent_id,
                    created_by_user_id, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'draft', ?10, ?11, ?12, ?13,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.project_id,
                    case_number,
                    identifier,
                    input.case_type,
                    input.key,
                    input.title,
                    input.summary,
                    fields,
                    input.parent_case_id,
                    input.created_by_agent_id,
                    input.created_by_user_id
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!("SELECT {CASE_COLUMNS} FROM cases WHERE id = ?1"),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("case was just inserted");
                Ok(row_to_case(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(CaseError::ReferenceNotFound)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list(
        &self,
        company_id: &str,
        project_id: Option<&str>,
    ) -> Result<Vec<CaseRecord>, CaseError> {
        let conn = crate::connection::connect(&self.db).await?;
        let (sql, params): (String, Vec<libsql::Value>) = match project_id {
            Some(project_id) => (
                format!(
                    "SELECT {CASE_COLUMNS} FROM cases
                     WHERE company_id = ?1 AND project_id = ?2 ORDER BY case_number DESC"
                ),
                vec![company_id.into(), project_id.into()],
            ),
            None => (
                format!(
                    "SELECT {CASE_COLUMNS} FROM cases WHERE company_id = ?1 ORDER BY case_number DESC"
                ),
                vec![company_id.into()],
            ),
        };
        let mut rows = conn.query(&sql, params).await?;
        let mut cases = Vec::new();
        while let Some(row) = rows.next().await? {
            cases.push(row_to_case(&row)?);
        }
        Ok(cases)
    }

    async fn get(&self, company_id: &str, id: &str) -> Result<Option<CaseRecord>, CaseError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!("SELECT {CASE_COLUMNS} FROM cases WHERE company_id = ?1 AND id = ?2"),
                libsql::params![company_id, id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_case(&row)?)),
            None => Ok(None),
        }
    }

    async fn company_of(&self, id: &str) -> Result<Option<String>, CaseError> {
        let conn = crate::connection::connect(&self.db).await?;
        Ok(helpers::row_company(&conn, "cases", id).await?)
    }

    async fn update(
        &self,
        company_id: &str,
        id: &str,
        patch: CasePatch,
    ) -> Result<Option<CaseRecord>, CaseError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut sets = Vec::new();
        let mut values: Vec<libsql::Value> = Vec::new();
        let mut param = 0usize;
        if let Some(title) = patch.title {
            param += 1;
            sets.push(format!("title = ?{param}"));
            values.push(libsql::Value::from(title));
        }
        if let Some(summary) = patch.summary {
            match summary {
                Some(summary) => {
                    param += 1;
                    sets.push(format!("summary = ?{param}"));
                    values.push(libsql::Value::from(summary));
                }
                None => sets.push("summary = NULL".to_owned()),
            }
        }
        if let Some(fields) = patch.fields {
            param += 1;
            sets.push(format!("fields = ?{param}"));
            values.push(libsql::Value::from(fields.to_string()));
        }
        if let Some(parent) = patch.parent_case_id {
            if let Some(parent_id) = &parent
                && !helpers::row_belongs_to_company(&conn, "cases", parent_id, company_id).await?
            {
                return Err(CaseError::ReferenceNotFound);
            }
            match parent {
                Some(parent_id) => {
                    param += 1;
                    sets.push(format!("parent_case_id = ?{param}"));
                    values.push(libsql::Value::from(parent_id));
                }
                None => sets.push("parent_case_id = NULL".to_owned()),
            }
        }
        if sets.is_empty() {
            return Err(CaseError::CaseNotFound);
        }
        let company_param = param + 1;
        let id_param = param + 2;
        values.push(libsql::Value::from(company_id.to_owned()));
        values.push(libsql::Value::from(id.to_owned()));
        let sql = format!(
            "UPDATE cases SET {}, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE company_id = ?{company_param} AND id = ?{id_param}",
            sets.join(", ")
        );
        let updated = conn.execute(&sql, values).await?;
        if updated == 0 {
            return Ok(None);
        }
        let mut rows = conn
            .query(
                &format!("SELECT {CASE_COLUMNS} FROM cases WHERE company_id = ?1 AND id = ?2"),
                libsql::params![company_id, id],
            )
            .await?;
        let row = rows.next().await?.expect("case exists");
        Ok(Some(row_to_case(&row)?))
    }

    async fn set_status(
        &self,
        company_id: &str,
        id: &str,
        status: &str,
    ) -> Result<Option<CaseRecord>, CaseError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT status FROM cases WHERE company_id = ?1 AND id = ?2",
                libsql::params![company_id, id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let from = helpers::row_text(&row, 0)?.expect("status");
        if !allowed_case_transition(&from, status) {
            return Err(CaseError::InvalidStatusTransition(from, status.to_owned()));
        }
        conn.execute(
            "UPDATE cases SET status = ?1,
                    completed_at = CASE WHEN ?1 IN ('done', 'cancelled')
                                       THEN strftime('%Y-%m-%dT%H:%M:%fZ','now')
                                       ELSE NULL END,
                    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE company_id = ?2 AND id = ?3",
            libsql::params![status, company_id, id],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {CASE_COLUMNS} FROM cases WHERE company_id = ?1 AND id = ?2"),
                libsql::params![company_id, id],
            )
            .await?;
        let row = rows.next().await?.expect("case exists");
        Ok(Some(row_to_case(&row)?))
    }

    async fn delete(&self, company_id: &str, id: &str) -> Result<Option<CaseRecord>, CaseError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!("SELECT {CASE_COLUMNS} FROM cases WHERE company_id = ?1 AND id = ?2"),
                libsql::params![company_id, id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let record = row_to_case(&row)?;
        conn.execute("DELETE FROM cases WHERE id = ?1", libsql::params![id])
            .await?;
        Ok(Some(record))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoCaseRepository) {
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
        (dir, TursoCaseRepository::new(db))
    }

    #[tokio::test]
    async fn lifecycle_numbers_identifiers_and_transitions() {
        let (_dir, repo) = repo().await;
        let first = repo
            .create(NewCase {
                company_id: "c1".to_owned(),
                project_id: None,
                case_type: "support".to_owned(),
                key: Some("k1".to_owned()),
                title: "First".to_owned(),
                summary: Some("s".to_owned()),
                fields: Some(serde_json::json!({ "severity": "high" })),
                parent_case_id: None,
                created_by_agent_id: None,
                created_by_user_id: Some("u1".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(first.case_number, 1);
        assert_eq!(first.identifier, "ALPHA-CASE-1");
        let second = repo
            .create(NewCase {
                company_id: "c1".to_owned(),
                project_id: None,
                case_type: "support".to_owned(),
                key: None,
                title: "Second".to_owned(),
                summary: None,
                fields: None,
                parent_case_id: None,
                created_by_agent_id: None,
                created_by_user_id: None,
            })
            .await
            .unwrap();
        assert_eq!(second.case_number, 2);
        assert_eq!(second.identifier, "ALPHA-CASE-2");

        // Parent reference.
        let child = repo
            .create(NewCase {
                company_id: "c1".to_owned(),
                project_id: None,
                case_type: "subtask".to_owned(),
                key: None,
                title: "Child".to_owned(),
                summary: None,
                fields: None,
                parent_case_id: Some(first.id.clone()),
                created_by_agent_id: None,
                created_by_user_id: None,
            })
            .await
            .unwrap();
        assert_eq!(child.parent_case_id.as_deref(), Some(first.id.as_str()));

        // State machine.
        let in_progress = repo
            .set_status("c1", &first.id, "in_progress")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(in_progress.status, "in_progress");
        let in_review = repo
            .set_status("c1", &first.id, "in_review")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(in_review.status, "in_review");
        let done = repo
            .set_status("c1", &first.id, "done")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(done.status, "done");
        assert!(done.completed_at.is_some());
        // Terminal cannot move forward.
        assert!(matches!(
            repo.set_status("c1", &first.id, "in_review")
                .await
                .unwrap_err(),
            CaseError::InvalidStatusTransition(_, _)
        ));

        // List/get/update/delete + cross-company.
        assert_eq!(repo.list("c1", None).await.unwrap().len(), 3);
        let updated = repo
            .update(
                "c1",
                &second.id,
                CasePatch {
                    title: Some("Second v2".to_owned()),
                    summary: Some(None),
                    fields: None,
                    parent_case_id: None,
                },
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.title, "Second v2");
        assert!(updated.summary.is_none());
        assert!(repo.get("c2", &first.id).await.unwrap().is_none());
        assert!(repo.delete("c1", &child.id).await.unwrap().is_some());
        assert_eq!(repo.list("c1", None).await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn duplicate_type_key_rejected() {
        let (_dir, repo) = repo().await;
        repo.create(NewCase {
            company_id: "c1".to_owned(),
            project_id: None,
            case_type: "support".to_owned(),
            key: Some("dup".to_owned()),
            title: "A".to_owned(),
            summary: None,
            fields: None,
            parent_case_id: None,
            created_by_agent_id: None,
            created_by_user_id: None,
        })
        .await
        .unwrap();
        assert!(matches!(
            repo.create(NewCase {
                company_id: "c1".to_owned(),
                project_id: None,
                case_type: "support".to_owned(),
                key: Some("dup".to_owned()),
                title: "B".to_owned(),
                summary: None,
                fields: None,
                parent_case_id: None,
                created_by_agent_id: None,
                created_by_user_id: None,
            })
            .await
            .unwrap_err(),
            CaseError::ReferenceNotFound
        ));
    }
}
