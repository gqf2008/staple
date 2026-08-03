//! Staple Turso/libSQL data layer.
//!
//! Owns the connection layer (`connection`), the versioned SQL migrations
//! (`migrations`), and — in later milestones — the repositories.

pub mod connection;
pub mod migrations;
pub mod repositories;

pub use connection::{DataError, DbConfig, connect, open};
pub use libsql::{Connection, Database};
pub use migrations::{
    MigrateError, Migration, load_migrations, migrate, migrate_conn, migrate_down,
};
pub use repositories::{
    AgentCostRow, AssetError, AssetRecord, AssetRepository, BudgetSummary, CompanyPatch,
    CompanyRecord, CompanyRepository, CompleteHeartbeatRun, CostError, CostEventOutcome,
    CostEventRecord, CostRepository, DocumentError, DocumentRecord, DocumentRepository, GoalError,
    GoalPatch, GoalRecord, GoalRepository, HeartbeatError, HeartbeatRepository, HeartbeatRunRecord,
    IssueAttachmentRecord, IssueCommentError, IssueCommentRecord, IssueCommentRepository,
    IssueError, IssuePatch, IssueRecord, IssueRelationError, IssueRelationRecord,
    IssueRelationRepository, IssueRepository, NewAsset, NewCompany, NewCostEvent, NewGoal,
    NewHeartbeatRun, NewIssue, NewIssueAttachment, NewIssueComment, NewIssueDocument,
    NewIssueRelation, NewProject, NewWorkProduct, ProjectError, ProjectPatch, ProjectRecord,
    ProjectRepository, RepoError, TursoAssetRepository, TursoCompanyRepository,
    TursoCostRepository, TursoDocumentRepository, TursoGoalRepository, TursoHeartbeatRepository,
    TursoIssueCommentRepository, TursoIssueRelationRepository, TursoIssueRepository,
    TursoProjectRepository, TursoWorkProductRepository, UpdateIssueDocument, WorkProductError,
    WorkProductPatch, WorkProductRecord, WorkProductRepository, allowed_status_transition, assets,
    companies, costs, documents, goals, heartbeat_runs, issue_comments, issue_relations, issues,
    projects, work_products,
};
