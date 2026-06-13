use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;

pub type Result<T> = core::result::Result<T, IngestError>;
pub type HttpResult<T> = Result<T>;

#[derive(Debug, Clone)]
pub struct IngestError {
    pub code: String,
    pub status: StatusCode,
    pub message: String,
}

impl IngestError {
    pub fn new(code: impl Into<String>, status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            status,
            message: message.into(),
        }
    }

    pub fn bad_request(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(code, StatusCode::BAD_REQUEST, message)
    }

    pub fn bad_gateway(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(code, StatusCode::BAD_GATEWAY, message)
    }
}

impl fmt::Display for IngestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} with status {}: {}",
            self.code, self.status, self.message
        )
    }
}

impl std::error::Error for IngestError {}

impl IntoResponse for IngestError {
    fn into_response(self) -> Response {
        let status = self.status;
        let body = Json(json!({
            "error": {
                "code": self.code,
                "message": self.message,
            }
        }));

        (status, body).into_response()
    }
}
