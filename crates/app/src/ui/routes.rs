//! UI form handlers: accept HTML form posts and redirect back to the page.

use serde::Deserialize;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{content::Form, error::see_other, path_param, route},
};

use crate::{audit::log_activity, state::AppState};

/// Shared `{id}` path parameter for UI routes.
#[path_param(error = bad_request("Invalid id"))]
pub(crate) struct Id(String);

/// `POST /issues/{id}/comments/ui` — adds a comment, redirects to the issue.
#[route(POST "/issues/{id}/comments/ui")]
pub async fn add_comment_ui(
    cx: &Cx,
    Form(form): Form<CommentForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let body = form.body.trim().to_owned();
    if !body.is_empty()
        && let Ok(comment) = state
            .comments
            .create(staple_data::NewIssueComment {
                issue_id: issue_id.clone(),
                author_agent_id: None,
                author_user_id: Some("board".to_owned()),
                body,
            })
            .await
    {
        let _ = log_activity(
            &state.activity,
            &comment.company_id,
            "comment.created",
            "issue_comment",
            &comment.id,
            Some(serde_json::json!({ "issueId": comment.issue_id })),
        )
        .await;
    }
    Ok(see_other(&format!("/issues/{issue_id}")))
}

/// `POST /companies/{company_id}/approvals/ui` — creates an approval,
/// redirects to the approvals page.
#[route(POST "/companies/{company_id}/approvals/ui")]
pub async fn create_approval_ui(
    cx: &Cx,
    Form(form): Form<ApprovalForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let r#type = form.r#type.trim().to_owned();
    if !r#type.is_empty()
        && let Ok(approval) = state
            .approvals
            .create(staple_data::NewApproval {
                company_id: company_id.clone(),
                r#type,
                requested_by_agent_id: None,
                requested_by_user_id: Some("board".to_owned()),
                payload: form.payload.unwrap_or_else(|| "{}".to_owned()),
            })
            .await
    {
        let _ = log_activity(
            &state.activity,
            &approval.company_id,
            "approval.created",
            "approval",
            &approval.id,
            Some(serde_json::json!({ "type": approval.r#type })),
        )
        .await;
    }
    Ok(see_other(&format!("/companies/{company_id}/approvals")))
}

/// `POST /approvals/{id}/decide/ui` — approves or rejects, redirects back.
#[route(POST "/approvals/{id}/decide/ui")]
pub async fn decide_approval_ui(
    cx: &Cx,
    Form(form): Form<DecisionForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let approval_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(approval) = state.approvals.get(&approval_id).await.ok().flatten() else {
        return Ok(see_other("/"));
    };
    let company_id = approval.company_id;
    if let Ok(decided) = state
        .approvals
        .decide(
            &approval_id,
            staple_data::ApprovalDecision {
                decision: form.decision,
                decision_note: None,
                decided_by_user_id: Some("board".to_owned()),
            },
        )
        .await
        && let Some(decided) = decided
    {
        let _ = log_activity(
            &state.activity,
            &decided.company_id,
            "approval.decided",
            "approval",
            &decided.id,
            Some(serde_json::json!({ "status": decided.status, "type": decided.r#type })),
        )
        .await;
    }
    Ok(see_other(&format!("/companies/{company_id}/approvals")))
}

/// Shared `{company_id}` path parameter for UI routes.
#[path_param(error = bad_request("Invalid company id"))]
pub(crate) struct CompanyId(String);

/// Comment form fields.
#[derive(Debug, Deserialize)]
pub struct CommentForm {
    /// Comment body.
    pub body: String,
}

/// Approval creation form fields.
#[derive(Debug, Deserialize)]
pub struct ApprovalForm {
    /// Approval type.
    pub r#type: String,
    /// Payload JSON.
    pub payload: Option<String>,
}

/// Decision form fields.
#[derive(Debug, Deserialize)]
pub struct DecisionForm {
    /// `approved` or `rejected`.
    pub decision: String,
}

/// `POST /issues/{id}/status/ui` — moves an issue to a status, redirects to
/// the company board.
#[route(POST "/issues/{id}/status/ui")]
pub async fn move_status_ui(
    cx: &Cx,
    Form(form): Form<StatusForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(issue) = state.issues.get(&issue_id).await.ok().flatten() else {
        return Ok(see_other("/"));
    };
    let company_id = issue.company_id;
    let status = form.status.trim().to_owned();
    if !status.is_empty() {
        let _ = state
            .issues
            .update(
                &issue_id,
                staple_data::IssuePatch {
                    title: None,
                    description: None,
                    status: Some(status),
                    priority: None,
                    assignee_agent_id: None,
                    billing_code: None,
                },
            )
            .await;
    }
    Ok(see_other(&format!("/companies/{company_id}/board")))
}

/// `POST /companies/{company_id}/settings/ui` — applies a settings form
/// action (company / budget / secret / skill), redirects to settings.
#[route(POST "/companies/{company_id}/settings/ui")]
pub async fn settings_ui(
    cx: &Cx,
    Form(form): Form<SettingsForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match form.action.as_str() {
        "company" => {
            let name = form.name.as_deref().unwrap_or("").trim().to_owned();
            if !name.is_empty() {
                let _ = state
                    .companies
                    .update(
                        &company_id,
                        staple_data::CompanyPatch {
                            name: Some(name),
                            description: Some(Some(form.description.clone().unwrap_or_default())),
                            status: None,
                            budget_monthly_cents: None,
                            spent_monthly_cents: None,
                            attachment_max_bytes: None,
                            brand_color: None,
                            require_board_approval_for_new_agents: None,
                        },
                    )
                    .await;
            }
        }
        "budget" => {
            if let Some(cents) = form.budget_monthly_cents {
                let _ = state.costs.set_company_budget(&company_id, cents).await;
            }
        }
        "secret" => {
            let name = form.name.as_deref().unwrap_or("").trim().to_owned();
            let value = form.value.clone().unwrap_or_default();
            if !name.is_empty() && !value.is_empty() {
                let _ = state
                    .secrets
                    .create_secret(staple_data::NewSecret {
                        company_id: company_id.clone(),
                        name,
                        value,
                    })
                    .await;
            }
        }
        "skill" => {
            let name = form.name.as_deref().unwrap_or("").trim().to_owned();
            if !name.is_empty() {
                let _ = state
                    .skills
                    .create(staple_data::NewSkill {
                        company_id: company_id.clone(),
                        name,
                        description: form.description.clone().filter(|d| !d.trim().is_empty()),
                        restriction_policy: staple_data::SkillRestrictionPolicy {
                            allowed_agent_ids: Vec::new(),
                            allowed_roles: Vec::new(),
                            deny_agent_ids: Vec::new(),
                        },
                    })
                    .await;
            }
        }
        _ => {}
    }
    Ok(see_other(&format!("/companies/{company_id}/settings")))
}

/// Status move form.
#[derive(Debug, serde::Deserialize)]
pub struct StatusForm {
    /// Target status.
    pub status: String,
}

/// Settings form: `action` selects the section.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsForm {
    /// Action (`company` | `budget` | `secret` | `skill`).
    pub action: String,
    /// Company name (company action).
    #[serde(default)]
    pub name: Option<String>,
    /// Company description (company action).
    #[serde(default)]
    pub description: Option<String>,
    /// Budget in cents (budget action).
    #[serde(default)]
    pub budget_monthly_cents: Option<i64>,
    /// Secret/skill name.
    #[serde(default)]
    pub value: Option<String>,
}
