use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};

use crate::internal::{
    normalize::{console::normalize_console_payload, har::normalize_har_payload},
    telemetry,
};

#[worker::send]
pub async fn ingest_har(Json(payload): Json<Value>) -> Response {
    match accept_har(payload).await {
        Ok(count) => accepted_response("har", count),
        Err(error) => error.into_response(),
    }
}

#[worker::send]
pub async fn ingest_console(Json(payload): Json<Value>) -> Response {
    match accept_console(payload).await {
        Ok(count) => accepted_response("console", count),
        Err(error) => error.into_response(),
    }
}

async fn accept_har(payload: Value) -> crate::internal::error::IngestResult<usize> {
    let records = normalize_har_payload(payload)?;
    let count = records.len();
    telemetry::export_har(&records).await?;
    Ok(count)
}

async fn accept_console(payload: Value) -> crate::internal::error::IngestResult<usize> {
    let records = normalize_console_payload(payload)?;
    let count = records.len();
    telemetry::export_console(&records).await?;
    Ok(count)
}

fn accepted_response(kind: &str, accepted: usize) -> Response {
    (
        StatusCode::ACCEPTED,
        Json(json!({"status": "accepted", "kind": kind, "accepted": accepted})),
    )
        .into_response()
}
