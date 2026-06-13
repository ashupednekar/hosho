use opentelemetry::KeyValue;

#[derive(Clone, Debug)]
pub struct MetricPoint {
    pub name: &'static str,
    pub description: &'static str,
    pub unit: &'static str,
    pub kind: MetricKind,
    pub value: MetricValue,
    pub attributes: Vec<KeyValue>,
    pub time_unix_nano: Option<u64>,
}

#[derive(Clone, Debug)]
pub enum MetricKind {
    Counter,
    Gauge,
}

#[derive(Clone, Debug)]
pub enum MetricValue {
    Int(i64),
    Double(f64),
}
