//! Permission grant scope evaluation (upstream SPEC §9.8).
//!
//! Pure functions over JSON scope documents. A grant is:
//!   - unscoped (`scope` null/empty) → matches any requested scope, except
//!     `tasks:assign_scope` which requires a structured (non-empty) scope;
//!   - scoped → every recognized constraint family present in the scope must
//!     match the requested scope; unknown keys do not constrain.
//!
//! Recognized constraint families:
//! - project: `projectId` | `projectIds` | `allow:["project:<id>"]`
//! - agent: `agentId(s)` | `assigneeAgentId(s)` | `targetAgentId(s)` |
//!   `allow:["agent:<id>"]`
//! - user: `userId` | `userIds`
//! - subtree: `managerAgentId(s)` | `managedSubtreeAgentId(s)` |
//!   `subtreeAgentId(s)` | `subtreeRootAgentId(s)` |
//!   `allow:["subtree:<id>"]` (target must sit under the root in the
//!   `reports_to` org graph)

use serde_json::Value;

/// Permission keys aligned with upstream `PERMISSION_KEYS`.
pub const PERMISSION_KEYS: &[&str] = &[
    "agents:create",
    "agents:configure",
    "agents:suggest-changes",
    "skills:create",
    "skills:suggest-changes",
    "environments:manage",
    "tools:admin",
    "tools:manage_connections",
    "tools:manage_profiles",
    "tools:view_audit",
    "audit:view_agent_actions",
    "tools:use",
    "tools:manage_runtime",
    "inbox:manage",
    "users:invite",
    "users:manage_permissions",
    "tasks:assign",
    "tasks:assign_scope",
    "tasks:manage_active_checkouts",
    "pipelines:write",
    "joins:approve",
];

/// Whether `key` is a known permission key.
#[must_use]
pub fn is_permission_key(key: &str) -> bool {
    PERMISSION_KEYS.contains(&key)
}

/// One org-graph row used for subtree evaluation.
#[derive(Debug, Clone)]
pub struct AgentHierarchyRow {
    /// Agent id.
    pub id: String,
    /// Manager agent id.
    pub reports_to: Option<String>,
}

fn scope_value_list(value: Option<&Value>) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };
    match value {
        Value::String(raw) if !raw.trim().is_empty() => vec![raw.trim().to_owned()],
        Value::Array(items) => items
            .iter()
            .filter_map(|item| item.as_str())
            .map(|item| item.trim().to_owned())
            .filter(|item| !item.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

fn prefixed_scope_values(scope: &Value, prefix: &str) -> Vec<String> {
    scope_value_list(scope.get("allow"))
        .into_iter()
        .filter(|rule| rule.starts_with(prefix))
        .map(|rule| rule[prefix.len()..].to_owned())
        .filter(|value| !value.is_empty())
        .collect()
}

fn scope_values_for_keys(scope: &Value, keys: &[&str]) -> Vec<String> {
    keys.iter()
        .flat_map(|key| scope_value_list(scope.get(*key)))
        .collect()
}

fn scope_includes_id(ids: &[String], id: Option<&str>) -> bool {
    id.is_some_and(|id| ids.iter().any(|candidate| candidate == id))
}

/// Whether `target_agent_id` is `root_agent_id` or descends from it through
/// `reports_to`.
#[must_use]
pub fn agent_is_in_subtree(
    hierarchy: &[AgentHierarchyRow],
    root_agent_id: &str,
    target_agent_id: &str,
) -> bool {
    if root_agent_id == target_agent_id {
        return true;
    }
    let by_id: std::collections::HashMap<&str, &AgentHierarchyRow> =
        hierarchy.iter().map(|row| (row.id.as_str(), row)).collect();
    let mut cursor = target_agent_id;
    for _ in 0..50 {
        let Some(row) = by_id.get(cursor) else {
            return false;
        };
        let Some(reports_to) = row.reports_to.as_deref() else {
            return false;
        };
        if reports_to == root_agent_id {
            return true;
        }
        cursor = reports_to;
    }
    false
}

/// Evaluates a grant's scope against a requested scope.
///
/// `require_structured_scope` forces a non-empty scope (used for
/// `tasks:assign_scope`). `hierarchy` must be provided when the grant uses a
/// subtree constraint.
#[must_use]
pub fn scope_allows(
    grant_scope: Option<&Value>,
    requested_scope: Option<&Value>,
    require_structured_scope: bool,
    hierarchy: Option<&[AgentHierarchyRow]>,
) -> bool {
    let Some(grant_scope) = grant_scope else {
        return !require_structured_scope;
    };
    if grant_scope.is_null() || grant_scope.as_object().is_none_or(|obj| obj.is_empty()) {
        return !require_structured_scope;
    }
    let Some(requested_scope) = requested_scope else {
        return false;
    };

    let target_assignee_agent_id = requested_scope
        .get("assigneeAgentId")
        .and_then(Value::as_str)
        .or_else(|| requested_scope.get("targetAgentId").and_then(Value::as_str));
    let requested_project_id = requested_scope.get("projectId").and_then(Value::as_str);
    let requested_user_id = requested_scope.get("userId").and_then(Value::as_str);

    let mut project_ids = scope_value_list(grant_scope.get("projectId"));
    project_ids.extend(scope_value_list(grant_scope.get("projectIds")));
    project_ids.extend(prefixed_scope_values(grant_scope, "project:"));
    if !project_ids.is_empty() && !scope_includes_id(&project_ids, requested_project_id) {
        return false;
    }

    let mut target_agent_ids = scope_values_for_keys(
        grant_scope,
        &[
            "agentId",
            "agentIds",
            "assigneeAgentId",
            "assigneeAgentIds",
            "targetAgentId",
            "targetAgentIds",
        ],
    );
    target_agent_ids.extend(prefixed_scope_values(grant_scope, "agent:"));
    if !target_agent_ids.is_empty()
        && !scope_includes_id(&target_agent_ids, target_assignee_agent_id)
    {
        return false;
    }

    let target_user_ids = scope_values_for_keys(grant_scope, &["userId", "userIds"]);
    if !target_user_ids.is_empty() && !scope_includes_id(&target_user_ids, requested_user_id) {
        return false;
    }

    let mut subtree_root_agent_ids = scope_values_for_keys(
        grant_scope,
        &[
            "managerAgentId",
            "managerAgentIds",
            "managedSubtreeAgentId",
            "managedSubtreeAgentIds",
            "subtreeAgentId",
            "subtreeAgentIds",
            "subtreeRootAgentId",
            "subtreeRootAgentIds",
        ],
    );
    subtree_root_agent_ids.extend(prefixed_scope_values(grant_scope, "subtree:"));
    if !subtree_root_agent_ids.is_empty() {
        let Some(target_assignee_agent_id) = target_assignee_agent_id else {
            return false;
        };
        let Some(hierarchy) = hierarchy else {
            return false;
        };
        if !subtree_root_agent_ids
            .iter()
            .any(|root| agent_is_in_subtree(hierarchy, root, target_assignee_agent_id))
        {
            return false;
        }
    }

    // Every recognized constraint family matched (or none constrained the
    // grant — unknown metadata keys never constrain).
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn hierarchy() -> Vec<AgentHierarchyRow> {
        vec![
            AgentHierarchyRow {
                id: "root".into(),
                reports_to: None,
            },
            AgentHierarchyRow {
                id: "mid".into(),
                reports_to: Some("root".into()),
            },
            AgentHierarchyRow {
                id: "leaf".into(),
                reports_to: Some("mid".into()),
            },
            AgentHierarchyRow {
                id: "other".into(),
                reports_to: None,
            },
        ]
    }

    #[test]
    fn unscoped_grant_matches_anything() {
        assert!(scope_allows(None, Some(&json!({})), false, None));
        assert!(scope_allows(
            Some(&json!({})),
            Some(&json!({ "projectId": "p1" })),
            false,
            None
        ));
        // tasks:assign_scope requires a structured scope.
        assert!(!scope_allows(None, Some(&json!({})), true, None));
        assert!(!scope_allows(
            Some(&json!({})),
            Some(&json!({ "projectId": "p1" })),
            true,
            None
        ));
    }

    #[test]
    fn project_scope_allows_and_denies() {
        let grant = json!({ "projectId": "p1" });
        assert!(scope_allows(
            Some(&grant),
            Some(&json!({ "projectId": "p1" })),
            false,
            None
        ));
        assert!(!scope_allows(
            Some(&grant),
            Some(&json!({ "projectId": "p2" })),
            false,
            None
        ));
        assert!(!scope_allows(
            Some(&grant),
            Some(&json!({ "assigneeAgentId": "a1" })),
            false,
            None
        ));
    }

    #[test]
    fn project_ids_and_allow_prefix_forms() {
        let grant = json!({ "projectIds": ["p1", "p2"] });
        assert!(scope_allows(
            Some(&grant),
            Some(&json!({ "projectId": "p2" })),
            false,
            None
        ));
        let grant = json!({ "allow": ["project:p1", "project:p3"] });
        assert!(scope_allows(
            Some(&grant),
            Some(&json!({ "projectId": "p3" })),
            false,
            None
        ));
        assert!(!scope_allows(
            Some(&grant),
            Some(&json!({ "projectId": "p2" })),
            false,
            None
        ));
    }

    #[test]
    fn agent_scope_allows_and_denies() {
        let grant = json!({ "assigneeAgentIds": ["a1", "a2"] });
        assert!(scope_allows(
            Some(&grant),
            Some(&json!({ "assigneeAgentId": "a1" })),
            false,
            None
        ));
        assert!(!scope_allows(
            Some(&grant),
            Some(&json!({ "assigneeAgentId": "a3" })),
            false,
            None
        ));
        // targetAgentId alias works.
        let grant = json!({ "targetAgentId": "a1" });
        assert!(scope_allows(
            Some(&grant),
            Some(&json!({ "targetAgentId": "a1" })),
            false,
            None
        ));
        let grant = json!({ "allow": ["agent:a1"] });
        assert!(scope_allows(
            Some(&grant),
            Some(&json!({ "assigneeAgentId": "a1" })),
            false,
            None
        ));
    }

    #[test]
    fn subtree_scope_requires_descendant() {
        let grant = json!({ "subtreeRootAgentIds": ["root"] });
        let h = hierarchy();
        assert!(scope_allows(
            Some(&grant),
            Some(&json!({ "assigneeAgentId": "leaf" })),
            false,
            Some(&h)
        ));
        assert!(scope_allows(
            Some(&grant),
            Some(&json!({ "assigneeAgentId": "root" })),
            false,
            Some(&h)
        ));
        assert!(!scope_allows(
            Some(&grant),
            Some(&json!({ "assigneeAgentId": "other" })),
            false,
            Some(&h)
        ));
        assert!(!scope_allows(
            Some(&grant),
            Some(&json!({ "assigneeAgentId": "leaf" })),
            false,
            None
        ));
        // managerAlias form.
        let grant = json!({ "managedSubtreeAgentId": "mid" });
        assert!(scope_allows(
            Some(&grant),
            Some(&json!({ "assigneeAgentId": "leaf" })),
            false,
            Some(&h)
        ));
        assert!(!scope_allows(
            Some(&grant),
            Some(&json!({ "assigneeAgentId": "root" })),
            false,
            Some(&h)
        ));
    }

    #[test]
    fn user_scope_for_inbox_manage() {
        let grant = json!({ "userIds": ["u1", "u2"] });
        assert!(scope_allows(
            Some(&grant),
            Some(&json!({ "userId": "u2" })),
            false,
            None
        ));
        assert!(!scope_allows(
            Some(&grant),
            Some(&json!({ "userId": "u3" })),
            false,
            None
        ));
    }

    #[test]
    fn multiple_families_must_all_match() {
        let grant = json!({ "projectId": "p1", "assigneeAgentId": "a1" });
        assert!(scope_allows(
            Some(&grant),
            Some(&json!({ "projectId": "p1", "assigneeAgentId": "a1" })),
            false,
            None
        ));
        assert!(!scope_allows(
            Some(&grant),
            Some(&json!({ "projectId": "p1", "assigneeAgentId": "a2" })),
            false,
            None
        ));
    }

    #[test]
    fn unknown_keys_do_not_constrain() {
        let grant = json!({ "note": "metadata only" });
        assert!(scope_allows(
            Some(&grant),
            Some(&json!({ "projectId": "p1" })),
            false,
            None
        ));
    }

    #[test]
    fn permission_keys_known() {
        assert!(is_permission_key("tasks:assign_scope"));
        assert!(is_permission_key("inbox:manage"));
        assert!(!is_permission_key("nope:unknown"));
    }
}
