use serde_json::Value;

use crate::internal::{
    error::{IngestError, IngestResult},
    specs::{
        har::{
            HarBatchEnvelope, HarCapture, HarEntryEnvelope, HAR_BATCH_SCHEMA, HAR_ENTRY_SCHEMA,
            NETWORK_REQUEST_SCHEMA,
        },
        network::{HttpRequestSpec, HttpResponseSpec, NetworkRequest},
    },
};

use super::{
    headers::har_headers, identity::identity_for_entry, initiator::browser_context,
    time::timing_from_entry, url::sanitize_url,
};

pub fn normalize_har_payload(payload: Value) -> IngestResult<Vec<NetworkRequest>> {
    let entries = extract_entries(payload)?;
    entries
        .into_iter()
        .map(|(capture, entry)| normalize_entry(capture, entry))
        .collect()
}

fn extract_entries(payload: Value) -> IngestResult<Vec<(HarCapture, Value)>> {
    match payload.get("schema").and_then(Value::as_str) {
        Some(HAR_ENTRY_SCHEMA) => entry_envelope(payload),
        Some(HAR_BATCH_SCHEMA) => batch_envelope(payload),
        _ if looks_like_har_entry(&payload) => Ok(vec![(HarCapture::default(), payload)]),
        _ if payload.pointer("/log/entries").is_some() => har_log(payload),
        _ => Err(IngestError::InvalidPayload("unsupported HAR payload")),
    }
}

fn normalize_entry(capture: HarCapture, entry: Value) -> IngestResult<NetworkRequest> {
    let request = request_spec(&entry)?;
    let response = response_spec(&entry);

    Ok(NetworkRequest {
        schema: NETWORK_REQUEST_SCHEMA.to_string(),
        capture,
        identity: identity_for_entry(&entry, &request),
        request,
        response,
        timing: timing_from_entry(&entry),
        browser: browser_context(&entry),
    })
}

fn entry_envelope(payload: Value) -> IngestResult<Vec<(HarCapture, Value)>> {
    let envelope: HarEntryEnvelope = serde_json::from_value(payload)
        .map_err(|_| IngestError::InvalidPayload("invalid HAR entry envelope"))?;
    Ok(vec![(envelope.capture, envelope.entry)])
}

fn batch_envelope(payload: Value) -> IngestResult<Vec<(HarCapture, Value)>> {
    let envelope: HarBatchEnvelope = serde_json::from_value(payload)
        .map_err(|_| IngestError::InvalidPayload("invalid HAR batch envelope"))?;
    Ok(envelope
        .entries
        .into_iter()
        .map(|entry| (envelope.capture.clone(), entry))
        .collect())
}

fn har_log(payload: Value) -> IngestResult<Vec<(HarCapture, Value)>> {
    let entries = payload
        .pointer("/log/entries")
        .and_then(Value::as_array)
        .ok_or(IngestError::InvalidPayload("invalid HAR log"))?;
    Ok(entries
        .iter()
        .cloned()
        .map(|entry| (HarCapture::default(), entry))
        .collect())
}

fn looks_like_har_entry(payload: &Value) -> bool {
    payload.get("request").is_some()
        && payload.get("response").is_some()
        && payload.get("startedDateTime").is_some()
}

fn request_spec(entry: &Value) -> IngestResult<HttpRequestSpec> {
    let request = entry
        .get("request")
        .ok_or(IngestError::InvalidPayload("HAR entry missing request"))?;
    let url = request
        .get("url")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let parts = sanitize_url(url);

    Ok(HttpRequestSpec {
        method: request
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or("_OTHER")
            .to_string(),
        url: url.to_string(),
        url_sanitized: parts.sanitized,
        scheme: parts.scheme,
        host: parts.host,
        port: parts.port,
        http_version: request
            .get("httpVersion")
            .and_then(Value::as_str)
            .map(str::to_string),
        headers: har_headers(request.get("headers")),
        headers_size: non_negative_i64(request.get("headersSize")),
        body_size: non_negative_i64(request.get("bodySize")),
    })
}

fn response_spec(entry: &Value) -> HttpResponseSpec {
    let response = entry.get("response").unwrap_or(&Value::Null);

    HttpResponseSpec {
        status: non_negative_i64(response.get("status")),
        status_text: response
            .get("statusText")
            .and_then(Value::as_str)
            .map(str::to_string),
        http_version: response
            .get("httpVersion")
            .and_then(Value::as_str)
            .map(str::to_string),
        mime_type: response
            .pointer("/content/mimeType")
            .and_then(Value::as_str)
            .map(str::to_string),
        headers: har_headers(response.get("headers")),
        headers_size: non_negative_i64(response.get("headersSize")),
        body_size: non_negative_i64(response.get("bodySize")),
        from_cache: response
            .get("_fromDiskCache")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            || response
                .get("_fromMemoryCache")
                .and_then(Value::as_bool)
                .unwrap_or(false),
    }
}

fn non_negative_i64(value: Option<&Value>) -> Option<i64> {
    value.and_then(Value::as_i64).filter(|number| *number >= 0)
}
