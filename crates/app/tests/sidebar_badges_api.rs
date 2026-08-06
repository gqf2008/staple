//! Sidebar badge counts API + layout badge rendering (issue #217 B1).

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

async fn seed_badges_fixture(state: &AppState, db: &staple_data::Database) -> String {
    let company = state
        .companies
        .create(staple_data::NewCompany {
            name: "Badges Co".to_owned(),
            description: None,
            budget_monthly_cents: 100_000,
            attachment_max_bytes: 1024,
        })
        .await
        .unwrap();
    let agent_a = state
        .agents
        .create(staple_data::NewAgent {
            company_id: company.id.clone(),
            name: "a".to_owned(),
            role: "worker".to_owned(),
            title: None,
            icon: None,
            reports_to: None,
            adapter_type: "cli".to_owned(),
            budget_monthly_cents: 0,
        })
        .await
        .unwrap();
    let agent_b = state
        .agents
        .create(staple_data::NewAgent {
            company_id: company.id.clone(),
            name: "b".to_owned(),
            role: "worker".to_owned(),
            title: None,
            icon: None,
            reports_to: None,
            adapter_type: "cli".to_owned(),
            budget_monthly_cents: 0,
        })
        .await
        .unwrap();
    // Approvals: 2 actionable (pending + revision_requested), 1 approved.
    for (index, _status) in ["pending", "revision_requested", "approved"]
        .iter()
        .enumerate()
    {
        state
            .approvals
            .create(staple_data::NewApproval {
                company_id: company.id.clone(),
                r#type: "request_board_approval".to_owned(),
                requested_by_agent_id: Some(agent_a.id.clone()),
                requested_by_user_id: None,
                payload: format!("{{\"kind\":\"test\",\"index\":{index}}}"),
            })
            .await
            .unwrap();
    }
    let conn = staple_data::connect(db).await.unwrap();
    conn.execute(
        "UPDATE approvals SET status = 'revision_requested' WHERE id IN
         (SELECT id FROM approvals WHERE company_id = ?1 ORDER BY created_at LIMIT 1 OFFSET 1)",
        [company.id.clone()],
    )
    .await
    .unwrap();
    conn.execute(
        "UPDATE approvals SET status = 'approved' WHERE id IN
         (SELECT id FROM approvals WHERE company_id = ?1 ORDER BY created_at LIMIT 1 OFFSET 2)",
        [company.id.clone()],
    )
    .await
    .unwrap();
    // Heartbeat runs: agent A latest failed, agent B latest succeeded.
    for (agent_id, status) in [(&agent_a.id, "failed"), (&agent_b.id, "succeeded")] {
        let run = state
            .heartbeat
            .start(staple_data::NewHeartbeatRun {
                company_id: company.id.clone(),
                agent_id: agent_id.clone(),
                invocation_source: "manual".to_owned(),
                issue_id: None,
                context_snapshot: None,
                trigger_detail: Some("badge".to_owned()),
            })
            .await
            .unwrap();
        state
            .heartbeat
            .complete(
                &run.id,
                staple_data::CompleteHeartbeatRun {
                    status: status.to_owned(),
                    error: if status == "failed" {
                        Some("boom".to_owned())
                    } else {
                        None
                    },
                    error_kind: None,
                },
            )
            .await
            .unwrap();
    }
    // Pending join request.
    let invite = state
        .invites
        .create_invite(staple_data::NewInvite {
            company_id: company.id.clone(),
            invite_type: "company_join".to_owned(),
            allowed_join_types: "human".to_owned(),
            defaults_payload: None,
            expires_at: "2999-01-01T00:00:00.000Z".to_owned(),
            invited_by_user_id: None,
        })
        .await
        .unwrap();
    state
        .invites
        .create_join_request(staple_data::NewJoinRequest {
            invite_id: invite.0.id.clone(),
            company_id: company.id.clone(),
            request_type: "human".to_owned(),
            request_ip: "127.0.0.1".to_owned(),
            requesting_user_id: None,
            request_email_snapshot: Some("p@example.com".to_owned()),
            agent_name: None,
            adapter_type: None,
            capabilities: None,
            agent_defaults_payload: None,
            claim_secret_hash: None,
            claim_secret_expires_at: None,
        })
        .await
        .unwrap();
    company.id
}

#[tokio::test]
async fn sidebar_badges_counts() {
    let (state, db) = test_state().await;
    let app = router(state.clone());
    let company_id = seed_badges_fixture(&state, &db).await;

    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/sidebar-badges"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["approvals"], 2);
    assert_eq!(body["failedRuns"], 1);
    assert_eq!(body["joinRequests"], 1);
    assert_eq!(body["inbox"], 4);
}

#[tokio::test]
async fn sidebar_badges_are_company_scoped() {
    let (state, db) = test_state().await;
    let app = router(state.clone());
    let company_id = seed_badges_fixture(&state, &db).await;
    let other = state
        .companies
        .create(staple_data::NewCompany {
            name: "Other Co".to_owned(),
            description: None,
            budget_monthly_cents: 100_000,
            attachment_max_bytes: 1024,
        })
        .await
        .unwrap();
    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{}/sidebar-badges", other.id),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["approvals"], 0);
    assert_eq!(body["failedRuns"], 0);
    assert_eq!(body["joinRequests"], 0);
    assert_eq!(body["inbox"], 0);
    let _ = company_id;
}

#[tokio::test]
async fn layout_renders_sidebar_badges() {
    let (state, db) = test_state().await;
    let app = router(state.clone());
    let company_id = seed_badges_fixture(&state, &db).await;

    let request = http::Request::builder()
        .method(Method::GET)
        .uri(format!("/companies/{company_id}/inbox"))
        .body(Body::empty())
        .unwrap();
    let response = app.handle(request).await;
    let status = response.status();
    let (_, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&bytes).into_owned();
    assert_eq!(status, StatusCode::OK);
    // inbox badge (4) and approvals badge (2) render in the sidebar.
    assert!(html.contains("badge badge-default"), "badge class present");
    assert!(html.contains(">4<"), "inbox badge count rendered");
    assert!(html.contains(">2<"), "approvals badge count rendered");
}
