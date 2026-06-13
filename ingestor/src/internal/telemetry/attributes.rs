use opentelemetry::{Array, KeyValue, Value as OtelValue};
use serde_json::{json, Value};

pub fn otlp_attributes(attrs: &[KeyValue]) -> Vec<Value> {
    attrs
        .iter()
        .map(|attr| json!({"key": attr.key.to_string(), "value": otlp_value(&attr.value)}))
        .collect()
}

pub fn string_attr(key: &str, value: &Option<String>) -> Option<KeyValue> {
    value
        .as_ref()
        .map(|value| KeyValue::new(key.to_string(), value.clone()))
}

pub fn int_attr(key: &str, value: Option<i64>) -> Option<KeyValue> {
    value.map(|value| KeyValue::new(key.to_string(), value))
}

pub fn double_attr(key: &str, value: Option<f64>) -> Option<KeyValue> {
    value.map(|value| KeyValue::new(key.to_string(), value))
}

pub fn string_array_attr(key: String, values: Vec<String>) -> KeyValue {
    let values = values.into_iter().map(Into::into).collect::<Vec<_>>();
    KeyValue::new(key, OtelValue::Array(Array::String(values)))
}

fn otlp_value(value: &OtelValue) -> Value {
    match value {
        OtelValue::Bool(value) => json!({"boolValue": value}),
        OtelValue::I64(value) => json!({"intValue": value.to_string()}),
        OtelValue::F64(value) => json!({"doubleValue": value}),
        OtelValue::String(value) => json!({"stringValue": value.as_str()}),
        OtelValue::Array(Array::String(values)) => json!({
            "arrayValue": {"values": values.iter().map(|value| json!({"stringValue": value.as_str()})).collect::<Vec<_>>()}
        }),
        OtelValue::Array(value) => json!({"stringValue": value.to_string()}),
        _ => json!({"stringValue": value.to_string()}),
    }
}
