//! Skill catalog routes: versions, policies, comments, stars, test inputs,
//! test run templates, and test runs (upstream company_skills.ts +
//! company_skill_policies.ts).

use serde::Deserialize;
use serde_json::json;
use staple_data::{
    NewSkillComment, NewSkillStar, NewSkillTestInput, NewSkillTestRun, NewSkillTestRunTemplate,
    NewSkillVersion, SetSkillPolicy,
};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    error::ApiError,
    routes::{CompanyId, SkillId},
    state::AppState,
};

/// Body for `POST .../skills/{skillId}/versions`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishVersionRequest {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub release_id: Option<String>,
    #[serde(default)]
    pub release_name: Option<String>,
    #[serde(default)]
    pub released_at: Option<String>,
    #[serde(default = "default_array")]
    pub file_inventory: serde_json::Value,
    #[serde(default)]
    pub author_agent_id: Option<String>,
    #[serde(default)]
    pub author_user_id: Option<String>,
}

/// Body for `PUT .../skill-policies`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPolicyRequest {
    #[serde(default = "default_one")]
    pub schema_version: i64,
    pub default_effect: String,
    #[serde(default = "default_array")]
    pub rules: serde_json::Value,
}

/// Body for `POST .../skills/{skillId}/comments`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCommentRequest {
    #[serde(default)]
    pub parent_comment_id: Option<String>,
    #[serde(default)]
    pub author_agent_id: Option<String>,
    #[serde(default)]
    pub author_user_id: Option<String>,
    pub body: String,
}

/// Body for `POST .../skills/{skillId}/stars`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStarRequest {
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
}

/// Body for `POST .../skills/{skillId}/test-inputs`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTestInputRequest {
    pub name: String,
    pub content: String,
    #[serde(default)]
    pub created_by: Option<String>,
}

/// Body for `POST .../skill-test-run-templates`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTestRunTemplateRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub body: String,
    #[serde(default)]
    pub created_by_agent_id: Option<String>,
    #[serde(default)]
    pub created_by_user_id: Option<String>,
    #[serde(default)]
    pub updated_by_agent_id: Option<String>,
    #[serde(default)]
    pub updated_by_user_id: Option<String>,
}

/// Body for `POST .../skills/{skillId}/test-runs`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTestRunRequest {
    #[serde(default)]
    pub input_id: Option<String>,
    pub input_snapshot: String,
    pub skill_version_id: String,
    pub agent_id: String,
    #[serde(default = "default_object")]
    pub agent_config_snapshot: serde_json::Value,
    pub issue_id: String,
    #[serde(default)]
    pub template_id: Option<String>,
    #[serde(default)]
    pub template_name: Option<String>,
    #[serde(default)]
    pub template_body: Option<String>,
    #[serde(default)]
    pub rendered_template_body: Option<String>,
    #[serde(default)]
    pub harness_issue_description: String,
    #[serde(default = "default_queued")]
    pub status: String,
    #[serde(default = "default_output")]
    pub output_document_key: String,
    #[serde(default)]
    pub output_snapshot: String,
    #[serde(default)]
    pub error: Option<String>,
}

fn default_array() -> serde_json::Value {
    serde_json::json!([])
}
fn default_object() -> serde_json::Value {
    serde_json::json!({})
}
fn default_one() -> i64 {
    1
}
fn default_queued() -> String {
    "queued".to_owned()
}
fn default_output() -> String {
    "output".to_owned()
}

/// `POST /api/companies/{companyId}/skills/{skillId}/versions` — publishes a
/// new skill version (revision auto-increments).
#[route(POST "/api/companies/{company_id}/skills/{skill_id}/versions")]
pub async fn publish_version(
    cx: &Cx,
    Json(body): Json<PublishVersionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    crate::auth::require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let skill_id = path_param::<SkillId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let version = state
        .skill_catalog
        .publish_version(NewSkillVersion {
            company_id,
            company_skill_id: skill_id,
            label: body.label,
            release_id: body.release_id,
            release_name: body.release_name,
            released_at: body.released_at,
            file_inventory: body.file_inventory,
            author_agent_id: body.author_agent_id,
            author_user_id: body.author_user_id,
        })
        .await
        .map_err(skill_catalog_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&version).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/skills/{skillId}/versions` — lists skill
/// versions.
#[route(GET "/api/companies/{company_id}/skills/{skill_id}/versions")]
pub async fn list_versions(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let skill_id = path_param::<SkillId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let versions = state
        .skill_catalog
        .list_versions(&company_id, &skill_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&versions).unwrap_or_default()))
}

/// `PUT /api/companies/{companyId}/skill-policies` — sets the company skill
/// policy (upsert, revision auto-increments).
#[route(PUT "/api/companies/{company_id}/skill-policies")]
pub async fn set_policy(
    cx: &Cx,
    Json(body): Json<SetPolicyRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    crate::auth::require_board(cx)?;
    if body.default_effect != "allow" && body.default_effect != "deny" {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["defaultEffect"], "message": "Must be allow or deny" }]),
        ));
    }
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let policy = state
        .skill_catalog
        .set_policy(SetSkillPolicy {
            company_id,
            schema_version: body.schema_version,
            default_effect: body.default_effect,
            rules: body.rules,
        })
        .await
        .map_err(skill_catalog_error_to_api)?;
    Ok(Json(serde_json::to_value(&policy).unwrap_or_default()))
}

/// `GET /api/companies/{companyId}/skill-policies` — fetches the company
/// skill policy.
#[route(GET "/api/companies/{company_id}/skill-policies")]
pub async fn get_policy(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let policy = state
        .skill_catalog
        .get_policy(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    match policy {
        Some(policy) => Ok(Json(serde_json::to_value(&policy).unwrap_or_default())),
        None => Err(ApiError::not_found("Skill policy not found")),
    }
}

/// `POST /api/companies/{companyId}/skills/{skillId}/comments` — creates a
/// skill comment.
#[route(POST "/api/companies/{company_id}/skills/{skill_id}/comments")]
pub async fn create_comment(
    cx: &Cx,
    Json(body): Json<CreateCommentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    crate::auth::require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let skill_id = path_param::<SkillId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let comment = state
        .skill_catalog
        .create_comment(NewSkillComment {
            company_id,
            company_skill_id: skill_id,
            parent_comment_id: body.parent_comment_id,
            author_agent_id: body.author_agent_id,
            author_user_id: body.author_user_id,
            body: body.body,
        })
        .await
        .map_err(skill_catalog_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&comment).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/skills/{skillId}/comments` — lists skill
/// comments.
#[route(GET "/api/companies/{company_id}/skills/{skill_id}/comments")]
pub async fn list_comments(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let skill_id = path_param::<SkillId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let comments = state
        .skill_catalog
        .list_comments(&company_id, &skill_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&comments).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/skills/{skillId}/stars` — stars a skill
/// for an agent or user.
#[route(POST "/api/companies/{company_id}/skills/{skill_id}/stars")]
pub async fn create_star(
    cx: &Cx,
    Json(body): Json<CreateStarRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    crate::auth::require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let skill_id = path_param::<SkillId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let star = state
        .skill_catalog
        .create_star(NewSkillStar {
            company_id,
            company_skill_id: skill_id,
            agent_id: body.agent_id,
            user_id: body.user_id,
        })
        .await
        .map_err(skill_catalog_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&star).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/skills/{skillId}/stars` — lists stars for
/// a skill.
#[route(GET "/api/companies/{company_id}/skills/{skill_id}/stars")]
pub async fn list_stars(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let skill_id = path_param::<SkillId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let stars = state
        .skill_catalog
        .list_stars(&company_id, &skill_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&stars).unwrap_or_default()))
}

/// `POST .../skills/{skillId}/test-inputs` — creates a skill test input.
#[route(POST "/api/companies/{company_id}/skills/{skill_id}/test-inputs")]
pub async fn create_test_input(
    cx: &Cx,
    Json(body): Json<CreateTestInputRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    crate::auth::require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let skill_id = path_param::<SkillId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let input = state
        .skill_catalog
        .create_test_input(NewSkillTestInput {
            company_id,
            skill_id,
            name: body.name,
            content: body.content,
            created_by: body.created_by,
        })
        .await
        .map_err(skill_catalog_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&input).unwrap_or_default()),
    ))
}

/// `GET .../skills/{skillId}/test-inputs` — lists skill test inputs.
#[route(GET "/api/companies/{company_id}/skills/{skill_id}/test-inputs")]
pub async fn list_test_inputs(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let skill_id = path_param::<SkillId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let inputs = state
        .skill_catalog
        .list_test_inputs(&company_id, &skill_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&inputs).unwrap_or_default()))
}

/// `POST .../skill-test-run-templates` — creates a skill test run template.
#[route(POST "/api/companies/{company_id}/skill-test-run-templates")]
pub async fn create_test_run_template(
    cx: &Cx,
    Json(body): Json<CreateTestRunTemplateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    crate::auth::require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let template = state
        .skill_catalog
        .create_test_run_template(NewSkillTestRunTemplate {
            company_id,
            name: body.name,
            description: body.description,
            body: body.body,
            created_by_agent_id: body.created_by_agent_id,
            created_by_user_id: body.created_by_user_id,
            updated_by_agent_id: body.updated_by_agent_id,
            updated_by_user_id: body.updated_by_user_id,
        })
        .await
        .map_err(skill_catalog_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&template).unwrap_or_default()),
    ))
}

/// `GET .../skill-test-run-templates` — lists skill test run templates.
#[route(GET "/api/companies/{company_id}/skill-test-run-templates")]
pub async fn list_test_run_templates(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let templates = state
        .skill_catalog
        .list_test_run_templates(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&templates).unwrap_or_default()))
}

/// `POST .../skills/{skillId}/test-runs` — creates a skill test run.
#[route(POST "/api/companies/{company_id}/skills/{skill_id}/test-runs")]
pub async fn create_test_run(
    cx: &Cx,
    Json(body): Json<CreateTestRunRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    crate::auth::require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let skill_id = path_param::<SkillId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let run = state
        .skill_catalog
        .create_test_run(NewSkillTestRun {
            company_id,
            skill_id,
            input_id: body.input_id,
            input_snapshot: body.input_snapshot,
            skill_version_id: body.skill_version_id,
            agent_id: body.agent_id,
            agent_config_snapshot: body.agent_config_snapshot,
            issue_id: body.issue_id,
            template_id: body.template_id,
            template_name: body.template_name,
            template_body: body.template_body,
            rendered_template_body: body.rendered_template_body,
            harness_issue_description: body.harness_issue_description,
            status: body.status,
            output_document_key: body.output_document_key,
            output_snapshot: body.output_snapshot,
            error: body.error,
        })
        .await
        .map_err(skill_catalog_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&run).unwrap_or_default()),
    ))
}

/// `GET .../skills/{skillId}/test-runs` — lists skill test runs.
#[route(GET "/api/companies/{company_id}/skills/{skill_id}/test-runs")]
pub async fn list_test_runs(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let skill_id = path_param::<SkillId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let runs = state
        .skill_catalog
        .list_test_runs(&company_id, &skill_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&runs).unwrap_or_default()))
}

fn skill_catalog_error_to_api(error: staple_data::SkillCatalogError) -> ApiError {
    use staple_data::SkillCatalogError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::SkillNotFound => ApiError::not_found("Skill not found"),
        E::ReferenceNotFound => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["references"], "message": "Referenced record not found or out of company" }]),
        ),
        E::AlreadyExists => ApiError::conflict("Record already exists"),
        other => ApiError::internal(other.to_string()),
    }
}
