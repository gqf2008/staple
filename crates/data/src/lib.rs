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
    AssetError, AssetRecord, AssetRepository, CompanyPatch, CompanyRecord, CompanyRepository,
    DocumentError, DocumentRecord, DocumentRepository, GoalError, GoalPatch, GoalRecord,
    GoalRepository, IssueAttachmentRecord, IssueCommentError, IssueCommentRecord,
    IssueCommentRepository, IssueError, IssuePatch, IssueRecord, IssueRelationError,
    IssueRelationRecord, IssueRelationRepository, IssueRepository, NewAsset, NewCompany, NewGoal,
    NewIssue, NewIssueAttachment, NewIssueComment, NewIssueDocument, NewIssueRelation, NewProject,
    NewWorkProduct, ProjectError, ProjectPatch, ProjectRecord, ProjectRepository, RepoError,
    TursoAssetRepository, TursoCompanyRepository, TursoDocumentRepository, TursoGoalRepository,
    TursoIssueCommentRepository, TursoIssueRelationRepository, TursoIssueRepository,
    TursoProjectRepository, TursoWorkProductRepository, UpdateIssueDocument, WorkProductError,
    WorkProductPatch, WorkProductRecord, WorkProductRepository, allowed_status_transition, assets,
    companies, documents, goals, issue_comments, issue_relations, issues, projects, work_products,
};
