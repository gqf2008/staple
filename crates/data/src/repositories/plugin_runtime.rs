//! Plugin runtime repository: scoped state, entities, jobs/runs, logs,
//! webhook deliveries, database namespaces, and migration ledger.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `plugin_state` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginStateRecord {
    /// State id.
    pub id: String,
    /// Plugin id.
    pub plugin_id: String,
    /// Scope kind.
    pub scope_kind: String,
    /// Scope id.
    pub scope_id: Option<String>,
    /// Namespace.
    pub namespace: String,
    /// State key.
    pub state_key: String,
    /// JSON value.
    pub value_json: serde_json::Value,
    /// ISO 8601 last write.
    pub updated_at: String,
}

/// A row of the `plugin_entities` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginEntityRecord {
    /// Entity id.
    pub id: String,
    /// Plugin id.
    pub plugin_id: String,
    /// Company id (null for instance scope).
    pub company_id: Option<String>,
    /// Entity type.
    pub entity_type: String,
    /// Scope kind.
    pub scope_kind: String,
    /// Scope id.
    pub scope_id: Option<String>,
    /// External id.
    pub external_id: Option<String>,
    /// Title.
    pub title: Option<String>,
    /// Status.
    pub status: Option<String>,
    /// Data JSON.
    pub data: serde_json::Value,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// A row of the `plugin_jobs` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginJobRecord {
    /// Job id.
    pub id: String,
    /// Plugin id.
    pub plugin_id: String,
    /// Job key.
    pub job_key: String,
    /// Schedule.
    pub schedule: String,
    /// Status (`active` | `paused` | `error`).
    pub status: String,
    /// ISO 8601 last run.
    pub last_run_at: Option<String>,
    /// ISO 8601 next run.
    pub next_run_at: Option<String>,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// A row of the `plugin_job_runs` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginJobRunRecord {
    /// Run id.
    pub id: String,
    /// Job id.
    pub job_id: String,
    /// Plugin id.
    pub plugin_id: String,
    /// Company id.
    pub company_id: Option<String>,
    /// Trigger (`scheduled` | `manual`).
    pub trigger: String,
    /// Status.
    pub status: String,
    /// Duration ms.
    pub duration_ms: Option<i64>,
    /// Error.
    pub error: Option<String>,
    /// Log lines.
    pub logs: Vec<String>,
    /// ISO 8601 start.
    pub started_at: Option<String>,
    /// ISO 8601 finish.
    pub finished_at: Option<String>,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// A row of the `plugin_logs` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginLogRecord {
    /// Log id.
    pub id: String,
    /// Plugin id.
    pub plugin_id: String,
    /// Company id.
    pub company_id: Option<String>,
    /// Level.
    pub level: String,
    /// Message.
    pub message: String,
    /// Meta JSON.
    pub meta: Option<serde_json::Value>,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// A row of the `plugin_webhook_deliveries` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginWebhookDeliveryRecord {
    /// Delivery id.
    pub id: String,
    /// Plugin id.
    pub plugin_id: String,
    /// Company id.
    pub company_id: Option<String>,
    /// Webhook key.
    pub webhook_key: String,
    /// External dedup id.
    pub external_id: Option<String>,
    /// Status.
    pub status: String,
    /// Duration ms.
    pub duration_ms: Option<i64>,
    /// Error.
    pub error: Option<String>,
    /// Payload JSON.
    pub payload: serde_json::Value,
    /// Headers JSON.
    pub headers: serde_json::Value,
    /// ISO 8601 start.
    pub started_at: Option<String>,
    /// ISO 8601 finish.
    pub finished_at: Option<String>,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// A row of the `plugin_database_namespaces` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginDatabaseNamespaceRecord {
    /// Namespace id.
    pub id: String,
    /// Plugin id.
    pub plugin_id: String,
    /// Plugin key.
    pub plugin_key: String,
    /// Namespace name.
    pub namespace_name: String,
    /// Namespace mode (`schema` | `table`).
    pub namespace_mode: String,
    /// Status.
    pub status: String,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// A row of the `plugin_migrations` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginMigrationRecord {
    /// Migration id.
    pub id: String,
    /// Plugin id.
    pub plugin_id: String,
    /// Plugin key.
    pub plugin_key: String,
    /// Namespace name.
    pub namespace_name: String,
    /// Migration key.
    pub migration_key: String,
    /// Checksum.
    pub checksum: String,
    /// Plugin version.
    pub plugin_version: String,
    /// Status (`applied` | `failed` | `pending`).
    pub status: String,
    /// ISO 8601 start.
    pub started_at: String,
    /// ISO 8601 applied.
    pub applied_at: Option<String>,
    /// Error message.
    pub error_message: Option<String>,
}

/// Plugin runtime repository errors.
#[derive(Debug, Error)]
pub enum PluginRuntimeError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The plugin does not exist.
    #[error("plugin not found")]
    PluginNotFound,
    /// The parent job does not exist.
    #[error("job not found")]
    JobNotFound,
    /// The row does not exist.
    #[error("not found")]
    NotFound,
    /// The run is already terminal.
    #[error("run already terminal")]
    RunTerminal,
}

/// Plugin runtime persistence contract.
#[async_trait]
pub trait PluginRuntimeRepository: Send + Sync {
    // State ---------------------------------------------------------------
    async fn state_get(
        &self,
        plugin_id: &str,
        scope_kind: &str,
        scope_id: Option<&str>,
        namespace: &str,
        state_key: &str,
    ) -> Result<Option<PluginStateRecord>, PluginRuntimeError>;
    async fn state_set(
        &self,
        plugin_id: &str,
        scope_kind: &str,
        scope_id: Option<&str>,
        namespace: &str,
        state_key: &str,
        value: serde_json::Value,
    ) -> Result<PluginStateRecord, PluginRuntimeError>;
    async fn state_delete(
        &self,
        plugin_id: &str,
        scope_kind: &str,
        scope_id: Option<&str>,
        namespace: &str,
        state_key: &str,
    ) -> Result<Option<PluginStateRecord>, PluginRuntimeError>;
    async fn state_list(
        &self,
        plugin_id: &str,
        scope_kind: &str,
        scope_id: Option<&str>,
        namespace: &str,
    ) -> Result<Vec<PluginStateRecord>, PluginRuntimeError>;

    // Entities -------------------------------------------------------------
    async fn entity_upsert(
        &self,
        input: NewPluginEntity,
    ) -> Result<PluginEntityRecord, PluginRuntimeError>;
    async fn entity_list(
        &self,
        plugin_id: &str,
        company_id: Option<&str>,
    ) -> Result<Vec<PluginEntityRecord>, PluginRuntimeError>;
    async fn entity_delete(
        &self,
        plugin_id: &str,
        id: &str,
    ) -> Result<Option<PluginEntityRecord>, PluginRuntimeError>;

    // Jobs -----------------------------------------------------------------
    async fn job_upsert(&self, input: NewPluginJob) -> Result<PluginJobRecord, PluginRuntimeError>;
    async fn job_list(&self, plugin_id: &str) -> Result<Vec<PluginJobRecord>, PluginRuntimeError>;
    async fn job_update(
        &self,
        plugin_id: &str,
        job_key: &str,
        schedule: Option<String>,
        status: Option<String>,
    ) -> Result<Option<PluginJobRecord>, PluginRuntimeError>;
    async fn job_run_create(
        &self,
        input: NewPluginJobRun,
    ) -> Result<PluginJobRunRecord, PluginRuntimeError>;
    async fn job_run_complete(
        &self,
        plugin_id: &str,
        run_id: &str,
        status: &str,
        error: Option<String>,
        logs: Vec<String>,
    ) -> Result<Option<PluginJobRunRecord>, PluginRuntimeError>;
    async fn job_run_list(
        &self,
        plugin_id: &str,
        company_id: Option<&str>,
    ) -> Result<Vec<PluginJobRunRecord>, PluginRuntimeError>;

    // Logs -----------------------------------------------------------------
    async fn log_append(&self, input: NewPluginLog) -> Result<PluginLogRecord, PluginRuntimeError>;
    async fn log_list(
        &self,
        plugin_id: &str,
        company_id: Option<&str>,
    ) -> Result<Vec<PluginLogRecord>, PluginRuntimeError>;

    // Webhooks -------------------------------------------------------------
    async fn webhook_create(
        &self,
        input: NewPluginWebhook,
    ) -> Result<PluginWebhookDeliveryRecord, PluginRuntimeError>;
    async fn webhook_complete(
        &self,
        plugin_id: &str,
        id: &str,
        status: &str,
        error: Option<String>,
        duration_ms: Option<i64>,
    ) -> Result<Option<PluginWebhookDeliveryRecord>, PluginRuntimeError>;
    async fn webhook_list(
        &self,
        plugin_id: &str,
        company_id: Option<&str>,
    ) -> Result<Vec<PluginWebhookDeliveryRecord>, PluginRuntimeError>;

    // Database namespaces + migrations --------------------------------------
    async fn namespace_upsert(
        &self,
        input: NewPluginNamespace,
    ) -> Result<PluginDatabaseNamespaceRecord, PluginRuntimeError>;
    async fn namespace_list(
        &self,
        plugin_id: &str,
    ) -> Result<Vec<PluginDatabaseNamespaceRecord>, PluginRuntimeError>;
    async fn migration_record(
        &self,
        input: NewPluginMigration,
    ) -> Result<PluginMigrationRecord, PluginRuntimeError>;
    async fn migration_list(
        &self,
        plugin_id: &str,
    ) -> Result<Vec<PluginMigrationRecord>, PluginRuntimeError>;
}

/// Input for upserting an entity.
#[derive(Debug, Clone)]
pub struct NewPluginEntity {
    pub plugin_id: String,
    pub company_id: Option<String>,
    pub entity_type: String,
    pub scope_kind: String,
    pub scope_id: Option<String>,
    pub external_id: Option<String>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub data: serde_json::Value,
}

/// Input for upserting a job.
#[derive(Debug, Clone)]
pub struct NewPluginJob {
    pub plugin_id: String,
    pub job_key: String,
    pub schedule: String,
}

/// Input for creating a job run.
#[derive(Debug, Clone)]
pub struct NewPluginJobRun {
    pub job_id: String,
    pub plugin_id: String,
    pub company_id: Option<String>,
    pub trigger: String,
}

/// Input for appending a log.
#[derive(Debug, Clone)]
pub struct NewPluginLog {
    pub plugin_id: String,
    pub company_id: Option<String>,
    pub level: String,
    pub message: String,
    pub meta: Option<serde_json::Value>,
}

/// Input for creating a webhook delivery.
#[derive(Debug, Clone)]
pub struct NewPluginWebhook {
    pub plugin_id: String,
    pub company_id: Option<String>,
    pub webhook_key: String,
    pub external_id: Option<String>,
    pub payload: serde_json::Value,
    pub headers: serde_json::Value,
}

/// Input for upserting a database namespace.
#[derive(Debug, Clone)]
pub struct NewPluginNamespace {
    pub plugin_id: String,
    pub plugin_key: String,
    pub namespace_name: String,
    pub namespace_mode: String,
}

/// Input for recording a migration.
#[derive(Debug, Clone)]
pub struct NewPluginMigration {
    pub plugin_id: String,
    pub plugin_key: String,
    pub namespace_name: String,
    pub migration_key: String,
    pub checksum: String,
    pub plugin_version: String,
    pub status: String,
    pub error_message: Option<String>,
}

/// Turso/libSQL implementation of [`PluginRuntimeRepository`].
#[derive(Debug)]
pub struct TursoPluginRuntimeRepository {
    db: Database,
}

impl TursoPluginRuntimeRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

fn row_to_state(row: &libsql::Row) -> Result<PluginStateRecord, libsql::Error> {
    Ok(PluginStateRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        plugin_id: helpers::row_text(row, 1)?.expect("plugin_id"),
        scope_kind: helpers::row_text(row, 2)?.expect("scope_kind"),
        scope_id: helpers::row_text(row, 3)?,
        namespace: helpers::row_text(row, 4)?.expect("namespace"),
        state_key: helpers::row_text(row, 5)?.expect("state_key"),
        value_json: helpers::row_text(row, 6)?
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or(serde_json::Value::Null),
        updated_at: helpers::row_text(row, 7)?.expect("updated_at"),
    })
}

fn row_to_entity(row: &libsql::Row) -> Result<PluginEntityRecord, libsql::Error> {
    Ok(PluginEntityRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        plugin_id: helpers::row_text(row, 1)?.expect("plugin_id"),
        company_id: helpers::row_text(row, 2)?,
        entity_type: helpers::row_text(row, 3)?.expect("entity_type"),
        scope_kind: helpers::row_text(row, 4)?.expect("scope_kind"),
        scope_id: helpers::row_text(row, 5)?,
        external_id: helpers::row_text(row, 6)?,
        title: helpers::row_text(row, 7)?,
        status: helpers::row_text(row, 8)?,
        data: helpers::row_text(row, 9)?
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or(serde_json::Value::Null),
        created_at: helpers::row_text(row, 10)?.expect("created_at"),
    })
}

fn row_to_job(row: &libsql::Row) -> Result<PluginJobRecord, libsql::Error> {
    Ok(PluginJobRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        plugin_id: helpers::row_text(row, 1)?.expect("plugin_id"),
        job_key: helpers::row_text(row, 2)?.expect("job_key"),
        schedule: helpers::row_text(row, 3)?.expect("schedule"),
        status: helpers::row_text(row, 4)?.expect("status"),
        last_run_at: helpers::row_text(row, 5)?,
        next_run_at: helpers::row_text(row, 6)?,
        created_at: helpers::row_text(row, 7)?.expect("created_at"),
    })
}

fn row_to_job_run(row: &libsql::Row) -> Result<PluginJobRunRecord, libsql::Error> {
    Ok(PluginJobRunRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        job_id: helpers::row_text(row, 1)?.expect("job_id"),
        plugin_id: helpers::row_text(row, 2)?.expect("plugin_id"),
        company_id: helpers::row_text(row, 3)?,
        trigger: helpers::row_text(row, 4)?.expect("trigger"),
        status: helpers::row_text(row, 5)?.expect("status"),
        duration_ms: helpers::row_i64_opt(row, 6)?,
        error: helpers::row_text(row, 7)?,
        logs: helpers::row_text(row, 8)?
            .map(|raw| serde_json::from_str::<Vec<String>>(&raw).unwrap_or_default())
            .unwrap_or_default(),
        started_at: helpers::row_text(row, 9)?,
        finished_at: helpers::row_text(row, 10)?,
        created_at: helpers::row_text(row, 11)?.expect("created_at"),
    })
}

fn row_to_log(row: &libsql::Row) -> Result<PluginLogRecord, libsql::Error> {
    Ok(PluginLogRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        plugin_id: helpers::row_text(row, 1)?.expect("plugin_id"),
        company_id: helpers::row_text(row, 2)?,
        level: helpers::row_text(row, 3)?.expect("level"),
        message: helpers::row_text(row, 4)?.expect("message"),
        meta: helpers::row_text(row, 5)?.and_then(|raw| serde_json::from_str(&raw).ok()),
        created_at: helpers::row_text(row, 6)?.expect("created_at"),
    })
}

fn row_to_webhook(row: &libsql::Row) -> Result<PluginWebhookDeliveryRecord, libsql::Error> {
    Ok(PluginWebhookDeliveryRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        plugin_id: helpers::row_text(row, 1)?.expect("plugin_id"),
        company_id: helpers::row_text(row, 2)?,
        webhook_key: helpers::row_text(row, 3)?.expect("webhook_key"),
        external_id: helpers::row_text(row, 4)?,
        status: helpers::row_text(row, 5)?.expect("status"),
        duration_ms: helpers::row_i64_opt(row, 6)?,
        error: helpers::row_text(row, 7)?,
        payload: helpers::row_text(row, 8)?
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or(serde_json::Value::Null),
        headers: helpers::row_text(row, 9)?
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or(serde_json::Value::Null),
        started_at: helpers::row_text(row, 10)?,
        finished_at: helpers::row_text(row, 11)?,
        created_at: helpers::row_text(row, 12)?.expect("created_at"),
    })
}

fn row_to_namespace(row: &libsql::Row) -> Result<PluginDatabaseNamespaceRecord, libsql::Error> {
    Ok(PluginDatabaseNamespaceRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        plugin_id: helpers::row_text(row, 1)?.expect("plugin_id"),
        plugin_key: helpers::row_text(row, 2)?.expect("plugin_key"),
        namespace_name: helpers::row_text(row, 3)?.expect("namespace_name"),
        namespace_mode: helpers::row_text(row, 4)?.expect("namespace_mode"),
        status: helpers::row_text(row, 5)?.expect("status"),
        created_at: helpers::row_text(row, 6)?.expect("created_at"),
    })
}

fn row_to_migration(row: &libsql::Row) -> Result<PluginMigrationRecord, libsql::Error> {
    Ok(PluginMigrationRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        plugin_id: helpers::row_text(row, 1)?.expect("plugin_id"),
        plugin_key: helpers::row_text(row, 2)?.expect("plugin_key"),
        namespace_name: helpers::row_text(row, 3)?.expect("namespace_name"),
        migration_key: helpers::row_text(row, 4)?.expect("migration_key"),
        checksum: helpers::row_text(row, 5)?.expect("checksum"),
        plugin_version: helpers::row_text(row, 6)?.expect("plugin_version"),
        status: helpers::row_text(row, 7)?.expect("status"),
        started_at: helpers::row_text(row, 8)?.expect("started_at"),
        applied_at: helpers::row_text(row, 9)?,
        error_message: helpers::row_text(row, 10)?,
    })
}

const STATE_COLUMNS: &str = "id, plugin_id, scope_kind, scope_id, namespace, state_key,
    value_json, updated_at";
const ENTITY_COLUMNS: &str = "id, plugin_id, company_id, entity_type, scope_kind, scope_id,
    external_id, title, status, data, created_at";
const JOB_COLUMNS: &str = "id, plugin_id, job_key, schedule, status, last_run_at, next_run_at,
    created_at";
const JOB_RUN_COLUMNS: &str = "id, job_id, plugin_id, company_id, trigger, status, duration_ms,
    error, logs, started_at, finished_at, created_at";
const LOG_COLUMNS: &str = "id, plugin_id, company_id, level, message, meta, created_at";
const WEBHOOK_COLUMNS: &str = "id, plugin_id, company_id, webhook_key, external_id, status,
    duration_ms, error, payload, headers, started_at, finished_at, created_at";
const NAMESPACE_COLUMNS: &str = "id, plugin_id, plugin_key, namespace_name, namespace_mode,
    status, created_at";
const MIGRATION_COLUMNS: &str = "id, plugin_id, plugin_key, namespace_name, migration_key,
    checksum, plugin_version, status, started_at, applied_at, error_message";

async fn ensure_plugin(
    conn: &libsql::Connection,
    plugin_id: &str,
) -> Result<(), PluginRuntimeError> {
    if !helpers::find_row(conn, "plugins", plugin_id).await? {
        return Err(PluginRuntimeError::PluginNotFound);
    }
    Ok(())
}

#[async_trait]
impl PluginRuntimeRepository for TursoPluginRuntimeRepository {
    async fn state_get(
        &self,
        plugin_id: &str,
        scope_kind: &str,
        scope_id: Option<&str>,
        namespace: &str,
        state_key: &str,
    ) -> Result<Option<PluginStateRecord>, PluginRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {STATE_COLUMNS} FROM plugin_state
                     WHERE plugin_id = ?1 AND scope_kind = ?2 AND scope_id IS ?3
                       AND namespace = ?4 AND state_key = ?5"
                ),
                libsql::params![plugin_id, scope_kind, scope_id, namespace, state_key],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_state(&row)?)),
            None => Ok(None),
        }
    }

    async fn state_set(
        &self,
        plugin_id: &str,
        scope_kind: &str,
        scope_id: Option<&str>,
        namespace: &str,
        state_key: &str,
        value: serde_json::Value,
    ) -> Result<PluginStateRecord, PluginRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        ensure_plugin(&conn, plugin_id).await?;
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO plugin_state
               (id, plugin_id, scope_kind, scope_id, namespace, state_key, value_json, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT (plugin_id, scope_kind, scope_id, namespace, state_key)
             DO UPDATE SET value_json = excluded.value_json,
                           updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            libsql::params![
                id.clone(),
                plugin_id,
                scope_kind,
                scope_id,
                namespace,
                state_key,
                value.to_string()
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {STATE_COLUMNS} FROM plugin_state
                     WHERE plugin_id = ?1 AND scope_kind = ?2 AND scope_id IS ?3
                       AND namespace = ?4 AND state_key = ?5"
                ),
                libsql::params![plugin_id, scope_kind, scope_id, namespace, state_key],
            )
            .await?;
        let row = rows.next().await?.expect("state was just upserted");
        Ok(row_to_state(&row)?)
    }

    async fn state_delete(
        &self,
        plugin_id: &str,
        scope_kind: &str,
        scope_id: Option<&str>,
        namespace: &str,
        state_key: &str,
    ) -> Result<Option<PluginStateRecord>, PluginRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {STATE_COLUMNS} FROM plugin_state
                     WHERE plugin_id = ?1 AND scope_kind = ?2 AND scope_id IS ?3
                       AND namespace = ?4 AND state_key = ?5"
                ),
                libsql::params![plugin_id, scope_kind, scope_id, namespace, state_key],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let record = row_to_state(&row)?;
        conn.execute(
            "DELETE FROM plugin_state WHERE id = ?1",
            libsql::params![record.id.clone()],
        )
        .await?;
        Ok(Some(record))
    }

    async fn state_list(
        &self,
        plugin_id: &str,
        scope_kind: &str,
        scope_id: Option<&str>,
        namespace: &str,
    ) -> Result<Vec<PluginStateRecord>, PluginRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {STATE_COLUMNS} FROM plugin_state
                     WHERE plugin_id = ?1 AND scope_kind = ?2 AND scope_id IS ?3 AND namespace = ?4
                     ORDER BY state_key"
                ),
                libsql::params![plugin_id, scope_kind, scope_id, namespace],
            )
            .await?;
        let mut states = Vec::new();
        while let Some(row) = rows.next().await? {
            states.push(row_to_state(&row)?);
        }
        Ok(states)
    }

    async fn entity_upsert(
        &self,
        input: NewPluginEntity,
    ) -> Result<PluginEntityRecord, PluginRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        ensure_plugin(&conn, &input.plugin_id).await?;
        if let Some(company_id) = &input.company_id
            && !helpers::company_exists(&conn, company_id).await?
        {
            return Err(PluginRuntimeError::NotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO plugin_entities
               (id, plugin_id, company_id, entity_type, scope_kind, scope_id, external_id,
                title, status, data, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT (company_id, plugin_id, entity_type, external_id)
             DO UPDATE SET scope_kind = excluded.scope_kind,
                           scope_id = excluded.scope_id,
                           title = excluded.title,
                           status = excluded.status,
                           data = excluded.data,
                           updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            libsql::params![
                id.clone(),
                input.plugin_id.clone(),
                input.company_id.clone(),
                input.entity_type.clone(),
                input.scope_kind.clone(),
                input.scope_id.clone(),
                input.external_id.clone(),
                input.title.clone(),
                input.status.clone(),
                input.data.to_string()
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {ENTITY_COLUMNS} FROM plugin_entities
                     WHERE company_id IS ?1 AND plugin_id = ?2 AND entity_type = ?3 AND external_id IS ?4"
                ),
                libsql::params![input.company_id, input.plugin_id, input.entity_type, input.external_id],
            )
            .await?;
        let row = rows.next().await?.expect("entity was just upserted");
        Ok(row_to_entity(&row)?)
    }

    async fn entity_list(
        &self,
        plugin_id: &str,
        company_id: Option<&str>,
    ) -> Result<Vec<PluginEntityRecord>, PluginRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {ENTITY_COLUMNS} FROM plugin_entities
                     WHERE plugin_id = ?1 AND company_id IS ?2 ORDER BY entity_type, external_id"
                ),
                libsql::params![plugin_id, company_id],
            )
            .await?;
        let mut entities = Vec::new();
        while let Some(row) = rows.next().await? {
            entities.push(row_to_entity(&row)?);
        }
        Ok(entities)
    }

    async fn entity_delete(
        &self,
        plugin_id: &str,
        id: &str,
    ) -> Result<Option<PluginEntityRecord>, PluginRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {ENTITY_COLUMNS} FROM plugin_entities WHERE plugin_id = ?1 AND id = ?2"
                ),
                libsql::params![plugin_id, id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let record = row_to_entity(&row)?;
        conn.execute(
            "DELETE FROM plugin_entities WHERE id = ?1",
            libsql::params![id],
        )
        .await?;
        Ok(Some(record))
    }

    async fn job_upsert(&self, input: NewPluginJob) -> Result<PluginJobRecord, PluginRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        ensure_plugin(&conn, &input.plugin_id).await?;
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO plugin_jobs (id, plugin_id, job_key, schedule, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT (plugin_id, job_key)
             DO UPDATE SET schedule = excluded.schedule,
                           updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            libsql::params![
                id.clone(),
                input.plugin_id.clone(),
                input.job_key.clone(),
                input.schedule
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {JOB_COLUMNS} FROM plugin_jobs WHERE plugin_id = ?1 AND job_key = ?2"
                ),
                libsql::params![input.plugin_id, input.job_key],
            )
            .await?;
        let row = rows.next().await?.expect("job was just upserted");
        Ok(row_to_job(&row)?)
    }

    async fn job_list(&self, plugin_id: &str) -> Result<Vec<PluginJobRecord>, PluginRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {JOB_COLUMNS} FROM plugin_jobs WHERE plugin_id = ?1 ORDER BY job_key"
                ),
                libsql::params![plugin_id],
            )
            .await?;
        let mut jobs = Vec::new();
        while let Some(row) = rows.next().await? {
            jobs.push(row_to_job(&row)?);
        }
        Ok(jobs)
    }

    async fn job_update(
        &self,
        plugin_id: &str,
        job_key: &str,
        schedule: Option<String>,
        status: Option<String>,
    ) -> Result<Option<PluginJobRecord>, PluginRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut sets = Vec::new();
        let mut values: Vec<libsql::Value> = Vec::new();
        let mut param = 0usize;
        if let Some(schedule) = schedule {
            param += 1;
            sets.push(format!("schedule = ?{param}"));
            values.push(libsql::Value::from(schedule));
        }
        if let Some(status) = status {
            param += 1;
            sets.push(format!("status = ?{param}"));
            values.push(libsql::Value::from(status));
        }
        if sets.is_empty() {
            return Err(PluginRuntimeError::NotFound);
        }
        let plugin_param = param + 1;
        let key_param = param + 2;
        values.push(libsql::Value::from(plugin_id.to_owned()));
        values.push(libsql::Value::from(job_key.to_owned()));
        let updated = conn
            .execute(
                &format!(
                    "UPDATE plugin_jobs SET {}, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                     WHERE plugin_id = ?{plugin_param} AND job_key = ?{key_param}",
                    sets.join(", ")
                ),
                values,
            )
            .await?;
        if updated == 0 {
            return Ok(None);
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {JOB_COLUMNS} FROM plugin_jobs WHERE plugin_id = ?1 AND job_key = ?2"
                ),
                libsql::params![plugin_id, job_key],
            )
            .await?;
        let row = rows.next().await?.expect("job exists");
        Ok(Some(row_to_job(&row)?))
    }

    async fn job_run_create(
        &self,
        input: NewPluginJobRun,
    ) -> Result<PluginJobRunRecord, PluginRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut job_rows = conn
            .query(
                "SELECT id, plugin_id, job_key FROM plugin_jobs WHERE id = ?1 AND plugin_id = ?2",
                libsql::params![input.job_id.clone(), input.plugin_id.clone()],
            )
            .await?;
        if job_rows.next().await?.is_none() {
            return Err(PluginRuntimeError::JobNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO plugin_job_runs
               (id, job_id, plugin_id, company_id, trigger, status, started_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'pending',
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.job_id,
                input.plugin_id,
                input.company_id,
                input.trigger
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {JOB_RUN_COLUMNS} FROM plugin_job_runs WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("run was just inserted");
        Ok(row_to_job_run(&row)?)
    }

    async fn job_run_complete(
        &self,
        plugin_id: &str,
        run_id: &str,
        status: &str,
        error: Option<String>,
        logs: Vec<String>,
    ) -> Result<Option<PluginJobRunRecord>, PluginRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {JOB_RUN_COLUMNS} FROM plugin_job_runs WHERE plugin_id = ?1 AND id = ?2"
                ),
                libsql::params![plugin_id, run_id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let run = row_to_job_run(&row)?;
        if matches!(run.status.as_str(), "succeeded" | "failed" | "cancelled") {
            return Err(PluginRuntimeError::RunTerminal);
        }
        let logs_json = serde_json::to_string(&logs).unwrap_or_else(|_| "[]".to_owned());
        conn.execute(
            "UPDATE plugin_job_runs SET status = ?1, error = ?2, logs = ?3,
                    duration_ms = CAST((julianday('now') - julianday(started_at)) * 86400000 AS INTEGER),
                    finished_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?4",
            libsql::params![status, error, logs_json, run_id],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {JOB_RUN_COLUMNS} FROM plugin_job_runs WHERE id = ?1"),
                libsql::params![run_id],
            )
            .await?;
        let row = rows.next().await?.expect("run exists");
        Ok(Some(row_to_job_run(&row)?))
    }

    async fn job_run_list(
        &self,
        plugin_id: &str,
        company_id: Option<&str>,
    ) -> Result<Vec<PluginJobRunRecord>, PluginRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {JOB_RUN_COLUMNS} FROM plugin_job_runs
                     WHERE plugin_id = ?1 AND company_id IS ?2 ORDER BY created_at DESC"
                ),
                libsql::params![plugin_id, company_id],
            )
            .await?;
        let mut runs = Vec::new();
        while let Some(row) = rows.next().await? {
            runs.push(row_to_job_run(&row)?);
        }
        Ok(runs)
    }

    async fn log_append(&self, input: NewPluginLog) -> Result<PluginLogRecord, PluginRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        ensure_plugin(&conn, &input.plugin_id).await?;
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO plugin_logs (id, plugin_id, company_id, level, message, meta, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.plugin_id,
                input.company_id,
                input.level,
                input.message,
                input.meta.map(|value| value.to_string())
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {LOG_COLUMNS} FROM plugin_logs WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("log was just inserted");
        Ok(row_to_log(&row)?)
    }

    async fn log_list(
        &self,
        plugin_id: &str,
        company_id: Option<&str>,
    ) -> Result<Vec<PluginLogRecord>, PluginRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {LOG_COLUMNS} FROM plugin_logs
                     WHERE plugin_id = ?1 AND company_id IS ?2 ORDER BY created_at DESC LIMIT 200"
                ),
                libsql::params![plugin_id, company_id],
            )
            .await?;
        let mut logs = Vec::new();
        while let Some(row) = rows.next().await? {
            logs.push(row_to_log(&row)?);
        }
        Ok(logs)
    }

    async fn webhook_create(
        &self,
        input: NewPluginWebhook,
    ) -> Result<PluginWebhookDeliveryRecord, PluginRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        ensure_plugin(&conn, &input.plugin_id).await?;
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO plugin_webhook_deliveries
               (id, plugin_id, company_id, webhook_key, external_id, status, payload, headers,
                started_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'processing', ?6, ?7,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.plugin_id,
                input.company_id,
                input.webhook_key,
                input.external_id,
                input.payload.to_string(),
                input.headers.to_string()
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {WEBHOOK_COLUMNS} FROM plugin_webhook_deliveries WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("webhook was just inserted");
        Ok(row_to_webhook(&row)?)
    }

    async fn webhook_complete(
        &self,
        plugin_id: &str,
        id: &str,
        status: &str,
        error: Option<String>,
        duration_ms: Option<i64>,
    ) -> Result<Option<PluginWebhookDeliveryRecord>, PluginRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let updated = conn
            .execute(
                "UPDATE plugin_webhook_deliveries SET status = ?1, error = ?2,
                        duration_ms = COALESCE(?3, duration_ms),
                        finished_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE plugin_id = ?4 AND id = ?5 AND status = 'processing'",
                libsql::params![status, error, duration_ms, plugin_id, id],
            )
            .await?;
        if updated == 0 {
            return Ok(None);
        }
        let mut rows = conn
            .query(
                &format!("SELECT {WEBHOOK_COLUMNS} FROM plugin_webhook_deliveries WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("delivery exists");
        Ok(Some(row_to_webhook(&row)?))
    }

    async fn webhook_list(
        &self,
        plugin_id: &str,
        company_id: Option<&str>,
    ) -> Result<Vec<PluginWebhookDeliveryRecord>, PluginRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {WEBHOOK_COLUMNS} FROM plugin_webhook_deliveries
                     WHERE plugin_id = ?1 AND company_id IS ?2 ORDER BY created_at DESC LIMIT 200"
                ),
                libsql::params![plugin_id, company_id],
            )
            .await?;
        let mut webhooks = Vec::new();
        while let Some(row) = rows.next().await? {
            webhooks.push(row_to_webhook(&row)?);
        }
        Ok(webhooks)
    }

    async fn namespace_upsert(
        &self,
        input: NewPluginNamespace,
    ) -> Result<PluginDatabaseNamespaceRecord, PluginRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        ensure_plugin(&conn, &input.plugin_id).await?;
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO plugin_database_namespaces
               (id, plugin_id, plugin_key, namespace_name, namespace_mode, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'active',
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT (plugin_id)
             DO UPDATE SET namespace_name = excluded.namespace_name,
                           namespace_mode = excluded.namespace_mode,
                           status = 'active',
                           updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            libsql::params![
                id.clone(),
                input.plugin_id.clone(),
                input.plugin_key.clone(),
                input.namespace_name.clone(),
                input.namespace_mode.clone()
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {NAMESPACE_COLUMNS} FROM plugin_database_namespaces WHERE plugin_id = ?1"
                ),
                libsql::params![input.plugin_id],
            )
            .await?;
        let row = rows.next().await?.expect("namespace was just upserted");
        Ok(row_to_namespace(&row)?)
    }

    async fn namespace_list(
        &self,
        plugin_id: &str,
    ) -> Result<Vec<PluginDatabaseNamespaceRecord>, PluginRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {NAMESPACE_COLUMNS} FROM plugin_database_namespaces WHERE plugin_id = ?1"
                ),
                libsql::params![plugin_id],
            )
            .await?;
        let mut namespaces = Vec::new();
        while let Some(row) = rows.next().await? {
            namespaces.push(row_to_namespace(&row)?);
        }
        Ok(namespaces)
    }

    async fn migration_record(
        &self,
        input: NewPluginMigration,
    ) -> Result<PluginMigrationRecord, PluginRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        ensure_plugin(&conn, &input.plugin_id).await?;
        let id = Uuid::new_v4().to_string();
        let applied_at = if input.status == "applied" {
            Some("strftime('%Y-%m-%dT%H:%M:%fZ','now')".to_owned())
        } else {
            None
        };
        let sql = if let Some(applied_at) = applied_at {
            format!(
                "INSERT INTO plugin_migrations
                   (id, plugin_id, plugin_key, namespace_name, migration_key, checksum,
                    plugin_version, status, started_at, applied_at, error_message)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'), {applied_at}, ?9)
                 ON CONFLICT (plugin_id, migration_key)
                 DO UPDATE SET checksum = excluded.checksum,
                               plugin_version = excluded.plugin_version,
                               status = excluded.status,
                               applied_at = {applied_at},
                               error_message = excluded.error_message"
            )
        } else {
            "INSERT INTO plugin_migrations
               (id, plugin_id, plugin_key, namespace_name, migration_key, checksum,
                plugin_version, status, started_at, applied_at, error_message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'), NULL, ?9)
             ON CONFLICT (plugin_id, migration_key)
             DO UPDATE SET checksum = excluded.checksum,
                           plugin_version = excluded.plugin_version,
                           status = excluded.status,
                           applied_at = NULL,
                           error_message = excluded.error_message"
                .to_owned()
        };
        conn.execute(
            &sql,
            libsql::params![
                id.clone(),
                input.plugin_id.clone(),
                input.plugin_key.clone(),
                input.namespace_name.clone(),
                input.migration_key.clone(),
                input.checksum.clone(),
                input.plugin_version.clone(),
                input.status.clone(),
                input.error_message.clone()
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {MIGRATION_COLUMNS} FROM plugin_migrations WHERE plugin_id = ?1 AND migration_key = ?2"
                ),
                libsql::params![input.plugin_id, input.migration_key],
            )
            .await?;
        let row = rows.next().await?.expect("migration was just upserted");
        Ok(row_to_migration(&row)?)
    }

    async fn migration_list(
        &self,
        plugin_id: &str,
    ) -> Result<Vec<PluginMigrationRecord>, PluginRuntimeError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {MIGRATION_COLUMNS} FROM plugin_migrations WHERE plugin_id = ?1 ORDER BY migration_key"
                ),
                libsql::params![plugin_id],
            )
            .await?;
        let mut migrations = Vec::new();
        while let Some(row) = rows.next().await? {
            migrations.push(row_to_migration(&row)?);
        }
        Ok(migrations)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{
        migrate, open,
        repositories::plugins::{NewPlugin, PluginRepository, TursoPluginRepository},
    };

    async fn repo() -> (TempDir, TursoPluginRuntimeRepository, TursoPluginRepository) {
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
        let plugins = TursoPluginRepository::new(
            open(&crate::DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        );
        let runtime = TursoPluginRuntimeRepository::new(db);
        (dir, runtime, plugins)
    }

    #[tokio::test]
    async fn state_entities_jobs_logs_webhooks_namespaces_migrations() {
        let (_dir, runtime, plugins) = repo().await;
        let plugin = plugins
            .register(NewPlugin {
                plugin_key: "acme.runtime".to_owned(),
                package_name: "@acme/runtime".to_owned(),
                version: "1.0.0".to_owned(),
                api_version: 1,
                categories: Vec::new(),
                manifest_json: serde_json::json!({ "id": "acme.runtime" }),
                install_order: None,
                package_path: None,
            })
            .await
            .unwrap();

        // State.
        let state = runtime
            .state_set(
                &plugin.id,
                "company",
                Some("c1"),
                "default",
                "k",
                serde_json::json!({"a":1}),
            )
            .await
            .unwrap();
        assert_eq!(state.value_json["a"], 1);
        let got = runtime
            .state_get(&plugin.id, "company", Some("c1"), "default", "k")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(got.id, state.id);
        assert!(
            runtime
                .state_delete(&plugin.id, "company", Some("c1"), "default", "k")
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            runtime
                .state_get(&plugin.id, "company", Some("c1"), "default", "k")
                .await
                .unwrap()
                .is_none()
        );

        // Entities.
        let entity = runtime
            .entity_upsert(NewPluginEntity {
                plugin_id: plugin.id.clone(),
                company_id: Some("c1".to_owned()),
                entity_type: "issue".to_owned(),
                scope_kind: "issue".to_owned(),
                scope_id: Some("i1".to_owned()),
                external_id: Some("LIN-1".to_owned()),
                title: Some("T".to_owned()),
                status: Some("open".to_owned()),
                data: serde_json::json!({}),
            })
            .await
            .unwrap();
        assert_eq!(entity.external_id.as_deref(), Some("LIN-1"));
        assert!(
            runtime
                .entity_list(&plugin.id, Some("c1"))
                .await
                .unwrap()
                .len()
                == 1
        );
        assert!(
            runtime
                .entity_delete(&plugin.id, &entity.id)
                .await
                .unwrap()
                .is_some()
        );

        // Jobs + runs.
        let job = runtime
            .job_upsert(NewPluginJob {
                plugin_id: plugin.id.clone(),
                job_key: "nightly".to_owned(),
                schedule: "0 0 * * *".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(job.schedule, "0 0 * * *");
        let run = runtime
            .job_run_create(NewPluginJobRun {
                job_id: job.id.clone(),
                plugin_id: plugin.id.clone(),
                company_id: Some("c1".to_owned()),
                trigger: "manual".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(run.status, "pending");
        let completed = runtime
            .job_run_complete(
                &plugin.id,
                &run.id,
                "succeeded",
                None,
                vec!["ok".to_owned()],
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(completed.status, "succeeded");
        assert_eq!(completed.logs, vec!["ok"]);
        assert!(matches!(
            runtime
                .job_run_complete(&plugin.id, &run.id, "failed", None, Vec::new())
                .await
                .unwrap_err(),
            PluginRuntimeError::RunTerminal
        ));
        assert!(
            runtime
                .job_run_list(&plugin.id, Some("c1"))
                .await
                .unwrap()
                .len()
                == 1
        );

        // Logs.
        runtime
            .log_append(NewPluginLog {
                plugin_id: plugin.id.clone(),
                company_id: Some("c1".to_owned()),
                level: "info".to_owned(),
                message: "hello".to_owned(),
                meta: None,
            })
            .await
            .unwrap();
        assert!(
            runtime
                .log_list(&plugin.id, Some("c1"))
                .await
                .unwrap()
                .len()
                == 1
        );

        // Webhooks.
        let webhook = runtime
            .webhook_create(NewPluginWebhook {
                plugin_id: plugin.id.clone(),
                company_id: Some("c1".to_owned()),
                webhook_key: "issue.created".to_owned(),
                external_id: Some("evt-1".to_owned()),
                payload: serde_json::json!({ "id": 1 }),
                headers: serde_json::json!({ "x-signature": "sig" }),
            })
            .await
            .unwrap();
        assert_eq!(webhook.status, "processing");
        let completed = runtime
            .webhook_complete(&plugin.id, &webhook.id, "succeeded", None, Some(12))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(completed.status, "succeeded");
        assert_eq!(completed.duration_ms, Some(12));

        // Namespaces + migrations.
        let ns = runtime
            .namespace_upsert(NewPluginNamespace {
                plugin_id: plugin.id.clone(),
                plugin_key: "acme.runtime".to_owned(),
                namespace_name: "acme_runtime".to_owned(),
                namespace_mode: "schema".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(ns.status, "active");
        let migration = runtime
            .migration_record(NewPluginMigration {
                plugin_id: plugin.id.clone(),
                plugin_key: "acme.runtime".to_owned(),
                namespace_name: "acme_runtime".to_owned(),
                migration_key: "0001_init".to_owned(),
                checksum: "abc123".to_owned(),
                plugin_version: "1.0.0".to_owned(),
                status: "applied".to_owned(),
                error_message: None,
            })
            .await
            .unwrap();
        assert_eq!(migration.status, "applied");
        assert!(runtime.migration_list(&plugin.id).await.unwrap().len() == 1);
        assert!(runtime.namespace_list(&plugin.id).await.unwrap().len() == 1);

        // Unknown plugin rejected.
        assert!(matches!(
            runtime
                .state_set(
                    "missing",
                    "company",
                    Some("c1"),
                    "default",
                    "k",
                    serde_json::json!({})
                )
                .await
                .unwrap_err(),
            PluginRuntimeError::PluginNotFound
        ));
    }
}
