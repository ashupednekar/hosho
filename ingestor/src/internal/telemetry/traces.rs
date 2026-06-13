use serde_json::{json, Value};

use crate::internal::specs::network::{allowed_header_attributes, NetworkRequest};
use opentelemetry::KeyValue;

use super::{
    attributes::{double_attr, int_attr, otlp_attributes, string_array_attr, string_attr},
    config::SCOPE_VERSION,
    ids::ids_for_request,
    resource::resource_attributes,
};

pub fn har_traces(records: &[NetworkRequest]) -> Option<Value> {
    (!records.is_empty()).then(|| {
        json!({"resourceSpans": [{
            "resource": {"attributes": otlp_attributes(&resource_attributes(None))},
            "scopeSpans": [{"scope": {"name": "hosho.har", "version": SCOPE_VERSION}, "spans": spans(records)}]
        }]})
    })
}

fn spans(records: &[NetworkRequest]) -> Vec<Value> {
    records.iter().map(span_for_request).collect()
}

fn span_for_request(record: &NetworkRequest) -> Value {
    let ids = ids_for_request(record);
    let start = record.timing.start_unix_nano();
    let end = record.timing.end_unix_nano();

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

fn span_attrs(record: &NetworkRequest) -> Vec<KeyValue> {
    let mut attrs = base_attrs(record);
    attrs.extend(timing_attrs(record));
    attrs.extend(header_attrs(record));
    attrs.extend(trigger_attrs(record));
    attrs
}

fn base_attrs(record: &NetworkRequest) -> Vec<KeyValue> {
    [
        Some(KeyValue::new("hosho.schema", record.schema.clone())),
        Some(KeyValue::new(
            "http.request.method",
            record.request.method.clone(),
        )),
        Some(KeyValue::new("url.full", record.request.url.clone())),
        string_attr("url.scheme", &record.request.scheme),
        string_attr("server.address", &record.request.host),
        record
            .request
            .port
            .map(|port| KeyValue::new("server.port", i64::from(port))),
        int_attr("http.response.status_code", record.response.status),
        Some(KeyValue::new(
            "hosho.har.entry_hash",
            record.identity.entry_hash.clone(),
        )),
        Some(KeyValue::new(
            "hosho.har.dedupe_key",
            record.identity.dedupe_key.clone(),
        )),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn timing_attrs(record: &NetworkRequest) -> Vec<KeyValue> {
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

fn header_attrs(record: &NetworkRequest) -> Vec<KeyValue> {
    allowed_header_attributes("http.request.header", &record.request.headers)
        .into_iter()
        .chain(allowed_header_attributes(
            "http.response.header",
            &record.response.headers,
        ))
        .map(|(key, values)| string_array_attr(key, values))
        .collect()
}

fn trigger_attrs(record: &NetworkRequest) -> Vec<KeyValue> {
    let Some(trigger) = record.trace.trigger.as_ref() else {
        return Vec::new();
    };

    [
        string_attr("code.function", &trigger.function_name),
        string_attr("code.file.path", &trigger.url),
        int_attr("code.line.number", trigger.line_number),
        int_attr("code.column.number", trigger.column_number),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn span_status(record: &NetworkRequest) -> Value {
    match record.response.status {
        Some(100..=399) => json!({}),
        Some(status @ 400..=599) => json!({"code": 2, "message": status.to_string()}),
        _ => json!({"code": 2, "message": "network_error"}),
    }
}
