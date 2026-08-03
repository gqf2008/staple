//! Agent runtime repository: task sessions, runtime state, wakeup requests,
//! and issue recovery actions.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `agent_task_sessions` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTaskSessionRecord {
    /// Session id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Agent id.
    pub agent_id: String,
    /// Adapter type.
    pub adapter_type: String,
    /// Task key.
    pub task_key: String,
    /// Session params JSON.
    pub session_params_json: Option<serde_json::Value>,
    /// Display id.
    pub session_display_id: Option<String>,
    /// Last heartbeat run id.
    pub last_run_id: Option<String>,
    /// Last error.
    pub last_error: Option<String>,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// A row of the `agent_runtime_state` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRuntimeStateRecord {
    /// Agent id.
    pub agent_id: String,
    /// Owning company id.
    pub company_id: String,
    /// Adapter type.
    pub adapter_type: String,
    /// Session id.
    pub session_id: Option<String>,
    /// State JSON.
    pub state_json: serde_json::Value,
    /// Last run id.
    pub last_run_id: Option<String>,
    /// Last run status.
    pub last_run_status: Option<String>,
    /// Total input tokens.
    pub total_input_tokens: i64,
    /// Total output tokens.
    pub total_output_tokens: i64,
    /// Total cached input tokens.
    pub total_cached_input_tokens: i64,
    /// Total cost cents.
    pub total_cost_cents: i64,
    /// Last error.
    pub last_error: Option<String>,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// A row of the `agent_wakeup_requests` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentWakeupRequestRecord {
    /// Request id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Agent id.
    pub agent_id: String,
    /// Source.
    pub source: String,
    /// Trigger detail.
    pub trigger_detail: Option<String>,
    /// Reason.
    pub reason: Option<String>,
    /// Payload JSON.
    pub payload: Option<serde_json::Value>,
    /// Status.
    pub status: String,
    /// Coalesced count.
    pub coalesced_count: i64,
    /// Requesting actor type.
    pub requested_by_actor_type: Option<String>,
    /// Requesting actor id.
    pub requested_by_actor_id: Option<String>,
    /// Idempotency key.
    pub idempotency_key: Option<String>,
    /// Run id.
    pub run_id: Option<String>,
    /// ISO 8601 requested.
    pub requested_at: String,
    /// ISO 8601 claimed.
    pub claimed_at: Option<String>,
    /// ISO 8601 finished.
    pub finished_at: Option<String>,
    /// Error.
    pub error: Option<String>,
}

/// A row of the `issue_recovery_actions` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueRecoveryActionRecord {
    /// Action id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Source issue id.
    pub source_issue_id: String,
    /// Recovery issue id.
    pub recovery_issue_id: Option<String>,
    /// Kind.
    pub kind: String,
    /// Status (`active` | `escalated` | `resolved` | `cancelled`).
    pub status: String,
    /// Owner type.
    pub owner_type: String,
    /// Owner agent id.
    pub owner_agent_id: Option<String>,
    /// Owner user id.
    pub owner_user_id: Option<String>,
    /// Previous owner agent id.
    pub previous_owner_agent_id: Option<String>,
    /// Return owner agent id.
    pub return_owner_agent_id: Option<String>,
    /// Cause.
    pub cause: String,
    /// Fingerprint.
    pub fingerprint: String,
    /// Evidence JSON.
    pub evidence: serde_json::Value,
    /// Next action.
    pub next_action: String,
    /// Wake policy JSON.
    pub wake_policy: Option<serde_json::Value>,
    /// Monitor policy JSON.
    pub monitor_policy: Option<serde_json::Value>,
    /// Attempt count.
    pub attempt_count: i64,
    /// Max attempts.
    pub max_attempts: Option<i64>,
    /// ISO 8601 timeout.
    pub timeout_at: Option<String>,
    /// ISO 8601 last attempt.
    pub last_attempt_at: Option<String>,
    /// Outcome.
    pub outcome: Option<String>,
    /// Resolution note.
    pub resolution_note: Option<String>,
    /// ISO 8601 resolution.
    pub resolved_at: Option<String>,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// Input for upserting a task session.
#[derive(Debug, Clone)]
pub struct NewTaskSession {
    pub company_id: String,
    pub agent_id: String,
    pub adapter_type: String,
    pub task_key: String,
    pub session_params_json: Option<serde_json::Value>,
    pub session_display_id: Option<String>,
    pub last_run_id: Option<String>,
}

/// Input for upserting runtime state.
#[derive(Debug, Clone)]
pub struct NewRuntimeState {
    pub company_id: String,
    pub agent_id: String,
    pub adapter_type: String,
    pub session_id: Option<String>,
    pub state_json: serde_json::Value,
    pub last_run_id: Option<String>,
    pub last_run_status: Option<String>,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cached_input_tokens: i64,
    pub total_cost_cents: i64,
    pub last_error: Option<String>,
}

/// Input for enqueueing a wakeup request.
#[derive(Debug, Clone)]
pub struct NewWakeupRequest {
    pub company_id: String,
    pub agent_id: String,
    pub source: String,
    pub trigger_detail: Option<String>,
    pub reason: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub requested_by_actor_type: Option<String>,
    pub requested_by_actor_id: Option<String>,
    pub idempotency_key: Option<String>,
}

/// Input for creating a recovery action.
#[derive(Debug, Clone)]
pub struct NewRecoveryAction {
    pub company_id: String,
    pub source_issue_id: String,
    pub recovery_issue_id: Option<String>,
    pub kind: String,
    pub owner_agent_id: Option<String>,
    pub cause: String,
    pub fingerprint: String,
    pub evidence: serde_json::Value,
    pub next_action: String,
    pub wake_policy: Option<serde_json::Value>,
    pub monitor_policy: Option<serde_json::Value>,
    pub max_attempts: Option<i64>,
    pub timeout_at: Option<String>,
}

/// Agent runtime repository errors.
#[derive(Debug, Error)]
pub enum AgentRuntimeError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The company does not exist.
    #[error("company not found")]
    CompanyNotFound,
    /// The agent does not exist in this company.
    #[error("agent not found")]
    AgentNotFound,
    /// The source issue does not exist in this company.
    #[error("source issue not found")]
    SourceIssueNotFound,
    /// The referenced row does not exist.
    #[error("not found")]
    NotFound,
    /// A wakeup request is already claimed/finished.
    #[error("wakeup request not queued")]
    NotQueued,
    /// A recovery action is not active/escalated.
    #[error("recovery action not active")]
    NotActive,
}

/// Agent runtime persistence contract.
#[async_trait]
pub trait AgentRuntimeRepository: Send + Sync {
    // Task sessions --------------------------------------------------------
    async fn session_upsert(
        &self,
        input: NewTaskSession,
    ) -> Result<AgentTaskSessionRecord, AgentRuntimeError>;
    async fn session_list(
        &self,
        company_id: &str,
    ) -> Result<Vec<AgentTaskSessionRecord>, AgentRuntimeError>;
    async fn session_get(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<AgentTaskSessionRecord>, AgentRuntimeError>;

    // Runtime state --------------------------------------------------------
    async fn runtime_upsert(
        &self,
        input: NewRuntimeState,
    ) -> Result<AgentRuntimeStateRecord, AgentRuntimeError>;
    async fn runtime_get(
        &self,
        company_id: &str,
        agent_id: &str,
    ) -> Result<Option<AgentRuntimeStateRecord>, AgentRuntimeError>;

    // Wakeup requests ------------------------------------------------------
    async fn wakeup_enqueue(
        &self,
        input: NewWakeupRequest,
    ) -> Result<AgentWakeupRequestRecord, AgentRuntimeError>;
    async fn wakeup_claim(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<AgentWakeupRequestRecord>, AgentRuntimeError>;
    async fn wakeup_finish(
        &self,
        company_id: &str,
        id: &str,
        status: &str,
        error: Option<String>,
        run_id: Option<String>,
    ) -> Result<Option<AgentWakeupRequestRecord>, AgentRuntimeError>;
    async fn wakeup_list(
        &self,
        company_id: &str,
    ) -> Result<Vec<AgentWakeupRequestRecord>, AgentRuntimeError>;

    // Recovery actions -----------------------------------------------------
    async fn recovery_create(
        &self,
        input: NewRecoveryAction,
    ) -> Result<IssueRecoveryActionRecord, AgentRuntimeError>;
    async fn recovery_list(
        &self,
        company_id: &str,
    ) -> Result<Vec<IssueRecoveryActionRecord>, AgentRuntimeError>;
    async fn recovery_get(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<IssueRecoveryActionRecord>, AgentRuntimeError>;
    async fn recovery_set_status(
        &self,
        company_id: &str,
        id: &str,
        status: &str,
        outcome: Option<String>,
        resolution_note: Option<String>,
    ) -> Result<Option<IssueRecoveryActionRecord>, AgentRuntimeError>;
    async fn recovery_bump_attempt(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<IssueRecoveryActionRecord>, AgentRuntimeError>;
}

/// Turso/libSQL implementation of [`AgentRuntimeRepository`].
#[derive(Debug)]
pub struct TursoAgentRuntimeRepository {
    db: Database,
}

impl TursoAgentRuntimeRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

fn row_to_session(row: &libsql::Row) -> Result<AgentTaskSessionRecord, libsql::Error> {
    Ok(AgentTaskSessionRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        agent_id: helpers::row_text(row, 2)?.expect("agent_id"),
        adapter_type: helpers::row_text(row, 3)?.expect("adapter_type"),
        task_key: helpers::row_text(row, 4)?.expect("task_key"),
        session_params_json: helpers::row_text(row, 5)?
            .and_then(|raw| serde_json::from_str(&raw).ok()),
        session_display_id: helpers::row_text(row, 6)?,
        last_run_id: helpers::row_text(row, 7)?,
        last_error: helpers::row_text(row, 8)?,
        created_at: helpers::row_text(row, 9)?.expect("created_at"),
    })
}

fn row_to_runtime(row: &libsql::Row) -> Result<AgentRuntimeStateRecord, libsql::Error> {
    Ok(AgentRuntimeStateRecord {
        agent_id: helpers::row_text(row, 0)?.expect("agent_id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        adapter_type: helpers::row_text(row, 2)?.expect("adapter_type"),
        session_id: helpers::row_text(row, 3)?,
        state_json: helpers::row_text(row, 4)?
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or(serde_json::Value::Null),
        last_run_id: helpers::row_text(row, 5)?,
        last_run_status: helpers::row_text(row, 6)?,
        total_input_tokens: helpers::row_i64(row, 7)?,
        total_output_tokens: helpers::row_i64(row, 8)?,
        total_cached_input_tokens: helpers::row_i64(row, 9)?,
        total_cost_cents: helpers::row_i64(row, 10)?,
        last_error: helpers::row_text(row, 11)?,
        created_at: helpers::row_text(row, 12)?.expect("created_at"),
    })
}

fn row_to_wakeup(row: &libsql::Row) -> Result<AgentWakeupRequestRecord, libsql::Error> {
    Ok(AgentWakeupRequestRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        agent_id: helpers::row_text(row, 2)?.expect("agent_id"),
        source: helpers::row_text(row, 3)?.expect("source"),
        trigger_detail: helpers::row_text(row, 4)?,
        reason: helpers::row_text(row, 5)?,
        payload: helpers::row_text(row, 6)?.and_then(|raw| serde_json::from_str(&raw).ok()),
        status: helpers::row_text(row, 7)?.expect("status"),
        coalesced_count: helpers::row_i64(row, 8)?,
        requested_by_actor_type: helpers::row_text(row, 9)?,
        requested_by_actor_id: helpers::row_text(row, 10)?,
        idempotency_key: helpers::row_text(row, 11)?,
        run_id: helpers::row_text(row, 12)?,
        requested_at: helpers::row_text(row, 13)?.expect("requested_at"),
        claimed_at: helpers::row_text(row, 14)?,
        finished_at: helpers::row_text(row, 15)?,
        error: helpers::row_text(row, 16)?,
    })
}

fn row_to_recovery(row: &libsql::Row) -> Result<IssueRecoveryActionRecord, libsql::Error> {
    Ok(IssueRecoveryActionRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        source_issue_id: helpers::row_text(row, 2)?.expect("source_issue_id"),
        recovery_issue_id: helpers::row_text(row, 3)?,
        kind: helpers::row_text(row, 4)?.expect("kind"),
        status: helpers::row_text(row, 5)?.expect("status"),
        owner_type: helpers::row_text(row, 6)?.expect("owner_type"),
        owner_agent_id: helpers::row_text(row, 7)?,
        owner_user_id: helpers::row_text(row, 8)?,
        previous_owner_agent_id: helpers::row_text(row, 9)?,
        return_owner_agent_id: helpers::row_text(row, 10)?,
        cause: helpers::row_text(row, 11)?.expect("cause"),
        fingerprint: helpers::row_text(row, 12)?.expect("fingerprint"),
        evidence: helpers::row_text(row, 13)?
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or(serde_json::Value::Null),
        next_action: helpers::row_text(row, 14)?.expect("next_action"),
        wake_policy: helpers::row_text(row, 15)?.and_then(|raw| serde_json::from_str(&raw).ok()),
        monitor_policy: helpers::row_text(row, 16)?.and_then(|raw| serde_json::from_str(&raw).ok()),
        attempt_count: helpers::row_i64(row, 17)?,
        max_attempts: helpers::row_i64_opt(row, 18)?,
        timeout_at: helpers::row_text(row, 19)?,
        last_attempt_at: helpers::row_text(row, 20)?,
        outcome: helpers::row_text(row, 21)?,
        resolution_note: helpers::row_text(row, 22)?,
        resolved_at: helpers::row_text(row, 23)?,
        created_at: helpers::row_text(row, 24)?.expect("created_at"),
    })
}

const SESSION_COLUMNS: &str = "id, company_id, agent_id, adapter_type, task_key,
    session_params_json, session_display_id, last_run_id, last_error, created_at";
const RUNTIME_COLUMNS: &str = "agent_id, company_id, adapter_type, session_id, state_json,
    last_run_id, last_run_status, total_input_tokens, total_output_tokens,
    total_cached_input_tokens, total_cost_cents, last_error, created_at";
const WAKEUP_COLUMNS: &str = "id, company_id, agent_id, source, trigger_detail, reason, payload,
    status, coalesced_count, requested_by_actor_type, requested_by_actor_id, idempotency_key,
    run_id, requested_at, claimed_at, finished_at, error";
const RECOVERY_COLUMNS: &str = "id, company_id, source_issue_id, recovery_issue_id, kind, status,
    owner_type, owner_agent_id, owner_user_id, previous_owner_agent_id, return_owner_agent_id,
    cause, fingerprint, evidence, next_action, wake_policy, monitor_policy, attempt_count,
    max_attempts, timeout_at, last_attempt_at, outcome, resolution_note, resolved_at, created_at";

async fn ensure_agent(
    conn: &libsql::Connection,
    company_id: &str,
    agent_id: &str,
) -> Result<(), AgentRuntimeError> {
    if !helpers::row_belongs_to_company(conn, "agents", agent_id, company_id).await? {
        return Err(AgentRuntimeError::AgentNotFound);
    }
    Ok(())
}

#[async_trait]
impl AgentRuntimeRepository for TursoAgentRuntimeRepository {
    async fn session_upsert(
        &self,
        input: NewTaskSession,
    ) -> Result<AgentTaskSessionRecord, AgentRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(AgentRuntimeError::CompanyNotFound);
        }
        ensure_agent(&conn, &input.company_id, &input.agent_id).await?;
        if let Some(last_run_id) = &input.last_run_id
            && !helpers::row_belongs_to_company(
                &conn,
                "heartbeat_runs",
                last_run_id,
                &input.company_id,
            )
            .await?
        {
            return Err(AgentRuntimeError::NotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO agent_task_sessions
               (id, company_id, agent_id, adapter_type, task_key, session_params_json,
                session_display_id, last_run_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT (company_id, agent_id, adapter_type, task_key)
             DO UPDATE SET session_params_json = excluded.session_params_json,
                           session_display_id = excluded.session_display_id,
                           last_run_id = excluded.last_run_id,
                           last_error = NULL,
                           updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            libsql::params![
                id.clone(),
                input.company_id.clone(),
                input.agent_id.clone(),
                input.adapter_type.clone(),
                input.task_key.clone(),
                input.session_params_json.map(|v| v.to_string()),
                input.session_display_id,
                input.last_run_id
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {SESSION_COLUMNS} FROM agent_task_sessions
                     WHERE company_id = ?1 AND agent_id = ?2 AND adapter_type = ?3 AND task_key = ?4"
                ),
                libsql::params![input.company_id, input.agent_id, input.adapter_type, input.task_key],
            )
            .await?;
        let row = rows.next().await?.expect("session was just upserted");
        Ok(row_to_session(&row)?)
    }

    async fn session_list(
        &self,
        company_id: &str,
    ) -> Result<Vec<AgentTaskSessionRecord>, AgentRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!("SELECT {SESSION_COLUMNS} FROM agent_task_sessions WHERE company_id = ?1 ORDER BY updated_at DESC"),
                libsql::params![company_id],
            )
            .await?;
        let mut sessions = Vec::new();
        while let Some(row) = rows.next().await? {
            sessions.push(row_to_session(&row)?);
        }
        Ok(sessions)
    }

    async fn session_get(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<AgentTaskSessionRecord>, AgentRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!("SELECT {SESSION_COLUMNS} FROM agent_task_sessions WHERE company_id = ?1 AND id = ?2"),
                libsql::params![company_id, id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_session(&row)?)),
            None => Ok(None),
        }
    }

    async fn runtime_upsert(
        &self,
        input: NewRuntimeState,
    ) -> Result<AgentRuntimeStateRecord, AgentRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(AgentRuntimeError::CompanyNotFound);
        }
        ensure_agent(&conn, &input.company_id, &input.agent_id).await?;
        conn.execute(
            "INSERT INTO agent_runtime_state
               (agent_id, company_id, adapter_type, session_id, state_json, last_run_id,
                last_run_status, total_input_tokens, total_output_tokens,
                total_cached_input_tokens, total_cost_cents, last_error, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT (company_id, agent_id)
             DO UPDATE SET adapter_type = excluded.adapter_type,
                           session_id = excluded.session_id,
                           state_json = excluded.state_json,
                           last_run_id = excluded.last_run_id,
                           last_run_status = excluded.last_run_status,
                           total_input_tokens = excluded.total_input_tokens,
                           total_output_tokens = excluded.total_output_tokens,
                           total_cached_input_tokens = excluded.total_cached_input_tokens,
                           total_cost_cents = excluded.total_cost_cents,
                           last_error = excluded.last_error,
                           updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            libsql::params![
                input.agent_id.clone(),
                input.company_id.clone(),
                input.adapter_type.clone(),
                input.session_id.clone(),
                input.state_json.to_string(),
                input.last_run_id.clone(),
                input.last_run_status.clone(),
                input.total_input_tokens,
                input.total_output_tokens,
                input.total_cached_input_tokens,
                input.total_cost_cents,
                input.last_error.clone()
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {RUNTIME_COLUMNS} FROM agent_runtime_state WHERE company_id = ?1 AND agent_id = ?2"
                ),
                libsql::params![input.company_id, input.agent_id],
            )
            .await?;
        let row = rows.next().await?.expect("runtime state was just upserted");
        Ok(row_to_runtime(&row)?)
    }

    async fn runtime_get(
        &self,
        company_id: &str,
        agent_id: &str,
    ) -> Result<Option<AgentRuntimeStateRecord>, AgentRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {RUNTIME_COLUMNS} FROM agent_runtime_state WHERE company_id = ?1 AND agent_id = ?2"
                ),
                libsql::params![company_id, agent_id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_runtime(&row)?)),
            None => Ok(None),
        }
    }

    async fn wakeup_enqueue(
        &self,
        input: NewWakeupRequest,
    ) -> Result<AgentWakeupRequestRecord, AgentRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(AgentRuntimeError::CompanyNotFound);
        }
        ensure_agent(&conn, &input.company_id, &input.agent_id).await?;
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO agent_wakeup_requests
                   (id, company_id, agent_id, source, trigger_detail, reason, payload, status,
                    requested_by_actor_type, requested_by_actor_id, idempotency_key,
                    requested_at, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'queued', ?8, ?9, ?10,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id.clone(),
                    input.agent_id.clone(),
                    input.source.clone(),
                    input.trigger_detail.clone(),
                    input.reason.clone(),
                    input.payload.map(|v| v.to_string()),
                    input.requested_by_actor_type.clone(),
                    input.requested_by_actor_id.clone(),
                    input.idempotency_key.clone()
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!(
                            "SELECT {WAKEUP_COLUMNS} FROM agent_wakeup_requests WHERE id = ?1"
                        ),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("wakeup was just inserted");
                Ok(row_to_wakeup(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                // Duplicate idempotency key: coalesce onto the existing queued row.
                conn.execute(
                    "UPDATE agent_wakeup_requests SET coalesced_count = coalesced_count + 1,
                            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                     WHERE company_id = ?1 AND agent_id = ?2 AND idempotency_key = ?3
                       AND status = 'queued'",
                    libsql::params![
                        input.company_id.clone(),
                        input.agent_id.clone(),
                        input.idempotency_key.clone()
                    ],
                )
                .await?;
                let mut rows = conn
                    .query(
                        &format!(
                            "SELECT {WAKEUP_COLUMNS} FROM agent_wakeup_requests
                             WHERE company_id = ?1 AND agent_id = ?2 AND idempotency_key = ?3"
                        ),
                        libsql::params![input.company_id, input.agent_id, input.idempotency_key],
                    )
                    .await?;
                let row = rows.next().await?.expect("existing wakeup");
                Ok(row_to_wakeup(&row)?)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn wakeup_claim(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<AgentWakeupRequestRecord>, AgentRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let updated = conn
            .execute(
                "UPDATE agent_wakeup_requests SET status = 'claimed',
                        claimed_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                        updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE company_id = ?1 AND id = ?2 AND status = 'queued'",
                libsql::params![company_id, id],
            )
            .await?;
        if updated == 0 {
            return Ok(None);
        }
        let mut rows = conn
            .query(
                &format!("SELECT {WAKEUP_COLUMNS} FROM agent_wakeup_requests WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("wakeup exists");
        Ok(Some(row_to_wakeup(&row)?))
    }

    async fn wakeup_finish(
        &self,
        company_id: &str,
        id: &str,
        status: &str,
        error: Option<String>,
        run_id: Option<String>,
    ) -> Result<Option<AgentWakeupRequestRecord>, AgentRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let updated = conn
            .execute(
                "UPDATE agent_wakeup_requests SET status = ?1, error = ?2, run_id = COALESCE(?3, run_id),
                        finished_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                        updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE company_id = ?4 AND id = ?5 AND status = 'claimed'",
                libsql::params![status, error, run_id, company_id, id],
            )
            .await?;
        if updated == 0 {
            return Ok(None);
        }
        let mut rows = conn
            .query(
                &format!("SELECT {WAKEUP_COLUMNS} FROM agent_wakeup_requests WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("wakeup exists");
        Ok(Some(row_to_wakeup(&row)?))
    }

    async fn wakeup_list(
        &self,
        company_id: &str,
    ) -> Result<Vec<AgentWakeupRequestRecord>, AgentRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {WAKEUP_COLUMNS} FROM agent_wakeup_requests
                     WHERE company_id = ?1 ORDER BY requested_at DESC"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut requests = Vec::new();
        while let Some(row) = rows.next().await? {
            requests.push(row_to_wakeup(&row)?);
        }
        Ok(requests)
    }

    async fn recovery_create(
        &self,
        input: NewRecoveryAction,
    ) -> Result<IssueRecoveryActionRecord, AgentRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(AgentRuntimeError::CompanyNotFound);
        }
        if !helpers::row_belongs_to_company(
            &conn,
            "issues",
            &input.source_issue_id,
            &input.company_id,
        )
        .await?
        {
            return Err(AgentRuntimeError::SourceIssueNotFound);
        }
        if let Some(recovery_issue_id) = &input.recovery_issue_id
            && !helpers::row_belongs_to_company(
                &conn,
                "issues",
                recovery_issue_id,
                &input.company_id,
            )
            .await?
        {
            return Err(AgentRuntimeError::NotFound);
        }
        if let Some(owner_agent_id) = &input.owner_agent_id
            && !helpers::row_belongs_to_company(&conn, "agents", owner_agent_id, &input.company_id)
                .await?
        {
            return Err(AgentRuntimeError::AgentNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO issue_recovery_actions
                   (id, company_id, source_issue_id, recovery_issue_id, kind, status,
                    owner_agent_id, cause, fingerprint, evidence, next_action, wake_policy,
                    monitor_policy, max_attempts, timeout_at, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'active', ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id.clone(),
                    input.source_issue_id.clone(),
                    input.recovery_issue_id.clone(),
                    input.kind.clone(),
                    input.owner_agent_id.clone(),
                    input.cause.clone(),
                    input.fingerprint.clone(),
                    input.evidence.to_string(),
                    input.next_action.clone(),
                    input.wake_policy.map(|v| v.to_string()),
                    input.monitor_policy.map(|v| v.to_string()),
                    input.max_attempts,
                    input.timeout_at.clone()
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!(
                            "SELECT {RECOVERY_COLUMNS} FROM issue_recovery_actions WHERE id = ?1"
                        ),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows
                    .next()
                    .await?
                    .expect("recovery action was just inserted");
                Ok(row_to_recovery(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(AgentRuntimeError::NotFound)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn recovery_list(
        &self,
        company_id: &str,
    ) -> Result<Vec<IssueRecoveryActionRecord>, AgentRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {RECOVERY_COLUMNS} FROM issue_recovery_actions
                     WHERE company_id = ?1 ORDER BY created_at DESC"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut actions = Vec::new();
        while let Some(row) = rows.next().await? {
            actions.push(row_to_recovery(&row)?);
        }
        Ok(actions)
    }

    async fn recovery_get(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<IssueRecoveryActionRecord>, AgentRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {RECOVERY_COLUMNS} FROM issue_recovery_actions WHERE company_id = ?1 AND id = ?2"
                ),
                libsql::params![company_id, id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_recovery(&row)?)),
            None => Ok(None),
        }
    }

    async fn recovery_set_status(
        &self,
        company_id: &str,
        id: &str,
        status: &str,
        outcome: Option<String>,
        resolution_note: Option<String>,
    ) -> Result<Option<IssueRecoveryActionRecord>, AgentRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let updated = conn
            .execute(
                "UPDATE issue_recovery_actions SET status = ?1, outcome = COALESCE(?2, outcome),
                        resolution_note = COALESCE(?3, resolution_note),
                        resolved_at = CASE WHEN ?1 IN ('resolved', 'cancelled')
                                          THEN strftime('%Y-%m-%dT%H:%M:%fZ','now')
                                          ELSE resolved_at END,
                        updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE company_id = ?4 AND id = ?5 AND status IN ('active', 'escalated')",
                libsql::params![status, outcome, resolution_note, company_id, id],
            )
            .await?;
        if updated == 0 {
            let mut rows = conn
                .query(
                    &format!(
                        "SELECT {RECOVERY_COLUMNS} FROM issue_recovery_actions WHERE company_id = ?1 AND id = ?2"
                    ),
                    libsql::params![company_id, id],
                )
                .await?;
            return match rows.next().await? {
                Some(_) => Ok(None),
                None => Err(AgentRuntimeError::NotFound),
            };
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {RECOVERY_COLUMNS} FROM issue_recovery_actions WHERE company_id = ?1 AND id = ?2"
                ),
                libsql::params![company_id, id],
            )
            .await?;
        let row = rows.next().await?.expect("recovery action exists");
        Ok(Some(row_to_recovery(&row)?))
    }

    async fn recovery_bump_attempt(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<IssueRecoveryActionRecord>, AgentRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let updated = conn
            .execute(
                "UPDATE issue_recovery_actions
                 SET attempt_count = attempt_count + 1,
                     last_attempt_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE company_id = ?1 AND id = ?2 AND status IN ('active', 'escalated')",
                libsql::params![company_id, id],
            )
            .await?;
        if updated == 0 {
            return Ok(None);
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {RECOVERY_COLUMNS} FROM issue_recovery_actions WHERE company_id = ?1 AND id = ?2"
                ),
                libsql::params![company_id, id],
            )
            .await?;
        let row = rows.next().await?.expect("recovery action exists");
        Ok(Some(row_to_recovery(&row)?))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoAgentRuntimeRepository) {
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
             VALUES ('a1', 'c1', 'One', 'worker', 'cli')",
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
        (dir, TursoAgentRuntimeRepository::new(db))
    }

    #[tokio::test]
    async fn sessions_and_runtime_state() {
        let (_dir, repo) = repo().await;
        let session = repo
            .session_upsert(NewTaskSession {
                company_id: "c1".to_owned(),
                agent_id: "a1".to_owned(),
                adapter_type: "cli".to_owned(),
                task_key: "task-1".to_owned(),
                session_params_json: Some(serde_json::json!({ "branch": "main" })),
                session_display_id: Some("sess-1".to_owned()),
                last_run_id: None,
            })
            .await
            .unwrap();
        assert_eq!(session.task_key, "task-1");
        let again = repo
            .session_upsert(NewTaskSession {
                company_id: "c1".to_owned(),
                agent_id: "a1".to_owned(),
                adapter_type: "cli".to_owned(),
                task_key: "task-1".to_owned(),
                session_params_json: None,
                session_display_id: Some("sess-2".to_owned()),
                last_run_id: None,
            })
            .await
            .unwrap();
        assert_eq!(again.id, session.id);
        assert_eq!(again.session_display_id.as_deref(), Some("sess-2"));
        assert!(repo.session_list("c1").await.unwrap().len() == 1);
        assert!(repo.session_get("c1", &session.id).await.unwrap().is_some());

        let runtime = repo
            .runtime_upsert(NewRuntimeState {
                company_id: "c1".to_owned(),
                agent_id: "a1".to_owned(),
                adapter_type: "cli".to_owned(),
                session_id: Some("sess-2".to_owned()),
                state_json: serde_json::json!({ "step": 3 }),
                last_run_id: None,
                last_run_status: Some("running".to_owned()),
                total_input_tokens: 10,
                total_output_tokens: 5,
                total_cached_input_tokens: 2,
                total_cost_cents: 7,
                last_error: None,
            })
            .await
            .unwrap();
        assert_eq!(runtime.total_input_tokens, 10);
        assert!(repo.runtime_get("c1", "a1").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn wakeup_enqueue_claim_finish_coalesce() {
        let (_dir, repo) = repo().await;
        let first = repo
            .wakeup_enqueue(NewWakeupRequest {
                company_id: "c1".to_owned(),
                agent_id: "a1".to_owned(),
                source: "scheduler".to_owned(),
                trigger_detail: Some("issue i1".to_owned()),
                reason: Some("timeout".to_owned()),
                payload: None,
                requested_by_actor_type: Some("board".to_owned()),
                requested_by_actor_id: None,
                idempotency_key: Some("wake-1".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(first.status, "queued");
        // Duplicate idempotency key coalesces.
        let second = repo
            .wakeup_enqueue(NewWakeupRequest {
                company_id: "c1".to_owned(),
                agent_id: "a1".to_owned(),
                source: "scheduler".to_owned(),
                trigger_detail: None,
                reason: None,
                payload: None,
                requested_by_actor_type: None,
                requested_by_actor_id: None,
                idempotency_key: Some("wake-1".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(second.id, first.id);
        assert_eq!(second.coalesced_count, 1);

        let claimed = repo.wakeup_claim("c1", &first.id).await.unwrap().unwrap();
        assert_eq!(claimed.status, "claimed");
        assert!(repo.wakeup_claim("c1", &first.id).await.unwrap().is_none());
        let finished = repo
            .wakeup_finish("c1", &first.id, "finished", None, Some("run-1".to_owned()))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(finished.status, "finished");
        assert!(repo.wakeup_list("c1").await.unwrap().len() == 1);
    }

    #[tokio::test]
    async fn recovery_state_machine() {
        let (_dir, repo) = repo().await;
        let action = repo
            .recovery_create(NewRecoveryAction {
                company_id: "c1".to_owned(),
                source_issue_id: "i1".to_owned(),
                recovery_issue_id: None,
                kind: "restore".to_owned(),
                owner_agent_id: Some("a1".to_owned()),
                cause: "lost_process".to_owned(),
                fingerprint: "fp-1".to_owned(),
                evidence: serde_json::json!({ "run": "r1" }),
                next_action: "resume".to_owned(),
                wake_policy: None,
                monitor_policy: None,
                max_attempts: Some(3),
                timeout_at: None,
            })
            .await
            .unwrap();
        assert_eq!(action.status, "active");

        // Bump attempts.
        let bumped = repo
            .recovery_bump_attempt("c1", &action.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(bumped.attempt_count, 1);

        // Escalate then restore (re-activate).
        let escalated = repo
            .recovery_set_status(
                "c1",
                &action.id,
                "escalated",
                None,
                Some("need board".to_owned()),
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(escalated.status, "escalated");
        // Active->escalated transition allows restore back to active.
        let restored = repo
            .recovery_set_status("c1", &action.id, "active", None, None)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(restored.status, "active");
        assert!(restored.resolved_at.is_none());

        // Resolve with outcome.
        let resolved = repo
            .recovery_set_status(
                "c1",
                &action.id,
                "resolved",
                Some("restored".to_owned()),
                Some("done".to_owned()),
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(resolved.status, "resolved");
        assert_eq!(resolved.outcome.as_deref(), Some("restored"));
        assert!(resolved.resolved_at.is_some());

        // Terminal action cannot transition again.
        assert!(
            repo.recovery_set_status("c1", &action.id, "cancelled", None, None)
                .await
                .unwrap()
                .is_none()
        );
        // Bumping a resolved action is not found.
        assert!(
            repo.recovery_bump_attempt("c1", &action.id)
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn recovery_duplicate_active_fingerprint_rejected() {
        let (_dir, repo) = repo().await;
        let base = NewRecoveryAction {
            company_id: "c1".to_owned(),
            source_issue_id: "i1".to_owned(),
            recovery_issue_id: None,
            kind: "restore".to_owned(),
            owner_agent_id: None,
            cause: "lost_process".to_owned(),
            fingerprint: "fp-dup".to_owned(),
            evidence: serde_json::json!({}),
            next_action: "resume".to_owned(),
            wake_policy: None,
            monitor_policy: None,
            max_attempts: None,
            timeout_at: None,
        };
        repo.recovery_create(base.clone()).await.unwrap();
        // Duplicate active fingerprint -> unique index violation.
        assert!(matches!(
            repo.recovery_create(base).await.unwrap_err(),
            AgentRuntimeError::NotFound
        ));
    }
}
