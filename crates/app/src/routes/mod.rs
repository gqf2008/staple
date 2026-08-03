//! API route modules.

use topcoat::router::path_param;

pub mod assets;
pub mod comments;
pub mod companies;
pub mod documents;
pub mod goals;
pub mod health;
pub mod heartbeat;
pub mod issues;
pub mod projects;
pub mod relations;
pub mod work_products;

/// Shared `{id}` path parameter (goals, projects, and future resources).
#[path_param(error = bad_request("Invalid id"))]
pub(crate) struct Id(String);

/// Shared `{company_id}` path parameter.
#[path_param(error = bad_request("Invalid company id"))]
pub(crate) struct CompanyId(String);

/// Whether a string looks like a UUID (upstream validators use `z.uuid()`).
#[must_use]
pub(crate) fn is_uuid(value: &str) -> bool {
    uuid::Uuid::parse_str(value).is_ok()
}
