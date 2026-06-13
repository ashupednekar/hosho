use super::strategy::{ExportStrategy, OtlpCollectorStrategy};
use crate::{
    internal::telemetry::{ExportRequest, TelemetryBatch},
    prelude::Result,
    settings::settings,
};
use axum::http::StatusCode;
use standard_error::{Interpolate, StandardError, Status};
use worker::{Fetch, Headers, Method, Request, RequestInit};

pub async fn export(batch: TelemetryBatch) -> Result<()> {
    OtlpHttpExporter::new(&settings.otel_collector_endpoint)
        .export(batch, &OtlpCollectorStrategy)
        .await
}

struct OtlpHttpExporter {
    endpoint: &'static str,
}

impl OtlpHttpExporter {
    const fn new(endpoint: &'static str) -> Self {
        Self { endpoint }
    }

    async fn export(&self, batch: TelemetryBatch, strategy: &impl ExportStrategy) -> Result<()> {
        for request in strategy.requests(batch) {
            self.send(request).await?;
        }
        Ok(())
    }

    async fn send(&self, request: ExportRequest) -> Result<()> {
        let body = serde_json::to_string(&request.body)
            .map_err(|error| StandardError::new("ERR-OTEL-001").code(StatusCode::BAD_REQUEST))?;
        let response = Fetch::Request(self.request(request.signal.path(), body)?)
            .send()
            .await
            .map_err(|error| StandardError::new("ERR-OTEL-002").code(StatusCode::BAD_REQUEST))?;

        if (200..300).contains(&response.status_code()) {
            Ok(())
        } else {
            Err(StandardError::new("ERR-OTEL-003"))
        }
    }

    fn request(&self, path: &str, body: String) -> Result<Request> {
        let headers = Headers::new();
        headers
            .set("content-type", "application/json")
            .map_err(|error| {
                StandardError::new("ERR-OTEL-002").interpolate_err(error.to_string())
            })?;

        let mut init = RequestInit::new();
        init.with_method(Method::Post)
            .with_headers(headers)
            .with_body(Some(worker::wasm_bindgen::JsValue::from_str(&body)));

        Request::new_with_init(&format!("{}{}", self.endpoint, path), &init)
            .map_err(|error| StandardError::new("ERR-OTEL-002").interpolate_err(error.to_string()))
    }
}
