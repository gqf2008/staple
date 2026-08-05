//! Local CLI adapter: spawns a child process per run.
//!
//! The first built-in adapter proving the invoke → observe → cancel
//! lifecycle against a real process. `command` defaults to `sh -c` so any
//! shell task can run; override with a specific binary for Claude Code,
//! Codex, etc.

use std::{
    collections::HashMap,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures_core::Stream;
use tokio::{
    io::AsyncReadExt,
    process::{Child, Command},
    sync::{Mutex, Notify, mpsc},
    task::JoinHandle,
};
use uuid::Uuid;

use crate::contract::{
    AdapterError, AgentAdapter, InvocationInput, OutputStream, RunHandle, RunStatus,
};

/// Shared state for one managed child process.
struct ManagedProcess {
    #[allow(dead_code)]
    marker: (),
    status: Mutex<RunStatus>,
    notify: Arc<Notify>,
    /// Accumulated stdout (for observe / error messages).
    output: Mutex<String>,
    /// Incremental stdout sender (for stream subscribers); taken by the
    /// child task so the channel closes when the run finishes.
    tx: Mutex<Option<mpsc::UnboundedSender<String>>>,
    /// The single subscriber receiver (consumed by the first `stream` call).
    rx: Mutex<Option<mpsc::UnboundedReceiver<String>>>,
}

/// A stream over a run's mpsc receiver.
struct ReceiverStream {
    rx: mpsc::UnboundedReceiver<String>,
}

impl Stream for ReceiverStream {
    type Item = String;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // The channel closes when the run task drops the sender, ending the
        // stream for subscribers.
        self.rx.poll_recv(cx)
    }
}

/// Local CLI adapter configuration.
#[derive(Debug, Clone)]
pub struct CliAdapterConfig {
    /// Adapter type name (defaults to `cli_local`).
    pub name: String,
    /// Program to run (`sh` by default).
    pub program: String,
    /// Argument prefix for the task (`-c` for `sh`).
    pub args: Vec<String>,
}

impl Default for CliAdapterConfig {
    fn default() -> Self {
        Self {
            name: "cli_local".to_owned(),
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
        &self.config.name
    }

    async fn invoke(&self, input: InvocationInput) -> Result<RunHandle, AdapterError> {
        let run_id = Uuid::new_v4().to_string();
        let started_at = iso_now();
        let (tx, rx) = mpsc::unbounded_channel::<String>();
        let state = Arc::new(ManagedProcess {
            marker: (),
            status: Mutex::new(RunStatus::Running),
            notify: Arc::new(Notify::new()),
            output: Mutex::new(String::new()),
            tx: Mutex::new(Some(tx)),
            rx: Mutex::new(Some(rx)),
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

        let mut child = command
            .spawn()
            .map_err(|error| AdapterError::Invoke(error.to_string()))?;
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            run_child(child, stdout, stderr, state_clone).await;
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

    async fn stream(&self, run_id: &str) -> Result<OutputStream, AdapterError> {
        let runs = self.runs.lock().await;
        let (_, state) = runs
            .get(run_id)
            .ok_or_else(|| AdapterError::Observe("unknown run".to_owned()))?;
        let rx = state
            .rx
            .lock()
            .await
            .take()
            .ok_or_else(|| AdapterError::Observe("stream already consumed".to_owned()))?;
        Ok(Box::pin(ReceiverStream { rx }))
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

/// Reads a child's stdout/stderr incrementally, broadcasts stdout chunks,
/// and stores the final status when the child exits.
async fn run_child(
    mut child: Child,
    stdout: Option<tokio::process::ChildStdout>,
    stderr: Option<tokio::process::ChildStderr>,
    state: Arc<ManagedProcess>,
) {
    let mut output_buf = String::new();
    let mut error_buf = String::new();
    // Taking the sender means it is dropped when this task finishes, which
    // closes the stream for subscribers.
    let sender = state.tx.lock().await.take();

    if let Some(mut stdout) = stdout {
        let mut chunk = [0u8; 4096];
        loop {
            let read = tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::from_secs(3600)) => {
                    // No-op timeout keeps the select well-formed; reads return
                    // immediately when data is available.
                    0
                }
                result = stdout.read(&mut chunk) => match result {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(_) => break,
                },
            };
            if read == 0 {
                break;
            }
            let text = String::from_utf8_lossy(&chunk[..read]).to_string();
            output_buf.push_str(&text);
            if let Some(sender) = &sender {
                let _ = sender.send(text);
            }
        }
    }
    if let Some(mut stderr) = stderr {
        let mut chunk = [0u8; 4096];
        loop {
            match stderr.read(&mut chunk).await {
                Ok(0) => break,
                Ok(n) => error_buf.push_str(&String::from_utf8_lossy(&chunk[..n])),
                Err(_) => break,
            }
        }
    }

    let exit = child.wait().await;
    let mut guard = state.status.lock().await;
    *guard = match exit {
        Ok(status) if status.success() => RunStatus::Succeeded {
            output: output_buf.trim().to_owned(),
        },
        Ok(status) => RunStatus::Failed {
            error: format!(
                "exit {:?}: {}{}",
                status.code(),
                output_buf.trim(),
                error_buf.trim()
            )
            .trim()
            .to_owned(),
        },
        Err(error) => RunStatus::Failed {
            error: error.to_string(),
        },
    };
    *state.output.lock().await = output_buf;
    state.notify.notify_waiters();
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
            name: "cli_local".to_owned(),
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
    async fn stream_yields_incremental_output() {
        let adapter = adapter();
        let handle = adapter
            .invoke(InvocationInput {
                task: "printf 'one\ntwo\nthree\n'".to_owned(),
                cwd: None,
                env: vec![],
            })
            .await
            .unwrap();
        let mut stream = Box::pin(adapter.stream(&handle.run_id).await.unwrap());
        let mut collected = String::new();
        loop {
            let chunk = std::future::poll_fn(|cx| stream.as_mut().poll_next(cx)).await;
            match chunk {
                Some(chunk) => collected.push_str(&chunk),
                None => break,
            }
        }
        assert!(collected.contains("one"), "got: {collected:?}");
        assert!(collected.contains("three"), "got: {collected:?}");
        // observe still reports the final captured output.
        let status = wait_terminal(&adapter, &handle.run_id).await;
        match status {
            RunStatus::Succeeded { output } => {
                assert!(output.contains("one"), "output: {output:?}");
            }
            other => panic!("expected success, got {other:?}"),
        }
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
