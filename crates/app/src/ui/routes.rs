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
