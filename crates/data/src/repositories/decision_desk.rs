//! Decision desk: queues, queue items, and triage.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A decision queue.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionQueueRecord {
    /// Queue id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Queue name.
    pub name: String,
    /// Queue key.
    pub key: Option<String>,
    /// Queue title.
    pub title: Option<String>,
    /// Creator actor type.
    pub created_by_type: Option<String>,
    /// Creator agent id.
    pub created_by_agent_id: Option<String>,
    /// Creator user id.
    pub created_by_user_id: Option<String>,
    /// Creator run id.
    pub created_by_run_id: Option<String>,
    /// Creator agent api key id.
    pub created_by_agent_api_key_id: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Retention override in days.
    pub retention_days: Option<i64>,
    /// Seed rules JSON.
    pub seed_rules: serde_json::Value,
    /// Whether seed rules are enabled.
    pub seed_rules_enabled: bool,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// A queue item.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionQueueItemRecord {
    /// Item id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Queue id.
    pub queue_id: String,
    /// Source kind.
    pub source_kind: String,
    /// Source id.
    pub source_id: String,
    /// Added-by actor type.
    pub added_by_type: Option<String>,
    /// Added-by agent id.
    pub added_by_agent_id: Option<String>,
    /// Added-by user id.
    pub added_by_user_id: Option<String>,
    /// Added-by run id.
    pub added_by_run_id: Option<String>,
    /// Added-by agent api key id.
    pub added_by_agent_api_key_id: Option<String>,
    /// Responsible user id.
    pub responsible_user_id: Option<String>,
    /// Payload JSON.
    pub payload: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// Input for upserting triage state.
#[derive(Debug, Clone)]
pub struct TriageInput {
    /// Decide-by time.
    pub decide_by: Option<String>,
    /// Snoozed until time.
    pub snoozed_until: Option<String>,
    /// Decision.
    pub decision: Option<String>,
    /// Deciding user.
    pub decided_by_user_id: Option<String>,
}

/// Triage state for an attention source.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionTriageRecord {
    /// Triage id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Source kind.
    pub source_kind: String,
    /// Source id.
    pub source_id: String,
    /// Decide-by time.
    pub decide_by: Option<String>,
    /// Decide-by date (calendar).
    pub decide_by_date: Option<String>,
    /// Snoozed until time.
    pub snoozed_until: Option<String>,
    /// Set-by actor type.
    pub set_by_type: Option<String>,
    /// Set-by agent id.
    pub set_by_agent_id: Option<String>,
    /// Set-by user id.
    pub set_by_user_id: Option<String>,
    /// Set-by run id.
    pub set_by_run_id: Option<String>,
    /// Set-by agent api key id.
    pub set_by_agent_api_key_id: Option<String>,
    /// Responsible user id.
    pub responsible_user_id: Option<String>,
    /// Decision.
    pub decision: Option<String>,
    /// Deciding user.
    pub decided_by_user_id: Option<String>,
    /// Triage version.
    pub version: i64,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// An immutable triage event.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionTriageEventRecord {
    /// Event id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Triage id.
    pub triage_id: String,
    /// Event type (`decided` | `snoozed` | `kept` | `archived` | `restored`).
    pub event_type: String,
    /// Decision at the time.
    pub decision: Option<String>,
    /// Deciding user.
    pub decided_by_user_id: Option<String>,
    /// Queue id.
    pub queue_id: Option<String>,
    /// Source kind.
    pub source_kind: Option<String>,
    /// Source id.
    pub source_id: Option<String>,
    /// Action.
    pub action: Option<String>,
    /// Actor type.
    pub actor_type: Option<String>,
    /// Actor agent id.
    pub actor_agent_id: Option<String>,
    /// Actor user id.
    pub actor_user_id: Option<String>,
    /// Actor run id.
    pub actor_run_id: Option<String>,
    /// Agent api key id.
    pub agent_api_key_id: Option<String>,
    /// Responsible user id.
    pub responsible_user_id: Option<String>,
    /// Details JSON.
    pub details: Option<serde_json::Value>,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// Retention state for a triage source.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionRetentionRecord {
    /// Retention id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Triage id.
    pub triage_id: String,
    /// Source kind.
    pub source_kind: String,
    /// Source id.
    pub source_id: String,
    /// Keep marker (skip the sweeper).
    pub keep: bool,
    /// Archived flag.
    pub archived: bool,
    /// ISO 8601 archive time.
    pub archived_at: Option<String>,
    /// Archive reason.
    pub archived_reason: Option<String>,
    /// ISO 8601 restore time.
    pub restored_at: Option<String>,
    /// Source activity timestamp.
    pub source_activity_at: Option<String>,
    /// Archiver actor type.
    pub archived_by_type: Option<String>,
    /// Archiver agent id.
    pub archived_by_agent_id: Option<String>,
    /// Archiver user id.
    pub archived_by_user_id: Option<String>,
    /// Archiver run id.
    pub archived_by_run_id: Option<String>,
    /// Retention version.
    pub version: i64,
    /// Archive version.
    pub archive_version: i64,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// An archive-notification outbox row.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionOutboxRecord {
    /// Outbox id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Triage id.
    pub triage_id: String,
    /// Notification kind.
    pub notification_kind: String,
    /// Recipient user id.
    pub recipient_user_id: Option<String>,
    /// Status (`pending` | `sent` | `failed`).
    pub status: String,
    /// Attempt count.
    pub attempt_count: i64,
    /// Last error.
    pub last_error: Option<String>,
    /// Dedupe key.
    pub dedupe_key: String,
    /// Archive version.
    pub archive_version: i64,
    /// ISO 8601 delivery time.
    pub delivered_at: Option<String>,
    /// ISO 8601 last attempt time.
    pub last_attempt_at: Option<String>,
    /// Origin agent id.
    pub origin_agent_id: Option<String>,
    /// Origin issue id.
    pub origin_issue_id: Option<String>,
    /// Source id.
    pub source_id: Option<String>,
    /// Source kind.
    pub source_kind: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
    /// ISO 8601 sent time.
    pub sent_at: Option<String>,
    /// ISO 8601 last update time.
    pub updated_at: Option<String>,
}

/// Result of a retention sweep.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionSweepResult {
    /// Number of triage rows archived.
    pub archived: i64,
    /// Number of notifications enqueued.
    pub notifications_enqueued: i64,
}

/// Decision desk errors.
#[derive(Debug, Error)]
pub enum DecisionError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The company does not exist.
    #[error("company not found")]
    CompanyNotFound,
    /// The queue does not exist in this company.
    #[error("queue not found")]
    QueueNotFound,
    /// The queue name is already taken.
    #[error("queue already exists")]
    QueueExists,
    /// The item is already in the queue.
    #[error("queue item already exists")]
    ItemExists,
}

/// Decision desk persistence contract.
#[async_trait]
pub trait DecisionRepository: Send + Sync {
    /// Creates a queue.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError`] on invalid references or duplicate names.
    async fn create_queue(
        &self,
        company_id: &str,
        name: &str,
        description: Option<String>,
        retention_days: Option<i64>,
    ) -> Result<DecisionQueueRecord, DecisionError>;

    /// Lists queues.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError`] on database failure.
    async fn list_queues(
        &self,
        company_id: &str,
    ) -> Result<Vec<DecisionQueueRecord>, DecisionError>;

    /// Adds an item to a queue.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError`] on invalid references or duplicates.
    async fn add_item(
        &self,
        company_id: &str,
        queue_id: &str,
        source_kind: &str,
        source_id: &str,
        payload: Option<String>,
    ) -> Result<DecisionQueueItemRecord, DecisionError>;

    /// Lists queue items.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError`] on database failure.
    async fn list_items(
        &self,
        company_id: &str,
        queue_id: &str,
    ) -> Result<Vec<DecisionQueueItemRecord>, DecisionError>;

    /// Upserts triage state for an attention source.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError`] on database failure.
    async fn set_triage(
        &self,
        company_id: &str,
        source_kind: &str,
        source_id: &str,
        input: TriageInput,
    ) -> Result<DecisionTriageRecord, DecisionError>;

    /// Lists triage state for a company.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError`] on database failure.
    async fn list_triage(
        &self,
        company_id: &str,
    ) -> Result<Vec<DecisionTriageRecord>, DecisionError>;

    /// Appends an immutable triage event.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError`] when the triage row is missing.
    async fn append_triage_event(
        &self,
        company_id: &str,
        triage_id: &str,
        event_type: &str,
        decision: Option<String>,
        decided_by_user_id: Option<String>,
    ) -> Result<DecisionTriageEventRecord, DecisionError>;

    /// Lists triage events for a company (optionally one triage).
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError`] on database failure.
    async fn list_triage_events(
        &self,
        company_id: &str,
        triage_id: Option<&str>,
    ) -> Result<Vec<DecisionTriageEventRecord>, DecisionError>;

    /// Marks a triage source as keep (skip the sweeper).
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError`] when the triage row is missing.
    async fn retention_set_keep(
        &self,
        company_id: &str,
        source_kind: &str,
        source_id: &str,
        keep: bool,
    ) -> Result<DecisionRetentionRecord, DecisionError>;

    /// Archives a triage source and enqueues a deduped notification.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError`] when the triage row is missing.
    async fn retention_archive(
        &self,
        company_id: &str,
        source_kind: &str,
        source_id: &str,
        reason: Option<String>,
        recipient_user_id: Option<String>,
    ) -> Result<DecisionRetentionRecord, DecisionError>;

    /// Restores an archived triage source.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError`] when the triage row is missing.
    async fn retention_restore(
        &self,
        company_id: &str,
        source_kind: &str,
        source_id: &str,
    ) -> Result<DecisionRetentionRecord, DecisionError>;

    /// Lists retention state for a company.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError`] on database failure.
    async fn list_retention(
        &self,
        company_id: &str,
    ) -> Result<Vec<DecisionRetentionRecord>, DecisionError>;

    /// Lists archive-notification outbox rows for a company.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError`] on database failure.
    async fn list_outbox(
        &self,
        company_id: &str,
    ) -> Result<Vec<DecisionOutboxRecord>, DecisionError>;

    /// Marks an outbox row sent.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError`] on database failure.
    async fn outbox_mark_sent(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<DecisionOutboxRecord>, DecisionError>;

    /// Sweeps triage rows older than `older_than_days` that are not kept:
    /// archives them and enqueues deduped notifications. Built-in task.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError`] on database failure.
    async fn sweep(
        &self,
        company_id: &str,
        older_than_days: i64,
    ) -> Result<DecisionSweepResult, DecisionError>;
}

/// Turso/libSQL implementation of [`DecisionRepository`].
#[derive(Debug)]
pub struct TursoDecisionRepository {
    db: Database,
}

impl TursoDecisionRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

fn row_opt_i64(row: &libsql::Row, idx: i32) -> Result<Option<i64>, libsql::Error> {
    let value = row.get_value(idx)?;
    if value.is_null() {
        Ok(None)
    } else {
        Ok(Some(*value.as_integer().expect("INTEGER column")))
    }
}

#[async_trait]
impl DecisionRepository for TursoDecisionRepository {
    async fn create_queue(
        &self,
        company_id: &str,
        name: &str,
        description: Option<String>,
        retention_days: Option<i64>,
    ) -> Result<DecisionQueueRecord, DecisionError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, company_id).await? {
            return Err(DecisionError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO decision_queues (id, company_id, name, description, retention_days,
                                              created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![id.clone(), company_id, name, description, retention_days],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        "SELECT id, company_id, name, key, title, created_by_type, created_by_agent_id,
                        created_by_user_id, created_by_run_id, created_by_agent_api_key_id,
                        description, retention_days, seed_rules, seed_rules_enabled, created_at
                         FROM decision_queues WHERE id = ?1",
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("queue was just inserted");
                Ok(DecisionQueueRecord {
                    id: helpers::row_text(&row, 0)?.expect("id"),
                    company_id: helpers::row_text(&row, 1)?.expect("company_id"),
                    name: helpers::row_text(&row, 2)?.expect("name"),
                    key: helpers::row_text(&row, 3)?,
                    title: helpers::row_text(&row, 4)?,
                    created_by_type: helpers::row_text(&row, 5)?,
                    created_by_agent_id: helpers::row_text(&row, 6)?,
                    created_by_user_id: helpers::row_text(&row, 7)?,
                    created_by_run_id: helpers::row_text(&row, 8)?,
                    created_by_agent_api_key_id: helpers::row_text(&row, 9)?,
                    description: helpers::row_text(&row, 10)?,
                    retention_days: row_opt_i64(&row, 11)?,
                    seed_rules: helpers::row_text(&row, 12)?
                        .and_then(|raw| serde_json::from_str(&raw).ok())
                        .unwrap_or_else(|| serde_json::json!([])),
                    seed_rules_enabled: helpers::row_i64(&row, 13)? != 0,
                    created_at: helpers::row_text(&row, 14)?.expect("created_at"),
                })
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(DecisionError::QueueExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_queues(
        &self,
        company_id: &str,
    ) -> Result<Vec<DecisionQueueRecord>, DecisionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, name, key, title, created_by_type, created_by_agent_id,
                        created_by_user_id, created_by_run_id, created_by_agent_api_key_id,
                        description, retention_days, seed_rules, seed_rules_enabled, created_at
                 FROM decision_queues WHERE company_id = ?1 ORDER BY name",
                libsql::params![company_id],
            )
            .await?;
        let mut queues = Vec::new();
        while let Some(row) = rows.next().await? {
            queues.push(DecisionQueueRecord {
                id: helpers::row_text(&row, 0)?.expect("id"),
                company_id: helpers::row_text(&row, 1)?.expect("company_id"),
                name: helpers::row_text(&row, 2)?.expect("name"),
                key: helpers::row_text(&row, 3)?,
                title: helpers::row_text(&row, 4)?,
                created_by_type: helpers::row_text(&row, 5)?,
                created_by_agent_id: helpers::row_text(&row, 6)?,
                created_by_user_id: helpers::row_text(&row, 7)?,
                created_by_run_id: helpers::row_text(&row, 8)?,
                created_by_agent_api_key_id: helpers::row_text(&row, 9)?,
                description: helpers::row_text(&row, 10)?,
                retention_days: row_opt_i64(&row, 11)?,
                seed_rules: helpers::row_text(&row, 12)?
                    .and_then(|raw| serde_json::from_str(&raw).ok())
                    .unwrap_or_else(|| serde_json::json!([])),
                seed_rules_enabled: helpers::row_i64(&row, 13)? != 0,
                created_at: helpers::row_text(&row, 14)?.expect("created_at"),
            });
        }
        Ok(queues)
    }

    async fn add_item(
        &self,
        company_id: &str,
        queue_id: &str,
        source_kind: &str,
        source_id: &str,
        payload: Option<String>,
    ) -> Result<DecisionQueueItemRecord, DecisionError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(&conn, "decision_queues", queue_id, company_id).await? {
            return Err(DecisionError::QueueNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO decision_queue_items (id, company_id, queue_id, source_kind,
                                                   source_id, payload, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    company_id,
                    queue_id,
                    source_kind,
                    source_id,
                    payload
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        "SELECT id, company_id, queue_id, source_kind, source_id, added_by_type,
                        added_by_agent_id, added_by_user_id, added_by_run_id,
                        added_by_agent_api_key_id, responsible_user_id, payload, created_at
                         FROM decision_queue_items WHERE id = ?1",
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("item was just inserted");
                Ok(DecisionQueueItemRecord {
                    id: helpers::row_text(&row, 0)?.expect("id"),
                    company_id: helpers::row_text(&row, 1)?.expect("company_id"),
                    queue_id: helpers::row_text(&row, 2)?.expect("queue_id"),
                    source_kind: helpers::row_text(&row, 3)?.expect("source_kind"),
                    source_id: helpers::row_text(&row, 4)?.expect("source_id"),
                    added_by_type: helpers::row_text(&row, 5)?,
                    added_by_agent_id: helpers::row_text(&row, 6)?,
                    added_by_user_id: helpers::row_text(&row, 7)?,
                    added_by_run_id: helpers::row_text(&row, 8)?,
                    added_by_agent_api_key_id: helpers::row_text(&row, 9)?,
                    responsible_user_id: helpers::row_text(&row, 10)?,
                    payload: helpers::row_text(&row, 11)?,
                    created_at: helpers::row_text(&row, 12)?.expect("created_at"),
                })
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(DecisionError::ItemExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_items(
        &self,
        company_id: &str,
        queue_id: &str,
    ) -> Result<Vec<DecisionQueueItemRecord>, DecisionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, queue_id, source_kind, source_id, added_by_type,
                        added_by_agent_id, added_by_user_id, added_by_run_id,
                        added_by_agent_api_key_id, responsible_user_id, payload, created_at
                 FROM decision_queue_items WHERE company_id = ?1 AND queue_id = ?2 ORDER BY created_at",
                libsql::params![company_id, queue_id],
            )
            .await?;
        let mut items = Vec::new();
        while let Some(row) = rows.next().await? {
            items.push(DecisionQueueItemRecord {
                id: helpers::row_text(&row, 0)?.expect("id"),
                company_id: helpers::row_text(&row, 1)?.expect("company_id"),
                queue_id: helpers::row_text(&row, 2)?.expect("queue_id"),
                source_kind: helpers::row_text(&row, 3)?.expect("source_kind"),
                source_id: helpers::row_text(&row, 4)?.expect("source_id"),
                added_by_type: helpers::row_text(&row, 5)?,
                added_by_agent_id: helpers::row_text(&row, 6)?,
                added_by_user_id: helpers::row_text(&row, 7)?,
                added_by_run_id: helpers::row_text(&row, 8)?,
                added_by_agent_api_key_id: helpers::row_text(&row, 9)?,
                responsible_user_id: helpers::row_text(&row, 10)?,
                payload: helpers::row_text(&row, 11)?,
                created_at: helpers::row_text(&row, 12)?.expect("created_at"),
            });
        }
        Ok(items)
    }

    async fn set_triage(
        &self,
        company_id: &str,
        source_kind: &str,
        source_id: &str,
        input: TriageInput,
    ) -> Result<DecisionTriageRecord, DecisionError> {
        let conn = crate::connection::connect(&self.db).await?;
        conn.execute(
            "INSERT INTO decision_triage (id, company_id, source_kind, source_id, decide_by,
                                          snoozed_until, decision, decided_by_user_id, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT (company_id, source_kind, source_id) DO UPDATE SET
                 decide_by = excluded.decide_by,
                 snoozed_until = excluded.snoozed_until,
                 decision = excluded.decision,
                 decided_by_user_id = excluded.decided_by_user_id,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            libsql::params![
                Uuid::new_v4().to_string(),
                company_id,
                source_kind,
                source_id,
                input.decide_by,
                input.snoozed_until,
                input.decision,
                input.decided_by_user_id
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, source_kind, source_id, decide_by, decide_by_date,
                        snoozed_until, set_by_type, set_by_agent_id, set_by_user_id,
                        set_by_run_id, set_by_agent_api_key_id, responsible_user_id, decision,
                        decided_by_user_id, version, created_at
                 FROM decision_triage WHERE company_id = ?1 AND source_kind = ?2 AND source_id = ?3",
                libsql::params![company_id, source_kind, source_id],
            )
            .await?;
        let row = rows.next().await?.expect("triage exists");
        Ok(DecisionTriageRecord {
            id: helpers::row_text(&row, 0)?.expect("id"),
            company_id: helpers::row_text(&row, 1)?.expect("company_id"),
            source_kind: helpers::row_text(&row, 2)?.expect("source_kind"),
            source_id: helpers::row_text(&row, 3)?.expect("source_id"),
            decide_by: helpers::row_text(&row, 4)?,
            decide_by_date: helpers::row_text(&row, 5)?,
            snoozed_until: helpers::row_text(&row, 6)?,
            set_by_type: helpers::row_text(&row, 7)?,
            set_by_agent_id: helpers::row_text(&row, 8)?,
            set_by_user_id: helpers::row_text(&row, 9)?,
            set_by_run_id: helpers::row_text(&row, 10)?,
            set_by_agent_api_key_id: helpers::row_text(&row, 11)?,
            responsible_user_id: helpers::row_text(&row, 12)?,
            decision: helpers::row_text(&row, 13)?,
            decided_by_user_id: helpers::row_text(&row, 14)?,
            version: helpers::row_i64(&row, 15)?,
            created_at: helpers::row_text(&row, 16)?.expect("created_at"),
        })
    }

    async fn list_triage(
        &self,
        company_id: &str,
    ) -> Result<Vec<DecisionTriageRecord>, DecisionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, source_kind, source_id, decide_by, decide_by_date,
                        snoozed_until, set_by_type, set_by_agent_id, set_by_user_id,
                        set_by_run_id, set_by_agent_api_key_id, responsible_user_id, decision,
                        decided_by_user_id, version, created_at
                 FROM decision_triage WHERE company_id = ?1 ORDER BY updated_at DESC",
                libsql::params![company_id],
            )
            .await?;
        let mut triage = Vec::new();
        while let Some(row) = rows.next().await? {
            triage.push(DecisionTriageRecord {
                id: helpers::row_text(&row, 0)?.expect("id"),
                company_id: helpers::row_text(&row, 1)?.expect("company_id"),
                source_kind: helpers::row_text(&row, 2)?.expect("source_kind"),
                source_id: helpers::row_text(&row, 3)?.expect("source_id"),
                decide_by: helpers::row_text(&row, 4)?,
                decide_by_date: helpers::row_text(&row, 5)?,
                snoozed_until: helpers::row_text(&row, 6)?,
                set_by_type: helpers::row_text(&row, 7)?,
                set_by_agent_id: helpers::row_text(&row, 8)?,
                set_by_user_id: helpers::row_text(&row, 9)?,
                set_by_run_id: helpers::row_text(&row, 10)?,
                set_by_agent_api_key_id: helpers::row_text(&row, 11)?,
                responsible_user_id: helpers::row_text(&row, 12)?,
                decision: helpers::row_text(&row, 13)?,
                decided_by_user_id: helpers::row_text(&row, 14)?,
                version: helpers::row_i64(&row, 15)?,
                created_at: helpers::row_text(&row, 16)?.expect("created_at"),
            });
        }
        Ok(triage)
    }

    async fn append_triage_event(
        &self,
        company_id: &str,
        triage_id: &str,
        event_type: &str,
        decision: Option<String>,
        decided_by_user_id: Option<String>,
    ) -> Result<DecisionTriageEventRecord, DecisionError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(&conn, "decision_triage", triage_id, company_id).await?
        {
            return Err(DecisionError::QueueNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO decision_triage_events
               (id, company_id, triage_id, event_type, decision, decided_by_user_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                company_id,
                triage_id,
                event_type,
                decision,
                decided_by_user_id
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, triage_id, event_type, decision, decided_by_user_id,
                        queue_id, source_kind, source_id, action, actor_type, actor_agent_id,
                        actor_user_id, actor_run_id, agent_api_key_id, responsible_user_id,
                        details, created_at
                 FROM decision_triage_events WHERE id = ?1",
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("event was just inserted");
        Ok(row_to_event(&row)?)
    }

    async fn list_triage_events(
        &self,
        company_id: &str,
        triage_id: Option<&str>,
    ) -> Result<Vec<DecisionTriageEventRecord>, DecisionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = match triage_id {
            Some(triage_id) => {
                conn.query(
                    "SELECT id, company_id, triage_id, event_type, decision, decided_by_user_id,
                        queue_id, source_kind, source_id, action, actor_type, actor_agent_id,
                        actor_user_id, actor_run_id, agent_api_key_id, responsible_user_id,
                        details, created_at
                     FROM decision_triage_events WHERE company_id = ?1 AND triage_id = ?2
                     ORDER BY created_at",
                    libsql::params![company_id, triage_id],
                )
                .await?
            }
            None => {
                conn.query(
                    "SELECT id, company_id, triage_id, event_type, decision, decided_by_user_id,
                        queue_id, source_kind, source_id, action, actor_type, actor_agent_id,
                        actor_user_id, actor_run_id, agent_api_key_id, responsible_user_id,
                        details, created_at
                     FROM decision_triage_events WHERE company_id = ?1 ORDER BY created_at",
                    libsql::params![company_id],
                )
                .await?
            }
        };
        let mut events = Vec::new();
        while let Some(row) = rows.next().await? {
            events.push(row_to_event(&row)?);
        }
        Ok(events)
    }

    async fn retention_set_keep(
        &self,
        company_id: &str,
        source_kind: &str,
        source_id: &str,
        keep: bool,
    ) -> Result<DecisionRetentionRecord, DecisionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let triage_id = resolve_triage(&conn, company_id, source_kind, source_id).await?;
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO decision_retention
               (id, company_id, triage_id, source_kind, source_id, keep, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT (company_id, triage_id)
             DO UPDATE SET keep = excluded.keep,
                           updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            libsql::params![
                id.clone(),
                company_id,
                triage_id.clone(),
                source_kind,
                source_id,
                i64::from(keep)
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, triage_id, source_kind, source_id, keep, archived,
                        archived_at, archived_reason, restored_at, source_activity_at,
                        archived_by_type, archived_by_agent_id, archived_by_user_id,
                        archived_by_run_id, version, archive_version, created_at
                 FROM decision_retention WHERE company_id = ?1 AND triage_id = ?2",
                libsql::params![company_id, triage_id],
            )
            .await?;
        let row = rows.next().await?.expect("retention was just upserted");
        Ok(row_to_retention(&row)?)
    }

    async fn retention_archive(
        &self,
        company_id: &str,
        source_kind: &str,
        source_id: &str,
        reason: Option<String>,
        recipient_user_id: Option<String>,
    ) -> Result<DecisionRetentionRecord, DecisionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let triage_id = resolve_triage(&conn, company_id, source_kind, source_id).await?;
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO decision_retention
               (id, company_id, triage_id, source_kind, source_id, keep, archived,
                archived_at, archived_reason, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 0, 1, strftime('%Y-%m-%dT%H:%M:%fZ','now'), ?6,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT (company_id, triage_id)
             DO UPDATE SET archived = 1, keep = 0,
                           archived_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                           archived_reason = COALESCE(?6, archived_reason),
                           restored_at = NULL,
                           updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            libsql::params![
                id.clone(),
                company_id,
                triage_id.clone(),
                source_kind,
                source_id,
                reason
            ],
        )
        .await?;
        // Deduped outbox entry (one per triage + kind).
        let dedupe_key = format!("{company_id}:{triage_id}:archive");
        let outbox_id = Uuid::new_v4().to_string();
        let outbox_result = conn
            .execute(
                "INSERT INTO decision_archive_notification_outbox
                   (id, company_id, triage_id, notification_kind, recipient_user_id, status,
                    dedupe_key, created_at)
                 VALUES (?1, ?2, ?3, 'archive', ?4, 'pending', ?5,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))
                 ON CONFLICT (company_id, dedupe_key) DO NOTHING",
                libsql::params![
                    outbox_id,
                    company_id,
                    triage_id.clone(),
                    recipient_user_id,
                    dedupe_key
                ],
            )
            .await?;
        let _ = outbox_result;
        let mut rows = conn
            .query(
                "SELECT id, company_id, triage_id, source_kind, source_id, keep, archived,
                        archived_at, archived_reason, restored_at, source_activity_at,
                        archived_by_type, archived_by_agent_id, archived_by_user_id,
                        archived_by_run_id, version, archive_version, created_at
                 FROM decision_retention WHERE company_id = ?1 AND triage_id = ?2",
                libsql::params![company_id, triage_id],
            )
            .await?;
        let row = rows.next().await?.expect("retention exists");
        Ok(row_to_retention(&row)?)
    }

    async fn retention_restore(
        &self,
        company_id: &str,
        source_kind: &str,
        source_id: &str,
    ) -> Result<DecisionRetentionRecord, DecisionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let triage_id = resolve_triage(&conn, company_id, source_kind, source_id).await?;
        let updated = conn
            .execute(
                "UPDATE decision_retention SET archived = 0, restored_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                        updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE company_id = ?1 AND triage_id = ?2",
                libsql::params![company_id, triage_id.clone()],
            )
            .await?;
        if updated == 0 {
            return Err(DecisionError::QueueNotFound);
        }
        let mut rows = conn
            .query(
                "SELECT id, company_id, triage_id, source_kind, source_id, keep, archived,
                        archived_at, archived_reason, restored_at, source_activity_at,
                        archived_by_type, archived_by_agent_id, archived_by_user_id,
                        archived_by_run_id, version, archive_version, created_at
                 FROM decision_retention WHERE company_id = ?1 AND triage_id = ?2",
                libsql::params![company_id, triage_id],
            )
            .await?;
        let row = rows.next().await?.expect("retention exists");
        Ok(row_to_retention(&row)?)
    }

    async fn list_retention(
        &self,
        company_id: &str,
    ) -> Result<Vec<DecisionRetentionRecord>, DecisionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, triage_id, source_kind, source_id, keep, archived,
                        archived_at, archived_reason, restored_at, source_activity_at,
                        archived_by_type, archived_by_agent_id, archived_by_user_id,
                        archived_by_run_id, version, archive_version, created_at
                 FROM decision_retention WHERE company_id = ?1 ORDER BY updated_at DESC",
                libsql::params![company_id],
            )
            .await?;
        let mut retention = Vec::new();
        while let Some(row) = rows.next().await? {
            retention.push(row_to_retention(&row)?);
        }
        Ok(retention)
    }

    async fn list_outbox(
        &self,
        company_id: &str,
    ) -> Result<Vec<DecisionOutboxRecord>, DecisionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, triage_id, notification_kind, recipient_user_id, status,
                        attempt_count, last_error, dedupe_key, archive_version, delivered_at,
                        last_attempt_at, origin_agent_id, origin_issue_id, source_id,
                        source_kind, created_at, sent_at, updated_at
                 FROM decision_archive_notification_outbox
                 WHERE company_id = ?1 ORDER BY created_at DESC",
                libsql::params![company_id],
            )
            .await?;
        let mut outbox = Vec::new();
        while let Some(row) = rows.next().await? {
            outbox.push(row_to_outbox(&row)?);
        }
        Ok(outbox)
    }

    async fn outbox_mark_sent(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<DecisionOutboxRecord>, DecisionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let updated = conn
            .execute(
                "UPDATE decision_archive_notification_outbox
                 SET status = 'sent', sent_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE company_id = ?1 AND id = ?2 AND status = 'pending'",
                libsql::params![company_id, id],
            )
            .await?;
        if updated == 0 {
            return Ok(None);
        }
        let mut rows = conn
            .query(
                "SELECT id, company_id, triage_id, notification_kind, recipient_user_id, status,
                        attempt_count, last_error, dedupe_key, archive_version, delivered_at,
                        last_attempt_at, origin_agent_id, origin_issue_id, source_id,
                        source_kind, created_at, sent_at, updated_at
                 FROM decision_archive_notification_outbox WHERE id = ?1",
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("outbox exists");
        Ok(Some(row_to_outbox(&row)?))
    }

    async fn sweep(
        &self,
        company_id: &str,
        older_than_days: i64,
    ) -> Result<DecisionSweepResult, DecisionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut archived = 0i64;
        let mut notifications = 0i64;
        // Candidate triage rows older than the window, without an active
        // retention row that keeps or archives them.
        let mut rows = conn
            .query(
                "SELECT t.id, t.source_kind, t.source_id
                 FROM decision_triage t
                 LEFT JOIN decision_retention r ON r.company_id = t.company_id AND r.triage_id = t.id
                 WHERE t.company_id = ?1
                   AND (r.id IS NULL OR (r.keep = 0 AND r.archived = 0))
                   AND t.updated_at < datetime('now', ?2)",
                libsql::params![company_id, format!("-{older_than_days} days")],
            )
            .await?;
        let mut candidates = Vec::new();
        while let Some(row) = rows.next().await? {
            candidates.push((
                helpers::row_text(&row, 0)?.expect("triage id"),
                helpers::row_text(&row, 1)?.expect("source_kind"),
                helpers::row_text(&row, 2)?.expect("source_id"),
            ));
        }
        for (triage_id, source_kind, source_id) in candidates {
            let updated = conn
                .execute(
                    "INSERT INTO decision_retention
                       (id, company_id, triage_id, source_kind, source_id, keep, archived,
                        archived_at, archived_reason, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, 0, 1, strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                             'sweeper:90d', strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                             strftime('%Y-%m-%dT%H:%M:%fZ','now'))
                     ON CONFLICT (company_id, triage_id) DO UPDATE SET archived = 1, keep = 0,
                           archived_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                           restored_at = NULL,
                           updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
                    libsql::params![
                        Uuid::new_v4().to_string(),
                        company_id,
                        triage_id.clone(),
                        source_kind,
                        source_id
                    ],
                )
                .await?;
            archived += updated as i64;
            let dedupe_key = format!("{company_id}:{triage_id}:archive");
            let outbox_result = conn
                .execute(
                    "INSERT INTO decision_archive_notification_outbox
                       (id, company_id, triage_id, notification_kind, status, dedupe_key, created_at)
                     VALUES (?1, ?2, ?3, 'archive', 'pending', ?4,
                             strftime('%Y-%m-%dT%H:%M:%fZ','now'))
                     ON CONFLICT (company_id, dedupe_key) DO NOTHING",
                    libsql::params![Uuid::new_v4().to_string(), company_id, triage_id, dedupe_key],
                )
                .await?;
            notifications += outbox_result as i64;
        }
        Ok(DecisionSweepResult {
            archived,
            notifications_enqueued: notifications,
        })
    }
}

fn row_to_event(row: &libsql::Row) -> Result<DecisionTriageEventRecord, libsql::Error> {
    Ok(DecisionTriageEventRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        triage_id: helpers::row_text(row, 2)?.expect("triage_id"),
        event_type: helpers::row_text(row, 3)?.expect("event_type"),
        decision: helpers::row_text(row, 4)?,
        decided_by_user_id: helpers::row_text(row, 5)?,
        queue_id: helpers::row_text(row, 6)?,
        source_kind: helpers::row_text(row, 7)?,
        source_id: helpers::row_text(row, 8)?,
        action: helpers::row_text(row, 9)?,
        actor_type: helpers::row_text(row, 10)?,
        actor_agent_id: helpers::row_text(row, 11)?,
        actor_user_id: helpers::row_text(row, 12)?,
        actor_run_id: helpers::row_text(row, 13)?,
        agent_api_key_id: helpers::row_text(row, 14)?,
        responsible_user_id: helpers::row_text(row, 15)?,
        details: helpers::row_text(row, 16)?.and_then(|raw| serde_json::from_str(&raw).ok()),
        created_at: helpers::row_text(row, 17)?.expect("created_at"),
    })
}

fn row_to_retention(row: &libsql::Row) -> Result<DecisionRetentionRecord, libsql::Error> {
    Ok(DecisionRetentionRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        triage_id: helpers::row_text(row, 2)?.expect("triage_id"),
        source_kind: helpers::row_text(row, 3)?.expect("source_kind"),
        source_id: helpers::row_text(row, 4)?.expect("source_id"),
        keep: helpers::row_i64(row, 5)? != 0,
        archived: helpers::row_i64(row, 6)? != 0,
        archived_at: helpers::row_text(row, 7)?,
        archived_reason: helpers::row_text(row, 8)?,
        restored_at: helpers::row_text(row, 9)?,
        source_activity_at: helpers::row_text(row, 10)?,
        archived_by_type: helpers::row_text(row, 11)?,
        archived_by_agent_id: helpers::row_text(row, 12)?,
        archived_by_user_id: helpers::row_text(row, 13)?,
        archived_by_run_id: helpers::row_text(row, 14)?,
        version: helpers::row_i64(row, 15)?,
        archive_version: helpers::row_i64(row, 16)?,
        created_at: helpers::row_text(row, 17)?.expect("created_at"),
    })
}

fn row_to_outbox(row: &libsql::Row) -> Result<DecisionOutboxRecord, libsql::Error> {
    Ok(DecisionOutboxRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        triage_id: helpers::row_text(row, 2)?.expect("triage_id"),
        notification_kind: helpers::row_text(row, 3)?.expect("notification_kind"),
        recipient_user_id: helpers::row_text(row, 4)?,
        status: helpers::row_text(row, 5)?.expect("status"),
        attempt_count: helpers::row_i64(row, 6)?,
        last_error: helpers::row_text(row, 7)?,
        dedupe_key: helpers::row_text(row, 8)?.expect("dedupe_key"),
        archive_version: helpers::row_i64(row, 9)?,
        delivered_at: helpers::row_text(row, 10)?,
        last_attempt_at: helpers::row_text(row, 11)?,
        origin_agent_id: helpers::row_text(row, 12)?,
        origin_issue_id: helpers::row_text(row, 13)?,
        source_id: helpers::row_text(row, 14)?,
        source_kind: helpers::row_text(row, 15)?,
        created_at: helpers::row_text(row, 16)?.expect("created_at"),
        sent_at: helpers::row_text(row, 17)?,
        updated_at: helpers::row_text(row, 18)?,
    })
}

async fn resolve_triage(
    conn: &libsql::Connection,
    company_id: &str,
    source_kind: &str,
    source_id: &str,
) -> Result<String, DecisionError> {
    let mut rows = conn
        .query(
            "SELECT id FROM decision_triage WHERE company_id = ?1 AND source_kind = ?2 AND source_id = ?3",
            libsql::params![company_id, source_kind, source_id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(helpers::row_text(&row, 0)?.expect("triage id")),
        None => Err(DecisionError::QueueNotFound),
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoDecisionRepository) {
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
        let repo = TursoDecisionRepository::new(db);
        (dir, repo)
    }

    #[tokio::test]
    async fn queue_item_triage_roundtrip() {
        let (_dir, repo) = repo().await;
        let queue = repo
            .create_queue(
                "c1",
                "approvals",
                Some("pending approvals".to_owned()),
                Some(30),
            )
            .await
            .unwrap();
        assert_eq!(queue.name, "approvals");

        // Duplicate queue name rejected.
        let error = repo
            .create_queue("c1", "approvals", None, None)
            .await
            .unwrap_err();
        assert!(matches!(error, DecisionError::QueueExists));

        // Items.
        let item = repo
            .add_item("c1", &queue.id, "approval", "a1", Some("{}".to_owned()))
            .await
            .unwrap();
        assert_eq!(item.source_id, "a1");
        let error = repo
            .add_item("c1", &queue.id, "approval", "a1", None)
            .await
            .unwrap_err();
        assert!(matches!(error, DecisionError::ItemExists));
        let items = repo.list_items("c1", &queue.id).await.unwrap();
        assert_eq!(items.len(), 1);

        // Triage upsert.
        let triage = repo
            .set_triage(
                "c1",
                "approval",
                "a1",
                TriageInput {
                    decide_by: Some("2026-08-10T00:00:00Z".to_owned()),
                    snoozed_until: None,
                    decision: Some("approved".to_owned()),
                    decided_by_user_id: Some("board".to_owned()),
                },
            )
            .await
            .unwrap();
        assert_eq!(triage.decision.as_deref(), Some("approved"));
        let triage2 = repo
            .set_triage(
                "c1",
                "approval",
                "a1",
                TriageInput {
                    decide_by: None,
                    snoozed_until: None,
                    decision: Some("rejected".to_owned()),
                    decided_by_user_id: Some("board".to_owned()),
                },
            )
            .await
            .unwrap();
        assert_eq!(triage2.decision.as_deref(), Some("rejected"));
        let all = repo.list_triage("c1").await.unwrap();
        assert_eq!(all.len(), 1);
    }

    #[tokio::test]
    async fn retention_archive_restore_and_outbox_dedupe() {
        let (_dir, repo) = repo().await;
        let triage = repo
            .set_triage(
                "c1",
                "issue",
                "i1",
                TriageInput {
                    decide_by: None,
                    snoozed_until: None,
                    decision: Some("approved".to_owned()),
                    decided_by_user_id: Some("board".to_owned()),
                },
            )
            .await
            .unwrap();
        repo.append_triage_event(
            "c1",
            &triage.id,
            "decided",
            Some("approved".to_owned()),
            None,
        )
        .await
        .unwrap();
        assert!(
            repo.list_triage_events("c1", Some(&triage.id))
                .await
                .unwrap()
                .len()
                == 1
        );

        let kept = repo
            .retention_set_keep("c1", "issue", "i1", true)
            .await
            .unwrap();
        assert!(kept.keep);
        let archived = repo
            .retention_archive(
                "c1",
                "issue",
                "i1",
                Some("90d".to_owned()),
                Some("u1".to_owned()),
            )
            .await
            .unwrap();
        assert!(archived.archived);
        assert!(!archived.keep);
        // Archiving again keeps the outbox deduped (one row).
        repo.retention_archive(
            "c1",
            "issue",
            "i1",
            Some("90d".to_owned()),
            Some("u1".to_owned()),
        )
        .await
        .unwrap();
        let outbox = repo.list_outbox("c1").await.unwrap();
        assert_eq!(outbox.len(), 1);
        assert_eq!(outbox[0].status, "pending");

        // Mark sent.
        let sent = repo
            .outbox_mark_sent("c1", &outbox[0].id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(sent.status, "sent");
        assert!(
            repo.outbox_mark_sent("c1", &outbox[0].id)
                .await
                .unwrap()
                .is_none()
        );

        // Restore.
        let restored = repo.retention_restore("c1", "issue", "i1").await.unwrap();
        assert!(!restored.archived);
        assert!(restored.restored_at.is_some());
        assert!(repo.list_retention("c1").await.unwrap().len() == 1);
    }

    #[tokio::test]
    async fn sweep_archives_old_triage_and_enqueues_dedupe_notifications() {
        let (_dir, repo) = repo().await;
        // Old triage row (created now; backdate via SQL so the sweeper sees it).
        let triage = repo
            .set_triage(
                "c1",
                "issue",
                "old",
                TriageInput {
                    decide_by: None,
                    snoozed_until: None,
                    decision: None,
                    decided_by_user_id: None,
                },
            )
            .await
            .unwrap();
        let conn = crate::connect(&repo.db).await.unwrap();
        conn.execute(
            "UPDATE decision_triage SET updated_at = datetime('now', '-100 days') WHERE id = ?1",
            libsql::params![triage.id],
        )
        .await
        .unwrap();
        // Kept triage must survive the sweep.
        let kept = repo
            .set_triage(
                "c1",
                "issue",
                "kept",
                TriageInput {
                    decide_by: None,
                    snoozed_until: None,
                    decision: None,
                    decided_by_user_id: None,
                },
            )
            .await
            .unwrap();
        conn.execute(
            "UPDATE decision_triage SET updated_at = datetime('now', '-100 days') WHERE id = ?1",
            libsql::params![kept.id],
        )
        .await
        .unwrap();
        repo.retention_set_keep("c1", "issue", "kept", true)
            .await
            .unwrap();

        let result = repo.sweep("c1", 90).await.unwrap();
        assert_eq!(result.archived, 1);
        assert_eq!(result.notifications_enqueued, 1);
        let retention = repo.list_retention("c1").await.unwrap();
        let old_row = retention.iter().find(|r| r.source_id == "old").unwrap();
        assert!(old_row.archived);
        let kept_row = retention.iter().find(|r| r.source_id == "kept").unwrap();
        assert!(!kept_row.archived);

        // Second sweep is a no-op (already archived, notifications deduped).
        let again = repo.sweep("c1", 90).await.unwrap();
        assert_eq!(again.archived, 0);
        assert_eq!(again.notifications_enqueued, 0);
    }

    #[tokio::test]
    async fn column_alignment_readback() {
        let (_dir, repo) = repo().await;
        let conn = crate::connect(&repo.db).await.unwrap();

        // Queue: new columns are read back after a SQL update.
        let queue = repo
            .create_queue("c1", "approvals", Some("desc".to_owned()), Some(30))
            .await
            .unwrap();
        conn.execute(
            "UPDATE decision_queues SET key = 'approvals', title = 'Pending approvals',
                    created_by_type = 'user', created_by_user_id = 'u1',
                    seed_rules = '[]', seed_rules_enabled = 1
             WHERE id = ?1",
            libsql::params![queue.id.clone()],
        )
        .await
        .unwrap();
        let queues = repo.list_queues("c1").await.unwrap();
        assert_eq!(queues.len(), 1);
        assert_eq!(queues[0].key.as_deref(), Some("approvals"));
        assert_eq!(queues[0].title.as_deref(), Some("Pending approvals"));
        assert_eq!(queues[0].created_by_type.as_deref(), Some("user"));
        assert_eq!(queues[0].created_by_user_id.as_deref(), Some("u1"));
        assert!(queues[0].seed_rules_enabled);

        // Item: added-by / responsible columns.
        let item = repo
            .add_item("c1", &queue.id, "approval", "a1", Some("{}".to_owned()))
            .await
            .unwrap();
        conn.execute(
            "UPDATE decision_queue_items SET added_by_type = 'agent',
                    added_by_agent_id = '11111111-1111-1111-1111-111111111111',
                    responsible_user_id = 'u2'
             WHERE id = ?1",
            libsql::params![item.id.clone()],
        )
        .await
        .unwrap();
        let items = repo.list_items("c1", &queue.id).await.unwrap();
        assert_eq!(items[0].added_by_type.as_deref(), Some("agent"));
        assert_eq!(items[0].responsible_user_id.as_deref(), Some("u2"));

        // Triage: decide-by-date / set-by / version.
        let triage = repo
            .set_triage(
                "c1",
                "approval",
                "a1",
                TriageInput {
                    decide_by: Some("2026-08-10T00:00:00Z".to_owned()),
                    snoozed_until: None,
                    decision: None,
                    decided_by_user_id: None,
                },
            )
            .await
            .unwrap();
        conn.execute(
            "UPDATE decision_triage SET decide_by_date = '2026-08-10',
                    set_by_type = 'user', set_by_user_id = 'u1', version = 3
             WHERE id = ?1",
            libsql::params![triage.id.clone()],
        )
        .await
        .unwrap();
        let triage_rows = repo.list_triage("c1").await.unwrap();
        assert_eq!(triage_rows[0].decide_by_date.as_deref(), Some("2026-08-10"));
        assert_eq!(triage_rows[0].set_by_type.as_deref(), Some("user"));
        assert_eq!(triage_rows[0].set_by_user_id.as_deref(), Some("u1"));
        assert_eq!(triage_rows[0].version, 3);
        assert!(!triage_rows[0].created_at.is_empty());

        // Triage event: actor / action / details.
        let event = repo
            .append_triage_event(
                "c1",
                &triage.id,
                "decided",
                Some("approve".to_owned()),
                Some("u1".to_owned()),
            )
            .await
            .unwrap();
        conn.execute(
            "UPDATE decision_triage_events SET action = 'decided',
                    actor_type = 'user', actor_user_id = 'u1', responsible_user_id = 'u2',
                    details = '{\"note\":\"ok\"}'
             WHERE id = ?1",
            libsql::params![event.id.clone()],
        )
        .await
        .unwrap();
        let events = repo
            .list_triage_events("c1", Some(&triage.id))
            .await
            .unwrap();
        assert_eq!(events[0].action.as_deref(), Some("decided"));
        assert_eq!(events[0].actor_user_id.as_deref(), Some("u1"));
        assert_eq!(events[0].responsible_user_id.as_deref(), Some("u2"));
        assert!(events[0].details.is_some());

        // Retention + outbox read back new columns after archive.
        let archived = repo
            .retention_archive("c1", "approval", "a1", Some("90d".to_owned()), None)
            .await
            .unwrap();
        assert_eq!(archived.archive_version, 0);
        let outbox = repo.list_outbox("c1").await.unwrap();
        assert!(!outbox.is_empty());
        conn.execute(
            "UPDATE decision_archive_notification_outbox SET delivered_at = '2026-08-05T00:00:00Z',
                    source_kind = 'approval', source_id = 'a1'
             WHERE id = ?1",
            libsql::params![outbox[0].id.clone()],
        )
        .await
        .unwrap();
        let outbox_rows = repo.list_outbox("c1").await.unwrap();
        assert_eq!(outbox_rows[0].source_kind.as_deref(), Some("approval"));
        assert!(outbox_rows[0].delivered_at.is_some());
    }
}
