use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::{json, Value};

use crate::{
    internal::{
        specs::{capture::ConsoleRecord, network::NetworkRequest},
        telemetry,
    },
    prelude::Result,
};

#[worker::send]
pub async fn ingest_har(Json(payload): Json<Value>) -> Result<Json<Value>> {
    let records = vec![NetworkRequest::from(
        crate::internal::specs::capture::HarRecord::try_from(payload)?,
    )];
    let count = records.len();
    telemetry::export_har(&records).await?;
    Ok(Json(json!({
        "accepted": count
    })))
}

#[worker::send]
pub async fn ingest_console(Json(payload): Json<Value>) -> Result<Json<Value>> {
    let records = vec![ConsoleRecord::try_from(payload)?];
    let count = records.len();
    telemetry::export_console(&records).await?;
    Ok(Json(json!({
        "accepted": count
    })))
}
