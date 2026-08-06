//! Shared workspace concurrency policy parsing and resolution (issue #206).
//!
//! Mirrors upstream `SharedWorkspaceConcurrency = "auto" | "serialize" |
//! "allow"`. Resolution priority: issue execution workspace settings
//! `sharedWorkspaceConcurrency` > project execution workspace policy
//! `sharedWorkspaceConcurrency` (only when the policy is `enabled`) > default
//! (`auto` semantics, handled by the caller when `None` is returned).

use serde::{Deserialize, Serialize};

/// How runs may share a workspace when a project/issue has one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SharedWorkspaceConcurrency {
    /// Default: the runtime decides (usually serialized execution).
    Auto,
    /// Serialize runs sharing the workspace.
    Serialize,
    /// Allow concurrent runs in the same workspace.
    Allow,
}

impl SharedWorkspaceConcurrency {
    /// The wire representation (`"auto" | "serialize" | "allow"`).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Serialize => "serialize",
            Self::Allow => "allow",
        }
    }
}

impl std::str::FromStr for SharedWorkspaceConcurrency {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "auto" => Ok(Self::Auto),
            "serialize" => Ok(Self::Serialize),
            "allow" => Ok(Self::Allow),
            _ => Err(()),
        }
    }
}

/// Parses a JSON value as [`SharedWorkspaceConcurrency`]; invalid or missing
/// values yield `None`.
#[must_use]
pub fn parse_shared_workspace_concurrency(
    value: &serde_json::Value,
) -> Option<SharedWorkspaceConcurrency> {
    value.as_str().and_then(|raw| raw.parse().ok())
}

/// Resolves the effective shared workspace concurrency from issue settings
/// and project policy.
///
/// Priority (mirrors upstream):
/// 1. `issue_settings` JSON field `sharedWorkspaceConcurrency`
/// 2. `project_policy` JSON field `sharedWorkspaceConcurrency` when
///    `project_policy.enabled == true`
/// 3. `None` (default `auto` semantics, applied by the caller)
#[must_use]
pub fn resolve_shared_workspace_concurrency(
    issue_settings: Option<&str>,
    project_policy: Option<&serde_json::Value>,
) -> Option<SharedWorkspaceConcurrency> {
    if let Some(settings) = issue_settings
        && let Ok(value) = serde_json::from_str::<serde_json::Value>(settings)
        && let Some(concurrency) = value
            .get("sharedWorkspaceConcurrency")
            .and_then(parse_shared_workspace_concurrency)
    {
        return Some(concurrency);
    }

    if let Some(policy) = project_policy
        && policy.get("enabled").and_then(serde_json::Value::as_bool) == Some(true)
        && let Some(concurrency) = policy
            .get("sharedWorkspaceConcurrency")
            .and_then(parse_shared_workspace_concurrency)
    {
        return Some(concurrency);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn enum_serde_round_trips_camel_case() {
        for (expected, variant) in [
            ("auto", SharedWorkspaceConcurrency::Auto),
            ("serialize", SharedWorkspaceConcurrency::Serialize),
            ("allow", SharedWorkspaceConcurrency::Allow),
        ] {
            assert_eq!(variant.as_str(), expected);
            assert_eq!(
                serde_json::to_string(&variant).unwrap(),
                format!("\"{expected}\"")
            );
            assert_eq!(
                serde_json::from_str::<SharedWorkspaceConcurrency>(&format!("\"{expected}\""))
                    .unwrap(),
                variant
            );
        }
        assert!(serde_json::from_str::<SharedWorkspaceConcurrency>("\"bogus\"").is_err());
    }

    #[test]
    fn parse_accepts_valid_and_rejects_invalid() {
        assert_eq!(
            parse_shared_workspace_concurrency(&json!("auto")),
            Some(SharedWorkspaceConcurrency::Auto)
        );
        assert_eq!(
            parse_shared_workspace_concurrency(&json!("serialize")),
            Some(SharedWorkspaceConcurrency::Serialize)
        );
        assert_eq!(
            parse_shared_workspace_concurrency(&json!("allow")),
            Some(SharedWorkspaceConcurrency::Allow)
        );
        assert_eq!(parse_shared_workspace_concurrency(&json!("bogus")), None);
        assert_eq!(parse_shared_workspace_concurrency(&json!(42)), None);
        assert_eq!(parse_shared_workspace_concurrency(&json!(null)), None);
        assert_eq!(parse_shared_workspace_concurrency(&json!({})), None);
    }

    #[test]
    fn resolve_issue_settings_win_over_project_policy() {
        let issue = r#"{"sharedWorkspaceConcurrency":"allow"}"#;
        let policy = json!({ "enabled": true, "sharedWorkspaceConcurrency": "serialize" });
        assert_eq!(
            resolve_shared_workspace_concurrency(Some(issue), Some(&policy)),
            Some(SharedWorkspaceConcurrency::Allow)
        );
    }

    #[test]
    fn resolve_project_policy_applies_only_when_enabled() {
        let policy = json!({ "enabled": true, "sharedWorkspaceConcurrency": "serialize" });
        assert_eq!(
            resolve_shared_workspace_concurrency(None, Some(&policy)),
            Some(SharedWorkspaceConcurrency::Serialize)
        );

        // Disabled policy is ignored even when it carries the field.
        let disabled = json!({ "enabled": false, "sharedWorkspaceConcurrency": "serialize" });
        assert_eq!(
            resolve_shared_workspace_concurrency(None, Some(&disabled)),
            None
        );

        // Missing `enabled` is treated as disabled.
        let missing_enabled = json!({ "sharedWorkspaceConcurrency": "serialize" });
        assert_eq!(
            resolve_shared_workspace_concurrency(None, Some(&missing_enabled)),
            None
        );
    }

    #[test]
    fn resolve_defaults_to_none() {
        assert_eq!(resolve_shared_workspace_concurrency(None, None), None);
        assert_eq!(resolve_shared_workspace_concurrency(Some("{}"), None), None);
        let enabled_no_field = json!({ "enabled": true });
        assert_eq!(
            resolve_shared_workspace_concurrency(None, Some(&enabled_no_field)),
            None
        );
    }

    #[test]
    fn resolve_invalid_issue_settings_fall_through_to_project() {
        let policy = json!({ "enabled": true, "sharedWorkspaceConcurrency": "serialize" });
        assert_eq!(
            resolve_shared_workspace_concurrency(Some("not json"), Some(&policy)),
            Some(SharedWorkspaceConcurrency::Serialize)
        );
        // Invalid issue value does not mask a valid project policy either.
        let bad_issue = r#"{"sharedWorkspaceConcurrency":"bogus"}"#;
        assert_eq!(
            resolve_shared_workspace_concurrency(Some(bad_issue), Some(&policy)),
            Some(SharedWorkspaceConcurrency::Serialize)
        );
    }
}
