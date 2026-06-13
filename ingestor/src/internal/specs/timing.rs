use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestTiming {
    pub started_at: Option<String>,
    pub duration_ms: Option<f64>,
    pub blocked_ms: Option<f64>,
    pub dns_ms: Option<f64>,
    pub connect_ms: Option<f64>,
    pub ssl_ms: Option<f64>,
    pub send_ms: Option<f64>,
    pub wait_ms: Option<f64>,
    pub receive_ms: Option<f64>,
}

impl RequestTiming {
    pub fn start_unix_nano(&self) -> Option<u64> {
        unix_nano(self.started_at.as_deref())
    }

    pub fn end_unix_nano(&self) -> Option<u64> {
        let duration_ns = (self.duration_ms? * 1_000_000.0).round();
        self.start_unix_nano()?
            .checked_add(duration_ns.max(0.0) as u64)
    }
}

pub fn unix_nano(timestamp: Option<&str>) -> Option<u64> {
    let parsed = DateTime::<FixedOffset>::parse_from_rfc3339(timestamp?).ok()?;
    parsed
        .timestamp_nanos_opt()
        .and_then(|ns| u64::try_from(ns).ok())
}
