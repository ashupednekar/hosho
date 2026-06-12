use chrono::{DateTime, FixedOffset};
use serde_json::Value;

use crate::internal::specs::timing::RequestTiming;

pub fn timing_from_entry(entry: &Value) -> RequestTiming {
    let timings = entry.get("timings").unwrap_or(&Value::Null);

    RequestTiming {
        started_at: entry
            .get("startedDateTime")
            .and_then(Value::as_str)
            .map(str::to_string),
        duration_ms: non_negative_f64(entry.get("time")),
        blocked_ms: non_negative_f64(timings.get("blocked")),
        dns_ms: non_negative_f64(timings.get("dns")),
        connect_ms: non_negative_f64(timings.get("connect")),
        ssl_ms: non_negative_f64(timings.get("ssl")),
        send_ms: non_negative_f64(timings.get("send")),
        wait_ms: non_negative_f64(timings.get("wait")),
        receive_ms: non_negative_f64(timings.get("receive")),
    }
}

pub fn unix_nano(timestamp: Option<&str>) -> Option<u64> {
    let parsed = DateTime::<FixedOffset>::parse_from_rfc3339(timestamp?).ok()?;
    parsed
        .timestamp_nanos_opt()
        .and_then(|ns| u64::try_from(ns).ok())
}

pub fn end_unix_nano(start: Option<u64>, duration_ms: Option<f64>) -> Option<u64> {
    let duration_ns = (duration_ms? * 1_000_000.0).round();
    start?.checked_add(duration_ns.max(0.0) as u64)
}

fn non_negative_f64(value: Option<&Value>) -> Option<f64> {
    value
        .and_then(Value::as_f64)
        .filter(|number| *number >= 0.0)
}
