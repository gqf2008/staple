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

// ---------------------------------------------------------------------------
// Agent UI actions
// ---------------------------------------------------------------------------

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
    let _ = state
        .agents
        .update_status(
            &company_id,
            &agent_id,
            &form.status,
            Some(Some(form.pause_reason.unwrap_or_default())),
        )
        .await;
    Ok(see_other(&format!("/agents/{agent_id}")))
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
    let _ = state
        .agents
        .set_budget(&company_id, &agent_id, form.budget_monthly_cents)
        .await;
    Ok(see_other(&format!("/agents/{agent_id}")))
}

/// `POST /issues/{id}/archive/ui` and `/unarchive/ui` — inbox controls.
#[route(POST "/issues/{id}/archive/ui")]
pub async fn archive_issue_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if let Some(issue) = state.issues.get(&issue_id).await.ok().flatten() {
        let _ = state.issues.set_hidden(&issue_id, true).await;
        return Ok(see_other(&format!("/companies/{}/inbox", issue.company_id)));
    }
    Ok(see_other("/"))
}

/// `POST /issues/{id}/unarchive/ui`.
#[route(POST "/issues/{id}/unarchive/ui")]
pub async fn unarchive_issue_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if let Some(issue) = state.issues.get(&issue_id).await.ok().flatten() {
        let _ = state.issues.set_hidden(&issue_id, false).await;
        return Ok(see_other(&format!("/companies/{}/inbox", issue.company_id)));
    }
    Ok(see_other("/"))
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
    if !title.is_empty() {
        let _ = state
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
            .await;
    }
    Ok(see_other(&format!("/companies/{company_id}/routines")))
}

/// `POST /routines/{id}/trigger/ui`.
#[route(POST "/routines/{id}/trigger/ui")]
pub async fn routine_trigger_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let routine_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if let Ok(Some(routine)) = state.routines.get(&routine_id).await {
        let _ = state
            .routines
            .trigger(&routine.company_id, &routine_id)
            .await;
        return Ok(see_other(&format!(
            "/companies/{}/routines",
            routine.company_id
        )));
    }
    Ok(see_other("/"))
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
    Ok(see_other(&format!("/companies/{company_id}/secrets")))
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
    Ok(see_other(&format!("/companies/{company_id}/skills")))
}

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

// ---------------------------------------------------------------------------
// Form structs + path params
// ---------------------------------------------------------------------------

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
