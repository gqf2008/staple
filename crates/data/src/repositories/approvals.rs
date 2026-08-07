//! Approvals repository with the §8.3 state machine.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `approvals` table.
#[derive(Debug, Clone)]
pub struct ApprovalRecord {
    /// Approval id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// `hire_agent | approve_ceo_strategy | budget_override_required |
    /// request_board_approval`.
    pub r#type: String,
    /// Requester agent id.
    pub requested_by_agent_id: Option<String>,
    /// Requester user id.
    pub requested_by_user_id: Option<String>,
    /// `pending | revision_requested | approved | rejected | cancelled`.
    pub status: String,
    /// Payload JSON.
    pub payload: String,
    /// Decision note.
    pub decision_note: Option<String>,
    /// Deciding user id.
    pub decided_by_user_id: Option<String>,
    /// ISO 8601 decision time.
    pub decided_at: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// Input for creating an approval.
#[derive(Debug, Clone)]
pub struct NewApproval {
    /// Owning company id.
    pub company_id: String,
    /// Type.
    pub r#type: String,
    /// Requester agent id.
    pub requested_by_agent_id: Option<String>,
    /// Requester user id.
    pub requested_by_user_id: Option<String>,
    /// Payload JSON.
    pub payload: String,
}

/// Input for deciding an approval.
#[derive(Debug, Clone)]
pub struct ApprovalDecision {
    /// `approved | rejected`.
    pub decision: String,
    /// Decision note.
    pub decision_note: Option<String>,
    /// Deciding user id.
    pub decided_by_user_id: Option<String>,
}

/// Approval repository errors.
#[derive(Debug, Error)]
pub enum ApprovalError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The company does not exist.
    #[error("company not found")]
    CompanyNotFound,
    /// The approval is not in a decidable state.
    #[error("approval is not pending")]
    NotPending,
    /// The decision value is not `approved`, `rejected`, or `request_revision`.
    #[error("invalid decision")]
    InvalidDecision,
}

/// Validates a transition against the §8.3 approval state machine.
#[must_use]
pub fn allowed_approval_transition(from: &str, to: &str) -> bool {
    matches!(
        (from, to),
        (
            "pending",
            "approved" | "rejected" | "revision_requested" | "cancelled"
        ) | (
            "revision_requested",
            "approved" | "rejected" | "cancelled" | "pending"
        )
    )
}

/// Approval persistence contract.
#[async_trait]
pub trait ApprovalRepository: Send + Sync {
    /// Creates a pending approval.
    ///
    /// # Errors
    ///
    /// Returns [`ApprovalError`] when the company is missing.
    async fn create(&self, input: NewApproval) -> Result<ApprovalRecord, ApprovalError>;

    /// Lists approvals, optionally filtered by status.
    ///
    /// # Errors
    ///
    /// Returns [`ApprovalError`] on database failure.
    async fn list(
        &self,
        company_id: &str,
        status: Option<&str>,
    ) -> Result<Vec<ApprovalRecord>, ApprovalError>;

    /// Fetches one approval by id.
    ///
    /// # Errors
    ///
    /// Returns [`ApprovalError`] on database failure.
    async fn get(&self, id: &str) -> Result<Option<ApprovalRecord>, ApprovalError>;

    /// Decides a pending approval (`approved` or `rejected`).
    ///
    /// # Errors
    ///
    /// Returns [`ApprovalError`] on invalid state or decision.
    async fn decide(
        &self,
        id: &str,
        decision: ApprovalDecision,
    ) -> Result<Option<ApprovalRecord>, ApprovalError>;

    /// Cancels a pending approval.
    ///
    /// # Errors
    ///
    /// Returns [`ApprovalError`] on invalid state.
    async fn cancel(&self, id: &str) -> Result<Option<ApprovalRecord>, ApprovalError>;
}

/// Turso/libSQL implementation of [`ApprovalRepository`].
#[derive(Debug)]
pub struct TursoApprovalRepository {
    db: Database,
}

impl TursoApprovalRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

const COLUMNS: &str = "id, company_id, type, requested_by_agent_id, requested_by_user_id,
    status, payload, decision_note, decided_by_user_id, decided_at, created_at";

fn row_to_approval(row: &libsql::Row) -> Result<ApprovalRecord, libsql::Error> {
    Ok(ApprovalRecord {
        id: helpers::row_text(row, 0)?.expect("id is NOT NULL"),
        company_id: helpers::row_text(row, 1)?.expect("company_id is NOT NULL"),
        r#type: helpers::row_text(row, 2)?.expect("type is NOT NULL"),
        requested_by_agent_id: helpers::row_text(row, 3)?,
        requested_by_user_id: helpers::row_text(row, 4)?,
        status: helpers::row_text(row, 5)?.expect("status is NOT NULL"),
        payload: helpers::row_text(row, 6)?.expect("payload is NOT NULL"),
        decision_note: helpers::row_text(row, 7)?,
        decided_by_user_id: helpers::row_text(row, 8)?,
        decided_at: helpers::row_text(row, 9)?,
        created_at: helpers::row_text(row, 10)?.expect("created_at is NOT NULL"),
    })
}

#[async_trait]
impl ApprovalRepository for TursoApprovalRepository {
    async fn create(&self, input: NewApproval) -> Result<ApprovalRecord, ApprovalError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ApprovalError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO approvals (id, company_id, type, requested_by_agent_id,
                                    requested_by_user_id, status, payload, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.r#type,
                input.requested_by_agent_id,
                input.requested_by_user_id,
                input.payload
            ],
        )
        .await?;
        Ok(self.get(&id).await?.expect("approval was just inserted"))
    }

    async fn list(
        &self,
        company_id: &str,
        status: Option<&str>,
    ) -> Result<Vec<ApprovalRecord>, ApprovalError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql = match status {
            Some(_) => format!(
                "SELECT {COLUMNS} FROM approvals WHERE company_id = ?1 AND status = ?2 ORDER BY created_at DESC"
            ),
            None => format!(
                "SELECT {COLUMNS} FROM approvals WHERE company_id = ?1 ORDER BY created_at DESC"
            ),
        };
        let params: Vec<libsql::Value> = match status {
            Some(status) => vec![company_id.into(), status.into()],
            None => vec![company_id.into()],
        };
        let mut rows = conn.query(&sql, params).await?;
        let mut approvals = Vec::new();
        while let Some(row) = rows.next().await? {
            approvals.push(row_to_approval(&row)?);
        }
        Ok(approvals)
    }

    async fn get(&self, id: &str) -> Result<Option<ApprovalRecord>, ApprovalError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql = format!("SELECT {COLUMNS} FROM approvals WHERE id = ?1");
        let mut rows = conn.query(&sql, libsql::params![id]).await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_approval(&row)?)),
            None => Ok(None),
        }
    }

    async fn decide(
        &self,
        id: &str,
        decision: ApprovalDecision,
    ) -> Result<Option<ApprovalRecord>, ApprovalError> {
        if !matches!(
            decision.decision.as_str(),
            "approved" | "rejected" | "request_revision"
        ) {
            return Err(ApprovalError::InvalidDecision);
        }
        let conn = crate::connection::connect(&self.db).await?;
        let approval = self.get(id).await?;
        let Some(approval) = approval else {
            return Ok(None);
        };
        let target_status = if decision.decision == "request_revision" {
            "revision_requested"
        } else {
            decision.decision.as_str()
        };
        if !allowed_approval_transition(&approval.status, target_status) {
            return Err(ApprovalError::NotPending);
        }
        conn.execute(
            "UPDATE approvals
             SET status = ?1, decision_note = ?2, decided_by_user_id = ?3,
                 decided_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?4",
            libsql::params![
                target_status,
                decision.decision_note,
                decision.decided_by_user_id,
                id
            ],
        )
        .await?;
        Ok(self.get(id).await?)
    }

    async fn cancel(&self, id: &str) -> Result<Option<ApprovalRecord>, ApprovalError> {
        let conn = crate::connection::connect(&self.db).await?;
        let approval = self.get(id).await?;
        let Some(approval) = approval else {
            return Ok(None);
        };
        if !allowed_approval_transition(&approval.status, "cancelled") {
            return Err(ApprovalError::NotPending);
        }
        conn.execute(
            "UPDATE approvals
             SET status = 'cancelled', updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?1",
            libsql::params![id],
        )
        .await?;
        Ok(self.get(id).await?)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{Connection, migrate, open};

    async fn repo() -> (TempDir, TursoApprovalRepository, Connection) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let conn = crate::connect(&db).await.unwrap();
        let repo = TursoApprovalRepository::new(db);
        (dir, repo, conn)
    }

    #[tokio::test]
    async fn approval_state_machine() {
        let (_dir, repo, conn) = repo().await;
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
            (),
        )
        .await
        .unwrap();

        let created = repo
            .create(NewApproval {
                company_id: "c1".to_owned(),
                r#type: "budget_override_required".to_owned(),
                requested_by_agent_id: None,
                requested_by_user_id: Some("u1".to_owned()),
                payload: r#"{"budgetMonthlyCents":500}"#.to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(created.status, "pending");

        // Approve.
        let decided = repo
            .decide(
                &created.id,
                ApprovalDecision {
                    decision: "approved".to_owned(),
                    decision_note: Some("ok".to_owned()),
                    decided_by_user_id: Some("board".to_owned()),
                },
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(decided.status, "approved");
        assert!(decided.decided_at.is_some());

        // Decide again -> NotPending.
        let error = repo
            .decide(
                &created.id,
                ApprovalDecision {
                    decision: "rejected".to_owned(),
                    decision_note: None,
                    decided_by_user_id: None,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(error, ApprovalError::NotPending));

        // Cancel another pending approval.
        let second = repo
            .create(NewApproval {
                company_id: "c1".to_owned(),
                r#type: "request_board_approval".to_owned(),
                requested_by_agent_id: None,
                requested_by_user_id: None,
                payload: "{}".to_owned(),
            })
            .await
            .unwrap();
        let cancelled = repo.cancel(&second.id).await.unwrap().unwrap();
        assert_eq!(cancelled.status, "cancelled");

        // Invalid decision value.
        let third = repo
            .create(NewApproval {
                company_id: "c1".to_owned(),
                r#type: "hire_agent".to_owned(),
                requested_by_agent_id: None,
                requested_by_user_id: None,
                payload: "{}".to_owned(),
            })
            .await
            .unwrap();
        let error = repo
            .decide(
                &third.id,
                ApprovalDecision {
                    decision: "maybe".to_owned(),
                    decision_note: None,
                    decided_by_user_id: None,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(error, ApprovalError::InvalidDecision));

        // List with status filter.
        let pending = repo.list("c1", Some("pending")).await.unwrap();
        assert_eq!(pending.len(), 1);
        let all = repo.list("c1", None).await.unwrap();
        assert_eq!(all.len(), 3);
    }

    #[tokio::test]
    async fn create_requires_company() {
        let (_dir, repo, _conn) = repo().await;
        let error = repo
            .create(NewApproval {
                company_id: "missing".to_owned(),
                r#type: "hire_agent".to_owned(),
                requested_by_agent_id: None,
                requested_by_user_id: None,
                payload: "{}".to_owned(),
            })
            .await
            .unwrap_err();
        assert!(matches!(error, ApprovalError::CompanyNotFound));
    }
    #[tokio::test]
    async fn request_revision_state_machine() {
        let (_dir, repo, conn) = repo().await;
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
            (),
        )
        .await
        .unwrap();
        let created = repo
            .create(NewApproval {
                company_id: "c1".to_owned(),
                r#type: "budget_override_required".to_owned(),
                requested_by_agent_id: None,
                requested_by_user_id: None,
                payload: "{}".to_owned(),
            })
            .await
            .unwrap();

        // pending -> revision_requested via request_revision decision.
        let revised = repo
            .decide(
                &created.id,
                ApprovalDecision {
                    decision: "request_revision".to_owned(),
                    decision_note: Some("please revise".to_owned()),
                    decided_by_user_id: Some("board".to_owned()),
                },
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(revised.status, "revision_requested");
        assert_eq!(revised.decision_note.as_deref(), Some("please revise"));

        // revision_requested -> approved.
        let approved = repo
            .decide(
                &created.id,
                ApprovalDecision {
                    decision: "approved".to_owned(),
                    decision_note: None,
                    decided_by_user_id: Some("board".to_owned()),
                },
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(approved.status, "approved");
    }
}
