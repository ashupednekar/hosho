use std::{
    borrow::Cow,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use axum::http::StatusCode;
use opentelemetry::{
    logs::{AnyValue, LogRecord as _, Logger as _, LoggerProvider as _, Severity},
    trace::{SpanContext, SpanId, SpanKind, Status as SpanStatus, TraceFlags, TraceId, TraceState},
    Array, InstrumentationScope, KeyValue, Value as OtelValue,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_otlp::{LogExporter as OtlpLogExporter, SpanExporter as OtlpSpanExporter};
use opentelemetry_sdk::{logs::LogExporter as _, trace::SpanExporter as _};
use opentelemetry_sdk::{
    logs::{LogBatch, SdkLogRecord, SdkLoggerProvider},
    trace::{SpanData, SpanEvents, SpanLinks},
    Resource,
};
use standard_error::{Interpolate, StandardError, Status};

use crate::{
    internal::{
        specs::{
            capture::{ConsoleRecord, HarRecord},
            network::{allowed_header_attributes, NetworkRequest},
        },
        telemetry::ids::ids_for_request,
    },
    prelude::Result,
    settings::settings,
};

#[allow(async_fn_in_trait)]
pub trait Export {
    async fn export(&self) -> Result<()>;
}

impl Export for ConsoleRecord {
    async fn export(&self) -> Result<()> {
        let mut exporter = OtlpLogExporter::builder()
            .with_http()
            .with_endpoint(otlp_endpoint("/v1/logs"))
            .build()
            .map_err(|error| {
                StandardError::new("ERR-OTEL-002")
                    .code(StatusCode::BAD_GATEWAY)
                    .interpolate_err(error.to_string())
            })?;
        exporter.set_resource(&otel_resource());

        let (record, scope) = console_log_record(self)?;
        let batch = [(&record, &scope)];
        exporter
            .export(LogBatch::new(&batch))
            .await
            .map_err(|error| {
                StandardError::new("ERR-OTEL-003")
                    .code(StatusCode::BAD_GATEWAY)
                    .interpolate_err(error.to_string())
            })
    }
}

impl Export for HarRecord {
    async fn export(&self) -> Result<()> {
        let mut exporter = OtlpSpanExporter::builder()
            .with_http()
            .with_endpoint(otlp_endpoint("/v1/traces"))
            .build()
            .map_err(|error| {
                StandardError::new("ERR-OTEL-002")
                    .code(StatusCode::BAD_GATEWAY)
                    .interpolate_err(error.to_string())
            })?;
        exporter.set_resource(&otel_resource());

        exporter.export(har_spans(self)?).await.map_err(|error| {
            StandardError::new("ERR-OTEL-003")
                .code(StatusCode::BAD_GATEWAY)
                .interpolate_err(error.to_string())
        })
    }
}

fn otlp_endpoint(path: &str) -> String {
    format!(
        "{}{}",
        settings.otel_collector_endpoint.trim_end_matches('/'),
        path
    )
}

fn console_log_record(record: &ConsoleRecord) -> Result<(SdkLogRecord, InstrumentationScope)> {
    let scope = instrumentation_scope("hosho.console");
    let provider = SdkLoggerProvider::builder().build();
    let logger = provider.logger_with_scope(scope.clone());
    let mut log = logger.create_log_record();

    if let Some(timestamp) = record_time(record.captured_at.as_deref()) {
        log.set_timestamp(timestamp);
    }
    log.set_observed_timestamp(SystemTime::now());
    log.set_severity_number(severity(&record.level));
    log.set_severity_text(severity(&record.level).name());
    log.set_body(AnyValue::String(record.body.clone().into()));
    log.add_attribute("hosho.schema", "hosho.console.record.v1");
    if let Some(url) = record.url.as_deref() {
        log.add_attribute("url.full", url.to_string());
    }
    if !record.args.is_empty() {
        log.add_attribute(
            "hosho.console.args_json",
            serde_json::to_string(&record.args).map_err(|error| {
                StandardError::new("ERR-OTEL-001")
                    .code(StatusCode::INTERNAL_SERVER_ERROR)
                    .interpolate_err(error.to_string())
            })?,
        );
    }

    Ok((log, scope))
}

fn har_spans(record: &HarRecord) -> Result<Vec<SpanData>> {
    let network = NetworkRequest::from(record.clone());
    let ids = ids_for_request(&network);
    let start = network.timing.start_unix_nano().ok_or_else(|| {
        StandardError::new("ERR-OTEL-004")
            .code(StatusCode::BAD_REQUEST)
            .interpolate_err("HAR timing.startedAt is missing or not RFC3339".to_string())
    })?;
    let total_ms = network
        .timing
        .duration_ms
        .or_else(|| {
            let sum = [
                network.timing.blocked_ms,
                network.timing.dns_ms,
                network.timing.connect_ms,
                network.timing.send_ms,
                network.timing.wait_ms,
                network.timing.receive_ms,
            ]
            .into_iter()
            .flatten()
            .filter(|value| *value >= 0.0)
            .sum::<f64>();
            (sum > 0.0).then_some(sum)
        })
        .unwrap_or_default()
        .max(0.0);

    let scope = instrumentation_scope("hosho.har");
    let trace_id = parse_trace_id(&ids.trace_id)?;
    let root_span_id = parse_span_id(&ids.span_id)?;
    let root_span_id_text = ids.span_id;
    let mut spans = vec![span_data(
        trace_id,
        root_span_id,
        SpanId::INVALID,
        SpanKind::Client,
        format!("{} complete", network.request.method),
        start,
        nanos_at(start, total_ms),
        root_attrs(&network, record),
        span_status(network.response.status),
        scope.clone(),
    )];

    let mut offset_ms = 0.0;
    let mut add_phase = |name: &str, duration_ms: f64, mut attrs: Vec<KeyValue>| -> Result<()> {
        let duration_ms = duration_ms.max(0.0);
        let phase_start = nanos_at(start, offset_ms);
        let phase_end = nanos_at(start, offset_ms + duration_ms);
        attrs.push(KeyValue::new("hosho.har.phase", name.to_string()));
        attrs.push(KeyValue::new("hosho.har.phase.duration_ms", duration_ms));
        spans.push(span_data(
            trace_id,
            parse_span_id(&span_id(&root_span_id_text, name))?,
            root_span_id,
            SpanKind::Internal,
            name.to_string(),
            phase_start,
            phase_end,
            attrs,
            SpanStatus::Unset,
            scope.clone(),
        ));
        offset_ms += duration_ms;
        Ok(())
    };

    if let Some(blocked_ms) = phase_ms(network.timing.blocked_ms) {
        add_phase("blocked", blocked_ms, Vec::new())?;
    }
    if let Some(dns_ms) = phase_ms(network.timing.dns_ms) {
        add_phase("dns", dns_ms, Vec::new())?;
    }

    let connect_ms = phase_ms(network.timing.connect_ms);
    let ssl_ms = phase_ms(network.timing.ssl_ms);
    match (connect_ms, ssl_ms) {
        (Some(connect_ms), Some(ssl_ms)) if connect_ms > ssl_ms => {
            add_phase("connect", connect_ms - ssl_ms, Vec::new())?;
            add_phase("ssl", ssl_ms, Vec::new())?;
        }
        (Some(connect_ms), _) => add_phase("connect", connect_ms, Vec::new())?,
        (None, Some(ssl_ms)) => add_phase("ssl", ssl_ms, Vec::new())?,
        (None, None) => {}
    }

    if let Some(send_ms) = phase_ms(network.timing.send_ms) {
        add_phase(
            "request sent",
            send_ms,
            request_phase_attrs(&network, record),
        )?;
    }
    if let Some(wait_ms) = phase_ms(network.timing.wait_ms) {
        add_phase("response", wait_ms, response_phase_attrs(&network))?;
    }

    add_phase("headers", 0.0, header_phase_attrs(&network))?;

    if let Some(receive_ms) = phase_ms(network.timing.receive_ms) {
        add_phase("response body", receive_ms, response_body_attrs(record))?;
    }

    Ok(spans)
}

fn span_data(
    trace_id: TraceId,
    span_id: SpanId,
    parent_span_id: SpanId,
    span_kind: SpanKind,
    name: String,
    start: u64,
    end: u64,
    attributes: Vec<KeyValue>,
    status: SpanStatus,
    scope: InstrumentationScope,
) -> SpanData {
    SpanData {
        span_context: SpanContext::new(
            trace_id,
            span_id,
            TraceFlags::SAMPLED,
            false,
            TraceState::NONE,
        ),
        parent_span_id,
        parent_span_is_remote: false,
        span_kind,
        name: Cow::Owned(name),
        start_time: system_time(start),
        end_time: system_time(end),
        attributes,
        dropped_attributes_count: 0,
        events: SpanEvents::default(),
        links: SpanLinks::default(),
        status,
        instrumentation_scope: scope,
    }
}

fn root_attrs(network: &NetworkRequest, record: &HarRecord) -> Vec<KeyValue> {
    let mut attrs = vec![
        KeyValue::new("hosho.schema", network.schema.clone()),
        KeyValue::new("http.request.method", network.request.method.clone()),
        KeyValue::new("url.full", network.request.url.clone()),
        KeyValue::new("hosho.har.entry_hash", network.identity.entry_hash.clone()),
        KeyValue::new("hosho.har.dedupe_key", network.identity.dedupe_key.clone()),
    ];

    if let Some(scheme) = network.request.scheme.as_deref() {
        attrs.push(KeyValue::new("url.scheme", scheme.to_string()));
    }
    if let Some(host) = network.request.host.as_deref() {
        attrs.push(KeyValue::new("server.address", host.to_string()));
    }
    if let Some(port) = network.request.port {
        attrs.push(KeyValue::new("server.port", i64::from(port)));
    }
    if let Some(status) = network.response.status {
        attrs.push(KeyValue::new("http.response.status_code", status));
    }
    if let Some(size) = network.request.headers_size {
        attrs.push(KeyValue::new("http.request.header.size", size));
    }
    if let Some(size) = network.request.body_size {
        attrs.push(KeyValue::new("http.request.body.size", size));
    }
    if let Some(size) = network.response.headers_size {
        attrs.push(KeyValue::new("http.response.header.size", size));
    }
    if let Some(size) = network.response.body_size {
        attrs.push(KeyValue::new("http.response.body.size", size));
    }
    if let Some(mime_type) = network.response.mime_type.as_deref() {
        attrs.push(KeyValue::new(
            "http.response.body.mime_type",
            mime_type.to_string(),
        ));
    }

    for (key, values) in allowed_header_attributes("http.request.header", &network.request.headers)
    {
        attrs.push(string_array_attr(key, values));
    }
    for (key, values) in
        allowed_header_attributes("http.response.header", &network.response.headers)
    {
        attrs.push(string_array_attr(key, values));
    }

    if let Some(trigger) = network.trace.trigger.as_ref() {
        if let Some(function_name) = trigger.function_name.as_deref() {
            attrs.push(KeyValue::new("code.function", function_name.to_string()));
        }
        if let Some(url) = trigger.url.as_deref() {
            attrs.push(KeyValue::new("code.file.path", url.to_string()));
        }
        if let Some(line_number) = trigger.line_number {
            attrs.push(KeyValue::new("code.line.number", line_number));
        }
        if let Some(column_number) = trigger.column_number {
            attrs.push(KeyValue::new("code.column.number", column_number));
        }
    }

    if let Some(post_data) = record.request.post_data.as_ref() {
        if let Some(mime_type) = post_data.mime_type.as_deref() {
            attrs.push(KeyValue::new(
                "http.request.body.mime_type",
                mime_type.to_string(),
            ));
        }
        if let Some(text) = post_data.text.as_deref() {
            attrs.push(KeyValue::new("hosho.http.request.body", text.to_string()));
        }
    }
    if let Some(content) = record.response.content.as_ref() {
        if let Some(encoding) = content.encoding.as_deref() {
            attrs.push(KeyValue::new(
                "http.response.body.encoding",
                encoding.to_string(),
            ));
        }
        if let Some(text) = content.text.as_deref() {
            attrs.push(KeyValue::new("hosho.http.response.body", text.to_string()));
        }
    }

    attrs
}

fn request_phase_attrs(network: &NetworkRequest, record: &HarRecord) -> Vec<KeyValue> {
    let mut attrs = vec![KeyValue::new(
        "http.request.method",
        network.request.method.clone(),
    )];
    if let Some(post_data) = record.request.post_data.as_ref() {
        if let Some(mime_type) = post_data.mime_type.as_deref() {
            attrs.push(KeyValue::new(
                "http.request.body.mime_type",
                mime_type.to_string(),
            ));
        }
        if let Some(text) = post_data.text.as_deref() {
            attrs.push(KeyValue::new("hosho.http.request.body", text.to_string()));
        }
    }
    attrs
}

fn response_phase_attrs(network: &NetworkRequest) -> Vec<KeyValue> {
    let mut attrs = Vec::new();
    if let Some(status) = network.response.status {
        attrs.push(KeyValue::new("http.response.status_code", status));
    }
    attrs
}

fn header_phase_attrs(network: &NetworkRequest) -> Vec<KeyValue> {
    let mut attrs = Vec::new();
    if let Some(size) = network.request.headers_size {
        attrs.push(KeyValue::new("http.request.header.size", size));
    }
    if let Some(size) = network.response.headers_size {
        attrs.push(KeyValue::new("http.response.header.size", size));
    }
    for (key, values) in allowed_header_attributes("http.request.header", &network.request.headers)
    {
        attrs.push(string_array_attr(key, values));
    }
    for (key, values) in
        allowed_header_attributes("http.response.header", &network.response.headers)
    {
        attrs.push(string_array_attr(key, values));
    }
    attrs
}

fn response_body_attrs(record: &HarRecord) -> Vec<KeyValue> {
    let mut attrs = Vec::new();
    if let Some(content) = record.response.content.as_ref() {
        if let Some(mime_type) = content.mime_type.as_deref() {
            attrs.push(KeyValue::new(
                "http.response.body.mime_type",
                mime_type.to_string(),
            ));
        }
        if let Some(encoding) = content.encoding.as_deref() {
            attrs.push(KeyValue::new(
                "http.response.body.encoding",
                encoding.to_string(),
            ));
        }
        if let Some(text) = content.text.as_deref() {
            attrs.push(KeyValue::new("hosho.http.response.body", text.to_string()));
        }
    }
    attrs
}

fn otel_resource() -> Resource {
    Resource::builder_empty()
        .with_attributes([
            KeyValue::new("service.name", "hosho-browser"),
            KeyValue::new(
                "service.namespace",
                std::env::var("SERVICE_NAMESPACE").unwrap_or_else(|_| "hosho".to_string()),
            ),
            KeyValue::new("telemetry.sdk.name", "hosho"),
            KeyValue::new("telemetry.sdk.language", "webjs"),
        ])
        .build()
}

fn string_array_attr(key: String, values: Vec<String>) -> KeyValue {
    KeyValue::new(
        key,
        OtelValue::Array(Array::String(values.into_iter().map(Into::into).collect())),
    )
}

fn span_status(status: Option<i64>) -> SpanStatus {
    match status {
        Some(100..=399) => SpanStatus::Unset,
        Some(status @ 400..=599) => SpanStatus::error(format!("HTTP {status}")),
        Some(status) => SpanStatus::error(format!("unexpected HTTP status {status}")),
        None => SpanStatus::error("missing HTTP response status"),
    }
}

fn severity(level: &str) -> Severity {
    match level {
        "debug" => Severity::Debug,
        "warn" | "warning" => Severity::Warn,
        "error" => Severity::Error,
        _ => Severity::Info,
    }
}

fn record_time(timestamp: Option<&str>) -> Option<SystemTime> {
    crate::internal::specs::timing::unix_nano(timestamp).map(system_time)
}

fn system_time(nanos: u64) -> SystemTime {
    UNIX_EPOCH
        .checked_add(Duration::from_nanos(nanos))
        .unwrap_or(UNIX_EPOCH)
}

fn phase_ms(value: Option<f64>) -> Option<f64> {
    value.filter(|number| number.is_finite() && *number >= 0.0)
}

fn nanos_at(start: u64, offset_ms: f64) -> u64 {
    start.saturating_add((offset_ms.max(0.0) * 1_000_000.0).round() as u64)
}

fn parse_trace_id(value: &str) -> Result<TraceId> {
    let trace_id = TraceId::from_hex(value).map_err(|error| {
        StandardError::new("ERR-OTEL-001")
            .code(StatusCode::INTERNAL_SERVER_ERROR)
            .interpolate_err(format!("invalid trace id {value}: {error}"))
    })?;
    if trace_id == TraceId::INVALID {
        return Err(StandardError::new("ERR-OTEL-001")
            .code(StatusCode::INTERNAL_SERVER_ERROR)
            .interpolate_err("trace id must not be all zeroes".to_string()));
    }
    Ok(trace_id)
}

fn parse_span_id(value: &str) -> Result<SpanId> {
    let span_id = SpanId::from_hex(value).map_err(|error| {
        StandardError::new("ERR-OTEL-001")
            .code(StatusCode::INTERNAL_SERVER_ERROR)
            .interpolate_err(format!("invalid span id {value}: {error}"))
    })?;
    if span_id == SpanId::INVALID {
        return Err(StandardError::new("ERR-OTEL-001")
            .code(StatusCode::INTERNAL_SERVER_ERROR)
            .interpolate_err("span id must not be all zeroes".to_string()));
    }
    Ok(span_id)
}

fn span_id(parent_span_id: &str, name: &str) -> String {
    crate::internal::specs::network::hash_parts("hosho.phase.span.v1", &[parent_span_id, name])
        .trim_start_matches("sha256:")
        .chars()
        .take(16)
        .collect()
}

fn instrumentation_scope(name: &'static str) -> InstrumentationScope {
    InstrumentationScope::builder(name)
        .with_version(env!("CARGO_PKG_VERSION"))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::specs::capture::{ConsoleRecord, HarRecord};
    use serde_json::Value;

    #[test]
    fn console_fixture_builds_otlp_log_record() {
        let payload: Value =
            serde_json::from_str(include_str!("../../../tests/fixtures/console_record.json"))
                .unwrap();
        let record = ConsoleRecord::try_from(payload).unwrap();

        let (log, scope) = console_log_record(&record).unwrap();

        assert_eq!(scope.name(), "hosho.console");
        assert_eq!(log.severity_text(), Some("ERROR"));
        assert_eq!(log.severity_number(), Some(Severity::Error));
        assert_eq!(
            log.body(),
            Some(&AnyValue::String("upload failed 42".to_string().into()))
        );
        assert!(log
            .attributes_iter()
            .any(|(key, _)| key.as_str() == "url.full"));
    }

    #[test]
    fn har_fixture_builds_phase_spans_with_raw_body() {
        let payload: Value =
            serde_json::from_str(include_str!("../../../tests/fixtures/har_record.json")).unwrap();
        let record = HarRecord::try_from(payload).unwrap();

        let spans = har_spans(&record).unwrap();
        let names = spans
            .iter()
            .map(|span| span.name.as_ref())
            .collect::<Vec<_>>();

        assert!(names.contains(&"POST complete"));
        assert!(names.contains(&"ssl"));
        assert!(names.contains(&"request sent"));
        assert!(names.contains(&"response"));
        assert!(names.contains(&"headers"));

        let root = &spans[0];
        assert!(attrs(root).any(|attr| {
            attr.key.as_str() == "hosho.http.request.body"
                && attr.value.as_str() == "raw=not-json&bytes=\0"
        }));
    }

    fn attrs(span: &SpanData) -> impl Iterator<Item = &KeyValue> {
        span.attributes.iter()
    }
}
