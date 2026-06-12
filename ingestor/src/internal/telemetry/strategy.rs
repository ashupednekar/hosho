use super::{
    config::OtlpSignal,
    payload::{ExportRequest, TelemetryBatch},
};

pub(crate) trait ExportStrategy {
    fn requests(&self, batch: TelemetryBatch) -> Vec<ExportRequest>;
}

pub struct OtlpCollectorStrategy;

impl ExportStrategy for OtlpCollectorStrategy {
    fn requests(&self, batch: TelemetryBatch) -> Vec<ExportRequest> {
        let mut requests = Vec::new();
        if let Some(body) = batch.traces {
            requests.push(ExportRequest {
                signal: OtlpSignal::Traces,
                body,
            });
        }
        if let Some(body) = batch.logs {
            requests.push(ExportRequest {
                signal: OtlpSignal::Logs,
                body,
            });
        }
        if let Some(body) = batch.metrics {
            requests.push(ExportRequest {
                signal: OtlpSignal::Metrics,
                body,
            });
        }
        requests
    }
}
