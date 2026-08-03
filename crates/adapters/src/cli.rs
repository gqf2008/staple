//! Local CLI adapter: spawns a child process per run.
//!
//! The first built-in adapter proving the invoke → observe → cancel
//! lifecycle against a real process. `command` defaults to `sh -c` so any
//! shell task can run; override with a specific binary for Claude Code,
//! Codex, etc.

use std::{collections::HashMap, sync::Arc};

use tokio::{
    process::Command,
    sync::{Mutex, Notify},
    task::JoinHandle,
};
use uuid::Uuid;

use crate::contract::{AdapterError, AgentAdapter, InvocationInput, RunHandle, RunStatus};

/// Shared state for one managed child process.
struct ManagedProcess {
    #[allow(dead_code)]
    marker: (),
    status: Mutex<RunStatus>,
    notify: Arc<Notify>,
}

/// Local CLI adapter configuration.
#[derive(Debug, Clone)]
pub struct CliAdapterConfig {
    /// Program to run (`sh` by default).
    pub program: String,
    /// Argument prefix for the task (`-c` for `sh`).
    pub args: Vec<String>,
}

impl Default for CliAdapterConfig {
    fn default() -> Self {
        Self {
            program: "sh".to_owned(),
            args: vec!["-c".to_owned()],
        }
    }
}

/// A managed child process: its task handle and shared state.
type ManagedRun = (JoinHandle<()>, Arc<ManagedProcess>);

/// Local CLI adapter.
pub struct CliAdapter {
    config: CliAdapterConfig,
    runs: Mutex<HashMap<String, ManagedRun>>,
}

impl CliAdapter {
    /// Creates a new adapter with the given config.
    #[must_use]
    pub fn new(config: CliAdapterConfig) -> Self {
        Self {
            config,
            runs: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl AgentAdapter for CliAdapter {
    fn name(&self) -> &str {
        "cli_local"
    }

    async fn invoke(&self, input: InvocationInput) -> Result<RunHandle, AdapterError> {
        let run_id = Uuid::new_v4().to_string();
        let started_at = iso_now();
        let state = Arc::new(ManagedProcess {
            marker: (),
            status: Mutex::new(RunStatus::Running),
            notify: Arc::new(Notify::new()),
        });

        let mut command = Command::new(&self.config.program);
        command.args(&self.config.args).arg(&input.task);
        if let Some(cwd) = &input.cwd {
            command.current_dir(cwd);
        }
        command.envs(input.env.iter().cloned());
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());
        command.kill_on_drop(true);

        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            let output = command.output().await;
            let mut guard = state_clone.status.lock().await;
            *guard = match output {
                Ok(output) if output.status.success() => RunStatus::Succeeded {
                    output: String::from_utf8_lossy(&output.stdout).trim().to_owned(),
                },
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    RunStatus::Failed {
                        error: format!("exit {:?}: {}{}", output.status.code(), stdout, stderr)
                            .trim()
                            .to_owned(),
                    }
                }
                Err(error) => RunStatus::Failed {
                    error: error.to_string(),
                },
            };
            state_clone.notify.notify_waiters();
        });

        self.runs
            .lock()
            .await
            .insert(run_id.clone(), (handle, state));
        Ok(RunHandle { run_id, started_at })
    }

    async fn observe(&self, run_id: &str) -> Result<RunStatus, AdapterError> {
        let runs = self.runs.lock().await;
        let (_, state) = runs
            .get(run_id)
            .ok_or_else(|| AdapterError::Observe("unknown run".to_owned()))?;
        Ok(state.status.lock().await.clone())
    }

    async fn cancel(&self, run_id: &str) -> Result<(), AdapterError> {
        let runs = self.runs.lock().await;
        let (handle, state) = runs
            .get(run_id)
            .ok_or_else(|| AdapterError::Cancel("unknown run".to_owned()))?;
        handle.abort();
        *state.status.lock().await = RunStatus::Cancelled;
        state.notify.notify_waiters();
        Ok(())
    }
}

fn iso_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock before epoch");
    let secs = now.as_secs();
    let millis = now.subsec_millis();
    let days = secs / 86_400;
    let (y, m, d) = civil_from_days(days as i64);
    let (hh, mm, ss) = ((secs % 86_400) / 3600, (secs % 3600) / 60, secs % 60);
    format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}.{millis:03}Z")
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    // Howard Hinnant's civil_from_days algorithm.
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as i64;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as i64;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn adapter() -> CliAdapter {
        CliAdapter::new(CliAdapterConfig {
            program: "sh".to_owned(),
            args: vec!["-c".to_owned()],
        })
    }

    /// Polls observe until a terminal status arrives (the child task may not
    /// have finished by the time invoke returns).
    async fn wait_terminal(adapter: &CliAdapter, run_id: &str) -> RunStatus {
        for _ in 0..100 {
            let status = adapter.observe(run_id).await.unwrap();
            if status.is_terminal() {
                return status;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        panic!("run did not finish in time");
    }

    #[tokio::test]
    async fn invoke_observe_success() {
        let adapter = adapter();
        let handle = adapter
            .invoke(InvocationInput {
                task: "echo hello-from-cli".to_owned(),
                cwd: None,
                env: vec![],
            })
            .await
            .unwrap();
        match wait_terminal(&adapter, &handle.run_id).await {
            RunStatus::Succeeded { output } => {
                assert_eq!(output, "hello-from-cli");
            }
            other => panic!("expected success, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn invoke_observe_failure() {
        let adapter = adapter();
        let handle = adapter
            .invoke(InvocationInput {
                task: "echo boom >&2; exit 3".to_owned(),
                cwd: None,
                env: vec![],
            })
            .await
            .unwrap();
        let status = wait_terminal(&adapter, &handle.run_id).await;
        assert!(matches!(status, RunStatus::Failed { .. }));
    }

    #[tokio::test]
    async fn invoke_observe_cancel() {
        let adapter = adapter();
        let handle = adapter
            .invoke(InvocationInput {
                task: "sleep 30".to_owned(),
                cwd: None,
                env: vec![],
            })
            .await
            .unwrap();
        assert!(matches!(
            adapter.observe(&handle.run_id).await.unwrap(),
            RunStatus::Running
        ));
        adapter.cancel(&handle.run_id).await.unwrap();
        assert_eq!(
            adapter.observe(&handle.run_id).await.unwrap(),
            RunStatus::Cancelled
        );
    }

    #[test]
    fn iso_now_is_iso8601() {
        let now = iso_now();
        assert!(now.ends_with('Z'));
        assert!(now.contains('T'));
        assert_eq!(now.len(), 24);
    }
}
