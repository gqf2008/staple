//! Repository layer: traits plus Turso/libSQL implementations.
//!
//! Every repository method that reads or writes rows keeps its queries
//! company-scoped, and the schema's composite foreign keys enforce company
//! boundaries at the SQL level.

pub mod activity;
pub mod agent_runtime;
pub mod agents;
pub mod api_keys;
pub mod approvals;
pub mod assets;
pub mod board_keys;
pub mod budget_policies;
pub mod cases;
pub mod companies;
pub mod costs;
pub mod decision_desk;
pub mod documents;
pub mod environments;
pub mod external_objects;
pub mod goals;
pub mod heartbeat_runs;
pub mod helpers;
pub mod invites;
pub mod issue_comments;
pub mod issue_relations;
pub mod issue_structure;
pub mod issues;
pub mod labels;
pub mod memberships;
pub mod permission_grants;
pub mod pipelines;
pub mod plugin_runtime;
pub mod plugins;
pub mod preferences;
pub mod projects;
pub mod routines;
pub mod secrets;
pub mod skills;
pub mod work_products;
pub mod workspaces;

pub use activity::{
    ActivityEntry, ActivityError, ActivityRepository, NewActivity, TursoActivityRepository,
};
pub use agent_runtime::{
    AgentRuntimeError, AgentRuntimeRepository, AgentRuntimeStateRecord, AgentTaskSessionRecord,
    AgentWakeupRequestRecord, IssueRecoveryActionRecord, NewRecoveryAction, NewRuntimeState,
    NewTaskSession, NewWakeupRequest, TursoAgentRuntimeRepository,
};
pub use agents::{
    AgentBudgetRecord, AgentError, AgentHierarchyRow, AgentRecord, AgentRepository,
    TursoAgentRepository,
};
pub use api_keys::{
    AgentApiKeyRecord, AgentPrincipal, ApiKeyError, ApiKeyRepository, NewAgentApiKey,
    TursoApiKeyRepository,
};
pub use approvals::{
    ApprovalDecision, ApprovalError, ApprovalRecord, ApprovalRepository, NewApproval,
    TursoApprovalRepository, allowed_approval_transition,
};
pub use assets::{
    AssetError, AssetRecord, AssetRepository, IssueAttachmentRecord, NewAsset, NewIssueAttachment,
    TursoAssetRepository,
};
pub use board_keys::{
    BoardApiKeyRecord, BoardKeyError, BoardKeyRepository, CliAuthChallengeRecord, NewBoardApiKey,
    NewCliAuthChallenge, TursoBoardKeyRepository,
};
pub use budget_policies::{
    BudgetIncidentRecord, BudgetPolicyError, BudgetPolicyRecord, BudgetPolicyRepository,
    NewBudgetIncident, NewBudgetPolicy, TursoBudgetPolicyRepository,
};
pub use cases::{
    CaseAttachmentRecord, CaseDocumentRecord, CaseError, CaseEventRecord, CaseIssueLinkRecord,
    CaseLabelRecord, CasePatch, CaseRecord, CaseRepository, NewCase, NewCaseEvent,
    TursoCaseRepository, allowed_case_transition,
};
pub use companies::{
    CompanyPatch, CompanyRecord, CompanyRepository, NewCompany, RepoError, TursoCompanyRepository,
};
pub use costs::{
    AgentCostRow, BudgetSummary, CostError, CostEventOutcome, CostEventRecord, CostRepository,
    NewCostEvent, TursoCostRepository,
};
pub use decision_desk::{
    DecisionError, DecisionOutboxRecord, DecisionQueueItemRecord, DecisionQueueRecord,
    DecisionRepository, DecisionRetentionRecord, DecisionSweepResult, DecisionTriageEventRecord,
    DecisionTriageRecord, TriageInput, TursoDecisionRepository,
};
pub use documents::{
    DocumentError, DocumentRecord, DocumentRepository, NewIssueDocument, TursoDocumentRepository,
    UpdateIssueDocument,
};
pub use environments::{
    EnvironmentError, EnvironmentRecord, EnvironmentRepository, NewEnvironment,
    TursoEnvironmentRepository,
};
pub use external_objects::{
    ExternalObjectCatalogRecord, ExternalObjectCatalogRepository, ExternalObjectError,
    ExternalObjectMentionRecord, ExternalObjectRecord, ExternalObjectRepository, NewExternalObject,
    NewExternalObjectCatalog, NewExternalObjectMention, TursoExternalObjectCatalogRepository,
    TursoExternalObjectRepository,
};
pub use goals::{GoalError, GoalPatch, GoalRecord, GoalRepository, NewGoal, TursoGoalRepository};
pub use heartbeat_runs::{
    CompleteHeartbeatRun, HeartbeatError, HeartbeatRepository, HeartbeatRunRecord, NewHeartbeatRun,
    TursoHeartbeatRepository,
};
pub use invites::{
    InviteError, InviteRecord, InviteRepository, JoinRequestRecord, NewInvite, NewJoinRequest,
    TursoInviteRepository,
};
pub use issue_comments::{
    IssueCommentError, IssueCommentRecord, IssueCommentRepository, NewIssueComment,
    TursoIssueCommentRepository,
};
pub use issue_relations::{
    IssueRelationError, IssueRelationRecord, IssueRelationRepository, NewIssueRelation,
    TursoIssueRelationRepository,
};
pub use issue_structure::{
    ExecutionDecisionRecord, IssueApprovalRecord, IssueReadStateRecord, IssueStructureError,
    IssueStructureRepository, NewExecutionDecision, NewThreadInteraction, ThreadInteractionRecord,
    TursoIssueStructureRepository,
};
pub use issues::{
    IssueError, IssuePatch, IssueRecord, IssueRepository, NewIssue, TursoIssueRepository,
    allowed_status_transition,
};
pub use labels::{
    IssueLabelRecord, LabelError, LabelRecord, LabelRepository, NewLabel, TursoLabelRepository,
};
pub use memberships::{
    CompanyMembershipRecord, InstanceUserRoleRecord, MembershipError, MembershipRepository,
    NewCompanyMembership, NewInstanceUserRole, TursoMembershipRepository,
};
pub use permission_grants::{
    NewPermissionGrant, PermissionGrantError, PermissionGrantRecord, PermissionGrantRepository,
    TursoPermissionGrantRepository,
};
pub use pipelines::{
    NewPipeline, NewPipelineCase, NewPipelineCaseEvent, NewStage, NewTransition,
    PipelineAutomationExecutionRecord, PipelineCaseBlockerRecord, PipelineCaseDocumentRecord,
    PipelineCaseEventRecord, PipelineCaseIssueLinkRecord, PipelineCaseRecord,
    PipelineDocumentRecord, PipelineError, PipelineRecord, PipelineRepository, PipelineStageRecord,
    PipelineTransitionRecord, TursoPipelineRepository,
};
pub use plugin_runtime::{
    NewPluginEntity, NewPluginJob, NewPluginJobRun, NewPluginLog, NewPluginMigration,
    NewPluginNamespace, NewPluginWebhook, PluginDatabaseNamespaceRecord, PluginEntityRecord,
    PluginJobRecord, PluginJobRunRecord, PluginLogRecord, PluginMigrationRecord,
    PluginRuntimeError, PluginRuntimeRepository, PluginStateRecord, PluginWebhookDeliveryRecord,
    TursoPluginRuntimeRepository,
};
pub use plugins::{
    NewManagedResource, NewPlugin, PluginCompanySettingRecord, PluginConfigRecord, PluginError,
    PluginManagedResourceRecord, PluginRecord, PluginRepository, TursoPluginRepository,
    UpsertCompanySettings, UpsertPluginConfig,
};
pub use preferences::{
    CompanyLogoRecord, PreferenceError, PreferenceRepository, SidebarPreferenceRecord,
    TursoPreferenceRepository,
};
pub use projects::{
    NewProject, ProjectError, ProjectPatch, ProjectRecord, ProjectRepository,
    TursoProjectRepository,
};
pub use routines::{
    NewRoutine, NewTrigger, RoutineError, RoutineRecord, RoutineRepository, RoutineRunRecord,
    TursoRoutineRepository, UpdateRoutine,
};
pub use secrets::{
    CompanySecretRecord, NewSecret, SecretError, SecretRepository, SecretVersionRecord,
    TursoSecretRepository,
};
pub use skills::{NewSkill, SkillError, SkillRecord, SkillRepository, TursoSkillRepository};
pub use work_products::{
    NewWorkProduct, TursoWorkProductRepository, WorkProductError, WorkProductPatch,
    WorkProductRecord, WorkProductRepository,
};
pub use workspaces::{
    ExecutionWorkspaceRecord, NewExecutionWorkspace, NewProjectWorkspace, NewRuntimeService,
    NewWorkspaceOperation, ProjectWorkspaceRecord, RuntimeServiceRecord, TursoWorkspaceRepository,
    WorkspaceError, WorkspaceOperationRecord, WorkspaceRepository,
};
