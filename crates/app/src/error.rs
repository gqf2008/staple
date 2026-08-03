//! Unified JSON error responses.
//!
//! Every application error renders as `{"error": "<message>"}`, with an
//! optional `details` object for validation failures — matching the reference
//! server's `{ error: ... }` JSON error shape.

use serde_json::{Value, json};
use topcoat::{
    Result,
    context::Cx,
    router::{Body, IntoResponse, Response, StatusCode},
};

/// An API error with an HTTP status code and a JSON body.
#[derive(Debug, Clone)]
pub struct ApiError {
    /// HTTP status code.
    pub(crate) status: StatusCode,
    message: String,
    details: Option<Value>,
}

impl ApiError {
    /// Creates an error with the given status and message.
    #[must_use]
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
            details: None,
        }
    }

    /// HTTP 400.
    #[must_use]
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    /// HTTP 401.
    #[must_use]
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, message)
    }

    /// HTTP 403.
    #[must_use]
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, message)
    }

    /// HTTP 404.
    #[must_use]
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    /// HTTP 409.
    #[must_use]
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, message)
    }

    /// HTTP 422, with validation details.
    #[must_use]
    pub fn unprocessable(message: impl Into<String>, details: Value) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message: message.into(),
            details: Some(details),
        }
    }

    /// HTTP 500. The message never leaks the underlying error to clients.
    #[must_use]
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }

    /// Renders the error as `{"error": "...", "details": ...}`.
    pub fn into_json_response(&self) -> Response {
        let mut body = json!({ "error": self.message.clone() });
        if let Some(details) = &self.details {
            body["details"] = details.clone();
        }
        let bytes = serde_json::to_vec(&body).expect("JSON serialization cannot fail");
        Response::builder()
            .status(self.status)
            .header("Content-Type", "application/json")
            .body(Body::from(bytes))
            .expect("valid response construction")
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ApiError {}

impl From<topcoat::router::error::BadRequestError> for ApiError {
    fn from(_: topcoat::router::error::BadRequestError) -> Self {
        Self::bad_request("Invalid company id")
    }
}

impl IntoResponse for ApiError {
    fn into_response(self, _cx: &Cx) -> Result<Response> {
        Ok(self.into_json_response())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn body_of(response: Response) -> Value {
        let (_, body) = response.into_parts();
        let bytes = topcoat::router::to_bytes(body, usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn renders_error_message() {
        let response = ApiError::not_found("company not found").into_json_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            "application/json"
        );
        assert_eq!(
            body_of(response).await,
            json!({ "error": "company not found" })
        );
    }

    #[tokio::test]
    async fn renders_validation_details() {
        let response = ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["name"], "message": "required" }]),
        )
        .into_json_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body_of(response).await,
            json!({
                "error": "Validation error",
                "details": [{ "path": ["name"], "message": "required" }]
            })
        );
    }

    #[tokio::test]
    async fn internal_error_status() {
        let response = ApiError::internal("boom").into_json_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body_of(response).await, json!({ "error": "boom" }));
    }
}
