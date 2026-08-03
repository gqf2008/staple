//! Staple Turso/libSQL data layer.
//!
//! Owns the connection layer (`connection`), the versioned SQL migrations
//! (`migrations`), and — in later milestones — the repositories.

pub mod connection;
pub mod migrations;
pub mod repositories;
pub mod secrets;

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
    DecisionQueueRecord, DecisionRepository, DecisionTriageRecord, TriageInput, DocumentError, DocumentRecord,
    DocumentRepository, ExternalObjectError, ExternalObjectRecord, ExternalObjectRepository,
    GoalError, GoalPatch, GoalRecord, GoalRepository, HeartbeatError, HeartbeatRepository,
    HeartbeatRunRecord, IssueAttachmentRecord, IssueCommentError, IssueCommentRecord,
    IssueCommentRepository, IssueError, IssuePatch, IssueRecord, IssueRelationError,
    IssueRelationRecord, IssueRelationRepository, IssueRepository, NewActivity, NewAgentApiKey,
    NewApproval, NewAsset, NewCompany, NewCostEvent, NewExternalObject, NewGoal, NewHeartbeatRun,
    NewIssue, NewIssueAttachment, NewIssueComment, NewIssueDocument, NewIssueRelation, NewProject,
    NewSecret, NewWorkProduct, ProjectError, ProjectPatch, ProjectRecord, ProjectRepository,
    RepoError, SecretError, SecretRepository, SecretVersionRecord, TursoActivityRepository,
    TursoApiKeyRepository, TursoApprovalRepository, TursoAssetRepository, TursoCompanyRepository,
    TursoCostRepository, TursoDecisionRepository, TursoDocumentRepository,
    TursoExternalObjectRepository, TursoGoalRepository, TursoHeartbeatRepository,
    TursoIssueCommentRepository, TursoIssueRelationRepository, TursoIssueRepository,
    TursoProjectRepository, TursoSecretRepository, TursoWorkProductRepository, UpdateIssueDocument,
    WorkProductError, WorkProductPatch, WorkProductRecord, WorkProductRepository, activity,
    allowed_approval_transition, allowed_status_transition, api_keys, approvals, assets, companies,
    costs, decision_desk, documents, external_objects, goals, heartbeat_runs, issue_comments,
    issue_relations, issues, projects, work_products,
};
pub use secrets::{CipherError, SecretCipher, default_key_path, redact};
