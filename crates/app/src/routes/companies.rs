//! Company CRUD routes.

use serde::{Deserialize, Serialize};
use serde_json::json;
use staple_data::{CompanyPatch, NewCompany};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{dto::CompanyDto, error::ApiError, routes::CompanyId, state::AppState};

/// Largest allowed attachment size in bytes (upstream constant).
const MAX_COMPANY_ATTACHMENT_MAX_BYTES: i64 = 1024 * 1024 * 1024;

/// Body for `POST /api/companies`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCompanyRequest {
    /// Display name (required, non-empty; validated at the API layer so a
    /// missing field yields 422 instead of an extractor 400).
    #[serde(default)]
    pub name: Option<String>,
    /// Optional description.
    pub description: Option<String>,
    /// Monthly budget in cents.
    #[serde(default)]
    pub budget_monthly_cents: Option<i64>,
    /// Largest attachment size in bytes.
    #[serde(default)]
    pub attachment_max_bytes: Option<i64>,
}

/// Body for `PATCH /api/companies/{companyId}`. `null` clears nullable fields.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCompanyRequest {
    /// New name.
    #[serde(default)]
    pub name: Option<String>,
    /// New description (`null` clears).
    #[serde(default)]
    pub description: Option<Option<String>>,
    /// New status.
    #[serde(default)]
    pub status: Option<String>,
    /// New monthly budget in cents.
    #[serde(default)]
    pub budget_monthly_cents: Option<i64>,
    /// New spent-this-month amount in cents.
    #[serde(default)]
    pub spent_monthly_cents: Option<i64>,
    /// New attachment size limit in bytes.
    #[serde(default)]
    pub attachment_max_bytes: Option<i64>,
    /// New require-board-approval flag.
    #[serde(default)]
    pub require_board_approval_for_new_agents: Option<bool>,
    /// New brand color (`null` clears).
    #[serde(default)]
    pub brand_color: Option<Option<String>>,
}

/// A single validation issue, mirroring the upstream Zod error shape.
#[derive(Debug, Serialize)]
pub struct ValidationIssue {
    path: Vec<String>,
    message: String,
}

fn issue(path: &str, message: &str) -> ValidationIssue {
    ValidationIssue {
        path: vec![path.to_owned()],
        message: message.to_owned(),
    }
}

fn validate_create(body: &CreateCompanyRequest) -> Result<(), ApiError> {
    let mut issues = Vec::new();
    let name = body.name.as_deref().unwrap_or_default();
    if name.trim().is_empty() {
        issues.push(issue("name", "String must contain at least 1 character(s)"));
    }
    if let Some(budget) = body.budget_monthly_cents
        && budget < 0
    {
        issues.push(issue(
            "budgetMonthlyCents",
            "Number must be greater than or equal to 0",
        ));
    }
    if let Some(max) = body.attachment_max_bytes
        && !(1..=MAX_COMPANY_ATTACHMENT_MAX_BYTES).contains(&max)
    {
        issues.push(issue(
            "attachmentMaxBytes",
            &format!("Number must be less than or equal to {MAX_COMPANY_ATTACHMENT_MAX_BYTES}"),
        ));
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(validation_error(issues))
    }
}

fn validate_update(body: &UpdateCompanyRequest) -> Result<(), ApiError> {
    let mut issues = Vec::new();
    if let Some(name) = &body.name
        && name.trim().is_empty()
    {
        issues.push(issue("name", "String must contain at least 1 character(s)"));
    }
    if let Some(status) = &body.status
        && !matches!(status.as_str(), "active" | "paused" | "archived")
    {
        issues.push(issue(
            "status",
            "Invalid enum value. Expected 'active' | 'paused' | 'archived'",
        ));
    }
    if let Some(budget) = body.budget_monthly_cents
        && budget < 0
    {
        issues.push(issue(
            "budgetMonthlyCents",
            "Number must be greater than or equal to 0",
        ));
    }
    if let Some(spent) = body.spent_monthly_cents
        && spent < 0
    {
        issues.push(issue(
            "spentMonthlyCents",
            "Number must be greater than or equal to 0",
        ));
    }
    if let Some(max) = body.attachment_max_bytes
        && !(1..=MAX_COMPANY_ATTACHMENT_MAX_BYTES).contains(&max)
    {
        issues.push(issue(
            "attachmentMaxBytes",
            &format!("Number must be less than or equal to {MAX_COMPANY_ATTACHMENT_MAX_BYTES}"),
        ));
    }
    if let Some(Some(color)) = &body.brand_color
        && !is_valid_brand_color(color)
    {
        issues.push(issue("brandColor", "Invalid brand color, expected #RRGGBB"));
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(validation_error(issues))
    }
}

fn validation_error(issues: Vec<ValidationIssue>) -> ApiError {
    ApiError::unprocessable("Validation error", json!(issues))
}

fn is_valid_brand_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value[1..].chars().all(|ch| ch.is_ascii_hexdigit())
}

/// `POST /api/companies` — creates a company, returns 201.
#[route(POST "/api/companies")]
pub async fn create_company(
    cx: &Cx,
    Json(body): Json<CreateCompanyRequest>,
) -> Result<(StatusCode, Json<CompanyDto>), ApiError> {
    validate_create(&body)?;
    let state = app_context::<AppState>(cx);
    let company = state
        .companies
        .create(NewCompany {
            name: body.name.unwrap_or_default().trim().to_owned(),
            description: body.description,
            budget_monthly_cents: body.budget_monthly_cents.unwrap_or(0),
            attachment_max_bytes: body.attachment_max_bytes.unwrap_or(0),
        })
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok((StatusCode::CREATED, Json(company.into())))
}

/// `GET /api/companies` — lists all companies.
#[route(GET "/api/companies")]
pub async fn list_companies(cx: &Cx) -> Result<Json<Vec<CompanyDto>>, ApiError> {
    let state = app_context::<AppState>(cx);
    let companies = state
        .companies
        .list()
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(companies.into_iter().map(CompanyDto::from).collect()))
}

/// `GET /api/companies/{companyId}` — fetches one company.
#[route(GET "/api/companies/{company_id}")]
pub async fn get_company(cx: &Cx) -> Result<Json<CompanyDto>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company = state
        .companies
        .get(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    match company {
        Some(company) => Ok(Json(company.into())),
        None => Err(ApiError::not_found("Company not found")),
    }
}

/// `PATCH /api/companies/{companyId}` — partially updates a company.
#[route(PATCH "/api/companies/{company_id}")]
pub async fn update_company(
    cx: &Cx,
    Json(body): Json<UpdateCompanyRequest>,
) -> Result<Json<CompanyDto>, ApiError> {
    validate_update(&body)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let patch = CompanyPatch {
        name: body.name.map(|value| value.trim().to_owned()),
        description: body.description,
        status: body.status,
        budget_monthly_cents: body.budget_monthly_cents,
        spent_monthly_cents: body.spent_monthly_cents,
        attachment_max_bytes: body.attachment_max_bytes,
        require_board_approval_for_new_agents: body.require_board_approval_for_new_agents,
        brand_color: body.brand_color,
    };
    let company = state
        .companies
        .update(&company_id, patch)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    match company {
        Some(company) => Ok(Json(company.into())),
        None => Err(ApiError::not_found("Company not found")),
    }
}
