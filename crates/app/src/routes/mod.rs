//! API route modules.

use serde::Deserialize;
use topcoat::router::path_param;

pub mod activity;
pub mod adapters;
pub mod agent_runtime;
pub mod agents;
pub mod approvals;
pub mod assets;
pub mod attention;
pub mod auth;
pub mod board_chat;
pub mod board_claim;
pub mod board_keys;
pub mod budget_policies;
pub mod cases;
pub mod comments;
pub mod companies;
pub mod costs;
pub mod decision_actions;
pub mod decisions;
pub mod documents;
pub mod external_objects;
pub mod goals;
pub mod health;
pub mod heartbeat;
pub mod infrastructure;
pub mod instructions;
pub mod invites;
pub mod issue_structure;
pub mod issues;
pub mod memberships;
pub mod permission_grants;
pub mod pipelines;
pub mod plugin_runtime;
pub mod plugins;
pub mod portability;
pub mod preferences;
pub mod projects;
pub mod relations;
pub mod routines;
pub mod scattered;
pub mod secret_bindings;
pub mod secrets;
pub mod skill_catalog;
pub mod skills;
pub mod team_catalog;
pub mod toolchain;
pub mod work_products;
pub mod workspaces;

/// Shared `{id}` path parameter (goals, projects, and future resources).
#[path_param(error = bad_request("Invalid id"))]
pub(crate) struct Id(String);

/// `{skill_id}` path parameter (skill catalog routes).
#[path_param(error = bad_request("Invalid skill id"))]
pub(crate) struct SkillId(String);

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

/// Deserializes an optional JSON value with explicit-`null` semantics for
/// PATCH bodies: missing → `None` (leave unchanged), `null` → `Some(None)`
/// (clear), any value → `Some(Some(value))` (set).
///
/// Plain `Option<Option<T>>` with `#[serde(default)]` cannot distinguish
/// `null` from a missing field, so update requests use this via
/// `#[serde(default, deserialize_with = "crate::routes::deserialize_optional_json")]`.
pub(crate) fn deserialize_optional_json<'de, D>(
    deserializer: D,
) -> Result<Option<Option<serde_json::Value>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(Some(if value.is_null() { None } else { Some(value) }))
}
