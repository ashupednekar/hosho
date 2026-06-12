use serde::{Deserialize, Serialize};

use super::telemetry::TelemetryAttribute;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricPoint {
    pub name: &'static str,
    pub description: &'static str,
    pub unit: &'static str,
    pub kind: MetricKind,
    pub value: MetricValue,
    pub attributes: Vec<TelemetryAttribute>,
    pub time_unix_nano: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MetricKind {
    Counter,
    Gauge,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MetricValue {
    Int(i64),
    Double(f64),
}
