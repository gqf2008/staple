//! Staple domain models and business rules.
//!
//! This crate is intentionally I/O-free: pure types, validation, and rules
//! that the rest of the system builds on.

pub mod permissions;

pub use permissions::{
    AgentHierarchyRow, PERMISSION_KEYS, agent_is_in_subtree, is_permission_key, scope_allows,
};
