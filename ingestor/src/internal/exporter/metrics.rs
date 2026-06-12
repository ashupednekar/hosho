use opentelemetry::metrics::MeterProvider as _;
use opentelemetry::KeyValue;

use crate::internal::exporter::logs::ConsoleLogPayload;
use crate::internal::exporter::otel::meter_provider;
use crate::internal::exporter::spec::har::HarPayload;

pub async fn send_console_metrics(log: &ConsoleLogPayload) -> worker::Result<()> {
    let provider = meter_provider()?;
    let meter = provider.meter("hosho.console");
    let counter = meter
        .u64_counter("hosho.console.logs")
        .with_unit("{log}")
        .build();

    counter.add(1, &[KeyValue::new("log.level", normalized_level(log))]);
    provider.force_flush().map_err(export_error)
}

pub async fn send_har_metrics(har: &HarPayload) -> worker::Result<()> {
    let provider = meter_provider()?;
    let meter = provider.meter("hosho.har");
    let attributes = har_attributes(har);

    meter
        .u64_counter("hosho.har.requests")
        .with_unit("{request}")
        .build()
        .add(1, &attributes);

    meter
        .f64_gauge("hosho.har.requests_per_page_load")
        .with_unit("{request}")
        .build()
        .record(1.0, &attributes);

    if let Some(duration_ms) = har.timing.duration_ms {
        meter
            .f64_gauge("hosho.har.request.duration")
            .with_unit("ms")
            .build()
            .record(duration_ms, &attributes);
    }

    provider.force_flush().map_err(export_error)
}

fn har_attributes(har: &HarPayload) -> Vec<KeyValue> {
    let mut attributes = vec![
        KeyValue::new("hosho.signal", "har"),
        KeyValue::new(
            "http.response.status_class",
            status_class(har.response.status),
        ),
    ];

    if let Some(method) = &har.request.method {
        attributes.push(KeyValue::new("http.request.method", method.clone()));
    }

    if let Some(status) = har.response.status {
        attributes.push(KeyValue::new("http.response.status_code", status));
        attributes.push(KeyValue::new("error", status >= 500));
    }

    if let Some(url) = &har.request.url {
        attributes.push(KeyValue::new("url.full", url.clone()));
    }

    attributes
}

fn normalized_level(log: &ConsoleLogPayload) -> &'static str {
    match log.level.as_str() {
        "debug" => "debug",
        "warn" => "warn",
        "error" => "error",
        "info" => "info",
        "log" => "log",
        _ => "info",
    }
}

fn status_class(status: Option<i64>) -> &'static str {
    match status {
        Some(200..=299) => "2xx",
        Some(300..=399) => "3xx",
        Some(400..=499) => "4xx",
        Some(500..=599) => "5xx",
        Some(_) => "other",
        None => "unknown",
    }
}

fn export_error(error: impl std::fmt::Display) -> worker::Error {
    worker::Error::RustError(format!("otel metric export failed: {error}"))
}
