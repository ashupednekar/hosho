use opentelemetry::{KeyValue, Value as OtelValue};
use opentelemetry_sdk::Resource;

use crate::internal::specs::telemetry::{TelemetryAttribute, TelemetryValue};

use super::config::{SERVICE_NAME, SERVICE_NAMESPACE};

pub fn resource_attributes(instance_id: Option<&str>) -> Vec<TelemetryAttribute> {
    let mut attrs = vec![
        KeyValue::new("service.name", SERVICE_NAME),
        KeyValue::new("service.namespace", SERVICE_NAMESPACE),
        KeyValue::new("telemetry.sdk.name", "hosho"),
        KeyValue::new("telemetry.sdk.language", "webjs"),
    ];

    if let Some(instance_id) = instance_id {
        attrs.push(KeyValue::new(
            "service.instance.id",
            instance_id.to_string(),
        ));
    }

    Resource::builder_empty()
        .with_attributes(attrs)
        .build()
        .iter()
        .map(|(key, value)| TelemetryAttribute::new(key.to_string(), value_from_otel(value)))
        .collect()
}

fn value_from_otel(value: &OtelValue) -> TelemetryValue {
    match value {
        OtelValue::Bool(value) => TelemetryValue::Bool(*value),
        OtelValue::I64(value) => TelemetryValue::Int(*value),
        OtelValue::F64(value) => TelemetryValue::Double(*value),
        OtelValue::String(value) => TelemetryValue::String(value.as_str().to_string()),
        OtelValue::Array(value) => TelemetryValue::String(format!("{value:?}")),
        _ => TelemetryValue::String(format!("{value:?}")),
    }
}
