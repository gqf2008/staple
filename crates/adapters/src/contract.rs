//! Adapter contract: invoke / observe / cancel.
//!
//! Mirrors the upstream heartbeat semantics: a run is created by `invoke`,
//! its progress is read by `observe`, and it can be stopped by `cancel`.

use std::{path::PathBuf, pin::Pin};

use futures_core::Stream;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result of an adapter environment probe.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeResult {
    /// Whether the adapter environment is usable.
    pub available: bool,
    /// Human-readable detail (e.g. which binary was found, or the error).
    pub detail: String,
}

/// Adapter errors.
#[derive(Debug, Error)]
pub enum AdapterError {
    /// The invocation could not be started.
    #[error("invoke failed: {0}")]
    Invoke(String),
    /// Observing the run failed.
    #[error("observe failed: {0}")]
    Observe(String),
    /// Cancelling the run failed.
    #[error("cancel failed: {0}")]
    Cancel(String),
}

/// Input for invoking a run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationInput {
    /// The task/instructions for the run.
    pub task: String,
    /// Working directory for local adapters.
    pub cwd: Option<PathBuf>,
    /// Extra environment variables.
    #[serde(default)]
    pub env: Vec<(String, String)>,
}

/// Handle returned by `invoke`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunHandle {
    /// Stable run id.
    pub run_id: String,
    /// ISO 8601 start time.
    pub started_at: String,
}

/// Observed run status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RunStatus {
    /// Still executing.
    Running,
    /// Finished successfully.
    Succeeded {
        /// Captured output.
        output: String,
    },
    /// Finished with an error.
    Failed {
        /// Error message.
        error: String,
    },
    /// Cancelled before completion.
    Cancelled,
}

/// One incremental output event from a run.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum OutputEvent {
    /// Regular assistant text.
    Delta {
        /// Text content.
        content: String,
    },
    /// Structured tool invocation parsed from the transcript (rendered as a
    /// collapsible tool accordion in Board Chat).
    ToolCall(crate::tool_call::ToolCall),
    /// Tool / diagnostic stderr output (rendered as a collapsible block).
    Stderr {
        /// Diagnostic text.
        content: String,
        /// Optional block label (frontend defaults to "stderr").
        name: Option<String>,
    },
}

/// Incremental output stream for a run (events as they are produced).
pub type OutputStream = Pin<Box<dyn Stream<Item = OutputEvent> + Send + 'static>>;

/// The adapter contract implemented by every built-in and plugin adapter.
#[async_trait::async_trait]
pub trait AgentAdapter: Send + Sync {
    /// Adapter type name (e.g. `codex_local`, `http`, `webhook`).
    fn name(&self) -> &str;

    /// Starts a run.
    ///
    /// # Errors
    ///
    /// Returns [`AdapterError`] when the run cannot be started.
    async fn invoke(&self, input: InvocationInput) -> Result<RunHandle, AdapterError>;

    /// Reads the current status of a run.
    ///
    /// # Errors
    ///
    /// Returns [`AdapterError`] when the run is unknown or unreadable.
    async fn observe(&self, run_id: &str) -> Result<RunStatus, AdapterError>;

    /// Subscribes to a run's incremental output (chunks as they are
    /// produced; the stream ends when the run finishes).
    ///
    /// # Errors
    ///
    /// Returns [`AdapterError`] when the run is unknown.
    async fn stream(&self, run_id: &str) -> Result<OutputStream, AdapterError>;

    /// Stops a run.
    ///
    /// # Errors
    ///
    /// Returns [`AdapterError`] when the run cannot be cancelled.
    async fn cancel(&self, run_id: &str) -> Result<(), AdapterError>;

    /// Probes whether the adapter's environment is usable (e.g. its CLI is
    /// installed and responds). The default reports the adapter as
    /// registered; adapters with real environments override this.
    ///
    /// # Errors
    ///
    /// Returns [`AdapterError`] when the probe itself cannot run.
    async fn probe(&self) -> Result<ProbeResult, AdapterError> {
        Ok(ProbeResult {
            available: true,
            detail: "adapter registered".to_owned(),
        })
    }
}

/// Convenience: `?`-free status check.
impl RunStatus {
    /// Whether the run has finished.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        !matches!(self, Self::Running)
    }
}

#[cfg(test)]
mod tests {
    use super::OutputEvent;

    #[test]
    fn output_event_wire_shape() {
        assert_eq!(
            serde_json::to_string(&OutputEvent::Delta {
                content: "hi".to_owned()
            })
            .unwrap(),
            r#"{"type":"delta","content":"hi"}"#
        );
        assert_eq!(
            serde_json::to_string(&OutputEvent::ToolCall(crate::tool_call::ToolCall {
                id: "t1".to_owned(),
                name: "shell".to_owned(),
                arguments: serde_json::json!({ "command": "ls" }),
            }))
            .unwrap(),
            r#"{"type":"toolCall","id":"t1","name":"shell","arguments":{"command":"ls"}}"#
        );
        assert_eq!(
            serde_json::to_string(&OutputEvent::Stderr {
                content: "oops".to_owned(),
                name: None
            })
            .unwrap(),
            r#"{"type":"stderr","content":"oops","name":null}"#
        );
    }
}
