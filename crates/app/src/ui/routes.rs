//! UI form handlers: accept HTML form posts and redirect back to the page.

use serde::Deserialize;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{content::Form, error::see_other, path_param, route},
};

use crate::{
    audit::log_activity,
    error::ApiError,
    i18n::{lang_from_request, t, with_lang},
    state::AppState,
    team_catalog,
    ui::pages::onboarding_view,
};
use topcoat::router::IntoResponse;

/// Shared `{id}` path parameter for UI routes.
#[path_param(error = bad_request("Invalid id"))]
pub(crate) struct Id(String);

/// Builds the redirect URI with a `flash` query parameter (issue #231).
/// Appends `?flash=<code>`, or `&flash=<code>` when `path` already carries a
/// query string (the existing query is preserved).
fn flash_uri(path: &str, code: &str) -> String {
    let separator = if path.contains('?') { '&' } else { '?' };
    format!("{path}{separator}flash={code}")
}

/// Redirects to `path` with a `flash` code consumed by the global toast
/// (issue #231).
fn see_other_flash(path: &str, code: &str) -> topcoat::router::error::SeeOther {
    see_other(&flash_uri(path, code))
}

/// `POST /issues/{id}/comments/ui` — adds a comment, redirects to the issue.
#[route(POST "/issues/{id}/comments/ui")]
pub async fn add_comment_ui(
    cx: &Cx,
    Form(form): Form<CommentForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let body = form.body.trim().to_owned();
    let mut code = "error";
    if !body.is_empty()
        && let Ok(comment) = state
            .comments
            .create(staple_data::NewIssueComment {
                issue_id: issue_id.clone(),
                author_agent_id: None,
                author_user_id: Some(crate::auth::current_actor(cx)),
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
        code = "comment-added";
    }
    Ok(see_other_flash(&format!("/issues/{issue_id}"), code))
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
    let code = if r#type.is_empty() {
        "invalid"
    } else if let Ok(approval) = state
        .approvals
        .create(staple_data::NewApproval {
            company_id: company_id.clone(),
            r#type,
            requested_by_agent_id: None,
            requested_by_user_id: Some(crate::auth::current_actor(cx)),
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
        "created"
    } else {
        "error"
    };
    Ok(see_other_flash(
        &format!("/companies/{company_id}/approvals"),
        code,
    ))
}

/// `POST /approvals/{id}/decide/ui` — approves or rejects, redirects back.
#[route(POST "/approvals/{id}/decide/ui")]
pub async fn decide_approval_ui(
    cx: &Cx,
    Form(form): Form<DecisionForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let approval_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if state
        .approvals
        .get(&approval_id)
        .await
        .ok()
        .flatten()
        .is_none()
    {
        return Ok(see_other("/"));
    }
    let mut code = "error";
    if let Ok(decided) = state
        .approvals
        .decide(
            &approval_id,
            staple_data::ApprovalDecision {
                decision: form.decision,
                decision_note: form.note.filter(|note| !note.trim().is_empty()),
                decided_by_user_id: Some(crate::auth::current_actor(cx)),
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
        code = "decided";
    }
    Ok(see_other_flash(&format!("/approvals/{approval_id}"), code))
}

/// `POST /approvals/{id}/comments/ui` — adds an approval comment and
/// redirects back to the approval detail page.
#[route(POST "/approvals/{id}/comments/ui")]
pub async fn add_approval_comment_ui(
    cx: &Cx,
    Form(form): Form<ApprovalCommentForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let approval_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let body = form.body.trim().to_owned();
    let mut code = "error";
    if !body.is_empty()
        && let Ok(Some(approval)) = state.approvals.get(&approval_id).await
    {
        let _ = state
            .infrastructure
            .create_approval_comment(staple_data::NewApprovalComment {
                company_id: approval.company_id,
                approval_id: approval_id.clone(),
                author_agent_id: None,
                author_user_id: Some(crate::auth::current_actor(cx)),
                body,
            })
            .await;
        code = "comment-added";
    }
    Ok(see_other_flash(&format!("/approvals/{approval_id}"), code))
}

/// Shared `{company_id}` path parameter for UI routes.
#[path_param(error = bad_request("Invalid company id"))]
pub(crate) struct CompanyId(String);

/// Company create form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyUiForm {
    /// Company name.
    pub name: String,
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
    /// Monthly budget in cents.
    #[serde(default)]
    pub budget_monthly_cents: Option<i64>,
    /// Largest attachment size in bytes.
    #[serde(default)]
    pub attachment_max_bytes: Option<i64>,
}

/// `POST /companies/ui` — creates a company, redirects to its overview.
#[route(POST "/companies/ui")]
pub async fn create_company_ui(
    cx: &Cx,
    Form(form): Form<CompanyUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let state = app_context::<AppState>(cx);
    let name = form.name.trim().to_owned();
    if name.is_empty() {
        return Ok(see_other_flash("/", "invalid"));
    }
    if let Ok(company) = state
        .companies
        .create(staple_data::NewCompany {
            name,
            description: form.description.filter(|value| !value.trim().is_empty()),
            budget_monthly_cents: form.budget_monthly_cents.unwrap_or(0),
            attachment_max_bytes: form.attachment_max_bytes.unwrap_or(0),
        })
        .await
    {
        let _ = log_activity(
            &state.activity,
            &company.id,
            "company.created",
            "company",
            &company.id,
            Some(serde_json::json!({ "name": company.name })),
        )
        .await;
        return Ok(see_other_flash(
            &format!("/companies/{}", company.id),
            "created",
        ));
    }
    Ok(see_other_flash("/", "error"))
}

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
    /// Optional decision note.
    #[serde(default)]
    pub note: Option<String>,
}

/// Approval comment form fields.
#[derive(Debug, Deserialize)]
pub struct ApprovalCommentForm {
    /// Comment body.
    pub body: String,
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
    let mut code = "invalid";
    if !status.is_empty() {
        code = if state
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
                    execution_workspace_settings: None,
                },
            )
            .await
            .is_ok()
        {
            "updated"
        } else {
            "error"
        };
    }
    Ok(see_other_flash(
        &format!("/companies/{company_id}/board"),
        code,
    ))
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
    let mut code = "invalid";
    match form.action.as_str() {
        "company" => {
            let name = form.name.as_deref().unwrap_or("").trim().to_owned();
            if !name.is_empty() {
                code = if state
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
                    .await
                    .is_ok()
                {
                    "saved"
                } else {
                    "error"
                };
            }
        }
        "budget" => {
            if let Some(cents) = form.budget_monthly_cents {
                code = if state
                    .costs
                    .set_company_budget(&company_id, cents)
                    .await
                    .is_ok()
                {
                    "saved"
                } else {
                    "error"
                };
            }
        }
        "secret" => {
            let name = form.name.as_deref().unwrap_or("").trim().to_owned();
            let value = form.value.clone().unwrap_or_default();
            if !name.is_empty() && !value.is_empty() {
                code = if state
                    .secrets
                    .create_secret(staple_data::NewSecret {
                        company_id: company_id.clone(),
                        name,
                        value,
                    })
                    .await
                    .is_ok()
                {
                    "saved"
                } else {
                    "error"
                };
            }
        }
        "skill" => {
            let name = form.name.as_deref().unwrap_or("").trim().to_owned();
            if !name.is_empty() {
                code = if state
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
                    .await
                    .is_ok()
                {
                    "saved"
                } else {
                    "error"
                };
            }
        }
        _ => {}
    }
    Ok(see_other_flash(
        &format!("/companies/{company_id}/settings"),
        code,
    ))
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

// ---------------------------------------------------------------------------
// Agent UI actions
// ---------------------------------------------------------------------------

/// Agent create form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAgentUiForm {
    /// Agent name.
    pub name: String,
    /// Agent role.
    #[serde(default)]
    pub role: Option<String>,
    /// Optional title.
    #[serde(default)]
    pub title: Option<String>,
    /// Adapter type.
    #[serde(default)]
    pub adapter_type: Option<String>,
    /// Monthly budget in cents.
    #[serde(default)]
    pub budget_monthly_cents: Option<i64>,
    /// Manager agent id.
    #[serde(default)]
    pub reports_to: Option<String>,
}

/// `POST /companies/{company_id}/agents/ui` — creates an agent, redirects to
/// the agents page.
#[route(POST "/companies/{company_id}/agents/ui")]
pub async fn create_agent_ui(
    cx: &Cx,
    Form(form): Form<NewAgentUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let name = form.name.trim().to_owned();
    let role = form.role.clone().unwrap_or_else(|| "general".to_owned());
    if name.is_empty() {
        return Ok(see_other_flash(
            &format!("/companies/{company_id}/agents"),
            "invalid",
        ));
    }
    if let Ok(agent) = state
        .agents
        .create(staple_data::NewAgent {
            company_id: company_id.clone(),
            name,
            role: role.clone(),
            title: form.title.filter(|value| !value.trim().is_empty()),
            icon: None,
            reports_to: form.reports_to.filter(|value| !value.trim().is_empty()),
            adapter_type: form
                .adapter_type
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "cli_local".to_owned()),
            budget_monthly_cents: form.budget_monthly_cents.unwrap_or(0),
        })
        .await
    {
        let _ = log_activity(
            &state.activity,
            &company_id,
            "agent.created",
            "agent",
            &agent.id,
            Some(serde_json::json!({ "name": agent.name })),
        )
        .await;
        if let Err(error) = crate::instructions::materialize_default_instructions(
            state,
            &company_id,
            &agent.id,
            &role,
        )
        .await
        {
            tracing::warn!(
                error = %error,
                company_id = %company_id,
                agent_id = %agent.id,
                "failed to materialize default agent instructions"
            );
        }
        return Ok(see_other_flash(&format!("/agents/{}", agent.id), "created"));
    }
    Ok(see_other_flash(
        &format!("/companies/{company_id}/agents"),
        "error",
    ))
}

/// `POST /agents/{id}/status/ui` — pause/resume an agent.
#[route(POST "/agents/{agent_id}/status/ui")]
pub async fn agent_status_ui(
    cx: &Cx,
    Form(form): Form<AgentStatusForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let agent_id = path_param::<AgentUiId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state.agents.company_of(&agent_id).await.ok().flatten() else {
        return Ok(see_other("/"));
    };
    let code = if state
        .agents
        .update_status(
            &company_id,
            &agent_id,
            &form.status,
            Some(Some(form.pause_reason.unwrap_or_default())),
        )
        .await
        .is_ok()
    {
        "updated"
    } else {
        "error"
    };
    Ok(see_other_flash(&format!("/agents/{agent_id}"), code))
}

/// `POST /agents/{id}/budget/ui` — set an agent budget.
#[route(POST "/agents/{agent_id}/budget/ui")]
pub async fn agent_budget_ui(
    cx: &Cx,
    Form(form): Form<AgentBudgetForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let agent_id = path_param::<AgentUiId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state.agents.company_of(&agent_id).await.ok().flatten() else {
        return Ok(see_other("/"));
    };
    let code = if state
        .agents
        .set_budget(&company_id, &agent_id, form.budget_monthly_cents)
        .await
        .is_ok()
    {
        "saved"
    } else {
        "error"
    };
    Ok(see_other_flash(&format!("/agents/{agent_id}"), code))
}

/// `POST /issues/{id}/archive/ui` and `/unarchive/ui` — inbox controls.
#[route(POST "/issues/{id}/archive/ui")]
pub async fn archive_issue_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if let Some(issue) = state.issues.get(&issue_id).await.ok().flatten() {
        let code = if state.issues.set_hidden(&issue_id, true).await.is_ok() {
            "archived"
        } else {
            "error"
        };
        return Ok(see_other_flash(
            &format!("/companies/{}/inbox", issue.company_id),
            code,
        ));
    }
    Ok(see_other_flash("/", "error"))
}

/// `POST /issues/{id}/unarchive/ui`.
#[route(POST "/issues/{id}/unarchive/ui")]
pub async fn unarchive_issue_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if let Some(issue) = state.issues.get(&issue_id).await.ok().flatten() {
        let code = if state.issues.set_hidden(&issue_id, false).await.is_ok() {
            "unarchived"
        } else {
            "error"
        };
        return Ok(see_other_flash(
            &format!("/companies/{}/inbox", issue.company_id),
            code,
        ));
    }
    Ok(see_other_flash("/", "error"))
}

/// `{item_key}` path parameter (attention dismissal UI).
#[path_param(error = bad_request("Invalid item key"))]
pub(crate) struct ItemKey(String);

/// `POST /companies/{company_id}/attention/{item_key}/dismiss/ui` — dismisses
/// an attention item for the current board user and returns to What needs me.
#[route(POST "/companies/{company_id}/attention/{item_key}/dismiss/ui")]
pub async fn dismiss_attention_item_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let item_key = path_param::<ItemKey>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let user_id = topcoat::context::try_request_context::<http::request::Parts>(cx)
        .and_then(|parts| parts.headers.get("x-board-user"))
        .and_then(|value| value.to_str().ok())
        .unwrap_or("board")
        .to_owned();
    let ok = state
        .attention_dismissals
        .upsert(staple_data::NewDismissal {
            company_id: company_id.clone(),
            user_id,
            item_key,
            kind: "dismiss".to_owned(),
            snoozed_until: None,
        })
        .await
        .is_ok();
    let code = if ok { "dismissed" } else { "error" };
    Ok(see_other_flash(
        &format!("/companies/{company_id}/what-needs-me"),
        code,
    ))
}

// ---------------------------------------------------------------------------
// Decision desk UI actions
// ---------------------------------------------------------------------------

/// `POST /companies/{company_id}/decision-queues/ui` — create a queue.
#[route(POST "/companies/{company_id}/decision-queues/ui")]
pub async fn decision_queue_ui(
    cx: &Cx,
    Form(form): Form<DecisionQueueForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let name = form.name.trim().to_owned();
    if !name.is_empty() {
        let _ = state
            .decisions
            .create_queue(&company_id, &name, None, None)
            .await;
    }
    Ok(see_other(&format!("/companies/{company_id}/decision-desk")))
}

/// `POST /companies/{company_id}/decision-retention/{source_kind}/{source_id}/{action}/ui`.
#[route(POST "/companies/{company_id}/decision-retention/{source_kind}/{source_id}/keep/ui")]
pub async fn retention_keep_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let kind = path_param::<SourceKindUi>(cx)?.to_string();
    let id = path_param::<SourceIdUi>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let _ = state
        .decisions
        .retention_set_keep(&company_id, &kind, &id, true)
        .await;
    Ok(see_other(&format!("/companies/{company_id}/decision-desk")))
}

/// `POST .../restore/ui`.
#[route(POST "/companies/{company_id}/decision-retention/{source_kind}/{source_id}/restore/ui")]
pub async fn retention_restore_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let kind = path_param::<SourceKindUi>(cx)?.to_string();
    let id = path_param::<SourceIdUi>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let _ = state
        .decisions
        .retention_restore(&company_id, &kind, &id)
        .await;
    Ok(see_other(&format!("/companies/{company_id}/decision-desk")))
}

// ---------------------------------------------------------------------------
// Access UI actions
// ---------------------------------------------------------------------------

/// `POST /companies/{company_id}/memberships/ui`.
#[route(POST "/companies/{company_id}/memberships/ui")]
pub async fn membership_ui(
    cx: &Cx,
    Form(form): Form<MembershipForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let name = form.name.trim().to_owned();
    if !name.is_empty() {
        let _ = state
            .memberships
            .upsert(staple_data::NewCompanyMembership {
                company_id: company_id.clone(),
                principal_type: "user".to_owned(),
                principal_id: name,
                membership_role: form.role.clone().or(Some("operator".to_owned())),
            })
            .await;
    }
    Ok(see_other(&format!("/companies/{company_id}/access")))
}

/// `POST /companies/{company_id}/invites/ui`.
#[route(POST "/companies/{company_id}/invites/ui")]
pub async fn invite_ui(
    cx: &Cx,
    Form(form): Form<InviteForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if let Some(name) = form.name.filter(|n| !n.trim().is_empty()) {
        let _ = state
            .invites
            .create_invite(staple_data::NewInvite {
                company_id: company_id.clone(),
                invite_type: "company_join".to_owned(),
                allowed_join_types: "both".to_owned(),
                defaults_payload: Some(serde_json::json!({ "name": name })),
                expires_at: "2999-01-01T00:00:00.000Z".to_owned(),
                invited_by_user_id: None,
            })
            .await;
    }
    Ok(see_other(&format!("/companies/{company_id}/access")))
}

/// `POST /companies/{company_id}/invites/{id}/revoke/ui`.
#[route(POST "/companies/{company_id}/invites/{id}/revoke/ui")]
pub async fn invite_revoke_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let _ = state.invites.revoke_invite(&company_id, &id).await;
    Ok(see_other(&format!("/companies/{company_id}/access")))
}

/// `POST /companies/{company_id}/join-requests/{id}/approve/ui`.
#[route(POST "/companies/{company_id}/join-requests/{id}/approve/ui")]
pub async fn join_approve_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let _ = state.invites.approve(&company_id, &id, None).await;
    Ok(see_other(&format!("/companies/{company_id}/access")))
}

/// `POST /companies/{company_id}/join-requests/{id}/reject/ui`.
#[route(POST "/companies/{company_id}/join-requests/{id}/reject/ui")]
pub async fn join_reject_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let _ = state.invites.reject(&company_id, &id, None).await;
    Ok(see_other(&format!("/companies/{company_id}/access")))
}

// ---------------------------------------------------------------------------
// Routines UI actions
// ---------------------------------------------------------------------------

/// `POST /companies/{company_id}/routines/ui`.
#[route(POST "/companies/{company_id}/routines/ui")]
pub async fn routine_ui(
    cx: &Cx,
    Form(form): Form<RoutineForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let title = form.title.trim().to_owned();
    let code = if title.is_empty() {
        "invalid"
    } else if state
        .routines
        .create(staple_data::NewRoutine {
            company_id: company_id.clone(),
            project_id: None,
            goal_id: None,
            parent_issue_id: None,
            title,
            description: None,
            assignee_agent_id: None,
            priority: "medium".to_owned(),
            variables: None,
        })
        .await
        .is_ok()
    {
        "created"
    } else {
        "error"
    };
    Ok(see_other_flash(
        &format!("/companies/{company_id}/routines"),
        code,
    ))
}

/// `POST /routines/{id}/trigger/ui`.
#[route(POST "/routines/{id}/trigger/ui")]
pub async fn routine_trigger_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let routine_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if let Ok(Some(routine)) = state.routines.get(&routine_id).await {
        let code = if state
            .routines
            .trigger(&routine.company_id, &routine_id)
            .await
            .is_ok()
        {
            "triggered"
        } else {
            "error"
        };
        return Ok(see_other_flash(&format!("/routines/{routine_id}"), code));
    }
    Ok(see_other_flash("/", "error"))
}

// ---------------------------------------------------------------------------
// Secrets / skills UI actions (dedicated pages)
// ---------------------------------------------------------------------------

/// `POST /companies/{company_id}/secrets/ui`.
#[route(POST "/companies/{company_id}/secrets/ui")]
pub async fn secret_ui(
    cx: &Cx,
    Form(form): Form<SecretForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let name = form.name.as_deref().unwrap_or("").trim().to_owned();
    let value = form.value.clone().unwrap_or_default();
    let code = if name.is_empty() || value.is_empty() {
        "invalid"
    } else if state
        .secrets
        .create_secret(staple_data::NewSecret {
            company_id: company_id.clone(),
            name,
            value,
        })
        .await
        .is_ok()
    {
        "saved"
    } else {
        "error"
    };
    Ok(see_other_flash(
        &format!("/companies/{company_id}/secrets"),
        code,
    ))
}

/// `POST /companies/{company_id}/skills/ui`.
#[route(POST "/companies/{company_id}/skills/ui")]
pub async fn skill_ui(
    cx: &Cx,
    Form(form): Form<SkillForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let name = form.name.as_deref().unwrap_or("").trim().to_owned();
    let code = if name.is_empty() {
        "invalid"
    } else if state
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
        .await
        .is_ok()
    {
        "created"
    } else {
        "error"
    };
    Ok(see_other_flash(
        &format!("/companies/{company_id}/skills"),
        code,
    ))
}

// ---------------------------------------------------------------------------
// Plugin UI actions
// ---------------------------------------------------------------------------

/// `POST /companies/{company_id}/plugins/register/ui` — registers (or
/// re-registers) a plugin in the instance registry.
#[route(POST "/companies/{company_id}/plugins/register/ui")]
pub async fn plugin_register_ui(
    cx: &Cx,
    Form(form): Form<PluginRegisterForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let plugin_key = form.plugin_key.trim().to_owned();
    let package_name = form.package_name.trim().to_owned();
    let version = form.version.trim().to_owned();
    if !plugin_key.is_empty() && !package_name.is_empty() && !version.is_empty() {
        let categories = form
            .categories
            .as_deref()
            .unwrap_or("")
            .split(',')
            .map(|entry| entry.trim().to_owned())
            .filter(|entry| !entry.is_empty())
            .collect();
        let manifest = serde_json::from_str::<serde_json::Value>(
            form.manifest_json.as_deref().unwrap_or("{}"),
        )
        .unwrap_or_else(|_| serde_json::json!({ "id": plugin_key.clone() }));
        let _ = state
            .plugins
            .register(staple_data::NewPlugin {
                plugin_key,
                package_name,
                version,
                api_version: form.api_version.unwrap_or(1),
                categories,
                manifest_json: manifest,
                install_order: form.install_order,
                package_path: form.package_path.filter(|path| !path.trim().is_empty()),
            })
            .await;
    }
    Ok(see_other(&format!("/companies/{company_id}/plugins")))
}

/// `POST /companies/{company_id}/plugins/{plugin_id}/settings/ui` — enables
/// or disables a plugin for a company.
#[route(POST "/companies/{company_id}/plugins/{plugin_id}/settings/ui")]
pub async fn plugin_company_settings_ui(
    cx: &Cx,
    Form(form): Form<PluginCompanySettingsForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let plugin_id = path_param::<PluginUiId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let enabled = form
        .enabled
        .as_deref()
        .map(|value| value == "1")
        .unwrap_or(true);
    let _ = state
        .plugins
        .upsert_company_settings(staple_data::UpsertCompanySettings {
            company_id: company_id.clone(),
            plugin_id,
            enabled,
            settings_json: serde_json::json!({}),
        })
        .await;
    Ok(see_other(&format!("/companies/{company_id}/plugins")))
}

/// `POST /plugins/{id}/status/ui` — updates plugin status/error.
#[route(POST "/plugins/{id}/status/ui")]
pub async fn plugin_status_ui(
    cx: &Cx,
    Form(form): Form<PluginStatusForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let plugin_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let last_error = form.last_error.filter(|error| !error.trim().is_empty());
    let _ = state
        .plugins
        .update_status(&plugin_id, &form.status, Some(last_error))
        .await;
    Ok(see_other(&format!("/plugins/{plugin_id}")))
}

/// `POST /plugins/{id}/configs/ui` — upserts a per-company plugin config.
#[route(POST "/plugins/{id}/configs/ui")]
pub async fn plugin_config_ui(
    cx: &Cx,
    Form(form): Form<PluginConfigForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let plugin_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company_id = form.company_id.trim().to_owned();
    if !company_id.is_empty() {
        let config = serde_json::from_str::<serde_json::Value>(&form.config_json)
            .unwrap_or_else(|_| serde_json::json!({}));
        let _ = state
            .plugins
            .upsert_config(staple_data::UpsertPluginConfig {
                plugin_id: plugin_id.clone(),
                company_id,
                config_json: config,
            })
            .await;
    }
    Ok(see_other(&format!("/plugins/{plugin_id}")))
}

/// `POST /plugins/{id}/state/ui` — sets scoped plugin state.
#[route(POST "/plugins/{id}/state/ui")]
pub async fn plugin_state_ui(
    cx: &Cx,
    Form(form): Form<PluginStateForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let plugin_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let key = form.key.trim().to_owned();
    if !key.is_empty() {
        let value = serde_json::from_str::<serde_json::Value>(&form.value)
            .unwrap_or_else(|_| serde_json::json!({}));
        let scope_id = form.scope_id.filter(|value| !value.trim().is_empty());
        let _ = state
            .plugin_runtime
            .state_set(
                &plugin_id,
                &form.scope_kind,
                scope_id.as_deref(),
                &form.namespace,
                &key,
                value,
            )
            .await;
    }
    Ok(see_other(&format!(
        "/plugins/{plugin_id}?scopeKind={}&namespace={}",
        form.scope_kind, form.namespace
    )))
}

/// `POST /plugins/{id}/entities/ui` — upserts a plugin entity.
#[route(POST "/plugins/{id}/entities/ui")]
pub async fn plugin_entity_ui(
    cx: &Cx,
    Form(form): Form<PluginEntityForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let plugin_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let entity_type = form.entity_type.trim().to_owned();
    if !entity_type.is_empty() {
        let data = serde_json::from_str::<serde_json::Value>(&form.data)
            .unwrap_or_else(|_| serde_json::json!({}));
        let _ = state
            .plugin_runtime
            .entity_upsert(staple_data::NewPluginEntity {
                plugin_id: plugin_id.clone(),
                company_id: form.company_id.filter(|value| !value.trim().is_empty()),
                entity_type,
                scope_kind: form.scope_kind,
                scope_id: form.scope_id.filter(|value| !value.trim().is_empty()),
                external_id: form.external_id.filter(|value| !value.trim().is_empty()),
                title: form.title.filter(|value| !value.trim().is_empty()),
                status: form.status.filter(|value| !value.trim().is_empty()),
                data,
            })
            .await;
    }
    Ok(see_other(&format!("/plugins/{plugin_id}")))
}

/// `POST /plugins/{id}/jobs/{job_key}/runs/ui` — manually runs a plugin job.
#[route(POST "/plugins/{id}/jobs/{job_key}/runs/ui")]
pub async fn plugin_job_run_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let plugin_id = path_param::<Id>(cx)?.to_string();
    let job_key = path_param::<JobKeyUi>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if let Ok(jobs) = state.plugin_runtime.job_list(&plugin_id).await
        && let Some(job) = jobs.into_iter().find(|job| job.job_key == job_key)
    {
        let _ = state
            .plugin_runtime
            .job_run_create(staple_data::NewPluginJobRun {
                job_id: job.id,
                plugin_id: plugin_id.clone(),
                company_id: None,
                trigger: "manual".to_owned(),
            })
            .await;
    }
    Ok(see_other(&format!("/plugins/{plugin_id}")))
}

/// Plugin register form.
#[derive(Debug, serde::Deserialize)]
pub struct PluginRegisterForm {
    /// Unique plugin key.
    pub plugin_key: String,
    /// Package name.
    pub package_name: String,
    /// Version.
    pub version: String,
    /// Plugin API version.
    #[serde(default)]
    pub api_version: Option<i64>,
    /// Comma-separated categories.
    #[serde(default)]
    pub categories: Option<String>,
    /// Manifest JSON.
    #[serde(default)]
    pub manifest_json: Option<String>,
    /// Install order.
    #[serde(default)]
    pub install_order: Option<i64>,
    /// Package path.
    #[serde(default)]
    pub package_path: Option<String>,
}

/// Plugin company enable/disable form.
#[derive(Debug, serde::Deserialize)]
pub struct PluginCompanySettingsForm {
    /// Whether the plugin is enabled for the company (`1`/`0`).
    #[serde(default)]
    pub enabled: Option<String>,
}

/// Plugin status update form.
#[derive(Debug, serde::Deserialize)]
pub struct PluginStatusForm {
    /// New status.
    pub status: String,
    /// Optional last error.
    #[serde(default)]
    pub last_error: Option<String>,
}

/// Plugin config upsert form.
#[derive(Debug, serde::Deserialize)]
pub struct PluginConfigForm {
    /// Company id.
    pub company_id: String,
    /// Config JSON.
    pub config_json: String,
}

/// Plugin state set form.
#[derive(Debug, serde::Deserialize)]
pub struct PluginStateForm {
    /// Scope kind.
    pub scope_kind: String,
    /// Scope id.
    #[serde(default)]
    pub scope_id: Option<String>,
    /// Namespace.
    pub namespace: String,
    /// State key.
    pub key: String,
    /// JSON value.
    pub value: String,
}

/// Plugin entity upsert form.
#[derive(Debug, serde::Deserialize)]
pub struct PluginEntityForm {
    /// Company id.
    #[serde(default)]
    pub company_id: Option<String>,
    /// Entity type.
    pub entity_type: String,
    /// Scope kind.
    pub scope_kind: String,
    /// Scope id.
    #[serde(default)]
    pub scope_id: Option<String>,
    /// External id.
    #[serde(default)]
    pub external_id: Option<String>,
    /// Title.
    #[serde(default)]
    pub title: Option<String>,
    /// Status.
    #[serde(default)]
    pub status: Option<String>,
    /// Data JSON.
    pub data: String,
}

/// `{plugin_id}` path parameter for UI routes.
#[path_param(error = bad_request("Invalid plugin id"))]
pub(crate) struct PluginUiId(String);

/// `{job_key}` path parameter for UI routes.
#[path_param(error = bad_request("Invalid job key"))]
pub(crate) struct JobKeyUi(String);

// ---------------------------------------------------------------------------
// Instance settings UI actions
// ---------------------------------------------------------------------------

/// `POST /instance/user-roles/ui`.
#[route(POST "/instance/user-roles/ui")]
pub async fn instance_role_ui(
    cx: &Cx,
    Form(form): Form<InstanceUserForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let state = app_context::<AppState>(cx);
    let user_id = form.user_id.trim().to_owned();
    if !user_id.is_empty() {
        let _ = state
            .memberships
            .upsert_role(staple_data::NewInstanceUserRole {
                user_id,
                role: "instance_admin".to_owned(),
            })
            .await;
    }
    Ok(see_other("/instance/settings"))
}

/// `POST /board-api-keys/ui`.
#[route(POST "/board-api-keys/ui")]
pub async fn board_key_ui(
    cx: &Cx,
    Form(form): Form<InstanceUserForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let state = app_context::<AppState>(cx);
    let user_id = form.user_id.trim().to_owned();
    let name = form.name.unwrap_or_default().trim().to_owned();
    if !user_id.is_empty() && !name.is_empty() {
        let _ = state
            .board_keys
            .create_key(staple_data::NewBoardApiKey {
                user_id,
                name,
                expires_at: None,
            })
            .await;
    }
    Ok(see_other("/instance/settings"))
}

/// `POST /board-api-keys/{id}/revoke/ui`.
#[route(POST "/board-api-keys/{id}/revoke/ui")]
pub async fn board_key_revoke_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let _ = state.board_keys.revoke_key(&id).await;
    Ok(see_other("/instance/settings"))
}

/// `POST /cli-auth-challenges/ui`.
#[route(POST "/cli-auth-challenges/ui")]
pub async fn cli_challenge_ui(
    cx: &Cx,
    Form(form): Form<ChallengeForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let state = app_context::<AppState>(cx);
    let command = form.command.trim().to_owned();
    let pending_key_name = form.pending_key_name.unwrap_or_default().trim().to_owned();
    if !command.is_empty() && !pending_key_name.is_empty() {
        let _ = state
            .board_keys
            .create_challenge(staple_data::NewCliAuthChallenge {
                command,
                client_name: Some("board-ui".to_owned()),
                requested_access: "board".to_owned(),
                requested_company_id: None,
                pending_key_name,
                expires_at: "2999-01-01T00:00:00.000Z".to_owned(),
            })
            .await;
    }
    Ok(see_other("/instance/settings"))
}

/// `POST /cli-auth-challenges/{id}/approve/ui`.
#[route(POST "/cli-auth-challenges/{id}/approve/ui")]
pub async fn cli_challenge_approve_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let _ = state.board_keys.approve_challenge(&id, None).await;
    Ok(see_other("/instance/settings"))
}

/// `POST /cli-auth-challenges/{id}/cancel/ui`.
#[route(POST "/cli-auth-challenges/{id}/cancel/ui")]
pub async fn cli_challenge_cancel_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let _ = state.board_keys.cancel_challenge(&id).await;
    Ok(see_other("/instance/settings"))
}

/// `POST /instance/settings/general/ui` — saves instance general settings and
/// redirects back to the instance settings page.
#[route(POST "/instance/settings/general/ui")]
pub async fn instance_general_settings_ui(
    cx: &Cx,
    Form(form): Form<InstanceGeneralForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let state = app_context::<AppState>(cx);
    let general = serde_json::json!({
        "censorUsernameInLogs": form.censor_username_in_logs,
        "keyboardShortcuts": form.keyboard_shortcuts,
        "feedbackDataSharingPreference": form.feedback_data_sharing_preference,
        "backupRetention": {
            "dailyDays": form.backup_daily_days,
            "weeklyWeeks": form.backup_weekly_weeks,
            "monthlyMonths": form.backup_monthly_months,
        },
    });
    let _ = state
        .infrastructure
        .update_instance_settings(None, Some(general), None)
        .await;
    Ok(see_other("/instance/settings"))
}

/// `POST /instance/settings/experimental/ui` — saves instance experimental
/// settings (raw JSON) and redirects back to the instance settings page.
#[route(POST "/instance/settings/experimental/ui")]
pub async fn instance_experimental_settings_ui(
    cx: &Cx,
    Form(form): Form<InstanceExperimentalForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let state = app_context::<AppState>(cx);
    let experimental = serde_json::from_str::<serde_json::Value>(&form.json)
        .unwrap_or_else(|_| serde_json::json!({}));
    let _ = state
        .infrastructure
        .update_instance_settings(None, None, Some(experimental))
        .await;
    Ok(see_other("/instance/settings"))
}

/// `POST /profile/settings/ui` — saves the board profile sidebar preference
/// and redirects back to the profile settings page.
#[route(POST "/profile/settings/ui")]
pub async fn profile_settings_ui(
    cx: &Cx,
    Form(form): Form<ProfileSettingsForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let state = app_context::<AppState>(cx);
    let user_id = form.user_id.trim().to_owned();
    let mut company_order = Vec::new();
    for part in form.company_order.split(',') {
        let id = part.trim();
        if !id.is_empty() {
            company_order.push(id.to_owned());
        }
    }
    if !user_id.is_empty() {
        let _ = state
            .infrastructure
            .set_user_sidebar_preference(&user_id, serde_json::json!(company_order))
            .await;
    }
    Ok(see_other(&format!("/profile/settings?user={user_id}")))
}

// ---------------------------------------------------------------------------
// Form structs + path params
// ---------------------------------------------------------------------------

/// Instance general settings form fields.
#[derive(Debug, serde::Deserialize)]
pub struct InstanceGeneralForm {
    /// Censor usernames in logs.
    #[serde(default)]
    pub censor_username_in_logs: bool,
    /// Enable keyboard shortcuts.
    #[serde(default)]
    pub keyboard_shortcuts: bool,
    /// Feedback data sharing preference (`prompt` | `enabled` | `disabled`).
    #[serde(default = "default_feedback_preference")]
    pub feedback_data_sharing_preference: String,
    /// Backup retention: daily days.
    #[serde(default = "default_retention_days")]
    pub backup_daily_days: i64,
    /// Backup retention: weekly weeks.
    #[serde(default = "default_retention_weeks")]
    pub backup_weekly_weeks: i64,
    /// Backup retention: monthly months.
    #[serde(default = "default_retention_months")]
    pub backup_monthly_months: i64,
}

fn default_feedback_preference() -> String {
    "prompt".to_owned()
}

fn default_retention_days() -> i64 {
    30
}

fn default_retention_weeks() -> i64 {
    12
}

fn default_retention_months() -> i64 {
    12
}

/// Instance experimental settings form fields.
#[derive(Debug, serde::Deserialize)]
pub struct InstanceExperimentalForm {
    /// Experimental settings JSON.
    pub json: String,
}

/// Profile settings form fields.
#[derive(Debug, serde::Deserialize)]
pub struct ProfileSettingsForm {
    /// User id.
    pub user_id: String,
    /// Comma-separated company ids in sidebar order.
    #[serde(default)]
    pub company_order: String,
}

/// Agent status form.
#[derive(Debug, serde::Deserialize)]
pub struct AgentStatusForm {
    /// Target status.
    pub status: String,
    /// Pause reason.
    #[serde(default)]
    pub pause_reason: Option<String>,
}

/// Agent budget form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentBudgetForm {
    /// Monthly budget in cents.
    pub budget_monthly_cents: i64,
}

/// Decision queue form.
#[derive(Debug, serde::Deserialize)]
pub struct DecisionQueueForm {
    /// Queue name.
    pub name: String,
}

/// Membership form.
#[derive(Debug, serde::Deserialize)]
pub struct MembershipForm {
    /// Principal id (user).
    pub name: String,
    /// Membership role.
    #[serde(default)]
    pub role: Option<String>,
}

/// Invite form.
#[derive(Debug, serde::Deserialize)]
pub struct InviteForm {
    /// Invite name.
    pub name: Option<String>,
}

/// Routine form.
#[derive(Debug, serde::Deserialize)]
pub struct RoutineForm {
    /// Routine title.
    pub title: String,
}

/// Secret form.
#[derive(Debug, serde::Deserialize)]
pub struct SecretForm {
    /// Secret name.
    #[serde(default)]
    pub name: Option<String>,
    /// Secret value.
    #[serde(default)]
    pub value: Option<String>,
}

/// Skill form.
#[derive(Debug, serde::Deserialize)]
pub struct SkillForm {
    /// Skill name.
    #[serde(default)]
    pub name: Option<String>,
    /// Skill description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Instance user form (userId + optional name).
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceUserForm {
    /// User id.
    pub user_id: String,
    /// Optional key name.
    #[serde(default)]
    pub name: Option<String>,
}

/// CLI challenge form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeForm {
    /// Command text.
    pub command: String,
    /// Pending key name.
    #[serde(default)]
    pub pending_key_name: Option<String>,
}

/// `{agent_id}` path parameter for UI routes.
#[path_param(error = bad_request("Invalid agent id"))]
pub(crate) struct AgentUiId(String);

/// `{source_kind}` path parameter.
#[path_param(error = bad_request("Invalid source kind"))]
pub(crate) struct SourceKindUi(String);

/// `{source_id}` path parameter.
#[path_param(error = bad_request("Invalid source id"))]
pub(crate) struct SourceIdUi(String);

/// `POST /projects/{id}/edit/ui` — updates a project, redirects to it.
#[route(POST "/projects/{id}/edit/ui")]
pub async fn project_edit_ui(
    cx: &Cx,
    Form(form): Form<ProjectEditForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let project_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(project) = state.projects.get(&project_id).await.ok().flatten() else {
        return Ok(see_other("/"));
    };
    let name = form.name.as_deref().unwrap_or("").trim().to_owned();
    let mut patch = staple_data::ProjectPatch {
        goal_id: None,
        name: (!name.is_empty()).then_some(name),
        description: Some(Some(form.description.clone().unwrap_or_default())),
        status: form.status.clone().or(Some(project.status.clone())),
        lead_agent_id: None,
        target_date: None,
        execution_workspace_policy: None,
    };
    if form.status.is_none() {
        patch.status = None;
    }
    let _ = state.projects.update(&project_id, patch).await;
    Ok(see_other(&format!("/projects/{project_id}")))
}

/// `POST /companies/{company_id}/workspaces/{id}/materialize/ui` — materializes
/// a git workspace and redirects back to the workspaces page.
#[route(POST "/companies/{company_id}/workspaces/{id}/materialize/ui")]
pub async fn workspace_materialize_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let workspace_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let secret_name = "github_token".to_owned();
    if let Ok(Some(workspace)) = state
        .workspaces
        .get_execution_workspace(&company_id, &workspace_id)
        .await
        && let Some(repo_url) = workspace.repo_url.clone().filter(|u| !u.is_empty())
        && let Ok(Some(token)) = state
            .secrets
            .get_secret_value(&company_id, &secret_name)
            .await
    {
        match crate::git::materialize_repo(&repo_url, &token, &company_id, &workspace_id, false)
            .await
        {
            Ok((_, _)) => {
                let _ = state
                    .workspaces
                    .set_materialization(&company_id, &workspace_id, true, None, Some(secret_name))
                    .await;
            }
            Err(error) => {
                let redacted = crate::git::redact_credentials(&error, &token);
                let _ = state
                    .workspaces
                    .set_materialization(
                        &company_id,
                        &workspace_id,
                        false,
                        Some(redacted),
                        Some(secret_name),
                    )
                    .await;
            }
        }
    }
    Ok(see_other(&format!("/workspaces/{workspace_id}")))
}

/// Project edit form.
#[derive(Debug, serde::Deserialize)]
pub struct ProjectEditForm {
    /// Project name.
    #[serde(default)]
    pub name: Option<String>,
    /// Project description.
    #[serde(default)]
    pub description: Option<String>,
    /// Project status.
    #[serde(default)]
    pub status: Option<String>,
}

/// `POST /adapters/{type}/invoke/ui` — invokes an adapter run, redirects to
/// the adapter detail with the new run id.
#[route(POST "/adapters/{type}/invoke/ui")]
pub async fn adapter_invoke_ui(
    cx: &Cx,
    Form(form): Form<AdapterInvokeForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let adapter_type = path_param::<Type>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(adapter) = state.adapters.get(&adapter_type) else {
        return Ok(see_other("/adapters"));
    };
    let task = form.task.trim().to_owned();
    if !task.is_empty()
        && let Ok(handle) = adapter
            .invoke(staple_adapters::InvocationInput {
                task,
                cwd: None,
                env: vec![],
            })
            .await
    {
        return Ok(see_other(&format!(
            "/adapters/{adapter_type}?runId={}",
            handle.run_id
        )));
    }
    Ok(see_other(&format!("/adapters/{adapter_type}")))
}

/// `POST /adapters/{type}/runs/{run_id}/cancel/ui` — cancels a run.
#[route(POST "/adapters/{type}/runs/{run_id}/cancel/ui")]
pub async fn adapter_cancel_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let adapter_type = path_param::<Type>(cx)?.to_string();
    let run_id = path_param::<RunId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if let Some(adapter) = state.adapters.get(&adapter_type) {
        let _ = adapter.cancel(&run_id).await;
    }
    Ok(see_other(&format!(
        "/adapters/{adapter_type}?runId={run_id}"
    )))
}

/// Adapter invoke form.
#[derive(Debug, serde::Deserialize)]
pub struct AdapterInvokeForm {
    /// Task instructions.
    pub task: String,
}

/// `{type}` path parameter for UI routes.
#[path_param(error = bad_request("Invalid adapter type"))]
pub(crate) struct Type(String);

/// `{run_id}` path parameter for UI routes.
#[path_param(error = bad_request("Invalid run id"))]
pub(crate) struct RunId(String);

/// `POST /companies/{company_id}/cases/ui` — creates a case, redirects to the
/// cases list.
#[route(POST "/companies/{company_id}/cases/ui")]
pub async fn case_create_ui(
    cx: &Cx,
    Form(form): Form<CaseForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let case_type = form.case_type.trim().to_owned();
    let title = form.title.trim().to_owned();
    if !case_type.is_empty() && !title.is_empty() {
        let _ = state
            .cases
            .create(staple_data::NewCase {
                company_id: company_id.clone(),
                project_id: None,
                case_type,
                key: None,
                title,
                summary: None,
                fields: None,
                parent_case_id: None,
                created_by_agent_id: None,
                created_by_user_id: Some(crate::auth::current_actor(cx)),
            })
            .await;
    }
    Ok(see_other(&format!("/companies/{company_id}/cases")))
}

/// `POST /cases/{id}/status/ui` — moves a case, redirects to it.
#[route(POST "/cases/{id}/status/ui")]
pub async fn case_status_ui(
    cx: &Cx,
    Form(form): Form<CaseStatusForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let case_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let status = form.status.trim().to_owned();
    if !status.is_empty()
        && let Ok(Some(company_id)) = state.cases.company_of(&case_id).await
    {
        let _ = state.cases.set_status(&company_id, &case_id, &status).await;
    }
    Ok(see_other(&format!("/cases/{case_id}")))
}

/// Case create form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseForm {
    /// Case type.
    pub case_type: String,
    /// Title.
    pub title: String,
}

/// Case status form.
#[derive(Debug, serde::Deserialize)]
pub struct CaseStatusForm {
    /// Target status.
    pub status: String,
}

/// `POST /companies/{company_id}/pipelines/ui` — creates a pipeline.
#[route(POST "/companies/{company_id}/pipelines/ui")]
pub async fn pipeline_create_ui(
    cx: &Cx,
    Form(form): Form<PipelineForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let key = form.key.trim().to_owned();
    let name = form.name.trim().to_owned();
    if key.is_empty() || name.is_empty() {
        return Ok(see_other_flash(
            &format!("/companies/{company_id}/pipelines"),
            "invalid",
        ));
    }
    if let Ok(pipeline) = state
        .pipelines
        .create_pipeline(staple_data::NewPipeline {
            company_id: company_id.clone(),
            project_id: None,
            key,
            name,
            description: None,
            enforce_transitions: form.enforce == Some("1".to_owned()),
            created_by_user_id: Some(crate::auth::current_actor(cx)),
        })
        .await
    {
        return Ok(see_other_flash(
            &format!("/pipelines/{}", pipeline.id),
            "created",
        ));
    }
    Ok(see_other_flash(
        &format!("/companies/{company_id}/pipelines"),
        "error",
    ))
}

/// `POST /pipelines/{id}/stages/ui` — creates a stage.
#[route(POST "/pipelines/{id}/stages/ui")]
pub async fn pipeline_stage_ui(
    cx: &Cx,
    Form(form): Form<StageForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let pipeline_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .pipelines
        .company_of_pipeline(&pipeline_id)
        .await
        .ok()
        .flatten()
    else {
        return Ok(see_other("/"));
    };
    let key = form.key.trim().to_owned();
    let name = form.name.trim().to_owned();
    if !key.is_empty() && !name.is_empty() {
        let _ = state
            .pipelines
            .create_stage(staple_data::NewStage {
                company_id: company_id.clone(),
                pipeline_id: pipeline_id.clone(),
                key,
                name,
                kind: form.kind.clone(),
                position: form.position.unwrap_or(1),
                config: None,
            })
            .await;
    }
    Ok(see_other(&format!("/pipelines/{pipeline_id}")))
}

/// `POST /pipelines/{id}/transitions/ui` — creates a transition edge.
#[route(POST "/pipelines/{id}/transitions/ui")]
pub async fn pipeline_transition_ui(
    cx: &Cx,
    Form(form): Form<TransitionForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let pipeline_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .pipelines
        .company_of_pipeline(&pipeline_id)
        .await
        .ok()
        .flatten()
    else {
        return Ok(see_other("/"));
    };
    if let (Some(from), Some(to)) = (&form.from_stage_id, &form.to_stage_id) {
        let _ = state
            .pipelines
            .create_transition(staple_data::NewTransition {
                company_id: company_id.clone(),
                pipeline_id: pipeline_id.clone(),
                from_stage_id: from.clone(),
                to_stage_id: to.clone(),
                label: None,
            })
            .await;
    }
    Ok(see_other(&format!("/pipelines/{pipeline_id}")))
}

/// `POST /pipelines/{id}/cases/ui` — creates a pipeline case.
#[route(POST "/pipelines/{id}/cases/ui")]
pub async fn pipeline_case_ui(
    cx: &Cx,
    Form(form): Form<PipelineCaseForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let pipeline_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .pipelines
        .company_of_pipeline(&pipeline_id)
        .await
        .ok()
        .flatten()
    else {
        return Ok(see_other("/"));
    };
    let case_key = form.case_key.trim().to_owned();
    let title = form.title.trim().to_owned();
    if !case_key.is_empty()
        && !title.is_empty()
        && let Some(stage_id) = &form.stage_id
    {
        let _ = state
            .pipelines
            .create_case(staple_data::NewPipelineCase {
                company_id: company_id.clone(),
                pipeline_id: pipeline_id.clone(),
                stage_id: stage_id.clone(),
                case_key,
                title,
                summary: None,
                fields: None,
                workspace_ref: None,
                parent_case_id: None,
                created_by_user_id: Some(crate::auth::current_actor(cx)),
            })
            .await;
    }
    Ok(see_other(&format!("/pipelines/{pipeline_id}")))
}

/// `POST /pipeline-cases/{id}/move/ui` — moves a pipeline case.
#[route(POST "/pipeline-cases/{id}/move/ui")]
pub async fn pipeline_case_move_ui(
    cx: &Cx,
    Form(form): Form<MoveForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let case_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .pipelines
        .company_of_case(&case_id)
        .await
        .ok()
        .flatten()
    else {
        return Ok(see_other("/"));
    };
    if let Some(to_stage_id) = &form.to_stage_id {
        let _ = state
            .pipelines
            .move_case(
                &company_id,
                &case_id,
                to_stage_id,
                "user",
                Some(crate::auth::current_actor(cx)),
                None,
                false,
            )
            .await;
    }
    Ok(see_other(&format!("/pipeline-cases/{case_id}")))
}

/// Pipeline create form.
#[derive(Debug, serde::Deserialize)]
pub struct PipelineForm {
    /// Pipeline key.
    pub key: String,
    /// Pipeline name.
    pub name: String,
    /// Enforce transitions (checkbox `1`).
    #[serde(default)]
    pub enforce: Option<String>,
}

/// Pipeline settings form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineSettingsForm {
    /// Pipeline name.
    pub name: String,
    /// Pipeline description.
    #[serde(default)]
    pub description: Option<String>,
    /// Pipeline status (`active` | `archived`).
    #[serde(default)]
    pub status: Option<String>,
}

/// `POST /pipelines/{id}/settings/ui` — updates pipeline settings, redirects
/// back to the pipeline detail page.
#[route(POST "/pipelines/{id}/settings/ui")]
pub async fn pipeline_settings_ui(
    cx: &Cx,
    Form(form): Form<PipelineSettingsForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let pipeline_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .pipelines
        .company_of_pipeline(&pipeline_id)
        .await
        .ok()
        .flatten()
    else {
        return Ok(see_other("/"));
    };
    let name = form.name.trim().to_owned();
    let code = if name.is_empty() {
        "invalid"
    } else if state
        .pipelines
        .update_pipeline(
            &company_id,
            &pipeline_id,
            Some(name),
            Some(Some(form.description.unwrap_or_default())),
            Some(form.status.as_deref() == Some("archived")),
        )
        .await
        .is_ok()
    {
        "saved"
    } else {
        "error"
    };
    Ok(see_other_flash(&format!("/pipelines/{pipeline_id}"), code))
}

/// Stage form.
#[derive(Debug, serde::Deserialize)]
pub struct StageForm {
    /// Stage key.
    pub key: String,
    /// Stage name.
    pub name: String,
    /// Stage kind.
    pub kind: String,
    /// Stage position.
    #[serde(default)]
    pub position: Option<i64>,
}

/// Transition form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransitionForm {
    /// From stage id.
    pub from_stage_id: Option<String>,
    /// To stage id.
    pub to_stage_id: Option<String>,
}

/// Pipeline case form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineCaseForm {
    /// Case key.
    pub case_key: String,
    /// Title.
    pub title: String,
    /// Stage id.
    pub stage_id: Option<String>,
}

/// Move form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveForm {
    /// Target stage id.
    pub to_stage_id: Option<String>,
}

/// `POST /pipeline-cases/{id}/issue-links/ui` — links an issue to a case.
#[route(POST "/pipeline-cases/{id}/issue-links/ui")]
pub async fn pipeline_link_issue_ui(
    cx: &Cx,
    Form(form): Form<LinkIssueUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let case_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if let Some(company_id) = state
        .pipelines
        .company_of_case(&case_id)
        .await
        .ok()
        .flatten()
    {
        let issue_id = form.issue_id.trim().to_owned();
        if !issue_id.is_empty() {
            let _ = state
                .pipelines
                .link_issue(&company_id, &case_id, &issue_id, &form.role)
                .await;
        }
    }
    Ok(see_other(&format!("/pipeline-cases/{case_id}")))
}

/// `POST /pipeline-cases/{id}/blockers/ui` — adds a blocker edge.
#[route(POST "/pipeline-cases/{id}/blockers/ui")]
pub async fn pipeline_add_blocker_ui(
    cx: &Cx,
    Form(form): Form<BlockerUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let case_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if let Some(company_id) = state
        .pipelines
        .company_of_case(&case_id)
        .await
        .ok()
        .flatten()
    {
        let blocked_by = form.blocked_by_case_id.trim().to_owned();
        if !blocked_by.is_empty() {
            let _ = state
                .pipelines
                .add_blocker(&company_id, &case_id, &blocked_by)
                .await;
        }
    }
    Ok(see_other(&format!("/pipeline-cases/{case_id}")))
}

/// `POST /pipeline-cases/{id}/documents/ui` — links a document to a case.
#[route(POST "/pipeline-cases/{id}/documents/ui")]
pub async fn pipeline_link_case_doc_ui(
    cx: &Cx,
    Form(form): Form<DocumentUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let case_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if let Some(company_id) = state
        .pipelines
        .company_of_case(&case_id)
        .await
        .ok()
        .flatten()
    {
        let document_id = form.document_id.trim().to_owned();
        if !document_id.is_empty() && !form.key.trim().is_empty() {
            let _ = state
                .pipelines
                .link_case_document(&company_id, &case_id, &document_id, &form.key)
                .await;
        }
    }
    Ok(see_other(&format!("/pipeline-cases/{case_id}")))
}

/// `POST /pipelines/{id}/documents/ui` — links a document to a pipeline.
#[route(POST "/pipelines/{id}/documents/ui")]
pub async fn pipeline_link_doc_ui(
    cx: &Cx,
    Form(form): Form<DocumentUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let pipeline_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if let Some(company_id) = state
        .pipelines
        .company_of_pipeline(&pipeline_id)
        .await
        .ok()
        .flatten()
    {
        let document_id = form.document_id.trim().to_owned();
        if !document_id.is_empty() && !form.key.trim().is_empty() {
            let _ = state
                .pipelines
                .link_pipeline_document(&company_id, &pipeline_id, &document_id, &form.key)
                .await;
        }
    }
    Ok(see_other(&format!("/pipelines/{pipeline_id}")))
}

/// Link-issue form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkIssueUiForm {
    /// Issue id.
    pub issue_id: String,
    /// Role.
    #[serde(default)]
    pub role: String,
}

/// Blocker form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockerUiForm {
    /// Blocking case id.
    pub blocked_by_case_id: String,
}

/// Document form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentUiForm {
    /// Document id.
    pub document_id: String,
    /// Key.
    pub key: String,
}

/// `GET /static/board.js` — native drag & drop behavior for the board page.
#[route(GET "/static/board.js")]
pub async fn board_js(_cx: &Cx) -> Result<topcoat::router::Response, ApiError> {
    let body = topcoat::router::Body::from(include_str!("board.js"));
    let response = topcoat::router::Response::builder()
        .header("Content-Type", "text/javascript; charset=utf-8")
        .body(body)
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(response)
}

// --- Goals & Projects UI forms -------------------------------------------

/// Goal form (create + edit).
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalUiForm {
    /// Goal title.
    pub title: String,
    /// Optional description.
    pub description: Option<String>,
    /// `company | team | agent | task`.
    pub level: Option<String>,
    /// Parent goal id (`""` clears).
    pub parent_id: Option<String>,
    /// Owning agent id (`""` clears).
    pub owner_agent_id: Option<String>,
    /// Goal status.
    pub status: Option<String>,
}

/// Goal status form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalStatusForm {
    /// Target status.
    pub status: String,
}

/// Project form (create).
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUiForm {
    /// Project name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Linked goal id (`""` clears).
    pub goal_id: Option<String>,
    /// Lead agent id (`""` clears).
    pub lead_agent_id: Option<String>,
    /// Project status.
    pub status: Option<String>,
    /// Target date.
    pub target_date: Option<String>,
}

/// `POST /companies/{companyId}/goals/ui` — creates a goal, redirects to the
/// goals page.
#[route(POST "/companies/{company_id}/goals/ui")]
pub async fn create_goal_ui(
    cx: &Cx,
    Form(form): Form<GoalUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let title = form.title.trim().to_owned();
    let code = if title.is_empty() {
        "invalid"
    } else if let Ok(goal) = state
        .goals
        .create(staple_data::NewGoal {
            company_id: company_id.clone(),
            title,
            description: form.description,
            level: form.level.unwrap_or_else(|| "company".to_owned()),
            parent_id: form.parent_id.filter(|value| !value.is_empty()),
            owner_agent_id: form.owner_agent_id.filter(|value| !value.is_empty()),
            status: form.status.unwrap_or_else(|| "planned".to_owned()),
        })
        .await
    {
        let _ = log_activity(
            &state.activity,
            &goal.company_id,
            "goal.created",
            "goal",
            &goal.id,
            Some(serde_json::json!({ "title": goal.title })),
        )
        .await;
        "created"
    } else {
        "error"
    };
    Ok(see_other_flash(
        &format!("/companies/{company_id}/goals"),
        code,
    ))
}

/// `POST /goals/{id}/edit/ui` — updates a goal, redirects to its detail.
#[route(POST "/goals/{id}/edit/ui")]
pub async fn update_goal_ui(
    cx: &Cx,
    Form(form): Form<GoalUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let goal_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(_goal) = state.goals.get(&goal_id).await.ok().flatten() else {
        return Ok(see_other("/"));
    };
    let title = form.title.trim().to_owned();
    let code = if title.is_empty() {
        "invalid"
    } else if state
        .goals
        .update(
            &goal_id,
            staple_data::GoalPatch {
                title: Some(title),
                description: Some(form.description),
                level: form.level,
                parent_id: Some(form.parent_id.filter(|value| !value.is_empty())),
                owner_agent_id: Some(form.owner_agent_id.filter(|value| !value.is_empty())),
                status: form.status,
            },
        )
        .await
        .is_ok()
    {
        "saved"
    } else {
        "error"
    };
    Ok(see_other_flash(&format!("/goals/{goal_id}"), code))
}

/// `POST /goals/{id}/status/ui` — sets a goal status, redirects to its detail.
#[route(POST "/goals/{id}/status/ui")]
pub async fn set_goal_status_ui(
    cx: &Cx,
    Form(form): Form<GoalStatusForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let goal_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(goal) = state.goals.get(&goal_id).await.ok().flatten() else {
        return Ok(see_other("/"));
    };
    let company_id = goal.company_id.clone();
    let status = form.status.clone();
    let code = if state
        .goals
        .update(
            &goal_id,
            staple_data::GoalPatch {
                title: None,
                description: None,
                level: None,
                parent_id: None,
                owner_agent_id: None,
                status: Some(status),
            },
        )
        .await
        .is_ok()
    {
        "updated"
    } else {
        "error"
    };
    let _ = log_activity(
        &state.activity,
        &company_id,
        "goal.status_updated",
        "goal",
        &goal_id,
        Some(serde_json::json!({ "status": form.status })),
    )
    .await;
    Ok(see_other_flash(&format!("/goals/{goal_id}"), code))
}

/// `POST /companies/{companyId}/projects/ui` — creates a project, redirects to
/// the projects page.
#[route(POST "/companies/{company_id}/projects/ui")]
pub async fn create_project_ui(
    cx: &Cx,
    Form(form): Form<ProjectUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let name = form.name.trim().to_owned();
    let code = if name.is_empty() {
        "invalid"
    } else if let Ok(project) = state
        .projects
        .create(staple_data::NewProject {
            company_id: company_id.clone(),
            goal_id: form.goal_id.filter(|value| !value.is_empty()),
            name,
            description: form.description,
            status: form.status.unwrap_or_else(|| "backlog".to_owned()),
            lead_agent_id: form.lead_agent_id.filter(|value| !value.is_empty()),
            target_date: form.target_date,
            env: None,
            execution_workspace_policy: None,
        })
        .await
    {
        let _ = log_activity(
            &state.activity,
            &project.company_id,
            "project.created",
            "project",
            &project.id,
            Some(serde_json::json!({ "name": project.name })),
        )
        .await;
        "created"
    } else {
        "error"
    };
    Ok(see_other_flash(
        &format!("/companies/{company_id}/projects"),
        code,
    ))
}

// --- Decisions & training example UI forms --------------------------------

/// Decision create form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionUiForm {
    /// Decision title.
    pub title: String,
    /// Decision body.
    pub body: Option<String>,
    /// Options JSON array.
    pub options: Option<String>,
    /// Decision status.
    pub status: Option<String>,
    /// ISO 8601 expiry.
    pub expires_at: String,
    /// Origin agent id.
    pub origin_agent_id: String,
    /// Origin issue id.
    pub origin_issue_id: String,
    /// Origin run id.
    pub origin_run_id: String,
}

/// Decision resolve form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveDecisionUiForm {
    /// Target status.
    pub status: String,
    /// Chosen option id.
    pub chosen_option_id: Option<String>,
    /// Deciding user id.
    pub decided_by_user_id: Option<String>,
}

/// Training example create form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingExampleUiForm {
    /// Source kind.
    pub source_kind: String,
    /// Source id.
    pub source_id: String,
    /// Issue id.
    pub issue_id: String,
    /// ISO 8601 cutoff.
    pub cutoff_at: String,
    /// Snapshot JSON.
    pub snapshot: Option<String>,
    /// Creating user id.
    pub created_by_user_id: String,
}

/// `POST /companies/{companyId}/decisions/ui` — creates a decision, redirects
/// to the decisions page.
#[route(POST "/companies/{company_id}/decisions/ui")]
pub async fn create_decision_ui(
    cx: &Cx,
    Form(form): Form<DecisionUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let title = form.title.trim().to_owned();
    let expires_at = form.expires_at.trim().to_owned();
    let options = serde_json::from_str(form.options.as_deref().unwrap_or("[]"))
        .unwrap_or_else(|_| serde_json::json!([]));
    let code = if title.is_empty()
        || expires_at.is_empty()
        || form.origin_agent_id.is_empty()
        || form.origin_issue_id.is_empty()
        || form.origin_run_id.is_empty()
    {
        "invalid"
    } else if let Ok(decision) = state
        .decision_actions
        .create_decision(staple_data::NewDecision {
            company_id: company_id.clone(),
            bundle_id: None,
            origin_agent_id: form.origin_agent_id,
            origin_issue_id: form.origin_issue_id,
            origin_run_id: form.origin_run_id,
            rule_key: None,
            title,
            body: form.body.unwrap_or_default(),
            options,
            inputs: None,
            status: form.status.unwrap_or_else(|| "open".to_owned()),
            execution_status: None,
            chosen_option_id: None,
            input_values: None,
            decided_by_user_id: None,
            decided_at: None,
            expires_at,
            idempotency_key: None,
            signed_spec: "manual".to_owned(),
            target_snapshots: serde_json::json!({}),
            continuation_policy: "none".to_owned(),
            metadata: serde_json::json!({}),
        })
        .await
    {
        let _ = log_activity(
            &state.activity,
            &decision.company_id,
            "decision.created",
            "decision",
            &decision.id,
            Some(serde_json::json!({ "title": decision.title })),
        )
        .await;
        "created"
    } else {
        "error"
    };
    Ok(see_other_flash(
        &format!("/companies/{company_id}/decisions"),
        code,
    ))
}

/// `POST /decisions/{id}/resolve/ui` — resolves a decision, redirects to its
/// detail.
#[route(POST "/decisions/{id}/resolve/ui")]
pub async fn resolve_decision_ui(
    cx: &Cx,
    Form(form): Form<ResolveDecisionUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let decision_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .decision_actions
        .decision_company(&decision_id)
        .await
        .ok()
        .flatten()
    else {
        return Ok(see_other("/"));
    };
    let _ = state
        .decision_actions
        .resolve_decision(staple_data::ResolveDecision {
            company_id: company_id.clone(),
            decision_id: decision_id.clone(),
            status: form.status,
            execution_status: None,
            chosen_option_id: form.chosen_option_id.filter(|value| !value.is_empty()),
            decided_by_user_id: form.decided_by_user_id.filter(|value| !value.is_empty()),
            decided_at: None,
            input_values: None,
        })
        .await;
    let _ = log_activity(
        &state.activity,
        &company_id,
        "decision.resolved",
        "decision",
        &decision_id,
        Some(serde_json::json!({ "id": decision_id })),
    )
    .await;
    Ok(see_other(&format!("/decisions/{decision_id}")))
}

/// `POST /companies/{companyId}/decision-training-examples/ui` — creates a
/// training example, redirects to the list.
#[route(POST "/companies/{company_id}/decision-training-examples/ui")]
pub async fn create_training_example_ui(
    cx: &Cx,
    Form(form): Form<TrainingExampleUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let snapshot = serde_json::from_str(form.snapshot.as_deref().unwrap_or("{}"))
        .unwrap_or_else(|_| serde_json::json!({}));
    if !form.source_kind.trim().is_empty()
        && !form.source_id.trim().is_empty()
        && !form.cutoff_at.trim().is_empty()
        && let Ok(example) = state
            .decision_actions
            .create_training_example(staple_data::NewDecisionTrainingExample {
                company_id: company_id.clone(),
                source_kind: form.source_kind,
                source_id: form.source_id,
                issue_id: form.issue_id,
                cutoff_at: form.cutoff_at,
                notes: String::new(),
                notes_history: serde_json::json!([]),
                decision_outcome: None,
                retention_policy: "scrub_deleted_comments_v1".to_owned(),
                snapshot,
                created_by_user_id: form.created_by_user_id,
            })
            .await
    {
        let _ = log_activity(
            &state.activity,
            &example.company_id,
            "decision.training_example_created",
            "decision_training_example",
            &example.id,
            Some(serde_json::json!({ "sourceKind": example.source_kind })),
        )
        .await;
    }
    Ok(see_other(&format!(
        "/companies/{company_id}/decision-training-examples"
    )))
}

// --- Status cards / summary slots / finance / feedback UI forms ----------

/// Status card create form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusCardUiForm {
    /// Card title.
    pub title: Option<String>,
    /// Interest prompt.
    pub interest_prompt: String,
    /// Queries JSON.
    pub queries: Option<String>,
    /// Refresh policy JSON.
    pub refresh_policy: Option<String>,
    /// Summarizer agent id.
    pub agent_id: Option<String>,
}

/// Summary slot upsert form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummarySlotUiForm {
    /// Scope kind.
    pub scope_kind: String,
    /// Scope id.
    pub scope_id: Option<String>,
    /// Slot key.
    pub slot_key: String,
    /// Slot status.
    pub status: Option<String>,
}

/// Finance event create form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinanceEventUiForm {
    /// Event kind.
    pub event_kind: String,
    /// Biller.
    pub biller: String,
    /// Amount in cents.
    pub amount_cents: i64,
    /// Debit or credit.
    pub direction: Option<String>,
    /// Agent id.
    pub agent_id: Option<String>,
    /// Issue id.
    pub issue_id: Option<String>,
    /// ISO 8601 occurred at.
    pub occurred_at: String,
}

/// Feedback vote create form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackVoteUiForm {
    /// Issue id.
    pub issue_id: String,
    /// Target type.
    pub target_type: String,
    /// Target id.
    pub target_id: String,
    /// Author user id.
    pub author_user_id: String,
    /// Vote (up/down).
    pub vote: String,
}

/// `POST /companies/{companyId}/status-cards/ui` — creates a status card.
#[route(POST "/companies/{company_id}/status-cards/ui")]
pub async fn create_status_card_ui(
    cx: &Cx,
    Form(form): Form<StatusCardUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let interest_prompt = form.interest_prompt.trim().to_owned();
    let queries = serde_json::from_str(form.queries.as_deref().unwrap_or("[]"))
        .unwrap_or_else(|_| serde_json::json!([]));
    let refresh_policy = serde_json::from_str(form.refresh_policy.as_deref().unwrap_or("{}"))
        .unwrap_or_else(|_| serde_json::json!({}));
    if !interest_prompt.is_empty()
        && let Ok(card) = state
            .scattered
            .create_status_card(staple_data::NewStatusCard {
                company_id: company_id.clone(),
                created_by_user_id: Some(crate::auth::current_actor(cx)),
                created_by_agent_id: None,
                title: form.title.filter(|value| !value.is_empty()),
                title_pinned: false,
                interest_prompt,
                queries,
                query_version: 0,
                agent_id: form.agent_id.filter(|value| !value.is_empty()),
                refresh_policy,
                state: "compiling".to_owned(),
                document_id: None,
            })
            .await
    {
        let _ = log_activity(
            &state.activity,
            &card.company_id,
            "status_card.created",
            "status_card",
            &card.id,
            Some(serde_json::json!({ "title": card.title })),
        )
        .await;
    }
    Ok(see_other(&format!("/companies/{company_id}/status-cards")))
}

/// `POST /companies/{companyId}/status-cards/{id}/archive/ui` — archives a
/// status card.
#[route(POST "/companies/{company_id}/status-cards/{id}/archive/ui")]
pub async fn archive_status_card_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let card_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let _ = state
        .scattered
        .archive_status_card(&company_id, &card_id)
        .await;
    Ok(see_other(&format!("/companies/{company_id}/status-cards")))
}

/// `POST /companies/{companyId}/summary-slots/ui` — upserts a summary slot.
#[route(POST "/companies/{company_id}/summary-slots/ui")]
pub async fn upsert_summary_slot_ui(
    cx: &Cx,
    Form(form): Form<SummarySlotUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let _ = state
        .scattered
        .upsert_summary_slot(staple_data::NewSummarySlot {
            company_id: company_id.clone(),
            scope_kind: form.scope_kind,
            scope_id: form.scope_id.filter(|value| !value.is_empty()),
            slot_key: form.slot_key,
            document_id: None,
            status: form.status.unwrap_or_else(|| "idle".to_owned()),
            failure_reason: None,
            generating_issue_id: None,
            last_generated_at: None,
            last_generated_by_agent_id: None,
            last_model: None,
        })
        .await;
    Ok(see_other(&format!("/companies/{company_id}/summary-slots")))
}

/// `POST /companies/{companyId}/finance-events/ui` — creates a finance event.
#[route(POST "/companies/{company_id}/finance-events/ui")]
pub async fn create_finance_event_ui(
    cx: &Cx,
    Form(form): Form<FinanceEventUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if !form.event_kind.trim().is_empty()
        && !form.biller.trim().is_empty()
        && !form.occurred_at.trim().is_empty()
        && let Ok(event) = state
            .scattered
            .create_finance_event(staple_data::NewFinanceEvent {
                company_id: company_id.clone(),
                agent_id: form.agent_id.filter(|value| !value.is_empty()),
                issue_id: form.issue_id.filter(|value| !value.is_empty()),
                project_id: None,
                goal_id: None,
                heartbeat_run_id: None,
                cost_event_id: None,
                billing_code: None,
                description: None,
                event_kind: form.event_kind,
                direction: form.direction.unwrap_or_else(|| "debit".to_owned()),
                biller: form.biller,
                provider: None,
                execution_adapter_type: None,
                pricing_tier: None,
                region: None,
                model: None,
                quantity: None,
                unit: None,
                amount_cents: form.amount_cents,
                currency: "USD".to_owned(),
                estimated: false,
                external_invoice_id: None,
                metadata_json: None,
                occurred_at: form.occurred_at,
            })
            .await
    {
        let _ = log_activity(
            &state.activity,
            &event.company_id,
            "finance_event.created",
            "finance_event",
            &event.id,
            Some(serde_json::json!({ "eventKind": event.event_kind })),
        )
        .await;
    }
    Ok(see_other(&format!(
        "/companies/{company_id}/finance-events"
    )))
}

/// `POST /companies/{companyId}/feedback-votes/ui` — creates a feedback vote.
#[route(POST "/companies/{company_id}/feedback-votes/ui")]
pub async fn create_feedback_vote_ui(
    cx: &Cx,
    Form(form): Form<FeedbackVoteUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if !form.target_type.trim().is_empty()
        && !form.target_id.trim().is_empty()
        && !form.author_user_id.trim().is_empty()
        && !form.vote.trim().is_empty()
        && let Ok(vote) = state
            .scattered
            .create_feedback_vote(staple_data::NewFeedbackVote {
                company_id: company_id.clone(),
                issue_id: form.issue_id,
                target_type: form.target_type,
                target_id: form.target_id,
                author_user_id: form.author_user_id,
                vote: form.vote,
                reason: None,
                shared_with_labs: false,
                shared_at: None,
                consent_version: None,
                redaction_summary: None,
            })
            .await
    {
        let _ = log_activity(
            &state.activity,
            &vote.company_id,
            "feedback.vote_created",
            "feedback_vote",
            &vote.id,
            Some(serde_json::json!({ "targetType": vote.target_type })),
        )
        .await;
    }
    Ok(see_other(&format!(
        "/companies/{company_id}/feedback-votes"
    )))
}

// --- Skill catalog & secret bindings UI forms ----------------------------

/// Skill version publish form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillVersionUiForm {
    /// Version label.
    pub label: Option<String>,
}

/// Skill policy form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillPolicyUiForm {
    /// Default effect (allow/deny).
    pub default_effect: String,
    /// Rules JSON.
    pub rules: Option<String>,
}

/// Skill comment form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillCommentUiForm {
    /// Comment body.
    pub body: String,
}

/// Skill star form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillStarUiForm {
    /// Starring user id.
    pub user_id: Option<String>,
}

/// Skill test input form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillTestInputUiForm {
    /// Test name.
    pub name: String,
    /// Test content.
    pub content: String,
}

/// Secret provider config form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretProviderUiForm {
    /// Provider key.
    pub provider: String,
    /// Display name.
    pub display_name: String,
    /// Config JSON.
    pub config: Option<String>,
}

/// Secret binding form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretBindingUiForm {
    /// Secret id.
    pub secret_id: String,
    /// Target type.
    pub target_type: String,
    /// Target id.
    pub target_id: String,
    /// Config path.
    pub config_path: String,
    /// Version selector.
    pub version_selector: Option<String>,
}

/// User secret definition form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSecretDefinitionUiForm {
    /// Secret key.
    pub key: String,
    /// Display name.
    pub name: String,
    /// Provider.
    pub provider: String,
    /// Managed mode.
    pub managed_mode: Option<String>,
}

/// User secret declaration form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSecretDeclarationUiForm {
    /// Definition id.
    pub user_secret_definition_id: String,
    /// Target type.
    pub target_type: String,
    /// Target id.
    pub target_id: String,
    /// Env key.
    pub env_key: String,
    /// Config path.
    pub config_path: String,
}

/// `POST /companies/{companyId}/skills/{skillId}/version/ui` — publishes a
/// skill version.
#[route(POST "/companies/{company_id}/skills/{skill_id}/version/ui")]
pub async fn publish_skill_version_ui(
    cx: &Cx,
    Form(form): Form<SkillVersionUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let skill_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let _ = state
        .skill_catalog
        .publish_version(staple_data::NewSkillVersion {
            company_id: company_id.clone(),
            company_skill_id: skill_id.clone(),
            label: form.label.filter(|value| !value.is_empty()),
            release_id: None,
            release_name: None,
            released_at: None,
            file_inventory: serde_json::json!([]),
            author_agent_id: None,
            author_user_id: Some(crate::auth::current_actor(cx)),
        })
        .await;
    Ok(see_other(&format!(
        "/companies/{company_id}/skills/{skill_id}"
    )))
}

/// `POST /companies/{companyId}/skills/policy/ui` — sets the company skill
/// policy.
#[route(POST "/companies/{company_id}/skills/policy/ui")]
pub async fn set_skill_policy_ui(
    cx: &Cx,
    Form(form): Form<SkillPolicyUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let rules = serde_json::from_str(form.rules.as_deref().unwrap_or("[]"))
        .unwrap_or_else(|_| serde_json::json!([]));
    let _ = state
        .skill_catalog
        .set_policy(staple_data::SetSkillPolicy {
            company_id: company_id.clone(),
            schema_version: 1,
            default_effect: form.default_effect,
            rules,
        })
        .await;
    Ok(see_other(&format!("/companies/{company_id}/skills")))
}

/// `POST /companies/{companyId}/skills/{skillId}/comments/ui` — adds a skill
/// comment.
#[route(POST "/companies/{company_id}/skills/{skill_id}/comments/ui")]
pub async fn add_skill_comment_ui(
    cx: &Cx,
    Form(form): Form<SkillCommentUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let skill_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if !form.body.trim().is_empty() {
        let _ = state
            .skill_catalog
            .create_comment(staple_data::NewSkillComment {
                company_id: company_id.clone(),
                company_skill_id: skill_id.clone(),
                parent_comment_id: None,
                author_agent_id: None,
                author_user_id: Some(crate::auth::current_actor(cx)),
                body: form.body,
            })
            .await;
    }
    Ok(see_other(&format!(
        "/companies/{company_id}/skills/{skill_id}"
    )))
}

/// `POST /companies/{companyId}/skills/{skillId}/stars/ui` — stars a skill.
#[route(POST "/companies/{company_id}/skills/{skill_id}/stars/ui")]
pub async fn star_skill_ui(
    cx: &Cx,
    Form(form): Form<SkillStarUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let skill_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let _ = state
        .skill_catalog
        .create_star(staple_data::NewSkillStar {
            company_id: company_id.clone(),
            company_skill_id: skill_id.clone(),
            agent_id: None,
            user_id: form.user_id.filter(|value| !value.is_empty()),
        })
        .await;
    Ok(see_other(&format!(
        "/companies/{company_id}/skills/{skill_id}"
    )))
}

/// `POST /companies/{companyId}/skills/{skillId}/test-inputs/ui` — adds a
/// skill test input.
#[route(POST "/companies/{company_id}/skills/{skill_id}/test-inputs/ui")]
pub async fn add_skill_test_input_ui(
    cx: &Cx,
    Form(form): Form<SkillTestInputUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let skill_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if !form.name.trim().is_empty() {
        let _ = state
            .skill_catalog
            .create_test_input(staple_data::NewSkillTestInput {
                company_id: company_id.clone(),
                skill_id: skill_id.clone(),
                name: form.name,
                content: form.content,
                created_by: Some(crate::auth::current_actor(cx)),
            })
            .await;
    }
    Ok(see_other(&format!(
        "/companies/{company_id}/skills/{skill_id}"
    )))
}

/// `POST /companies/{companyId}/secret-bindings/providers/ui` — creates a
/// secret provider config.
#[route(POST "/companies/{company_id}/secret-bindings/providers/ui")]
pub async fn create_secret_provider_ui(
    cx: &Cx,
    Form(form): Form<SecretProviderUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let config = serde_json::from_str(form.config.as_deref().unwrap_or("{}"))
        .unwrap_or_else(|_| serde_json::json!({}));
    if !form.provider.trim().is_empty() && !form.display_name.trim().is_empty() {
        let _ = state
            .secret_bindings
            .create_provider_config(staple_data::NewSecretProviderConfig {
                company_id: company_id.clone(),
                provider: form.provider,
                display_name: form.display_name,
                status: "active".to_owned(),
                is_default: false,
                config,
                created_by_agent_id: None,
                created_by_user_id: Some(crate::auth::current_actor(cx)),
            })
            .await;
    }
    Ok(see_other(&format!(
        "/companies/{company_id}/secret-bindings"
    )))
}

/// `POST /companies/{companyId}/secret-bindings/bindings/ui` — sets a secret
/// binding.
#[route(POST "/companies/{company_id}/secret-bindings/bindings/ui")]
pub async fn set_secret_binding_ui(
    cx: &Cx,
    Form(form): Form<SecretBindingUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if !form.secret_id.trim().is_empty()
        && !form.target_type.trim().is_empty()
        && !form.target_id.trim().is_empty()
        && !form.config_path.trim().is_empty()
    {
        let _ = state
            .secret_bindings
            .set_binding(staple_data::NewSecretBinding {
                company_id: company_id.clone(),
                secret_id: form.secret_id,
                target_type: form.target_type,
                target_id: form.target_id,
                config_path: form.config_path,
                version_selector: form
                    .version_selector
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "latest".to_owned()),
                required: false,
                label: None,
                projection_class: "env".to_owned(),
                projection_allowlist_key: None,
            })
            .await;
    }
    Ok(see_other(&format!(
        "/companies/{company_id}/secret-bindings"
    )))
}

/// `POST /companies/{companyId}/user-secrets/definitions/ui` — creates a user
/// secret definition.
#[route(POST "/companies/{company_id}/user-secrets/definitions/ui")]
pub async fn create_user_secret_definition_ui(
    cx: &Cx,
    Form(form): Form<UserSecretDefinitionUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if !form.key.trim().is_empty()
        && !form.name.trim().is_empty()
        && !form.provider.trim().is_empty()
    {
        let _ = state
            .secret_bindings
            .create_user_secret_definition(staple_data::NewUserSecretDefinition {
                company_id: company_id.clone(),
                key: form.key,
                name: form.name,
                description: None,
                status: "active".to_owned(),
                provider: form.provider,
                managed_mode: form.managed_mode.unwrap_or_else(|| "manual".to_owned()),
                provider_config_id: None,
                provider_metadata: None,
                usage_guidance: None,
                created_by_agent_id: None,
                created_by_user_id: Some(crate::auth::current_actor(cx)),
            })
            .await;
    }
    Ok(see_other(&format!("/companies/{company_id}/user-secrets")))
}

/// `POST /companies/{companyId}/user-secrets/declarations/ui` — creates a user
/// secret declaration.
#[route(POST "/companies/{company_id}/user-secrets/declarations/ui")]
pub async fn create_user_secret_declaration_ui(
    cx: &Cx,
    Form(form): Form<UserSecretDeclarationUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if !form.user_secret_definition_id.trim().is_empty()
        && !form.target_type.trim().is_empty()
        && !form.target_id.trim().is_empty()
        && !form.env_key.trim().is_empty()
    {
        let _ = state
            .secret_bindings
            .create_user_secret_declaration(staple_data::NewUserSecretDeclaration {
                company_id: company_id.clone(),
                user_secret_definition_id: form.user_secret_definition_id,
                target_type: form.target_type,
                target_id: form.target_id,
                config_path: form.config_path,
                env_key: form.env_key,
                version_selector: "latest".to_owned(),
                required: false,
                allow_missing_override: true,
                label: None,
            })
            .await;
    }
    Ok(see_other(&format!("/companies/{company_id}/user-secrets")))
}

// --- Infrastructure UI forms (folders / watchdogs / users / envs) --------

/// Folder create form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderUiForm {
    /// Folder kind.
    pub kind: String,
    /// Folder name.
    pub name: String,
    /// Folder slug.
    pub slug: String,
    /// Parent folder id.
    pub parent_id: Option<String>,
    /// Color.
    pub color: Option<String>,
}

/// Watchdog create form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchdogUiForm {
    /// Watchdog agent id.
    pub watchdog_agent_id: String,
    /// Instructions.
    pub instructions: Option<String>,
    /// Status.
    pub status: Option<String>,
}

/// User create form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserUiForm {
    /// User id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Email.
    pub email: String,
}

/// Environment create form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvUiForm {
    /// Environment name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Driver.
    pub driver: Option<String>,
}

/// `POST /companies/{companyId}/folders/ui` — creates a folder.
#[route(POST "/companies/{company_id}/folders/ui")]
pub async fn create_folder_ui(
    cx: &Cx,
    Form(form): Form<FolderUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if !form.kind.trim().is_empty() && !form.name.trim().is_empty() && !form.slug.trim().is_empty()
    {
        let _ = state
            .infrastructure
            .create_folder(staple_data::NewFolder {
                company_id: company_id.clone(),
                kind: form.kind,
                parent_id: form.parent_id.filter(|value| !value.is_empty()),
                name: form.name,
                slug: form.slug,
                system_key: None,
                color: form.color.filter(|value| !value.is_empty()),
                position: 0,
            })
            .await;
    }
    Ok(see_other(&format!("/companies/{company_id}/folders")))
}

/// `POST /companies/{companyId}/folders/{id}/delete/ui` — deletes a folder.
#[route(POST "/companies/{company_id}/folders/{id}/delete/ui")]
pub async fn delete_folder_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let folder_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let _ = state
        .infrastructure
        .delete_folder(&company_id, &folder_id)
        .await;
    Ok(see_other(&format!("/companies/{company_id}/folders")))
}

/// `POST /issues/{issueId}/watchdogs/ui` — creates an issue watchdog.
#[route(POST "/issues/{id}/watchdogs/ui")]
pub async fn create_watchdog_ui(
    cx: &Cx,
    Form(form): Form<WatchdogUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(issue) = state.issues.get(&issue_id).await.ok().flatten() else {
        return Ok(see_other("/"));
    };
    if !form.watchdog_agent_id.trim().is_empty() {
        let _ = state
            .infrastructure
            .create_watchdog(staple_data::NewIssueWatchdog {
                company_id: issue.company_id.clone(),
                issue_id: issue_id.clone(),
                watchdog_agent_id: form.watchdog_agent_id,
                instructions: form.instructions,
                status: form.status.unwrap_or_else(|| "active".to_owned()),
                watchdog_issue_id: None,
                created_by_agent_id: None,
                created_by_user_id: Some(crate::auth::current_actor(cx)),
                created_by_run_id: None,
            })
            .await;
    }
    Ok(see_other(&format!("/issues/{issue_id}/watchdogs")))
}

/// `POST /users/ui` — creates an auth user.
#[route(POST "/users/ui")]
pub async fn create_user_ui(
    cx: &Cx,
    Form(form): Form<UserUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let state = app_context::<AppState>(cx);
    if !form.id.trim().is_empty() && !form.name.trim().is_empty() && !form.email.trim().is_empty() {
        let _ = state
            .infrastructure
            .create_user(staple_data::NewUser {
                id: form.id,
                name: form.name,
                email: form.email,
                email_verified: false,
                image: None,
            })
            .await;
    }
    Ok(see_other("/users"))
}

/// `POST /environments/ui` — creates an environment.
#[route(POST "/environments/ui")]
pub async fn create_environment_ui(
    cx: &Cx,
    Form(form): Form<EnvUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let state = app_context::<AppState>(cx);
    if !form.name.trim().is_empty() {
        let _ = state
            .environments
            .create(staple_data::NewEnvironment {
                name: form.name,
                description: form.description,
                driver: form.driver.unwrap_or_else(|| "local".to_owned()),
                config: None,
            })
            .await;
    }
    Ok(see_other("/environments"))
}
// --- Status card updates / smoke runs / feedback exports UI forms --------

/// Status card update form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusCardUpdateUiForm {
    /// Update kind.
    pub kind: String,
    /// Trigger.
    pub trigger: String,
    /// Status.
    pub status: String,
}

/// Smoke run form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmokeRunUiForm {
    /// Trigger.
    pub trigger: String,
    /// Status.
    pub status: Option<String>,
}

/// Feedback export form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackExportUiForm {
    /// Feedback vote id.
    pub feedback_vote_id: String,
    /// Issue id.
    pub issue_id: String,
    /// Author user id.
    pub author_user_id: String,
    /// Target type.
    pub target_type: String,
    /// Target id.
    pub target_id: String,
    /// Vote.
    pub vote: String,
    /// Target summary JSON.
    pub target_summary: Option<String>,
}

/// `POST /companies/{companyId}/status-cards/{id}/updates/ui` — creates a
/// status card update.
#[route(POST "/companies/{company_id}/status-cards/{id}/updates/ui")]
pub async fn create_status_card_update_ui(
    cx: &Cx,
    Form(form): Form<StatusCardUpdateUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let card_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let _ = state
        .scattered
        .create_status_card_update(staple_data::NewStatusCardUpdate {
            card_id: card_id.clone(),
            kind: form.kind,
            trigger: form.trigger,
            generation_issue_id: None,
            run_id: None,
            changes: serde_json::json!([]),
            input_tokens: 0,
            output_tokens: 0,
            cost_cents: 0,
            model: None,
            query_version: None,
            change_summary: None,
            status: form.status,
            error: None,
        })
        .await;
    Ok(see_other(&format!(
        "/companies/{company_id}/status-cards/{card_id}/updates"
    )))
}

/// `POST /companies/{companyId}/smoke-runs/ui` — creates a smoke run.
#[route(POST "/companies/{company_id}/smoke-runs/ui")]
pub async fn create_smoke_run_ui(
    cx: &Cx,
    Form(form): Form<SmokeRunUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if !form.trigger.trim().is_empty() {
        let _ = state
            .scattered
            .create_smoke_run(staple_data::NewSmokeRun {
                company_id: company_id.clone(),
                trigger: form.trigger,
                status: form.status.unwrap_or_else(|| "running".to_owned()),
                finished_at: None,
                summary: serde_json::json!({}),
            })
            .await;
    }
    Ok(see_other(&format!("/companies/{company_id}/smoke-runs")))
}

/// `POST /companies/{companyId}/feedback-exports/ui` — creates a feedback
/// export.
#[route(POST "/companies/{company_id}/feedback-exports/ui")]
pub async fn create_feedback_export_ui(
    cx: &Cx,
    Form(form): Form<FeedbackExportUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let target_summary = serde_json::from_str(form.target_summary.as_deref().unwrap_or("{}"))
        .unwrap_or_else(|_| serde_json::json!({}));
    if !form.feedback_vote_id.trim().is_empty()
        && !form.target_type.trim().is_empty()
        && !form.target_id.trim().is_empty()
        && !form.author_user_id.trim().is_empty()
    {
        let _ = state
            .scattered
            .create_feedback_export(staple_data::NewFeedbackExport {
                company_id: company_id.clone(),
                feedback_vote_id: form.feedback_vote_id,
                issue_id: form.issue_id,
                project_id: None,
                author_user_id: form.author_user_id,
                target_type: form.target_type,
                target_id: form.target_id,
                vote: form.vote,
                status: "local_only".to_owned(),
                destination: None,
                export_id: None,
                consent_version: None,
                payload_digest: None,
                payload_snapshot: None,
                target_summary,
                redaction_summary: None,
                attempt_count: 0,
                exported_at: None,
                failure_reason: None,
            })
            .await;
    }
    Ok(see_other(&format!(
        "/companies/{company_id}/feedback-exports"
    )))
}

// --- Toolchain UI forms (profiles / connections / gateways / catalog / invocations) ----

/// Tool profile create form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolProfileUiForm {
    /// Profile key.
    pub profile_key: String,
    /// Display name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Status.
    pub status: Option<String>,
    /// Default action for new tools.
    pub default_action: Option<String>,
    /// Metadata JSON.
    pub metadata: Option<String>,
}

/// Tool profile entry create form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolProfileEntryUiForm {
    /// Profile id.
    pub profile_id: String,
    /// Selector type (application/connection/catalog_entry/tool_name).
    pub selector_type: String,
    /// Effect (include/exclude).
    pub effect: Option<String>,
    /// Tool name pattern.
    pub tool_name: Option<String>,
    /// Risk level.
    pub risk_level: Option<String>,
    /// Conditions JSON.
    pub conditions: Option<String>,
}

/// Tool connection create form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolConnectionUiForm {
    /// Application id.
    pub application_id: String,
    /// Connection name.
    pub name: String,
    /// Connection uid.
    pub uid: String,
    /// Connection kind.
    pub connection_kind: Option<String>,
    /// Ownership.
    pub ownership: Option<String>,
    /// Transport.
    pub transport: String,
    /// Auth kind.
    pub auth_kind: Option<String>,
    /// Status.
    pub status: Option<String>,
    /// Enabled checkbox (`1`).
    #[serde(default)]
    pub enabled: Option<String>,
    /// Config JSON.
    pub config: Option<String>,
    /// Transport config JSON.
    pub transport_config: Option<String>,
}

/// Tool connection install create form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolInstallUiForm {
    /// Connection id.
    pub connection_id: String,
    /// Target type (agent/issue/project).
    pub target_type: String,
    /// Target id.
    pub target_id: String,
}

/// Tool gateway create form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolGatewayUiForm {
    /// Gateway name.
    pub name: String,
    /// Gateway slug.
    pub slug: String,
    /// Display slug.
    pub display_slug: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Status.
    pub status: Option<String>,
    /// Profile id.
    pub profile_id: String,
    /// Default profile mode.
    pub default_profile_mode: Option<String>,
    /// Context scope type.
    pub context_scope_type: Option<String>,
    /// Context scope id.
    pub context_scope_id: Option<String>,
}

/// Tool gateway session create form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolGatewaySessionUiForm {
    /// Gateway id.
    pub gateway_id: String,
    /// Agent id.
    pub agent_id: String,
    /// Run id.
    pub run_id: String,
    /// Token hash.
    pub token_hash: String,
    /// Expiry timestamp.
    pub expires_at: String,
}

/// Tool catalog entry create form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCatalogEntryUiForm {
    /// Connection id.
    pub connection_id: String,
    /// Entry kind.
    pub entry_kind: Option<String>,
    /// Entry name.
    pub name: String,
    /// Tool name.
    pub tool_name: String,
    /// Title.
    pub title: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Risk level.
    pub risk_level: Option<String>,
    /// Status.
    pub status: Option<String>,
    /// Version.
    pub version: Option<String>,
    /// Input schema JSON.
    pub input_schema: Option<String>,
    /// Read only checkbox (`1`).
    #[serde(default)]
    pub is_read_only: Option<String>,
    /// Write checkbox (`1`).
    #[serde(default)]
    pub is_write: Option<String>,
    /// Destructive checkbox (`1`).
    #[serde(default)]
    pub is_destructive: Option<String>,
}

/// Tool invocation create form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolInvocationUiForm {
    /// Tool name.
    pub tool_name: String,
    /// Actor type.
    pub actor_type: Option<String>,
    /// Status.
    pub status: Option<String>,
    /// Approval state.
    pub approval_state: Option<String>,
    /// Risk level.
    pub risk_level: Option<String>,
    /// Connection id.
    pub connection_id: Option<String>,
    /// Agent id.
    pub agent_id: Option<String>,
    /// Arguments summary JSON.
    pub arguments_summary: Option<String>,
}

/// `POST /companies/{companyId}/tools/profiles/ui` — creates a tool profile.
#[route(POST "/companies/{company_id}/tools/profiles/ui")]
pub async fn create_tool_profile_ui(
    cx: &Cx,
    Form(form): Form<ToolProfileUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if !form.profile_key.trim().is_empty() && !form.name.trim().is_empty() {
        let metadata = serde_json::from_str(form.metadata.as_deref().unwrap_or("{}"))
            .unwrap_or_else(|_| serde_json::json!({}));
        let _ = state
            .tool_catalog
            .create_profile(staple_data::NewToolProfile {
                company_id: company_id.clone(),
                profile_key: form.profile_key,
                name: form.name,
                description: form.description.filter(|value| !value.is_empty()),
                status: form.status.unwrap_or_else(|| "active".to_owned()),
                default_action: form.default_action.unwrap_or_else(|| "deny".to_owned()),
                new_tools_reviewed_at: None,
                metadata,
            })
            .await;
    }
    Ok(see_other(&format!(
        "/companies/{company_id}/tools/profiles"
    )))
}

/// `POST /companies/{companyId}/tools/profile-entries/ui` — creates a tool
/// profile entry.
#[route(POST "/companies/{company_id}/tools/profile-entries/ui")]
pub async fn create_tool_profile_entry_ui(
    cx: &Cx,
    Form(form): Form<ToolProfileEntryUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if !form.profile_id.trim().is_empty() && !form.selector_type.trim().is_empty() {
        let conditions = form
            .conditions
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .and_then(|value| serde_json::from_str(value).ok());
        let _ = state
            .tool_catalog
            .create_profile_entry(staple_data::NewToolProfileEntry {
                company_id: company_id.clone(),
                profile_id: form.profile_id,
                selector_type: form.selector_type,
                effect: form.effect.unwrap_or_else(|| "include".to_owned()),
                application_id: None,
                connection_id: None,
                catalog_entry_id: None,
                tool_name: form.tool_name.filter(|value| !value.is_empty()),
                risk_level: form.risk_level.filter(|value| !value.is_empty()),
                conditions,
            })
            .await;
    }
    Ok(see_other(&format!(
        "/companies/{company_id}/tools/profiles"
    )))
}

/// `POST /companies/{companyId}/tools/connections/ui` — creates a tool
/// connection.
#[route(POST "/companies/{company_id}/tools/connections/ui")]
pub async fn create_tool_connection_ui(
    cx: &Cx,
    Form(form): Form<ToolConnectionUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if !form.name.trim().is_empty()
        && !form.uid.trim().is_empty()
        && !form.application_id.trim().is_empty()
    {
        let config = serde_json::from_str(form.config.as_deref().unwrap_or("{}"))
            .unwrap_or_else(|_| serde_json::json!({}));
        let transport_config =
            serde_json::from_str(form.transport_config.as_deref().unwrap_or("{}"))
                .unwrap_or_else(|_| serde_json::json!({}));
        let _ = state
            .tool_connections
            .create_connection(staple_data::NewToolConnection {
                company_id: company_id.clone(),
                application_id: form.application_id,
                name: form.name,
                uid: form.uid,
                connection_kind: form
                    .connection_kind
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "managed".to_owned()),
                ownership: form
                    .ownership
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "customer".to_owned()),
                transport: form.transport,
                auth_kind: form
                    .auth_kind
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "none".to_owned()),
                status: form
                    .status
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "draft".to_owned()),
                enabled: form.enabled.is_some(),
                config,
                transport_config,
                credential_refs: serde_json::json!([]),
                credential_secret_refs: serde_json::json!([]),
                created_by_agent_id: None,
                created_by_user_id: Some(crate::auth::current_actor(cx)),
            })
            .await;
    }
    Ok(see_other(&format!(
        "/companies/{company_id}/tools/connections"
    )))
}

/// `POST /companies/{companyId}/tools/installs/ui` — creates a tool connection
/// install.
#[route(POST "/companies/{company_id}/tools/installs/ui")]
pub async fn create_tool_install_ui(
    cx: &Cx,
    Form(form): Form<ToolInstallUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if !form.connection_id.trim().is_empty()
        && !form.target_type.trim().is_empty()
        && !form.target_id.trim().is_empty()
    {
        let _ = state
            .tool_connections
            .create_install(staple_data::NewToolConnectionInstall {
                company_id: company_id.clone(),
                connection_id: form.connection_id,
                target_type: form.target_type,
                target_id: form.target_id,
                created_by_agent_id: None,
                created_by_user_id: Some(crate::auth::current_actor(cx)),
            })
            .await;
    }
    Ok(see_other(&format!(
        "/companies/{company_id}/tools/connections"
    )))
}

/// `POST /companies/{companyId}/tools/gateways/ui` — creates a tool gateway.
#[route(POST "/companies/{company_id}/tools/gateways/ui")]
pub async fn create_tool_gateway_ui(
    cx: &Cx,
    Form(form): Form<ToolGatewayUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if !form.name.trim().is_empty()
        && !form.slug.trim().is_empty()
        && !form.profile_id.trim().is_empty()
    {
        let _ = state
            .tool_gateway
            .create_gateway(staple_data::NewToolMcpGateway {
                company_id: company_id.clone(),
                name: form.name,
                slug: form.slug,
                display_slug: form
                    .display_slug
                    .filter(|value| !value.is_empty())
                    .unwrap_or_default(),
                description: form.description.filter(|value| !value.is_empty()),
                status: form
                    .status
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "active".to_owned()),
                profile_id: form.profile_id,
                default_profile_mode: form
                    .default_profile_mode
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "gateway_only".to_owned()),
                context_scope_type: form
                    .context_scope_type
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "none".to_owned()),
                context_scope_id: form.context_scope_id.filter(|value| !value.is_empty()),
                agent_id: None,
                project_id: None,
                issue_id: None,
                approval_issue_id: None,
                auth_config: serde_json::json!({}),
                header_policy: serde_json::json!({}),
                metadata_policy: serde_json::json!({}),
                on_demand_tools_config: serde_json::json!({}),
                metadata: serde_json::json!({}),
                created_by_agent_id: None,
                created_by_user_id: Some(crate::auth::current_actor(cx)),
            })
            .await;
    }
    Ok(see_other(&format!(
        "/companies/{company_id}/tools/gateways"
    )))
}

/// `POST /companies/{companyId}/tools/gateway-sessions/ui` — creates a tool
/// gateway session.
#[route(POST "/companies/{company_id}/tools/gateway-sessions/ui")]
pub async fn create_tool_gateway_session_ui(
    cx: &Cx,
    Form(form): Form<ToolGatewaySessionUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if !form.gateway_id.trim().is_empty()
        && !form.agent_id.trim().is_empty()
        && !form.run_id.trim().is_empty()
        && !form.token_hash.trim().is_empty()
        && !form.expires_at.trim().is_empty()
    {
        let _ = state
            .tool_gateway
            .create_gateway_session(staple_data::NewToolGatewaySession {
                company_id: company_id.clone(),
                agent_id: form.agent_id,
                run_id: form.run_id,
                issue_id: None,
                project_id: None,
                gateway_id: Some(form.gateway_id),
                gateway_token_id: None,
                gateway_public_id: None,
                client_subject_type: None,
                client_subject_id: None,
                client_name: None,
                mcp_session_id: None,
                correlation_id: None,
                token_hash: form.token_hash,
                expires_at: form.expires_at,
            })
            .await;
    }
    Ok(see_other(&format!(
        "/companies/{company_id}/tools/gateways"
    )))
}

/// `POST /companies/{companyId}/tools/catalog/ui` — creates a tool catalog
/// entry.
#[route(POST "/companies/{company_id}/tools/catalog/ui")]
pub async fn create_tool_catalog_entry_ui(
    cx: &Cx,
    Form(form): Form<ToolCatalogEntryUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if !form.connection_id.trim().is_empty()
        && !form.name.trim().is_empty()
        && !form.tool_name.trim().is_empty()
    {
        let input_schema = serde_json::from_str(form.input_schema.as_deref().unwrap_or("{}"))
            .unwrap_or_else(|_| serde_json::json!({}));
        let is_read_only = form.is_read_only.is_some();
        let is_write = form.is_write.is_some();
        let is_destructive = form.is_destructive.is_some();
        let _ = state
            .tool_catalog
            .create_catalog_entry(staple_data::NewToolCatalogEntry {
                company_id: company_id.clone(),
                application_id: None,
                connection_id: form.connection_id,
                entry_kind: form
                    .entry_kind
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "tool".to_owned()),
                name: form.name,
                tool_name: form.tool_name,
                title: form.title.filter(|value| !value.is_empty()),
                description: form.description.filter(|value| !value.is_empty()),
                input_schema,
                output_schema: None,
                annotations: serde_json::json!({}),
                risk_level: form
                    .risk_level
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "medium".to_owned()),
                is_read_only,
                is_write,
                is_destructive,
                status: form
                    .status
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "active".to_owned()),
                version: form.version.filter(|value| !value.is_empty()),
                version_hash: String::new(),
                schema_hash: None,
                reviewed_by_agent_id: None,
                reviewed_by_user_id: None,
            })
            .await;
    }
    Ok(see_other(&format!("/companies/{company_id}/tools/catalog")))
}

/// `POST /companies/{companyId}/tools/invocations/ui` — creates a tool
/// invocation.
#[route(POST "/companies/{company_id}/tools/invocations/ui")]
pub async fn create_tool_invocation_ui(
    cx: &Cx,
    Form(form): Form<ToolInvocationUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let tool_name = form.tool_name.trim().to_owned();
    if !tool_name.is_empty() {
        let arguments_summary = form
            .arguments_summary
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .and_then(|value| serde_json::from_str(value).ok());
        let _ = state
            .tool_gateway
            .create_invocation(staple_data::NewToolInvocation {
                company_id: company_id.clone(),
                idempotency_key: None,
                actor_type: form
                    .actor_type
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "system".to_owned()),
                actor_id: None,
                agent_id: form.agent_id.filter(|value| !value.is_empty()),
                issue_id: None,
                run_id: None,
                gateway_id: None,
                gateway_token_id: None,
                gateway_public_id: None,
                client_subject_type: None,
                client_subject_id: None,
                client_name: None,
                mcp_session_id: None,
                correlation_id: None,
                application_id: None,
                connection_id: form.connection_id.filter(|value| !value.is_empty()),
                catalog_entry_id: None,
                catalog_version_hash: None,
                catalog_schema_hash: None,
                provider_type: None,
                application_key: None,
                upstream_tool_name: None,
                risk_level: form.risk_level.filter(|value| !value.is_empty()),
                tool_name,
                arguments_hash: None,
                arguments_summary,
                policy_decision: None,
                matched_policy_ids: serde_json::json!([]),
                policy_explanation: None,
                credential_scope_summary: None,
                header_policy_summary: None,
                approval_state: form
                    .approval_state
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "not_required".to_owned()),
                status: form
                    .status
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "pending".to_owned()),
                upstream_request_id: None,
                result_hash: None,
                result_summary: None,
                result_size_bytes: None,
                result_artifact_id: None,
                error_code: None,
                error_message: None,
                started_at: None,
                completed_at: None,
            })
            .await;
    }
    Ok(see_other(&format!(
        "/companies/{company_id}/tools/invocations"
    )))
}

// --- Board claim & chat UI forms -----------------------------------------

/// Issue claim form (assign to an agent).
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimUiForm {
    /// Agent id to assign.
    pub agent_id: String,
}

/// `POST /issues/{id}/claim/ui` — claims an issue by assigning it to an
/// agent, then redirects back to the issue.
#[route(POST "/issues/{id}/claim/ui")]
pub async fn claim_issue_ui(
    cx: &Cx,
    Form(form): Form<ClaimUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(issue) = state.issues.get(&issue_id).await.ok().flatten() else {
        return Ok(see_other("/"));
    };
    let mut code = "invalid";
    if !form.agent_id.trim().is_empty() {
        let agent_id = form.agent_id.clone();
        code = if state
            .issues
            .update(
                &issue_id,
                staple_data::IssuePatch {
                    title: None,
                    description: None,
                    status: None,
                    priority: None,
                    assignee_agent_id: Some(Some(agent_id)),
                    billing_code: None,
                    execution_workspace_settings: None,
                },
            )
            .await
            .is_ok()
        {
            "claimed"
        } else {
            "error"
        };
        let _ = log_activity(
            &state.activity,
            &issue.company_id,
            "issue.claimed",
            "issue",
            &issue_id,
            Some(serde_json::json!({ "assigneeAgentId": form.agent_id })),
        )
        .await;
    }
    Ok(see_other_flash(&format!("/issues/{issue_id}"), code))
}

/// `GET /static/board_chat.js` — streaming chat behavior for the board chat
/// page.
#[route(GET "/static/board_chat.js")]
pub async fn board_chat_js(_cx: &Cx) -> Result<topcoat::router::Response, ApiError> {
    let body = topcoat::router::Body::from(include_str!("board_chat.js"));
    let response = topcoat::router::Response::builder()
        .header("Content-Type", "text/javascript; charset=utf-8")
        .body(body)
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(response)
}

// --- CLI auth UI forms ---------------------------------------------------

/// Board API key create form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardKeyUiForm {
    /// Owning user id.
    pub user_id: String,
    /// Key name.
    pub name: String,
}

/// CLI auth challenge create form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliChallengeUiForm {
    /// Command text.
    pub command: String,
    /// Pending key name.
    pub pending_key_name: String,
    /// Requested access.
    pub requested_access: Option<String>,
    /// Requested company id.
    pub requested_company_id: Option<String>,
}

/// `POST /cli-auth/keys/ui` — creates a board API key.
#[route(POST "/cli-auth/keys/ui")]
pub async fn create_board_key_ui(
    cx: &Cx,
    Form(form): Form<BoardKeyUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let state = app_context::<AppState>(cx);
    if !form.user_id.trim().is_empty() && !form.name.trim().is_empty() {
        let _ = state
            .board_keys
            .create_key(staple_data::NewBoardApiKey {
                user_id: form.user_id,
                name: form.name,
                expires_at: None,
            })
            .await;
    }
    Ok(see_other("/cli-auth"))
}

/// `POST /cli-auth/challenges/ui` — creates a CLI auth challenge.
#[route(POST "/cli-auth/challenges/ui")]
pub async fn create_cli_challenge_ui(
    cx: &Cx,
    Form(form): Form<CliChallengeUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let state = app_context::<AppState>(cx);
    let expires_at = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(1))
        .map(|time| time.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
        .unwrap_or_else(|| "2026-12-31T00:00:00.000Z".to_owned());
    if !form.command.trim().is_empty() && !form.pending_key_name.trim().is_empty() {
        let _ = state
            .board_keys
            .create_challenge(staple_data::NewCliAuthChallenge {
                command: form.command,
                client_name: Some("board-ui".to_owned()),
                requested_access: form
                    .requested_access
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "board".to_owned()),
                requested_company_id: form.requested_company_id.filter(|value| !value.is_empty()),
                pending_key_name: form.pending_key_name,
                expires_at,
            })
            .await;
    }
    Ok(see_other("/cli-auth"))
}

/// `POST /cli-auth/challenges/{id}/approve/ui` — approves a challenge.
#[route(POST "/cli-auth/challenges/{id}/approve/ui")]
pub async fn approve_cli_challenge_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let challenge_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let _ = state
        .board_keys
        .approve_challenge(&challenge_id, Some(crate::auth::current_actor(cx)))
        .await;
    Ok(see_other("/cli-auth"))
}

/// `POST /cli-auth/challenges/{id}/cancel/ui` — cancels a challenge.
#[route(POST "/cli-auth/challenges/{id}/cancel/ui")]
pub async fn cancel_cli_challenge_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let challenge_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let _ = state.board_keys.cancel_challenge(&challenge_id).await;
    Ok(see_other("/cli-auth"))
}

// --- Company portability UI forms ----------------------------------------

/// Import form.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportUiForm {
    /// Manifest JSON.
    pub manifest: String,
    /// Strategy (`skip` | `overwrite`).
    pub strategy: Option<String>,
}

/// `POST /companies/{companyId}/import/ui` — imports a manifest and redirects
/// with a result summary.
#[route(POST "/companies/{company_id}/import/ui")]
pub async fn import_company_ui(
    cx: &Cx,
    Form(form): Form<ImportUiForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let strategy = match form.strategy.as_deref() {
        Some("overwrite") => staple_data::ImportStrategy::Overwrite,
        _ => staple_data::ImportStrategy::Skip,
    };
    let manifest: staple_data::CompanyManifest = match serde_json::from_str(&form.manifest) {
        Ok(manifest) => manifest,
        Err(error) => {
            return Ok(see_other(&format!(
                "/companies/{company_id}/export-import?result=invalid+manifest:+{error}"
            )));
        }
    };
    let message = match state
        .portability
        .import_company(&company_id, manifest, strategy)
        .await
    {
        Ok(summary) => format!(
            "imported+{},+skipped+{},+failed+{}",
            summary.imported, summary.skipped, summary.failed
        ),
        Err(error) => format!("error:+{}", error.to_string().replace(' ', "+")),
    };
    Ok(see_other(&format!(
        "/companies/{company_id}/export-import?result={message}"
    )))
}

/// Form for a review-queue decision.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewDecisionForm {
    /// Case id.
    pub case_id: String,
    /// Decision (`accept` | `dismiss` | `approve` | `request_changes` | `reject`).
    pub decision: String,
    /// Expected case version (optimistic lock).
    #[serde(default)]
    pub expected_version: Option<i64>,
    /// Reason (required for request_changes/reject).
    #[serde(default)]
    pub reason: Option<String>,
    /// Pending suggestion id, when deciding a suggestion.
    #[serde(default)]
    pub suggestion_id: Option<String>,
}

/// `POST /companies/{company_id}/review-queue/decide/ui` — applies a review
/// decision from the review-queue page and redirects back.
#[route(POST "/companies/{company_id}/review-queue/decide/ui")]
pub async fn review_queue_decide_ui(
    cx: &Cx,
    Form(form): Form<ReviewDecisionForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let result = if let Some(suggestion_id) = form.suggestion_id.filter(|value| !value.is_empty()) {
        state
            .pipelines
            .resolve_suggestion(
                &company_id,
                &form.case_id,
                &suggestion_id,
                &form.decision,
                form.expected_version,
                form.reason.as_deref(),
                "user",
                Some("board"),
                None,
            )
            .await
    } else {
        state
            .pipelines
            .review_case(
                &company_id,
                &form.case_id,
                &form.decision,
                form.expected_version.unwrap_or(0),
                form.reason.as_deref(),
                "user",
                Some("board"),
                None,
            )
            .await
    };
    if let Ok(case) = result {
        let _ = log_activity(
            &state.activity,
            &company_id,
            "pipeline.review_decided",
            "pipeline_case",
            &case.id,
            Some(serde_json::json!({ "decision": form.decision })),
        )
        .await;
    }
    Ok(see_other(&format!("/companies/{company_id}/review-queue")))
}

/// `GET /static/board_zip.js` — zip portability behavior for the
/// export/import page.
#[route(GET "/static/board_zip.js")]
pub async fn board_zip_js(_cx: &Cx) -> Result<topcoat::router::Response, ApiError> {
    let body = topcoat::router::Body::from(include_str!("board_zip.js"));
    let response = topcoat::router::Response::builder()
        .header("Content-Type", "text/javascript; charset=utf-8")
        .body(body)
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(response)
}

/// `{user_id}` path parameter for UI routes.
#[path_param(error = bad_request("Invalid user id"))]
pub(crate) struct UserId(String);

/// Form for setting a user's company access.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyAccessForm {
    /// Selected company ids.
    #[serde(default)]
    pub company_ids: Vec<String>,
}

/// `POST /instance/users/{user_id}/company-access/ui` — sets a user's
/// companies and redirects back to instance settings.
#[route(POST "/instance/users/{user_id}/company-access/ui")]
pub async fn user_company_access_ui(
    cx: &Cx,
    Form(form): Form<CompanyAccessForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let user_id = path_param::<UserId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let _ = state
        .memberships
        .set_user_company_access(&user_id, &form.company_ids)
        .await;
    Ok(see_other("/instance/settings"))
}

/// Form for the onboarding wizard (form fields are snake_case).
#[derive(Debug, Deserialize)]
pub struct OnboardingForm {
    /// Current step (1..4); the handler advances to the next.
    pub step: usize,
    /// Company name.
    #[serde(default)]
    pub company_name: String,
    /// Mission text (typed).
    #[serde(default)]
    pub mission: String,
    /// Mission preset button value.
    #[serde(default)]
    pub mission_preset: Option<String>,
    /// Lead agent name.
    #[serde(default)]
    pub lead_name: String,
    /// Selected adapter type (connection-model step).
    #[serde(default)]
    pub adapter_type: String,
}

/// `POST /onboarding/ui` — advances the onboarding wizard; the final step
/// creates the company, default team, first task, and a heartbeat run.
#[route(POST "/onboarding/ui")]
pub async fn onboarding_ui(
    cx: &Cx,
    Form(form): Form<OnboardingForm>,
) -> Result<topcoat::router::Response, ApiError> {
    let lang = lang_from_request(cx);
    let mission = form.mission_preset.unwrap_or(form.mission);
    if form.step >= 5 {
        let state = app_context::<AppState>(cx);
        let lead_name = if form.lead_name.trim().is_empty() {
            "Chief of Staff".to_owned()
        } else {
            form.lead_name.clone()
        };
        let adapter_type = if form.adapter_type.trim().is_empty() {
            "cli_local".to_owned()
        } else {
            form.adapter_type.clone()
        };
        let company = state
            .companies
            .create(staple_data::NewCompany {
                name: form.company_name.clone(),
                description: Some(mission.clone()),
                budget_monthly_cents: 100_0000,
                attachment_max_bytes: 10_485_760,
            })
            .await
            .map_err(|error| ApiError::internal(error.to_string()))?;

        // Lead agent.
        let lead = state
            .agents
            .create(staple_data::NewAgent {
                company_id: company.id.clone(),
                name: lead_name.clone(),
                role: "worker".to_owned(),
                title: Some("Chief of Staff".to_owned()),
                icon: None,
                reports_to: None,
                adapter_type: adapter_type.clone(),
                budget_monthly_cents: 0,
            })
            .await
            .map_err(|error| ApiError::internal(error.to_string()))?;
        let _ = crate::instructions::materialize_default_instructions(
            state,
            &company.id,
            &lead.id,
            "default",
        )
        .await;

        // Default team (when the teams-catalog package is present).
        if let Some(team) = team_catalog::detail(team_catalog::default_catalog_ref()) {
            let mut agent_ids: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            for slug in &team.agent_slugs {
                if let Ok(agent) = state
                    .agents
                    .create(staple_data::NewAgent {
                        company_id: company.id.clone(),
                        name: slug.clone(),
                        role: "worker".to_owned(),
                        title: None,
                        icon: None,
                        reports_to: Some(lead.id.clone()),
                        adapter_type: "cli".to_owned(),
                        budget_monthly_cents: 0,
                    })
                    .await
                {
                    agent_ids.insert(slug.clone(), agent.id.clone());
                    let _ = crate::instructions::materialize_default_instructions(
                        state,
                        &company.id,
                        &agent.id,
                        slug,
                    )
                    .await;
                }
            }
            let mut project_ids: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            for slug in &team.project_slugs {
                if let Ok(project) = state
                    .projects
                    .create(staple_data::NewProject {
                        company_id: company.id.clone(),
                        goal_id: None,
                        name: slug.clone(),
                        description: None,
                        status: "backlog".to_owned(),
                        lead_agent_id: None,
                        target_date: None,
                        env: None,
                        execution_workspace_policy: None,
                    })
                    .await
                {
                    project_ids.insert(slug.clone(), project.id);
                }
            }
            for skill in team_catalog::team_skills(&team_catalog::catalog_root(), &team.id) {
                let _ = state
                    .skills
                    .create(staple_data::NewSkill {
                        company_id: company.id.clone(),
                        name: skill.name,
                        description: skill.description,
                        restriction_policy: staple_data::SkillRestrictionPolicy::default(),
                    })
                    .await;
            }
            for task in team_catalog::team_tasks(&team_catalog::catalog_root(), &team.id)
                .iter()
                .filter(|task| task.recurring)
            {
                if let Ok(routine) = state
                    .routines
                    .create(staple_data::NewRoutine {
                        company_id: company.id.clone(),
                        project_id: task
                            .project
                            .as_deref()
                            .and_then(|slug| project_ids.get(slug))
                            .cloned(),
                        goal_id: None,
                        parent_issue_id: None,
                        title: task.title.clone(),
                        description: Some(task.description.clone()),
                        assignee_agent_id: task
                            .assignee
                            .as_deref()
                            .and_then(|slug| agent_ids.get(slug))
                            .cloned(),
                        priority: "medium".to_owned(),
                        variables: None,
                    })
                    .await
                {
                    let _ = state
                        .routines
                        .create_trigger(staple_data::NewTrigger {
                            company_id: company.id.clone(),
                            routine_id: routine.id,
                            schedule_kind: "cron".to_owned(),
                            schedule_expr: Some(
                                task.schedule
                                    .clone()
                                    .unwrap_or_else(|| "0 9 * * *".to_owned()),
                            ),
                        })
                        .await;
                }
            }
        }

        // First task.
        let task = state
            .issues
            .create(staple_data::NewIssue {
                company_id: company.id.clone(),
                project_id: None,
                goal_id: None,
                parent_id: None,
                title: "Hire your first engineer and draft a hiring plan".to_owned(),
                description: Some("Onboarding starter task.".to_owned()),
                status: Some("todo".to_owned()),
                priority: Some("high".to_owned()),
                assignee_agent_id: Some(lead.id.clone()),
                assignee_user_id: None,
                created_by_user_id: None,
                work_mode: None,
                billing_code: None,
                execution_workspace_settings: None,
            })
            .await
            .map_err(|error| ApiError::internal(error.to_string()))?;

        // Heartbeat run for the lead agent on the first task.
        let _ = state
            .heartbeat
            .start(staple_data::NewHeartbeatRun {
                company_id: company.id.clone(),
                agent_id: lead.id.clone(),
                invocation_source: "manual".to_owned(),
                issue_id: Some(task.id.clone()),
                context_snapshot: None,
                trigger_detail: Some("onboarding".to_owned()),
            })
            .await;

        let _ = log_activity(
            &state.activity,
            &company.id,
            "onboarding.completed",
            "company",
            &company.id,
            Some(serde_json::json!({ "leadAgent": lead.id, "task": task.id })),
        )
        .await;

        let location = with_lang(&format!("/companies/{}/dashboard", company.id), lang);
        let response = topcoat::router::Response::builder()
            .status(303)
            .header("Location", location)
            .body(topcoat::router::Body::empty())
            .map_err(|error| ApiError::internal(error.to_string()))?;
        return Ok(response);
    }

    let next_step = form.step.saturating_add(1).clamp(1, 5);
    let state = app_context::<AppState>(cx);
    let adapter_names = state.adapters.names();
    let view = onboarding_view(
        cx,
        lang,
        next_step,
        form.company_name,
        mission,
        form.lead_name,
        form.adapter_type,
        String::new(),
        adapter_names,
    )
    .await
    .map_err(|error| ApiError::internal(error.to_string()))?;
    view.into_response(cx)
        .map_err(|error| ApiError::internal(error.to_string()))
}

/// `POST /onboarding/adapter-test/ui` — checks the selected adapter and
/// re-renders the connection-model step with the result.
#[route(POST "/onboarding/adapter-test/ui")]
pub async fn onboarding_adapter_test_ui(
    cx: &Cx,
    Form(form): Form<OnboardingForm>,
) -> Result<topcoat::router::Response, ApiError> {
    let lang = lang_from_request(cx);
    let state = app_context::<AppState>(cx);
    let adapter_names = state.adapters.names();
    let test_result = match state.adapters.get(&form.adapter_type) {
        Some(adapter) => match adapter.probe().await {
            Ok(result) => {
                if result.available {
                    format!("{}: {}", t(lang, "onboarding.adapterOk"), result.detail)
                } else {
                    format!("{}: {}", t(lang, "onboarding.adapterFail"), result.detail)
                }
            }
            Err(error) => format!("{}: {error}", t(lang, "onboarding.adapterFail")),
        },
        None => t(lang, "onboarding.adapterFail"),
    };
    let view = onboarding_view(
        cx,
        lang,
        4,
        form.company_name,
        form.mission_preset.unwrap_or(form.mission),
        form.lead_name,
        form.adapter_type,
        test_result,
        adapter_names,
    )
    .await
    .map_err(|error| ApiError::internal(error.to_string()))?;
    view.into_response(cx)
        .map_err(|error| ApiError::internal(error.to_string()))
}

/// `GET /static/command_palette.js` — global Cmd/Ctrl+K command palette.
#[route(GET "/static/command_palette.js")]
pub async fn command_palette_js(_cx: &Cx) -> Result<topcoat::router::Response, ApiError> {
    let body = topcoat::router::Body::from(include_str!("command_palette.js"));
    let response = topcoat::router::Response::builder()
        .header("Content-Type", "text/javascript; charset=utf-8")
        .body(body)
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(response)
}

/// `GET /static/ui_feedback.js` — global mutating-form loading + flash toast
/// behavior (issue #231).
#[route(GET "/static/ui_feedback.js")]
pub async fn ui_feedback_js(_cx: &Cx) -> Result<topcoat::router::Response, ApiError> {
    let body = topcoat::router::Body::from(include_str!("ui_feedback.js"));
    let response = topcoat::router::Response::builder()
        .header("Content-Type", "text/javascript; charset=utf-8")
        .body(body)
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::flash_uri;

    #[test]
    fn flash_uri_appends_new_query_without_existing_query() {
        assert_eq!(
            flash_uri("/issues/issue-1", "created"),
            "/issues/issue-1?flash=created"
        );
    }

    #[test]
    fn flash_uri_uses_ampersand_and_preserves_existing_query() {
        assert_eq!(
            flash_uri("/companies/company-1/settings?lang=en", "saved"),
            "/companies/company-1/settings?lang=en&flash=saved"
        );
        assert_eq!(
            flash_uri(
                "/companies/company-1/export-import?result=ok&lang=en",
                "saved"
            ),
            "/companies/company-1/export-import?result=ok&lang=en&flash=saved"
        );
    }
}
