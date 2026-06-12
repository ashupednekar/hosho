use serde_json::Value;

use super::config::OtlpSignal;

pub struct TelemetryBatch {
    pub traces: Option<Value>,
    pub logs: Option<Value>,
    pub metrics: Option<Value>,
}

pub struct ExportRequest {
    pub signal: OtlpSignal,
    pub body: Value,
}
