use opentelemetry::{Array, KeyValue, Value as OtelValue};
use serde_json::{json, Value};

pub trait ToOtlpJson {
    fn to_otlp_json(&self) -> Value;
}

pub fn otlp_attributes(attrs: &[KeyValue]) -> Vec<Value> {
    attrs.iter().map(ToOtlpJson::to_otlp_json).collect()
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

impl ToOtlpJson for KeyValue {
    fn to_otlp_json(&self) -> Value {
        json!({"key": self.key.to_string(), "value": self.value.to_otlp_json()})
    }
}

impl ToOtlpJson for OtelValue {
    fn to_otlp_json(&self) -> Value {
        match self {
            OtelValue::Array(values) => values.to_otlp_json(),
            value => scalar_value(value),
        }
    }
}

impl ToOtlpJson for Array {
    fn to_otlp_json(&self) -> Value {
        let values = match self {
            Array::Bool(values) => values
                .iter()
                .map(|value| json!({"boolValue": value}))
                .collect::<Vec<_>>(),
            Array::I64(values) => values
                .iter()
                .map(|value| json!({"intValue": value.to_string()}))
                .collect::<Vec<_>>(),
            Array::F64(values) => values
                .iter()
                .map(|value| json!({"doubleValue": value}))
                .collect::<Vec<_>>(),
            Array::String(values) => values
                .iter()
                .map(|value| json!({"stringValue": value.as_str()}))
                .collect::<Vec<_>>(),
            _ => vec![json!({"stringValue": self.to_string()})],
        };

        json!({"arrayValue": {"values": values}})
    }
}

fn scalar_value(value: &OtelValue) -> Value {
    match value {
        OtelValue::Bool(value) => json!({"boolValue": value}),
        OtelValue::I64(value) => json!({"intValue": value.to_string()}),
        OtelValue::F64(value) => json!({"doubleValue": value}),
        OtelValue::String(value) => json!({"stringValue": value.as_str()}),
        _ => json!({"stringValue": value.to_string()}),
    }
}
