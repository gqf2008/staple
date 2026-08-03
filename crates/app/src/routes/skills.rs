//! Company skills routes with the policy evaluator.

use serde::Deserialize;
use serde_json::json;
use staple_data::{NewSkill, SkillRestrictionPolicy};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    error::ApiError,
    routes::{CompanyId, is_uuid},
    state::AppState,
};

/// Body for `POST /api/companies/{companyId}/skills`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSkillRequest {
    /// Skill name.
    pub name: String,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
    /// Restriction policy.
    #[serde(default)]
    pub restriction_policy: Option<SkillRestrictionPolicy>,
}

/// Body for `POST /api/companies/{companyId}/skills/evaluate`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateSkillRequest {
    /// Agent id.
    pub agent_id: String,
    /// Skill name.
    pub skill: String,
}

/// `POST /api/companies/{companyId}/skills` — creates a skill (board only).
#[route(POST "/api/companies/{company_id}/skills")]
pub async fn create_skill(
    cx: &Cx,
    Json(body): Json<CreateSkillRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    crate::auth::require_board(cx)?;
    if body.name.trim().is_empty() {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["name"], "message": "String must contain at least 1 character(s)" }]),
        ));
    }
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let skill = state
        .skills
        .create(NewSkill {
            company_id,
            name: body.name.trim().to_owned(),
            description: body.description,
            restriction_policy: body.restriction_policy.unwrap_or_default(),
        })
        .await
        .map_err(skill_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&skill).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/skills` — lists skills.
#[route(GET "/api/companies/{company_id}/skills")]
pub async fn list_skills(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let skills = state
        .skills
        .list(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&skills).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/skills/evaluate` — evaluates the policy
/// for an agent; unknown skills/agents are denied.
#[route(POST "/api/companies/{company_id}/skills/evaluate")]
pub async fn evaluate_skill_route(
    cx: &Cx,
    Json(body): Json<EvaluateSkillRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !is_uuid(&body.agent_id) {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["agentId"], "message": "Invalid uuid" }]),
        ));
    }
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    match state
        .skills
        .evaluate(&company_id, &body.agent_id, &body.skill)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(evaluation) => Ok(Json(serde_json::to_value(&evaluation).unwrap_or_default())),
        None => Ok(Json(
            json!({ "allowed": false, "reason": "skill or agent not found" }),
        )),
    }
}

fn skill_error_to_api(error: staple_data::SkillError) -> ApiError {
    use staple_data::SkillError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::AlreadyExists => ApiError::conflict("Skill already exists"),
        other => ApiError::internal(other.to_string()),
    }
}
