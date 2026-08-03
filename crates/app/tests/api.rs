//! API integration tests: health endpoint, unified JSON error handling, and
//! company CRUD.

use std::sync::Arc;

use http::{Method, Request, header::CONTENT_TYPE};
use serde_json::{Value, json};
use staple_app::router;
use staple_app::state::AppState;
use staple_data::{DbConfig, TursoCompanyRepository, migrate, open};
use topcoat::router::{Body, Router, StatusCode, to_bytes};

async fn test_state() -> AppState {
    let dir = tempfile::tempdir().unwrap();
    let db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    migrate(&db).await.unwrap();
    // Keep the temp dir alive for the lifetime of the test process.
    std::mem::forget(dir);
    AppState {
        companies: Arc::new(TursoCompanyRepository::new(db)),
    }
}

async fn send(
    router: &Router,
    method: Method,
    path: &str,
    body: Option<&str>,
) -> (StatusCode, String) {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(_body) = body {
        builder = builder.header(CONTENT_TYPE, "application/json");
    }
    let request = builder
        .body(Body::from(body.unwrap_or_default().to_owned()))
        .unwrap();
    let response = router.handle(request).await;
    let status = response.status();
    let (_, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

async fn send_json(
    router: &Router,
    method: Method,
    path: &str,
    body: Value,
) -> (StatusCode, Value) {
    let (status, text) = send(router, method, path, Some(&body.to_string())).await;
    (status, serde_json::from_str(&text).unwrap_or(Value::Null))
}

#[tokio::test]
async fn health_returns_200_ok() {
    let (status, body) = send(
        &router(test_state().await),
        Method::GET,
        "/api/health",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, r#"{"status":"ok"}"#);
}

#[tokio::test]
async fn unknown_api_path_returns_json_404() {
    let (status, body) = send(
        &router(test_state().await),
        Method::GET,
        "/api/does-not-exist",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body, r#"{"error":"not found"}"#);
}

#[tokio::test]
async fn wrong_method_returns_json_405() {
    let (status, body) = send(
        &router(test_state().await),
        Method::POST,
        "/api/health",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(body, r#"{"error":"method not allowed"}"#);
}

#[tokio::test]
async fn non_api_404_is_also_json() {
    let (status, body) = send(&router(test_state().await), Method::GET, "/nope", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body, r#"{"error":"not found"}"#);
}

#[tokio::test]
async fn company_crud_flow() {
    let state = test_state().await;
    let app = router(state);

    // Create -> 201 with camelCase body.
    let (status, created) = send_json(
        &app,
        Method::POST,
        "/api/companies",
        json!({ "name": "Acme Corp", "budgetMonthlyCents": 2500 }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["name"], "Acme Corp");
    assert_eq!(created["status"], "active");
    assert_eq!(created["budgetMonthlyCents"], 2500);
    assert_eq!(created["spentMonthlyCents"], 0);
    assert_eq!(created["issuePrefix"], "ACM");
    let company_id = created["id"].as_str().unwrap().to_owned();

    // List -> contains the created company.
    let (status, list) = send_json(&app, Method::GET, "/api/companies", json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["id"], company_id);

    // Get by id.
    let (status, fetched) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["name"], "Acme Corp");

    // Patch -> updated fields.
    let (status, updated) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/companies/{company_id}"),
        json!({ "name": "Acme 2", "status": "paused", "description": null }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["name"], "Acme 2");
    assert_eq!(updated["status"], "paused");
    assert_eq!(updated["description"], Value::Null);
}

#[tokio::test]
async fn company_get_missing_returns_404() {
    let app = router(test_state().await);
    let (status, body) = send_json(
        &app,
        Method::GET,
        "/api/companies/does-not-exist",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body, json!({ "error": "Company not found" }));
}

#[tokio::test]
async fn company_patch_missing_returns_404() {
    let app = router(test_state().await);
    let (status, body) = send_json(
        &app,
        Method::PATCH,
        "/api/companies/does-not-exist",
        json!({ "name": "x" }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body, json!({ "error": "Company not found" }));
}

#[tokio::test]
async fn company_create_validation_failure_returns_422() {
    let app = router(test_state().await);

    // Empty name.
    let (status, body) =
        send_json(&app, Method::POST, "/api/companies", json!({ "name": "" })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "Validation error");
    assert_eq!(body["details"][0]["path"][0], "name");

    // Negative budget.
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies",
        json!({ "name": "Acme", "budgetMonthlyCents": -1 }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["details"][0]["path"][0], "budgetMonthlyCents");

    // Missing name.
    let (status, _) = send_json(&app, Method::POST, "/api/companies", json!({})).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn company_patch_validation_failure_returns_422() {
    let app = router(test_state().await);
    let (_, created) = send_json(
        &app,
        Method::POST,
        "/api/companies",
        json!({ "name": "Acme" }),
    )
    .await;
    let company_id = created["id"].as_str().unwrap().to_owned();

    let (status, body) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/companies/{company_id}"),
        json!({ "status": "invalid-status" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "Validation error");
    assert_eq!(body["details"][0]["path"][0], "status");

    let (status, _) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/companies/{company_id}"),
        json!({ "brandColor": "red" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn company_create_malformed_json_returns_json_400() {
    let app = router(test_state().await);
    let (status, body) = send(&app, Method::POST, "/api/companies", Some(r#"{not json"#)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        body.contains("\"error\"") && body.to_lowercase().contains("bad request"),
        "unexpected body: {body}"
    );
}
