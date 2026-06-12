mod attributes;
mod config;
mod exporter;
mod ids;
mod logs;
mod metrics;
mod payload;
mod resource;
mod strategy;
mod traces;

use crate::internal::{
    error::IngestResult,
    metrics::{console::metrics_for_console, har::metrics_for_har},
    specs::{console::ConsoleRecord, network::NetworkRequest},
};

pub async fn export_har(records: &[NetworkRequest]) -> IngestResult<()> {
    exporter::export(payload::TelemetryBatch {
        traces: traces::har_traces(records),
        logs: None,
        metrics: metrics::metric_payload(&metrics_for_har(records)),
    })
    .await
}

pub async fn export_console(records: &[ConsoleRecord]) -> IngestResult<()> {
    exporter::export(payload::TelemetryBatch {
        traces: None,
        logs: logs::console_logs(records),
        metrics: metrics::metric_payload(&metrics_for_console(records)),
    })
    .await
}
