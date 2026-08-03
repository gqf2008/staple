//! Structured request logging and JSON error normalization.
//!
//! The `request_log` layer wraps every request. It records method, path,
//! status, and duration with `tracing`, and rewrites non-JSON error
//! responses (framework 404/405/500, extractor failures) into the unified
//! `{"error": "..."}` JSON shape.

use std::time::Instant;

use serde_json::json;
use topcoat::{
    Result,
    context::CxBuilder,
    router::{
        Body, Next, Response, StatusCode,
        error::{
            BadRequestError, ForbiddenError, MethodNotAllowedError, NotFoundError, RedirectError,
            UnauthorizedError,
        },
        header, layer, to_bytes,
    },
};

/// Wraps every request: structured logging plus JSON error normalization.
#[layer("/")]
pub async fn request_log(cx: &mut CxBuilder, body: Body, next: Next<'_>) -> Result<Response> {
    let parts = cx.get::<http::request::Parts>();
    let method = parts
        .map(|parts| parts.method.as_str().to_owned())
        .unwrap_or_else(|| "-".to_owned());
    let path = parts
        .map(|parts| parts.uri.path().to_owned())
        .unwrap_or_else(|| "-".to_owned());
    let start = Instant::now();

    let response = match next.run(cx, body).await {
        Ok(response) => response,
        Err(error) => {
            // Redirects are not errors; let the framework render them.
            if error.downcast_ref::<RedirectError>().is_some() {
                return Err(error);
            }
            let status = error_status(&error);
            log(status, &method, &path, start);
            return Ok(json_response(status, &error_message(status, &error)));
        }
    };

    let status = response.status();
    log(status, &method, &path, start);

    if is_plain_error_response(&response) {
        let message = response_message(response).await?;
        return Ok(json_response(status, &message));
    }

    Ok(response)
}

/// Logs a completed request at a level derived from its status.
fn log(status: StatusCode, method: &str, path: &str, start: Instant) {
    let duration_ms = start.elapsed().as_millis();
    match status.as_u16() {
        500.. => tracing::error!(
            method,
            path,
            status = status.as_u16(),
            duration_ms,
            "request failed"
        ),
        400..=499 => tracing::warn!(
            method,
            path,
            status = status.as_u16(),
            duration_ms,
            "request error"
        ),
        _ => tracing::info!(
            method,
            path,
            status = status.as_u16(),
            duration_ms,
            "request"
        ),
    }
}

/// Maps a framework error to its HTTP status code.
fn error_status(error: &topcoat::Error) -> StatusCode {
    if error.downcast_ref::<BadRequestError>().is_some() {
        StatusCode::BAD_REQUEST
    } else if error.downcast_ref::<UnauthorizedError>().is_some() {
        StatusCode::UNAUTHORIZED
    } else if error.downcast_ref::<ForbiddenError>().is_some() {
        StatusCode::FORBIDDEN
    } else if error.downcast_ref::<NotFoundError>().is_some() {
        StatusCode::NOT_FOUND
    } else if error.downcast_ref::<MethodNotAllowedError>().is_some() {
        StatusCode::METHOD_NOT_ALLOWED
    } else {
        // Unknown framework errors render as 500, matching the router's own
        // fallback.
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

/// Derives a client-safe message for an error: the framework text for 4xx,
/// a generic message for 5xx so internals never leak.
fn error_message(status: StatusCode, error: &topcoat::Error) -> String {
    if status.is_server_error() {
        "internal server error".to_owned()
    } else {
        error.to_string()
    }
}

/// True when the response is an error but not already JSON.
fn is_plain_error_response(response: &Response) -> bool {
    if !(response.status().is_client_error() || response.status().is_server_error()) {
        return false;
    }
    !response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json"))
}

/// Reads the plain-text body of an error response as the JSON message.
async fn response_message(response: Response) -> Result<String> {
    let (_, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX)
        .await
        .map_err(|error| topcoat::Error::from(std::io::Error::other(error)))?;
    let text = String::from_utf8_lossy(&bytes);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        Ok("error".to_owned())
    } else {
        Ok(trimmed.to_owned())
    }
}

/// Builds a JSON error response.
fn json_response(status: StatusCode, message: &str) -> Response {
    let bytes =
        serde_json::to_vec(&json!({ "error": message })).expect("JSON serialization cannot fail");
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Body::from(bytes))
        .expect("valid response construction")
}
