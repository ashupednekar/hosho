use serde_json::{json, Value};

use crate::internal::specs::metrics::{MetricKind, MetricPoint, MetricValue};

use super::{attributes::otlp_attributes, config::SCOPE_VERSION, resource::resource_attributes};

pub fn metric_payload(points: &[MetricPoint]) -> Option<Value> {
    (!points.is_empty()).then(|| {
        json!({"resourceMetrics": [{
            "resource": {"attributes": otlp_attributes(&resource_attributes(None))},
            "scopeMetrics": [{"scope": {"name": "hosho.metrics", "version": SCOPE_VERSION}, "metrics": metrics(points)}]
        }]})
    })
}

fn metrics(points: &[MetricPoint]) -> Vec<Value> {
    points.iter().map(metric).collect()
}

fn metric(point: &MetricPoint) -> Value {
    match point.kind {
        MetricKind::Counter => json!({
            "name": point.name,
            "description": point.description,
            "unit": point.unit,
            "sum": {"aggregationTemporality": 2, "isMonotonic": true, "dataPoints": [data_point(point)]}
        }),
        MetricKind::Gauge => json!({
            "name": point.name,
            "description": point.description,
            "unit": point.unit,
            "gauge": {"dataPoints": [data_point(point)]}
        }),
    }
}

fn data_point(point: &MetricPoint) -> Value {
    let mut data = json!({
        "attributes": otlp_attributes(&point.attributes),
        "timeUnixNano": point.time_unix_nano.unwrap_or_default().to_string(),
    });

    if let Some(object) = data.as_object_mut() {
        match point.value {
            MetricValue::Int(value) => object.insert("asInt".to_string(), json!(value.to_string())),
            MetricValue::Double(value) => object.insert("asDouble".to_string(), json!(value)),
        };
    }

    data
}
