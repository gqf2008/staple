//! Staple agent adapter contracts and implementations.
//!
//! Phase 4: the adapter contract (invoke / observe / cancel), a registry for
//! type-keyed discovery, and the first built-in adapters (local CLI and
//! HTTP).

pub mod cli;
pub mod contract;
pub mod http;
pub mod registry;

pub use cli::{CliAdapter, CliAdapterConfig};
pub use contract::{AdapterError, AgentAdapter, InvocationInput, RunHandle, RunStatus};
pub use http::{HttpAdapter, HttpAdapterConfig};
pub use registry::AdapterRegistry;
