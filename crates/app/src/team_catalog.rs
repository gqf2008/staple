//! Teams catalog service: reads the upstream teams-catalog package
//! (generated/catalog.json + team directories) and aggregates installed
//! teams from agent metadata provenance.

use serde::Serialize;
use std::path::{Path, PathBuf};

/// One catalog team (subset of upstream `CatalogTeam`).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogTeam {
    /// Catalog id (e.g. `paperclipai:bundled:...`).
    pub id: String,
    /// Catalog key.
    pub key: String,
    /// Kind (`bundled` | `optional` | ...).
    pub kind: String,
    /// Category.
    pub category: String,
    /// Slug.
    pub slug: String,
    /// Display name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Package-relative team directory.
    pub path: String,
    /// Entrypoint file (e.g. `TEAM.md`).
    pub entrypoint: String,
    /// Content hash (`sha256:...`).
    pub content_hash: String,
    /// Counts JSON (agents/projects/tasks/routines/skills).
    pub counts: serde_json::Value,
    /// Tags.
    pub tags: Vec<String>,
    /// Files within the team package.
    pub files: Vec<String>,
}

/// A team file read result.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogTeamFileDetail {
    /// Relative path.
    pub path: String,
    /// Content type (`text` | `base64`).
    pub encoding: String,
    /// Text content (truncated) or base64 data.
    pub data: String,
    /// Whether the file was truncated.
    pub truncated: bool,
    /// Byte size.
    pub byte_size: u64,
}

/// An installed catalog team aggregated from agent provenance.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledCatalogTeam {
    /// Catalog id.
    pub catalog_id: String,
    /// Catalog key.
    pub catalog_key: Option<String>,
    /// Whether the team still exists in the catalog.
    pub present: bool,
    /// Current catalog content hash.
    pub current_content_hash: Option<String>,
    /// Installed origin hashes (sorted).
    pub installed_origin_hashes: Vec<String>,
    /// Number of agents installed from this team.
    pub agent_count: i64,
    /// Whether any installed origin hash differs from the current hash.
    pub out_of_date: bool,
}

const MAX_FILE_BYTES: u64 = 256 * 1024;

/// Resolves the catalog root: `PAPERCLIP_TEAMS_CATALOG_DIR` or the default
/// package location relative to this crate (repo `packages/teams-catalog`).
pub fn catalog_root() -> PathBuf {
    if let Some(root) = std::env::var_os("PAPERCLIP_TEAMS_CATALOG_DIR") {
        let path = PathBuf::from(root);
        if path.join("generated/catalog.json").exists() {
            return path;
        }
    }
    let default = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/teams-catalog");
    if default.join("generated/catalog.json").exists() {
        default
    } else {
        PathBuf::new()
    }
}

fn manifest_path() -> Option<PathBuf> {
    let root = catalog_root();
    if root.as_os_str().is_empty() {
        return None;
    }
    let manifest = root.join("generated/catalog.json");
    manifest.exists().then_some(manifest)
}

/// Lists all catalog teams. Returns an empty list when no catalog package
/// is present.
pub fn list() -> Vec<CatalogTeam> {
    let Some(manifest) = manifest_path() else {
        return Vec::new();
    };
    let Ok(text) = std::fs::read_to_string(&manifest) else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Vec::new();
    };
    let Some(teams) = value.get("teams").and_then(serde_json::Value::as_array) else {
        return Vec::new();
    };
    teams
        .iter()
        .filter_map(|team| {
            let id = team.get("id")?.as_str()?.to_owned();
            let key = team
                .get("key")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_owned();
            let kind = team
                .get("kind")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_owned();
            let category = team
                .get("category")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_owned();
            let slug = team
                .get("slug")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_owned();
            let name = team
                .get("name")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_owned();
            let description = team
                .get("description")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_owned();
            let path = team
                .get("path")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_owned();
            let entrypoint = team
                .get("entrypoint")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("TEAM.md")
                .to_owned();
            let content_hash = team
                .get("contentHash")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_owned();
            let counts = team
                .get("counts")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));
            let tags = team
                .get("tags")
                .and_then(serde_json::Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(serde_json::Value::as_str)
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default();
            let files = team
                .get("files")
                .and_then(serde_json::Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(serde_json::Value::as_str)
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default();
            Some(CatalogTeam {
                id,
                key,
                kind,
                category,
                slug,
                name,
                description,
                path,
                entrypoint,
                content_hash,
                counts,
                tags,
                files,
            })
        })
        .collect()
}

/// Finds a team by id or key.
#[must_use]
pub fn detail(catalog_ref: &str) -> Option<CatalogTeam> {
    list()
        .into_iter()
        .find(|team| team.id == catalog_ref || team.key == catalog_ref)
}

/// Reads a file from a team package.
#[must_use]
pub fn files(catalog_ref: &str, relative_path: &str) -> Option<CatalogTeamFileDetail> {
    let team = detail(catalog_ref)?;
    let root = catalog_root();
    let path = root.join(&team.path).join(relative_path);
    if !path.exists() || !path.is_file() {
        return None;
    }
    let metadata = std::fs::metadata(&path).ok()?;
    let byte_size = metadata.len();
    let bytes = std::fs::read(&path).ok()?;
    let truncated = bytes.len() as u64 > MAX_FILE_BYTES;
    let slice = if truncated {
        &bytes[..MAX_FILE_BYTES as usize]
    } else {
        &bytes[..]
    };
    if let Ok(text) = std::str::from_utf8(slice) {
        Some(CatalogTeamFileDetail {
            path: relative_path.to_owned(),
            encoding: "text".to_owned(),
            data: text.to_owned(),
            truncated,
            byte_size,
        })
    } else {
        Some(CatalogTeamFileDetail {
            path: relative_path.to_owned(),
            encoding: "base64".to_owned(),
            data: base64_encode(slice),
            truncated,
            byte_size,
        })
    }
}

fn base64_encode(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// Reads the catalog provenance from an agent's metadata.
pub fn read_catalog_team_provenance(
    metadata: &serde_json::Value,
) -> Option<(String, Option<String>, Option<String>)> {
    let catalog_team = metadata.get("paperclip")?.get("catalogTeam")?;
    let catalog_id = catalog_team.get("catalogId")?.as_str()?.to_owned();
    let catalog_key = catalog_team
        .get("catalogKey")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);
    let origin_hash = catalog_team
        .get("originHash")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);
    Some((catalog_id, catalog_key, origin_hash))
}

/// Aggregates installed catalog teams from company agents (by provenance).
pub fn installed(agents: &[staple_data::AgentRecord]) -> Vec<InstalledCatalogTeam> {
    let teams = list();
    let current_by_id: std::collections::HashMap<String, CatalogTeam> = teams
        .into_iter()
        .map(|team| (team.id.clone(), team))
        .collect();
    let mut by_catalog_id: std::collections::BTreeMap<String, InstalledCatalogTeam> =
        std::collections::BTreeMap::new();
    for agent in agents {
        let Some((catalog_id, catalog_key, origin_hash)) =
            read_catalog_team_provenance(&agent.metadata)
        else {
            continue;
        };
        let entry =
            by_catalog_id
                .entry(catalog_id.clone())
                .or_insert_with(|| InstalledCatalogTeam {
                    catalog_id,
                    catalog_key: None,
                    present: false,
                    current_content_hash: None,
                    installed_origin_hashes: Vec::new(),
                    agent_count: 0,
                    out_of_date: false,
                });
        entry.agent_count += 1;
        if entry.catalog_key.is_none() && catalog_key.is_some() {
            entry.catalog_key = catalog_key;
        }
        if let Some(hash) = origin_hash
            && !entry.installed_origin_hashes.contains(&hash)
        {
            entry.installed_origin_hashes.push(hash);
        }
    }
    let mut result: Vec<InstalledCatalogTeam> = Vec::new();
    for (catalog_id, mut entry) in by_catalog_id {
        let current = current_by_id.get(&catalog_id);
        entry.present = current.is_some();
        entry.current_content_hash = current.map(|team| team.content_hash.clone());
        if entry.catalog_key.is_none() {
            entry.catalog_key = current.map(|team| team.key.clone());
        }
        let current_hash = entry.current_content_hash.clone();
        entry.out_of_date = entry.present
            && current_hash.is_some()
            && !entry.installed_origin_hashes.is_empty()
            && entry
                .installed_origin_hashes
                .iter()
                .any(|hash| Some(hash.clone()) != current_hash);
        entry.installed_origin_hashes.sort();
        result.push(entry);
    }
    result
}

/// Canonicalizes a catalog reference for path use (id/key with `/` and `:`).
#[must_use]
pub fn normalize_catalog_ref(reference: &str) -> String {
    reference
        .chars()
        .map(|ch| {
            if ch.is_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

/// Validates that a relative file path stays inside the team directory.
#[must_use]
pub fn safe_relative_path(path: &str, team_dir: &Path, root: &Path) -> Option<PathBuf> {
    let candidate = root.join(team_dir).join(path);
    let canonical_team = root.join(team_dir).canonicalize().ok()?;
    let canonical_candidate = candidate.canonicalize().ok()?;
    canonical_candidate
        .starts_with(&canonical_team)
        .then_some(candidate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn seed_catalog() -> tempfile::TempDir {
        let dir = tempfile::TempDir::new().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("generated")).unwrap();
        let team_dir = root.join("catalog/bundled/acme/core-team");
        fs::create_dir_all(&team_dir).unwrap();
        fs::write(team_dir.join("TEAM.md"), "# Core Team\n\nhello").unwrap();
        fs::write(team_dir.join("AGENTS.md"), "# Agents").unwrap();
        let manifest = serde_json::json!({
            "schemaVersion": 1,
            "packageName": "@test/teams-catalog",
            "teams": [{
                "id": "test:bundled:acme:core-team",
                "key": "test/bundled/acme/core-team",
                "kind": "bundled",
                "category": "acme",
                "slug": "core-team",
                "name": "Core Team",
                "description": "A test team",
                "path": "catalog/bundled/acme/core-team",
                "entrypoint": "TEAM.md",
                "contentHash": "sha256:abc",
                "counts": { "agents": 2 },
                "tags": ["default"],
                "files": ["TEAM.md", "AGENTS.md"]
            }]
        });
        fs::write(
            root.join("generated/catalog.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        unsafe {
            std::env::set_var("PAPERCLIP_TEAMS_CATALOG_DIR", root);
        }
        dir
    }

    fn agent_with_metadata(metadata: serde_json::Value) -> staple_data::AgentRecord {
        staple_data::AgentRecord {
            id: "a1".to_owned(),
            company_id: "c1".to_owned(),
            name: "one".to_owned(),
            role: "worker".to_owned(),
            title: None,
            icon: None,
            status: "idle".to_owned(),
            reports_to: None,
            adapter_type: "cli".to_owned(),
            budget_monthly_cents: 0,
            spent_monthly_cents: 0,
            pause_reason: None,
            default_environment_id: None,
            error_reason: None,
            last_heartbeat_at: None,
            metadata,
            created_at: "2026-08-05T00:00:00.000Z".to_owned(),
        }
    }

    #[test]
    fn list_detail_and_files_from_seeded_catalog() {
        let _dir = seed_catalog();
        let teams = list();
        assert_eq!(teams.len(), 1);
        assert_eq!(teams[0].name, "Core Team");
        assert_eq!(teams[0].content_hash, "sha256:abc");
        let team = detail("test:bundled:acme:core-team").expect("team");
        assert_eq!(team.slug, "core-team");
        let by_key = detail("test/bundled/acme/core-team").expect("team by key");
        assert_eq!(by_key.id, team.id);
        let file = files("test:bundled:acme:core-team", "TEAM.md").expect("file");
        assert_eq!(file.encoding, "text");
        assert!(file.data.contains("Core Team"));
        let agents = files("test:bundled:acme:core-team", "AGENTS.md").expect("agents");
        assert_eq!(agents.encoding, "text");
        assert!(detail("missing").is_none());
        assert!(files("test:bundled:acme:core-team", "missing.md").is_none());
        unsafe {
            std::env::remove_var("PAPERCLIP_TEAMS_CATALOG_DIR");
        }
    }

    #[test]
    fn installed_aggregates_provenance() {
        let _dir = seed_catalog();
        let agents = vec![
            agent_with_metadata(serde_json::json!({
                "paperclip": { "catalogTeam": {
                    "catalogId": "test:bundled:acme:core-team",
                    "catalogKey": "test/bundled/acme/core-team",
                    "originHash": "sha256:old",
                } }
            })),
            agent_with_metadata(serde_json::json!({
                "paperclip": { "catalogTeam": {
                    "catalogId": "test:bundled:acme:core-team",
                    "originHash": "sha256:old",
                } }
            })),
        ];
        let result = installed(&agents);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].agent_count, 2);
        assert!(result[0].present);
        assert_eq!(
            result[0].current_content_hash.as_deref(),
            Some("sha256:abc")
        );
        assert_eq!(
            result[0].installed_origin_hashes,
            vec!["sha256:old".to_owned()]
        );
        assert!(result[0].out_of_date);
        unsafe {
            std::env::remove_var("PAPERCLIP_TEAMS_CATALOG_DIR");
        }
    }
}
