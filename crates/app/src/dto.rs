//! API response DTOs (camelCase JSON contract).

use serde::Serialize;
use staple_data::CompanyRecord;

/// Company resource, matching the upstream `Company` JSON shape.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyDto {
    /// Company id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// `active | paused | archived`.
    pub status: String,
    /// Pause reason, when paused.
    pub pause_reason: Option<String>,
    /// ISO 8601 pause time.
    pub paused_at: Option<String>,
    /// Issue identifier prefix.
    pub issue_prefix: String,
    /// Next issue number.
    pub issue_counter: i64,
    /// Monthly budget in cents.
    pub budget_monthly_cents: i64,
    /// Spent this month in cents.
    pub spent_monthly_cents: i64,
    /// Largest attachment size in bytes.
    pub attachment_max_bytes: i64,
    /// Default responsible user.
    pub default_responsible_user_id: Option<String>,
    /// Whether new agents need board approval.
    pub require_board_approval_for_new_agents: bool,
    /// Feedback data sharing consent state.
    pub feedback_data_sharing_enabled: bool,
    /// ISO 8601 consent time.
    pub feedback_data_sharing_consent_at: Option<String>,
    /// Consent giver.
    pub feedback_data_sharing_consent_by_user_id: Option<String>,
    /// Consent terms version.
    pub feedback_data_sharing_terms_version: Option<String>,
    /// Brand color.
    pub brand_color: Option<String>,
    /// Logo asset id.
    pub logo_asset_id: Option<String>,
    /// Logo URL.
    pub logo_url: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
    /// ISO 8601 last update time.
    pub updated_at: String,
}

impl From<CompanyRecord> for CompanyDto {
    fn from(record: CompanyRecord) -> Self {
        Self {
            id: record.id,
            name: record.name,
            description: record.description,
            status: record.status,
            pause_reason: record.pause_reason,
            paused_at: record.paused_at,
            issue_prefix: record.issue_prefix,
            issue_counter: record.issue_counter,
            budget_monthly_cents: record.budget_monthly_cents,
            spent_monthly_cents: record.spent_monthly_cents,
            attachment_max_bytes: record.attachment_max_bytes,
            default_responsible_user_id: record.default_responsible_user_id,
            require_board_approval_for_new_agents: record.require_board_approval_for_new_agents,
            feedback_data_sharing_enabled: record.feedback_data_sharing_enabled,
            feedback_data_sharing_consent_at: record.feedback_data_sharing_consent_at,
            feedback_data_sharing_consent_by_user_id: record
                .feedback_data_sharing_consent_by_user_id,
            feedback_data_sharing_terms_version: record.feedback_data_sharing_terms_version,
            brand_color: record.brand_color,
            logo_asset_id: record.logo_asset_id,
            logo_url: record.logo_url,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

/// Goal resource, matching the upstream goal JSON shape.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalDto {
    /// Goal id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Title.
    pub title: String,
    /// Optional description.
    pub description: Option<String>,
    /// `company | team | agent | task`.
    pub level: String,
    /// Parent goal id.
    pub parent_id: Option<String>,
    /// Owning agent id.
    pub owner_agent_id: Option<String>,
    /// `planned | active | achieved | cancelled`.
    pub status: String,
    /// ISO 8601 creation time.
    pub created_at: String,
    /// ISO 8601 last update time.
    pub updated_at: String,
}

impl From<staple_data::GoalRecord> for GoalDto {
    fn from(record: staple_data::GoalRecord) -> Self {
        Self {
            id: record.id,
            company_id: record.company_id,
            title: record.title,
            description: record.description,
            level: record.level,
            parent_id: record.parent_id,
            owner_agent_id: record.owner_agent_id,
            status: record.status,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

/// Project resource, matching the upstream project JSON shape.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDto {
    /// Project id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Linked goal id.
    pub goal_id: Option<String>,
    /// Name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// `backlog | planned | in_progress | completed | cancelled`.
    pub status: String,
    /// Lead agent id.
    pub lead_agent_id: Option<String>,
    /// Target date.
    pub target_date: Option<String>,
    /// Environment bindings.
    pub env: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
    /// ISO 8601 last update time.
    pub updated_at: String,
}

impl From<staple_data::ProjectRecord> for ProjectDto {
    fn from(record: staple_data::ProjectRecord) -> Self {
        Self {
            id: record.id,
            company_id: record.company_id,
            goal_id: record.goal_id,
            name: record.name,
            description: record.description,
            status: record.status,
            lead_agent_id: record.lead_agent_id,
            target_date: record.target_date,
            env: record.env,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

/// Issue resource, matching the upstream issue JSON shape (core fields).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueDto {
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
    /// Assignee agent id (single-assignee model).
    pub assignee_agent_id: Option<String>,
    /// Assignee user id.
    pub assignee_user_id: Option<String>,
    /// Creator agent id.
    pub created_by_agent_id: Option<String>,
    /// Creator user id.
    pub created_by_user_id: Option<String>,
    /// Per-company issue number.
    pub issue_number: i64,
    /// Stable identifier.
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

impl From<staple_data::IssueRecord> for IssueDto {
    fn from(record: staple_data::IssueRecord) -> Self {
        Self {
            id: record.id,
            company_id: record.company_id,
            project_id: record.project_id,
            goal_id: record.goal_id,
            parent_id: record.parent_id,
            title: record.title,
            description: record.description,
            status: record.status,
            priority: record.priority,
            assignee_agent_id: record.assignee_agent_id,
            assignee_user_id: record.assignee_user_id,
            created_by_agent_id: record.created_by_agent_id,
            created_by_user_id: record.created_by_user_id,
            issue_number: record.issue_number,
            identifier: record.identifier,
            request_depth: record.request_depth,
            work_mode: record.work_mode,
            billing_code: record.billing_code,
            started_at: record.started_at,
            completed_at: record.completed_at,
            cancelled_at: record.cancelled_at,
            hidden_at: record.hidden_at,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

/// Issue comment resource.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueCommentDto {
    /// Comment id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Issue id.
    pub issue_id: String,
    /// Author agent id.
    pub author_agent_id: Option<String>,
    /// Author user id.
    pub author_user_id: Option<String>,
    /// Comment body.
    pub body: String,
    /// ISO 8601 creation time.
    pub created_at: String,
    /// ISO 8601 last update time.
    pub updated_at: String,
}

impl From<staple_data::IssueCommentRecord> for IssueCommentDto {
    fn from(record: staple_data::IssueCommentRecord) -> Self {
        Self {
            id: record.id,
            company_id: record.company_id,
            issue_id: record.issue_id,
            author_agent_id: record.author_agent_id,
            author_user_id: record.author_user_id,
            body: record.body,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

/// Document resource.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentDto {
    /// Document id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Title.
    pub title: Option<String>,
    /// Format.
    pub format: String,
    /// Latest body.
    pub latest_body: String,
    /// Latest revision number.
    pub latest_revision_number: i64,
    /// ISO 8601 creation time.
    pub created_at: String,
    /// ISO 8601 last update time.
    pub updated_at: String,
}

impl From<staple_data::DocumentRecord> for DocumentDto {
    fn from(record: staple_data::DocumentRecord) -> Self {
        Self {
            id: record.id,
            company_id: record.company_id,
            title: record.title,
            format: record.format,
            latest_body: record.latest_body,
            latest_revision_number: record.latest_revision_number,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

/// Asset resource.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetDto {
    /// Asset id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Provider.
    pub provider: String,
    /// Object key.
    pub object_key: String,
    /// Content type.
    pub content_type: String,
    /// Size in bytes.
    pub byte_size: i64,
    /// SHA-256.
    pub sha256: String,
    /// Original filename.
    pub original_filename: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
}

impl From<staple_data::AssetRecord> for AssetDto {
    fn from(record: staple_data::AssetRecord) -> Self {
        Self {
            id: record.id,
            company_id: record.company_id,
            provider: record.provider,
            object_key: record.object_key,
            content_type: record.content_type,
            byte_size: record.byte_size,
            sha256: record.sha256,
            original_filename: record.original_filename,
            created_at: record.created_at,
        }
    }
}

/// Issue attachment resource.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueAttachmentDto {
    /// Attachment id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Issue id.
    pub issue_id: String,
    /// Asset id.
    pub asset_id: String,
    /// ISO 8601 creation time.
    pub created_at: String,
}

impl From<staple_data::IssueAttachmentRecord> for IssueAttachmentDto {
    fn from(record: staple_data::IssueAttachmentRecord) -> Self {
        Self {
            id: record.id,
            company_id: record.company_id,
            issue_id: record.issue_id,
            asset_id: record.asset_id,
            created_at: record.created_at,
        }
    }
}

/// Issue relation (blocker) resource.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueRelationDto {
    /// Relation id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Blocking issue id.
    pub issue_id: String,
    /// Blocked issue id.
    pub related_issue_id: String,
    /// Relation type.
    pub r#type: String,
    /// ISO 8601 creation time.
    pub created_at: String,
}

impl From<staple_data::IssueRelationRecord> for IssueRelationDto {
    fn from(record: staple_data::IssueRelationRecord) -> Self {
        Self {
            id: record.id,
            company_id: record.company_id,
            issue_id: record.issue_id,
            related_issue_id: record.related_issue_id,
            r#type: record.r#type,
            created_at: record.created_at,
        }
    }
}

/// Issue work product resource.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkProductDto {
    /// Work product id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Project id.
    pub project_id: Option<String>,
    /// Issue id.
    pub issue_id: String,
    /// Type.
    pub r#type: String,
    /// Provider.
    pub provider: String,
    /// External id.
    pub external_id: Option<String>,
    /// Title.
    pub title: String,
    /// URL.
    pub url: Option<String>,
    /// Status.
    pub status: String,
    /// Review state.
    pub review_state: String,
    /// Primary flag.
    pub is_primary: bool,
    /// Health status.
    pub health_status: String,
    /// Summary.
    pub summary: Option<String>,
    /// Metadata.
    pub metadata: Option<serde_json::Value>,
    /// ISO 8601 creation time.
    pub created_at: String,
    /// ISO 8601 last update time.
    pub updated_at: String,
}

impl From<staple_data::WorkProductRecord> for WorkProductDto {
    fn from(record: staple_data::WorkProductRecord) -> Self {
        Self {
            id: record.id,
            company_id: record.company_id,
            project_id: record.project_id,
            issue_id: record.issue_id,
            r#type: record.r#type,
            provider: record.provider,
            external_id: record.external_id,
            title: record.title,
            url: record.url,
            status: record.status,
            review_state: record.review_state,
            is_primary: record.is_primary,
            health_status: record.health_status,
            summary: record.summary,
            metadata: record
                .metadata
                .and_then(|value| serde_json::from_str(&value).ok()),
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

/// Heartbeat run resource.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatRunDto {
    /// Run id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Agent id.
    pub agent_id: String,
    /// Invocation source.
    pub invocation_source: String,
    /// Status.
    pub status: String,
    /// ISO 8601 start time.
    pub started_at: Option<String>,
    /// ISO 8601 finish time.
    pub finished_at: Option<String>,
    /// Error message.
    pub error: Option<String>,
    /// Failure attribution (`infrastructure | agent`).
    pub error_kind: Option<String>,
    /// Context snapshot.
    pub context_snapshot: Option<String>,
    /// Trigger detail.
    pub trigger_detail: Option<String>,
    /// Log bytes.
    pub log_bytes: i64,
    /// ISO 8601 creation time.
    pub created_at: String,
}

impl From<staple_data::HeartbeatRunRecord> for HeartbeatRunDto {
    fn from(record: staple_data::HeartbeatRunRecord) -> Self {
        Self {
            id: record.id,
            company_id: record.company_id,
            agent_id: record.agent_id,
            invocation_source: record.invocation_source,
            status: record.status,
            started_at: record.started_at,
            finished_at: record.finished_at,
            error: record.error,
            error_kind: record.error_kind,
            context_snapshot: record.context_snapshot,
            trigger_detail: record.trigger_detail,
            log_bytes: record.log_bytes,
            created_at: record.created_at,
        }
    }
}

/// Cost event resource.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CostEventDto {
    /// Event id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Agent id.
    pub agent_id: String,
    /// Issue id.
    pub issue_id: Option<String>,
    /// Billing code.
    pub billing_code: Option<String>,
    /// Provider.
    pub provider: String,
    /// Model.
    pub model: String,
    /// Input tokens.
    pub input_tokens: i64,
    /// Output tokens.
    pub output_tokens: i64,
    /// Cost in cents.
    pub cost_cents: i64,
    /// ISO 8601 occurrence time.
    pub occurred_at: String,
    /// ISO 8601 creation time.
    pub created_at: String,
}

impl From<staple_data::CostEventRecord> for CostEventDto {
    fn from(record: staple_data::CostEventRecord) -> Self {
        Self {
            id: record.id,
            company_id: record.company_id,
            agent_id: record.agent_id,
            issue_id: record.issue_id,
            billing_code: record.billing_code,
            provider: record.provider,
            model: record.model,
            input_tokens: record.input_tokens,
            output_tokens: record.output_tokens,
            cost_cents: record.cost_cents,
            occurred_at: record.occurred_at,
            created_at: record.created_at,
        }
    }
}

/// Company budget summary resource.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetSummaryDto {
    /// Company id.
    pub company_id: String,
    /// Monthly budget in cents.
    pub budget_monthly_cents: i64,
    /// Spent this month in cents.
    pub spent_monthly_cents: i64,
    /// Remaining cents.
    pub remaining_cents: i64,
    /// Agents paused by budget exhaustion.
    pub paused_agents: i64,
}

impl From<staple_data::BudgetSummary> for BudgetSummaryDto {
    fn from(summary: staple_data::BudgetSummary) -> Self {
        Self {
            company_id: summary.company_id,
            budget_monthly_cents: summary.budget_monthly_cents,
            spent_monthly_cents: summary.spent_monthly_cents,
            remaining_cents: summary.remaining_cents,
            paused_agents: summary.paused_agents,
        }
    }
}

/// Per-agent cost row.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCostRowDto {
    /// Agent id.
    pub agent_id: String,
    /// Agent name.
    pub agent_name: String,
    /// Agent status.
    pub status: String,
    /// Agent monthly budget.
    pub budget_monthly_cents: i64,
    /// Agent spent this month.
    pub spent_monthly_cents: i64,
}

impl From<staple_data::AgentCostRow> for AgentCostRowDto {
    fn from(row: staple_data::AgentCostRow) -> Self {
        Self {
            agent_id: row.agent_id,
            agent_name: row.agent_name,
            status: row.status,
            budget_monthly_cents: row.budget_monthly_cents,
            spent_monthly_cents: row.spent_monthly_cents,
        }
    }
}

/// Approval resource.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalDto {
    /// Approval id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Type.
    pub r#type: String,
    /// Requester agent id.
    pub requested_by_agent_id: Option<String>,
    /// Requester user id.
    pub requested_by_user_id: Option<String>,
    /// Status.
    pub status: String,
    /// Payload.
    pub payload: serde_json::Value,
    /// Decision note.
    pub decision_note: Option<String>,
    /// Deciding user id.
    pub decided_by_user_id: Option<String>,
    /// ISO 8601 decision time.
    pub decided_at: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
}

impl From<staple_data::ApprovalRecord> for ApprovalDto {
    fn from(record: staple_data::ApprovalRecord) -> Self {
        Self {
            id: record.id,
            company_id: record.company_id,
            r#type: record.r#type,
            requested_by_agent_id: record.requested_by_agent_id,
            requested_by_user_id: record.requested_by_user_id,
            status: record.status,
            payload: serde_json::from_str(&record.payload).unwrap_or(serde_json::Value::Null),
            decision_note: record.decision_note,
            decided_by_user_id: record.decided_by_user_id,
            decided_at: record.decided_at,
            created_at: record.created_at,
        }
    }
}

/// Activity log entry.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityEntryDto {
    /// Entry id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Actor type.
    pub actor_type: String,
    /// Actor id.
    pub actor_id: String,
    /// Action.
    pub action: String,
    /// Entity type.
    pub entity_type: String,
    /// Entity id.
    pub entity_id: String,
    /// Details.
    pub details: Option<serde_json::Value>,
    /// ISO 8601 creation time.
    pub created_at: String,
}

impl From<staple_data::ActivityEntry> for ActivityEntryDto {
    fn from(entry: staple_data::ActivityEntry) -> Self {
        Self {
            id: entry.id,
            company_id: entry.company_id,
            actor_type: entry.actor_type,
            actor_id: entry.actor_id,
            action: entry.action,
            entity_type: entry.entity_type,
            entity_id: entry.entity_id,
            details: entry
                .details
                .and_then(|value| serde_json::from_str(&value).ok()),
            created_at: entry.created_at,
        }
    }
}

/// Company secret metadata (no value material).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanySecretDto {
    /// Secret id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Name.
    pub name: String,
    /// Scope.
    pub scope: String,
    /// Provider.
    pub provider: String,
    /// Latest version.
    pub latest_version: i64,
    /// ISO 8601 creation time.
    pub created_at: String,
}

impl From<staple_data::CompanySecretRecord> for CompanySecretDto {
    fn from(record: staple_data::CompanySecretRecord) -> Self {
        Self {
            id: record.id,
            company_id: record.company_id,
            name: record.name,
            scope: record.scope,
            provider: record.provider,
            latest_version: record.latest_version,
            created_at: record.created_at,
        }
    }
}

/// Secret version metadata.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretVersionDto {
    /// Version id.
    pub id: String,
    /// Secret id.
    pub secret_id: String,
    /// Version number.
    pub version: i64,
    /// ISO 8601 creation time.
    pub created_at: String,
}

impl From<staple_data::SecretVersionRecord> for SecretVersionDto {
    fn from(record: staple_data::SecretVersionRecord) -> Self {
        Self {
            id: record.id,
            secret_id: record.secret_id,
            version: record.version,
            created_at: record.created_at,
        }
    }
}
