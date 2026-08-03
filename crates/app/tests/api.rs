//! API integration tests: health endpoint, unified JSON error handling, and
//! company CRUD.

use std::sync::Arc;

use http::{Method, Request, header::CONTENT_TYPE};
use serde_json::{Value, json};
use staple_app::router;
use staple_app::state::AppState;
use staple_app::storage::LocalStorage;
use staple_data::{
    DbConfig, TursoAssetRepository, TursoCompanyRepository, TursoDocumentRepository,
    TursoGoalRepository, TursoIssueCommentRepository, TursoIssueRelationRepository,
    TursoIssueRepository, TursoProjectRepository, TursoWorkProductRepository, migrate, open,
};
use topcoat::router::{Body, Router, StatusCode, to_bytes};

async fn test_state() -> AppState {
    let dir = tempfile::tempdir().unwrap();
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
    migrate(&companies_db).await.unwrap();
    let uploads = dir.path().join("uploads");
    // Keep the temp dir alive for the lifetime of the test process.
    std::mem::forget(dir);
    AppState {
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
