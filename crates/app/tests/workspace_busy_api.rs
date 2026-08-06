//! Shared-workspace busy gate (issue #206 phase B) API integration tests.

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

async fn seed_busy_workspace_fixture(state: &AppState) -> (String, String, String, String, String) {
    let company = state
        .companies
        .create(staple_data::NewCompany {
            name: "Busy Co".to_owned(),
            description: Some("busy".to_owned()),
            budget_monthly_cents: 100_000,
            attachment_max_bytes: 1024,
        })
        .await
        .unwrap();
    let agent = state
        .agents
        .create(staple_data::NewAgent {
            company_id: company.id.clone(),
            name: "worker".to_owned(),
            role: "worker".to_owned(),
            title: None,
            icon: None,
            reports_to: None,
            adapter_type: "cli".to_owned(),
            budget_monthly_cents: 0,
        })
        .await
        .unwrap();
    let project = state
        .projects
        .create(staple_data::NewProject {
            company_id: company.id.clone(),
            goal_id: None,
            name: "ship".to_owned(),
            description: None,
            status: "backlog".to_owned(),
            lead_agent_id: None,
            target_date: None,
            env: None,
            execution_workspace_policy: Some(serde_json::json!({
                "enabled": true,
                "sharedWorkspaceConcurrency": "serialize",
            })),
        })
        .await
        .unwrap();
    let workspace = state
        .workspaces
        .create_project_workspace(staple_data::NewProjectWorkspace {
            company_id: company.id.clone(),
            project_id: project.id.clone(),
            name: "shared".to_owned(),
            cwd: None,
            repo_url: None,
            is_primary: false,
            shared_workspace_key: Some("team-ws".to_owned()),
        })
        .await
        .unwrap();
    let execution_workspace = state
        .workspaces
        .create_execution_workspace(staple_data::NewExecutionWorkspace {
            company_id: company.id.clone(),
            project_id: project.id.clone(),
            project_workspace_id: Some(workspace.id.clone()),
            source_issue_id: None,
            mode: "reuse_existing".to_owned(),
            strategy_type: "shared".to_owned(),
            name: "exec".to_owned(),
            cwd: None,
            repo_url: None,
        })
        .await
        .unwrap();
    let issue = state
        .issues
        .create(staple_data::NewIssue {
            company_id: company.id.clone(),
            project_id: Some(project.id.clone()),
            goal_id: None,
            parent_id: None,
            title: "Busy issue".to_owned(),
            description: None,
            status: Some("todo".to_owned()),
            priority: Some("high".to_owned()),
            assignee_agent_id: Some(agent.id.clone()),
            assignee_user_id: None,
            created_by_user_id: None,
            work_mode: None,
            billing_code: None,
            execution_workspace_settings: None,
        })
        .await
        .unwrap();
    // Active operation occupies the shared workspace.
    state
        .workspaces
        .create_operation(staple_data::NewWorkspaceOperation {
            company_id: company.id.clone(),
            execution_workspace_id: Some(execution_workspace.id.clone()),
            heartbeat_run_id: None,
            issue_id: Some(issue.id.clone()),
            phase: "run".to_owned(),
            command: None,
            log_ref: None,
        })
        .await
        .unwrap();
    (company.id, agent.id, project.id, issue.id, workspace.id)
}

#[tokio::test]
async fn start_run_defers_when_shared_workspace_busy_under_serialize() {
    let (state, _db) = test_state().await;
    let app = router(state.clone());
    let (company_id, agent_id, _project_id, issue_id, _workspace_id) =
        seed_busy_workspace_fixture(&state).await;

    // Busy + serialize -> 409 deferred.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/heartbeat-runs"),
        json!({ "agentId": agent_id, "issueId": issue_id }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "body: {body}");
    assert!(body["error"].as_str().unwrap().contains("busy"));

    // A retry wakeup was enqueued for the agent.
    let wakeups = state.agent_runtime.wakeup_list(&company_id).await.unwrap();
    assert!(
        wakeups.iter().any(|w| w.source == "workspace_busy"),
        "expected a workspace_busy retry wakeup"
    );
}

#[tokio::test]
async fn start_run_proceeds_when_workspace_no_longer_busy_or_policy_allow() {
    let (state, db) = test_state().await;
    let app = router(state.clone());
    let (company_id, agent_id, _project_id, issue_id, workspace_id) =
        seed_busy_workspace_fixture(&state).await;

    // Clear the busy operation.
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "UPDATE workspace_operations SET status = 'succeeded' WHERE company_id = ?1",
        [company_id.clone()],
    )
    .await
    .unwrap();

    // Now the run starts.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/heartbeat-runs"),
        json!({ "agentId": agent_id, "issueId": issue_id }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    assert_eq!(body["status"], "running");

    // Re-busy the workspace (new active operation) under an allow policy.
    let allow_project = state
        .projects
        .create(staple_data::NewProject {
            company_id: company_id.clone(),
            goal_id: None,
            name: "allow".to_owned(),
            description: None,
            status: "backlog".to_owned(),
            lead_agent_id: None,
            target_date: None,
            env: None,
            execution_workspace_policy: Some(serde_json::json!({
                "enabled": true,
                "sharedWorkspaceConcurrency": "allow",
            })),
        })
        .await
        .unwrap();
    let allow_ws = state
        .workspaces
        .create_project_workspace(staple_data::NewProjectWorkspace {
            company_id: company_id.clone(),
            project_id: allow_project.id.clone(),
            name: "shared-allow".to_owned(),
            cwd: None,
            repo_url: None,
            is_primary: false,
            shared_workspace_key: Some("allow-ws".to_owned()),
        })
        .await
        .unwrap();
    let allow_exec = state
        .workspaces
        .create_execution_workspace(staple_data::NewExecutionWorkspace {
            company_id: company_id.clone(),
            project_id: allow_project.id.clone(),
            project_workspace_id: Some(allow_ws.id.clone()),
            source_issue_id: None,
            mode: "reuse_existing".to_owned(),
            strategy_type: "shared".to_owned(),
            name: "exec-allow".to_owned(),
            cwd: None,
            repo_url: None,
        })
        .await
        .unwrap();
    let allow_issue = state
        .issues
        .create(staple_data::NewIssue {
            company_id: company_id.clone(),
            project_id: Some(allow_project.id.clone()),
            goal_id: None,
            parent_id: None,
            title: "Allow issue".to_owned(),
            description: None,
            status: Some("todo".to_owned()),
            priority: Some("high".to_owned()),
            assignee_agent_id: Some(agent_id.clone()),
            assignee_user_id: None,
            created_by_user_id: None,
            work_mode: None,
            billing_code: None,
            execution_workspace_settings: None,
        })
        .await
        .unwrap();
    state
        .workspaces
        .create_operation(staple_data::NewWorkspaceOperation {
            company_id: company_id.clone(),
            execution_workspace_id: Some(allow_exec.id.clone()),
            heartbeat_run_id: None,
            issue_id: Some(allow_issue.id.clone()),
            phase: "run".to_owned(),
            command: None,
            log_ref: None,
        })
        .await
        .unwrap();
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/heartbeat-runs"),
        json!({ "agentId": agent_id, "issueId": allow_issue.id }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "allow should start: {body}");
    let _ = workspace_id;
}

#[tokio::test]
async fn scheduler_retries_busy_wakeup_with_bounded_attempts() {
    let (state, db) = test_state().await;
    let app = router(state.clone());
    let (company_id, agent_id, _project_id, issue_id, _workspace_id) =
        seed_busy_workspace_fixture(&state).await;

    // Enqueue attempt 1 retry (as start_run would).
    crate_scheduler_enqueue(&state, &company_id, &agent_id, &issue_id, 1).await;

    // First tick: workspace still busy -> re-enqueues attempt 2.
    let mut last_sweep = None;
    let config = staple_app::scheduler::SchedulerConfig {
        tick: std::time::Duration::from_secs(60),
        wakeup_batch: 10,
        sweep_interval_days: 1,
    };
    staple_app::scheduler::tick(&state, &config, &mut last_sweep)
        .await
        .unwrap();
    let wakeups = state.agent_runtime.wakeup_list(&company_id).await.unwrap();
    let retries: Vec<_> = wakeups
        .iter()
        .filter(|w| w.source == "workspace_busy" && w.status == "queued")
        .collect();
    assert_eq!(retries.len(), 1, "one queued retry expected");
    let attempt = retries[0]
        .payload
        .as_ref()
        .unwrap()
        .get("attempt")
        .and_then(serde_json::Value::as_u64)
        .unwrap();
    assert_eq!(attempt, 2, "bounded retry advanced to attempt 2");

    // Make the workspace free; a tick should start the run.
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "UPDATE workspace_operations SET status = 'succeeded' WHERE company_id = ?1",
        [company_id.clone()],
    )
    .await
    .unwrap();
    staple_app::scheduler::tick(&state, &config, &mut last_sweep)
        .await
        .unwrap();
    let runs = state.heartbeat.list(&company_id, None, 100).await.unwrap();
    assert!(
        runs.iter().any(|r| r.status == "running"),
        "retried wakeup should start a run"
    );
    let _ = app;
}

async fn crate_scheduler_enqueue(
    state: &AppState,
    company_id: &str,
    agent_id: &str,
    issue_id: &str,
    attempt: u64,
) {
    state
        .agent_runtime
        .wakeup_enqueue(staple_data::NewWakeupRequest {
            company_id: company_id.to_owned(),
            agent_id: agent_id.to_owned(),
            source: "workspace_busy".to_owned(),
            trigger_detail: Some(format!("workspace busy retry #{attempt}")),
            reason: Some("Shared workspace is busy; retrying".to_owned()),
            payload: Some(serde_json::json!({ "issueId": issue_id, "attempt": attempt })),
            requested_by_actor_type: Some("board".to_owned()),
            requested_by_actor_id: None,
            idempotency_key: Some(format!(
                "workspace-busy:{company_id}:{issue_id}:{agent_id}:{attempt}"
            )),
        })
        .await
        .unwrap();
}
