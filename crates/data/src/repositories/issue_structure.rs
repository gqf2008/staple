//! Issue structure extensions: thread interactions, read states, issue
//! approvals, and execution decisions (SPEC §7.16 addenda).

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A thread interaction on an issue.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadInteractionRecord {
    /// Interaction id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Issue id.
    pub issue_id: String,
    /// Kind.
    pub kind: String,
    /// Status (`pending` by default).
    pub status: String,
    /// Payload JSON.
    pub payload: String,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// Issue read state for a user.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueReadStateRecord {
    /// Read state id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Issue id.
    pub issue_id: String,
    /// User id.
    pub user_id: String,
    /// ISO 8601 last read time.
    pub last_read_at: String,
}

/// An issue-approval link.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueApprovalRecord {
    /// Issue id.
    pub issue_id: String,
    /// Approval id.
    pub approval_id: String,
    /// Owning company id.
    pub company_id: String,
}

/// An execution decision on an issue.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionDecisionRecord {
    /// Decision id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Issue id.
    pub issue_id: String,
    /// Stage id.
    pub stage_id: String,
    /// Stage type.
    pub stage_type: String,
    /// Actor agent id.
    pub actor_agent_id: Option<String>,
    /// Actor user id.
    pub actor_user_id: Option<String>,
    /// Outcome.
    pub outcome: String,
    /// Body.
    pub body: String,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// Input for creating a thread interaction.
#[derive(Debug, Clone)]
pub struct NewThreadInteraction {
    /// Owning company id.
    pub company_id: String,
    /// Issue id.
    pub issue_id: String,
    /// Kind.
    pub kind: String,
    /// Payload JSON.
    pub payload: String,
}

/// Input for an execution decision.
#[derive(Debug, Clone)]
pub struct NewExecutionDecision {
    /// Owning company id.
    pub company_id: String,
    /// Issue id.
    pub issue_id: String,
    /// Stage id.
    pub stage_id: String,
    /// Stage type.
    pub stage_type: String,
    /// Actor agent id.
    pub actor_agent_id: Option<String>,
    /// Actor user id.
    pub actor_user_id: Option<String>,
    /// Outcome.
    pub outcome: String,
    /// Body.
    pub body: String,
}

/// Issue structure repository errors.
#[derive(Debug, Error)]
pub enum IssueStructureError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The issue does not exist in this company.
    #[error("issue not found")]
    IssueNotFound,
    /// The referenced approval does not exist in this company.
    #[error("approval not found")]
    ApprovalNotFound,
    /// The link already exists.
    #[error("link already exists")]
    AlreadyExists,
}

/// Issue structure persistence contract.
#[async_trait]
pub trait IssueStructureRepository: Send + Sync {
    /// Creates a thread interaction.
    ///
    /// # Errors
    ///
    /// Returns [`IssueStructureError`] when the issue is missing.
    async fn create_thread_interaction(
        &self,
        input: NewThreadInteraction,
    ) -> Result<ThreadInteractionRecord, IssueStructureError>;

    /// Lists thread interactions for an issue.
    ///
    /// # Errors
    ///
    /// Returns [`IssueStructureError`] on database failure.
    async fn list_thread_interactions(
        &self,
        issue_id: &str,
    ) -> Result<Vec<ThreadInteractionRecord>, IssueStructureError>;

    /// Upserts the read state for a user on an issue.
    ///
    /// # Errors
    ///
    /// Returns [`IssueStructureError`] when the issue is missing.
    async fn upsert_read_state(
        &self,
        company_id: &str,
        issue_id: &str,
        user_id: &str,
        last_read_at: &str,
    ) -> Result<IssueReadStateRecord, IssueStructureError>;

    /// Fetches the read state for a user on an issue.
    ///
    /// # Errors
    ///
    /// Returns [`IssueStructureError`] on database failure.
    async fn get_read_state(
        &self,
        issue_id: &str,
        user_id: &str,
    ) -> Result<Option<IssueReadStateRecord>, IssueStructureError>;

    /// Links an approval to an issue.
    ///
    /// # Errors
    ///
    /// Returns [`IssueStructureError`] on invalid references or duplicates.
    async fn link_approval(
        &self,
        company_id: &str,
        issue_id: &str,
        approval_id: &str,
    ) -> Result<IssueApprovalRecord, IssueStructureError>;

    /// Lists approvals linked to an issue.
    ///
    /// # Errors
    ///
    /// Returns [`IssueStructureError`] on database failure.
    async fn list_issue_approvals(
        &self,
        issue_id: &str,
    ) -> Result<Vec<IssueApprovalRecord>, IssueStructureError>;

    /// Records an execution decision.
    ///
    /// # Errors
    ///
    /// Returns [`IssueStructureError`] when the issue is missing.
    async fn create_execution_decision(
        &self,
        input: NewExecutionDecision,
    ) -> Result<ExecutionDecisionRecord, IssueStructureError>;

    /// Lists execution decisions for an issue.
    ///
    /// # Errors
    ///
    /// Returns [`IssueStructureError`] on database failure.
    async fn list_execution_decisions(
        &self,
        issue_id: &str,
    ) -> Result<Vec<ExecutionDecisionRecord>, IssueStructureError>;
}

/// Turso/libSQL implementation of [`IssueStructureRepository`].
#[derive(Debug)]
pub struct TursoIssueStructureRepository {
    db: Database,
}

impl TursoIssueStructureRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl IssueStructureRepository for TursoIssueStructureRepository {
    async fn create_thread_interaction(
        &self,
        input: NewThreadInteraction,
    ) -> Result<ThreadInteractionRecord, IssueStructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(&conn, "issues", &input.issue_id, &input.company_id)
            .await?
        {
            return Err(IssueStructureError::IssueNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO issue_thread_interactions (id, company_id, issue_id, kind, status,
                                                    payload, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'pending', ?5,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.issue_id,
                input.kind,
                input.payload
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, issue_id, kind, status, payload, created_at
                 FROM issue_thread_interactions WHERE id = ?1",
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("interaction was just inserted");
        Ok(ThreadInteractionRecord {
            id: helpers::row_text(&row, 0)?.expect("id"),
            company_id: helpers::row_text(&row, 1)?.expect("company_id"),
            issue_id: helpers::row_text(&row, 2)?.expect("issue_id"),
            kind: helpers::row_text(&row, 3)?.expect("kind"),
            status: helpers::row_text(&row, 4)?.expect("status"),
            payload: helpers::row_text(&row, 5)?.expect("payload"),
            created_at: helpers::row_text(&row, 6)?.expect("created_at"),
        })
    }

    async fn list_thread_interactions(
        &self,
        issue_id: &str,
    ) -> Result<Vec<ThreadInteractionRecord>, IssueStructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, issue_id, kind, status, payload, created_at
                 FROM issue_thread_interactions WHERE issue_id = ?1 ORDER BY created_at",
                libsql::params![issue_id],
            )
            .await?;
        let mut interactions = Vec::new();
        while let Some(row) = rows.next().await? {
            interactions.push(ThreadInteractionRecord {
                id: helpers::row_text(&row, 0)?.expect("id"),
                company_id: helpers::row_text(&row, 1)?.expect("company_id"),
                issue_id: helpers::row_text(&row, 2)?.expect("issue_id"),
                kind: helpers::row_text(&row, 3)?.expect("kind"),
                status: helpers::row_text(&row, 4)?.expect("status"),
                payload: helpers::row_text(&row, 5)?.expect("payload"),
                created_at: helpers::row_text(&row, 6)?.expect("created_at"),
            });
        }
        Ok(interactions)
    }

    async fn upsert_read_state(
        &self,
        company_id: &str,
        issue_id: &str,
        user_id: &str,
        last_read_at: &str,
    ) -> Result<IssueReadStateRecord, IssueStructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(&conn, "issues", issue_id, company_id).await? {
            return Err(IssueStructureError::IssueNotFound);
        }
        conn.execute(
            "INSERT INTO issue_read_states (id, company_id, issue_id, user_id, last_read_at,
                                            created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT (company_id, issue_id, user_id) DO UPDATE SET
                 last_read_at = excluded.last_read_at,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            libsql::params![
                Uuid::new_v4().to_string(),
                company_id,
                issue_id,
                user_id,
                last_read_at
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, issue_id, user_id, last_read_at
                 FROM issue_read_states WHERE company_id = ?1 AND issue_id = ?2 AND user_id = ?3",
                libsql::params![company_id, issue_id, user_id],
            )
            .await?;
        let row = rows.next().await?.expect("read state exists");
        Ok(IssueReadStateRecord {
            id: helpers::row_text(&row, 0)?.expect("id"),
            company_id: helpers::row_text(&row, 1)?.expect("company_id"),
            issue_id: helpers::row_text(&row, 2)?.expect("issue_id"),
            user_id: helpers::row_text(&row, 3)?.expect("user_id"),
            last_read_at: helpers::row_text(&row, 4)?.expect("last_read_at"),
        })
    }

    async fn get_read_state(
        &self,
        issue_id: &str,
        user_id: &str,
    ) -> Result<Option<IssueReadStateRecord>, IssueStructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, issue_id, user_id, last_read_at
                 FROM issue_read_states WHERE issue_id = ?1 AND user_id = ?2",
                libsql::params![issue_id, user_id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(IssueReadStateRecord {
                id: helpers::row_text(&row, 0)?.expect("id"),
                company_id: helpers::row_text(&row, 1)?.expect("company_id"),
                issue_id: helpers::row_text(&row, 2)?.expect("issue_id"),
                user_id: helpers::row_text(&row, 3)?.expect("user_id"),
                last_read_at: helpers::row_text(&row, 4)?.expect("last_read_at"),
            })),
            None => Ok(None),
        }
    }

    async fn link_approval(
        &self,
        company_id: &str,
        issue_id: &str,
        approval_id: &str,
    ) -> Result<IssueApprovalRecord, IssueStructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(&conn, "issues", issue_id, company_id).await? {
            return Err(IssueStructureError::IssueNotFound);
        }
        if !helpers::row_belongs_to_company(&conn, "approvals", approval_id, company_id).await? {
            return Err(IssueStructureError::ApprovalNotFound);
        }
        let result = conn
            .execute(
                "INSERT INTO issue_approvals (issue_id, approval_id, company_id, created_at)
                 VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![issue_id, approval_id, company_id],
            )
            .await;
        match result {
            Ok(_) => Ok(IssueApprovalRecord {
                issue_id: issue_id.to_owned(),
                approval_id: approval_id.to_owned(),
                company_id: company_id.to_owned(),
            }),
            Err(error)
                if error.to_string().contains("PRIMARY KEY")
                    || error.to_string().contains("UNIQUE constraint failed") =>
            {
                Err(IssueStructureError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_issue_approvals(
        &self,
        issue_id: &str,
    ) -> Result<Vec<IssueApprovalRecord>, IssueStructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT issue_id, approval_id, company_id FROM issue_approvals
                 WHERE issue_id = ?1 ORDER BY created_at",
                libsql::params![issue_id],
            )
            .await?;
        let mut approvals = Vec::new();
        while let Some(row) = rows.next().await? {
            approvals.push(IssueApprovalRecord {
                issue_id: helpers::row_text(&row, 0)?.expect("issue_id"),
                approval_id: helpers::row_text(&row, 1)?.expect("approval_id"),
                company_id: helpers::row_text(&row, 2)?.expect("company_id"),
            });
        }
        Ok(approvals)
    }

    async fn create_execution_decision(
        &self,
        input: NewExecutionDecision,
    ) -> Result<ExecutionDecisionRecord, IssueStructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(&conn, "issues", &input.issue_id, &input.company_id)
            .await?
        {
            return Err(IssueStructureError::IssueNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO issue_execution_decisions (id, company_id, issue_id, stage_id, stage_type,
                                                    actor_agent_id, actor_user_id, outcome, body,
                                                    created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.issue_id,
                input.stage_id,
                input.stage_type,
                input.actor_agent_id,
                input.actor_user_id,
                input.outcome,
                input.body
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, issue_id, stage_id, stage_type, actor_agent_id,
                        actor_user_id, outcome, body, created_at
                 FROM issue_execution_decisions WHERE id = ?1",
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("decision was just inserted");
        Ok(ExecutionDecisionRecord {
            id: helpers::row_text(&row, 0)?.expect("id"),
            company_id: helpers::row_text(&row, 1)?.expect("company_id"),
            issue_id: helpers::row_text(&row, 2)?.expect("issue_id"),
            stage_id: helpers::row_text(&row, 3)?.expect("stage_id"),
            stage_type: helpers::row_text(&row, 4)?.expect("stage_type"),
            actor_agent_id: helpers::row_text(&row, 5)?,
            actor_user_id: helpers::row_text(&row, 6)?,
            outcome: helpers::row_text(&row, 7)?.expect("outcome"),
            body: helpers::row_text(&row, 8)?.expect("body"),
            created_at: helpers::row_text(&row, 9)?.expect("created_at"),
        })
    }

    async fn list_execution_decisions(
        &self,
        issue_id: &str,
    ) -> Result<Vec<ExecutionDecisionRecord>, IssueStructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, issue_id, stage_id, stage_type, actor_agent_id,
                        actor_user_id, outcome, body, created_at
                 FROM issue_execution_decisions WHERE issue_id = ?1 ORDER BY created_at",
                libsql::params![issue_id],
            )
            .await?;
        let mut decisions = Vec::new();
        while let Some(row) = rows.next().await? {
            decisions.push(ExecutionDecisionRecord {
                id: helpers::row_text(&row, 0)?.expect("id"),
                company_id: helpers::row_text(&row, 1)?.expect("company_id"),
                issue_id: helpers::row_text(&row, 2)?.expect("issue_id"),
                stage_id: helpers::row_text(&row, 3)?.expect("stage_id"),
                stage_type: helpers::row_text(&row, 4)?.expect("stage_type"),
                actor_agent_id: helpers::row_text(&row, 5)?,
                actor_user_id: helpers::row_text(&row, 6)?,
                outcome: helpers::row_text(&row, 7)?.expect("outcome"),
                body: helpers::row_text(&row, 8)?.expect("body"),
                created_at: helpers::row_text(&row, 9)?.expect("created_at"),
            });
        }
        Ok(decisions)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoIssueStructureRepository) {
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
            "INSERT INTO agents (id, company_id, name, role, adapter_type)
             VALUES ('a1', 'c1', 'one', 'engineer', 'codex_local')",
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
        conn.execute(
            "INSERT INTO approvals (id, company_id, type, payload, status)
             VALUES ('ap1', 'c1', 'hire_agent', '{}', 'pending')",
            (),
        )
        .await
        .unwrap();
        let repo = TursoIssueStructureRepository::new(db);
        (dir, repo)
    }

    #[tokio::test]
    async fn thread_read_approval_decision_roundtrip() {
        let (_dir, repo) = repo().await;

        let interaction = repo
            .create_thread_interaction(NewThreadInteraction {
                company_id: "c1".to_owned(),
                issue_id: "i1".to_owned(),
                kind: "review_request".to_owned(),
                payload: r#"{"reviewer":"u1"}"#.to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(interaction.status, "pending");
        let list = repo.list_thread_interactions("i1").await.unwrap();
        assert_eq!(list.len(), 1);

        let read = repo
            .upsert_read_state("c1", "i1", "u1", "2026-08-03T00:00:00.000Z")
            .await
            .unwrap();
        assert_eq!(read.last_read_at, "2026-08-03T00:00:00.000Z");
        let read2 = repo
            .upsert_read_state("c1", "i1", "u1", "2026-08-03T01:00:00.000Z")
            .await
            .unwrap();
        assert_eq!(read2.last_read_at, "2026-08-03T01:00:00.000Z");
        assert_eq!(read2.id, read.id);

        let link = repo.link_approval("c1", "i1", "ap1").await.unwrap();
        assert_eq!(link.approval_id, "ap1");
        let error = repo.link_approval("c1", "i1", "ap1").await.unwrap_err();
        assert!(matches!(error, IssueStructureError::AlreadyExists));

        let decision = repo
            .create_execution_decision(NewExecutionDecision {
                company_id: "c1".to_owned(),
                issue_id: "i1".to_owned(),
                stage_id: "stage-1".to_owned(),
                stage_type: "review".to_owned(),
                actor_agent_id: Some("a1".to_owned()),
                actor_user_id: None,
                outcome: "approved".to_owned(),
                body: "looks good".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(decision.outcome, "approved");
        let decisions = repo.list_execution_decisions("i1").await.unwrap();
        assert_eq!(decisions.len(), 1);
    }
}
