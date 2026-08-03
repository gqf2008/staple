//! Staple Topcoat application.
//!
//! This crate wires the HTTP surface: configuration, routes, the request
//! logging layer, and unified JSON error handling.

pub mod config;
pub mod error;
pub mod logging;
pub mod routes;

use topcoat::router::{Router, RouterBuilderDiscoverExt};

/// Builds the application router with all routes and layers.
#[must_use]
pub fn router() -> Router {
    Router::builder().discover().build()
}
