//! Shared workspace concurrency (issue #206) phase A integration tests:
//! `executionWorkspacePolicy` / `executionWorkspaceSettings` round-trip through
//! the project and issue APIs (create → read back → patch → clear), plus
//! resolution-priority unit coverage for `workspace_policy`.

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

async fn send_json(
    router: &Router,
    method: Method,
    path: &str,
    body: Value,
) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
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

#[tokio::test]
async fn project_execution_workspace_policy_round_trip_and_clear() {
    let (state, _db) = test_state().await;
    let app = router(state);
    let company_id = {
        let (status, body) = send_json(
            &app,
            Method::POST,
            "/api/companies",
            json!({ "name": "Alpha" }),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "body: {body}");
        body["id"].as_str().unwrap().to_owned()
    };

    let policy = json!({ "enabled": true, "sharedWorkspaceConcurrency": "serialize" });
    let (status, created) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/projects"),
        json!({ "name": "Ship", "executionWorkspacePolicy": policy }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {created}");
    assert_eq!(created["executionWorkspacePolicy"], policy);
    let project_id = created["id"].as_str().unwrap().to_owned();

    // GET read-back is consistent.
    let (status, fetched) = send_json(
        &app,
        Method::GET,
        &format!("/api/projects/{project_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {fetched}");
    assert_eq!(fetched["executionWorkspacePolicy"], policy);

    // PATCH updates the policy.
    let next = json!({ "enabled": true, "sharedWorkspaceConcurrency": "allow" });
    let (status, updated) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/projects/{project_id}"),
        json!({ "executionWorkspacePolicy": next }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {updated}");
    assert_eq!(updated["executionWorkspacePolicy"], next);

    // PATCH with null clears it.
    let (status, cleared) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/projects/{project_id}"),
        json!({ "executionWorkspacePolicy": null }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {cleared}");
    assert!(
        cleared["executionWorkspacePolicy"].is_null(),
        "body: {cleared}"
    );

    let (status, after_clear) = send_json(
        &app,
        Method::GET,
        &format!("/api/projects/{project_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(after_clear["executionWorkspacePolicy"].is_null());
}

#[tokio::test]
async fn issue_execution_workspace_settings_round_trip() {
    let (state, _db) = test_state().await;
    let app = router(state);
    let company_id = {
        let (status, body) = send_json(
            &app,
            Method::POST,
            "/api/companies",
            json!({ "name": "Beta" }),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "body: {body}");
        body["id"].as_str().unwrap().to_owned()
    };

    let settings = json!({ "sharedWorkspaceConcurrency": "allow" });
    let (status, created) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/issues"),
        json!({ "title": "Concurrent task", "executionWorkspaceSettings": settings }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {created}");
    assert_eq!(created["executionWorkspaceSettings"], settings);
    let issue_id = created["id"].as_str().unwrap().to_owned();

    // GET read-back is consistent.
    let (status, fetched) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {fetched}");
    assert_eq!(fetched["executionWorkspaceSettings"], settings);

    // PATCH updates the settings.
    let next = json!({ "sharedWorkspaceConcurrency": "serialize" });
    let (status, updated) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/issues/{issue_id}"),
        json!({ "executionWorkspaceSettings": next }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {updated}");
    assert_eq!(updated["executionWorkspaceSettings"], next);

    // PATCH with null clears it.
    let (status, cleared) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/issues/{issue_id}"),
        json!({ "executionWorkspaceSettings": null }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {cleared}");
    assert!(
        cleared["executionWorkspaceSettings"].is_null(),
        "body: {cleared}"
    );

    let (status, after_clear) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(after_clear["executionWorkspaceSettings"].is_null());
}

#[test]
fn resolve_shared_workspace_concurrency_priority() {
    use staple_app::workspace_policy::{
        SharedWorkspaceConcurrency, resolve_shared_workspace_concurrency,
    };

    // Issue settings win over project policy.
    let issue = r#"{"sharedWorkspaceConcurrency":"allow"}"#;
    let project = json!({ "enabled": true, "sharedWorkspaceConcurrency": "serialize" });
    assert_eq!(
        resolve_shared_workspace_concurrency(Some(issue), Some(&project)),
        Some(SharedWorkspaceConcurrency::Allow)
    );

    // Project policy applies only when enabled.
    assert_eq!(
        resolve_shared_workspace_concurrency(None, Some(&project)),
        Some(SharedWorkspaceConcurrency::Serialize)
    );
    let disabled = json!({ "enabled": false, "sharedWorkspaceConcurrency": "serialize" });
    assert_eq!(
        resolve_shared_workspace_concurrency(None, Some(&disabled)),
        None
    );

    // Default (auto semantics is handled by the caller).
    assert_eq!(resolve_shared_workspace_concurrency(None, None), None);
    assert_eq!(
        resolve_shared_workspace_concurrency(Some("{}"), Some(&json!({ "enabled": true }))),
        None
    );
}
