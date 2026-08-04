//! Heartbeat runs repository: the execution control plane.
//!
//! Implements atomic issue checkout (the `execution_run_id` lock), run
//! lifecycle (start/complete/cancel), failure attribution, and the task
//! watchdog authority contract (§9.9).

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `heartbeat_runs` table.
#[derive(Debug, Clone)]
pub struct HeartbeatRunRecord {
    /// Run id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Agent id.
    pub agent_id: String,
    /// `scheduler | manual | callback`.
    pub invocation_source: String,
    /// `queued | running | succeeded | failed | cancelled | timed_out`.
    pub status: String,
    /// ISO 8601 start time.
    pub started_at: Option<String>,
    /// ISO 8601 finish time.
    pub finished_at: Option<String>,
    /// Error message.
    pub error: Option<String>,
    /// Failure attribution: `infrastructure | agent | null`.
    pub error_kind: Option<String>,
    /// External run id.
    pub external_run_id: Option<String>,
    /// Context snapshot JSON (may carry `issueId`, watchdog metadata).
    pub context_snapshot: Option<String>,
    /// Trigger detail.
    pub trigger_detail: Option<String>,
    /// Log bytes.
    pub log_bytes: i64,
    /// ISO 8601 creation time.
    pub created_at: String,
    /// ISO 8601 last update time.
    pub updated_at: String,
    /// Responsible user id.
    pub responsible_user_id: Option<String>,
    /// Wakeup request id.
    pub wakeup_request_id: Option<String>,
    /// Process exit code.
    pub exit_code: Option<i64>,
    /// Termination signal.
    pub signal: Option<String>,
    /// Usage JSON.
    pub usage_json: Option<String>,
    /// Result JSON.
    pub result_json: Option<String>,
    /// Session id before the run.
    pub session_id_before: Option<String>,
    /// Session id after the run.
    pub session_id_after: Option<String>,
    /// Log store.
    pub log_store: Option<String>,
    /// Log reference.
    pub log_ref: Option<String>,
    /// Log SHA-256.
    pub log_sha256: Option<String>,
    /// Whether logs are compressed.
    pub log_compressed: i64,
    /// Stdout excerpt.
    pub stdout_excerpt: Option<String>,
    /// Stderr excerpt.
    pub stderr_excerpt: Option<String>,
    /// Error code.
    pub error_code: Option<String>,
    /// Process pid.
    pub process_pid: Option<i64>,
    /// Process group id.
    pub process_group_id: Option<i64>,
    /// ISO 8601 process start.
    pub process_started_at: Option<String>,
    /// ISO 8601 last output.
    pub last_output_at: Option<String>,
    /// Last output sequence.
    pub last_output_seq: Option<i64>,
    /// Last output stream.
    pub last_output_stream: Option<String>,
    /// Last output bytes.
    pub last_output_bytes: Option<i64>,
    /// Retry-of run id.
    pub retry_of_run_id: Option<String>,
    /// Process loss retry count.
    pub process_loss_retry_count: Option<i64>,
    /// ISO 8601 scheduled retry.
    pub scheduled_retry_at: Option<String>,
    /// Scheduled retry attempt.
    pub scheduled_retry_attempt: Option<i64>,
    /// Scheduled retry reason.
    pub scheduled_retry_reason: Option<String>,
    /// Issue comment status.
    pub issue_comment_status: Option<String>,
    /// Satisfying comment id.
    pub issue_comment_satisfied_by_comment_id: Option<String>,
    /// ISO 8601 comment retry queued.
    pub issue_comment_retry_queued_at: Option<String>,
    /// Liveness state.
    pub liveness_state: Option<String>,
    /// Liveness reason.
    pub liveness_reason: Option<String>,
    /// Continuation attempt.
    pub continuation_attempt: i64,
    /// ISO 8601 last useful action.
    pub last_useful_action_at: Option<String>,
    /// Next action.
    pub next_action: Option<String>,
}

/// Input for starting a heartbeat run.
#[derive(Debug, Clone)]
pub struct NewHeartbeatRun {
    /// Owning company id.
    pub company_id: String,
    /// Agent id (must belong to the company).
    pub agent_id: String,
    /// `scheduler | manual | callback`.
    pub invocation_source: String,
    /// Issue to check out for execution, if any.
    pub issue_id: Option<String>,
    /// Context snapshot JSON.
    pub context_snapshot: Option<String>,
    /// Trigger detail.
    pub trigger_detail: Option<String>,
}

/// Input for completing a run.
#[derive(Debug, Clone)]
pub struct CompleteHeartbeatRun {
    /// Terminal status (`succeeded | failed | cancelled | timed_out`).
    pub status: String,
    /// Error message.
    pub error: Option<String>,
    /// Failure attribution (`infrastructure | agent`).
    pub error_kind: Option<String>,
}

/// Heartbeat repository errors.
#[derive(Debug, Error)]
pub enum HeartbeatError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The agent does not exist or belongs to another company.
    #[error("agent not found")]
    AgentNotFound,
    /// The issue does not exist or belongs to another company.
    #[error("issue not found")]
    IssueNotFound,
    /// The issue is already checked out by another run (execution lock held).
    #[error("issue is already checked out by another run")]
    IssueExecutionLocked,
    /// The run is not in a state that allows the operation.
    #[error("run is not in a running state")]
    RunNotRunning,
    /// The watchdog is not authorized for the target issue.
    #[error("watchdog is not authorized for this issue")]
    WatchdogNotAuthorized,
}

/// Heartbeat persistence contract.
#[async_trait]
pub trait HeartbeatRepository: Send + Sync {
    /// Starts a run. When `issue_id` is given, atomically checks the issue
    /// out (sets the execution lock); concurrent starts on the same issue are
    /// rejected.
    ///
    /// # Errors
    ///
    /// Returns [`HeartbeatError`] on invalid references or when the execution
    /// lock is held by another run.
    async fn start(&self, input: NewHeartbeatRun) -> Result<HeartbeatRunRecord, HeartbeatError>;

    /// Completes a run, releasing the issue execution lock.
    ///
    /// # Errors
    ///
    /// Returns [`HeartbeatError`] when the run is missing or not running.
    async fn complete(
        &self,
        run_id: &str,
        input: CompleteHeartbeatRun,
    ) -> Result<Option<HeartbeatRunRecord>, HeartbeatError>;

    /// Cancels a run, releasing the issue execution lock.
    ///
    /// # Errors
    ///
    /// Returns [`HeartbeatError`] on database failure.
    async fn cancel(&self, run_id: &str) -> Result<Option<HeartbeatRunRecord>, HeartbeatError>;

    /// Fetches one run by id.
    ///
    /// # Errors
    ///
    /// Returns [`HeartbeatError`] on database failure.
    async fn get(&self, run_id: &str) -> Result<Option<HeartbeatRunRecord>, HeartbeatError>;

    /// Lists runs, optionally filtered by agent.
    ///
    /// # Errors
    ///
    /// Returns [`HeartbeatError`] on database failure.
    async fn list(
        &self,
        company_id: &str,
        agent_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<HeartbeatRunRecord>, HeartbeatError>;

    /// Whether a watchdog run is authorized to act on `target_issue_id`
    /// (inside the watched subtree, excluding `task_watchdog` origin
    /// branches). Non-watchdog runs are always authorized for their own
    /// checked-out issue.
    ///
    /// # Errors
    ///
    /// Returns [`HeartbeatError`] on database failure.
    async fn watchdog_authorized(
        &self,
        run_id: &str,
        target_issue_id: &str,
    ) -> Result<bool, HeartbeatError>;
}

/// Turso/libSQL implementation of [`HeartbeatRepository`].
#[derive(Debug)]
pub struct TursoHeartbeatRepository {
    db: Database,
}

impl TursoHeartbeatRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

const RUN_COLUMNS: &str = "id, company_id, agent_id, invocation_source, status, started_at,
    finished_at, error, error_kind, external_run_id, context_snapshot, trigger_detail,
    log_bytes, created_at, updated_at, responsible_user_id, wakeup_request_id, exit_code,
    signal, usage_json, result_json, session_id_before, session_id_after, log_store, log_ref,
    log_sha256, log_compressed, stdout_excerpt, stderr_excerpt, error_code, process_pid,
    process_group_id, process_started_at, last_output_at, last_output_seq, last_output_stream,
    last_output_bytes, retry_of_run_id, process_loss_retry_count, scheduled_retry_at,
    scheduled_retry_attempt, scheduled_retry_reason, issue_comment_status,
    issue_comment_satisfied_by_comment_id, issue_comment_retry_queued_at, liveness_state,
    liveness_reason, continuation_attempt, last_useful_action_at, next_action";

fn row_to_run(row: &libsql::Row) -> Result<HeartbeatRunRecord, libsql::Error> {
    Ok(HeartbeatRunRecord {
        id: helpers::row_text(row, 0)?.expect("id is NOT NULL"),
        company_id: helpers::row_text(row, 1)?.expect("company_id is NOT NULL"),
        agent_id: helpers::row_text(row, 2)?.expect("agent_id is NOT NULL"),
        invocation_source: helpers::row_text(row, 3)?.expect("invocation_source is NOT NULL"),
        status: helpers::row_text(row, 4)?.expect("status is NOT NULL"),
        started_at: helpers::row_text(row, 5)?,
        finished_at: helpers::row_text(row, 6)?,
        error: helpers::row_text(row, 7)?,
        error_kind: helpers::row_text(row, 8)?,
        external_run_id: helpers::row_text(row, 9)?,
        context_snapshot: helpers::row_text(row, 10)?,
        trigger_detail: helpers::row_text(row, 11)?,
        log_bytes: helpers::row_i64(row, 12)?,
        created_at: helpers::row_text(row, 13)?.expect("created_at is NOT NULL"),
        updated_at: helpers::row_text(row, 14)?.expect("updated_at is NOT NULL"),
        responsible_user_id: helpers::row_text(row, 15)?,
        wakeup_request_id: helpers::row_text(row, 16)?,
        exit_code: helpers::row_i64_opt(row, 17)?,
        signal: helpers::row_text(row, 18)?,
        usage_json: helpers::row_text(row, 19)?,
        result_json: helpers::row_text(row, 20)?,
        session_id_before: helpers::row_text(row, 21)?,
        session_id_after: helpers::row_text(row, 22)?,
        log_store: helpers::row_text(row, 23)?,
        log_ref: helpers::row_text(row, 24)?,
        log_sha256: helpers::row_text(row, 25)?,
        log_compressed: helpers::row_i64(row, 26)?,
        stdout_excerpt: helpers::row_text(row, 27)?,
        stderr_excerpt: helpers::row_text(row, 28)?,
        error_code: helpers::row_text(row, 29)?,
        process_pid: helpers::row_i64_opt(row, 30)?,
        process_group_id: helpers::row_i64_opt(row, 31)?,
        process_started_at: helpers::row_text(row, 32)?,
        last_output_at: helpers::row_text(row, 33)?,
        last_output_seq: helpers::row_i64_opt(row, 34)?,
        last_output_stream: helpers::row_text(row, 35)?,
        last_output_bytes: helpers::row_i64_opt(row, 36)?,
        retry_of_run_id: helpers::row_text(row, 37)?,
        process_loss_retry_count: helpers::row_i64_opt(row, 38)?,
        scheduled_retry_at: helpers::row_text(row, 39)?,
        scheduled_retry_attempt: helpers::row_i64_opt(row, 40)?,
        scheduled_retry_reason: helpers::row_text(row, 41)?,
        issue_comment_status: helpers::row_text(row, 42)?,
        issue_comment_satisfied_by_comment_id: helpers::row_text(row, 43)?,
        issue_comment_retry_queued_at: helpers::row_text(row, 44)?,
        liveness_state: helpers::row_text(row, 45)?,
        liveness_reason: helpers::row_text(row, 46)?,
        continuation_attempt: helpers::row_i64(row, 47)?,
        last_useful_action_at: helpers::row_text(row, 48)?,
        next_action: helpers::row_text(row, 49)?,
    })
}

/// Reads a run's status on a given connection.
async fn run_status(
    conn: &libsql::Connection,
    run_id: &str,
) -> Result<Option<String>, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT status FROM heartbeat_runs WHERE id = ?1",
            libsql::params![run_id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(helpers::row_text(&row, 0)?),
        None => Ok(None),
    }
}

/// Whether the run context marks it as a task watchdog.
fn is_watchdog(context_snapshot: &Option<String>) -> bool {
    let Some(raw) = context_snapshot.as_deref() else {
        return false;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) else {
        return false;
    };
    value.get("kind").and_then(|v| v.as_str()) == Some("task_watchdog")
}

#[async_trait]
impl HeartbeatRepository for TursoHeartbeatRepository {
    async fn start(&self, input: NewHeartbeatRun) -> Result<HeartbeatRunRecord, HeartbeatError> {
        let conn = crate::connection::connect(&self.db).await?;
        let tx = conn.transaction().await?;

        if !helpers::row_belongs_to_company(&tx, "agents", &input.agent_id, &input.company_id)
            .await?
        {
            return Err(HeartbeatError::AgentNotFound);
        }

        let mut checked_out_issue: Option<String> = None;
        if let Some(issue_id) = &input.issue_id {
            if !helpers::row_belongs_to_company(&tx, "issues", issue_id, &input.company_id).await? {
                return Err(HeartbeatError::IssueNotFound);
            }
            // Atomic checkout: only succeeds when the execution lock is free
            // or already held by this same run id (idempotent retry).
            let run_id_placeholder = Uuid::new_v4().to_string();
            let updated = tx
                .execute(
                    "UPDATE issues
                     SET checkout_run_id = ?1, execution_run_id = ?1,
                         execution_locked_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                     WHERE id = ?2
                       AND (execution_run_id IS NULL OR execution_run_id = ?1)",
                    libsql::params![run_id_placeholder.clone(), issue_id.clone()],
                )
                .await?;
            if updated == 0 {
                return Err(HeartbeatError::IssueExecutionLocked);
            }
            checked_out_issue = Some(run_id_placeholder);
        }

        let run_id = checked_out_issue.unwrap_or_else(|| Uuid::new_v4().to_string());
        tx.execute(
            "INSERT INTO heartbeat_runs (id, company_id, agent_id, invocation_source, status,
                                         started_at, error_kind, context_snapshot, trigger_detail,
                                         created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'running',
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'), NULL, ?5, ?6,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                run_id.clone(),
                input.company_id,
                input.agent_id,
                input.invocation_source,
                input.context_snapshot,
                input.trigger_detail
            ],
        )
        .await?;
        tx.commit().await?;
        Ok(self.get(&run_id).await?.expect("run was just inserted"))
    }

    async fn complete(
        &self,
        run_id: &str,
        input: CompleteHeartbeatRun,
    ) -> Result<Option<HeartbeatRunRecord>, HeartbeatError> {
        let conn = crate::connection::connect(&self.db).await?;
        let tx = conn.transaction().await?;
        let run_status = run_status(&tx, run_id).await?;
        let Some(run_status) = run_status else {
            return Ok(None);
        };
        if run_status != "running" {
            return Err(HeartbeatError::RunNotRunning);
        }

        // Release the execution lock if this run holds it.
        tx.execute(
            "UPDATE issues
             SET checkout_run_id = NULL, execution_run_id = NULL, execution_locked_at = NULL,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE execution_run_id = ?1",
            libsql::params![run_id],
        )
        .await?;
        tx.execute(
            "UPDATE heartbeat_runs
             SET status = ?1, finished_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                 error = ?2, error_kind = ?3,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?4",
            libsql::params![input.status, input.error, input.error_kind, run_id],
        )
        .await?;
        tx.commit().await?;
        self.get(run_id).await
    }

    async fn cancel(&self, run_id: &str) -> Result<Option<HeartbeatRunRecord>, HeartbeatError> {
        let conn = crate::connection::connect(&self.db).await?;
        let tx = conn.transaction().await?;
        let run_status = run_status(&tx, run_id).await?;
        let Some(run_status) = run_status else {
            return Ok(None);
        };
        if run_status != "running" {
            return Err(HeartbeatError::RunNotRunning);
        }
        tx.execute(
            "UPDATE issues
             SET checkout_run_id = NULL, execution_run_id = NULL, execution_locked_at = NULL,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE execution_run_id = ?1",
            libsql::params![run_id],
        )
        .await?;
        tx.execute(
            "UPDATE heartbeat_runs
             SET status = 'cancelled', finished_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?1",
            libsql::params![run_id],
        )
        .await?;
        tx.commit().await?;
        self.get(run_id).await
    }

    async fn get(&self, run_id: &str) -> Result<Option<HeartbeatRunRecord>, HeartbeatError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql = format!("SELECT {RUN_COLUMNS} FROM heartbeat_runs WHERE id = ?1");
        let mut rows = conn.query(&sql, libsql::params![run_id]).await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_run(&row)?)),
            None => Ok(None),
        }
    }

    async fn list(
        &self,
        company_id: &str,
        agent_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<HeartbeatRunRecord>, HeartbeatError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut sql = format!("SELECT {RUN_COLUMNS} FROM heartbeat_runs WHERE company_id = ?1");
        let mut params: Vec<libsql::Value> = vec![company_id.into()];
        if let Some(agent_id) = agent_id {
            sql.push_str(" AND agent_id = ?2");
            params.push(agent_id.into());
        }
        let limit_param = params.len() + 1;
        sql.push_str(&format!(" ORDER BY created_at DESC LIMIT ?{limit_param}"));
        params.push(limit.into());
        let mut rows = conn.query(&sql, params).await?;
        let mut runs = Vec::new();
        while let Some(row) = rows.next().await? {
            runs.push(row_to_run(&row)?);
        }
        Ok(runs)
    }

    async fn watchdog_authorized(
        &self,
        run_id: &str,
        target_issue_id: &str,
    ) -> Result<bool, HeartbeatError> {
        let conn = crate::connection::connect(&self.db).await?;
        let run = self.get(run_id).await?;
        let Some(run) = run else {
            return Ok(false);
        };

        if !is_watchdog(&run.context_snapshot) {
            // Regular runs may only act on the issue they checked out.
            let mut rows = conn
                .query(
                    "SELECT 1 FROM issues WHERE id = ?1 AND execution_run_id = ?2",
                    libsql::params![target_issue_id, run_id],
                )
                .await?;
            return Ok(rows.next().await?.is_some());
        }

        let Some(watched_issue_id) = run
            .context_snapshot
            .as_deref()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
            .and_then(|value| {
                value
                    .get("watchedIssueId")
                    .and_then(|value| value.as_str())
                    .map(str::to_owned)
            })
        else {
            return Ok(false);
        };

        // The target must be inside the watched subtree (the watched issue
        // plus descendants), excluding `task_watchdog` origin branches.
        let sql = r#"
            WITH RECURSIVE subtree(id, origin_kind) AS (
                SELECT id, origin_kind FROM issues WHERE id = ?1
                UNION ALL
                SELECT i.id, i.origin_kind
                FROM issues i
                JOIN subtree s ON i.parent_id = s.id
                WHERE s.origin_kind IS NULL OR s.origin_kind != 'task_watchdog'
            )
            SELECT 1 FROM subtree WHERE id = ?2 AND (origin_kind IS NULL OR origin_kind != 'task_watchdog')
        "#;
        let mut rows = conn
            .query(sql, libsql::params![watched_issue_id, target_issue_id])
            .await?;
        Ok(rows.next().await?.is_some())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{Connection, migrate, open};

    async fn repo() -> (TempDir, TursoHeartbeatRepository, Connection) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let conn = crate::connect(&db).await.unwrap();
        let repo = TursoHeartbeatRepository::new(db);
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
            "INSERT INTO agents (id, company_id, name, role, adapter_type)
             VALUES ('a1', 'c1', 'one', 'engineer', 'codex_local'),
                    ('a2', 'c1', 'two', 'engineer', 'codex_local')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO issues (id, company_id, title, issue_number, identifier, status)
             VALUES ('i1', 'c1', 'root', 1, 'ALPHA-1', 'in_progress'),
                    ('i2', 'c1', 'child', 2, 'ALPHA-2', 'todo'),
                    ('i3', 'c1', 'watchdog-branch', 3, 'ALPHA-3', 'todo'),
                    ('i4', 'c1', 'other', 4, 'ALPHA-4', 'todo')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "UPDATE issues SET parent_id = 'i1' WHERE id IN ('i2', 'i3')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "UPDATE issues SET origin_kind = 'task_watchdog' WHERE id = 'i3'",
            (),
        )
        .await
        .unwrap();
    }

    fn new_run(issue_id: Option<&str>) -> NewHeartbeatRun {
        NewHeartbeatRun {
            company_id: "c1".to_owned(),
            agent_id: "a1".to_owned(),
            invocation_source: "manual".to_owned(),
            issue_id: issue_id.map(str::to_owned),
            context_snapshot: None,
            trigger_detail: None,
        }
    }

    #[tokio::test]
    async fn start_complete_releases_lock() {
        let (_dir, repo, conn) = repo().await;
        seed(&conn).await;

        let run = repo.start(new_run(Some("i1"))).await.unwrap();
        assert_eq!(run.status, "running");

        // Lock is held (scope the read so its statement releases the lock).
        {
            let mut rows = conn
                .query("SELECT execution_run_id FROM issues WHERE id = 'i1'", ())
                .await
                .unwrap();
            let row = rows.next().await.unwrap().unwrap();
            assert_eq!(
                helpers::row_text(&row, 0).unwrap().as_deref(),
                Some(run.id.as_str())
            );
        }

        // Complete releases the lock.
        repo.complete(
            &run.id,
            CompleteHeartbeatRun {
                status: "succeeded".to_owned(),
                error: None,
                error_kind: None,
            },
        )
        .await
        .unwrap();
        {
            let mut rows = conn
                .query("SELECT execution_run_id FROM issues WHERE id = 'i1'", ())
                .await
                .unwrap();
            let row = rows.next().await.unwrap().unwrap();
            assert_eq!(helpers::row_text(&row, 0).unwrap(), None);
        }
    }

    #[tokio::test]
    async fn concurrent_starts_are_mutually_exclusive() {
        let (_dir, repo, conn) = repo().await;
        seed(&conn).await;
        repo.start(new_run(Some("i1"))).await.unwrap();

        let error = repo.start(new_run(Some("i1"))).await.unwrap_err();
        assert!(matches!(error, HeartbeatError::IssueExecutionLocked));

        // A different issue is still startable.
        repo.start(new_run(Some("i2"))).await.unwrap();
    }

    #[tokio::test]
    async fn cancel_releases_lock_and_marks_cancelled() {
        let (_dir, repo, conn) = repo().await;
        seed(&conn).await;
        let run = repo.start(new_run(Some("i1"))).await.unwrap();
        let cancelled = repo.cancel(&run.id).await.unwrap().unwrap();
        assert_eq!(cancelled.status, "cancelled");

        // Lock released; another run can start.
        repo.start(new_run(Some("i1"))).await.unwrap();
    }

    #[tokio::test]
    async fn complete_non_running_run_is_rejected() {
        let (_dir, repo, conn) = repo().await;
        seed(&conn).await;
        let run = repo.start(new_run(Some("i1"))).await.unwrap();
        repo.complete(
            &run.id,
            CompleteHeartbeatRun {
                status: "succeeded".to_owned(),
                error: None,
                error_kind: None,
            },
        )
        .await
        .unwrap();
        let error = repo
            .complete(
                &run.id,
                CompleteHeartbeatRun {
                    status: "failed".to_owned(),
                    error: Some("late".to_owned()),
                    error_kind: None,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(error, HeartbeatError::RunNotRunning));
    }

    #[tokio::test]
    async fn failure_attribution_is_persisted() {
        let (_dir, repo, conn) = repo().await;
        seed(&conn).await;
        let run = repo.start(new_run(None)).await.unwrap();
        let completed = repo
            .complete(
                &run.id,
                CompleteHeartbeatRun {
                    status: "failed".to_owned(),
                    error: Some("clone failed".to_owned()),
                    error_kind: Some("infrastructure".to_owned()),
                },
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(completed.error_kind.as_deref(), Some("infrastructure"));
        assert_eq!(completed.error.as_deref(), Some("clone failed"));
    }

    #[tokio::test]
    async fn watchdog_authorization_contract() {
        let (_dir, repo, conn) = repo().await;
        seed(&conn).await;

        // Regular run: only its own issue.
        let run = repo.start(new_run(Some("i1"))).await.unwrap();
        assert!(repo.watchdog_authorized(&run.id, "i1").await.unwrap());
        assert!(!repo.watchdog_authorized(&run.id, "i2").await.unwrap());
        repo.complete(
            &run.id,
            CompleteHeartbeatRun {
                status: "succeeded".to_owned(),
                error: None,
                error_kind: None,
            },
        )
        .await
        .unwrap();

        // Watchdog run: subtree of watched issue, excluding task_watchdog
        // branches.
        let watchdog_run = repo
            .start(NewHeartbeatRun {
                company_id: "c1".to_owned(),
                agent_id: "a2".to_owned(),
                invocation_source: "scheduler".to_owned(),
                issue_id: None,
                context_snapshot: Some(
                    r#"{"kind":"task_watchdog","watchedIssueId":"i1"}"#.to_owned(),
                ),
                trigger_detail: None,
            })
            .await
            .unwrap();
        assert!(
            repo.watchdog_authorized(&watchdog_run.id, "i1")
                .await
                .unwrap()
        );
        assert!(
            repo.watchdog_authorized(&watchdog_run.id, "i2")
                .await
                .unwrap()
        );
        assert!(
            !repo
                .watchdog_authorized(&watchdog_run.id, "i3")
                .await
                .unwrap()
        );
        assert!(
            !repo
                .watchdog_authorized(&watchdog_run.id, "i4")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn new_exec_fields_are_readable() {
        let (_dir, repo, conn) = repo().await;
        seed(&conn).await;
        let run = repo
            .start(NewHeartbeatRun {
                company_id: "c1".to_owned(),
                agent_id: "a1".to_owned(),
                invocation_source: "manual".to_owned(),
                issue_id: None,
                context_snapshot: None,
                trigger_detail: None,
            })
            .await
            .unwrap();
        conn.execute(
            "UPDATE heartbeat_runs SET liveness_state = 'running', continuation_attempt = 3,
                    responsible_user_id = 'u1', process_pid = 42, exit_code = 1,
                    log_sha256 = 'abc'
             WHERE id = ?1",
            libsql::params![run.id.clone()],
        )
        .await
        .unwrap();
        let fetched = repo.get(&run.id).await.unwrap().unwrap();
        assert_eq!(fetched.liveness_state.as_deref(), Some("running"));
        assert_eq!(fetched.continuation_attempt, 3);
        assert_eq!(fetched.responsible_user_id.as_deref(), Some("u1"));
        assert_eq!(fetched.process_pid, Some(42));
        assert_eq!(fetched.exit_code, Some(1));
        assert_eq!(fetched.log_sha256.as_deref(), Some("abc"));
    }
}
