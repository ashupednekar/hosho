use crate::internal::specs::{
    capture::ConsoleRecord,
    metrics::{MetricKind, MetricPoint, MetricValue},
    timing::unix_nano,
};
use opentelemetry::KeyValue;

pub fn metrics_for_console(records: &[ConsoleRecord]) -> Vec<MetricPoint> {
    records.iter().map(log_count).collect()
}

fn log_count(record: &ConsoleRecord) -> MetricPoint {
    MetricPoint {
        name: "hosho.console.logs",
        description: "Browser console logs captured by Hosho",
        unit: "1",
        kind: MetricKind::Counter,
        value: MetricValue::Int(1),
        attributes: vec![KeyValue::new("log.severity", record.level.clone())],
        time_unix_nano: unix_nano(record.captured_at.as_deref()),
    }
}
