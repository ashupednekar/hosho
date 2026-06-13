use axum::Json;
use serde_json::{json, Value};

use crate::{
    internal::{
        specs::{capture::ConsoleRecord, network::NetworkRequest},
        telemetry,
    },
    prelude::HttpResult,
};

#[worker::send]
pub async fn ingest_har(Json(payload): Json<Value>) -> HttpResult<Json<Value>> {
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
pub async fn ingest_console(Json(payload): Json<Value>) -> HttpResult<Json<Value>> {
    let records = vec![ConsoleRecord::try_from(payload)?];
    let count = records.len();
    telemetry::export_console(&records).await?;
    Ok(Json(json!({
        "accepted": count
    })))
}
