use crate::internal::{
    normalize::time::unix_nano,
    specs::{
        metrics::{MetricKind, MetricPoint, MetricValue},
        network::NetworkRequest,
        telemetry::{TelemetryAttribute, TelemetryValue},
    },
};

pub fn metrics_for_har(records: &[NetworkRequest]) -> Vec<MetricPoint> {
    records.iter().flat_map(metrics_for_record).collect()
}

fn metrics_for_record(record: &NetworkRequest) -> Vec<MetricPoint> {
    vec![request_count(record), request_duration(record)]
}

fn request_count(record: &NetworkRequest) -> MetricPoint {
    MetricPoint {
        name: "hosho.har.requests",
        description: "Browser HAR requests captured by Hosho",
        unit: "1",
        kind: MetricKind::Counter,
        value: MetricValue::Int(1),
        attributes: request_attrs(record),
        time_unix_nano: unix_nano(record.timing.started_at.as_deref()),
    }
}

fn request_duration(record: &NetworkRequest) -> MetricPoint {
    MetricPoint {
        name: "hosho.har.request.duration",
        description: "Browser HAR request duration",
        unit: "ms",
        kind: MetricKind::Gauge,
        value: MetricValue::Double(record.timing.duration_ms.unwrap_or_default()),
        attributes: request_attrs(record),
        time_unix_nano: unix_nano(record.timing.started_at.as_deref()),
    }
}

fn request_attrs(record: &NetworkRequest) -> Vec<TelemetryAttribute> {
    vec![
        TelemetryAttribute::new(
            "http.request.method",
            TelemetryValue::String(record.request.method.clone()),
        ),
        TelemetryAttribute::new(
            "http.response.status_code",
            TelemetryValue::Int(record.response.status.unwrap_or_default()),
        ),
    ]
}
