use crate::internal::{
    normalize::time::unix_nano,
    specs::{
        console::ConsoleRecord,
        metrics::{MetricKind, MetricPoint, MetricValue},
        telemetry::{TelemetryAttribute, TelemetryValue},
    },
};

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
        attributes: vec![TelemetryAttribute::new(
            "log.severity",
            TelemetryValue::String(record.level.clone()),
        )],
        time_unix_nano: unix_nano(record.captured_at.as_deref()),
    }
}
