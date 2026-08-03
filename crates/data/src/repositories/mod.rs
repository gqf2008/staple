//! Repository layer: traits plus Turso/libSQL implementations.
//!
//! Every repository method that reads or writes rows keeps its queries
//! company-scoped, and the schema's composite foreign keys enforce company
//! boundaries at the SQL level.

pub mod companies;
pub mod goals;
mod helpers;
pub mod projects;

pub use companies::{
    CompanyPatch, CompanyRecord, CompanyRepository, NewCompany, RepoError, TursoCompanyRepository,
};
pub use goals::{GoalError, GoalPatch, GoalRecord, GoalRepository, NewGoal, TursoGoalRepository};
pub use projects::{
    NewProject, ProjectError, ProjectPatch, ProjectRecord, ProjectRepository,
    TursoProjectRepository,
};
