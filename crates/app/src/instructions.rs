//! Agent instruction bundle helpers: default materialization and path
//! validation (parity with upstream `agent-instructions.ts` +
//! `default-agent-instructions.ts`).

use serde_json::json;
use staple_data::NewAgentInstructionFile;

use crate::{error::ApiError, state::AppState};

/// One file in a default instruction bundle: relative path, entry flag, and
/// built-in template content.
type BundleFile = (&'static str, bool, &'static str);

/// The default instruction bundle for a role. `ceo` agents get the full
/// startup set (`AGENTS.md` + `HEARTBEAT.md` + `SOUL.md` + `TOOLS.md`); every
/// other role gets `AGENTS.md` only. `AGENTS.md` is always the entry file.
#[must_use]
pub fn default_bundle_files(role: &str) -> &'static [BundleFile] {
    if role == "ceo" {
        &[
            (
                "AGENTS.md",
                true,
                include_str!("../onboarding-assets/ceo/AGENTS.md"),
            ),
            (
                "HEARTBEAT.md",
                false,
                include_str!("../onboarding-assets/ceo/HEARTBEAT.md"),
            ),
            (
                "SOUL.md",
                false,
                include_str!("../onboarding-assets/ceo/SOUL.md"),
            ),
            (
                "TOOLS.md",
                false,
                include_str!("../onboarding-assets/ceo/TOOLS.md"),
            ),
        ]
    } else {
        &[(
            "AGENTS.md",
            true,
            include_str!("../onboarding-assets/default/AGENTS.md"),
        )]
    }
}

/// Mounts the default instruction bundle for a newly created agent, returning
/// the number of files written. Idempotent: re-running replaces the files.
///
/// # Errors
///
/// Returns [`ApiError`] when the agent does not belong to the company or the
/// database write fails.
pub async fn materialize_default_instructions(
    state: &AppState,
    company_id: &str,
    agent_id: &str,
    role: &str,
) -> Result<usize, ApiError> {
    let mut written = 0;
    for (path, is_entry, content) in default_bundle_files(role) {
        state
            .instructions
            .upsert_agent_file(NewAgentInstructionFile {
                company_id: company_id.to_owned(),
                agent_id: agent_id.to_owned(),
                path: (*path).to_owned(),
                content: (*content).to_owned(),
                is_entry: *is_entry,
            })
            .await
            .map_err(|error| ApiError::internal(error.to_string()))?;
        written += 1;
    }
    Ok(written)
}

/// Normalizes an instruction file path and rejects traversal.
///
/// Backslashes become forward slashes; empty/`.`/`..` segments and absolute
/// paths are rejected so the path always stays inside the bundle root.
///
/// # Errors
///
/// Returns 422 when the path is empty, absolute, or contains traversal
/// segments.
pub fn validate_instruction_path(path: &str) -> Result<String, ApiError> {
    let normalized = path.replace('\\', "/");
    let looks_absolute = normalized.starts_with('/')
        || (normalized.len() >= 2
            && normalized.as_bytes()[0].is_ascii_alphabetic()
            && normalized.as_bytes()[1] == b':');
    if looks_absolute
        || normalized
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{
                "path": ["path"],
                "message": "Instructions file path must stay within the bundle root"
            }]),
        ));
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bundle_counts_match_roles() {
        assert_eq!(default_bundle_files("default").len(), 1);
        assert_eq!(default_bundle_files("engineer").len(), 1);
        assert_eq!(default_bundle_files("").len(), 1);
        assert_eq!(default_bundle_files("ceo").len(), 4);
        for (path, is_entry, content) in default_bundle_files("ceo") {
            assert!(!content.is_empty(), "{path} template must not be empty");
            assert_eq!(*is_entry, *path == "AGENTS.md");
        }
    }

    #[test]
    fn path_validation_accepts_relative_paths() {
        assert_eq!(validate_instruction_path("AGENTS.md").unwrap(), "AGENTS.md");
        assert_eq!(
            validate_instruction_path("docs/AGENTS.md").unwrap(),
            "docs/AGENTS.md"
        );
        // Backslashes normalize to forward slashes.
        assert_eq!(
            validate_instruction_path(r"docs\AGENTS.md").unwrap(),
            "docs/AGENTS.md"
        );
    }

    #[test]
    fn path_validation_rejects_traversal_and_absolute_paths() {
        for path in [
            "",
            ".",
            "..",
            "../x",
            "a/../b",
            "a/..",
            "/etc/passwd",
            "C:/windows",
            r"\etc\passwd",
            "a//b",
        ] {
            assert!(
                validate_instruction_path(path).is_err(),
                "path {path:?} should be rejected"
            );
        }
    }
}
