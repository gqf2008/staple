//! API route modules.

use topcoat::router::path_param;

pub mod activity;
pub mod adapters;
pub mod agents;
pub mod approvals;
pub mod assets;
pub mod auth;
pub mod board_keys;
pub mod budget_policies;
pub mod comments;
pub mod companies;
pub mod costs;
pub mod decisions;
pub mod documents;
pub mod external_objects;
pub mod goals;
pub mod health;
pub mod heartbeat;
pub mod invites;
pub mod issue_structure;
pub mod issues;
pub mod memberships;
pub mod permission_grants;
pub mod plugin_runtime;
pub mod plugins;
pub mod preferences;
pub mod projects;
pub mod relations;
pub mod routines;
pub mod secrets;
pub mod skills;
pub mod work_products;
pub mod workspaces;

/// Shared `{id}` path parameter (goals, projects, and future resources).
#[path_param(error = bad_request("Invalid id"))]
pub(crate) struct Id(String);

/// Shared `{company_id}` path parameter.
#[path_param(error = bad_request("Invalid company id"))]
pub(crate) struct CompanyId(String);

/// `{agent_id}` path parameter (agent-scoped routes).
#[path_param(error = bad_request("Invalid agent id"))]
pub(crate) struct AgentId(String);

/// Whether a string looks like a UUID (upstream validators use `z.uuid()`).
#[must_use]
pub(crate) fn is_uuid(value: &str) -> bool {
    uuid::Uuid::parse_str(value).is_ok()
}
