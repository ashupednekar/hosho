use crate::internal::{normalize::identity::hash_parts, specs::network::NetworkRequest};

pub struct SpanIds {
    pub trace_id: String,
    pub span_id: String,
}

pub fn ids_for_request(record: &NetworkRequest) -> SpanIds {
    if let Some((trace_id, span_id)) = traceparent_ids(record) {
        return SpanIds { trace_id, span_id };
    }

    SpanIds {
        trace_id: compact_hash("hosho.trace.v1", trace_parts(record), 32),
        span_id: compact_hash(
            "hosho.span.v1",
            vec![record.identity.dedupe_key.as_str()],
            16,
        ),
    }
}

fn traceparent_ids(record: &NetworkRequest) -> Option<(String, String)> {
    let header = record.request.headers.get("traceparent")?.first()?;
    let parts: Vec<&str> = header.split('-').collect();
    let valid = parts.len() == 4 && parts[1].len() == 32 && parts[2].len() == 16;
    valid.then(|| (parts[1].to_string(), parts[2].to_string()))
}

fn trace_parts(record: &NetworkRequest) -> Vec<&str> {
    vec![
        record.capture.session_id.as_deref().unwrap_or_default(),
        record.capture.page_id.as_deref().unwrap_or_default(),
        record.request.host.as_deref().unwrap_or_default(),
        record.timing.started_at.as_deref().unwrap_or_default(),
    ]
}

fn compact_hash(prefix: &str, parts: Vec<&str>, chars: usize) -> String {
    hash_parts(prefix, &parts)
        .trim_start_matches("sha256:")
        .chars()
        .take(chars)
        .collect()
}
