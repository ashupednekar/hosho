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
