//! API routes.

use serde::Serialize;
use topcoat::{
    Result,
    router::{content::Json, route},
};

/// Response body for `GET /api/health`.
#[derive(Debug, Serialize)]
pub struct Health {
    status: &'static str,
}

/// Health check endpoint.
#[route(GET "/api/health")]
pub async fn health() -> Result<Json<Health>> {
    Ok(Json(Health { status: "ok" }))
}
