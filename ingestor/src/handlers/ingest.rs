use axum::Json;
use worker::console_log;

use crate::internal::exporter::logs::{send_log, ConsoleLogPayload};
use crate::internal::exporter::metrics::{send_console_metrics, send_har_metrics};
use crate::internal::exporter::spec::har::HarPayload;
use crate::internal::exporter::traces::send_trace;

#[worker::send]
pub async fn ingest_har(har: Json<HarPayload>) -> String {
    if let Err(error) = send_trace(&har.0).await {
        console_log!("otel trace export failed: {:?}", error);
    }

    if let Err(error) = send_har_metrics(&har.0).await {
        console_log!("otel har metric export failed: {:?}", error);
    }

    "".into()
}

#[worker::send]
pub async fn ingest_logs(log: Json<ConsoleLogPayload>) -> String {
    if let Err(error) = send_log(&log.0).await {
        console_log!("otel log export failed: {:?}", error);
    }

    if let Err(error) = send_console_metrics(&log.0).await {
        console_log!("otel console metric export failed: {:?}", error);
    }

    "".into()
}
