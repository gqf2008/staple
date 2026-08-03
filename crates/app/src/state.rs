//! Shared application state, registered on the Topcoat app context.

use std::sync::Arc;

use staple_data::CompanyRepository;

/// Application-wide dependencies for route handlers.
#[derive(Clone)]
pub struct AppState {
    /// Companies repository.
    pub companies: Arc<dyn CompanyRepository>,
}
