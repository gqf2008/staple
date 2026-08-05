//! Staple Topcoat application.
//!
//! This crate wires the HTTP surface: configuration, routes, the request
//! logging layer, and unified JSON error handling.

pub mod audit;
pub mod auth;
pub mod board_claim;
pub mod config;
pub mod dto;
pub mod error;
pub mod git;
pub mod i18n;
pub mod logging;
pub mod permissions;
pub mod routes;
pub mod scheduler;
pub mod state;
pub mod storage;
pub mod team_catalog;
pub mod ui;

use topcoat::router::{Router, RouterBuilderDiscoverExt};

use crate::state::AppState;

/// Builds the application router with all routes and layers.
///
/// `state` is registered on the app context and made available to handlers.
#[must_use]
pub fn router(state: AppState) -> Router {
    Router::builder().app_context(state).discover().build()
}
