use worker::{Fetch, Headers, Method, Request, RequestInit};

use crate::internal::error::{IngestError, IngestResult};

use super::{
    config::OTEL_COLLECTOR_ENDPOINT,
    payload::{ExportRequest, TelemetryBatch},
    strategy::{ExportStrategy, OtlpCollectorStrategy},
};

pub async fn export(batch: TelemetryBatch) -> IngestResult<()> {
    OtlpHttpExporter::new(OTEL_COLLECTOR_ENDPOINT)
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

    async fn export(
        &self,
        batch: TelemetryBatch,
        strategy: &impl ExportStrategy,
    ) -> IngestResult<()> {
        for request in strategy.requests(batch) {
            self.send(request).await?;
        }
        Ok(())
    }

    async fn send(&self, request: ExportRequest) -> IngestResult<()> {
        let body = serde_json::to_string(&request.body)
            .map_err(|error| IngestError::Export(error.to_string()))?;
        let response = Fetch::Request(self.request(request.signal.path(), body)?)
            .send()
            .await
            .map_err(|error| IngestError::Export(error.to_string()))?;

        if (200..300).contains(&response.status_code()) {
            Ok(())
        } else {
            Err(IngestError::Export(format!(
                "OTLP collector returned {}",
                response.status_code()
            )))
        }
    }

    fn request(&self, path: &str, body: String) -> IngestResult<Request> {
        let headers = Headers::new();
        headers
            .set("content-type", "application/json")
            .map_err(|error| IngestError::Export(error.to_string()))?;

        let mut init = RequestInit::new();
        init.with_method(Method::Post)
            .with_headers(headers)
            .with_body(Some(worker::wasm_bindgen::JsValue::from_str(&body)));

        Request::new_with_init(&format!("{}{}", self.endpoint, path), &init)
            .map_err(|error| IngestError::Export(error.to_string()))
    }
}
