//! Pipelines repository (upstream pipelines.ts family, core subset).

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A pipeline definition.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineRecord {
    /// Pipeline id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Project id.
    pub project_id: Option<String>,
    /// Key (unique per company).
    pub key: String,
    /// Name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Enforce transitions flag.
    pub enforce_transitions: bool,
    /// Creating user id.
    pub created_by_user_id: Option<String>,
    /// ISO 8601 archive time.
    pub archived_at: Option<String>,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// A pipeline stage.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineStageRecord {
    /// Stage id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Pipeline id.
    pub pipeline_id: String,
    /// Key (unique per pipeline).
    pub key: String,
    /// Name.
    pub name: String,
    /// Kind (`working` | `review` | `done` | `cancelled`).
    pub kind: String,
    /// Position within the pipeline.
    pub position: i64,
    /// Config JSON.
    pub config: serde_json::Value,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// A pipeline transition edge.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineTransitionRecord {
    /// Transition id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Pipeline id.
    pub pipeline_id: String,
    /// From stage id.
    pub from_stage_id: String,
    /// To stage id.
    pub to_stage_id: String,
    /// Label.
    pub label: Option<String>,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// A pipeline case.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineCaseRecord {
    /// Case id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Pipeline id.
    pub pipeline_id: String,
    /// Current stage id.
    pub stage_id: String,
    /// Case key (unique per pipeline).
    pub case_key: String,
    /// Title.
    pub title: String,
    /// Summary.
    pub summary: Option<String>,
    /// Fields JSON.
    pub fields: serde_json::Value,
    /// Workspace ref JSON.
    pub workspace_ref: Option<serde_json::Value>,
    /// Parent case id.
    pub parent_case_id: Option<String>,
    /// Parent case version captured when this case was created.
    pub parent_case_version: Option<i64>,
    /// Request key (idempotent creation).
    pub request_key: Option<String>,
    /// Automation attempt id that created/owns this case.
    pub automation_attempt_id: Option<String>,
    /// Pending transition suggestion JSON (id/toStageKey/rationale/...).
    pub pending_suggestion: Option<serde_json::Value>,
    /// Version.
    pub version: i64,
    /// Lease owner type.
    pub lease_owner_type: Option<String>,
    /// Lease agent id.
    pub lease_agent_id: Option<String>,
    /// Lease user id.
    pub lease_user_id: Option<String>,
    /// Lease token.
    pub lease_token: Option<String>,
    /// ISO 8601 lease expiry.
    pub lease_expires_at: Option<String>,
    /// Terminal kind (`done` | `cancelled`).
    pub terminal_kind: Option<String>,
    /// ISO 8601 terminal time.
    pub terminal_at: Option<String>,
    /// ISO 8601 retire time.
    pub retired_at: Option<String>,
    /// Automation attempt that retired this case.
    pub retired_by_attempt_id: Option<String>,
    /// ISO 8601 when the case was hidden from the board.
    pub hidden_from_board_at: Option<String>,
    /// Retire reason.
    pub retired_reason: Option<String>,
    /// Child count.
    pub child_count: i64,
    /// Terminal child count.
    pub terminal_child_count: i64,
    /// Creating user id.
    pub created_by_user_id: Option<String>,
    /// Origin heartbeat run id.
    pub origin_run_id: Option<String>,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// A pipeline case event.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineCaseEventRecord {
    /// Event id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Case id.
    pub case_id: String,
    /// Event type.
    pub r#type: String,
    /// Actor type (`user` | `agent` | `system`).
    pub actor_type: String,
    /// Actor user id.
    pub actor_user_id: Option<String>,
    /// Actor agent id.
    pub actor_agent_id: Option<String>,
    /// Run id.
    pub run_id: Option<String>,
    /// From stage id.
    pub from_stage_id: Option<String>,
    /// To stage id.
    pub to_stage_id: Option<String>,
    /// Payload JSON.
    pub payload: serde_json::Value,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// Input for creating a pipeline.
#[derive(Debug, Clone)]
pub struct NewPipeline {
    pub company_id: String,
    pub project_id: Option<String>,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub enforce_transitions: bool,
    pub created_by_user_id: Option<String>,
}

/// Input for creating a stage.
#[derive(Debug, Clone)]
pub struct NewStage {
    pub company_id: String,
    pub pipeline_id: String,
    pub key: String,
    pub name: String,
    pub kind: String,
    pub position: i64,
    pub config: Option<serde_json::Value>,
}

/// Input for creating a transition.
#[derive(Debug, Clone)]
pub struct NewTransition {
    pub company_id: String,
    pub pipeline_id: String,
    pub from_stage_id: String,
    pub to_stage_id: String,
    pub label: Option<String>,
}

/// Input for creating a pipeline case.
#[derive(Debug, Clone)]
pub struct NewPipelineCase {
    pub company_id: String,
    pub pipeline_id: String,
    pub stage_id: String,
    pub case_key: String,
    pub title: String,
    pub summary: Option<String>,
    pub fields: Option<serde_json::Value>,
    pub workspace_ref: Option<serde_json::Value>,
    pub parent_case_id: Option<String>,
    pub created_by_user_id: Option<String>,
}

/// Input for appending an event.
#[derive(Debug, Clone)]
pub struct NewPipelineCaseEvent {
    pub company_id: String,
    pub case_id: String,
    pub r#type: String,
    pub actor_type: String,
    pub actor_user_id: Option<String>,
    pub actor_agent_id: Option<String>,
    pub run_id: Option<String>,
    pub from_stage_id: Option<String>,
    pub to_stage_id: Option<String>,
    pub payload: Option<serde_json::Value>,
}

/// A pipeline case ↔ issue link.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineCaseIssueLinkRecord {
    /// Link id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Case id.
    pub case_id: String,
    /// Issue id.
    pub issue_id: String,
    /// Role (`origin` | `conversation` | `work` | `automation`).
    pub role: String,
    /// ISO 8601 retire time.
    pub retired_at: Option<String>,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// A pipeline case blocker edge.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineCaseBlockerRecord {
    /// Blocker id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Blocked case id.
    pub case_id: String,
    /// Blocking case id.
    pub blocked_by_case_id: String,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// A pipeline-level document link.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineDocumentRecord {
    /// Link id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Pipeline id.
    pub pipeline_id: String,
    /// Document id.
    pub document_id: String,
    /// Key (unique per pipeline).
    pub key: String,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// A pipeline-case document link.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineCaseDocumentRecord {
    /// Link id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Case id.
    pub case_id: String,
    /// Document id.
    pub document_id: String,
    /// Key (unique per case).
    pub key: String,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// A pipeline automation execution record.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineAutomationExecutionRecord {
    /// Execution id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Case id.
    pub case_id: String,
    /// Automation id.
    pub automation_id: String,
    /// Triggering event id.
    pub triggering_event_id: String,
    /// Routine id.
    pub routine_id: String,
    /// Status (`succeeded` | `failed`).
    pub status: String,
    /// Execution issue id.
    pub execution_issue_id: Option<String>,
    /// Retry-of execution id.
    pub retry_of_execution_id: Option<String>,
    /// Generation.
    pub generation: i64,
    /// Error.
    pub error: Option<String>,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// Pipeline repository errors.
#[derive(Debug, Error)]
pub enum PipelineError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The company does not exist.
    #[error("company not found")]
    CompanyNotFound,
    /// A referenced record is missing or belongs to another company.
    #[error("reference not found")]
    ReferenceNotFound,
    /// The row does not exist.
    #[error("not found")]
    NotFound,
    /// A duplicate key exists.
    #[error("duplicate key")]
    Duplicate,
    /// The transition is not allowed by the pipeline.
    #[error("transition not allowed")]
    TransitionNotAllowed,
}

/// Pipeline persistence contract.
#[async_trait]
#[allow(clippy::too_many_arguments)]
pub trait PipelineRepository: Send + Sync {
    async fn create_pipeline(&self, input: NewPipeline) -> Result<PipelineRecord, PipelineError>;
    async fn list_pipelines(&self, company_id: &str) -> Result<Vec<PipelineRecord>, PipelineError>;
    async fn get_pipeline(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<PipelineRecord>, PipelineError>;
    /// Updates a pipeline's name, description, and archived status. `None`
    /// leaves a field unchanged; `Some(None)` clears the description.
    ///
    /// # Errors
    ///
    /// Returns [`PipelineError`] on database failure.
    async fn update_pipeline(
        &self,
        company_id: &str,
        id: &str,
        name: Option<String>,
        description: Option<Option<String>>,
        archived: Option<bool>,
    ) -> Result<Option<PipelineRecord>, PipelineError>;
    async fn set_pipeline_archived(
        &self,
        company_id: &str,
        id: &str,
        archived: bool,
    ) -> Result<Option<PipelineRecord>, PipelineError>;
    async fn delete_pipeline(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<PipelineRecord>, PipelineError>;
    /// Resolves the owning company of a pipeline.
    async fn company_of_pipeline(&self, id: &str) -> Result<Option<String>, PipelineError>;
    /// Resolves the owning company of a stage.
    async fn company_of_stage(&self, id: &str) -> Result<Option<String>, PipelineError>;

    async fn create_stage(&self, input: NewStage) -> Result<PipelineStageRecord, PipelineError>;
    async fn list_stages(
        &self,
        company_id: &str,
        pipeline_id: &str,
    ) -> Result<Vec<PipelineStageRecord>, PipelineError>;
    async fn delete_stage(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<PipelineStageRecord>, PipelineError>;

    async fn create_transition(
        &self,
        input: NewTransition,
    ) -> Result<PipelineTransitionRecord, PipelineError>;
    async fn list_transitions(
        &self,
        company_id: &str,
        pipeline_id: &str,
    ) -> Result<Vec<PipelineTransitionRecord>, PipelineError>;

    async fn create_case(
        &self,
        input: NewPipelineCase,
    ) -> Result<PipelineCaseRecord, PipelineError>;
    async fn list_cases(
        &self,
        company_id: &str,
        pipeline_id: &str,
    ) -> Result<Vec<PipelineCaseRecord>, PipelineError>;
    async fn get_case(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<PipelineCaseRecord>, PipelineError>;
    async fn company_of_case(&self, id: &str) -> Result<Option<String>, PipelineError>;
    async fn move_case(
        &self,
        company_id: &str,
        id: &str,
        to_stage_id: &str,
        actor_type: &str,
        actor_user_id: Option<String>,
        actor_agent_id: Option<String>,
        force: bool,
    ) -> Result<Option<PipelineCaseRecord>, PipelineError>;
    async fn update_case(
        &self,
        company_id: &str,
        id: &str,
        title: Option<String>,
        summary: Option<Option<String>>,
        fields: Option<serde_json::Value>,
    ) -> Result<Option<PipelineCaseRecord>, PipelineError>;
    async fn delete_case(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<PipelineCaseRecord>, PipelineError>;

    async fn add_event(
        &self,
        input: NewPipelineCaseEvent,
    ) -> Result<PipelineCaseEventRecord, PipelineError>;
    async fn list_events(
        &self,
        company_id: &str,
        case_id: &str,
    ) -> Result<Vec<PipelineCaseEventRecord>, PipelineError>;

    // Issue links ---------------------------------------------------------
    async fn link_issue(
        &self,
        company_id: &str,
        case_id: &str,
        issue_id: &str,
        role: &str,
    ) -> Result<PipelineCaseIssueLinkRecord, PipelineError>;
    async fn list_issue_links(
        &self,
        company_id: &str,
        case_id: &str,
    ) -> Result<Vec<PipelineCaseIssueLinkRecord>, PipelineError>;
    async fn unlink_issue(
        &self,
        company_id: &str,
        case_id: &str,
        issue_id: &str,
    ) -> Result<bool, PipelineError>;

    // Blockers ------------------------------------------------------------
    async fn add_blocker(
        &self,
        company_id: &str,
        case_id: &str,
        blocked_by_case_id: &str,
    ) -> Result<PipelineCaseBlockerRecord, PipelineError>;
    async fn list_blockers(
        &self,
        company_id: &str,
        case_id: &str,
    ) -> Result<Vec<PipelineCaseBlockerRecord>, PipelineError>;
    async fn remove_blocker(
        &self,
        company_id: &str,
        case_id: &str,
        blocked_by_case_id: &str,
    ) -> Result<bool, PipelineError>;

    // Documents -----------------------------------------------------------
    async fn link_pipeline_document(
        &self,
        company_id: &str,
        pipeline_id: &str,
        document_id: &str,
        key: &str,
    ) -> Result<PipelineDocumentRecord, PipelineError>;
    async fn list_pipeline_documents(
        &self,
        company_id: &str,
        pipeline_id: &str,
    ) -> Result<Vec<PipelineDocumentRecord>, PipelineError>;
    async fn link_case_document(
        &self,
        company_id: &str,
        case_id: &str,
        document_id: &str,
        key: &str,
    ) -> Result<PipelineCaseDocumentRecord, PipelineError>;
    async fn list_case_documents(
        &self,
        company_id: &str,
        case_id: &str,
    ) -> Result<Vec<PipelineCaseDocumentRecord>, PipelineError>;

    // Automation executions -----------------------------------------------
    async fn record_automation(
        &self,
        company_id: &str,
        case_id: &str,
        automation_id: &str,
        triggering_event_id: &str,
        routine_id: &str,
        status: &str,
        execution_issue_id: Option<String>,
        error: Option<String>,
    ) -> Result<PipelineAutomationExecutionRecord, PipelineError>;
    async fn list_automations(
        &self,
        company_id: &str,
        case_id: &str,
    ) -> Result<Vec<PipelineAutomationExecutionRecord>, PipelineError>;
}

/// Turso/libSQL implementation of [`PipelineRepository`].
#[derive(Debug)]
pub struct TursoPipelineRepository {
    db: Database,
}

impl TursoPipelineRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

fn row_to_pipeline(row: &libsql::Row) -> Result<PipelineRecord, libsql::Error> {
    Ok(PipelineRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        project_id: helpers::row_text(row, 2)?,
        key: helpers::row_text(row, 3)?.expect("key"),
        name: helpers::row_text(row, 4)?.expect("name"),
        description: helpers::row_text(row, 5)?,
        enforce_transitions: helpers::row_i64(row, 6)? != 0,
        created_by_user_id: helpers::row_text(row, 7)?,
        archived_at: helpers::row_text(row, 8)?,
        created_at: helpers::row_text(row, 9)?.expect("created_at"),
    })
}

fn row_to_stage(row: &libsql::Row) -> Result<PipelineStageRecord, libsql::Error> {
    Ok(PipelineStageRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        pipeline_id: helpers::row_text(row, 2)?.expect("pipeline_id"),
        key: helpers::row_text(row, 3)?.expect("key"),
        name: helpers::row_text(row, 4)?.expect("name"),
        kind: helpers::row_text(row, 5)?.expect("kind"),
        position: helpers::row_i64(row, 6)?,
        config: helpers::row_text(row, 7)?
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or(serde_json::Value::Null),
        created_at: helpers::row_text(row, 8)?.expect("created_at"),
    })
}

fn row_to_transition(row: &libsql::Row) -> Result<PipelineTransitionRecord, libsql::Error> {
    Ok(PipelineTransitionRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        pipeline_id: helpers::row_text(row, 2)?.expect("pipeline_id"),
        from_stage_id: helpers::row_text(row, 3)?.expect("from_stage_id"),
        to_stage_id: helpers::row_text(row, 4)?.expect("to_stage_id"),
        label: helpers::row_text(row, 5)?,
        created_at: helpers::row_text(row, 6)?.expect("created_at"),
    })
}

fn row_to_case(row: &libsql::Row) -> Result<PipelineCaseRecord, libsql::Error> {
    Ok(PipelineCaseRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        pipeline_id: helpers::row_text(row, 2)?.expect("pipeline_id"),
        stage_id: helpers::row_text(row, 3)?.expect("stage_id"),
        case_key: helpers::row_text(row, 4)?.expect("case_key"),
        title: helpers::row_text(row, 5)?.expect("title"),
        summary: helpers::row_text(row, 6)?,
        fields: helpers::row_text(row, 7)?
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or(serde_json::Value::Null),
        workspace_ref: helpers::row_text(row, 8)?.and_then(|raw| serde_json::from_str(&raw).ok()),
        parent_case_id: helpers::row_text(row, 9)?,
        parent_case_version: helpers::row_i64_opt(row, 10)?,
        request_key: helpers::row_text(row, 11)?,
        automation_attempt_id: helpers::row_text(row, 12)?,
        pending_suggestion: helpers::row_text(row, 13)?
            .and_then(|raw| serde_json::from_str(&raw).ok()),
        version: helpers::row_i64(row, 14)?,
        lease_owner_type: helpers::row_text(row, 15)?,
        lease_agent_id: helpers::row_text(row, 16)?,
        lease_user_id: helpers::row_text(row, 17)?,
        lease_token: helpers::row_text(row, 18)?,
        lease_expires_at: helpers::row_text(row, 19)?,
        terminal_kind: helpers::row_text(row, 20)?,
        terminal_at: helpers::row_text(row, 21)?,
        retired_at: helpers::row_text(row, 22)?,
        retired_by_attempt_id: helpers::row_text(row, 23)?,
        hidden_from_board_at: helpers::row_text(row, 24)?,
        retired_reason: helpers::row_text(row, 25)?,
        child_count: helpers::row_i64(row, 26)?,
        terminal_child_count: helpers::row_i64(row, 27)?,
        created_by_user_id: helpers::row_text(row, 28)?,
        origin_run_id: helpers::row_text(row, 29)?,
        created_at: helpers::row_text(row, 30)?.expect("created_at"),
    })
}

fn row_to_event(row: &libsql::Row) -> Result<PipelineCaseEventRecord, libsql::Error> {
    Ok(PipelineCaseEventRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        case_id: helpers::row_text(row, 2)?.expect("case_id"),
        r#type: helpers::row_text(row, 3)?.expect("type"),
        actor_type: helpers::row_text(row, 4)?.expect("actor_type"),
        actor_user_id: helpers::row_text(row, 5)?,
        actor_agent_id: helpers::row_text(row, 6)?,
        run_id: helpers::row_text(row, 7)?,
        from_stage_id: helpers::row_text(row, 8)?,
        to_stage_id: helpers::row_text(row, 9)?,
        payload: helpers::row_text(row, 10)?
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or(serde_json::Value::Null),
        created_at: helpers::row_text(row, 11)?.expect("created_at"),
    })
}

const PIPELINE_COLUMNS: &str = "id, company_id, project_id, key, name, description,
    enforce_transitions, created_by_user_id, archived_at, created_at";
const STAGE_COLUMNS: &str = "id, company_id, pipeline_id, key, name, kind, position, config,
    created_at";
const TRANSITION_COLUMNS: &str = "id, company_id, pipeline_id, from_stage_id, to_stage_id, label,
    created_at";
const CASE_COLUMNS: &str = "id, company_id, pipeline_id, stage_id, case_key, title, summary,
    fields, workspace_ref, parent_case_id, parent_case_version, request_key, automation_attempt_id,
    pending_suggestion, version, lease_owner_type, lease_agent_id, lease_user_id, lease_token,
    lease_expires_at, terminal_kind, terminal_at, retired_at, retired_by_attempt_id,
    hidden_from_board_at, retired_reason, child_count, terminal_child_count, created_by_user_id,
    origin_run_id, created_at";
const EVENT_COLUMNS: &str = "id, company_id, case_id, type, actor_type, actor_user_id,
    actor_agent_id, run_id, from_stage_id, to_stage_id, payload, created_at";

async fn ensure_pipeline(
    conn: &libsql::Connection,
    company_id: &str,
    pipeline_id: &str,
) -> Result<(), PipelineError> {
    if !helpers::row_belongs_to_company(conn, "pipelines", pipeline_id, company_id).await? {
        return Err(PipelineError::ReferenceNotFound);
    }
    Ok(())
}

#[async_trait]
impl PipelineRepository for TursoPipelineRepository {
    async fn create_pipeline(&self, input: NewPipeline) -> Result<PipelineRecord, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(PipelineError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO pipelines
                   (id, company_id, project_id, key, name, description, enforce_transitions,
                    created_by_user_id, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.project_id,
                    input.key,
                    input.name,
                    input.description,
                    i64::from(input.enforce_transitions),
                    input.created_by_user_id
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!("SELECT {PIPELINE_COLUMNS} FROM pipelines WHERE id = ?1"),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("pipeline was just inserted");
                Ok(row_to_pipeline(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(PipelineError::Duplicate)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_pipelines(&self, company_id: &str) -> Result<Vec<PipelineRecord>, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {PIPELINE_COLUMNS} FROM pipelines WHERE company_id = ?1 ORDER BY name"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut pipelines = Vec::new();
        while let Some(row) = rows.next().await? {
            pipelines.push(row_to_pipeline(&row)?);
        }
        Ok(pipelines)
    }

    async fn get_pipeline(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<PipelineRecord>, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {PIPELINE_COLUMNS} FROM pipelines WHERE company_id = ?1 AND id = ?2"
                ),
                libsql::params![company_id, id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_pipeline(&row)?)),
            None => Ok(None),
        }
    }

    async fn update_pipeline(
        &self,
        company_id: &str,
        id: &str,
        name: Option<String>,
        description: Option<Option<String>>,
        archived: Option<bool>,
    ) -> Result<Option<PipelineRecord>, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut sets = Vec::new();
        let mut values: Vec<libsql::Value> = Vec::new();
        if let Some(name) = name {
            sets.push(format!("name = ?{}", values.len() + 1));
            values.push(libsql::Value::from(name));
        }
        if let Some(description) = description {
            sets.push(format!("description = ?{}", values.len() + 1));
            values.push(
                description
                    .map(libsql::Value::from)
                    .unwrap_or(libsql::Value::Null),
            );
        }
        if let Some(archived) = archived {
            sets.push(format!(
                "archived_at = CASE WHEN ?{} = 1 \
                 THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') ELSE NULL END",
                values.len() + 1
            ));
            values.push(libsql::Value::from(i64::from(archived)));
        }
        if sets.is_empty() {
            return self.get_pipeline(company_id, id).await;
        }
        sets.push("updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')".to_owned());
        let company_param = values.len() + 1;
        let id_param = values.len() + 2;
        values.push(libsql::Value::from(company_id.to_owned()));
        values.push(libsql::Value::from(id.to_owned()));
        let sql = format!(
            "UPDATE pipelines SET {} WHERE company_id = ?{company_param} AND id = ?{id_param}",
            sets.join(", ")
        );
        let updated = conn.execute(&sql, values).await?;
        if updated == 0 {
            return Ok(None);
        }
        self.get_pipeline(company_id, id).await
    }

    async fn set_pipeline_archived(
        &self,
        company_id: &str,
        id: &str,
        archived: bool,
    ) -> Result<Option<PipelineRecord>, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let updated = conn
            .execute(
                "UPDATE pipelines SET archived_at = CASE WHEN ?1 = 1
                        THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') ELSE NULL END,
                        updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE company_id = ?2 AND id = ?3",
                libsql::params![i64::from(archived), company_id, id],
            )
            .await?;
        if updated == 0 {
            return Ok(None);
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {PIPELINE_COLUMNS} FROM pipelines WHERE company_id = ?1 AND id = ?2"
                ),
                libsql::params![company_id, id],
            )
            .await?;
        let row = rows.next().await?.expect("pipeline exists");
        Ok(Some(row_to_pipeline(&row)?))
    }

    async fn delete_pipeline(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<PipelineRecord>, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {PIPELINE_COLUMNS} FROM pipelines WHERE company_id = ?1 AND id = ?2"
                ),
                libsql::params![company_id, id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let record = row_to_pipeline(&row)?;
        conn.execute("DELETE FROM pipelines WHERE id = ?1", libsql::params![id])
            .await?;
        Ok(Some(record))
    }

    async fn company_of_pipeline(&self, id: &str) -> Result<Option<String>, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        Ok(helpers::row_company(&conn, "pipelines", id).await?)
    }

    async fn company_of_stage(&self, id: &str) -> Result<Option<String>, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        Ok(helpers::row_company(&conn, "pipeline_stages", id).await?)
    }

    async fn create_stage(&self, input: NewStage) -> Result<PipelineStageRecord, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        ensure_pipeline(&conn, &input.company_id, &input.pipeline_id).await?;
        let id = Uuid::new_v4().to_string();
        let config = input
            .config
            .map(|v| v.to_string())
            .unwrap_or_else(|| "{}".to_owned());
        let result = conn
            .execute(
                "INSERT INTO pipeline_stages
                   (id, company_id, pipeline_id, key, name, kind, position, config,
                    created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.pipeline_id,
                    input.key,
                    input.name,
                    input.kind,
                    input.position,
                    config
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!("SELECT {STAGE_COLUMNS} FROM pipeline_stages WHERE id = ?1"),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("stage was just inserted");
                Ok(row_to_stage(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(PipelineError::Duplicate)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_stages(
        &self,
        company_id: &str,
        pipeline_id: &str,
    ) -> Result<Vec<PipelineStageRecord>, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {STAGE_COLUMNS} FROM pipeline_stages
                     WHERE company_id = ?1 AND pipeline_id = ?2 ORDER BY position"
                ),
                libsql::params![company_id, pipeline_id],
            )
            .await?;
        let mut stages = Vec::new();
        while let Some(row) = rows.next().await? {
            stages.push(row_to_stage(&row)?);
        }
        Ok(stages)
    }

    async fn delete_stage(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<PipelineStageRecord>, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {STAGE_COLUMNS} FROM pipeline_stages WHERE company_id = ?1 AND id = ?2"
                ),
                libsql::params![company_id, id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let record = row_to_stage(&row)?;
        conn.execute(
            "DELETE FROM pipeline_stages WHERE id = ?1",
            libsql::params![id],
        )
        .await?;
        Ok(Some(record))
    }

    async fn create_transition(
        &self,
        input: NewTransition,
    ) -> Result<PipelineTransitionRecord, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        ensure_pipeline(&conn, &input.company_id, &input.pipeline_id).await?;
        for stage_id in [&input.from_stage_id, &input.to_stage_id] {
            let mut rows = conn
                .query(
                    "SELECT 1 FROM pipeline_stages
                     WHERE company_id = ?1 AND id = ?2 AND pipeline_id = ?3",
                    libsql::params![
                        input.company_id.clone(),
                        (*stage_id).clone(),
                        input.pipeline_id.clone()
                    ],
                )
                .await?;
            if rows.next().await?.is_none() {
                return Err(PipelineError::ReferenceNotFound);
            }
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO pipeline_transitions
                   (id, company_id, pipeline_id, from_stage_id, to_stage_id, label,
                    created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.pipeline_id,
                    input.from_stage_id,
                    input.to_stage_id,
                    input.label
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!(
                            "SELECT {TRANSITION_COLUMNS} FROM pipeline_transitions WHERE id = ?1"
                        ),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("transition was just inserted");
                Ok(row_to_transition(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(PipelineError::Duplicate)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_transitions(
        &self,
        company_id: &str,
        pipeline_id: &str,
    ) -> Result<Vec<PipelineTransitionRecord>, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {TRANSITION_COLUMNS} FROM pipeline_transitions
                     WHERE company_id = ?1 AND pipeline_id = ?2 ORDER BY created_at"
                ),
                libsql::params![company_id, pipeline_id],
            )
            .await?;
        let mut transitions = Vec::new();
        while let Some(row) = rows.next().await? {
            transitions.push(row_to_transition(&row)?);
        }
        Ok(transitions)
    }

    async fn create_case(
        &self,
        input: NewPipelineCase,
    ) -> Result<PipelineCaseRecord, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        ensure_pipeline(&conn, &input.company_id, &input.pipeline_id).await?;
        let mut stage_rows = conn
            .query(
                "SELECT 1 FROM pipeline_stages
                 WHERE company_id = ?1 AND id = ?2 AND pipeline_id = ?3",
                libsql::params![
                    input.company_id.clone(),
                    input.stage_id.clone(),
                    input.pipeline_id.clone()
                ],
            )
            .await?;
        if stage_rows.next().await?.is_none() {
            return Err(PipelineError::ReferenceNotFound);
        }
        if let Some(parent_id) = &input.parent_case_id {
            let mut parent_rows = conn
                .query(
                    "SELECT 1 FROM pipeline_cases
                     WHERE company_id = ?1 AND id = ?2 AND pipeline_id = ?3",
                    libsql::params![
                        input.company_id.clone(),
                        (*parent_id).clone(),
                        input.pipeline_id.clone()
                    ],
                )
                .await?;
            if parent_rows.next().await?.is_none() {
                return Err(PipelineError::ReferenceNotFound);
            }
        }
        let id = Uuid::new_v4().to_string();
        let fields = input
            .fields
            .map(|v| v.to_string())
            .unwrap_or_else(|| "{}".to_owned());
        let workspace_ref = input.workspace_ref.map(|v| v.to_string());
        let result = conn
            .execute(
                "INSERT INTO pipeline_cases
                   (id, company_id, pipeline_id, stage_id, case_key, title, summary, fields,
                    workspace_ref, parent_case_id, version, created_by_user_id, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 1, ?11,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id.clone(),
                    input.pipeline_id.clone(),
                    input.stage_id.clone(),
                    input.case_key.clone(),
                    input.title.clone(),
                    input.summary.clone(),
                    fields,
                    workspace_ref,
                    input.parent_case_id.clone(),
                    input.created_by_user_id.clone()
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!("SELECT {CASE_COLUMNS} FROM pipeline_cases WHERE id = ?1"),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("case was just inserted");
                Ok(row_to_case(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(PipelineError::Duplicate)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_cases(
        &self,
        company_id: &str,
        pipeline_id: &str,
    ) -> Result<Vec<PipelineCaseRecord>, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {CASE_COLUMNS} FROM pipeline_cases
                     WHERE company_id = ?1 AND pipeline_id = ?2 ORDER BY created_at DESC"
                ),
                libsql::params![company_id, pipeline_id],
            )
            .await?;
        let mut cases = Vec::new();
        while let Some(row) = rows.next().await? {
            cases.push(row_to_case(&row)?);
        }
        Ok(cases)
    }

    async fn get_case(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<PipelineCaseRecord>, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {CASE_COLUMNS} FROM pipeline_cases WHERE company_id = ?1 AND id = ?2"
                ),
                libsql::params![company_id, id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_case(&row)?)),
            None => Ok(None),
        }
    }

    async fn company_of_case(&self, id: &str) -> Result<Option<String>, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        Ok(helpers::row_company(&conn, "pipeline_cases", id).await?)
    }

    async fn move_case(
        &self,
        company_id: &str,
        id: &str,
        to_stage_id: &str,
        actor_type: &str,
        actor_user_id: Option<String>,
        actor_agent_id: Option<String>,
        force: bool,
    ) -> Result<Option<PipelineCaseRecord>, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {CASE_COLUMNS} FROM pipeline_cases WHERE company_id = ?1 AND id = ?2"
                ),
                libsql::params![company_id, id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let case = row_to_case(&row)?;
        if case.stage_id == to_stage_id {
            return Ok(Some(case));
        }
        // Target stage must be in the same pipeline/company.
        let mut target = conn
            .query(
                "SELECT kind FROM pipeline_stages
                 WHERE company_id = ?1 AND id = ?2 AND pipeline_id = ?3",
                libsql::params![company_id, to_stage_id, case.pipeline_id.clone()],
            )
            .await?;
        let Some(target_row) = target.next().await? else {
            return Err(PipelineError::ReferenceNotFound);
        };
        let target_kind = helpers::row_text(&target_row, 0)?.expect("kind");
        // Enforced transitions must be declared unless force is used.
        let mut pipeline_rows = conn
            .query(
                "SELECT enforce_transitions FROM pipelines WHERE company_id = ?1 AND id = ?2",
                libsql::params![company_id, case.pipeline_id.clone()],
            )
            .await?;
        let enforce_transitions =
            helpers::row_i64(&pipeline_rows.next().await?.expect("pipeline exists"), 0)? != 0;
        if enforce_transitions && !force {
            let mut t = conn
                .query(
                    "SELECT 1 FROM pipeline_transitions
                     WHERE company_id = ?1 AND pipeline_id = ?2
                       AND from_stage_id = ?3 AND to_stage_id = ?4",
                    libsql::params![
                        company_id,
                        case.pipeline_id.clone(),
                        case.stage_id.clone(),
                        to_stage_id
                    ],
                )
                .await?;
            if t.next().await?.is_none() {
                return Err(PipelineError::TransitionNotAllowed);
            }
        }
        let terminal_kind = if target_kind == "done" {
            Some("done")
        } else if target_kind == "cancelled" {
            Some("cancelled")
        } else {
            None
        };
        conn.execute(
            "UPDATE pipeline_cases SET stage_id = ?1, version = version + 1,
                    terminal_kind = ?2,
                    terminal_at = CASE WHEN ?2 IS NOT NULL
                                       THEN strftime('%Y-%m-%dT%H:%M:%fZ','now')
                                       ELSE NULL END,
                    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE company_id = ?3 AND id = ?4",
            libsql::params![to_stage_id, terminal_kind, company_id, id],
        )
        .await?;
        let event_type = if force {
            "transition_forced"
        } else {
            "transitioned"
        };
        conn.execute(
            "INSERT INTO pipeline_case_events
               (id, company_id, case_id, type, actor_type, actor_user_id, actor_agent_id,
                from_stage_id, to_stage_id, payload, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, '{}',
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                Uuid::new_v4().to_string(),
                company_id,
                id,
                event_type,
                actor_type,
                actor_user_id,
                actor_agent_id,
                case.stage_id,
                to_stage_id
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {CASE_COLUMNS} FROM pipeline_cases WHERE company_id = ?1 AND id = ?2"
                ),
                libsql::params![company_id, id],
            )
            .await?;
        let row = rows.next().await?.expect("case exists");
        Ok(Some(row_to_case(&row)?))
    }

    async fn update_case(
        &self,
        company_id: &str,
        id: &str,
        title: Option<String>,
        summary: Option<Option<String>>,
        fields: Option<serde_json::Value>,
    ) -> Result<Option<PipelineCaseRecord>, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut sets = Vec::new();
        let mut values: Vec<libsql::Value> = Vec::new();
        let mut param = 0usize;
        if let Some(title) = title {
            param += 1;
            sets.push(format!("title = ?{param}"));
            values.push(libsql::Value::from(title));
        }
        if let Some(summary) = summary {
            match summary {
                Some(summary) => {
                    param += 1;
                    sets.push(format!("summary = ?{param}"));
                    values.push(libsql::Value::from(summary));
                }
                None => sets.push("summary = NULL".to_owned()),
            }
        }
        if let Some(fields) = fields {
            param += 1;
            sets.push(format!("fields = ?{param}"));
            values.push(libsql::Value::from(fields.to_string()));
        }
        if sets.is_empty() {
            return Err(PipelineError::NotFound);
        }
        let company_param = param + 1;
        let id_param = param + 2;
        values.push(libsql::Value::from(company_id.to_owned()));
        values.push(libsql::Value::from(id.to_owned()));
        let sql = format!(
            "UPDATE pipeline_cases SET {}, version = version + 1,
                    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE company_id = ?{company_param} AND id = ?{id_param}",
            sets.join(", ")
        );
        let updated = conn.execute(&sql, values).await?;
        if updated == 0 {
            return Ok(None);
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {CASE_COLUMNS} FROM pipeline_cases WHERE company_id = ?1 AND id = ?2"
                ),
                libsql::params![company_id, id],
            )
            .await?;
        let row = rows.next().await?.expect("case exists");
        Ok(Some(row_to_case(&row)?))
    }

    async fn delete_case(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<PipelineCaseRecord>, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {CASE_COLUMNS} FROM pipeline_cases WHERE company_id = ?1 AND id = ?2"
                ),
                libsql::params![company_id, id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let record = row_to_case(&row)?;
        conn.execute(
            "DELETE FROM pipeline_cases WHERE id = ?1",
            libsql::params![id],
        )
        .await?;
        Ok(Some(record))
    }

    async fn add_event(
        &self,
        input: NewPipelineCaseEvent,
    ) -> Result<PipelineCaseEventRecord, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut case_rows = conn
            .query(
                "SELECT 1 FROM pipeline_cases WHERE company_id = ?1 AND id = ?2",
                libsql::params![input.company_id.clone(), input.case_id.clone()],
            )
            .await?;
        if case_rows.next().await?.is_none() {
            return Err(PipelineError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let payload = input
            .payload
            .map(|v| v.to_string())
            .unwrap_or_else(|| "{}".to_owned());
        conn.execute(
            "INSERT INTO pipeline_case_events
               (id, company_id, case_id, type, actor_type, actor_user_id, actor_agent_id,
                run_id, from_stage_id, to_stage_id, payload, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.case_id,
                input.r#type,
                input.actor_type,
                input.actor_user_id,
                input.actor_agent_id,
                input.run_id,
                input.from_stage_id,
                input.to_stage_id,
                payload
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {EVENT_COLUMNS} FROM pipeline_case_events WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("event was just inserted");
        Ok(row_to_event(&row)?)
    }

    async fn list_events(
        &self,
        company_id: &str,
        case_id: &str,
    ) -> Result<Vec<PipelineCaseEventRecord>, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {EVENT_COLUMNS} FROM pipeline_case_events
                     WHERE company_id = ?1 AND case_id = ?2 ORDER BY created_at"
                ),
                libsql::params![company_id, case_id],
            )
            .await?;
        let mut events = Vec::new();
        while let Some(row) = rows.next().await? {
            events.push(row_to_event(&row)?);
        }
        Ok(events)
    }

    async fn link_issue(
        &self,
        company_id: &str,
        case_id: &str,
        issue_id: &str,
        role: &str,
    ) -> Result<PipelineCaseIssueLinkRecord, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        ensure_case(&conn, company_id, case_id).await?;
        if !helpers::row_belongs_to_company(&conn, "issues", issue_id, company_id).await? {
            return Err(PipelineError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO pipeline_case_issue_links
                   (id, company_id, case_id, issue_id, role, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![id.clone(), company_id, case_id, issue_id, role],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        "SELECT id, company_id, case_id, issue_id, role, retired_at, created_at
                         FROM pipeline_case_issue_links WHERE id = ?1",
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("link was just inserted");
                Ok(row_to_issue_link(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(PipelineError::Duplicate)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_issue_links(
        &self,
        company_id: &str,
        case_id: &str,
    ) -> Result<Vec<PipelineCaseIssueLinkRecord>, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, case_id, issue_id, role, retired_at, created_at
                 FROM pipeline_case_issue_links
                 WHERE company_id = ?1 AND case_id = ?2 ORDER BY created_at",
                libsql::params![company_id, case_id],
            )
            .await?;
        let mut links = Vec::new();
        while let Some(row) = rows.next().await? {
            links.push(row_to_issue_link(&row)?);
        }
        Ok(links)
    }

    async fn unlink_issue(
        &self,
        company_id: &str,
        case_id: &str,
        issue_id: &str,
    ) -> Result<bool, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let updated = conn
            .execute(
                "DELETE FROM pipeline_case_issue_links
                 WHERE company_id = ?1 AND case_id = ?2 AND issue_id = ?3",
                libsql::params![company_id, case_id, issue_id],
            )
            .await?;
        Ok(updated > 0)
    }

    async fn add_blocker(
        &self,
        company_id: &str,
        case_id: &str,
        blocked_by_case_id: &str,
    ) -> Result<PipelineCaseBlockerRecord, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        ensure_case(&conn, company_id, case_id).await?;
        if !helpers::row_belongs_to_company(&conn, "pipeline_cases", blocked_by_case_id, company_id)
            .await?
        {
            return Err(PipelineError::ReferenceNotFound);
        }
        if case_id == blocked_by_case_id {
            return Err(PipelineError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO pipeline_case_blockers (id, company_id, case_id, blocked_by_case_id,
                                                     created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![id.clone(), company_id, case_id, blocked_by_case_id],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        "SELECT id, company_id, case_id, blocked_by_case_id, created_at
                         FROM pipeline_case_blockers WHERE id = ?1",
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("blocker was just inserted");
                Ok(row_to_blocker(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(PipelineError::Duplicate)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_blockers(
        &self,
        company_id: &str,
        case_id: &str,
    ) -> Result<Vec<PipelineCaseBlockerRecord>, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, case_id, blocked_by_case_id, created_at
                 FROM pipeline_case_blockers
                 WHERE company_id = ?1 AND case_id = ?2 ORDER BY created_at",
                libsql::params![company_id, case_id],
            )
            .await?;
        let mut blockers = Vec::new();
        while let Some(row) = rows.next().await? {
            blockers.push(row_to_blocker(&row)?);
        }
        Ok(blockers)
    }

    async fn remove_blocker(
        &self,
        company_id: &str,
        case_id: &str,
        blocked_by_case_id: &str,
    ) -> Result<bool, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let updated = conn
            .execute(
                "DELETE FROM pipeline_case_blockers
                 WHERE company_id = ?1 AND case_id = ?2 AND blocked_by_case_id = ?3",
                libsql::params![company_id, case_id, blocked_by_case_id],
            )
            .await?;
        Ok(updated > 0)
    }

    async fn link_pipeline_document(
        &self,
        company_id: &str,
        pipeline_id: &str,
        document_id: &str,
        key: &str,
    ) -> Result<PipelineDocumentRecord, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        ensure_pipeline(&conn, company_id, pipeline_id).await?;
        if !helpers::row_belongs_to_company(&conn, "documents", document_id, company_id).await? {
            return Err(PipelineError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO pipeline_documents (id, company_id, pipeline_id, document_id, key,
                                                 created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![id.clone(), company_id, pipeline_id, document_id, key],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        "SELECT id, company_id, pipeline_id, document_id, key, created_at
                         FROM pipeline_documents WHERE id = ?1",
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("document link was just inserted");
                Ok(row_to_pipeline_document(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(PipelineError::Duplicate)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_pipeline_documents(
        &self,
        company_id: &str,
        pipeline_id: &str,
    ) -> Result<Vec<PipelineDocumentRecord>, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, pipeline_id, document_id, key, created_at
                 FROM pipeline_documents
                 WHERE company_id = ?1 AND pipeline_id = ?2 ORDER BY key",
                libsql::params![company_id, pipeline_id],
            )
            .await?;
        let mut docs = Vec::new();
        while let Some(row) = rows.next().await? {
            docs.push(row_to_pipeline_document(&row)?);
        }
        Ok(docs)
    }

    async fn link_case_document(
        &self,
        company_id: &str,
        case_id: &str,
        document_id: &str,
        key: &str,
    ) -> Result<PipelineCaseDocumentRecord, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        ensure_case(&conn, company_id, case_id).await?;
        if !helpers::row_belongs_to_company(&conn, "documents", document_id, company_id).await? {
            return Err(PipelineError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO pipeline_case_documents (id, company_id, case_id, document_id, key,
                                                      created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![id.clone(), company_id, case_id, document_id, key],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        "SELECT id, company_id, case_id, document_id, key, created_at
                         FROM pipeline_case_documents WHERE id = ?1",
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("document link was just inserted");
                Ok(row_to_case_document(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(PipelineError::Duplicate)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_case_documents(
        &self,
        company_id: &str,
        case_id: &str,
    ) -> Result<Vec<PipelineCaseDocumentRecord>, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, case_id, document_id, key, created_at
                 FROM pipeline_case_documents
                 WHERE company_id = ?1 AND case_id = ?2 ORDER BY key",
                libsql::params![company_id, case_id],
            )
            .await?;
        let mut docs = Vec::new();
        while let Some(row) = rows.next().await? {
            docs.push(row_to_case_document(&row)?);
        }
        Ok(docs)
    }

    async fn record_automation(
        &self,
        company_id: &str,
        case_id: &str,
        automation_id: &str,
        triggering_event_id: &str,
        routine_id: &str,
        status: &str,
        execution_issue_id: Option<String>,
        error: Option<String>,
    ) -> Result<PipelineAutomationExecutionRecord, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        ensure_case(&conn, company_id, case_id).await?;
        if !helpers::row_belongs_to_company(&conn, "routines", routine_id, company_id).await? {
            return Err(PipelineError::ReferenceNotFound);
        }
        if let Some(issue_id) = &execution_issue_id
            && !helpers::row_belongs_to_company(&conn, "issues", issue_id, company_id).await?
        {
            return Err(PipelineError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO pipeline_automation_executions
                   (id, company_id, case_id, automation_id, triggering_event_id, routine_id,
                    status, execution_issue_id, generation, error, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    company_id,
                    case_id,
                    automation_id,
                    triggering_event_id,
                    routine_id,
                    status,
                    execution_issue_id,
                    error
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        "SELECT id, company_id, case_id, automation_id, triggering_event_id,
                                routine_id, status, execution_issue_id, retry_of_execution_id,
                                generation, error, created_at
                         FROM pipeline_automation_executions WHERE id = ?1",
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("automation was just inserted");
                Ok(row_to_automation(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(PipelineError::Duplicate)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_automations(
        &self,
        company_id: &str,
        case_id: &str,
    ) -> Result<Vec<PipelineAutomationExecutionRecord>, PipelineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, case_id, automation_id, triggering_event_id,
                        routine_id, status, execution_issue_id, retry_of_execution_id,
                        generation, error, created_at
                 FROM pipeline_automation_executions
                 WHERE company_id = ?1 AND case_id = ?2 ORDER BY created_at",
                libsql::params![company_id, case_id],
            )
            .await?;
        let mut automations = Vec::new();
        while let Some(row) = rows.next().await? {
            automations.push(row_to_automation(&row)?);
        }
        Ok(automations)
    }
}

fn row_to_issue_link(row: &libsql::Row) -> Result<PipelineCaseIssueLinkRecord, libsql::Error> {
    Ok(PipelineCaseIssueLinkRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        case_id: helpers::row_text(row, 2)?.expect("case_id"),
        issue_id: helpers::row_text(row, 3)?.expect("issue_id"),
        role: helpers::row_text(row, 4)?.expect("role"),
        retired_at: helpers::row_text(row, 5)?,
        created_at: helpers::row_text(row, 6)?.expect("created_at"),
    })
}

fn row_to_blocker(row: &libsql::Row) -> Result<PipelineCaseBlockerRecord, libsql::Error> {
    Ok(PipelineCaseBlockerRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        case_id: helpers::row_text(row, 2)?.expect("case_id"),
        blocked_by_case_id: helpers::row_text(row, 3)?.expect("blocked_by_case_id"),
        created_at: helpers::row_text(row, 4)?.expect("created_at"),
    })
}

fn row_to_pipeline_document(row: &libsql::Row) -> Result<PipelineDocumentRecord, libsql::Error> {
    Ok(PipelineDocumentRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        pipeline_id: helpers::row_text(row, 2)?.expect("pipeline_id"),
        document_id: helpers::row_text(row, 3)?.expect("document_id"),
        key: helpers::row_text(row, 4)?.expect("key"),
        created_at: helpers::row_text(row, 5)?.expect("created_at"),
    })
}

fn row_to_case_document(row: &libsql::Row) -> Result<PipelineCaseDocumentRecord, libsql::Error> {
    Ok(PipelineCaseDocumentRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        case_id: helpers::row_text(row, 2)?.expect("case_id"),
        document_id: helpers::row_text(row, 3)?.expect("document_id"),
        key: helpers::row_text(row, 4)?.expect("key"),
        created_at: helpers::row_text(row, 5)?.expect("created_at"),
    })
}

fn row_to_automation(
    row: &libsql::Row,
) -> Result<PipelineAutomationExecutionRecord, libsql::Error> {
    Ok(PipelineAutomationExecutionRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        case_id: helpers::row_text(row, 2)?.expect("case_id"),
        automation_id: helpers::row_text(row, 3)?.expect("automation_id"),
        triggering_event_id: helpers::row_text(row, 4)?.expect("triggering_event_id"),
        routine_id: helpers::row_text(row, 5)?.expect("routine_id"),
        status: helpers::row_text(row, 6)?.expect("status"),
        execution_issue_id: helpers::row_text(row, 7)?,
        retry_of_execution_id: helpers::row_text(row, 8)?,
        generation: helpers::row_i64(row, 9)?,
        error: helpers::row_text(row, 10)?,
        created_at: helpers::row_text(row, 11)?.expect("created_at"),
    })
}

async fn ensure_case(
    conn: &libsql::Connection,
    company_id: &str,
    case_id: &str,
) -> Result<(), PipelineError> {
    if !helpers::row_belongs_to_company(conn, "pipeline_cases", case_id, company_id).await? {
        return Err(PipelineError::ReferenceNotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoPipelineRepository) {
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
             VALUES ('a1', 'c1', 'one', 'engineer', 'cli')",
            (),
        )
        .await
        .unwrap();
        (dir, TursoPipelineRepository::new(db))
    }

    #[tokio::test]
    async fn pipeline_stage_case_lifecycle() {
        let (_dir, repo) = repo().await;
        let pipeline = repo
            .create_pipeline(NewPipeline {
                company_id: "c1".to_owned(),
                project_id: None,
                key: "intake".to_owned(),
                name: "Intake".to_owned(),
                description: None,
                enforce_transitions: true,
                created_by_user_id: Some("u1".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(pipeline.key, "intake");

        let todo = repo
            .create_stage(NewStage {
                company_id: "c1".to_owned(),
                pipeline_id: pipeline.id.clone(),
                key: "todo".to_owned(),
                name: "To do".to_owned(),
                kind: "working".to_owned(),
                position: 1,
                config: None,
            })
            .await
            .unwrap();
        let done = repo
            .create_stage(NewStage {
                company_id: "c1".to_owned(),
                pipeline_id: pipeline.id.clone(),
                key: "done".to_owned(),
                name: "Done".to_owned(),
                kind: "done".to_owned(),
                position: 2,
                config: None,
            })
            .await
            .unwrap();
        assert_eq!(repo.list_stages("c1", &pipeline.id).await.unwrap().len(), 2);

        let transition = repo
            .create_transition(NewTransition {
                company_id: "c1".to_owned(),
                pipeline_id: pipeline.id.clone(),
                from_stage_id: todo.id.clone(),
                to_stage_id: done.id.clone(),
                label: Some("complete".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(transition.label.as_deref(), Some("complete"));

        let case = repo
            .create_case(NewPipelineCase {
                company_id: "c1".to_owned(),
                pipeline_id: pipeline.id.clone(),
                stage_id: todo.id.clone(),
                case_key: "case-1".to_owned(),
                title: "First case".to_owned(),
                summary: None,
                fields: Some(serde_json::json!({ "region": "cn" })),
                workspace_ref: None,
                parent_case_id: None,
                created_by_user_id: Some("u1".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(case.version, 1);
        assert_eq!(case.stage_id, todo.id);

        // Duplicate case key rejected.
        assert!(matches!(
            repo.create_case(NewPipelineCase {
                company_id: "c1".to_owned(),
                pipeline_id: pipeline.id.clone(),
                stage_id: todo.id.clone(),
                case_key: "case-1".to_owned(),
                title: "Dup".to_owned(),
                summary: None,
                fields: None,
                workspace_ref: None,
                parent_case_id: None,
                created_by_user_id: None,
            })
            .await
            .unwrap_err(),
            PipelineError::Duplicate
        ));

        // Enforced move along an undeclared edge is rejected.
        let cancelled = repo
            .create_stage(NewStage {
                company_id: "c1".to_owned(),
                pipeline_id: pipeline.id.clone(),
                key: "cancelled".to_owned(),
                name: "Cancelled".to_owned(),
                kind: "cancelled".to_owned(),
                position: 3,
                config: None,
            })
            .await
            .unwrap();
        assert!(matches!(
            repo.move_case(
                "c1",
                &case.id,
                &cancelled.id,
                "user",
                Some("u1".to_owned()),
                None,
                false
            )
            .await
            .unwrap_err(),
            PipelineError::TransitionNotAllowed
        ));
        // With force, move succeeds, terminal kind set, event recorded.
        let moved = repo
            .move_case(
                "c1",
                &case.id,
                &done.id,
                "user",
                Some("u1".to_owned()),
                None,
                true,
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(moved.stage_id, done.id);
        assert_eq!(moved.terminal_kind.as_deref(), Some("done"));
        assert!(moved.terminal_at.is_some());
        let events = repo.list_events("c1", &case.id).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].r#type, "transition_forced");

        // Non-enforced pipeline allows direct move (declare new pipeline).
        let open = repo
            .create_pipeline(NewPipeline {
                company_id: "c1".to_owned(),
                project_id: None,
                key: "open".to_owned(),
                name: "Open".to_owned(),
                description: None,
                enforce_transitions: false,
                created_by_user_id: None,
            })
            .await
            .unwrap();
        let s1 = repo
            .create_stage(NewStage {
                company_id: "c1".to_owned(),
                pipeline_id: open.id.clone(),
                key: "s1".to_owned(),
                name: "S1".to_owned(),
                kind: "working".to_owned(),
                position: 1,
                config: None,
            })
            .await
            .unwrap();
        let s2 = repo
            .create_stage(NewStage {
                company_id: "c1".to_owned(),
                pipeline_id: open.id.clone(),
                key: "s2".to_owned(),
                name: "S2".to_owned(),
                kind: "review".to_owned(),
                position: 2,
                config: None,
            })
            .await
            .unwrap();
        let open_case = repo
            .create_case(NewPipelineCase {
                company_id: "c1".to_owned(),
                pipeline_id: open.id.clone(),
                stage_id: s1.id.clone(),
                case_key: "c2".to_owned(),
                title: "Open case".to_owned(),
                summary: None,
                fields: None,
                workspace_ref: None,
                parent_case_id: None,
                created_by_user_id: None,
            })
            .await
            .unwrap();
        let moved = repo
            .move_case(
                "c1",
                &open_case.id,
                &s2.id,
                "agent",
                None,
                Some("a1".to_owned()),
                false,
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(moved.stage_id, s2.id);
        assert!(moved.terminal_kind.is_none());

        // Archive + list + cross-company get.
        assert!(
            repo.set_pipeline_archived("c1", &pipeline.id, true)
                .await
                .unwrap()
                .unwrap()
                .archived_at
                .is_some()
        );
        assert_eq!(repo.list_pipelines("c1").await.unwrap().len(), 2);
        assert!(
            repo.get_pipeline("c2", &pipeline.id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(repo.get_case("c2", &case.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn extension_tables_roundtrip() {
        let (_dir, repo) = repo().await;
        // Need a pipeline + case first.
        let pipeline = repo
            .create_pipeline(NewPipeline {
                company_id: "c1".to_owned(),
                project_id: None,
                key: "ext".to_owned(),
                name: "Ext".to_owned(),
                description: None,
                enforce_transitions: false,
                created_by_user_id: None,
            })
            .await
            .unwrap();
        let stage = repo
            .create_stage(NewStage {
                company_id: "c1".to_owned(),
                pipeline_id: pipeline.id.clone(),
                key: "s1".to_owned(),
                name: "S1".to_owned(),
                kind: "working".to_owned(),
                position: 1,
                config: None,
            })
            .await
            .unwrap();
        let case = repo
            .create_case(NewPipelineCase {
                company_id: "c1".to_owned(),
                pipeline_id: pipeline.id.clone(),
                stage_id: stage.id.clone(),
                case_key: "c-ext".to_owned(),
                title: "Ext case".to_owned(),
                summary: None,
                fields: None,
                workspace_ref: None,
                parent_case_id: None,
                created_by_user_id: None,
            })
            .await
            .unwrap();
        let other = repo
            .create_case(NewPipelineCase {
                company_id: "c1".to_owned(),
                pipeline_id: pipeline.id.clone(),
                stage_id: stage.id.clone(),
                case_key: "c-other".to_owned(),
                title: "Other".to_owned(),
                summary: None,
                fields: None,
                workspace_ref: None,
                parent_case_id: None,
                created_by_user_id: None,
            })
            .await
            .unwrap();
        // Seed issue, document, routine via SQL.
        let conn = crate::connect(&repo.db).await.unwrap();
        conn.execute(
            "INSERT INTO issues (id, company_id, title, issue_number, identifier)
             VALUES ('i1', 'c1', 'T', 1, 'ALPHA-1')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO documents (id, company_id, title, created_at, updated_at)
             VALUES ('d1', 'c1', 'Plan',
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO routines (id, company_id, title, created_at, updated_at)
             VALUES ('r1', 'c1', 'Nightly',
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            (),
        )
        .await
        .unwrap();

        // Issue links.
        let link = repo.link_issue("c1", &case.id, "i1", "work").await.unwrap();
        assert_eq!(link.role, "work");
        assert!(repo.list_issue_links("c1", &case.id).await.unwrap().len() == 1);
        assert!(repo.unlink_issue("c1", &case.id, "i1").await.unwrap());
        assert!(!repo.unlink_issue("c1", &case.id, "i1").await.unwrap());

        // Blockers.
        let blocker = repo.add_blocker("c1", &case.id, &other.id).await.unwrap();
        assert_eq!(blocker.blocked_by_case_id, other.id);
        assert!(repo.list_blockers("c1", &case.id).await.unwrap().len() == 1);
        assert!(
            repo.remove_blocker("c1", &case.id, &other.id)
                .await
                .unwrap()
        );
        // Self-block rejected.
        assert!(matches!(
            repo.add_blocker("c1", &case.id, &case.id)
                .await
                .unwrap_err(),
            PipelineError::ReferenceNotFound
        ));

        // Documents.
        let pdoc = repo
            .link_pipeline_document("c1", &pipeline.id, "d1", "plan")
            .await
            .unwrap();
        assert_eq!(pdoc.key, "plan");
        assert!(
            repo.list_pipeline_documents("c1", &pipeline.id)
                .await
                .unwrap()
                .len()
                == 1
        );
        let cdoc = repo
            .link_case_document("c1", &case.id, "d1", "plan")
            .await
            .unwrap();
        assert_eq!(cdoc.document_id, "d1");
        assert!(
            repo.list_case_documents("c1", &case.id)
                .await
                .unwrap()
                .len()
                == 1
        );

        // Automations.
        let automation = repo
            .record_automation(
                "c1",
                &case.id,
                "auto-1",
                "evt-1",
                "r1",
                "succeeded",
                None,
                None,
            )
            .await
            .unwrap();
        assert_eq!(automation.status, "succeeded");
        assert!(repo.list_automations("c1", &case.id).await.unwrap().len() == 1);
        // Idempotency duplicate rejected.
        assert!(matches!(
            repo.record_automation(
                "c1",
                &case.id,
                "auto-1",
                "evt-1",
                "r1",
                "failed",
                None,
                Some("x".to_owned())
            )
            .await
            .unwrap_err(),
            PipelineError::Duplicate
        ));
        // Cross-company reads are empty.
        assert!(
            repo.list_issue_links("c2", &case.id)
                .await
                .unwrap()
                .is_empty()
        );
        assert!(repo.list_blockers("c2", &case.id).await.unwrap().is_empty());
        assert!(
            repo.list_automations("c2", &case.id)
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn update_pipeline_settings() {
        let (_dir, repo) = repo().await;
        let pipeline = repo
            .create_pipeline(NewPipeline {
                company_id: "c1".to_owned(),
                project_id: None,
                key: "intake".to_owned(),
                name: "Intake".to_owned(),
                description: None,
                enforce_transitions: true,
                created_by_user_id: None,
            })
            .await
            .unwrap();

        let updated = repo
            .update_pipeline(
                "c1",
                &pipeline.id,
                Some("Intake v2".to_owned()),
                Some(Some("Primary intake pipeline".to_owned())),
                Some(true),
            )
            .await
            .unwrap()
            .expect("pipeline");
        assert_eq!(updated.name, "Intake v2");
        assert_eq!(
            updated.description.as_deref(),
            Some("Primary intake pipeline")
        );
        assert!(updated.archived_at.is_some());

        // Clear the description and unarchive.
        let updated = repo
            .update_pipeline("c1", &pipeline.id, None, Some(None), Some(false))
            .await
            .unwrap()
            .expect("pipeline");
        assert_eq!(updated.name, "Intake v2");
        assert!(updated.description.is_none());
        assert!(updated.archived_at.is_none());

        // Empty patch leaves the row untouched and still returns it.
        let updated = repo
            .update_pipeline("c1", &pipeline.id, None, None, None)
            .await
            .unwrap()
            .expect("pipeline");
        assert_eq!(updated.name, "Intake v2");

        // Cross-company update is not found.
        assert!(
            repo.update_pipeline("c2", &pipeline.id, Some("x".to_owned()), None, None)
                .await
                .unwrap()
                .is_none()
        );
    }
}
