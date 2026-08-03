//! API integration tests: health endpoint, unified JSON error handling, and
//! company CRUD.

use std::sync::Arc;

use http::{Method, Request, header::CONTENT_TYPE};
use serde_json::{Value, json};
use staple_app::router;
use staple_app::state::AppState;
use staple_app::storage::LocalStorage;
use staple_data::{
    DbConfig, SecretCipher, TursoActivityRepository, TursoApiKeyRepository,
    TursoApprovalRepository, TursoAssetRepository, TursoCompanyRepository, TursoCostRepository,
    TursoDecisionRepository, TursoDocumentRepository, TursoExternalObjectRepository,
    TursoGoalRepository, TursoHeartbeatRepository, TursoIssueCommentRepository,
    TursoIssueRelationRepository, TursoIssueRepository, TursoProjectRepository,
    TursoSecretRepository, TursoWorkProductRepository, migrate, open,
};
use topcoat::router::{Body, Router, StatusCode, to_bytes};

async fn test_state() -> AppState {
    test_state_with_db().await.0
}

async fn test_state_with_db() -> (AppState, staple_data::Database) {
    let dir = tempfile::tempdir().unwrap();
    let seed_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let companies_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let goals_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let projects_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let issues_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let comments_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let documents_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let assets_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let relations_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let work_products_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let heartbeat_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let costs_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let approvals_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let activity_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let secrets_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let secret_cipher = SecretCipher::load_or_create(dir.path().join("master.key")).unwrap();
    let api_keys_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let decisions_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let external_objects_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    migrate(&companies_db).await.unwrap();
    let uploads = dir.path().join("uploads");
    // Keep the temp dir alive for the lifetime of the test process.
    std::mem::forget(dir);
    let state = AppState {
        companies: Arc::new(TursoCompanyRepository::new(companies_db)),
        goals: Arc::new(TursoGoalRepository::new(goals_db)),
        projects: Arc::new(TursoProjectRepository::new(projects_db)),
        issues: Arc::new(TursoIssueRepository::new(issues_db)),
        comments: Arc::new(TursoIssueCommentRepository::new(comments_db)),
        documents: Arc::new(TursoDocumentRepository::new(documents_db)),
        assets: Arc::new(TursoAssetRepository::new(assets_db)),
        relations: Arc::new(TursoIssueRelationRepository::new(relations_db)),
        storage: LocalStorage::new(uploads),
        work_products: Arc::new(TursoWorkProductRepository::new(work_products_db)),
        heartbeat: Arc::new(TursoHeartbeatRepository::new(heartbeat_db)),
        costs: Arc::new(TursoCostRepository::new(costs_db)),
        approvals: Arc::new(TursoApprovalRepository::new(approvals_db)),
        activity: Arc::new(TursoActivityRepository::new(activity_db)),
        secrets: Arc::new(TursoSecretRepository::new(secrets_db, secret_cipher)),
        api_keys: Arc::new(TursoApiKeyRepository::new(api_keys_db)),
        decisions: Arc::new(TursoDecisionRepository::new(decisions_db)),
        external_objects: Arc::new(TursoExternalObjectRepository::new(external_objects_db)),
    };
    (state, seed_db)
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

async fn create_company_via(app: &Router, name: &str) -> String {
    let (status, body) =
        send_json(app, Method::POST, "/api/companies", json!({ "name": name })).await;
    assert_eq!(status, StatusCode::CREATED);
    body["id"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn goal_crud_flow() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;

    // Create -> 201.
    let (status, created) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/goals"),
        json!({ "title": "Growth", "level": "team", "status": "active" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["title"], "Growth");
    assert_eq!(created["level"], "team");
    assert_eq!(created["status"], "active");
    let goal_id = created["id"].as_str().unwrap().to_owned();

    // List.
    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/goals"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Get by id.
    let (status, fetched) = send_json(
        &app,
        Method::GET,
        &format!("/api/goals/{goal_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["title"], "Growth");

    // Patch.
    let (status, updated) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/goals/{goal_id}"),
        json!({ "status": "achieved", "description": null }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["status"], "achieved");
    assert_eq!(updated["description"], Value::Null);

    // Delete -> 204, then 404.
    let (status, _) = send(&app, Method::DELETE, &format!("/api/goals/{goal_id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!("/api/goals/{goal_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body, json!({ "error": "Goal not found" }));
}

#[tokio::test]
async fn goal_hierarchy_constraints_are_enforced() {
    let app = router(test_state().await);
    let company_a = create_company_via(&app, "Alpha").await;
    let company_b = create_company_via(&app, "Beta").await;

    // Parent in another company is rejected with 422.
    let (_, parent) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_a}/goals"),
        json!({ "title": "Parent" }),
    )
    .await;
    let parent_id = parent["id"].as_str().unwrap().to_owned();

    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_b}/goals"),
        json!({ "title": "Child", "parentId": parent_id }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["details"][0]["path"][0], "parentId");
    assert_eq!(
        body["details"][0]["message"],
        "Parent goal belongs to a different company"
    );
}

#[tokio::test]
async fn goal_validation_failure_returns_422() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/goals"),
        json!({ "title": "", "level": "bogus" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "Validation error");
    let paths: Vec<&str> = body["details"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["path"][0].as_str().unwrap())
        .collect();
    assert!(paths.contains(&"title"));
    assert!(paths.contains(&"level"));
}

#[tokio::test]
async fn project_crud_flow() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;

    // Goal for linking.
    let (_, goal) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/goals"),
        json!({ "title": "Growth" }),
    )
    .await;
    let goal_id = goal["id"].as_str().unwrap().to_owned();

    // Create -> 201.
    let (status, created) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/projects"),
        json!({ "name": "Ship", "goalId": goal_id, "status": "planned" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["name"], "Ship");
    assert_eq!(created["goalId"], goal_id);
    let project_id = created["id"].as_str().unwrap().to_owned();

    // List.
    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/projects"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Get.
    let (status, _fetched) = send_json(
        &app,
        Method::GET,
        &format!("/api/projects/{project_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Patch.
    let (status, updated) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/projects/{project_id}"),
        json!({ "status": "in_progress", "description": "now" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["status"], "in_progress");

    // Delete -> 204.
    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/api/projects/{project_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn project_hierarchy_constraints_are_enforced() {
    let app = router(test_state().await);
    let company_a = create_company_via(&app, "Alpha").await;
    let company_b = create_company_via(&app, "Beta").await;

    let (_, goal) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_a}/goals"),
        json!({ "title": "G" }),
    )
    .await;
    let goal_id = goal["id"].as_str().unwrap().to_owned();

    // Cross-company goal link is rejected with 422.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_b}/projects"),
        json!({ "name": "P", "goalId": goal_id }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["details"][0]["path"][0], "goalId");
    assert_eq!(
        body["details"][0]["message"],
        "Goal belongs to a different company"
    );
}

#[tokio::test]
async fn project_404s() {
    let app = router(test_state().await);
    let (status, body) = send_json(&app, Method::GET, "/api/projects/missing", json!({})).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body, json!({ "error": "Project not found" }));

    let (status, body) = send_json(
        &app,
        Method::PATCH,
        "/api/projects/missing",
        json!({ "name": "x" }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body, json!({ "error": "Project not found" }));
}

#[tokio::test]
async fn delete_referenced_goal_returns_409() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let (_, goal) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/goals"),
        json!({ "title": "G" }),
    )
    .await;
    let goal_id = goal["id"].as_str().unwrap().to_owned();

    // Link a project to the goal, then deletion must conflict.
    send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/projects"),
        json!({ "name": "P", "goalId": goal_id }),
    )
    .await;

    let (status, body) = send(&app, Method::DELETE, &format!("/api/goals/{goal_id}"), None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body, r#"{"error":"Goal is referenced by other records"}"#);
}

#[tokio::test]
async fn issue_crud_flow() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;

    // Create -> 201 with identifier allocation.
    let (status, created) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/issues"),
        json!({ "title": "First task", "priority": "high" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["title"], "First task");
    assert_eq!(created["identifier"], "ACM-1");
    assert_eq!(created["issueNumber"], 1);
    assert_eq!(created["status"], "backlog");
    assert_eq!(created["priority"], "high");
    let issue_id = created["id"].as_str().unwrap().to_owned();

    // Assigned issue defaults to todo.
    let (_, assigned) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/issues"),
        json!({ "title": "Second", "assigneeUserId": "u1" }),
    )
    .await;
    assert_eq!(assigned["status"], "todo");
    assert_eq!(assigned["identifier"], "ACM-2");

    // List.
    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/issues"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 2);

    // Get.
    let (status, fetched) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["title"], "First task");

    // Patch through the state machine: backlog -> todo -> in_progress -> done.
    let (status, updated) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/issues/{issue_id}"),
        json!({ "status": "todo" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["status"], "todo");

    let (status, updated) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/issues/{issue_id}"),
        json!({ "status": "in_progress" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(updated["startedAt"].is_string());

    let (status, updated) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/issues/{issue_id}"),
        json!({ "status": "done", "title": "First task (done)" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["status"], "done");
    assert!(updated["completedAt"].is_string());

    // Delete -> 204.
    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/api/issues/{issue_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn issue_state_machine_rejects_invalid_transition() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let (_, created) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/issues"),
        json!({ "title": "T" }),
    )
    .await;
    let issue_id = created["id"].as_str().unwrap().to_owned();

    // backlog -> done is not allowed.
    let (status, body) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/issues/{issue_id}"),
        json!({ "status": "done" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["details"][0]["path"][0], "status");
    assert!(
        body["details"][0]["message"]
            .as_str()
            .unwrap()
            .contains("Invalid status transition")
    );
}

#[tokio::test]
async fn issue_validation_failure_returns_422() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/issues"),
        json!({ "title": "", "priority": "urgent" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "Validation error");
    let paths: Vec<&str> = body["details"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["path"][0].as_str().unwrap())
        .collect();
    assert!(paths.contains(&"title"));
    assert!(paths.contains(&"priority"));
}

#[tokio::test]
async fn issue_hierarchy_constraints_are_enforced() {
    let app = router(test_state().await);
    let company_a = create_company_via(&app, "Alpha").await;
    let company_b = create_company_via(&app, "Beta").await;

    let (_, goal) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_a}/goals"),
        json!({ "title": "G" }),
    )
    .await;
    let goal_id = goal["id"].as_str().unwrap().to_owned();

    // Cross-company goal link is rejected.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_b}/issues"),
        json!({ "title": "T", "goalId": goal_id }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["details"][0]["path"][0], "goal");
}

#[tokio::test]
async fn issue_404s() {
    let app = router(test_state().await);
    let (status, body) = send_json(&app, Method::GET, "/api/issues/missing", json!({})).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body, json!({ "error": "Issue not found" }));

    let (status, body) = send_json(
        &app,
        Method::PATCH,
        "/api/issues/missing",
        json!({ "title": "x" }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body, json!({ "error": "Issue not found" }));
}

async fn create_issue_via(app: &Router, company_id: &str, title: &str) -> String {
    let (status, body) = send_json(
        app,
        Method::POST,
        &format!("/api/companies/{company_id}/issues"),
        json!({ "title": title }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    body["id"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn comment_crud_flow() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let issue_id = create_issue_via(&app, &company_id, "T").await;

    // Add a comment -> 201.
    let (status, created) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/comments"),
        json!({ "body": "First comment", "authorUserId": "u1" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["body"], "First comment");
    assert_eq!(created["authorUserId"], "u1");
    let comment_id = created["id"].as_str().unwrap().to_owned();

    // List.
    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_id}/comments"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Get one.
    let (status, fetched) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_id}/comments/{comment_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["body"], "First comment");

    // Empty body -> 422.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/comments"),
        json!({ "body": " " }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Delete -> 204, then 404.
    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/api/issues/{issue_id}/comments/{comment_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/api/issues/{issue_id}/comments/{comment_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn document_crud_flow() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let issue_id = create_issue_via(&app, &company_id, "T").await;

    // Create -> 201 with revision 1.
    let (status, created) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/documents"),
        json!({ "key": "plan", "title": "Plan", "body": "# v1" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["latestBody"], "# v1");
    assert_eq!(created["latestRevisionNumber"], 1);

    // Duplicate key -> 409.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/documents"),
        json!({ "key": "plan", "body": "x" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Update -> revision 2.
    let (status, updated) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/issues/{issue_id}/documents/plan"),
        json!({ "body": "# v2", "changeSummary": "rewrite" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["latestBody"], "# v2");
    assert_eq!(updated["latestRevisionNumber"], 2);

    // Get by key.
    let (status, fetched) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_id}/documents/plan"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["latestRevisionNumber"], 2);

    // List.
    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_id}/documents"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Missing key -> 404.
    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_id}/documents/design"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body, json!({ "error": "Document not found" }));
}

#[tokio::test]
async fn blocker_flow() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let blocking = create_issue_via(&app, &company_id, "blocking").await;
    let blocked = create_issue_via(&app, &company_id, "blocked").await;

    // Add blocker -> 201.
    let (status, created) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{blocked}/blockers"),
        json!({ "blockerIssueId": blocking }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["issueId"], blocking);
    assert_eq!(created["relatedIssueId"], blocked);
    let relation_id = created["id"].as_str().unwrap().to_owned();

    // List blockers of the blocked issue.
    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{blocked}/blockers"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["issueId"], blocking);

    // Duplicate -> 409.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{blocked}/blockers"),
        json!({ "blockerIssueId": blocking }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Remove -> 204.
    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/api/issue-relations/{relation_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn asset_upload_and_attach_flow() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let issue_id = create_issue_via(&app, &company_id, "T").await;

    // Upload (multipart body built by hand).
    let boundary = "X-TEST-BOUNDARY";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"hello.txt\"\r\nContent-Type: text/plain\r\n\r\nhello\r\n--{boundary}--\r\n"
    );
    let request = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/companies/{company_id}/assets"))
        .header(
            CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();
    let response = app.handle(request).await;
    let status = response.status();
    let (_, response_body) = response.into_parts();
    let bytes = to_bytes(response_body, usize::MAX).await.unwrap();
    let asset: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(asset["originalFilename"], "hello.txt");
    assert_eq!(asset["byteSize"], 5);
    assert_eq!(asset["provider"], "local_disk");
    let asset_id = asset["id"].as_str().unwrap().to_owned();

    // Read content back.
    let (status, content) = send(
        &app,
        Method::GET,
        &format!("/api/assets/{asset_id}/content"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(content, "hello");

    // Attach to issue -> 201.
    let (status, attachment) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/attachments"),
        json!({ "assetId": asset_id }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(attachment["assetId"], asset_id);

    // List attachments.
    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_id}/attachments"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn work_product_crud_flow() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let issue_id = create_issue_via(&app, &company_id, "T").await;

    // Create -> 201.
    let (status, created) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/work-products"),
        json!({
            "type": "artifact",
            "provider": "paperclip",
            "title": "Report.pdf",
            "status": "active",
            "isPrimary": true,
            "metadata": { "kind": "workspace_file", "relativePath": "report.pdf" }
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["title"], "Report.pdf");
    assert_eq!(created["isPrimary"], true);
    let product_id = created["id"].as_str().unwrap().to_owned();

    // List.
    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_id}/work-products"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Patch.
    let (status, updated) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/work-products/{product_id}"),
        json!({ "status": "archived", "summary": null }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["status"], "archived");

    // Delete -> 204, then 404.
    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/api/work-products/{product_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/api/work-products/{product_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn work_product_validation_failure_returns_422() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let issue_id = create_issue_via(&app, &company_id, "T").await;
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/work-products"),
        json!({ "type": "", "provider": "", "title": "" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "Validation error");
}

#[tokio::test]
async fn heartbeat_run_lifecycle_and_lock() {
    let (state, db) = test_state_with_db().await;
    let app = router(state);
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO agents (id, company_id, name, role, adapter_type)
         VALUES ('11111111-1111-1111-1111-111111111111', 'c1', 'one', 'engineer', 'codex_local')",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO issues (id, company_id, title, issue_number, identifier, status)
         VALUES ('22222222-2222-2222-2222-222222222222', 'c1', 'T', 1, 'ALPHA-1', 'in_progress')",
        (),
    )
    .await
    .unwrap();

    // Start -> 201, running.
    let (status, run) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/heartbeat-runs",
        json!({ "agentId": "11111111-1111-1111-1111-111111111111", "invocationSource": "manual", "issueId": "22222222-2222-2222-2222-222222222222" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(run["status"], "running");
    let run_id = run["id"].as_str().unwrap().to_owned();

    // Concurrent start on the same issue -> 409.
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/heartbeat-runs",
        json!({ "agentId": "11111111-1111-1111-1111-111111111111", "invocationSource": "manual", "issueId": "22222222-2222-2222-2222-222222222222" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("already checked out")
    );

    // Observe.
    let (status, observed) = send_json(
        &app,
        Method::GET,
        &format!("/api/heartbeat-runs/{run_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(observed["status"], "running");

    // List.
    let (status, list) = send_json(
        &app,
        Method::GET,
        "/api/companies/c1/heartbeat-runs",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Complete -> releases the lock, another run can start.
    let (status, completed) = send_json(
        &app,
        Method::POST,
        &format!("/api/heartbeat-runs/{run_id}/complete"),
        json!({ "status": "succeeded" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(completed["status"], "succeeded");

    let (status, _) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/heartbeat-runs",
        json!({ "agentId": "11111111-1111-1111-1111-111111111111", "invocationSource": "manual", "issueId": "22222222-2222-2222-2222-222222222222" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn heartbeat_failure_attribution_and_cancel() {
    let (state, db) = test_state_with_db().await;
    let app = router(state);
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO agents (id, company_id, name, role, adapter_type)
         VALUES ('11111111-1111-1111-1111-111111111111', 'c1', 'one', 'engineer', 'codex_local')",
        (),
    )
    .await
    .unwrap();

    // Infrastructure failure attribution.
    let (_, run) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/heartbeat-runs",
        json!({ "agentId": "11111111-1111-1111-1111-111111111111", "invocationSource": "manual" }),
    )
    .await;
    let run_id = run["id"].as_str().unwrap().to_owned();
    let (status, completed) = send_json(
        &app,
        Method::POST,
        &format!("/api/heartbeat-runs/{run_id}/complete"),
        json!({ "status": "failed", "error": "clone failed", "errorKind": "infrastructure" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(completed["errorKind"], "infrastructure");
    assert_eq!(completed["error"], "clone failed");

    // Cancel.
    let (_, run) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/heartbeat-runs",
        json!({ "agentId": "11111111-1111-1111-1111-111111111111", "invocationSource": "manual" }),
    )
    .await;
    let run_id = run["id"].as_str().unwrap().to_owned();
    let (status, cancelled) = send_json(
        &app,
        Method::POST,
        &format!("/api/heartbeat-runs/{run_id}/cancel"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cancelled["status"], "cancelled");

    // Invalid completion status -> 422.
    let (_, run) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/heartbeat-runs",
        json!({ "agentId": "11111111-1111-1111-1111-111111111111" }),
    )
    .await;
    let run_id = run["id"].as_str().unwrap().to_owned();
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/heartbeat-runs/{run_id}/complete"),
        json!({ "status": "bogus" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn watchdog_authorization_via_api() {
    let (state, db) = test_state_with_db().await;
    let app = router(state);
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO agents (id, company_id, name, role, adapter_type)
         VALUES ('11111111-1111-1111-1111-111111111111', 'c1', 'one', 'engineer', 'codex_local')",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO issues (id, company_id, title, issue_number, identifier)
         VALUES ('22222222-2222-2222-2222-222222222222', 'c1', 'root', 1, 'ALPHA-1'),
                ('33333333-3333-3333-3333-333333333333', 'c1', 'child', 2, 'ALPHA-2'),
                ('44444444-4444-4444-4444-444444444444', 'c1', 'outside', 3, 'ALPHA-3')",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "UPDATE issues SET parent_id = '22222222-2222-2222-2222-222222222222' WHERE id = '33333333-3333-3333-3333-333333333333'",
        (),
    )
    .await
    .unwrap();

    // Watchdog run scoped to i1.
    let (_, run) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/heartbeat-runs",
        json!({
            "agentId": "11111111-1111-1111-1111-111111111111",
            "invocationSource": "scheduler",
            "contextSnapshot": { "kind": "task_watchdog", "watchedIssueId": "22222222-2222-2222-2222-222222222222" }
        }),
    )
    .await;
    let run_id = run["id"].as_str().unwrap().to_owned();

    // Child in subtree -> allowed.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/heartbeat-runs/{run_id}/watchdog-actions"),
        json!({ "issueId": "33333333-3333-3333-3333-333333333333", "action": "update_status" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["allowed"], true);

    // Outside the subtree -> 403.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/heartbeat-runs/{run_id}/watchdog-actions"),
        json!({ "issueId": "44444444-4444-4444-4444-444444444444", "action": "update_status" }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(body["error"].as_str().unwrap().contains("not authorized"));
}

#[tokio::test]
async fn cost_event_and_budget_hard_stop_flow() {
    let (state, db) = test_state_with_db().await;
    let app = router(state);
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes, budget_monthly_cents)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024, 100)",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO agents (id, company_id, name, role, adapter_type)
         VALUES ('11111111-1111-1111-1111-111111111111', 'c1', 'one', 'engineer', 'codex_local'),
                ('22222222-2222-2222-2222-222222222222', 'c1', 'two', 'engineer', 'codex_local')",
        (),
    )
    .await
    .unwrap();

    // Record a cost event -> 201 with event + hardStop info.
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/cost-events",
        json!({
            "agentId": "11111111-1111-1111-1111-111111111111",
            "provider": "anthropic",
            "model": "claude",
            "inputTokens": 100,
            "outputTokens": 50,
            "costCents": 60,
            "occurredAt": "2026-08-03T00:00:00.000Z"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["event"]["costCents"], 60);
    assert_eq!(body["event"]["provider"], "anthropic");
    // 60 < 100: no hard stop yet.
    assert_eq!(body["hardStop"]["triggered"], false);

    // Summary.
    let (status, summary) = send_json(
        &app,
        Method::GET,
        "/api/companies/c1/costs/summary",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(summary["spentMonthlyCents"], 60);
    assert_eq!(summary["remainingCents"], 40);
    assert_eq!(summary["pausedAgents"], 0);

    // by-agent.
    let (status, by_agent) = send_json(
        &app,
        Method::GET,
        "/api/companies/c1/costs/by-agent",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let spent = by_agent
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["agentId"] == "11111111-1111-1111-1111-111111111111")
        .expect("agent row");
    assert_eq!(spent["spentMonthlyCents"], 60);

    // Exhaust the company budget -> both agents paused.
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/cost-events",
        json!({
            "agentId": "22222222-2222-2222-2222-222222222222",
            "provider": "anthropic",
            "model": "claude",
            "costCents": 60,
            "occurredAt": "2026-08-03T00:00:01.000Z"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["hardStop"]["triggered"], true);
    let paused_ids: Vec<&str> = body["hardStop"]["pausedAgentIds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    assert!(paused_ids.contains(&"11111111-1111-1111-1111-111111111111"));
    assert!(paused_ids.contains(&"22222222-2222-2222-2222-222222222222"));

    // Reset: spending zeroed, agents resumed.
    let (status, reset) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/budget/reset",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(reset["resumedAgents"], 2);

    let (_, summary) = send_json(
        &app,
        Method::GET,
        "/api/companies/c1/costs/summary",
        json!({}),
    )
    .await;
    assert_eq!(summary["spentMonthlyCents"], 0);
    assert_eq!(summary["pausedAgents"], 0);

    // Set budget.
    let (status, summary) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/budget",
        json!({ "budgetMonthlyCents": 500 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(summary["budgetMonthlyCents"], 500);
}

#[tokio::test]
async fn cost_event_validation_and_404() {
    let (state, db) = test_state_with_db().await;
    let app = router(state);
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
        (),
    )
    .await
    .unwrap();

    // Missing agent -> 422.
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/cost-events",
        json!({
            "agentId": "not-a-uuid",
            "provider": "p",
            "model": "m",
            "costCents": -1,
            "occurredAt": "2026-08-03T00:00:00.000Z"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "Validation error");

    // Unknown company -> 404.
    let (status, _) = send_json(
        &app,
        Method::GET,
        "/api/companies/missing/costs/summary",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn approval_state_machine_and_gate() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;

    // Create approval -> 201 pending.
    let (status, created) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/approvals"),
        json!({
            "type": "budget_override_required",
            "requestedByUserId": "u1",
            "payload": { "budgetMonthlyCents": 500 }
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["status"], "pending");
    assert_eq!(created["type"], "budget_override_required");
    let approval_id = created["id"].as_str().unwrap().to_owned();

    // List.
    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/approvals"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Approve -> applies the budget override (approval gate).
    let (status, decided) = send_json(
        &app,
        Method::POST,
        &format!("/api/approvals/{approval_id}/decide"),
        json!({ "decision": "approved", "decisionNote": "ok", "decidedByUserId": "board" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(decided["status"], "approved");
    assert!(decided["decidedAt"].is_string());

    let (_, summary) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/costs/summary"),
        json!({}),
    )
    .await;
    assert_eq!(summary["budgetMonthlyCents"], 500);

    // Decide again -> 409.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/approvals/{approval_id}/decide"),
        json!({ "decision": "rejected" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Cancel a second approval.
    let (_, second) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/approvals"),
        json!({ "type": "hire_agent", "payload": {} }),
    )
    .await;
    let second_id = second["id"].as_str().unwrap().to_owned();
    let (status, cancelled) = send_json(
        &app,
        Method::POST,
        &format!("/api/approvals/{second_id}/cancel"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cancelled["status"], "cancelled");

    // Invalid type -> 422.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/approvals"),
        json!({ "type": "bogus" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn mutating_apis_write_audit_log() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;

    // A few mutations across entities.
    let (_, goal) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/goals"),
        json!({ "title": "G" }),
    )
    .await;
    let goal_id = goal["id"].as_str().unwrap().to_owned();
    let (_, project) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/projects"),
        json!({ "name": "P" }),
    )
    .await;
    let project_id = project["id"].as_str().unwrap().to_owned();
    let issue_id = create_issue_via(&app, &company_id, "T").await;
    send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/comments"),
        json!({ "body": "hello" }),
    )
    .await;

    // The audit trail must contain all of them.
    let (status, activity) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/activity"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let actions: Vec<&str> = activity
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["action"].as_str().unwrap())
        .collect();
    for expected in [
        "company.created",
        "goal.created",
        "project.created",
        "issue.created",
        "comment.created",
    ] {
        assert!(
            actions.contains(&expected),
            "missing audit entry {expected}: {actions:?}"
        );
    }

    // Entity ids match.
    let goal_entry = activity
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["action"] == "goal.created")
        .unwrap();
    assert_eq!(goal_entry["entityId"], goal_id);
    let project_entry = activity
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["action"] == "project.created")
        .unwrap();
    assert_eq!(project_entry["entityId"], project_id);
    assert!(
        activity
            .as_array()
            .unwrap()
            .iter()
            .all(|entry| entry["companyId"] == company_id)
    );
}

#[tokio::test]
async fn secret_lifecycle_rotation_rollback_and_redaction() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;

    // Create -> 201, version 1.
    let (status, created) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/secrets"),
        json!({ "name": "github_token", "value": "ghp_v1_secret" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["name"], "github_token");
    assert_eq!(created["latestVersion"], 1);
    assert_eq!(created["provider"], "local_encrypted");

    // Duplicate -> 409.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/secrets"),
        json!({ "name": "github_token", "value": "x" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Value readback.
    let (status, value) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/secrets/github_token/value"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["value"], "ghp_v1_secret");

    // Rotate -> v2.
    let (status, rotated) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/secrets/github_token/rotate"),
        json!({ "value": "ghp_v2_secret" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(rotated["latestVersion"], 2);
    let (_, value) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/secrets/github_token/value"),
        json!({}),
    )
    .await;
    assert_eq!(value["value"], "ghp_v2_secret");

    // Versions.
    let (status, versions) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/secrets/github_token/versions"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(versions.as_array().unwrap().len(), 2);

    // Rollback to v1 -> value is v1 again, version 3.
    let (status, rolled) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/secrets/github_token/rollback"),
        json!({ "version": 1 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(rolled["latestVersion"], 3);
    let (_, value) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/secrets/github_token/value"),
        json!({}),
    )
    .await;
    assert_eq!(value["value"], "ghp_v1_secret");

    // Redact: transcript containing the value is masked.
    let (status, redacted) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/redact"),
        json!({
            "text": "agent output: ghp_v1_secret was used",
            "names": ["github_token"]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        !redacted["redacted"]
            .as_str()
            .unwrap()
            .contains("ghp_v1_secret")
    );
    assert!(redacted["redacted"].as_str().unwrap().contains("***"));

    // List.
    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/secrets"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);
    // Values never leak in list responses.
    assert!(!list.to_string().contains("ghp_v1_secret"));

    // Delete -> 204, then 404.
    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/api/companies/{company_id}/secrets/github_token"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/secrets/github_token"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn secret_validation_and_audit() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;

    // Empty name/value -> 422.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/secrets"),
        json!({ "name": "", "value": "" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // A successful creation is audited.
    send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/secrets"),
        json!({ "name": "aws_key", "value": "AKIA123" }),
    )
    .await;

    // Audit entries exist.
    let (status, activity) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/activity"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let actions: Vec<&str> = activity
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["action"].as_str().unwrap())
        .collect();
    assert!(actions.contains(&"secret.created"));
}

async fn send_with_auth(
    router: &Router,
    method: Method,
    path: &str,
    body: Option<&str>,
    bearer: Option<&str>,
) -> (StatusCode, String) {
    let mut builder = Request::builder().method(method).uri(path);
    if body.is_some() {
        builder = builder.header(CONTENT_TYPE, "application/json");
    }
    if let Some(token) = bearer {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }
    let request = builder
        .body(Body::from(body.unwrap_or_default().to_owned()))
        .unwrap();
    let response = router.handle(request).await;
    let status = response.status();
    let (_, response_body) = response.into_parts();
    let bytes = to_bytes(response_body, usize::MAX).await.unwrap();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

#[tokio::test]
async fn three_identity_permission_matrix() {
    let (state, db) = test_state_with_db().await;
    let app = router(state);
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024), ('c2', 'Beta', 'BETA', 1024)",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO agents (id, company_id, name, role, adapter_type)
         VALUES ('11111111-1111-1111-1111-111111111111', 'c1', 'one', 'engineer', 'codex_local')",
        (),
    )
    .await
    .unwrap();

    // Board creates an API key for the agent.
    let (status, key_body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/agent-api-keys",
        json!({ "agentId": "11111111-1111-1111-1111-111111111111", "name": "dev" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let plaintext = key_body["plaintext"].as_str().unwrap().to_owned();
    assert!(plaintext.starts_with("sk-"));
    // Hash stored, not plaintext.
    assert_ne!(key_body["key"]["keyHash"], plaintext);

    // 1. Unauthenticated: invalid bearer -> 401 JSON.
    let (status, body) = send_with_auth(
        &app,
        Method::GET,
        "/api/companies/c1/issues",
        None,
        Some("sk-invalid"),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(body.contains("Invalid API key"));

    // 2. Agent: own company allowed.
    let (status, _) = send_with_auth(
        &app,
        Method::GET,
        "/api/companies/c1/issues",
        None,
        Some(&plaintext),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 3. Agent: cross-company access -> 403.
    let (status, body) = send_with_auth(
        &app,
        Method::GET,
        "/api/companies/c2/issues",
        None,
        Some(&plaintext),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(body.contains("cannot access another company"));

    // 4. Agent: board-only action (create company) -> 403.
    let (status, _) = send_with_auth(
        &app,
        Method::POST,
        "/api/companies",
        Some(r#"{"name":"Sneaky"}"#),
        Some(&plaintext),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // 5. Agent: board-only budget action -> 403.
    let (status, _) = send_with_auth(
        &app,
        Method::POST,
        "/api/companies/c1/budget",
        Some(r#"{"budgetMonthlyCents":9999}"#),
        Some(&plaintext),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // 6. Board: everything still works.
    let (status, _) = send_json(&app, Method::GET, "/api/companies/c1/issues", json!({})).await;
    assert_eq!(status, StatusCode::OK);

    // 7. Revoke -> agent key becomes invalid (401).
    let key_id = key_body["key"]["id"].as_str().unwrap().to_owned();
    send_json(
        &app,
        Method::POST,
        &format!("/api/agent-api-keys/{key_id}/revoke"),
        json!({}),
    )
    .await;
    let (status, _) = send_with_auth(
        &app,
        Method::GET,
        "/api/companies/c1/issues",
        None,
        Some(&plaintext),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn decision_desk_and_inbox_and_external_objects() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let issue_id = create_issue_via(&app, &company_id, "T").await;

    // Decision queue.
    let (status, queue) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/decision-queues"),
        json!({ "name": "approvals", "retentionDays": 30 }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(queue["name"], "approvals");
    let queue_id = queue["id"].as_str().unwrap().to_owned();

    // Duplicate queue -> 409.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/decision-queues"),
        json!({ "name": "approvals" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Queue item.
    let (status, item) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/decision-queues/{queue_id}/items"),
        json!({ "sourceKind": "issue", "sourceId": issue_id, "payload": { "n": 1 } }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(item["sourceId"], issue_id);
    let (status, items) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/decision-queues/{queue_id}/items"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(items.as_array().unwrap().len(), 1);

    // Triage.
    let (status, triage) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/decision-triage"),
        json!({ "sourceKind": "issue", "sourceId": issue_id, "decision": "keep", "decidedByUserId": "board" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(triage["decision"], "keep");

    // Inbox: issue present; archive -> gone; unarchive -> back.
    let (status, inbox) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/inbox"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(inbox.as_array().unwrap().len(), 1);

    let (status, archived) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/archive"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(archived["hiddenAt"].is_string());

    let (_, inbox) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/inbox"),
        json!({}),
    )
    .await;
    assert_eq!(inbox.as_array().unwrap().len(), 0);

    let (_, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/unarchive"),
        json!({}),
    )
    .await;
    let (_, inbox) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/inbox"),
        json!({}),
    )
    .await;
    assert_eq!(inbox.as_array().unwrap().len(), 1);

    // External objects.
    let (status, external) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/external-objects"),
        json!({ "kind": "github_pr", "externalId": "42", "url": "https://github.com/x/y/pull/42" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(external["status"], "pending");
    let external_id = external["id"].as_str().unwrap().to_owned();

    let (status, refreshed) = send_json(
        &app,
        Method::POST,
        &format!("/api/external-objects/{external_id}/refresh"),
        json!({ "status": "merged" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(refreshed["status"], "merged");
    assert!(refreshed["lastSyncedAt"].is_string());

    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_id}/external-objects"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);
}
