use std::time::{Duration, SystemTime, UNIX_EPOCH};

use opentelemetry::trace::{
    Span as _, SpanKind, Status, TraceContextExt as _, Tracer as _, TracerProvider as _,
};
use opentelemetry::{Context, KeyValue};

use crate::internal::exporter::otel::tracer_provider;
use crate::internal::exporter::spec::har::HarPayload;

pub async fn send_trace(har: &HarPayload) -> worker::Result<()> {
    let provider = tracer_provider()?;
    let tracer = provider.tracer("hosho.har");
    let start = har
        .timing
        .started_at
        .as_deref()
        .and_then(parse_iso8601_utc_nanos)
        .map(system_time_from_nanos);
    let end = start.and_then(|start| {
        har.timing
            .duration_ms
            .map(|duration| add_ms(start, duration))
    });

    let root_builder = tracer
        .span_builder(request_span_name(har))
        .with_kind(SpanKind::Client)
        .with_attributes(request_attributes(har));
    let root_builder = match start {
        Some(start) => root_builder.with_start_time(start),
        None => root_builder,
    };
    let mut root = tracer.build(root_builder);
    if har.response.status.is_some_and(|status| status >= 500) {
        root.set_status(Status::error("HTTP 5xx response"));
    }

    let root_context = Context::current_with_span(root);
    emit_phase_spans(har, &tracer, &root_context, start);
    match end {
        Some(end) => root_context.span().end_with_timestamp(end),
        None => root_context.span().end(),
    }

    provider.force_flush().map_err(export_error)
}

fn emit_phase_spans(
    har: &HarPayload,
    tracer: &opentelemetry_sdk::trace::SdkTracer,
    root_context: &Context,
    start: Option<SystemTime>,
) {
    let Some(base) = start else {
        return;
    };

    let phases = [
        ("request.blocked", har.timing.blocked_ms),
        ("connection.dns", har.timing.dns_ms),
        ("connection.tcp", har.timing.connect_ms),
        ("connection.tls", har.timing.ssl_ms),
        ("request.send", har.timing.send_ms),
        ("response.wait", har.timing.wait_ms),
        ("response.receive", har.timing.receive_ms),
    ];

    let mut offset = Duration::ZERO;
    for (name, duration_ms) in phases {
        let Some(duration_ms) = duration_ms.filter(|duration| *duration > 0.0) else {
            continue;
        };

        let phase_start = base + offset;
        let phase_duration = duration_from_ms(duration_ms);
        let phase_end = phase_start + phase_duration;
        offset += phase_duration;

        let mut span = tracer.build_with_context(
            tracer
                .span_builder(name)
                .with_kind(SpanKind::Internal)
                .with_start_time(phase_start)
                .with_attributes(vec![KeyValue::new(
                    "hosho.har.phase.duration_ms",
                    duration_ms,
                )]),
            root_context,
        );
        span.end_with_timestamp(phase_end);
    }

    if let Some(duration_ms) = har.timing.duration_ms {
        let closed_at = base + duration_from_ms(duration_ms);
        let mut span = tracer.build_with_context(
            tracer
                .span_builder("response.closed")
                .with_kind(SpanKind::Internal)
                .with_start_time(closed_at),
            root_context,
        );
        span.end_with_timestamp(closed_at);
    }
}

fn request_span_name(har: &HarPayload) -> String {
    har.request
        .method
        .clone()
        .filter(|method| !method.is_empty())
        .unwrap_or_else(|| "HTTP".to_string())
}

fn request_attributes(har: &HarPayload) -> Vec<KeyValue> {
    let mut attributes = vec![KeyValue::new("hosho.signal", "har")];

    if let Some(method) = &har.request.method {
        attributes.push(KeyValue::new("http.request.method", method.clone()));
    }

    if let Some(url) = &har.request.url {
        attributes.push(KeyValue::new("url.full", url.clone()));
    }

    if let Some(status) = har.response.status {
        attributes.push(KeyValue::new("http.response.status_code", status));
    }

    if let Some(started_at) = &har.timing.started_at {
        attributes.push(KeyValue::new("hosho.har.started_at", started_at.clone()));
    }

    push_timing(&mut attributes, "duration_ms", har.timing.duration_ms);
    push_timing(&mut attributes, "blocked_ms", har.timing.blocked_ms);
    push_timing(&mut attributes, "dns_ms", har.timing.dns_ms);
    push_timing(&mut attributes, "connect_ms", har.timing.connect_ms);
    push_timing(&mut attributes, "ssl_ms", har.timing.ssl_ms);
    push_timing(&mut attributes, "send_ms", har.timing.send_ms);
    push_timing(&mut attributes, "wait_ms", har.timing.wait_ms);
    push_timing(&mut attributes, "receive_ms", har.timing.receive_ms);

    attributes
}

fn push_timing(attributes: &mut Vec<KeyValue>, name: &str, value: Option<f64>) {
    if let Some(value) = value {
        attributes.push(KeyValue::new(format!("hosho.har.timing.{name}"), value));
    }
}

fn system_time_from_nanos(nanos: u128) -> SystemTime {
    UNIX_EPOCH + Duration::from_nanos(nanos.min(u64::MAX as u128) as u64)
}

fn add_ms(time: SystemTime, ms: f64) -> SystemTime {
    time + duration_from_ms(ms)
}

fn duration_from_ms(ms: f64) -> Duration {
    Duration::from_nanos((ms.max(0.0) * 1_000_000.0) as u64)
}

fn parse_iso8601_utc_nanos(value: &str) -> Option<u128> {
    let value = value.strip_suffix('Z')?;
    let (date, time) = value.split_once('T')?;
    let mut date_parts = date.split('-');
    let year = date_parts.next()?.parse::<i64>().ok()?;
    let month = date_parts.next()?.parse::<u32>().ok()?;
    let day = date_parts.next()?.parse::<u32>().ok()?;

    let mut time_parts = time.split(':');
    let hour = time_parts.next()?.parse::<u32>().ok()?;
    let minute = time_parts.next()?.parse::<u32>().ok()?;
    let second_and_fraction = time_parts.next()?;

    let (second, fraction) = match second_and_fraction.split_once('.') {
        Some((second, fraction)) => (second.parse::<u32>().ok()?, fraction),
        None => (second_and_fraction.parse::<u32>().ok()?, ""),
    };

    let fraction_nanos = parse_fraction_nanos(fraction);
    let days = days_from_civil(year, month, day);
    let seconds = days
        .checked_mul(86_400)?
        .checked_add(i64::from(hour) * 3_600)?
        .checked_add(i64::from(minute) * 60)?
        .checked_add(i64::from(second))?;

    if seconds < 0 {
        return None;
    }

    Some((seconds as u128) * 1_000_000_000 + u128::from(fraction_nanos))
}

fn parse_fraction_nanos(fraction: &str) -> u32 {
    let mut nanos = 0u32;
    let mut scale = 100_000_000u32;

    for char in fraction.chars().take(9) {
        if let Some(digit) = char.to_digit(10) {
            nanos += digit * scale;
            scale /= 10;
        } else {
            break;
        }
    }

    nanos
}

fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month = i64::from(month);
    let day = i64::from(day);
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

    era * 146_097 + day_of_era - 719_468
}

fn export_error(error: impl std::fmt::Display) -> worker::Error {
    worker::Error::RustError(format!("otel trace export failed: {error}"))
}
