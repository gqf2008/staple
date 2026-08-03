//! Approval routes (§8.3 state machine) and the budget-override approval gate.

use serde::Deserialize;
use serde_json::json;
use staple_data::{ApprovalDecision, NewApproval};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    audit::log_activity,
    dto::ApprovalDto,
    error::ApiError,
    routes::{CompanyId, Id},
    state::AppState,
};

/// Body for `POST /api/companies/{companyId}/approvals`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApprovalRequest {
    /// `hire_agent | approve_ceo_strategy | budget_override_required |
    /// request_board_approval`.
    pub r#type: String,
    /// Requester user id.
    #[serde(default)]
    pub requested_by_user_id: Option<String>,
    /// Payload (arbitrary JSON).
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

/// Body for `POST /api/approvals/{id}/decide`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecideApprovalRequest {
    /// `approved | rejected`.
    pub decision: String,
    /// Decision note.
    #[serde(default)]
    pub decision_note: Option<String>,
    /// Deciding user id.
    #[serde(default)]
    pub decided_by_user_id: Option<String>,
}

fn validate_create(body: &CreateApprovalRequest) -> Result<(), ApiError> {
    if !matches!(
        body.r#type.as_str(),
        "hire_agent"
            | "approve_ceo_strategy"
            | "budget_override_required"
            | "request_board_approval"
    ) {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{
                "path": ["type"],
                "message": "Invalid enum value. Expected 'hire_agent' | 'approve_ceo_strategy' | 'budget_override_required' | 'request_board_approval'"
            }]),
        ));
    }
    Ok(())
}

/// `POST /api/companies/{companyId}/approvals` — creates a pending approval.
#[route(POST "/api/companies/{company_id}/approvals")]
pub async fn create_approval(
    cx: &Cx,
    Json(body): Json<CreateApprovalRequest>,
) -> Result<(StatusCode, Json<ApprovalDto>), ApiError> {
    validate_create(&body)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let approval = state
        .approvals
        .create(NewApproval {
            company_id: company_id.clone(),
            r#type: body.r#type,
            requested_by_agent_id: None,
            requested_by_user_id: body.requested_by_user_id,
            payload: body
                .payload
                .unwrap_or(serde_json::Value::Object(Default::default()))
                .to_string(),
        })
        .await
        .map_err(approval_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "approval.created",
        "approval",
        &approval.id,
        Some(json!({ "type": approval.r#type })),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(approval.into())))
}

/// `GET /api/companies/{companyId}/approvals` — lists approvals.
#[route(GET "/api/companies/{company_id}/approvals")]
pub async fn list_approvals(cx: &Cx) -> Result<Json<Vec<ApprovalDto>>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let approvals = state
        .approvals
        .list(&company_id, None)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(approvals.into_iter().map(ApprovalDto::from).collect()))
}

/// `GET /api/approvals/{id}` — fetches one approval.
#[route(GET "/api/approvals/{id}")]
pub async fn get_approval(cx: &Cx) -> Result<Json<ApprovalDto>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .approvals
        .get(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(approval) => Ok(Json(approval.into())),
        None => Err(ApiError::not_found("Approval not found")),
    }
}

/// `POST /api/approvals/{id}/decide` — approves or rejects; approving a
/// `budget_override_required` approval applies the payload's
/// `budgetMonthlyCents` to the company (approval gate).
#[route(POST "/api/approvals/{id}/decide")]
pub async fn decide_approval(
    cx: &Cx,
    Json(body): Json<DecideApprovalRequest>,
) -> Result<Json<ApprovalDto>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let approval = state
        .approvals
        .decide(
            &id,
            ApprovalDecision {
                decision: body.decision,
                decision_note: body.decision_note,
                decided_by_user_id: body.decided_by_user_id,
            },
        )
        .await
        .map_err(approval_error_to_api)?;
    let Some(approval) = approval else {
        return Err(ApiError::not_found("Approval not found"));
    };

    // Approval gate: apply approved budget overrides.
    if approval.status == "approved" && approval.r#type == "budget_override_required" {
        let payload: serde_json::Value =
            serde_json::from_str(&approval.payload).unwrap_or(serde_json::Value::Null);
        if let Some(budget) = payload.get("budgetMonthlyCents").and_then(|v| v.as_i64()) {
            state
                .costs
                .set_company_budget(&approval.company_id, budget)
                .await
                .map_err(|error| ApiError::internal(error.to_string()))?;
        }
    }

    log_activity(
        &state.activity,
        &approval.company_id,
        "approval.decided",
        "approval",
        &approval.id,
        Some(json!({ "status": approval.status, "type": approval.r#type })),
    )
    .await?;
    Ok(Json(approval.into()))
}

/// `POST /api/approvals/{id}/cancel` — cancels a pending approval.
#[route(POST "/api/approvals/{id}/cancel")]
pub async fn cancel_approval(cx: &Cx) -> Result<Json<ApprovalDto>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let approval = state
        .approvals
        .cancel(&id)
        .await
        .map_err(approval_error_to_api)?;
    let Some(approval) = approval else {
        return Err(ApiError::not_found("Approval not found"));
    };
    log_activity(
        &state.activity,
        &approval.company_id,
        "approval.cancelled",
        "approval",
        &approval.id,
        None,
    )
    .await?;
    Ok(Json(approval.into()))
}

fn approval_error_to_api(error: staple_data::ApprovalError) -> ApiError {
    use staple_data::ApprovalError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::NotPending => ApiError::conflict("Approval is not in a pending state"),
        E::InvalidDecision => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["decision"], "message": "Decision must be 'approved' or 'rejected'" }]),
        ),
        other => ApiError::internal(other.to_string()),
    }
}
