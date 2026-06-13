mod attributes;
mod config;
mod exporter;
mod ids;
mod logs;
mod metrics;
mod resource;
mod strategy;
mod traces;

use serde_json::Value;
use crate::{
    internal::{
        metrics::{console::metrics_for_console, har::metrics_for_har},
        specs::{capture::ConsoleRecord, network::NetworkRequest},
    },
    prelude::Result,
};


use super::OtlpSignal;

pub struct TelemetryBatch {
    pub traces: Option<Value>,
    pub logs: Option<Value>,
    pub metrics: Option<Value>,
}

pub struct ExportRequest {
    pub signal: OtlpSignal,
    pub body: Value,
}

pub async fn export_har(records: &[NetworkRequest]) -> Result<()> {
    exporter::export(TelemetryBatch {
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
