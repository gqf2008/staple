//! Staple Turso/libSQL data layer.
//!
//! Owns the connection layer (`connection`), the versioned SQL migrations
//! (`migrations`), and — in later milestones — the repositories.

pub mod connection;
pub mod migrations;
pub mod repositories;
pub mod secrets;
pub mod skills;

pub use connection::{DataError, DbConfig, connect, open};
pub use libsql::{Connection, Database};
pub use migrations::{
    MigrateError, Migration, load_migrations, migrate, migrate_conn, migrate_down,
};
pub use repositories::helpers::sha256_hex;
pub use repositories::{
    ActivityEntry, ActivityError, ActivityRepository, AgentApiKeyRecord, AgentBudgetRecord,
    AgentCostRow, AgentError, AgentHierarchyRow, AgentPrincipal, AgentRecord, AgentRepository,
    AgentRuntimeError, AgentRuntimeRepository, AgentRuntimeStateRecord, AgentTaskSessionRecord,
    AgentWakeupRequestRecord, ApiKeyError, ApiKeyRepository, ApprovalDecision, ApprovalError,
    ApprovalRecord, ApprovalRepository, AssetError, AssetRecord, AssetRepository,
    BoardApiKeyRecord, BoardKeyError, BoardKeyRepository, BudgetIncidentRecord, BudgetPolicyError,
    BudgetPolicyRecord, BudgetPolicyRepository, BudgetSummary, CaseAttachmentRecord,
    CaseDocumentRecord, CaseError, CaseEventRecord, CaseIssueLinkRecord, CaseLabelRecord,
    CasePatch, CaseRecord, CaseRepository, CliAuthChallengeRecord, CompanyLogoRecord,
    CompanyMembershipRecord, CompanyPatch, CompanyRecord, CompanyRepository, CompanySecretRecord,
    CompleteHeartbeatRun, CostError, CostEventOutcome, CostEventRecord, CostRepository,
    DecisionActionError, DecisionActionRepository, DecisionBundleRecord,
    DecisionEffectExecutionRecord, DecisionError, DecisionOutboxRecord, DecisionQueueItemRecord,
    DecisionQueueRecord, DecisionRecord, DecisionRepository, DecisionRetentionRecord,
    DecisionSweepResult, DecisionTargetIssueRecord, DecisionTrainingExampleRecord,
    DecisionTriageEventRecord, DecisionTriageRecord, DocumentError, DocumentRecord,
    DocumentRepository, EnvironmentError, EnvironmentRecord, EnvironmentRepository,
    ExecutionDecisionRecord, ExecutionWorkspaceRecord, ExternalObjectCatalogRecord,
    ExternalObjectCatalogRepository, ExternalObjectError, ExternalObjectMentionRecord,
    ExternalObjectRecord, ExternalObjectRepository, GoalError, GoalPatch, GoalRecord,
    GoalRepository, HeartbeatError, HeartbeatRepository, HeartbeatRunRecord,
    InstanceUserRoleRecord, InviteError, InviteRecord, InviteRepository, IssueApprovalRecord,
    IssueAttachmentRecord, IssueCommentError, IssueCommentRecord, IssueCommentRepository,
    IssueError, IssueLabelRecord, IssuePatch, IssueReadStateRecord, IssueRecord,
    IssueRecoveryActionRecord, IssueRelationError, IssueRelationRecord, IssueRelationRepository,
    IssueRepository, IssueStructureError, IssueStructureRepository, JoinRequestRecord, LabelError,
    LabelRecord, LabelRepository, MembershipError, MembershipRepository, NewActivity,
    NewAgentApiKey, NewApproval, NewAsset, NewBoardApiKey, NewBudgetIncident, NewBudgetPolicy,
    NewCase, NewCaseEvent, NewCliAuthChallenge, NewCompany, NewCompanyMembership, NewCostEvent,
    NewDecision, NewDecisionBundle, NewDecisionEffectExecution, NewDecisionTrainingExample,
    NewEnvironment, NewExecutionDecision, NewExecutionWorkspace, NewExternalObject,
    NewExternalObjectCatalog, NewExternalObjectMention, NewGoal, NewHeartbeatRun,
    NewInstanceUserRole, NewInvite, NewIssue, NewIssueAttachment, NewIssueComment,
    NewIssueDocument, NewIssueRelation, NewJoinRequest, NewLabel, NewManagedResource,
    NewPermissionGrant, NewPipeline, NewPipelineCase, NewPipelineCaseEvent, NewPlugin,
    NewPluginEntity, NewPluginJob, NewPluginJobRun, NewPluginLog, NewPluginMigration,
    NewPluginNamespace, NewPluginWebhook, NewProject, NewProjectWorkspace, NewRecoveryAction,
    NewRoutine, NewRuntimeService, NewRuntimeState, NewSecret, NewSkill, NewStage, NewTaskSession,
    NewThreadInteraction, NewTransition, NewTrigger, NewWakeupRequest, NewWorkProduct,
    NewWorkspaceOperation, PermissionGrantError, PermissionGrantRecord, PermissionGrantRepository,
    PipelineAutomationExecutionRecord, PipelineCaseBlockerRecord, PipelineCaseDocumentRecord,
    PipelineCaseEventRecord, PipelineCaseIssueLinkRecord, PipelineCaseRecord,
    PipelineDocumentRecord, PipelineError, PipelineRecord, PipelineRepository, PipelineStageRecord,
    PipelineTransitionRecord, PluginCompanySettingRecord, PluginConfigRecord,
    PluginDatabaseNamespaceRecord, PluginEntityRecord, PluginError, PluginJobRecord,
    PluginJobRunRecord, PluginLogRecord, PluginManagedResourceRecord, PluginMigrationRecord,
    PluginRecord, PluginRepository, PluginRuntimeError, PluginRuntimeRepository, PluginStateRecord,
    PluginWebhookDeliveryRecord, PreferenceError, PreferenceRepository, ProjectError, ProjectPatch,
    ProjectRecord, ProjectRepository, ProjectWorkspaceRecord, RepoError, ResolveDecision,
    RoutineError, RoutineRecord, RoutineRepository, RoutineRunRecord, RuntimeServiceRecord,
    SecretError, SecretRepository, SecretVersionRecord, SidebarPreferenceRecord, SkillError,
    SkillRecord, SkillRepository, ThreadInteractionRecord, TriageInput, TursoActivityRepository,
    TursoAgentRepository, TursoAgentRuntimeRepository, TursoApiKeyRepository,
    TursoApprovalRepository, TursoAssetRepository, TursoBoardKeyRepository,
    TursoBudgetPolicyRepository, TursoCaseRepository, TursoCompanyRepository, TursoCostRepository,
    TursoDecisionActionRepository, TursoDecisionRepository, TursoDocumentRepository,
    TursoEnvironmentRepository, TursoExternalObjectCatalogRepository,
    TursoExternalObjectRepository, TursoGoalRepository, TursoHeartbeatRepository,
    TursoInviteRepository, TursoIssueCommentRepository, TursoIssueRelationRepository,
    TursoIssueRepository, TursoIssueStructureRepository, TursoLabelRepository,
    TursoMembershipRepository, TursoPermissionGrantRepository, TursoPipelineRepository,
    TursoPluginRepository, TursoPluginRuntimeRepository, TursoPreferenceRepository,
    TursoProjectRepository, TursoRoutineRepository, TursoSecretRepository, TursoSkillRepository,
    TursoWorkProductRepository, TursoWorkspaceRepository, UpdateIssueDocument, UpdateRoutine,
    UpsertCompanySettings, UpsertPluginConfig, WorkProductError, WorkProductPatch,
    WorkProductRecord, WorkProductRepository, WorkspaceError, WorkspaceOperationRecord,
    WorkspaceRepository, activity, agent_runtime, agents, allowed_approval_transition,
    allowed_case_transition, allowed_status_transition, api_keys, approvals, assets, board_keys,
    budget_policies, cases, companies, costs, decision_actions, decision_desk, documents,
    environments, external_objects, goals, heartbeat_runs, invites, issue_comments,
    issue_relations, issue_structure, issues, labels, memberships, permission_grants, pipelines,
    plugin_runtime, plugins, preferences, projects, routines, work_products, workspaces,
};
pub use secrets::{CipherError, SecretCipher, default_key_path, redact};
pub use skills::{AgentFacts, SkillEvaluation, SkillFacts, SkillRestrictionPolicy, evaluate_skill};
