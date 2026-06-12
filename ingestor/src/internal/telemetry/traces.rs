use serde_json::{json, Value};

use crate::internal::{
    normalize::{
        headers::allowed_header_attributes,
        time::{end_unix_nano, unix_nano},
    },
    specs::{
        network::NetworkRequest,
        telemetry::{TelemetryAttribute, TelemetryValue},
    },
};

use super::{
    attributes::{double_attr, int_attr, otlp_attributes, string_attr},
    config::SCOPE_VERSION,
    ids::ids_for_request,
    resource::resource_attributes,
};

pub fn har_traces(records: &[NetworkRequest]) -> Option<Value> {
    (!records.is_empty()).then(|| {
        json!({"resourceSpans": [{
            "resource": {"attributes": otlp_attributes(&resource_attributes(records.first().and_then(|r| r.capture.session_id.as_deref())))},
            "scopeSpans": [{"scope": {"name": "hosho.har", "version": SCOPE_VERSION}, "spans": spans(records)}]
        }]})
    })
}

fn spans(records: &[NetworkRequest]) -> Vec<Value> {
    records.iter().map(span_for_request).collect()
}

fn span_for_request(record: &NetworkRequest) -> Value {
    let ids = ids_for_request(record);
    let start = unix_nano(record.timing.started_at.as_deref());
    let end = end_unix_nano(start, record.timing.duration_ms);

    json!({
        "traceId": ids.trace_id,
        "spanId": ids.span_id,
        "name": record.request.method,
        "kind": 3,
        "startTimeUnixNano": start.unwrap_or_default().to_string(),
        "endTimeUnixNano": end.unwrap_or_else(|| start.unwrap_or_default()).to_string(),
        "attributes": otlp_attributes(&span_attrs(record)),
        "status": span_status(record),
    })
}

fn span_attrs(record: &NetworkRequest) -> Vec<TelemetryAttribute> {
    let mut attrs = base_attrs(record);
    attrs.extend(timing_attrs(record));
    attrs.extend(header_attrs(record));
    attrs
}

fn base_attrs(record: &NetworkRequest) -> Vec<TelemetryAttribute> {
    [
        Some(TelemetryAttribute::new(
            "hosho.schema",
            TelemetryValue::String(record.schema.clone()),
        )),
        Some(TelemetryAttribute::new(
            "http.request.method",
            TelemetryValue::String(record.request.method.clone()),
        )),
        Some(TelemetryAttribute::new(
            "url.full",
            TelemetryValue::String(record.request.url_sanitized.clone()),
        )),
        string_attr("url.scheme", &record.request.scheme),
        string_attr("server.address", &record.request.host),
        record.request.port.map(|port| {
            TelemetryAttribute::new("server.port", TelemetryValue::Int(i64::from(port)))
        }),
        int_attr("http.response.status_code", record.response.status),
        string_attr("hosho.capture.session_id", &record.capture.session_id),
        string_attr("hosho.capture.page_id", &record.capture.page_id),
        Some(TelemetryAttribute::new(
            "hosho.har.entry_hash",
            TelemetryValue::String(record.identity.entry_hash.clone()),
        )),
        Some(TelemetryAttribute::new(
            "hosho.har.dedupe_key",
            TelemetryValue::String(record.identity.dedupe_key.clone()),
        )),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn timing_attrs(record: &NetworkRequest) -> Vec<TelemetryAttribute> {
    [
        double_attr("hosho.har.timing.blocked_ms", record.timing.blocked_ms),
        double_attr("hosho.har.timing.dns_ms", record.timing.dns_ms),
        double_attr("hosho.har.timing.connect_ms", record.timing.connect_ms),
        double_attr("hosho.har.timing.ssl_ms", record.timing.ssl_ms),
        double_attr("hosho.har.timing.send_ms", record.timing.send_ms),
        double_attr("hosho.har.timing.wait_ms", record.timing.wait_ms),
        double_attr("hosho.har.timing.receive_ms", record.timing.receive_ms),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn header_attrs(record: &NetworkRequest) -> Vec<TelemetryAttribute> {
    allowed_header_attributes("http.request.header", &record.request.headers)
        .into_iter()
        .chain(allowed_header_attributes(
            "http.response.header",
            &record.response.headers,
        ))
        .map(|(key, values)| TelemetryAttribute::new(key, TelemetryValue::StringArray(values)))
        .collect()
}

fn span_status(record: &NetworkRequest) -> Value {
    match record.response.status {
        Some(100..=399) => json!({}),
        Some(status @ 400..=599) => json!({"code": 2, "message": status.to_string()}),
        _ => json!({"code": 2, "message": "network_error"}),
    }
}
