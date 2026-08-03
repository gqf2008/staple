//! Shared application state, registered on the Topcoat app context.

use std::sync::Arc;

use staple_data::{CompanyRepository, GoalRepository, ProjectRepository};

/// Application-wide dependencies for route handlers.
#[derive(Clone)]
pub struct AppState {
    /// Companies repository.
    pub companies: Arc<dyn CompanyRepository>,
    /// Goals repository.
    pub goals: Arc<dyn GoalRepository>,
    /// Projects repository.
    pub projects: Arc<dyn ProjectRepository>,
}
