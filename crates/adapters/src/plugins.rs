//! External adapter plugin mechanism.
//!
//! Adapters can be declared in a JSON manifest (the Rust-side equivalent of
//! the upstream `adapter-plugins.json`). The contract is versioned; built-in
//! adapters can be shadowed by a plugin with the same type name. Plugin
//! failures produce explicit diagnostics — nothing fails silently.

use std::{fs, path::Path};

use serde::Deserialize;
use thiserror::Error;

use crate::{
    cli::{CliAdapter, CliAdapterConfig},
    contract::AgentAdapter,
    http::{HttpAdapter, HttpAdapterConfig},
    registry::AdapterRegistry,
};

/// The current plugin contract version. Plugins with a different
/// `contract_version` are rejected with a diagnostic.
pub const CURRENT_CONTRACT_VERSION: u32 = 1;

/// Adapter plugin manifest file.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    /// Manifest schema version (must equal [`CURRENT_CONTRACT_VERSION`]).
    pub contract_version: u32,
    /// Adapter declarations.
    #[serde(default)]
    pub adapters: Vec<PluginAdapterDecl>,
}

/// One adapter declaration in the manifest.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginAdapterDecl {
    /// Adapter type name (shadows a built-in with the same name).
    pub r#type: String,
    /// Plugin kind.
    pub kind: PluginKind,
    /// Base URL for `http` plugins.
    #[serde(default)]
    pub base_url: Option<String>,
    /// Program for `cli` plugins.
    #[serde(default)]
    pub program: Option<String>,
    /// Args for `cli` plugins.
    #[serde(default)]
    pub args: Vec<String>,
}

/// Plugin adapter kind.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PluginKind {
    /// HTTP runtime plugin.
    Http,
    /// Local CLI plugin.
    Cli,
}

/// Diagnostic for one plugin (or the manifest itself).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginReport {
    /// Adapter type name (`*manifest*` for manifest-level problems).
    pub r#type: String,
    /// Whether the plugin was loaded successfully.
    pub loaded: bool,
    /// Diagnostic message when not loaded.
    pub error: Option<String>,
}

/// Plugin loading errors (manifest-level failures).
#[derive(Debug, Error)]
pub enum PluginError {
    /// The manifest file could not be read.
    #[error("cannot read plugin manifest {0}: {1}")]
    Read(String, #[source] std::io::Error),
    /// The manifest is not valid JSON.
    #[error("invalid plugin manifest JSON: {0}")]
    InvalidJson(String),
    /// The manifest contract version is unsupported.
    #[error("unsupported plugin contract version {0}; expected {CURRENT_CONTRACT_VERSION}")]
    UnsupportedContract(u32),
}

impl PluginManifest {
    /// Loads and validates the manifest at `path`.
    ///
    /// # Errors
    ///
    /// Returns [`PluginError`] for read/JSON/contract failures.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, PluginError> {
        let path = path.as_ref();
        let raw = fs::read_to_string(path)
            .map_err(|error| PluginError::Read(path.display().to_string(), error))?;
        let manifest: Self = serde_json::from_str(&raw)
            .map_err(|error| PluginError::InvalidJson(error.to_string()))?;
        if manifest.contract_version != CURRENT_CONTRACT_VERSION {
            return Err(PluginError::UnsupportedContract(manifest.contract_version));
        }
        Ok(manifest)
    }

    /// Builds an adapter from one declaration.
    fn build(&self, decl: &PluginAdapterDecl) -> Result<Box<dyn AgentAdapter>, String> {
        match decl.kind {
            PluginKind::Http => {
                let base_url = decl
                    .base_url
                    .clone()
                    .ok_or_else(|| "http plugin requires baseUrl".to_owned())?;
                Ok(Box::new(HttpAdapter::new(HttpAdapterConfig {
                    name: decl.r#type.clone(),
                    base_url,
                })))
            }
            PluginKind::Cli => Ok(Box::new(CliAdapter::new(CliAdapterConfig {
                name: decl.r#type.clone(),
                program: decl.program.clone().unwrap_or_else(|| "sh".to_owned()),
                args: if decl.args.is_empty() {
                    vec!["-c".to_owned()]
                } else {
                    decl.args.clone()
                },
            }))),
        }
    }

    /// Loads every plugin in this manifest, returning per-plugin diagnostics.
    /// Loaded adapters are registered into `registry` (shadowing built-ins
    /// with the same type name).
    pub fn register_into(&self, registry: &mut AdapterRegistry) -> Vec<PluginReport> {
        let mut reports = Vec::new();
        for decl in &self.adapters {
            match self.build(decl) {
                Ok(adapter) => {
                    registry.register(adapter);
                    reports.push(PluginReport {
                        r#type: decl.r#type.clone(),
                        loaded: true,
                        error: None,
                    });
                }
                Err(message) => reports.push(PluginReport {
                    r#type: decl.r#type.clone(),
                    loaded: false,
                    error: Some(message),
                }),
            }
        }
        reports
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;
    use crate::contract::{InvocationInput, RunStatus};

    fn write_manifest(json: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(json.as_bytes()).unwrap();
        file
    }

    #[test]
    fn loads_cli_plugin_and_runs_it() {
        let file = write_manifest(
            r#"{
                "contractVersion": 1,
                "adapters": [
                    { "type": "plugin_echo", "kind": "cli" }
                ]
            }"#,
        );
        let manifest = PluginManifest::load(file.path()).unwrap();
        let mut registry = AdapterRegistry::new();
        let reports = manifest.register_into(&mut registry);
        assert_eq!(reports.len(), 1);
        assert!(reports[0].loaded);

        let adapter = registry.get("plugin_echo").expect("plugin registered");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let handle = rt.block_on(adapter.invoke(InvocationInput {
            task: "echo plugin-ok".to_owned(),
            cwd: None,
            env: vec![],
        }));
        let run_id = handle.unwrap().run_id;
        // Poll until terminal.
        let status = rt.block_on(async {
            for _ in 0..100 {
                let status = adapter.observe(&run_id).await.unwrap();
                if status.is_terminal() {
                    return status;
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
            panic!("run did not finish");
        });
        assert_eq!(
            status,
            RunStatus::Succeeded {
                output: "plugin-ok".to_owned()
            }
        );
    }

    #[test]
    fn plugin_shadows_builtin() {
        let file = write_manifest(
            r#"{
                "contractVersion": 1,
                "adapters": [
                    { "type": "cli_local", "kind": "cli", "program": "echo", "args": [] }
                ]
            }"#,
        );
        let manifest = PluginManifest::load(file.path()).unwrap();
        let mut registry = AdapterRegistry::new();
        registry.register(Box::new(CliAdapter::new(CliAdapterConfig::default())));
        let reports = manifest.register_into(&mut registry);
        assert!(reports[0].loaded);

        // The registry now serves the plugin's echo-based adapter, which
        // echoes its own program name instead of running the task.
        let adapter = registry.get("cli_local").unwrap();
        assert_eq!(adapter.name(), "cli_local");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let handle = rt.block_on(adapter.invoke(InvocationInput {
            task: "ignored".to_owned(),
            cwd: None,
            env: vec![],
        }));
        let run_id = handle.unwrap().run_id;
        let status = rt.block_on(async {
            for _ in 0..100 {
                let status = adapter.observe(&run_id).await.unwrap();
                if status.is_terminal() {
                    return status;
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
            panic!("run did not finish");
        });
        assert!(matches!(status, RunStatus::Succeeded { .. }));
    }

    #[test]
    fn rejects_unsupported_contract_version() {
        let file = write_manifest(
            r#"{
                "contractVersion": 99,
                "adapters": []
            }"#,
        );
        let error = PluginManifest::load(file.path()).unwrap_err();
        assert!(matches!(error, PluginError::UnsupportedContract(99)));
    }

    #[test]
    fn reports_invalid_plugin_diagnostic() {
        let file = write_manifest(
            r#"{
                "contractVersion": 1,
                "adapters": [
                    { "type": "bad_http", "kind": "http" }
                ]
            }"#,
        );
        let manifest = PluginManifest::load(file.path()).unwrap();
        let mut registry = AdapterRegistry::new();
        let reports = manifest.register_into(&mut registry);
        assert!(!reports[0].loaded);
        assert!(reports[0].error.as_deref().unwrap().contains("baseUrl"));
        assert!(registry.get("bad_http").is_none());
    }

    #[test]
    fn missing_manifest_is_an_explicit_error() {
        let error = PluginManifest::load("/nonexistent/plugins.json").unwrap_err();
        assert!(matches!(error, PluginError::Read(..)));
    }
}
