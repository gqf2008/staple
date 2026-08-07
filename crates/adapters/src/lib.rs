//! Staple agent adapter contracts and implementations.
//!
//! Phase 4: the adapter contract (invoke / observe / cancel), a registry for
//! type-keyed discovery, and the first built-in adapters (local CLI and
//! HTTP).

pub mod cli;
pub mod contract;
pub mod http;
pub mod plugins;
pub mod registry;

pub use cli::{CliAdapter, CliAdapterConfig};
pub use contract::{
    AdapterError, AgentAdapter, InvocationInput, OutputEvent, OutputStream, ProbeResult, RunHandle,
    RunStatus,
};
pub use http::{HttpAdapter, HttpAdapterConfig};
pub use plugins::{
    CURRENT_CONTRACT_VERSION, PluginAdapterDecl, PluginError, PluginKind, PluginManifest,
    PluginReport,
};
pub use registry::AdapterRegistry;
