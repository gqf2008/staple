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
pub use repositories::{
    ActivityEntry, ActivityError, ActivityRepository, AgentApiKeyRecord, AgentCostRow,
    AgentPrincipal, ApiKeyError, ApiKeyRepository, ApprovalDecision, ApprovalError, ApprovalRecord,
    ApprovalRepository, AssetError, AssetRecord, AssetRepository, BudgetSummary, CompanyPatch,
    CompanyRecord, CompanyRepository, CompanySecretRecord, CompleteHeartbeatRun, CostError,
    CostEventOutcome, CostEventRecord, CostRepository, DecisionError, DecisionQueueItemRecord,
    DecisionQueueRecord, DecisionRepository, DecisionTriageRecord, DocumentError, DocumentRecord,
    DocumentRepository, EnvironmentError, EnvironmentRecord, EnvironmentRepository,
    ExecutionDecisionRecord, ExecutionWorkspaceRecord, ExternalObjectError, ExternalObjectRecord,
    ExternalObjectRepository, GoalError, GoalPatch, GoalRecord, GoalRepository, HeartbeatError,
    HeartbeatRepository, HeartbeatRunRecord, IssueApprovalRecord, IssueAttachmentRecord,
    IssueCommentError, IssueCommentRecord, IssueCommentRepository, IssueError, IssueLabelRecord,
    IssuePatch, IssueReadStateRecord, IssueRecord, IssueRelationError, IssueRelationRecord,
    IssueRelationRepository, IssueRepository, IssueStructureError, IssueStructureRepository,
    LabelError, LabelRecord, LabelRepository, NewActivity, NewAgentApiKey, NewApproval, NewAsset,
    NewCompany, NewCostEvent, NewEnvironment, NewExecutionDecision, NewExecutionWorkspace,
    NewExternalObject, NewGoal, NewHeartbeatRun, NewIssue, NewIssueAttachment, NewIssueComment,
    NewIssueDocument, NewIssueRelation, NewLabel, NewProject, NewProjectWorkspace,
    NewRuntimeService, NewSecret, NewSkill, NewThreadInteraction, NewWorkProduct,
    NewWorkspaceOperation, ProjectError, ProjectPatch, ProjectRecord, ProjectRepository,
    ProjectWorkspaceRecord, RepoError, RuntimeServiceRecord, SecretError, SecretRepository,
    SecretVersionRecord, SkillError, SkillRecord, SkillRepository, ThreadInteractionRecord,
    TriageInput, TursoActivityRepository, TursoApiKeyRepository, TursoApprovalRepository,
    TursoAssetRepository, TursoCompanyRepository, TursoCostRepository, TursoDecisionRepository,
    TursoDocumentRepository, TursoEnvironmentRepository, TursoExternalObjectRepository,
    TursoGoalRepository, TursoHeartbeatRepository, TursoIssueCommentRepository,
    TursoIssueRelationRepository, TursoIssueRepository, TursoIssueStructureRepository,
    TursoLabelRepository, TursoProjectRepository, TursoSecretRepository, TursoSkillRepository,
    TursoWorkProductRepository, TursoWorkspaceRepository, UpdateIssueDocument, WorkProductError,
    WorkProductPatch, WorkProductRecord, WorkProductRepository, WorkspaceError,
    WorkspaceOperationRecord, WorkspaceRepository, activity, allowed_approval_transition,
    allowed_status_transition, api_keys, approvals, assets, companies, costs, decision_desk,
    documents, environments, external_objects, goals, heartbeat_runs, issue_comments,
    issue_relations, issue_structure, issues, labels, projects, work_products, workspaces,
};
pub use secrets::{CipherError, SecretCipher, default_key_path, redact};
pub use skills::{AgentFacts, SkillEvaluation, SkillFacts, SkillRestrictionPolicy, evaluate_skill};
