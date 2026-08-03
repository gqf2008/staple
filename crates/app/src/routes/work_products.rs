//! Issue work product routes.

use serde::Deserialize;
use serde_json::json;
use staple_data::{NewWorkProduct, WorkProductPatch};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    dto::WorkProductDto,
    error::ApiError,
    routes::{Id, is_uuid},
    state::AppState,
};

/// Body for `POST /api/issues/{issueId}/work-products`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkProductRequest {
    /// Type.
    pub r#type: String,
    /// Provider.
    pub provider: String,
    /// Title (required, non-empty).
    #[serde(default)]
    pub title: Option<String>,
    /// Project id.
    #[serde(default)]
    pub project_id: Option<String>,
    /// External id.
    #[serde(default)]
    pub external_id: Option<String>,
    /// URL.
    #[serde(default)]
    pub url: Option<String>,
    /// Status (default `active`).
    #[serde(default)]
    pub status: Option<String>,
    /// Review state (default `none`).
    #[serde(default)]
    pub review_state: Option<String>,
    /// Primary flag (default false).
    #[serde(default)]
    pub is_primary: Option<bool>,
    /// Health status (default `unknown`).
    #[serde(default)]
    pub health_status: Option<String>,
    /// Summary.
    #[serde(default)]
    pub summary: Option<String>,
    /// Metadata (arbitrary JSON).
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// Body for `PATCH /api/work-products/{id}`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkProductRequest {
    /// New title.
    #[serde(default)]
    pub title: Option<String>,
    /// New status.
    #[serde(default)]
    pub status: Option<String>,
    /// New review state.
    #[serde(default)]
    pub review_state: Option<String>,
    /// New health status.
    #[serde(default)]
    pub health_status: Option<String>,
    /// New summary (`null` clears).
    #[serde(default)]
    pub summary: Option<Option<String>>,
}

fn validate_create(body: &CreateWorkProductRequest) -> Result<(), ApiError> {
    let mut issues = Vec::new();
    if body.title.as_deref().unwrap_or_default().trim().is_empty() {
        issues.push(issue(
            "title",
            "String must contain at least 1 character(s)",
        ));
    }
    if body.r#type.trim().is_empty() {
        issues.push(issue("type", "String must contain at least 1 character(s)"));
    }
    if body.provider.trim().is_empty() {
        issues.push(issue(
            "provider",
            "String must contain at least 1 character(s)",
        ));
    }
    if let Some(project_id) = &body.project_id
        && !is_uuid(project_id)
    {
        issues.push(issue("projectId", "Invalid uuid"));
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::unprocessable("Validation error", json!(issues)))
    }
}

fn issue(path: &str, message: &str) -> serde_json::Value {
    json!({ "path": [path], "message": message })
}

/// `GET /api/issues/{issueId}/work-products` — lists an issue's work products.
#[route(GET "/api/issues/{id}/work-products")]
pub async fn list_work_products(cx: &Cx) -> Result<Json<Vec<WorkProductDto>>, ApiError> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let products = state
        .work_products
        .list_for_issue(&issue_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(
        products.into_iter().map(WorkProductDto::from).collect(),
    ))
}

/// `POST /api/issues/{issueId}/work-products` — creates one, returns 201.
#[route(POST "/api/issues/{id}/work-products")]
pub async fn create_work_product(
    cx: &Cx,
    Json(body): Json<CreateWorkProductRequest>,
) -> Result<(StatusCode, Json<WorkProductDto>), ApiError> {
    validate_create(&body)?;
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let product = state
        .work_products
        .create(NewWorkProduct {
            issue_id,
            project_id: body.project_id,
            r#type: body.r#type,
            provider: body.provider,
            external_id: body.external_id,
            title: body.title.unwrap_or_default().trim().to_owned(),
            url: body.url,
            status: body.status.unwrap_or_else(|| "active".to_owned()),
            review_state: body.review_state.unwrap_or_else(|| "none".to_owned()),
            is_primary: body.is_primary.unwrap_or(false),
            health_status: body.health_status.unwrap_or_else(|| "unknown".to_owned()),
            summary: body.summary,
            metadata: body.metadata.map(|value| value.to_string()),
        })
        .await
        .map_err(work_product_error_to_api)?;
    Ok((StatusCode::CREATED, Json(product.into())))
}

/// `PATCH /api/work-products/{id}` — updates a work product.
#[route(PATCH "/api/work-products/{id}")]
pub async fn update_work_product(
    cx: &Cx,
    Json(body): Json<UpdateWorkProductRequest>,
) -> Result<Json<WorkProductDto>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let patch = WorkProductPatch {
        title: body.title.map(|value| value.trim().to_owned()),
        status: body.status,
        review_state: body.review_state,
        health_status: body.health_status,
        summary: body.summary,
    };
    match state
        .work_products
        .update(&id, patch)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(product) => Ok(Json(product.into())),
        None => Err(ApiError::not_found("Work product not found")),
    }
}

/// `DELETE /api/work-products/{id}` — deletes a work product.
#[route(DELETE "/api/work-products/{id}")]
pub async fn delete_work_product(cx: &Cx) -> Result<StatusCode, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .work_products
        .delete(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err(ApiError::not_found("Work product not found")),
    }
}

fn work_product_error_to_api(error: staple_data::WorkProductError) -> ApiError {
    use staple_data::WorkProductError as E;
    match error {
        E::IssueNotFound => ApiError::not_found("Issue not found"),
        E::ProjectInDifferentCompany => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["projectId"], "message": "Project belongs to a different company" }]),
        ),
        other => ApiError::internal(other.to_string()),
    }
}
