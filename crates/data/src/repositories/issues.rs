//! Issues repository: trait plus Turso/libSQL implementation.
//!
//! Implements the §8.2 issue status machine, single-assignee model, stable
//! per-company issue numbers/identifiers, and service-level hierarchy checks
//! (parent/project/goal/assignee must belong to the same company).

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `issues` table (core columns).
#[derive(Debug, Clone)]
pub struct IssueRecord {
    /// Issue id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Linked project id.
    pub project_id: Option<String>,
    /// Linked goal id.
    pub goal_id: Option<String>,
    /// Parent issue id.
    pub parent_id: Option<String>,
    /// Title.
    pub title: String,
    /// Optional description.
    pub description: Option<String>,
    /// `backlog | todo | in_progress | in_review | done | blocked | cancelled`.
    pub status: String,
    /// `critical | high | medium | low`.
    pub priority: String,
    /// Single assignee agent.
    pub assignee_agent_id: Option<String>,
    /// Single assignee user.
    pub assignee_user_id: Option<String>,
    /// Creator agent.
    pub created_by_agent_id: Option<String>,
    /// Creator user.
    pub created_by_user_id: Option<String>,
    /// Per-company issue number.
    pub issue_number: i64,
    /// Stable identifier (`{prefix}-{number}`).
    pub identifier: String,
    /// Request depth.
    pub request_depth: i64,
    /// `standard | ask | planning`.
    pub work_mode: String,
    /// Billing code.
    pub billing_code: Option<String>,
    /// ISO 8601 start time.
    pub started_at: Option<String>,
    /// ISO 8601 completion time.
    pub completed_at: Option<String>,
    /// ISO 8601 cancellation time.
    pub cancelled_at: Option<String>,
    /// ISO 8601 hide time.
    pub hidden_at: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
    /// ISO 8601 last update time.
    pub updated_at: String,
}

/// Input for creating an issue.
#[derive(Debug, Clone)]
pub struct NewIssue {
    /// Owning company id.
    pub company_id: String,
    /// Linked project id.
    pub project_id: Option<String>,
    /// Linked goal id.
    pub goal_id: Option<String>,
    /// Parent issue id.
    pub parent_id: Option<String>,
    /// Title.
    pub title: String,
    /// Optional description.
    pub description: Option<String>,
    /// Status (defaults: `todo` when assigned, else `backlog`).
    pub status: Option<String>,
    /// Priority (default `medium`).
    pub priority: Option<String>,
    /// Assignee agent (single-assignee model).
    pub assignee_agent_id: Option<String>,
    /// Assignee user.
    pub assignee_user_id: Option<String>,
    /// Creator user.
    pub created_by_user_id: Option<String>,
    /// Work mode (default `standard`).
    pub work_mode: Option<String>,
    /// Billing code.
    pub billing_code: Option<String>,
}

/// Partial issue update.
#[derive(Debug, Default)]
pub struct IssuePatch {
    /// New title.
    pub title: Option<String>,
    /// New description.
    pub description: Option<Option<String>>,
    /// New status (validated against §8.2 transitions).
    pub status: Option<String>,
    /// New priority.
    pub priority: Option<String>,
    /// New assignee agent.
    pub assignee_agent_id: Option<Option<String>>,
    /// New billing code.
    pub billing_code: Option<Option<String>>,
}

/// Issues repository errors.
#[derive(Debug, Error)]
pub enum IssueError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The owning company does not exist.
    #[error("company not found")]
    CompanyNotFound,
    /// A referenced row does not exist.
    #[error("referenced record not found: {0}")]
    ReferenceNotFound(&'static str),
    /// A referenced row belongs to a different company.
    #[error("referenced record belongs to a different company: {0}")]
    ReferenceInDifferentCompany(&'static str),
    /// The requested status transition violates §8.2.
    #[error("invalid status transition: {from} -> {to}")]
    InvalidStatusTransition { from: String, to: String },
}

/// Validates a status transition against the §8.2 state machine.
#[must_use]
pub fn allowed_status_transition(from: &str, to: &str) -> bool {
    matches!(
        (from, to),
        ("backlog", "todo" | "cancelled")
            | ("todo", "in_progress" | "blocked" | "cancelled")
            | (
                "in_progress",
                "in_review" | "blocked" | "done" | "cancelled"
            )
            | ("in_review", "in_progress" | "done" | "cancelled")
            | ("blocked", "todo" | "in_progress" | "cancelled")
    )
}

/// Issue persistence contract.
#[async_trait]
pub trait IssueRepository: Send + Sync {
    /// Creates an issue, allocating the next per-company issue number and
    /// validating all references.
    ///
    /// # Errors
    ///
    /// Returns [`IssueError`] when the company or a reference is missing or
    /// belongs to a different company.
    async fn create(&self, input: NewIssue) -> Result<IssueRecord, IssueError>;

    /// Lists all issues of one company.
    ///
    /// # Errors
    ///
    /// Returns [`IssueError`] on database failure.
    async fn list(&self, company_id: &str) -> Result<Vec<IssueRecord>, IssueError>;

    /// Fetches one issue by id.
    ///
    /// # Errors
    ///
    /// Returns [`IssueError`] on database failure.
    async fn get(&self, id: &str) -> Result<Option<IssueRecord>, IssueError>;

    /// Applies a partial update, validating status transitions.
    ///
    /// # Errors
    ///
    /// Returns [`IssueError`] on database failure or invalid transitions.
    async fn update(&self, id: &str, patch: IssuePatch) -> Result<Option<IssueRecord>, IssueError>;

    /// Deletes an issue, returning the deleted row.
    ///
    /// # Errors
    ///
    /// Returns [`IssueError`] on database failure.
    async fn delete(&self, id: &str) -> Result<Option<IssueRecord>, IssueError>;
}

/// Turso/libSQL implementation of [`IssueRepository`].
#[derive(Debug)]
pub struct TursoIssueRepository {
    db: Database,
}

impl TursoIssueRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

const ISSUE_COLUMNS: &str = "id, company_id, project_id, goal_id, parent_id, title, description,
    status, priority, assignee_agent_id, assignee_user_id, created_by_agent_id,
    created_by_user_id, issue_number, identifier, request_depth, work_mode, billing_code,
    started_at, completed_at, cancelled_at, hidden_at, created_at, updated_at";

fn row_to_issue(row: &libsql::Row) -> Result<IssueRecord, libsql::Error> {
    Ok(IssueRecord {
        id: helpers::row_text(row, 0)?.expect("id is NOT NULL"),
        company_id: helpers::row_text(row, 1)?.expect("company_id is NOT NULL"),
        project_id: helpers::row_text(row, 2)?,
        goal_id: helpers::row_text(row, 3)?,
        parent_id: helpers::row_text(row, 4)?,
        title: helpers::row_text(row, 5)?.expect("title is NOT NULL"),
        description: helpers::row_text(row, 6)?,
        status: helpers::row_text(row, 7)?.expect("status is NOT NULL"),
        priority: helpers::row_text(row, 8)?.expect("priority is NOT NULL"),
        assignee_agent_id: helpers::row_text(row, 9)?,
        assignee_user_id: helpers::row_text(row, 10)?,
        created_by_agent_id: helpers::row_text(row, 11)?,
        created_by_user_id: helpers::row_text(row, 12)?,
        issue_number: helpers::row_i64(row, 13)?,
        identifier: helpers::row_text(row, 14)?.expect("identifier is NOT NULL"),
        request_depth: helpers::row_i64(row, 15)?,
        work_mode: helpers::row_text(row, 16)?.expect("work_mode is NOT NULL"),
        billing_code: helpers::row_text(row, 17)?,
        started_at: helpers::row_text(row, 18)?,
        completed_at: helpers::row_text(row, 19)?,
        cancelled_at: helpers::row_text(row, 20)?,
        hidden_at: helpers::row_text(row, 21)?,
        created_at: helpers::row_text(row, 22)?.expect("created_at is NOT NULL"),
        updated_at: helpers::row_text(row, 23)?.expect("updated_at is NOT NULL"),
    })
}

#[async_trait]
impl IssueRepository for TursoIssueRepository {
    async fn create(&self, input: NewIssue) -> Result<IssueRecord, IssueError> {
        let conn = crate::connection::connect(&self.db).await?;
        let tx = conn.transaction().await?;

        if !helpers::company_exists(&tx, &input.company_id).await? {
            return Err(IssueError::CompanyNotFound);
        }
        for (reference, value) in [
            ("project", input.project_id.as_deref()),
            ("goal", input.goal_id.as_deref()),
            ("parent", input.parent_id.as_deref()),
        ] {
            let Some(value) = value else { continue };
            let table = match reference {
                "project" => "projects",
                "goal" => "goals",
                _ => "issues",
            };
            if !helpers::find_row(&tx, table, value).await? {
                return Err(IssueError::ReferenceNotFound(reference));
            }
            if !helpers::row_belongs_to_company(&tx, table, value, &input.company_id).await? {
                return Err(IssueError::ReferenceInDifferentCompany(reference));
            }
        }
        if let Some(assignee) = &input.assignee_agent_id {
            if !helpers::find_row(&tx, "agents", assignee).await? {
                return Err(IssueError::ReferenceNotFound("assignee_agent"));
            }
            if !helpers::row_belongs_to_company(&tx, "agents", assignee, &input.company_id).await? {
                return Err(IssueError::ReferenceInDifferentCompany("assignee_agent"));
            }
        }

        // Allocate the next issue number and identifier atomically.
        let mut rows = tx
            .query(
                "SELECT issue_prefix, issue_counter FROM companies WHERE id = ?1",
                libsql::params![input.company_id.clone()],
            )
            .await?;
        let row = rows.next().await?.expect("company exists");
        let prefix = helpers::row_text(&row, 0)?.expect("issue_prefix is NOT NULL");
        let counter = helpers::row_i64(&row, 1)?;
        let number = counter + 1;
        tx.execute(
            "UPDATE companies SET issue_counter = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?2",
            libsql::params![number, input.company_id.clone()],
        )
        .await?;

        let status = input.status.unwrap_or_else(|| {
            if input.assignee_agent_id.is_some() || input.assignee_user_id.is_some() {
                "todo".to_owned()
            } else {
                "backlog".to_owned()
            }
        });
        let id = Uuid::new_v4().to_string();
        let identifier = format!("{prefix}-{number}");
        tx.execute(
            "INSERT INTO issues (id, company_id, project_id, goal_id, parent_id, title,
                                 description, status, priority, assignee_agent_id,
                                 assignee_user_id, created_by_user_id, issue_number,
                                 identifier, request_depth, work_mode, billing_code,
                                 created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                     ?17, strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.project_id,
                input.goal_id,
                input.parent_id,
                input.title,
                input.description,
                status,
                input.priority.unwrap_or_else(|| "medium".to_owned()),
                input.assignee_agent_id,
                input.assignee_user_id,
                input.created_by_user_id,
                number,
                identifier,
                0i64,
                input.work_mode.unwrap_or_else(|| "standard".to_owned()),
                input.billing_code
            ],
        )
        .await?;
        tx.commit().await?;
        Ok(self.get(&id).await?.expect("issue was just inserted"))
    }

    async fn list(&self, company_id: &str) -> Result<Vec<IssueRecord>, IssueError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql = format!(
            "SELECT {ISSUE_COLUMNS} FROM issues WHERE company_id = ?1 ORDER BY issue_number"
        );
        let mut rows = conn.query(&sql, libsql::params![company_id]).await?;
        let mut issues = Vec::new();
        while let Some(row) = rows.next().await? {
            issues.push(row_to_issue(&row)?);
        }
        Ok(issues)
    }

    async fn get(&self, id: &str) -> Result<Option<IssueRecord>, IssueError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql = format!("SELECT {ISSUE_COLUMNS} FROM issues WHERE id = ?1");
        let mut rows = conn.query(&sql, libsql::params![id]).await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_issue(&row)?)),
            None => Ok(None),
        }
    }

    async fn update(&self, id: &str, patch: IssuePatch) -> Result<Option<IssueRecord>, IssueError> {
        let conn = crate::connection::connect(&self.db).await?;
        let existing = self.get(id).await?;
        let Some(existing) = existing else {
            return Ok(None);
        };

        if let Some(new_status) = &patch.status
            && !allowed_status_transition(&existing.status, new_status)
        {
            return Err(IssueError::InvalidStatusTransition {
                from: existing.status,
                to: new_status.clone(),
            });
        }

        let mut sets = Vec::new();
        let mut values: Vec<libsql::Value> = Vec::new();
        let mut param = 0usize;

        // Status first, so the §8.2 side-effect CASE expressions can reuse
        // its bound parameter.
        let status_param = if let Some(status) = &patch.status {
            param += 1;
            sets.push(format!("status = ?{param}"));
            values.push(libsql::Value::from(status.clone()));
            param
        } else {
            0
        };

        let mut push = |column: &str, value: Option<Option<libsql::Value>>| match value {
            Some(Some(value)) => {
                param += 1;
                sets.push(format!("{column} = ?{param}"));
                values.push(value);
            }
            Some(None) => sets.push(format!("{column} = NULL")),
            None => {}
        };
        push("title", patch.title.map(|value| Some(value.into())));
        push(
            "description",
            patch.description.map(|value| value.map(Into::into)),
        );
        push("priority", patch.priority.map(|value| Some(value.into())));
        push(
            "assignee_agent_id",
            patch.assignee_agent_id.map(|value| value.map(Into::into)),
        );
        push(
            "billing_code",
            patch.billing_code.map(|value| value.map(Into::into)),
        );
        // §8.2 side effects: entering in_progress stamps started_at once,
        // done stamps completed_at, cancelled stamps cancelled_at.
        if status_param > 0 {
            sets.push(format!(
                "started_at = CASE WHEN ?{status_param} = 'in_progress' AND started_at IS NULL \
                 THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') ELSE started_at END"
            ));
            sets.push(format!(
                "completed_at = CASE WHEN ?{status_param} = 'done' \
                 THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') ELSE completed_at END"
            ));
            sets.push(format!(
                "cancelled_at = CASE WHEN ?{status_param} = 'cancelled' \
                 THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') ELSE cancelled_at END"
            ));
        }

        if sets.is_empty() {
            return Ok(Some(existing));
        }
        param += 1;
        values.push(libsql::Value::from(id.to_owned()));
        let sql = format!(
            "UPDATE issues SET {}, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?{param}",
            sets.join(", ")
        );
        conn.execute(&sql, values).await?;
        Ok(self.get(id).await?)
    }

    async fn delete(&self, id: &str) -> Result<Option<IssueRecord>, IssueError> {
        let conn = crate::connection::connect(&self.db).await?;
        let issue = self.get(id).await?;
        let Some(issue) = issue else {
            return Ok(None);
        };
        match conn
            .execute("DELETE FROM issues WHERE id = ?1", libsql::params![id])
            .await
        {
            Ok(_) => Ok(Some(issue)),
            Err(error) if error.to_string().contains("FOREIGN KEY constraint failed") => {
                Err(IssueError::ReferenceInDifferentCompany("children"))
            }
            Err(error) => Err(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{Connection, migrate, open};

    async fn repo() -> (TempDir, TursoIssueRepository, Connection) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let conn = crate::connect(&db).await.unwrap();
        let repo = TursoIssueRepository::new(db);
        (dir, repo, conn)
    }

    async fn seed(conn: &Connection) {
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024), ('c2', 'Beta', 'BETA', 1024)",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO agents (id, company_id, name, role, adapter_type)
             VALUES ('a1', 'c1', 'one', 'engineer', 'codex_local'),
                    ('a2', 'c2', 'two', 'engineer', 'codex_local')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO goals (id, company_id, title, level)
             VALUES ('g1', 'c1', 'Goal One', 'company'), ('g2', 'c2', 'Goal Two', 'company')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO projects (id, company_id, name)
             VALUES ('p1', 'c1', 'Project One'), ('p2', 'c2', 'Project Two')",
            (),
        )
        .await
        .unwrap();
    }

    fn new_issue(company_id: &str) -> NewIssue {
        NewIssue {
            company_id: company_id.to_owned(),
            project_id: None,
            goal_id: None,
            parent_id: None,
            title: "Task".to_owned(),
            description: None,
            status: None,
            priority: None,
            assignee_agent_id: None,
            assignee_user_id: None,
            created_by_user_id: None,
            work_mode: None,
            billing_code: None,
        }
    }

    #[tokio::test]
    async fn create_allocates_numbers_and_identifiers() {
        let (_dir, repo, conn) = repo().await;
        seed(&conn).await;
        let first = repo.create(new_issue("c1")).await.unwrap();
        assert_eq!(first.issue_number, 1);
        assert_eq!(first.identifier, "ALPHA-1");
        assert_eq!(first.status, "backlog");

        let assigned = repo
            .create(NewIssue {
                assignee_agent_id: Some("a1".to_owned()),
                ..new_issue("c1")
            })
            .await
            .unwrap();
        assert_eq!(assigned.issue_number, 2);
        assert_eq!(assigned.identifier, "ALPHA-2");
        assert_eq!(assigned.status, "todo");
    }

    #[tokio::test]
    async fn create_validates_references() {
        let (_dir, repo, conn) = repo().await;
        seed(&conn).await;

        let error = repo
            .create(NewIssue {
                project_id: Some("p2".to_owned()),
                ..new_issue("c1")
            })
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            IssueError::ReferenceInDifferentCompany("project")
        ));

        let error = repo
            .create(NewIssue {
                goal_id: Some("g2".to_owned()),
                ..new_issue("c1")
            })
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            IssueError::ReferenceInDifferentCompany("goal")
        ));

        let error = repo
            .create(NewIssue {
                assignee_agent_id: Some("a2".to_owned()),
                ..new_issue("c1")
            })
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            IssueError::ReferenceInDifferentCompany("assignee_agent")
        ));
    }

    #[tokio::test]
    async fn status_machine_enforces_transitions() {
        let (_dir, repo, conn) = repo().await;
        seed(&conn).await;
        let issue = repo.create(new_issue("c1")).await.unwrap();

        // backlog -> todo is allowed; backlog -> done is not.
        let updated = repo
            .update(
                &issue.id,
                IssuePatch {
                    status: Some("todo".to_owned()),
                    ..Default::default()
                },
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.status, "todo");

        let error = repo
            .update(
                &issue.id,
                IssuePatch {
                    status: Some("done".to_owned()),
                    ..Default::default()
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(error, IssueError::InvalidStatusTransition { .. }));

        // Full happy path with side effects.
        let updated = repo
            .update(
                &issue.id,
                IssuePatch {
                    status: Some("in_progress".to_owned()),
                    ..Default::default()
                },
            )
            .await
            .unwrap()
            .unwrap();
        assert!(updated.started_at.is_some());
        let updated = repo
            .update(
                &issue.id,
                IssuePatch {
                    status: Some("done".to_owned()),
                    ..Default::default()
                },
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.status, "done");
        assert!(updated.completed_at.is_some());
    }

    #[tokio::test]
    async fn list_get_update_delete_roundtrip() {
        let (_dir, repo, conn) = repo().await;
        seed(&conn).await;
        let created = repo.create(new_issue("c1")).await.unwrap();

        let list = repo.list("c1").await.unwrap();
        assert_eq!(list.len(), 1);
        assert!(repo.list("c2").await.unwrap().is_empty());

        let fetched = repo.get(&created.id).await.unwrap().unwrap();
        assert_eq!(fetched.title, "Task");

        let deleted = repo.delete(&created.id).await.unwrap().unwrap();
        assert_eq!(deleted.id, created.id);
        assert!(repo.get(&created.id).await.unwrap().is_none());
    }
}
