use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

pub type IngestResult<T> = Result<T, IngestError>;

#[derive(Debug)]
pub enum IngestError {
    InvalidPayload(&'static str),
    Export(String),
}

impl IntoResponse for IngestError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            IngestError::InvalidPayload(message) => (StatusCode::BAD_REQUEST, message.to_string()),
            IngestError::Export(message) => (StatusCode::BAD_GATEWAY, message),
        };

        (status, Json(json!({"status": "error", "message": message}))).into_response()
    }
}
