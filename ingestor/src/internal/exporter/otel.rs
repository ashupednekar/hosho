use async_trait::async_trait;
use opentelemetry::KeyValue;
use opentelemetry_http::{
    Bytes, HttpClient, HttpError, Request as OtelRequest, Response as OtelResponse,
};
use opentelemetry_otlp::{Protocol, WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::{
    logs::SdkLoggerProvider, metrics::SdkMeterProvider, trace::SdkTracerProvider, Resource,
};
use worker::js_sys::Uint8Array;
use worker::wasm_bindgen::JsValue;
use worker::{Fetch, Headers, Method, Request, RequestInit};


#[derive(Debug)]
struct TelemetryClient;

#[async_trait]
impl HttpClient for TelemetryClient {
    async fn send_bytes(
        &self,
        request: OtelRequest<Bytes>,
    ) -> Result<OtelResponse<Bytes>, HttpError> {
        worker::send::SendFuture::new(send_worker_request(request)).await
    }
}

async fn send_worker_request(
    request: OtelRequest<Bytes>,
) -> Result<OtelResponse<Bytes>, HttpError> {
    let (parts, body) = request.into_parts();
    let headers = Headers::new();

    for (name, value) in parts.headers.iter() {
        headers
            .set(name.as_str(), value.to_str().map_err(http_error)?)
            .map_err(http_error)?;
    }

    let body = Uint8Array::from(body.as_ref());
    let mut init = RequestInit::new();
    init.with_method(Method::from(parts.method.as_str().to_string()))
        .with_headers(headers)
        .with_body(Some(JsValue::from(body)));

    let worker_request =
        Request::new_with_init(&parts.uri.to_string(), &init).map_err(http_error)?;
    let mut worker_response = Fetch::Request(worker_request)
        .send()
        .await
        .map_err(http_error)?;
    let status = worker_response.status_code();
    let headers = worker_response.headers().entries().collect::<Vec<_>>();
    let body = worker_response.bytes().await.map_err(http_error)?;

    let mut response = OtelResponse::builder().status(status);
    for (name, value) in headers {
        response = response.header(name, value);
    }

    response.body(Bytes::from(body)).map_err(http_error)
}

pub fn logger_provider() -> worker::Result<SdkLoggerProvider> {
    let exporter = opentelemetry_otlp::LogExporter::builder()
        .with_http()
        .with_http_client(TelemetryClient)
        .with_endpoint(otel_url("/v1/logs"))
        .with_protocol(Protocol::HttpBinary)
        .build()
        .map_err(exporter_error)?;

    Ok(SdkLoggerProvider::builder()
        .with_resource(resource())
        .with_simple_exporter(exporter)
        .build())
}

pub fn tracer_provider() -> worker::Result<SdkTracerProvider> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_http_client(TelemetryClient)
        .with_endpoint(otel_url("/v1/traces"))
        .with_protocol(Protocol::HttpBinary)
        .build()
        .map_err(exporter_error)?;

    Ok(SdkTracerProvider::builder()
        .with_resource(resource())
        .with_simple_exporter(exporter)
        .build())
}

pub fn meter_provider() -> worker::Result<SdkMeterProvider> {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_http()
        .with_http_client(TelemetryClient)
        .with_endpoint(otel_url("/v1/metrics"))
        .with_protocol(Protocol::HttpBinary)
        .build()
        .map_err(exporter_error)?;

    Ok(SdkMeterProvider::builder()
        .with_resource(resource())
        .with_periodic_exporter(exporter)
        .build())
}

fn resource() -> Resource {
    Resource::builder()
        .with_service_name("hosho-browser")
        .with_attributes([
            KeyValue::new("service.namespace", "hosho"),
            KeyValue::new("deployment.environment.name", "browser-extension"),
        ])
        .build()
}

fn otel_url(path: &str) -> String {
    format!("{OTEL_ENDPOINT}{path}")
}

fn exporter_error(error: impl std::fmt::Display) -> worker::Error {
    worker::Error::RustError(format!("otel exporter setup failed: {error}"))
}

fn http_error(error: impl std::fmt::Display) -> HttpError {
    Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        error.to_string(),
    ))
}
