//! Issue structure extensions: thread interactions, read states, issue
//! approvals, and execution decisions (SPEC §7.16 addenda).

use std::collections::HashMap;

use async_trait::async_trait;
use libsql::{Database, Row};
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
    /// Continuation policy.
    pub continuation_policy: String,
    /// Idempotency key.
    pub idempotency_key: Option<String>,
    /// Source comment id.
    pub source_comment_id: Option<String>,
    /// Source run id.
    pub source_run_id: Option<String>,
    /// Title.
    pub title: Option<String>,
    /// Summary.
    pub summary: Option<String>,
    /// Creating agent id.
    pub created_by_agent_id: Option<String>,
    /// Creating user id.
    pub created_by_user_id: Option<String>,
    /// Resolving agent id.
    pub resolved_by_agent_id: Option<String>,
    /// Resolving user id.
    pub resolved_by_user_id: Option<String>,
    /// Result JSON.
    pub result: Option<serde_json::Value>,
    /// ISO 8601 resolution time.
    pub resolved_at: Option<String>,
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

/// Outcome of creating a thread interaction.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateThreadInteractionOutcome {
    /// The created interaction.
    pub interaction: ThreadInteractionRecord,
    /// Prior pending request confirmations superseded by this interaction.
    pub superseded: Vec<ThreadInteractionRecord>,
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
    /// Creating agent id.
    pub created_by_agent_id: Option<String>,
    /// Creating user id.
    pub created_by_user_id: Option<String>,
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
    /// Creating a `request_confirmation` from an agent expires prior pending
    /// requests from the same agent on the same issue with
    /// `superseded_by_newer_request`.
    ///
    /// # Errors
    ///
    /// Returns [`IssueStructureError`] when the issue is missing.
    async fn create_thread_interaction(
        &self,
        input: NewThreadInteraction,
    ) -> Result<CreateThreadInteractionOutcome, IssueStructureError>;

    /// Expires all but the newest pending `request_confirmation` per
    /// (company, issue, kind, created_by_agent_id) group, idempotently.
    ///
    /// # Errors
    ///
    /// Returns [`IssueStructureError`] on database failure.
    async fn expire_superseded_pending_confirmations(&self) -> Result<i64, IssueStructureError>;

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

/// Column list for reading `issue_thread_interactions` rows.
const INTERACTION_COLUMNS: &str = "id, company_id, issue_id, kind, status, continuation_policy,
                                  idempotency_key, source_comment_id, source_run_id, title,
                                  summary, created_by_agent_id, created_by_user_id,
                                  resolved_by_agent_id, resolved_by_user_id, payload, result,
                                  resolved_at, created_at";

/// Reads a thread interaction row into a [`ThreadInteractionRecord`].
fn row_to_interaction(row: &Row) -> Result<ThreadInteractionRecord, libsql::Error> {
    Ok(ThreadInteractionRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        issue_id: helpers::row_text(row, 2)?.expect("issue_id"),
        kind: helpers::row_text(row, 3)?.expect("kind"),
        status: helpers::row_text(row, 4)?.expect("status"),
        continuation_policy: helpers::row_text(row, 5)?
            .unwrap_or_else(|| "wake_assignee".to_owned()),
        idempotency_key: helpers::row_text(row, 6)?,
        source_comment_id: helpers::row_text(row, 7)?,
        source_run_id: helpers::row_text(row, 8)?,
        title: helpers::row_text(row, 9)?,
        summary: helpers::row_text(row, 10)?,
        created_by_agent_id: helpers::row_text(row, 11)?,
        created_by_user_id: helpers::row_text(row, 12)?,
        resolved_by_agent_id: helpers::row_text(row, 13)?,
        resolved_by_user_id: helpers::row_text(row, 14)?,
        payload: helpers::row_text(row, 15)?.expect("payload"),
        result: helpers::row_text(row, 16)?.and_then(|raw| serde_json::from_str(&raw).ok()),
        resolved_at: helpers::row_text(row, 17)?,
        created_at: helpers::row_text(row, 18)?.expect("created_at"),
    })
}

/// Fetches a thread interaction by id.
async fn fetch_interaction(
    conn: &libsql::Connection,
    id: &str,
) -> Result<ThreadInteractionRecord, IssueStructureError> {
    let mut rows = conn
        .query(
            &format!("SELECT {INTERACTION_COLUMNS} FROM issue_thread_interactions WHERE id = ?1"),
            libsql::params![id],
        )
        .await?;
    let row = rows.next().await?.expect("interaction was just inserted");
    Ok(row_to_interaction(&row)?)
}

#[async_trait]
impl IssueStructureRepository for TursoIssueStructureRepository {
    async fn create_thread_interaction(
        &self,
        input: NewThreadInteraction,
    ) -> Result<CreateThreadInteractionOutcome, IssueStructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let tx = conn.transaction().await?;
        if !helpers::row_belongs_to_company(&tx, "issues", &input.issue_id, &input.company_id)
            .await?
        {
            return Err(IssueStructureError::IssueNotFound);
        }
        let id = Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO issue_thread_interactions (id, company_id, issue_id, kind, status,
                                                    payload, created_by_agent_id,
                                                    created_by_user_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'pending', ?5, ?6, ?7,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id.clone(),
                input.issue_id.clone(),
                input.kind.clone(),
                input.payload,
                input.created_by_agent_id.clone(),
                input.created_by_user_id.clone()
            ],
        )
        .await?;

        let mut superseded_ids = Vec::new();
        if input.kind == "request_confirmation"
            && let Some(agent_id) = &input.created_by_agent_id
        {
            let mut rows = tx
                .query(
                    "SELECT id FROM issue_thread_interactions
                     WHERE company_id = ?1 AND issue_id = ?2 AND kind = ?3
                       AND created_by_agent_id = ?4 AND status = 'pending' AND id != ?5",
                    libsql::params![
                        input.company_id.clone(),
                        input.issue_id.clone(),
                        input.kind.clone(),
                        agent_id.clone(),
                        id.clone()
                    ],
                )
                .await?;
            while let Some(row) = rows.next().await? {
                superseded_ids.push(helpers::row_text(&row, 0)?.expect("id"));
            }
            let result = serde_json::json!({
                "version": 1,
                "outcome": "superseded_by_newer_request",
                "supersededByInteractionId": id,
            });
            for superseded_id in &superseded_ids {
                tx.execute(
                    "UPDATE issue_thread_interactions
                     SET status = 'expired',
                         result = ?1,
                         resolved_by_agent_id = ?2,
                         resolved_by_user_id = ?3,
                         resolved_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                     WHERE id = ?4 AND status = 'pending'",
                    libsql::params![
                        result.to_string(),
                        input.created_by_agent_id.clone(),
                        input.created_by_user_id.clone(),
                        superseded_id.clone()
                    ],
                )
                .await?;
            }
        }
        tx.commit().await?;

        let interaction = fetch_interaction(&conn, &id).await?;
        let mut superseded = Vec::new();
        if !superseded_ids.is_empty() {
            let placeholders = superseded_ids
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(", ");
            let params: Vec<libsql::Value> = superseded_ids
                .iter()
                .cloned()
                .map(libsql::Value::Text)
                .collect();
            let mut rows = conn
                .query(
                    &format!(
                        "SELECT {INTERACTION_COLUMNS} FROM issue_thread_interactions
                         WHERE id IN ({placeholders})"
                    ),
                    libsql::params_from_iter(params),
                )
                .await?;
            while let Some(row) = rows.next().await? {
                superseded.push(row_to_interaction(&row)?);
            }
        }
        Ok(CreateThreadInteractionOutcome {
            interaction,
            superseded,
        })
    }

    async fn expire_superseded_pending_confirmations(&self) -> Result<i64, IssueStructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let tx = conn.transaction().await?;
        let mut rows = tx
            .query(
                "SELECT id, company_id, issue_id, kind, created_by_agent_id
                 FROM issue_thread_interactions
                 WHERE kind = 'request_confirmation' AND status = 'pending'
                   AND created_by_agent_id IS NOT NULL
                 ORDER BY company_id, issue_id, kind, created_by_agent_id,
                          created_at DESC, id DESC",
                (),
            )
            .await?;
        let mut newest_by_group = HashMap::new();
        let mut expired: i64 = 0;
        while let Some(row) = rows.next().await? {
            let id = helpers::row_text(&row, 0)?.expect("id");
            let group = (
                helpers::row_text(&row, 1)?.expect("company_id"),
                helpers::row_text(&row, 2)?.expect("issue_id"),
                helpers::row_text(&row, 3)?.expect("kind"),
                helpers::row_text(&row, 4)?.expect("created_by_agent_id"),
            );
            if let Some(newest_id) = newest_by_group.get(&group) {
                let result = serde_json::json!({
                    "version": 1,
                    "outcome": "superseded_by_newer_request",
                    "supersededByInteractionId": newest_id,
                });
                let affected = tx
                    .execute(
                        "UPDATE issue_thread_interactions
                         SET status = 'expired',
                             result = ?1,
                             resolved_by_agent_id = NULL,
                             resolved_by_user_id = NULL,
                             resolved_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                         WHERE id = ?2 AND status = 'pending'",
                        libsql::params![result.to_string(), id],
                    )
                    .await?;
                expired += i64::try_from(affected).unwrap_or(i64::MAX);
            } else {
                newest_by_group.insert(group, id);
            }
        }
        tx.commit().await?;
        Ok(expired)
    }

    async fn list_thread_interactions(
        &self,
        issue_id: &str,
    ) -> Result<Vec<ThreadInteractionRecord>, IssueStructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, issue_id, kind, status, continuation_policy, idempotency_key,
                        source_comment_id, source_run_id, title, summary, created_by_agent_id,
                        created_by_user_id, resolved_by_agent_id, resolved_by_user_id, payload,
                        result, resolved_at, created_at
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
                continuation_policy: helpers::row_text(&row, 5)?
                    .unwrap_or_else(|| "wake_assignee".to_owned()),
                idempotency_key: helpers::row_text(&row, 6)?,
                source_comment_id: helpers::row_text(&row, 7)?,
                source_run_id: helpers::row_text(&row, 8)?,
                title: helpers::row_text(&row, 9)?,
                summary: helpers::row_text(&row, 10)?,
                created_by_agent_id: helpers::row_text(&row, 11)?,
                created_by_user_id: helpers::row_text(&row, 12)?,
                resolved_by_agent_id: helpers::row_text(&row, 13)?,
                resolved_by_user_id: helpers::row_text(&row, 14)?,
                payload: helpers::row_text(&row, 15)?.expect("payload"),
                result: helpers::row_text(&row, 16)?
                    .and_then(|raw| serde_json::from_str(&raw).ok()),
                resolved_at: helpers::row_text(&row, 17)?,
                created_at: helpers::row_text(&row, 18)?.expect("created_at"),
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

    async fn repo() -> (TempDir, libsql::Database, TursoIssueStructureRepository) {
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
        // A second handle over the same file keeps the original `db` usable
        // by the tests for direct SQL setup.
        let repo = TursoIssueStructureRepository::new(
            open(&crate::DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        );
        (dir, db, repo)
    }

    #[tokio::test]
    async fn thread_read_approval_decision_roundtrip() {
        let (_dir, _db, repo) = repo().await;

        let interaction = repo
            .create_thread_interaction(NewThreadInteraction {
                company_id: "c1".to_owned(),
                issue_id: "i1".to_owned(),
                kind: "review_request".to_owned(),
                payload: r#"{"reviewer":"u1"}"#.to_owned(),
                created_by_agent_id: None,
                created_by_user_id: None,
            })
            .await
            .unwrap();
        assert_eq!(interaction.interaction.status, "pending");
        assert!(interaction.superseded.is_empty());
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

    #[tokio::test]
    async fn new_request_confirmation_supersedes_older_pending() {
        let (_dir, db, repo) = repo().await;
        let conn = crate::connect(&db).await.unwrap();
        conn.execute(
            "INSERT INTO agents (id, company_id, name, role, adapter_type)
             VALUES ('a2', 'c1', 'two', 'engineer', 'codex_local')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO issues (id, company_id, title, issue_number, identifier)
             VALUES ('i2', 'c1', 'T2', 2, 'ALPHA-2')",
            (),
        )
        .await
        .unwrap();

        let base = |kind: &str, issue: &str, agent: Option<&str>| NewThreadInteraction {
            company_id: "c1".to_owned(),
            issue_id: issue.to_owned(),
            kind: kind.to_owned(),
            payload: "{}".to_owned(),
            created_by_agent_id: agent.map(str::to_owned),
            created_by_user_id: None,
        };

        let first = repo
            .create_thread_interaction(base("request_confirmation", "i1", Some("a1")))
            .await
            .unwrap();
        assert_eq!(first.interaction.status, "pending");
        assert_eq!(first.interaction.created_by_agent_id.as_deref(), Some("a1"));
        assert!(first.superseded.is_empty());

        // A different agent on the same issue must not supersede.
        let other_agent = repo
            .create_thread_interaction(base("request_confirmation", "i1", Some("a2")))
            .await
            .unwrap();
        assert!(other_agent.superseded.is_empty());

        // The same agent on a different issue must not supersede.
        let other_issue = repo
            .create_thread_interaction(base("request_confirmation", "i2", Some("a1")))
            .await
            .unwrap();
        assert!(other_issue.superseded.is_empty());

        // Non-confirmation kinds never supersede, even from the same agent.
        let review = repo
            .create_thread_interaction(base("review_request", "i1", Some("a1")))
            .await
            .unwrap();
        assert!(review.superseded.is_empty());

        // A newer request_confirmation from the same agent + issue supersedes.
        let replacement = repo
            .create_thread_interaction(base("request_confirmation", "i1", Some("a1")))
            .await
            .unwrap();
        assert_eq!(replacement.superseded.len(), 1);
        let superseded = &replacement.superseded[0];
        assert_eq!(superseded.id, first.interaction.id);
        assert_eq!(superseded.status, "expired");
        let result = superseded.result.as_ref().expect("expired result");
        assert_eq!(result["version"], 1);
        assert_eq!(result["outcome"], "superseded_by_newer_request");
        assert_eq!(
            result["supersededByInteractionId"],
            replacement.interaction.id
        );
        assert!(superseded.resolved_at.is_some());
        assert_eq!(superseded.resolved_by_agent_id.as_deref(), Some("a1"));

        // Read-back confirms the same state persisted.
        let list = repo.list_thread_interactions("i1").await.unwrap();
        let by_id = |id: &str| list.iter().find(|r| r.id == id).unwrap();
        assert_eq!(by_id(&first.interaction.id).status, "expired");
        assert_eq!(
            by_id(&first.interaction.id).result.as_ref().unwrap()["outcome"],
            "superseded_by_newer_request"
        );
        assert_eq!(
            by_id(&first.interaction.id).result.as_ref().unwrap()["supersededByInteractionId"],
            replacement.interaction.id
        );
        assert_eq!(by_id(&other_agent.interaction.id).status, "pending");
        assert_eq!(by_id(&review.interaction.id).status, "pending");
        assert_eq!(
            repo.list_thread_interactions("i2").await.unwrap()[0].status,
            "pending"
        );
    }

    #[tokio::test]
    async fn sweep_expires_non_newest_pending_confirmations_idempotently() {
        let (_dir, db, repo) = repo().await;
        let conn = crate::connect(&db).await.unwrap();
        conn.execute(
            "INSERT INTO agents (id, company_id, name, role, adapter_type)
             VALUES ('a2', 'c1', 'two', 'engineer', 'codex_local')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO issues (id, company_id, title, issue_number, identifier)
             VALUES ('i2', 'c1', 'T2', 2, 'ALPHA-2')",
            (),
        )
        .await
        .unwrap();

        // Seed pending rows directly (bypassing the create-time supersede) to
        // simulate legacy/racing duplicates the sweep must clean up.
        async fn insert_pending(
            conn: &libsql::Connection,
            id: &str,
            issue: &str,
            agent: Option<&str>,
            created_at: &str,
        ) {
            conn.execute(
                "INSERT INTO issue_thread_interactions
                    (id, company_id, issue_id, kind, status, payload, created_by_agent_id,
                     created_at, updated_at)
                 VALUES (?1, 'c1', ?2, 'request_confirmation', 'pending', '{}', ?3, ?4, ?4)",
                libsql::params![id, issue, agent.map(str::to_owned), created_at],
            )
            .await
            .unwrap();
        }

        insert_pending(&conn, "old-1", "i1", Some("a1"), "2026-08-01T00:00:00.000Z").await;
        insert_pending(&conn, "mid-1", "i1", Some("a1"), "2026-08-02T00:00:00.000Z").await;
        insert_pending(&conn, "new-1", "i1", Some("a1"), "2026-08-03T00:00:00.000Z").await;
        insert_pending(
            &conn,
            "a2-old",
            "i1",
            Some("a2"),
            "2026-08-01T00:00:00.000Z",
        )
        .await;
        insert_pending(
            &conn,
            "a2-new",
            "i1",
            Some("a2"),
            "2026-08-02T00:00:00.000Z",
        )
        .await;
        insert_pending(
            &conn,
            "other-issue",
            "i2",
            Some("a1"),
            "2026-08-01T00:00:00.000Z",
        )
        .await;
        // A user-created confirmation (no agent) must never be swept.
        conn.execute(
            "INSERT INTO issue_thread_interactions
                (id, company_id, issue_id, kind, status, payload, created_by_user_id,
                 created_at, updated_at)
             VALUES ('user-1', 'c1', 'i1', 'request_confirmation', 'pending', '{}', 'u1',
                     '2026-08-01T00:00:00.000Z', '2026-08-01T00:00:00.000Z')",
            (),
        )
        .await
        .unwrap();

        let expired = repo
            .expire_superseded_pending_confirmations()
            .await
            .unwrap();
        assert_eq!(expired, 3);

        let list = repo.list_thread_interactions("i1").await.unwrap();
        let by_id = |id: &str| list.iter().find(|r| r.id == id).unwrap();
        assert_eq!(by_id("old-1").status, "expired");
        assert_eq!(
            by_id("old-1").result.as_ref().unwrap()["supersededByInteractionId"],
            "new-1"
        );
        assert_eq!(by_id("mid-1").status, "expired");
        assert_eq!(
            by_id("mid-1").result.as_ref().unwrap()["supersededByInteractionId"],
            "new-1"
        );
        assert_eq!(by_id("new-1").status, "pending");
        assert_eq!(by_id("a2-old").status, "expired");
        assert_eq!(
            by_id("a2-old").result.as_ref().unwrap()["supersededByInteractionId"],
            "a2-new"
        );
        assert_eq!(by_id("a2-new").status, "pending");
        assert!(by_id("old-1").resolved_by_agent_id.is_none());
        assert!(by_id("old-1").resolved_at.is_some());
        // User-created and single-row groups are untouched.
        assert_eq!(by_id("user-1").status, "pending");
        assert_eq!(
            repo.list_thread_interactions("i2").await.unwrap()[0].status,
            "pending"
        );

        // Idempotent: a second sweep expires nothing and changes nothing.
        assert_eq!(
            repo.expire_superseded_pending_confirmations()
                .await
                .unwrap(),
            0
        );
        let list = repo.list_thread_interactions("i1").await.unwrap();
        let by_id = |id: &str| list.iter().find(|r| r.id == id).unwrap();
        assert_eq!(by_id("new-1").status, "pending");
        assert_eq!(by_id("a2-new").status, "pending");
        assert_eq!(by_id("user-1").status, "pending");
    }
}
