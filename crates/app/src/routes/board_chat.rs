//! Board Concierge chat: a streaming SSE endpoint powered by the built-in
//! board-member skill (upstream `/board/chat/stream`).

use std::convert::Infallible;

use bytes::Bytes;
use futures_core::Stream;
use http_body::Frame;
use http_body_util::StreamBody;
use serde::Deserialize;
use serde_json::json;
use staple_adapters::{InvocationInput, RunStatus};
use staple_data::board_member_skill;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{content::Json, route},
};

use crate::{error::ApiError, state::AppState};

/// A tiny `Stream` adapter over a tokio mpsc receiver of body frames.
struct FrameStream {
    rx: tokio::sync::mpsc::Receiver<Result<Frame<Bytes>, Infallible>>,
}

impl futures_core::Stream for FrameStream {
    type Item = Result<Frame<Bytes>, Infallible>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}

/// One prior chat message.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    /// `user` or `assistant`.
    pub role: String,
    /// Message content.
    pub content: String,
}

/// Body for `POST /api/board/chat/stream`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardChatRequest {
    /// Active company id (everything is company-scoped).
    pub company_id: String,
    /// Latest operator message.
    pub message: String,
    /// Adapter type (defaults to `cli_local`).
    #[serde(default)]
    pub adapter_type: Option<String>,
    /// Optional conversation history.
    #[serde(default)]
    pub history: Vec<ChatMessage>,
}

/// `POST /api/board/chat/stream` — streams a board-concierge reply as
/// server-sent events.
///
/// The endpoint loads the built-in board-member skill as the system prompt,
/// appends the active company + conversation history + the latest operator
/// message, invokes the chosen adapter, and streams status/delta/done/error
/// events as `text/event-stream`.
#[route(POST "/api/board/chat/stream")]
pub async fn board_chat_stream(
    cx: &Cx,
    Json(body): Json<BoardChatRequest>,
) -> Result<topcoat::router::Response, ApiError> {
    crate::auth::require_board(cx)?;
    let company_id = body.company_id.trim().to_owned();
    let message = body.message.trim().to_owned();
    if company_id.is_empty() || message.is_empty() {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([
                { "path": ["companyId"], "message": "companyId is required" },
                { "path": ["message"], "message": "message is required" },
            ]),
        ));
    }
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    if state
        .companies
        .get(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .is_none()
    {
        return Err(ApiError::not_found("Company not found"));
    }

    // Assemble the task: board-member skill system prompt + context.
    let skill = board_member_skill();
    let mut task = String::from(skill.system_prompt);
    task.push_str("\n\n## Active company\n");
    task.push_str(&company_id);
    if !body.history.is_empty() {
        task.push_str("\n\n## Conversation history\n");
        for message in &body.history {
            task.push_str(&format!("{}: {}\n", message.role, message.content));
        }
    }
    task.push_str("\n\n## Latest operator message\n");
    task.push_str(&message);

    let adapter_type = body.adapter_type.unwrap_or_else(|| "cli_local".to_owned());
    let adapter = state
        .adapters
        .get(&adapter_type)
        .ok_or_else(|| ApiError::not_found("Adapter not found"))?;
    let handle = adapter
        .invoke(InvocationInput {
            task,
            cwd: None,
            env: vec![],
        })
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;

    let registry = state.adapters.clone();
    let stream = adapter
        .stream(&handle.run_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Frame<Bytes>, Infallible>>(16);
    tokio::spawn(async move {
        let mut stream = Box::pin(stream);
        let started = std::time::Instant::now();
        loop {
            if started.elapsed().as_secs() > 120 {
                let _ = tx
                    .send(Ok(Frame::data(Bytes::from(
                        "event: error\ndata: {\"type\":\"error\",\"error\":\"timeout\"}\n\n",
                    ))))
                    .await;
                return;
            }
            let chunk = std::future::poll_fn(|cx| stream.as_mut().poll_next(cx)).await;
            let Some(chunk) = chunk else {
                // Stream ended: report the final status.
                let Some(adapter) = registry.get(&adapter_type) else {
                    return;
                };
                let event = match adapter.observe(&handle.run_id).await {
                    Ok(RunStatus::Succeeded { .. }) => "data: {\"type\":\"done\"}\n\n".to_owned(),
                    Ok(RunStatus::Failed { error }) => format!(
                        "event: error\ndata: {}\n\n",
                        serde_json::to_string(&serde_json::json!({ "type": "error", "error": error }))
                            .unwrap_or_else(|_| "{\"type\":\"error\",\"error\":\"unknown\"}".to_owned())
                    ),
                    Ok(RunStatus::Cancelled) => {
                        "event: error\ndata: {\"type\":\"error\",\"error\":\"cancelled\"}\n\n".to_owned()
                    }
                    Ok(RunStatus::Running) => {
                        "event: error\ndata: {\"type\":\"error\",\"error\":\"stream ended while running\"}\n\n"
                            .to_owned()
                    }
                    Err(error) => format!(
                        "event: error\ndata: {}\n\n",
                        serde_json::to_string(&serde_json::json!({ "type": "error", "error": error.to_string() }))
                            .unwrap_or_else(|_| "{\"type\":\"error\",\"error\":\"unknown\"}".to_owned())
                    ),
                };
                let _ = tx.send(Ok(Frame::data(Bytes::from(event)))).await;
                return;
            };
            let payload = match chunk {
                staple_adapters::OutputEvent::Delta(content) => serde_json::json!({
                    "type": "delta",
                    "content": content,
                }),
                staple_adapters::OutputEvent::Stderr(content) => serde_json::json!({
                    "type": "stderr",
                    "name": "stderr",
                    "content": content,
                }),
            };
            let event = format!(
                "data: {}\n\n",
                serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_owned())
            );
            if tx.send(Ok(Frame::data(Bytes::from(event)))).await.is_err() {
                return;
            }
        }
    });

    let body = topcoat::router::Body::new(StreamBody::new(FrameStream { rx }));
    topcoat::router::Response::builder()
        .header("Content-Type", "text/event-stream; charset=utf-8")
        .header("Cache-Control", "no-cache")
        .header("X-Accel-Buffering", "no")
        .body(body)
        .map_err(|error| ApiError::internal(error.to_string()))
}
