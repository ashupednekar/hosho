pub const OTEL_COLLECTOR_ENDPOINT: &str = "http://localhost:4318";
pub const SERVICE_NAME: &str = "hosho-browser";
pub const SERVICE_NAMESPACE: &str = "hosho";
pub const SCOPE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Copy)]
pub enum OtlpSignal {
    Traces,
    Logs,
    Metrics,
}

impl OtlpSignal {
    pub fn path(self) -> &'static str {
        match self {
            OtlpSignal::Traces => "/v1/traces",
            OtlpSignal::Logs => "/v1/logs",
            OtlpSignal::Metrics => "/v1/metrics",
        }
    }
}
