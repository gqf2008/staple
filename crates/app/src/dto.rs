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
