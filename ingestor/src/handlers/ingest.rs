use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use standard_error::StandardError;

use crate::{
    internal::{
        specs::capture::{ConsoleRecord, HarRecord},
        telemetry::Export,
    },
    prelude::Result,
};

#[worker::send]
pub async fn ingest_har(Json(payload): Json<Value>) -> Response {
    match accept_har(payload).await {
        Ok(body) => Json(body).into_response(),
        Err(error) => error_response(error),
    }
}

#[worker::send]
pub async fn ingest_console(Json(payload): Json<Value>) -> Response {
    match accept_console(payload).await {
        Ok(body) => Json(body).into_response(),
        Err(error) => error_response(error),
    }
}

async fn accept_har(payload: Value) -> Result<Value> {
    let record = HarRecord::try_from(payload)?;
    record.export().await?;
    Ok(json!({"accepted": 1}))
}

async fn accept_console(payload: Value) -> Result<Value> {
    let record = ConsoleRecord::try_from(payload)?;
    record.export().await?;
    Ok(json!({"accepted": 1}))
}

fn error_response(error: StandardError) -> Response {
    (
        error.status_code,
        Json(json!({
            "code": error.err_code,
            "detail": error.message,
        })),
    )
        .into_response()
}
