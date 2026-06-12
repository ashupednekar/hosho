use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TelemetryAttribute {
    pub key: String,
    pub value: TelemetryValue,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TelemetryValue {
    Bool(bool),
    Int(i64),
    Double(f64),
    String(String),
    StringArray(Vec<String>),
}

impl TelemetryAttribute {
    pub fn new(key: impl Into<String>, value: TelemetryValue) -> Self {
        Self {
            key: key.into(),
            value,
        }
    }
}

impl From<(&str, &str)> for TelemetryAttribute {
    fn from((key, value): (&str, &str)) -> Self {
        Self::new(key, TelemetryValue::String(value.to_string()))
    }
}
