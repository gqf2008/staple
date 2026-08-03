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
    CompanyPatch, CompanyRecord, CompanyRepository, GoalError, GoalPatch, GoalRecord,
    GoalRepository, NewCompany, NewGoal, NewProject, ProjectError, ProjectPatch, ProjectRecord,
    ProjectRepository, RepoError, TursoCompanyRepository, TursoGoalRepository,
    TursoProjectRepository, companies, goals, projects,
};
