//! HTTP adapter: talks to an HTTP agent runtime.
//!
//! Contract:
//! - `POST {base}/invoke` with `{task}` → `{run_id}`
//! - `GET  {base}/runs/{run_id}` → run status JSON
//! - `POST {base}/runs/{run_id}/cancel` → stops the run

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::contract::{
    AdapterError, AgentAdapter, InvocationInput, OutputStream, RunHandle, RunStatus,
};

/// HTTP adapter configuration.
#[derive(Debug, Clone)]
pub struct HttpAdapterConfig {
    /// Adapter type name (defaults to `http`).
    pub name: String,
    /// Base URL of the HTTP runtime.
    pub base_url: String,
}

/// HTTP adapter.
pub struct HttpAdapter {
    config: HttpAdapterConfig,
    client: reqwest::Client,
    _runs: Mutex<HashMap<String, ()>>,
}

impl HttpAdapter {
    /// Creates a new adapter.
    #[must_use]
    pub fn new(config: HttpAdapterConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
            _runs: Mutex::new(HashMap::new()),
        }
    }
}

#[derive(Serialize)]
struct InvokeRequest {
    task: String,
}

#[derive(Deserialize)]
struct InvokeResponse {
    run_id: String,
}

#[derive(Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum WireStatus {
    Running,
    Succeeded { output: String },
    Failed { error: String },
    Cancelled,
}

impl From<WireStatus> for RunStatus {
    fn from(wire: WireStatus) -> Self {
        match wire {
            WireStatus::Running => Self::Running,
            WireStatus::Succeeded { output } => Self::Succeeded { output },
            WireStatus::Failed { error } => Self::Failed { error },
            WireStatus::Cancelled => Self::Cancelled,
        }
    }
}

#[async_trait::async_trait]
impl AgentAdapter for HttpAdapter {
    fn name(&self) -> &str {
        &self.config.name
    }

    async fn invoke(&self, input: InvocationInput) -> Result<RunHandle, AdapterError> {
        let response = self
            .client
            .post(format!("{}/invoke", self.config.base_url))
            .json(&InvokeRequest { task: input.task })
            .send()
            .await
            .map_err(|error| AdapterError::Invoke(error.to_string()))?;
        let body: InvokeResponse = response
            .json()
            .await
            .map_err(|error| AdapterError::Invoke(error.to_string()))?;
        Ok(RunHandle {
            run_id: body.run_id,
            started_at: iso_now(),
        })
    }

    async fn stream(&self, _run_id: &str) -> Result<OutputStream, AdapterError> {
        Err(AdapterError::Observe(
            "streaming not supported for http adapter".to_owned(),
        ))
    }

    async fn observe(&self, run_id: &str) -> Result<RunStatus, AdapterError> {
        let response = self
            .client
            .get(format!("{}/runs/{}", self.config.base_url, run_id))
            .send()
            .await
            .map_err(|error| AdapterError::Observe(error.to_string()))?;
        if response.status().is_success() {
            let wire: WireStatus = response
                .json()
                .await
                .map_err(|error| AdapterError::Observe(error.to_string()))?;
            Ok(wire.into())
        } else {
            Err(AdapterError::Observe(format!("HTTP {}", response.status())))
        }
    }

    async fn cancel(&self, run_id: &str) -> Result<(), AdapterError> {
        let response = self
            .client
            .post(format!("{}/runs/{}/cancel", self.config.base_url, run_id))
            .send()
            .await
            .map_err(|error| AdapterError::Cancel(error.to_string()))?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(AdapterError::Cancel(format!("HTTP {}", response.status())))
        }
    }
}

fn iso_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock before epoch");
    format!("{:?}", now)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal HTTP runtime for tests.
    async fn serve(runs: std::sync::Arc<Mutex<HashMap<String, RunStatus>>>) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            loop {
                let Ok((mut stream, _)) = listener.accept().await else {
                    break;
                };
                let runs = runs.clone();
                tokio::spawn(async move {
                    use tokio::io::{AsyncReadExt, AsyncWriteExt};
                    let mut buf = vec![0u8; 4096];
                    let Ok(n) = stream.read(&mut buf).await else {
                        return;
                    };
                    let request = String::from_utf8_lossy(&buf[..n]).to_string();
                    let (status, body) = route(&request, &runs).await;
                    let response = format!(
                        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                });
            }
        });
        format!("http://{addr}")
    }

    async fn route(
        request: &str,
        runs: &Mutex<HashMap<String, RunStatus>>,
    ) -> (&'static str, String) {
        let first_line = request.lines().next().unwrap_or_default();
        let path = first_line.split_whitespace().nth(1).unwrap_or("/");
        match path {
            "/invoke" => {
                let mut runs = runs.lock().await;
                let run_id = format!("run-{}", runs.len() + 1);
                runs.insert(run_id.clone(), RunStatus::Running);
                ("200", format!(r#"{{"run_id":"{run_id}"}}"#))
            }
            p if p.starts_with("/runs/") && p.ends_with("/cancel") => {
                let run_id = p.trim_start_matches("/runs/").trim_end_matches("/cancel");
                let mut runs = runs.lock().await;
                if runs.contains_key(run_id) {
                    runs.insert(run_id.to_owned(), RunStatus::Cancelled);
                    ("200", "{}".to_owned())
                } else {
                    ("404", r#"{"error":"not found"}"#.to_owned())
                }
            }
            p if p.starts_with("/runs/") => {
                let run_id = p.trim_start_matches("/runs/");
                let runs = runs.lock().await;
                match runs.get(run_id) {
                    Some(RunStatus::Succeeded { output }) => (
                        "200",
                        format!(r#"{{"status":"succeeded","output":"{output}"}}"#),
                    ),
                    Some(RunStatus::Failed { error }) => {
                        ("200", format!(r#"{{"status":"failed","error":"{error}"}}"#))
                    }
                    Some(RunStatus::Cancelled) => ("200", r#"{"status":"cancelled"}"#.to_owned()),
                    Some(RunStatus::Running) | None => {
                        ("200", r#"{"status":"running"}"#.to_owned())
                    }
                }
            }
            _ => ("404", r#"{"error":"not found"}"#.to_owned()),
        }
    }

    #[tokio::test]
    async fn invoke_observe_cancel_lifecycle() {
        let runs = std::sync::Arc::new(Mutex::new(HashMap::new()));
        let base = serve(runs.clone()).await;
        let adapter = HttpAdapter::new(HttpAdapterConfig {
            name: "http".to_owned(),
            base_url: base,
        });

        let handle = adapter
            .invoke(InvocationInput {
                task: "do the thing".to_owned(),
                cwd: None,
                env: vec![],
            })
            .await
            .unwrap();
        assert!(handle.run_id.starts_with("run-"));

        assert_eq!(
            adapter.observe(&handle.run_id).await.unwrap(),
            RunStatus::Running
        );

        adapter.cancel(&handle.run_id).await.unwrap();
        assert_eq!(
            adapter.observe(&handle.run_id).await.unwrap(),
            RunStatus::Cancelled
        );

        // Unknown run: the runtime reports it as running (server default);
        // cancel of an unknown run is an error.
        assert!(adapter.cancel("nope").await.is_err());
    }
}
