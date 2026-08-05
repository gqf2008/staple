//! Adapter registry: type-keyed discovery of adapters.

use std::collections::HashMap;

use crate::contract::AgentAdapter;

/// Registry of adapters keyed by their `name()`.
#[derive(Default)]
pub struct AdapterRegistry {
    adapters: HashMap<String, Box<dyn AgentAdapter>>,
}

impl AdapterRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an adapter, replacing any existing adapter with the same
    /// name.
    pub fn register(&mut self, adapter: Box<dyn AgentAdapter>) {
        let name = adapter.name().to_owned();
        self.adapters.insert(name, adapter);
    }

    /// Finds an adapter by type name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&dyn AgentAdapter> {
        self.adapters.get(name).map(|adapter| adapter.as_ref())
    }

    /// Lists registered adapter type names.
    #[must_use]
    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.adapters.keys().cloned().collect();
        names.sort();
        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{
        AdapterError, AgentAdapter, InvocationInput, OutputStream, RunHandle, RunStatus,
    };

    struct FakeAdapter;

    #[async_trait::async_trait]
    impl AgentAdapter for FakeAdapter {
        fn name(&self) -> &str {
            "fake"
        }

        async fn invoke(&self, _input: InvocationInput) -> Result<RunHandle, AdapterError> {
            Ok(RunHandle {
                run_id: "r1".to_owned(),
                started_at: "now".to_owned(),
            })
        }

        async fn observe(&self, _run_id: &str) -> Result<RunStatus, AdapterError> {
            Ok(RunStatus::Succeeded {
                output: "ok".to_owned(),
            })
        }

        async fn stream(&self, _run_id: &str) -> Result<OutputStream, AdapterError> {
            Err(AdapterError::Observe(
                "streaming not supported for fake adapter".to_owned(),
            ))
        }

        async fn cancel(&self, _run_id: &str) -> Result<(), AdapterError> {
            Ok(())
        }
    }

    #[test]
    fn register_discover_and_replace() {
        let mut registry = AdapterRegistry::new();
        assert!(registry.get("fake").is_none());

        registry.register(Box::new(FakeAdapter));
        assert!(registry.get("fake").is_some());
        assert_eq!(registry.names(), vec!["fake".to_owned()]);

        registry.register(Box::new(FakeAdapter));
        assert_eq!(registry.names(), vec!["fake".to_owned()]);
    }
}
