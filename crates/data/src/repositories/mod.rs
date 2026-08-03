//! Repository layer: traits plus Turso/libSQL implementations.
//!
//! Every repository method that reads or writes rows keeps its queries
//! company-scoped, and the schema's composite foreign keys enforce company
//! boundaries at the SQL level.

pub mod companies;

pub use companies::{
    CompanyPatch, CompanyRecord, CompanyRepository, NewCompany, RepoError, TursoCompanyRepository,
};
