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

use crate::{
    internal::{
        metrics::{console::metrics_for_console, har::metrics_for_har},
        specs::{capture::ConsoleRecord, network::NetworkRequest},
    },
    prelude::Result,
};

pub async fn export_har(records: &[NetworkRequest]) -> Result<()> {
    exporter::export(payload::TelemetryBatch {
        traces: traces::har_traces(records),
        logs: None,
        metrics: metrics::metric_payload(&metrics_for_har(records)),
    })
    .await
}

pub async fn export_console(records: &[ConsoleRecord]) -> Result<()> {
    exporter::export(payload::TelemetryBatch {
        traces: None,
        logs: logs::console_logs(records),
        metrics: metrics::metric_payload(&metrics_for_console(records)),
    })
    .await
}
