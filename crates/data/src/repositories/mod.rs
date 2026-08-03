//! Repository layer: traits plus Turso/libSQL implementations.
//!
//! Every repository method that reads or writes rows keeps its queries
//! company-scoped, and the schema's composite foreign keys enforce company
//! boundaries at the SQL level.

pub mod activity;
pub mod approvals;
pub mod assets;
pub mod companies;
pub mod costs;
pub mod documents;
pub mod goals;
pub mod heartbeat_runs;
mod helpers;
pub mod issue_comments;
pub mod issue_relations;
pub mod issues;
pub mod projects;
pub mod secrets;
pub mod work_products;

pub use activity::{
    ActivityEntry, ActivityError, ActivityRepository, NewActivity, TursoActivityRepository,
};
pub use approvals::{
    ApprovalDecision, ApprovalError, ApprovalRecord, ApprovalRepository, NewApproval,
    TursoApprovalRepository, allowed_approval_transition,
};
pub use assets::{
    AssetError, AssetRecord, AssetRepository, IssueAttachmentRecord, NewAsset, NewIssueAttachment,
    TursoAssetRepository,
};
pub use companies::{
    CompanyPatch, CompanyRecord, CompanyRepository, NewCompany, RepoError, TursoCompanyRepository,
};
pub use costs::{
    AgentCostRow, BudgetSummary, CostError, CostEventOutcome, CostEventRecord, CostRepository,
    NewCostEvent, TursoCostRepository,
};
pub use documents::{
    DocumentError, DocumentRecord, DocumentRepository, NewIssueDocument, TursoDocumentRepository,
    UpdateIssueDocument,
};
pub use goals::{GoalError, GoalPatch, GoalRecord, GoalRepository, NewGoal, TursoGoalRepository};
pub use heartbeat_runs::{
    CompleteHeartbeatRun, HeartbeatError, HeartbeatRepository, HeartbeatRunRecord, NewHeartbeatRun,
    TursoHeartbeatRepository,
};
pub use issue_comments::{
    IssueCommentError, IssueCommentRecord, IssueCommentRepository, NewIssueComment,
    TursoIssueCommentRepository,
};
pub use issue_relations::{
    IssueRelationError, IssueRelationRecord, IssueRelationRepository, NewIssueRelation,
    TursoIssueRelationRepository,
};
pub use issues::{
    IssueError, IssuePatch, IssueRecord, IssueRepository, NewIssue, TursoIssueRepository,
    allowed_status_transition,
};
pub use projects::{
    NewProject, ProjectError, ProjectPatch, ProjectRecord, ProjectRepository,
    TursoProjectRepository,
};
pub use secrets::{
    CompanySecretRecord, NewSecret, SecretError, SecretRepository, SecretVersionRecord,
    TursoSecretRepository,
};
pub use work_products::{
    NewWorkProduct, TursoWorkProductRepository, WorkProductError, WorkProductPatch,
    WorkProductRecord, WorkProductRepository,
};
