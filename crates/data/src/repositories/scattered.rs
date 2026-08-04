//! Scattered domain: status cards, summary slots, smoke runs, feedback,
//! finance events, and document annotations (upstream batch 7).

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

macro_rules! json_col {
    ($row:expr, $idx:expr) => {
        helpers::row_text($row, $idx)?
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or_default()
    };
}

/// A status card (upstream status_cards.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusCardRecord {
    pub id: String,
    pub company_id: String,
    pub created_by_user_id: Option<String>,
    pub created_by_agent_id: Option<String>,
    pub title: Option<String>,
    pub title_pinned: bool,
    pub interest_prompt: String,
    pub queries: serde_json::Value,
    pub query_version: i64,
    pub query_compiled_at: Option<String>,
    pub query_compiled_by_agent_id: Option<String>,
    pub agent_id: Option<String>,
    pub refresh_policy: serde_json::Value,
    pub state: String,
    pub pending_change_count: i64,
    pub pending_change_hash: Option<String>,
    pub last_change_at: Option<String>,
    pub fingerprint: Option<serde_json::Value>,
    pub fingerprint_at: Option<String>,
    pub mentioned_issue_ids: serde_json::Value,
    pub document_id: Option<String>,
    pub last_update_run_kind: Option<String>,
    pub last_generated_at: Option<String>,
    pub last_model: Option<String>,
    pub generating_issue_id: Option<String>,
    pub failure_reason: Option<String>,
    pub next_eval_at: Option<String>,
    pub archived_at: Option<String>,
    pub archived_by_user_id: Option<String>,
    pub archived_by_agent_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a status card.
#[derive(Debug, Clone)]
pub struct NewStatusCard {
    pub company_id: String,
    pub created_by_user_id: Option<String>,
    pub created_by_agent_id: Option<String>,
    pub title: Option<String>,
    pub title_pinned: bool,
    pub interest_prompt: String,
    pub queries: serde_json::Value,
    pub query_version: i64,
    pub agent_id: Option<String>,
    pub refresh_policy: serde_json::Value,
    pub state: String,
    pub document_id: Option<String>,
}

/// A status card update (upstream status_card_updates).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusCardUpdateRecord {
    pub id: String,
    pub card_id: String,
    pub kind: String,
    pub trigger: String,
    pub generation_issue_id: Option<String>,
    pub run_id: Option<String>,
    pub changes: serde_json::Value,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_cents: i64,
    pub model: Option<String>,
    pub query_version: Option<i64>,
    pub change_summary: Option<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub status: String,
    pub error: Option<String>,
}

/// Input for creating a status card update.
#[derive(Debug, Clone)]
pub struct NewStatusCardUpdate {
    pub card_id: String,
    pub kind: String,
    pub trigger: String,
    pub generation_issue_id: Option<String>,
    pub run_id: Option<String>,
    pub changes: serde_json::Value,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_cents: i64,
    pub model: Option<String>,
    pub query_version: Option<i64>,
    pub change_summary: Option<String>,
    pub status: String,
    pub error: Option<String>,
}

/// A summary slot (upstream summary_slots.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SummarySlotRecord {
    pub id: String,
    pub company_id: String,
    pub scope_kind: String,
    pub scope_id: Option<String>,
    pub slot_key: String,
    pub document_id: Option<String>,
    pub status: String,
    pub failure_reason: Option<String>,
    pub generating_issue_id: Option<String>,
    pub last_generated_at: Option<String>,
    pub last_generated_by_agent_id: Option<String>,
    pub last_model: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for upserting a summary slot.
#[derive(Debug, Clone)]
pub struct NewSummarySlot {
    pub company_id: String,
    pub scope_kind: String,
    pub scope_id: Option<String>,
    pub slot_key: String,
    pub document_id: Option<String>,
    pub status: String,
    pub failure_reason: Option<String>,
    pub generating_issue_id: Option<String>,
    pub last_generated_at: Option<String>,
    pub last_generated_by_agent_id: Option<String>,
    pub last_model: Option<String>,
}

/// A smoke run (upstream smoke_lab.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmokeRunRecord {
    pub id: String,
    pub company_id: String,
    pub trigger: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub summary: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a smoke run.
#[derive(Debug, Clone)]
pub struct NewSmokeRun {
    pub company_id: String,
    pub trigger: String,
    pub status: String,
    pub finished_at: Option<String>,
    pub summary: serde_json::Value,
}

/// A smoke run step (upstream smoke_run_steps).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmokeRunStepRecord {
    pub id: String,
    pub company_id: String,
    pub run_id: String,
    pub path: String,
    pub scenario_step: String,
    pub status: String,
    pub detail: Option<String>,
    pub screenshot_artifact_ref: Option<serde_json::Value>,
    pub duration_ms: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for adding a smoke run step.
#[derive(Debug, Clone)]
pub struct NewSmokeRunStep {
    pub company_id: String,
    pub run_id: String,
    pub path: String,
    pub scenario_step: String,
    pub status: String,
    pub detail: Option<String>,
    pub screenshot_artifact_ref: Option<serde_json::Value>,
    pub duration_ms: Option<i64>,
}

/// A feedback vote (upstream feedback_votes.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackVoteRecord {
    pub id: String,
    pub company_id: String,
    pub issue_id: String,
    pub target_type: String,
    pub target_id: String,
    pub author_user_id: String,
    pub vote: String,
    pub reason: Option<String>,
    pub shared_with_labs: bool,
    pub shared_at: Option<String>,
    pub consent_version: Option<String>,
    pub redaction_summary: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a feedback vote.
#[derive(Debug, Clone)]
pub struct NewFeedbackVote {
    pub company_id: String,
    pub issue_id: String,
    pub target_type: String,
    pub target_id: String,
    pub author_user_id: String,
    pub vote: String,
    pub reason: Option<String>,
    pub shared_with_labs: bool,
    pub shared_at: Option<String>,
    pub consent_version: Option<String>,
    pub redaction_summary: Option<serde_json::Value>,
}

/// A feedback export (upstream feedback_exports.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackExportRecord {
    pub id: String,
    pub company_id: String,
    pub feedback_vote_id: String,
    pub issue_id: String,
    pub project_id: Option<String>,
    pub author_user_id: String,
    pub target_type: String,
    pub target_id: String,
    pub vote: String,
    pub status: String,
    pub destination: Option<String>,
    pub export_id: Option<String>,
    pub consent_version: Option<String>,
    pub schema_version: String,
    pub bundle_version: String,
    pub payload_version: String,
    pub payload_digest: Option<String>,
    pub payload_snapshot: Option<serde_json::Value>,
    pub target_summary: serde_json::Value,
    pub redaction_summary: Option<serde_json::Value>,
    pub attempt_count: i64,
    pub last_attempted_at: Option<String>,
    pub exported_at: Option<String>,
    pub failure_reason: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a feedback export.
#[derive(Debug, Clone)]
pub struct NewFeedbackExport {
    pub company_id: String,
    pub feedback_vote_id: String,
    pub issue_id: String,
    pub project_id: Option<String>,
    pub author_user_id: String,
    pub target_type: String,
    pub target_id: String,
    pub vote: String,
    pub status: String,
    pub destination: Option<String>,
    pub export_id: Option<String>,
    pub consent_version: Option<String>,
    pub payload_digest: Option<String>,
    pub payload_snapshot: Option<serde_json::Value>,
    pub target_summary: serde_json::Value,
    pub redaction_summary: Option<serde_json::Value>,
    pub attempt_count: i64,
    pub exported_at: Option<String>,
    pub failure_reason: Option<String>,
}

/// A finance event (upstream finance_events.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinanceEventRecord {
    pub id: String,
    pub company_id: String,
    pub agent_id: Option<String>,
    pub issue_id: Option<String>,
    pub project_id: Option<String>,
    pub goal_id: Option<String>,
    pub heartbeat_run_id: Option<String>,
    pub cost_event_id: Option<String>,
    pub billing_code: Option<String>,
    pub description: Option<String>,
    pub event_kind: String,
    pub direction: String,
    pub biller: String,
    pub provider: Option<String>,
    pub execution_adapter_type: Option<String>,
    pub pricing_tier: Option<String>,
    pub region: Option<String>,
    pub model: Option<String>,
    pub quantity: Option<i64>,
    pub unit: Option<String>,
    pub amount_cents: i64,
    pub currency: String,
    pub estimated: bool,
    pub external_invoice_id: Option<String>,
    pub metadata_json: Option<serde_json::Value>,
    pub occurred_at: String,
    pub created_at: String,
}

/// Input for creating a finance event.
#[derive(Debug, Clone)]
pub struct NewFinanceEvent {
    pub company_id: String,
    pub agent_id: Option<String>,
    pub issue_id: Option<String>,
    pub project_id: Option<String>,
    pub goal_id: Option<String>,
    pub heartbeat_run_id: Option<String>,
    pub cost_event_id: Option<String>,
    pub billing_code: Option<String>,
    pub description: Option<String>,
    pub event_kind: String,
    pub direction: String,
    pub biller: String,
    pub provider: Option<String>,
    pub execution_adapter_type: Option<String>,
    pub pricing_tier: Option<String>,
    pub region: Option<String>,
    pub model: Option<String>,
    pub quantity: Option<i64>,
    pub unit: Option<String>,
    pub amount_cents: i64,
    pub currency: String,
    pub estimated: bool,
    pub external_invoice_id: Option<String>,
    pub metadata_json: Option<serde_json::Value>,
    pub occurred_at: String,
}

/// A document annotation thread (upstream document_annotation_threads.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationThreadRecord {
    pub id: String,
    pub company_id: String,
    pub issue_id: Option<String>,
    pub routine_id: Option<String>,
    pub case_id: Option<String>,
    pub document_id: String,
    pub document_key: String,
    pub status: String,
    pub anchor_state: String,
    pub original_revision_id: Option<String>,
    pub original_revision_number: i64,
    pub current_revision_id: Option<String>,
    pub current_revision_number: i64,
    pub selected_text: String,
    pub prefix_text: String,
    pub suffix_text: String,
    pub normalized_start: i64,
    pub normalized_end: i64,
    pub markdown_start: i64,
    pub markdown_end: i64,
    pub anchor_confidence: String,
    pub anchor_selector: serde_json::Value,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
    pub resolved_by_agent_id: Option<String>,
    pub resolved_by_user_id: Option<String>,
    pub resolved_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating an annotation thread.
#[derive(Debug, Clone)]
pub struct NewAnnotationThread {
    pub company_id: String,
    pub issue_id: Option<String>,
    pub routine_id: Option<String>,
    pub case_id: Option<String>,
    pub document_id: String,
    pub document_key: String,
    pub status: String,
    pub anchor_state: String,
    pub original_revision_id: Option<String>,
    pub original_revision_number: i64,
    pub current_revision_id: Option<String>,
    pub current_revision_number: i64,
    pub selected_text: String,
    pub prefix_text: String,
    pub suffix_text: String,
    pub normalized_start: i64,
    pub normalized_end: i64,
    pub markdown_start: i64,
    pub markdown_end: i64,
    pub anchor_confidence: String,
    pub anchor_selector: serde_json::Value,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
}

/// A document annotation comment (upstream document_annotation_comments.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationCommentRecord {
    pub id: String,
    pub company_id: String,
    pub thread_id: String,
    pub issue_id: Option<String>,
    pub routine_id: Option<String>,
    pub case_id: Option<String>,
    pub document_id: String,
    pub body: String,
    pub author_type: String,
    pub author_agent_id: Option<String>,
    pub author_user_id: Option<String>,
    pub created_by_run_id: Option<String>,
    pub issue_comment_id: Option<String>,
    pub source_trust: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for adding an annotation comment.
#[derive(Debug, Clone)]
pub struct NewAnnotationComment {
    pub company_id: String,
    pub thread_id: String,
    pub issue_id: Option<String>,
    pub routine_id: Option<String>,
    pub case_id: Option<String>,
    pub document_id: String,
    pub body: String,
    pub author_type: String,
    pub author_agent_id: Option<String>,
    pub author_user_id: Option<String>,
    pub created_by_run_id: Option<String>,
    pub issue_comment_id: Option<String>,
    pub source_trust: Option<serde_json::Value>,
}

/// A document annotation anchor snapshot.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnchorSnapshotRecord {
    pub id: String,
    pub company_id: String,
    pub thread_id: String,
    pub document_id: String,
    pub from_revision_id: Option<String>,
    pub from_revision_number: Option<i64>,
    pub to_revision_id: Option<String>,
    pub to_revision_number: i64,
    pub previous_anchor: serde_json::Value,
    pub next_anchor: Option<serde_json::Value>,
    pub anchor_state: String,
    pub anchor_confidence: String,
    pub failure_reason: Option<String>,
    pub created_at: String,
}

/// Input for creating an anchor snapshot.
#[derive(Debug, Clone)]
pub struct NewAnchorSnapshot {
    pub company_id: String,
    pub thread_id: String,
    pub document_id: String,
    pub from_revision_id: Option<String>,
    pub from_revision_number: Option<i64>,
    pub to_revision_id: Option<String>,
    pub to_revision_number: i64,
    pub previous_anchor: serde_json::Value,
    pub next_anchor: Option<serde_json::Value>,
    pub anchor_state: String,
    pub anchor_confidence: String,
    pub failure_reason: Option<String>,
}

/// Scattered domain repository errors.
#[derive(Debug, Error)]
pub enum ScatteredError {
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    #[error("company not found")]
    CompanyNotFound,
    #[error("reference not found")]
    ReferenceNotFound,
    #[error("record already exists")]
    AlreadyExists,
    #[error("record not found")]
    NotFound,
}

/// Scattered domain persistence contract.
#[async_trait]
pub trait ScatteredRepository: Send + Sync {
    async fn create_status_card(
        &self,
        input: NewStatusCard,
    ) -> Result<StatusCardRecord, ScatteredError>;
    async fn list_status_cards(
        &self,
        company_id: &str,
    ) -> Result<Vec<StatusCardRecord>, ScatteredError>;
    async fn archive_status_card(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<StatusCardRecord>, ScatteredError>;
    async fn create_status_card_update(
        &self,
        input: NewStatusCardUpdate,
    ) -> Result<StatusCardUpdateRecord, ScatteredError>;
    async fn list_status_card_updates(
        &self,
        card_id: &str,
    ) -> Result<Vec<StatusCardUpdateRecord>, ScatteredError>;
    async fn upsert_summary_slot(
        &self,
        input: NewSummarySlot,
    ) -> Result<SummarySlotRecord, ScatteredError>;
    async fn list_summary_slots(
        &self,
        company_id: &str,
    ) -> Result<Vec<SummarySlotRecord>, ScatteredError>;
    async fn create_smoke_run(&self, input: NewSmokeRun) -> Result<SmokeRunRecord, ScatteredError>;
    async fn list_smoke_runs(
        &self,
        company_id: &str,
    ) -> Result<Vec<SmokeRunRecord>, ScatteredError>;
    async fn add_smoke_step(
        &self,
        input: NewSmokeRunStep,
    ) -> Result<SmokeRunStepRecord, ScatteredError>;
    async fn list_smoke_steps(
        &self,
        company_id: &str,
        run_id: &str,
    ) -> Result<Vec<SmokeRunStepRecord>, ScatteredError>;
    async fn create_feedback_vote(
        &self,
        input: NewFeedbackVote,
    ) -> Result<FeedbackVoteRecord, ScatteredError>;
    async fn list_feedback_votes(
        &self,
        company_id: &str,
        issue_id: &str,
    ) -> Result<Vec<FeedbackVoteRecord>, ScatteredError>;
    async fn create_feedback_export(
        &self,
        input: NewFeedbackExport,
    ) -> Result<FeedbackExportRecord, ScatteredError>;
    async fn list_feedback_exports(
        &self,
        company_id: &str,
    ) -> Result<Vec<FeedbackExportRecord>, ScatteredError>;
    async fn create_finance_event(
        &self,
        input: NewFinanceEvent,
    ) -> Result<FinanceEventRecord, ScatteredError>;
    async fn list_finance_events(
        &self,
        company_id: &str,
    ) -> Result<Vec<FinanceEventRecord>, ScatteredError>;
    async fn create_annotation_thread(
        &self,
        input: NewAnnotationThread,
    ) -> Result<AnnotationThreadRecord, ScatteredError>;
    async fn list_annotation_threads(
        &self,
        company_id: &str,
        document_id: &str,
    ) -> Result<Vec<AnnotationThreadRecord>, ScatteredError>;
    async fn add_annotation_comment(
        &self,
        input: NewAnnotationComment,
    ) -> Result<AnnotationCommentRecord, ScatteredError>;
    async fn list_annotation_comments(
        &self,
        company_id: &str,
        thread_id: &str,
    ) -> Result<Vec<AnnotationCommentRecord>, ScatteredError>;
    async fn create_anchor_snapshot(
        &self,
        input: NewAnchorSnapshot,
    ) -> Result<AnchorSnapshotRecord, ScatteredError>;
    async fn list_anchor_snapshots(
        &self,
        company_id: &str,
        thread_id: &str,
    ) -> Result<Vec<AnchorSnapshotRecord>, ScatteredError>;
}

/// Turso/libSQL implementation of [`ScatteredRepository`].
#[derive(Debug)]
pub struct TursoScatteredRepository {
    db: Database,
}

impl TursoScatteredRepository {
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

const STATUS_CARD_COLUMNS: &str = "id, company_id, created_by_user_id, created_by_agent_id, title,
                                   title_pinned, interest_prompt, queries, query_version,
                                   query_compiled_at, query_compiled_by_agent_id, agent_id,
                                   refresh_policy, state, pending_change_count,
                                   pending_change_hash, last_change_at, fingerprint,
                                   fingerprint_at, mentioned_issue_ids, document_id,
                                   last_update_run_kind, last_generated_at, last_model,
                                   generating_issue_id, failure_reason, next_eval_at,
                                   archived_at, archived_by_user_id, archived_by_agent_id,
                                   created_at, updated_at";
const CARD_UPDATE_COLUMNS: &str = "id, card_id, kind, trigger, generation_issue_id, run_id,
                                   changes, input_tokens, output_tokens, cost_cents, model,
                                   query_version, change_summary, started_at, finished_at,
                                   status, error";
const SUMMARY_SLOT_COLUMNS: &str = "id, company_id, scope_kind, scope_id, slot_key, document_id,
                                    status, failure_reason, generating_issue_id,
                                    last_generated_at, last_generated_by_agent_id, last_model,
                                    created_at, updated_at";
const SMOKE_RUN_COLUMNS: &str = "id, company_id, trigger, status, started_at, finished_at,
                                 summary, created_at, updated_at";
const SMOKE_STEP_COLUMNS: &str = "id, company_id, run_id, path, scenario_step, status, detail,
                                  screenshot_artifact_ref, duration_ms, created_at, updated_at";
const FEEDBACK_VOTE_COLUMNS: &str = "id, company_id, issue_id, target_type, target_id,
                                     author_user_id, vote, reason, shared_with_labs, shared_at,
                                     consent_version, redaction_summary, created_at, updated_at";
const FEEDBACK_EXPORT_COLUMNS: &str = "id, company_id, feedback_vote_id, issue_id, project_id,
                                       author_user_id, target_type, target_id, vote, status,
                                       destination, export_id, consent_version, schema_version,
                                       bundle_version, payload_version, payload_digest,
                                       payload_snapshot, target_summary, redaction_summary,
                                       attempt_count, last_attempted_at, exported_at,
                                       failure_reason, created_at, updated_at";
const FINANCE_EVENT_COLUMNS: &str = "id, company_id, agent_id, issue_id, project_id, goal_id,
                                     heartbeat_run_id, cost_event_id, billing_code, description,
                                     event_kind, direction, biller, provider,
                                     execution_adapter_type, pricing_tier, region, model,
                                     quantity, unit, amount_cents, currency, estimated,
                                     external_invoice_id, metadata_json, occurred_at, created_at";
const THREAD_COLUMNS: &str = "id, company_id, issue_id, routine_id, case_id, document_id,
                              document_key, status, anchor_state, original_revision_id,
                              original_revision_number, current_revision_id,
                              current_revision_number, selected_text, prefix_text, suffix_text,
                              normalized_start, normalized_end, markdown_start, markdown_end,
                              anchor_confidence, anchor_selector, created_by_agent_id,
                              created_by_user_id, resolved_by_agent_id, resolved_by_user_id,
                              resolved_at, created_at, updated_at";
const ANNOTATION_COMMENT_COLUMNS: &str = "id, company_id, thread_id, issue_id, routine_id,
                                          case_id, document_id, body, author_type,
                                          author_agent_id, author_user_id, created_by_run_id,
                                          issue_comment_id, source_trust, created_at, updated_at";
const ANCHOR_COLUMNS: &str = "id, company_id, thread_id, document_id, from_revision_id,
                              from_revision_number, to_revision_id, to_revision_number,
                              previous_anchor, next_anchor, anchor_state, anchor_confidence,
                              failure_reason, created_at";

#[async_trait]
impl ScatteredRepository for TursoScatteredRepository {
    async fn create_status_card(
        &self,
        input: NewStatusCard,
    ) -> Result<StatusCardRecord, ScatteredError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ScatteredError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO status_cards (id, company_id, created_by_user_id,
                                       created_by_agent_id, title, title_pinned,
                                       interest_prompt, queries, query_version, agent_id,
                                       refresh_policy, state, document_id, created_at,
                                       updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.created_by_user_id,
                input.created_by_agent_id,
                input.title,
                i64::from(input.title_pinned),
                input.interest_prompt,
                input.queries.to_string(),
                input.query_version,
                input.agent_id,
                input.refresh_policy.to_string(),
                input.state,
                input.document_id
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {STATUS_CARD_COLUMNS} FROM status_cards WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("card was just inserted");
        Ok(row_to_status_card(&row)?)
    }

    async fn list_status_cards(
        &self,
        company_id: &str,
    ) -> Result<Vec<StatusCardRecord>, ScatteredError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {STATUS_CARD_COLUMNS} FROM status_cards WHERE company_id = ?1
                     ORDER BY created_at DESC"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut cards = Vec::new();
        while let Some(row) = rows.next().await? {
            cards.push(row_to_status_card(&row)?);
        }
        Ok(cards)
    }

    async fn archive_status_card(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<StatusCardRecord>, ScatteredError> {
        let conn = crate::connection::connect(&self.db).await?;
        let updated = conn
            .execute(
                "UPDATE status_cards
                 SET archived_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE id = ?1 AND company_id = ?2",
                libsql::params![id, company_id],
            )
            .await?;
        if updated == 0 {
            return Ok(None);
        }
        let mut rows = conn
            .query(
                &format!("SELECT {STATUS_CARD_COLUMNS} FROM status_cards WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("card exists");
        Ok(Some(row_to_status_card(&row)?))
    }

    async fn create_status_card_update(
        &self,
        input: NewStatusCardUpdate,
    ) -> Result<StatusCardUpdateRecord, ScatteredError> {
        let conn = crate::connection::connect(&self.db).await?;
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO status_card_updates (id, card_id, kind, trigger,
                                              generation_issue_id, run_id, changes,
                                              input_tokens, output_tokens, cost_cents,
                                              model, query_version, change_summary,
                                              started_at, status, error)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'), ?14, ?15)",
            libsql::params![
                id.clone(),
                input.card_id,
                input.kind,
                input.trigger,
                input.generation_issue_id,
                input.run_id,
                input.changes.to_string(),
                input.input_tokens,
                input.output_tokens,
                input.cost_cents,
                input.model,
                input.query_version,
                input.change_summary,
                input.status,
                input.error
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {CARD_UPDATE_COLUMNS} FROM status_card_updates WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("update was just inserted");
        Ok(row_to_card_update(&row)?)
    }

    async fn list_status_card_updates(
        &self,
        card_id: &str,
    ) -> Result<Vec<StatusCardUpdateRecord>, ScatteredError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {CARD_UPDATE_COLUMNS} FROM status_card_updates WHERE card_id = ?1
                     ORDER BY started_at DESC"
                ),
                libsql::params![card_id],
            )
            .await?;
        let mut updates = Vec::new();
        while let Some(row) = rows.next().await? {
            updates.push(row_to_card_update(&row)?);
        }
        Ok(updates)
    }

    async fn upsert_summary_slot(
        &self,
        input: NewSummarySlot,
    ) -> Result<SummarySlotRecord, ScatteredError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ScatteredError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let now = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";
        let key_company_id = input.company_id.clone();
        let key_scope_kind = input.scope_kind.clone();
        let key_scope_id = input.scope_id.clone();
        let key_slot_key = input.slot_key.clone();
        conn.execute(
            &format!(
                "INSERT INTO summary_slots (id, company_id, scope_kind, scope_id, slot_key,
                                            document_id, status, failure_reason,
                                            generating_issue_id, last_generated_at,
                                            last_generated_by_agent_id, last_model, created_at,
                                            updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, {now}, {now})
                 ON CONFLICT (company_id, scope_kind, COALESCE(scope_id, ''), slot_key) DO UPDATE SET
                   document_id = excluded.document_id,
                   status = excluded.status,
                   failure_reason = excluded.failure_reason,
                   generating_issue_id = excluded.generating_issue_id,
                   last_generated_at = excluded.last_generated_at,
                   last_generated_by_agent_id = excluded.last_generated_by_agent_id,
                   last_model = excluded.last_model,
                   updated_at = {now}",
                now = now
            ),
            libsql::params![
                id.clone(),
                input.company_id,
                input.scope_kind,
                input.scope_id,
                input.slot_key,
                input.document_id,
                input.status,
                input.failure_reason,
                input.generating_issue_id,
                input.last_generated_at,
                input.last_generated_by_agent_id,
                input.last_model
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {SUMMARY_SLOT_COLUMNS} FROM summary_slots
                     WHERE company_id = ?1 AND scope_kind = ?2
                       AND COALESCE(scope_id, '') = COALESCE(?3, '') AND slot_key = ?4"
                ),
                libsql::params![key_company_id, key_scope_kind, key_scope_id, key_slot_key],
            )
            .await?;
        let row = rows.next().await?.expect("slot was just upserted");
        Ok(row_to_summary_slot(&row)?)
    }

    async fn list_summary_slots(
        &self,
        company_id: &str,
    ) -> Result<Vec<SummarySlotRecord>, ScatteredError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {SUMMARY_SLOT_COLUMNS} FROM summary_slots WHERE company_id = ?1
                     ORDER BY updated_at DESC"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut slots = Vec::new();
        while let Some(row) = rows.next().await? {
            slots.push(row_to_summary_slot(&row)?);
        }
        Ok(slots)
    }

    async fn create_smoke_run(&self, input: NewSmokeRun) -> Result<SmokeRunRecord, ScatteredError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ScatteredError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO smoke_runs (id, company_id, trigger, status, started_at, finished_at,
                                     summary, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ','now'), ?5, ?6,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.trigger,
                input.status,
                input.finished_at,
                input.summary.to_string()
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {SMOKE_RUN_COLUMNS} FROM smoke_runs WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("run was just inserted");
        Ok(row_to_smoke_run(&row)?)
    }

    async fn list_smoke_runs(
        &self,
        company_id: &str,
    ) -> Result<Vec<SmokeRunRecord>, ScatteredError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {SMOKE_RUN_COLUMNS} FROM smoke_runs WHERE company_id = ?1
                     ORDER BY started_at DESC"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut runs = Vec::new();
        while let Some(row) = rows.next().await? {
            runs.push(row_to_smoke_run(&row)?);
        }
        Ok(runs)
    }

    async fn add_smoke_step(
        &self,
        input: NewSmokeRunStep,
    ) -> Result<SmokeRunStepRecord, ScatteredError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(&conn, "smoke_runs", &input.run_id, &input.company_id)
            .await?
        {
            return Err(ScatteredError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO smoke_run_steps (id, company_id, run_id, path, scenario_step, status,
                                          detail, screenshot_artifact_ref, duration_ms,
                                          created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.run_id,
                input.path,
                input.scenario_step,
                input.status,
                input.detail,
                input.screenshot_artifact_ref.map(|v| v.to_string()),
                input.duration_ms
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {SMOKE_STEP_COLUMNS} FROM smoke_run_steps WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("step was just inserted");
        Ok(row_to_smoke_step(&row)?)
    }

    async fn list_smoke_steps(
        &self,
        company_id: &str,
        run_id: &str,
    ) -> Result<Vec<SmokeRunStepRecord>, ScatteredError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {SMOKE_STEP_COLUMNS} FROM smoke_run_steps
                     WHERE company_id = ?1 AND run_id = ?2 ORDER BY created_at"
                ),
                libsql::params![company_id, run_id],
            )
            .await?;
        let mut steps = Vec::new();
        while let Some(row) = rows.next().await? {
            steps.push(row_to_smoke_step(&row)?);
        }
        Ok(steps)
    }

    async fn create_feedback_vote(
        &self,
        input: NewFeedbackVote,
    ) -> Result<FeedbackVoteRecord, ScatteredError> {
        let conn = crate::connection::connect(&self.db).await?;
        if helpers::row_company(&conn, "issues", &input.issue_id).await?
            != Some(input.company_id.clone())
        {
            return Err(ScatteredError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO feedback_votes (id, company_id, issue_id, target_type, target_id,
                                             author_user_id, vote, reason, shared_with_labs,
                                             shared_at, consent_version, redaction_summary,
                                             created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.issue_id,
                    input.target_type,
                    input.target_id,
                    input.author_user_id,
                    input.vote,
                    input.reason,
                    i64::from(input.shared_with_labs),
                    input.shared_at,
                    input.consent_version,
                    input.redaction_summary.map(|v| v.to_string())
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!(
                            "SELECT {FEEDBACK_VOTE_COLUMNS} FROM feedback_votes WHERE id = ?1"
                        ),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("vote was just inserted");
                Ok(row_to_feedback_vote(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(ScatteredError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_feedback_votes(
        &self,
        company_id: &str,
        issue_id: &str,
    ) -> Result<Vec<FeedbackVoteRecord>, ScatteredError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {FEEDBACK_VOTE_COLUMNS} FROM feedback_votes
                     WHERE company_id = ?1 AND issue_id = ?2 ORDER BY created_at"
                ),
                libsql::params![company_id, issue_id],
            )
            .await?;
        let mut votes = Vec::new();
        while let Some(row) = rows.next().await? {
            votes.push(row_to_feedback_vote(&row)?);
        }
        Ok(votes)
    }

    async fn create_feedback_export(
        &self,
        input: NewFeedbackExport,
    ) -> Result<FeedbackExportRecord, ScatteredError> {
        let conn = crate::connection::connect(&self.db).await?;
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO feedback_exports (id, company_id, feedback_vote_id, issue_id,
                                               project_id, author_user_id, target_type,
                                               target_id, vote, status, destination, export_id,
                                               consent_version, payload_digest, payload_snapshot,
                                               target_summary, redaction_summary, attempt_count,
                                               exported_at, failure_reason, created_at,
                                               updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                         ?16, ?17, ?18, ?19, ?20,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.feedback_vote_id,
                    input.issue_id,
                    input.project_id,
                    input.author_user_id,
                    input.target_type,
                    input.target_id,
                    input.vote,
                    input.status,
                    input.destination,
                    input.export_id,
                    input.consent_version,
                    input.payload_digest,
                    input.payload_snapshot.map(|v| v.to_string()),
                    input.target_summary.to_string(),
                    input.redaction_summary.map(|v| v.to_string()),
                    input.attempt_count,
                    input.exported_at,
                    input.failure_reason
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!(
                            "SELECT {FEEDBACK_EXPORT_COLUMNS} FROM feedback_exports WHERE id = ?1"
                        ),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("export was just inserted");
                Ok(row_to_feedback_export(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(ScatteredError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_feedback_exports(
        &self,
        company_id: &str,
    ) -> Result<Vec<FeedbackExportRecord>, ScatteredError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {FEEDBACK_EXPORT_COLUMNS} FROM feedback_exports
                     WHERE company_id = ?1 ORDER BY created_at DESC"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut exports = Vec::new();
        while let Some(row) = rows.next().await? {
            exports.push(row_to_feedback_export(&row)?);
        }
        Ok(exports)
    }

    async fn create_finance_event(
        &self,
        input: NewFinanceEvent,
    ) -> Result<FinanceEventRecord, ScatteredError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ScatteredError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO finance_events (id, company_id, agent_id, issue_id, project_id,
                                         goal_id, heartbeat_run_id, cost_event_id, billing_code,
                                         description, event_kind, direction, biller, provider,
                                         execution_adapter_type, pricing_tier, region, model,
                                         quantity, unit, amount_cents, currency, estimated,
                                         external_invoice_id, metadata_json, occurred_at,
                                         created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                     ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.agent_id,
                input.issue_id,
                input.project_id,
                input.goal_id,
                input.heartbeat_run_id,
                input.cost_event_id,
                input.billing_code,
                input.description,
                input.event_kind,
                input.direction,
                input.biller,
                input.provider,
                input.execution_adapter_type,
                input.pricing_tier,
                input.region,
                input.model,
                input.quantity,
                input.unit,
                input.amount_cents,
                input.currency,
                i64::from(input.estimated),
                input.external_invoice_id,
                input.metadata_json.map(|v| v.to_string()),
                input.occurred_at
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {FINANCE_EVENT_COLUMNS} FROM finance_events WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("event was just inserted");
        Ok(row_to_finance_event(&row)?)
    }

    async fn list_finance_events(
        &self,
        company_id: &str,
    ) -> Result<Vec<FinanceEventRecord>, ScatteredError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {FINANCE_EVENT_COLUMNS} FROM finance_events WHERE company_id = ?1
                     ORDER BY occurred_at DESC"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut events = Vec::new();
        while let Some(row) = rows.next().await? {
            events.push(row_to_finance_event(&row)?);
        }
        Ok(events)
    }

    async fn create_annotation_thread(
        &self,
        input: NewAnnotationThread,
    ) -> Result<AnnotationThreadRecord, ScatteredError> {
        let conn = crate::connection::connect(&self.db).await?;
        if helpers::row_company(&conn, "documents", &input.document_id).await?
            != Some(input.company_id.clone())
        {
            return Err(ScatteredError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO document_annotation_threads (id, company_id, issue_id, routine_id,
                                                       case_id, document_id, document_key,
                                                       status, anchor_state, original_revision_id,
                                                       original_revision_number,
                                                       current_revision_id,
                                                       current_revision_number, selected_text,
                                                       prefix_text, suffix_text, normalized_start,
                                                       normalized_end, markdown_start,
                                                       markdown_end, anchor_confidence,
                                                       anchor_selector, created_by_agent_id,
                                                       created_by_user_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                     ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.issue_id,
                input.routine_id,
                input.case_id,
                input.document_id,
                input.document_key,
                input.status,
                input.anchor_state,
                input.original_revision_id,
                input.original_revision_number,
                input.current_revision_id,
                input.current_revision_number,
                input.selected_text,
                input.prefix_text,
                input.suffix_text,
                input.normalized_start,
                input.normalized_end,
                input.markdown_start,
                input.markdown_end,
                input.anchor_confidence,
                input.anchor_selector.to_string(),
                input.created_by_agent_id,
                input.created_by_user_id
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {THREAD_COLUMNS} FROM document_annotation_threads WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("thread was just inserted");
        Ok(row_to_thread(&row)?)
    }

    async fn list_annotation_threads(
        &self,
        company_id: &str,
        document_id: &str,
    ) -> Result<Vec<AnnotationThreadRecord>, ScatteredError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {THREAD_COLUMNS} FROM document_annotation_threads
                     WHERE company_id = ?1 AND document_id = ?2 ORDER BY created_at"
                ),
                libsql::params![company_id, document_id],
            )
            .await?;
        let mut threads = Vec::new();
        while let Some(row) = rows.next().await? {
            threads.push(row_to_thread(&row)?);
        }
        Ok(threads)
    }

    async fn add_annotation_comment(
        &self,
        input: NewAnnotationComment,
    ) -> Result<AnnotationCommentRecord, ScatteredError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(
            &conn,
            "document_annotation_threads",
            &input.thread_id,
            &input.company_id,
        )
        .await?
        {
            return Err(ScatteredError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO document_annotation_comments (id, company_id, thread_id, issue_id,
                                                        routine_id, case_id, document_id, body,
                                                        author_type, author_agent_id,
                                                        author_user_id, created_by_run_id,
                                                        issue_comment_id, source_trust,
                                                        created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.thread_id,
                input.issue_id,
                input.routine_id,
                input.case_id,
                input.document_id,
                input.body,
                input.author_type,
                input.author_agent_id,
                input.author_user_id,
                input.created_by_run_id,
                input.issue_comment_id,
                input.source_trust.map(|v| v.to_string())
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {ANNOTATION_COMMENT_COLUMNS} FROM document_annotation_comments WHERE id = ?1"
                ),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("comment was just inserted");
        Ok(row_to_annotation_comment(&row)?)
    }

    async fn list_annotation_comments(
        &self,
        company_id: &str,
        thread_id: &str,
    ) -> Result<Vec<AnnotationCommentRecord>, ScatteredError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {ANNOTATION_COMMENT_COLUMNS} FROM document_annotation_comments
                     WHERE company_id = ?1 AND thread_id = ?2 ORDER BY created_at"
                ),
                libsql::params![company_id, thread_id],
            )
            .await?;
        let mut comments = Vec::new();
        while let Some(row) = rows.next().await? {
            comments.push(row_to_annotation_comment(&row)?);
        }
        Ok(comments)
    }

    async fn create_anchor_snapshot(
        &self,
        input: NewAnchorSnapshot,
    ) -> Result<AnchorSnapshotRecord, ScatteredError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(
            &conn,
            "document_annotation_threads",
            &input.thread_id,
            &input.company_id,
        )
        .await?
        {
            return Err(ScatteredError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO document_annotation_anchor_snapshots (id, company_id, thread_id,
                                                                document_id, from_revision_id,
                                                                from_revision_number,
                                                                to_revision_id,
                                                                to_revision_number,
                                                                previous_anchor, next_anchor,
                                                                anchor_state, anchor_confidence,
                                                                failure_reason, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.thread_id,
                input.document_id,
                input.from_revision_id,
                input.from_revision_number,
                input.to_revision_id,
                input.to_revision_number,
                input.previous_anchor.to_string(),
                input.next_anchor.map(|v| v.to_string()),
                input.anchor_state,
                input.anchor_confidence,
                input.failure_reason
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {ANCHOR_COLUMNS} FROM document_annotation_anchor_snapshots WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("snapshot was just inserted");
        Ok(row_to_anchor(&row)?)
    }

    async fn list_anchor_snapshots(
        &self,
        company_id: &str,
        thread_id: &str,
    ) -> Result<Vec<AnchorSnapshotRecord>, ScatteredError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {ANCHOR_COLUMNS} FROM document_annotation_anchor_snapshots
                     WHERE company_id = ?1 AND thread_id = ?2 ORDER BY created_at"
                ),
                libsql::params![company_id, thread_id],
            )
            .await?;
        let mut snapshots = Vec::new();
        while let Some(row) = rows.next().await? {
            snapshots.push(row_to_anchor(&row)?);
        }
        Ok(snapshots)
    }
}

fn row_to_status_card(row: &libsql::Row) -> Result<StatusCardRecord, libsql::Error> {
    Ok(StatusCardRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        created_by_user_id: helpers::row_text(row, 2)?,
        created_by_agent_id: helpers::row_text(row, 3)?,
        title: helpers::row_text(row, 4)?,
        title_pinned: helpers::row_i64(row, 5)? != 0,
        interest_prompt: helpers::row_text(row, 6)?.expect("interest_prompt"),
        queries: json_col!(row, 7),
        query_version: helpers::row_i64(row, 8)?,
        query_compiled_at: helpers::row_text(row, 9)?,
        query_compiled_by_agent_id: helpers::row_text(row, 10)?,
        agent_id: helpers::row_text(row, 11)?,
        refresh_policy: json_col!(row, 12),
        state: helpers::row_text(row, 13)?.expect("state"),
        pending_change_count: helpers::row_i64(row, 14)?,
        pending_change_hash: helpers::row_text(row, 15)?,
        last_change_at: helpers::row_text(row, 16)?,
        fingerprint: helpers::row_text(row, 17)?.and_then(|v| serde_json::from_str(&v).ok()),
        fingerprint_at: helpers::row_text(row, 18)?,
        mentioned_issue_ids: json_col!(row, 19),
        document_id: helpers::row_text(row, 20)?,
        last_update_run_kind: helpers::row_text(row, 21)?,
        last_generated_at: helpers::row_text(row, 22)?,
        last_model: helpers::row_text(row, 23)?,
        generating_issue_id: helpers::row_text(row, 24)?,
        failure_reason: helpers::row_text(row, 25)?,
        next_eval_at: helpers::row_text(row, 26)?,
        archived_at: helpers::row_text(row, 27)?,
        archived_by_user_id: helpers::row_text(row, 28)?,
        archived_by_agent_id: helpers::row_text(row, 29)?,
        created_at: helpers::row_text(row, 30)?.expect("created_at"),
        updated_at: helpers::row_text(row, 31)?.expect("updated_at"),
    })
}

fn row_to_card_update(row: &libsql::Row) -> Result<StatusCardUpdateRecord, libsql::Error> {
    Ok(StatusCardUpdateRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        card_id: helpers::row_text(row, 1)?.expect("card_id"),
        kind: helpers::row_text(row, 2)?.expect("kind"),
        trigger: helpers::row_text(row, 3)?.expect("trigger"),
        generation_issue_id: helpers::row_text(row, 4)?,
        run_id: helpers::row_text(row, 5)?,
        changes: json_col!(row, 6),
        input_tokens: helpers::row_i64(row, 7)?,
        output_tokens: helpers::row_i64(row, 8)?,
        cost_cents: helpers::row_i64(row, 9)?,
        model: helpers::row_text(row, 10)?,
        query_version: helpers::row_i64_opt(row, 11)?,
        change_summary: helpers::row_text(row, 12)?,
        started_at: helpers::row_text(row, 13)?.expect("started_at"),
        finished_at: helpers::row_text(row, 14)?,
        status: helpers::row_text(row, 15)?.expect("status"),
        error: helpers::row_text(row, 16)?,
    })
}

fn row_to_summary_slot(row: &libsql::Row) -> Result<SummarySlotRecord, libsql::Error> {
    Ok(SummarySlotRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        scope_kind: helpers::row_text(row, 2)?.expect("scope_kind"),
        scope_id: helpers::row_text(row, 3)?,
        slot_key: helpers::row_text(row, 4)?.expect("slot_key"),
        document_id: helpers::row_text(row, 5)?,
        status: helpers::row_text(row, 6)?.expect("status"),
        failure_reason: helpers::row_text(row, 7)?,
        generating_issue_id: helpers::row_text(row, 8)?,
        last_generated_at: helpers::row_text(row, 9)?,
        last_generated_by_agent_id: helpers::row_text(row, 10)?,
        last_model: helpers::row_text(row, 11)?,
        created_at: helpers::row_text(row, 12)?.expect("created_at"),
        updated_at: helpers::row_text(row, 13)?.expect("updated_at"),
    })
}

fn row_to_smoke_run(row: &libsql::Row) -> Result<SmokeRunRecord, libsql::Error> {
    Ok(SmokeRunRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        trigger: helpers::row_text(row, 2)?.expect("trigger"),
        status: helpers::row_text(row, 3)?.expect("status"),
        started_at: helpers::row_text(row, 4)?.expect("started_at"),
        finished_at: helpers::row_text(row, 5)?,
        summary: json_col!(row, 6),
        created_at: helpers::row_text(row, 7)?.expect("created_at"),
        updated_at: helpers::row_text(row, 8)?.expect("updated_at"),
    })
}

fn row_to_smoke_step(row: &libsql::Row) -> Result<SmokeRunStepRecord, libsql::Error> {
    Ok(SmokeRunStepRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        run_id: helpers::row_text(row, 2)?.expect("run_id"),
        path: helpers::row_text(row, 3)?.expect("path"),
        scenario_step: helpers::row_text(row, 4)?.expect("scenario_step"),
        status: helpers::row_text(row, 5)?.expect("status"),
        detail: helpers::row_text(row, 6)?,
        screenshot_artifact_ref: helpers::row_text(row, 7)?
            .and_then(|v| serde_json::from_str(&v).ok()),
        duration_ms: helpers::row_i64_opt(row, 8)?,
        created_at: helpers::row_text(row, 9)?.expect("created_at"),
        updated_at: helpers::row_text(row, 10)?.expect("updated_at"),
    })
}

fn row_to_feedback_vote(row: &libsql::Row) -> Result<FeedbackVoteRecord, libsql::Error> {
    Ok(FeedbackVoteRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        issue_id: helpers::row_text(row, 2)?.expect("issue_id"),
        target_type: helpers::row_text(row, 3)?.expect("target_type"),
        target_id: helpers::row_text(row, 4)?.expect("target_id"),
        author_user_id: helpers::row_text(row, 5)?.expect("author_user_id"),
        vote: helpers::row_text(row, 6)?.expect("vote"),
        reason: helpers::row_text(row, 7)?,
        shared_with_labs: helpers::row_i64(row, 8)? != 0,
        shared_at: helpers::row_text(row, 9)?,
        consent_version: helpers::row_text(row, 10)?,
        redaction_summary: helpers::row_text(row, 11)?.and_then(|v| serde_json::from_str(&v).ok()),
        created_at: helpers::row_text(row, 12)?.expect("created_at"),
        updated_at: helpers::row_text(row, 13)?.expect("updated_at"),
    })
}

fn row_to_feedback_export(row: &libsql::Row) -> Result<FeedbackExportRecord, libsql::Error> {
    Ok(FeedbackExportRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        feedback_vote_id: helpers::row_text(row, 2)?.expect("feedback_vote_id"),
        issue_id: helpers::row_text(row, 3)?.expect("issue_id"),
        project_id: helpers::row_text(row, 4)?,
        author_user_id: helpers::row_text(row, 5)?.expect("author_user_id"),
        target_type: helpers::row_text(row, 6)?.expect("target_type"),
        target_id: helpers::row_text(row, 7)?.expect("target_id"),
        vote: helpers::row_text(row, 8)?.expect("vote"),
        status: helpers::row_text(row, 9)?.expect("status"),
        destination: helpers::row_text(row, 10)?,
        export_id: helpers::row_text(row, 11)?,
        consent_version: helpers::row_text(row, 12)?,
        schema_version: helpers::row_text(row, 13)?.expect("schema_version"),
        bundle_version: helpers::row_text(row, 14)?.expect("bundle_version"),
        payload_version: helpers::row_text(row, 15)?.expect("payload_version"),
        payload_digest: helpers::row_text(row, 16)?,
        payload_snapshot: helpers::row_text(row, 17)?.and_then(|v| serde_json::from_str(&v).ok()),
        target_summary: json_col!(row, 18),
        redaction_summary: helpers::row_text(row, 19)?.and_then(|v| serde_json::from_str(&v).ok()),
        attempt_count: helpers::row_i64(row, 20)?,
        last_attempted_at: helpers::row_text(row, 21)?,
        exported_at: helpers::row_text(row, 22)?,
        failure_reason: helpers::row_text(row, 23)?,
        created_at: helpers::row_text(row, 24)?.expect("created_at"),
        updated_at: helpers::row_text(row, 25)?.expect("updated_at"),
    })
}

fn row_to_finance_event(row: &libsql::Row) -> Result<FinanceEventRecord, libsql::Error> {
    Ok(FinanceEventRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        agent_id: helpers::row_text(row, 2)?,
        issue_id: helpers::row_text(row, 3)?,
        project_id: helpers::row_text(row, 4)?,
        goal_id: helpers::row_text(row, 5)?,
        heartbeat_run_id: helpers::row_text(row, 6)?,
        cost_event_id: helpers::row_text(row, 7)?,
        billing_code: helpers::row_text(row, 8)?,
        description: helpers::row_text(row, 9)?,
        event_kind: helpers::row_text(row, 10)?.expect("event_kind"),
        direction: helpers::row_text(row, 11)?.expect("direction"),
        biller: helpers::row_text(row, 12)?.expect("biller"),
        provider: helpers::row_text(row, 13)?,
        execution_adapter_type: helpers::row_text(row, 14)?,
        pricing_tier: helpers::row_text(row, 15)?,
        region: helpers::row_text(row, 16)?,
        model: helpers::row_text(row, 17)?,
        quantity: helpers::row_i64_opt(row, 18)?,
        unit: helpers::row_text(row, 19)?,
        amount_cents: helpers::row_i64(row, 20)?,
        currency: helpers::row_text(row, 21)?.expect("currency"),
        estimated: helpers::row_i64(row, 22)? != 0,
        external_invoice_id: helpers::row_text(row, 23)?,
        metadata_json: helpers::row_text(row, 24)?.and_then(|v| serde_json::from_str(&v).ok()),
        occurred_at: helpers::row_text(row, 25)?.expect("occurred_at"),
        created_at: helpers::row_text(row, 26)?.expect("created_at"),
    })
}

fn row_to_thread(row: &libsql::Row) -> Result<AnnotationThreadRecord, libsql::Error> {
    Ok(AnnotationThreadRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        issue_id: helpers::row_text(row, 2)?,
        routine_id: helpers::row_text(row, 3)?,
        case_id: helpers::row_text(row, 4)?,
        document_id: helpers::row_text(row, 5)?.expect("document_id"),
        document_key: helpers::row_text(row, 6)?.expect("document_key"),
        status: helpers::row_text(row, 7)?.expect("status"),
        anchor_state: helpers::row_text(row, 8)?.expect("anchor_state"),
        original_revision_id: helpers::row_text(row, 9)?,
        original_revision_number: helpers::row_i64(row, 10)?,
        current_revision_id: helpers::row_text(row, 11)?,
        current_revision_number: helpers::row_i64(row, 12)?,
        selected_text: helpers::row_text(row, 13)?.expect("selected_text"),
        prefix_text: helpers::row_text(row, 14)?.expect("prefix_text"),
        suffix_text: helpers::row_text(row, 15)?.expect("suffix_text"),
        normalized_start: helpers::row_i64(row, 16)?,
        normalized_end: helpers::row_i64(row, 17)?,
        markdown_start: helpers::row_i64(row, 18)?,
        markdown_end: helpers::row_i64(row, 19)?,
        anchor_confidence: helpers::row_text(row, 20)?.expect("anchor_confidence"),
        anchor_selector: json_col!(row, 21),
        created_by_agent_id: helpers::row_text(row, 22)?,
        created_by_user_id: helpers::row_text(row, 23)?,
        resolved_by_agent_id: helpers::row_text(row, 24)?,
        resolved_by_user_id: helpers::row_text(row, 25)?,
        resolved_at: helpers::row_text(row, 26)?,
        created_at: helpers::row_text(row, 27)?.expect("created_at"),
        updated_at: helpers::row_text(row, 28)?.expect("updated_at"),
    })
}

fn row_to_annotation_comment(row: &libsql::Row) -> Result<AnnotationCommentRecord, libsql::Error> {
    Ok(AnnotationCommentRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        thread_id: helpers::row_text(row, 2)?.expect("thread_id"),
        issue_id: helpers::row_text(row, 3)?,
        routine_id: helpers::row_text(row, 4)?,
        case_id: helpers::row_text(row, 5)?,
        document_id: helpers::row_text(row, 6)?.expect("document_id"),
        body: helpers::row_text(row, 7)?.expect("body"),
        author_type: helpers::row_text(row, 8)?.expect("author_type"),
        author_agent_id: helpers::row_text(row, 9)?,
        author_user_id: helpers::row_text(row, 10)?,
        created_by_run_id: helpers::row_text(row, 11)?,
        issue_comment_id: helpers::row_text(row, 12)?,
        source_trust: helpers::row_text(row, 13)?.and_then(|v| serde_json::from_str(&v).ok()),
        created_at: helpers::row_text(row, 14)?.expect("created_at"),
        updated_at: helpers::row_text(row, 15)?.expect("updated_at"),
    })
}

fn row_to_anchor(row: &libsql::Row) -> Result<AnchorSnapshotRecord, libsql::Error> {
    Ok(AnchorSnapshotRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        thread_id: helpers::row_text(row, 2)?.expect("thread_id"),
        document_id: helpers::row_text(row, 3)?.expect("document_id"),
        from_revision_id: helpers::row_text(row, 4)?,
        from_revision_number: helpers::row_i64_opt(row, 5)?,
        to_revision_id: helpers::row_text(row, 6)?,
        to_revision_number: helpers::row_i64(row, 7)?,
        previous_anchor: json_col!(row, 8),
        next_anchor: helpers::row_text(row, 9)?.and_then(|v| serde_json::from_str(&v).ok()),
        anchor_state: helpers::row_text(row, 10)?.expect("anchor_state"),
        anchor_confidence: helpers::row_text(row, 11)?.expect("anchor_confidence"),
        failure_reason: helpers::row_text(row, 12)?,
        created_at: helpers::row_text(row, 13)?.expect("created_at"),
    })
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoScatteredRepository) {
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
             VALUES ('a1', 'c1', 'Agent', 'engineer', 'cli')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO issues (id, company_id, title, issue_number, identifier)
             VALUES ('i1', 'c1', 'Issue 1', 1, 'ALPHA-1')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO heartbeat_runs (id, company_id, agent_id, invocation_source)
             VALUES ('r1', 'c1', 'a1', 'manual')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO documents (id, company_id, title) VALUES ('d1', 'c1', 'Doc')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO document_revisions (id, company_id, document_id, revision_number, body)
             VALUES ('dr1', 'c1', 'd1', 1, 'v1')",
            (),
        )
        .await
        .unwrap();
        let repo = TursoScatteredRepository::new(db);
        (dir, repo)
    }

    #[tokio::test]
    async fn status_cards_summary_slots_and_smoke() {
        let (_dir, repo) = repo().await;

        let card = repo
            .create_status_card(NewStatusCard {
                company_id: "c1".to_owned(),
                created_by_user_id: Some("u1".to_owned()),
                created_by_agent_id: None,
                title: Some("Team".to_owned()),
                title_pinned: true,
                interest_prompt: "what changed".to_owned(),
                queries: serde_json::json!([{ "kind": "status" }]),
                query_version: 1,
                agent_id: Some("a1".to_owned()),
                refresh_policy: serde_json::json!({ "interval": "15m" }),
                state: "active".to_owned(),
                document_id: None,
            })
            .await
            .unwrap();
        assert_eq!(card.state, "active");
        assert!(card.title_pinned);
        assert_eq!(repo.list_status_cards("c1").await.unwrap().len(), 1);
        let archived = repo
            .archive_status_card("c1", &card.id)
            .await
            .unwrap()
            .unwrap();
        assert!(archived.archived_at.is_some());

        let update = repo
            .create_status_card_update(NewStatusCardUpdate {
                card_id: card.id.clone(),
                kind: "full".to_owned(),
                trigger: "manual".to_owned(),
                generation_issue_id: Some("i1".to_owned()),
                run_id: Some("r1".to_owned()),
                changes: serde_json::json!([{ "issueId": "i1", "from": "todo", "to": "done" }]),
                input_tokens: 10,
                output_tokens: 5,
                cost_cents: 1,
                model: Some("m".to_owned()),
                query_version: Some(1),
                change_summary: Some("updated".to_owned()),
                status: "ok".to_owned(),
                error: None,
            })
            .await
            .unwrap();
        assert_eq!(update.status, "ok");
        assert_eq!(
            repo.list_status_card_updates(&card.id).await.unwrap().len(),
            1
        );

        let slot = repo
            .upsert_summary_slot(NewSummarySlot {
                company_id: "c1".to_owned(),
                scope_kind: "issue".to_owned(),
                scope_id: Some("i1".to_owned()),
                slot_key: "daily".to_owned(),
                document_id: Some("d1".to_owned()),
                status: "idle".to_owned(),
                failure_reason: None,
                generating_issue_id: None,
                last_generated_at: None,
                last_generated_by_agent_id: None,
                last_model: None,
            })
            .await
            .unwrap();
        assert_eq!(slot.slot_key, "daily");
        let slot2 = repo
            .upsert_summary_slot(NewSummarySlot {
                company_id: "c1".to_owned(),
                scope_kind: "issue".to_owned(),
                scope_id: Some("i1".to_owned()),
                slot_key: "daily".to_owned(),
                document_id: None,
                status: "running".to_owned(),
                failure_reason: None,
                generating_issue_id: None,
                last_generated_at: None,
                last_generated_by_agent_id: None,
                last_model: None,
            })
            .await
            .unwrap();
        assert_eq!(slot2.id, slot.id);
        assert_eq!(slot2.status, "running");
        assert_eq!(repo.list_summary_slots("c1").await.unwrap().len(), 1);

        let run = repo
            .create_smoke_run(NewSmokeRun {
                company_id: "c1".to_owned(),
                trigger: "manual".to_owned(),
                status: "running".to_owned(),
                finished_at: None,
                summary: serde_json::json!({}),
            })
            .await
            .unwrap();
        assert_eq!(run.trigger, "manual");
        let step = repo
            .add_smoke_step(NewSmokeRunStep {
                company_id: "c1".to_owned(),
                run_id: run.id.clone(),
                path: "issues".to_owned(),
                scenario_step: "open".to_owned(),
                status: "passed".to_owned(),
                detail: None,
                screenshot_artifact_ref: None,
                duration_ms: Some(100),
            })
            .await
            .unwrap();
        assert_eq!(step.status, "passed");
        assert_eq!(repo.list_smoke_steps("c1", &run.id).await.unwrap().len(), 1);

        // Cross-company isolation.
        assert!(repo.list_status_cards("c2").await.unwrap().is_empty());
        assert!(repo.list_smoke_runs("c2").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn feedback_finance_and_annotations() {
        let (_dir, repo) = repo().await;

        let vote = repo
            .create_feedback_vote(NewFeedbackVote {
                company_id: "c1".to_owned(),
                issue_id: "i1".to_owned(),
                target_type: "issue".to_owned(),
                target_id: "i1".to_owned(),
                author_user_id: "u1".to_owned(),
                vote: "up".to_owned(),
                reason: Some("nice".to_owned()),
                shared_with_labs: true,
                shared_at: None,
                consent_version: Some("v1".to_owned()),
                redaction_summary: None,
            })
            .await
            .unwrap();
        assert_eq!(vote.vote, "up");
        assert!(matches!(
            repo.create_feedback_vote(NewFeedbackVote {
                company_id: "c1".to_owned(),
                issue_id: "i1".to_owned(),
                target_type: "issue".to_owned(),
                target_id: "i1".to_owned(),
                author_user_id: "u1".to_owned(),
                vote: "down".to_owned(),
                reason: None,
                shared_with_labs: false,
                shared_at: None,
                consent_version: None,
                redaction_summary: None,
            })
            .await
            .unwrap_err(),
            ScatteredError::AlreadyExists
        ));
        assert_eq!(repo.list_feedback_votes("c1", "i1").await.unwrap().len(), 1);

        let export = repo
            .create_feedback_export(NewFeedbackExport {
                company_id: "c1".to_owned(),
                feedback_vote_id: vote.id.clone(),
                issue_id: "i1".to_owned(),
                project_id: None,
                author_user_id: "u1".to_owned(),
                target_type: "issue".to_owned(),
                target_id: "i1".to_owned(),
                vote: "up".to_owned(),
                status: "local_only".to_owned(),
                destination: None,
                export_id: None,
                consent_version: Some("v1".to_owned()),
                payload_digest: Some("abc".to_owned()),
                payload_snapshot: None,
                target_summary: serde_json::json!({ "title": "Issue 1" }),
                redaction_summary: None,
                attempt_count: 0,
                exported_at: None,
                failure_reason: None,
            })
            .await
            .unwrap();
        assert_eq!(export.schema_version, "paperclip-feedback-envelope-v2");
        assert!(matches!(
            repo.create_feedback_export(NewFeedbackExport {
                company_id: "c1".to_owned(),
                feedback_vote_id: vote.id.clone(),
                issue_id: "i1".to_owned(),
                project_id: None,
                author_user_id: "u1".to_owned(),
                target_type: "issue".to_owned(),
                target_id: "i1".to_owned(),
                vote: "up".to_owned(),
                status: "exported".to_owned(),
                destination: None,
                export_id: None,
                consent_version: None,
                payload_digest: None,
                payload_snapshot: None,
                target_summary: serde_json::json!({}),
                redaction_summary: None,
                attempt_count: 0,
                exported_at: None,
                failure_reason: None,
            })
            .await
            .unwrap_err(),
            ScatteredError::AlreadyExists
        ));

        let event = repo
            .create_finance_event(NewFinanceEvent {
                company_id: "c1".to_owned(),
                agent_id: Some("a1".to_owned()),
                issue_id: Some("i1".to_owned()),
                project_id: None,
                goal_id: None,
                heartbeat_run_id: Some("r1".to_owned()),
                cost_event_id: None,
                billing_code: Some("B-1".to_owned()),
                description: Some("run".to_owned()),
                event_kind: "execution".to_owned(),
                direction: "debit".to_owned(),
                biller: "openai".to_owned(),
                provider: Some("openai".to_owned()),
                execution_adapter_type: Some("cli".to_owned()),
                pricing_tier: None,
                region: None,
                model: Some("gpt-4o".to_owned()),
                quantity: Some(1),
                unit: Some("run".to_owned()),
                amount_cents: 100,
                currency: "USD".to_owned(),
                estimated: false,
                external_invoice_id: None,
                metadata_json: None,
                occurred_at: "2026-08-04T00:00:00.000Z".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(event.amount_cents, 100);
        assert_eq!(repo.list_finance_events("c1").await.unwrap().len(), 1);

        let thread = repo
            .create_annotation_thread(NewAnnotationThread {
                company_id: "c1".to_owned(),
                issue_id: Some("i1".to_owned()),
                routine_id: None,
                case_id: None,
                document_id: "d1".to_owned(),
                document_key: "body".to_owned(),
                status: "open".to_owned(),
                anchor_state: "active".to_owned(),
                original_revision_id: Some("dr1".to_owned()),
                original_revision_number: 1,
                current_revision_id: Some("dr1".to_owned()),
                current_revision_number: 1,
                selected_text: "hello".to_owned(),
                prefix_text: "".to_owned(),
                suffix_text: "".to_owned(),
                normalized_start: 0,
                normalized_end: 5,
                markdown_start: 0,
                markdown_end: 5,
                anchor_confidence: "exact".to_owned(),
                anchor_selector: serde_json::json!({ "start": 0, "end": 5 }),
                created_by_agent_id: Some("a1".to_owned()),
                created_by_user_id: None,
            })
            .await
            .unwrap();
        assert_eq!(thread.status, "open");
        assert_eq!(
            repo.list_annotation_threads("c1", "d1")
                .await
                .unwrap()
                .len(),
            1
        );

        let comment = repo
            .add_annotation_comment(NewAnnotationComment {
                company_id: "c1".to_owned(),
                thread_id: thread.id.clone(),
                issue_id: Some("i1".to_owned()),
                routine_id: None,
                case_id: None,
                document_id: "d1".to_owned(),
                body: "comment".to_owned(),
                author_type: "user".to_owned(),
                author_agent_id: None,
                author_user_id: Some("u1".to_owned()),
                created_by_run_id: None,
                issue_comment_id: None,
                source_trust: None,
            })
            .await
            .unwrap();
        assert_eq!(comment.body, "comment");
        assert_eq!(
            repo.list_annotation_comments("c1", &thread.id)
                .await
                .unwrap()
                .len(),
            1
        );

        let snapshot = repo
            .create_anchor_snapshot(NewAnchorSnapshot {
                company_id: "c1".to_owned(),
                thread_id: thread.id.clone(),
                document_id: "d1".to_owned(),
                from_revision_id: Some("dr1".to_owned()),
                from_revision_number: Some(1),
                to_revision_id: Some("dr1".to_owned()),
                to_revision_number: 1,
                previous_anchor: serde_json::json!({ "start": 0, "end": 5 }),
                next_anchor: None,
                anchor_state: "active".to_owned(),
                anchor_confidence: "exact".to_owned(),
                failure_reason: None,
            })
            .await
            .unwrap();
        assert_eq!(snapshot.to_revision_number, 1);
        assert_eq!(
            repo.list_anchor_snapshots("c1", &thread.id)
                .await
                .unwrap()
                .len(),
            1
        );

        // Cross-company isolation.
        assert!(repo.list_finance_events("c2").await.unwrap().is_empty());
        assert!(
            repo.list_annotation_threads("c2", "d1")
                .await
                .unwrap()
                .is_empty()
        );
    }
}
