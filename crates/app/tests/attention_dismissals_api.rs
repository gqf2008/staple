//! Attention inbox dismissal API integration tests (issue #204 A2):
//! dismiss/snooze upserts, list, restore (DELETE), validation, company/user
//! isolation, board-only auth, and audit logging.

use std::sync::Arc;

use http::{Method, Request, header::CONTENT_TYPE};
use serde_json::{Value, json};
use staple_adapters::{AdapterRegistry, CliAdapter, CliAdapterConfig};
use staple_app::router;
use staple_app::state::AppState;
use staple_app::storage::LocalStorage;
use staple_data::{
    DbConfig, SecretCipher, TursoActivityRepository, TursoAgentRepository,
    TursoAgentRuntimeRepository, TursoApiKeyRepository, TursoApprovalRepository,
    TursoAssetRepository, TursoAttentionDismissalRepository, TursoBoardKeyRepository,
    TursoBudgetPolicyRepository, TursoCaseRepository, TursoCompanyRepository, TursoCostRepository,
    TursoDecisionActionRepository, TursoDecisionRepository, TursoDocumentRepository,
    TursoEnvironmentRepository, TursoExternalObjectCatalogRepository,
    TursoExternalObjectRepository, TursoGoalRepository, TursoHeartbeatRepository,
    TursoInfrastructureRepository, TursoInstructionRepository, TursoInviteRepository,
    TursoIssueCommentRepository, TursoIssueRelationRepository, TursoIssueRepository,
    TursoIssueStructureRepository, TursoLabelRepository, TursoMembershipRepository,
    TursoPermissionGrantRepository, TursoPipelineRepository, TursoPluginRepository,
    TursoPluginRuntimeRepository, TursoPortabilityRepository, TursoPreferenceRepository,
    TursoProjectRepository, TursoRoutineRepository, TursoScatteredRepository,
    TursoSecretBindingRepository, TursoSecretRepository, TursoSkillCatalogRepository,
    TursoSkillRepository, TursoToolCatalogRepository, TursoToolConnectionRepository,
    TursoToolGatewayRepository, TursoWorkProductRepository, TursoWorkspaceRepository, migrate,
    open,
};
use topcoat::router::{Body, Router, StatusCode, to_bytes};

async fn send(
    router: &Router,
    method: Method,
    path: &str,
    body: Value,
    user: Option<&str>,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(path);
    if !body.is_null() {
        builder = builder.header(CONTENT_TYPE, "application/json");
    }
    if let Some(user) = user {
        builder = builder.header("X-Board-User", user);
    }
    if let Some(token) = bearer {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }
    let request = builder.body(Body::from(body.to_string())).unwrap();
    let response = router.handle(request).await;
    let status = response.status();
    let (_, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

async fn test_state() -> (AppState, staple_data::Database) {
    let dir = tempfile::tempdir().unwrap();
    let seed_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let companies_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let api_keys_db = open(&DbConfig::local(dir.path().join("test.db")))
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
    let cipher = SecretCipher::load_or_create(dir.path().join("key")).unwrap();
    migrate(&seed_db).await.unwrap();

    let state = AppState {
        companies: Arc::new(TursoCompanyRepository::new(companies_db)),
        agents: Arc::new(TursoAgentRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        attention_dismissals: Arc::new(TursoAttentionDismissalRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        agent_runtime: Arc::new(TursoAgentRuntimeRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        permission_grants: Arc::new(TursoPermissionGrantRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        memberships: Arc::new(TursoMembershipRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        board_claim: Arc::new(staple_app::board_claim::BoardClaimManager::new()),
        invites: Arc::new(TursoInviteRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        infrastructure: Arc::new(TursoInfrastructureRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        instructions: Arc::new(TursoInstructionRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        board_keys: Arc::new(TursoBoardKeyRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        budget_policies: Arc::new(TursoBudgetPolicyRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        cases: Arc::new(TursoCaseRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        preferences: Arc::new(TursoPreferenceRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        pipelines: Arc::new(TursoPipelineRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        portability: Arc::new(TursoPortabilityRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        plugins: Arc::new(TursoPluginRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        plugin_runtime: Arc::new(TursoPluginRuntimeRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        goals: Arc::new(TursoGoalRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        projects: Arc::new(TursoProjectRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        issues: Arc::new(TursoIssueRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        comments: Arc::new(TursoIssueCommentRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        documents: Arc::new(TursoDocumentRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        assets: Arc::new(TursoAssetRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        relations: Arc::new(TursoIssueRelationRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        storage: LocalStorage::new(dir.path().join("uploads")),
        work_products: Arc::new(TursoWorkProductRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        heartbeat: Arc::new(TursoHeartbeatRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        costs: Arc::new(TursoCostRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        approvals: Arc::new(TursoApprovalRepository::new(approvals_db)),
        activity: Arc::new(TursoActivityRepository::new(activity_db)),
        secrets: Arc::new(TursoSecretRepository::new(secrets_db, cipher)),
        api_keys: Arc::new(TursoApiKeyRepository::new(api_keys_db)),
        decisions: Arc::new(TursoDecisionRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        decision_actions: Arc::new(TursoDecisionActionRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        external_objects: Arc::new(TursoExternalObjectRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        external_object_catalog: Arc::new(TursoExternalObjectCatalogRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        skill_catalog: Arc::new(TursoSkillCatalogRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        secret_bindings: Arc::new(TursoSecretBindingRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        skills: Arc::new(TursoSkillRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        environments: Arc::new(TursoEnvironmentRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        labels: Arc::new(TursoLabelRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        issue_structure: Arc::new(TursoIssueStructureRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        routines: Arc::new(TursoRoutineRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        tool_catalog: Arc::new(TursoToolCatalogRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        tool_connections: Arc::new(TursoToolConnectionRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        tool_gateway: Arc::new(TursoToolGatewayRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        scattered: Arc::new(TursoScatteredRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        workspaces: Arc::new(TursoWorkspaceRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        adapters: Arc::new({
            let mut registry = AdapterRegistry::new();
            registry.register(Box::new(CliAdapter::new(CliAdapterConfig::default())));
            registry
        }),
        plugin_reports: Vec::new(),
    };
    // Keep the temp dir alive for the lifetime of the test process.
    std::mem::forget(dir);
    (state, seed_db)
}

async fn seed_companies_and_agent(db: &staple_data::Database) -> (String, String) {
    let conn = staple_data::connect(db).await.unwrap();
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
    (
        "c1".to_owned(),
        "11111111-1111-1111-1111-111111111111".to_owned(),
    )
}

async fn create_agent_key(app: &Router, company_id: &str, agent_id: &str) -> String {
    let (status, body) = send(
        app,
        Method::POST,
        &format!("/api/companies/{company_id}/agent-api-keys"),
        json!({ "agentId": agent_id, "name": "dev" }),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    body["plaintext"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn dismiss_snooze_list_restore_and_audit() {
    let (state, db) = test_state().await;
    let (company_id, _) = seed_companies_and_agent(&db).await;
    let app = router(state);

    // Dismiss one item as u-1.
    let (status, body) = send(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/inbox-dismissals"),
        json!({ "itemKey": "attention:issue-1", "kind": "dismiss" }),
        Some("u-1"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    assert_eq!(body["kind"], "dismiss");
    assert_eq!(body["itemKey"], "attention:issue-1");
    assert_eq!(body["userId"], "u-1");
    assert!(body["snoozedUntil"].is_null());

    // Snooze another item until 2099 (offset input is normalized to UTC).
    let (status, body) = send(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/inbox-dismissals"),
        json!({
            "itemKey": "attention:issue-2",
            "kind": "snooze",
            "snoozedUntil": "2099-01-01T08:00:00.000+08:00"
        }),
        Some("u-1"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    assert_eq!(body["kind"], "snooze");
    assert_eq!(body["snoozedUntil"], "2099-01-01T00:00:00.000Z");

    // List returns both rows for u-1.
    let (status, body) = send(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/inbox-dismissals"),
        json!({}),
        Some("u-1"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let rows = body.as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().any(|row| row["itemKey"] == "attention:issue-1"));
    assert!(rows.iter().any(|row| row["itemKey"] == "attention:issue-2"));

    // Restore (DELETE) the dismissed item.
    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/api/companies/{company_id}/inbox-dismissals/attention:issue-1"),
        json!({}),
        Some("u-1"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, body) = send(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/inbox-dismissals"),
        json!({}),
        Some("u-1"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body.as_array().unwrap().len(), 1);

    // Audit entries exist for all three mutations.
    let (status, body) = send(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/activity"),
        json!({}),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let actions: Vec<&str> = body
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["action"].as_str().unwrap())
        .collect();
    assert!(actions.contains(&"inbox.dismissed"));
    assert!(actions.contains(&"inbox.snoozed"));
    assert!(actions.contains(&"inbox.restored"));
}

#[tokio::test]
async fn validation_rejects_bad_bodies() {
    let (state, db) = test_state().await;
    let (company_id, _) = seed_companies_and_agent(&db).await;
    let app = router(state);
    let path = format!("/api/companies/{company_id}/inbox-dismissals");

    // Unknown kind.
    let (status, body) = send(
        &app,
        Method::POST,
        &path,
        json!({ "itemKey": "attention:issue-1", "kind": "archive" }),
        Some("u-1"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(body["details"][0]["path"][0], "kind");

    // Empty item key.
    let (status, body) = send(
        &app,
        Method::POST,
        &path,
        json!({ "itemKey": "  ", "kind": "dismiss" }),
        Some("u-1"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(body["details"][0]["path"][0], "itemKey");

    // Snooze without snoozedUntil.
    let (status, body) = send(
        &app,
        Method::POST,
        &path,
        json!({ "itemKey": "attention:issue-1", "kind": "snooze" }),
        Some("u-1"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(body["details"][0]["path"][0], "snoozedUntil");

    // Snooze with a past timestamp.
    let (status, body) = send(
        &app,
        Method::POST,
        &path,
        json!({
            "itemKey": "attention:issue-1",
            "kind": "snooze",
            "snoozedUntil": "2000-01-01T00:00:00.000Z"
        }),
        Some("u-1"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(body["details"][0]["path"][0], "snoozedUntil");

    // Snooze with a non-ISO timestamp.
    let (status, body) = send(
        &app,
        Method::POST,
        &path,
        json!({
            "itemKey": "attention:issue-1",
            "kind": "snooze",
            "snoozedUntil": "tomorrow"
        }),
        Some("u-1"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(body["details"][0]["path"][0], "snoozedUntil");

    // Dismiss must not include snoozedUntil.
    let (status, body) = send(
        &app,
        Method::POST,
        &path,
        json!({
            "itemKey": "attention:issue-1",
            "kind": "dismiss",
            "snoozedUntil": "2099-01-01T00:00:00.000Z"
        }),
        Some("u-1"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(body["details"][0]["path"][0], "snoozedUntil");

    // Nothing was persisted.
    let (status, body) = send(&app, Method::GET, &path, json!({}), Some("u-1"), None).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn company_and_user_isolation_and_permissions() {
    let (state, db) = test_state().await;
    let (company_id, agent_id) = seed_companies_and_agent(&db).await;
    let app = router(state);

    // Dismiss an item for c1/u-1.
    let (status, body) = send(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/inbox-dismissals"),
        json!({ "itemKey": "attention:issue-1", "kind": "dismiss" }),
        Some("u-1"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");

    // Another company's list is empty.
    let (status, body) = send(
        &app,
        Method::GET,
        "/api/companies/c2/inbox-dismissals",
        json!({}),
        Some("u-1"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body.as_array().unwrap().is_empty());

    // Another user's list is empty.
    let (status, body) = send(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/inbox-dismissals"),
        json!({}),
        Some("u-2"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body.as_array().unwrap().is_empty());

    // Restoring as another user does not clear u-1's row.
    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/api/companies/{company_id}/inbox-dismissals/attention:issue-1"),
        json!({}),
        Some("u-2"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, body) = send(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/inbox-dismissals"),
        json!({}),
        Some("u-1"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body.as_array().unwrap().len(), 1);

    // Missing company -> 404.
    let (status, body) = send(
        &app,
        Method::POST,
        "/api/companies/missing/inbox-dismissals",
        json!({ "itemKey": "attention:issue-1", "kind": "dismiss" }),
        Some("u-1"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "body: {body}");
    assert_eq!(body["error"], "Company not found");

    // Agent keys are rejected (board-only) and cannot cross companies.
    let key = create_agent_key(&app, &company_id, &agent_id).await;
    let (status, _) = send(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/inbox-dismissals"),
        json!({}),
        None,
        Some(&key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "board-only route rejects agents"
    );
    let (status, _) = send(
        &app,
        Method::GET,
        "/api/companies/c2/inbox-dismissals",
        json!({}),
        None,
        Some(&key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "agent cannot cross companies"
    );
}

#[tokio::test]
async fn defaults_to_board_user_without_header() {
    let (state, db) = test_state().await;
    let (company_id, _) = seed_companies_and_agent(&db).await;
    let app = router(state);

    let (status, body) = send(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/inbox-dismissals"),
        json!({ "itemKey": "attention:issue-1", "kind": "dismiss" }),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    assert_eq!(body["userId"], "board");

    let (status, body) = send(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/inbox-dismissals"),
        json!({}),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body.as_array().unwrap().len(), 1);
    assert_eq!(body[0]["userId"], "board");
}
