use serde_json::{json, Value};

use crate::internal::specs::telemetry::{TelemetryAttribute, TelemetryValue};

pub fn otlp_attributes(attrs: &[TelemetryAttribute]) -> Vec<Value> {
    attrs
        .iter()
        .map(|attr| json!({"key": attr.key, "value": otlp_value(&attr.value)}))
        .collect()
}

pub fn string_attr(key: &str, value: &Option<String>) -> Option<TelemetryAttribute> {
    value
        .as_ref()
        .map(|value| TelemetryAttribute::new(key, TelemetryValue::String(value.clone())))
}

pub fn int_attr(key: &str, value: Option<i64>) -> Option<TelemetryAttribute> {
    value.map(|value| TelemetryAttribute::new(key, TelemetryValue::Int(value)))
}

pub fn double_attr(key: &str, value: Option<f64>) -> Option<TelemetryAttribute> {
    value.map(|value| TelemetryAttribute::new(key, TelemetryValue::Double(value)))
}

fn otlp_value(value: &TelemetryValue) -> Value {
    match value {
        TelemetryValue::Bool(value) => json!({"boolValue": value}),
        TelemetryValue::Int(value) => json!({"intValue": value.to_string()}),
        TelemetryValue::Double(value) => json!({"doubleValue": value}),
        TelemetryValue::String(value) => json!({"stringValue": value}),
        TelemetryValue::StringArray(values) => {
            json!({"arrayValue": {"values": values.iter().map(|value| json!({"stringValue": value})).collect::<Vec<_>>()}})
        }
    }
}
