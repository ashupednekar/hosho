use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestTiming {
    pub started_at: Option<String>,
    pub duration_ms: Option<f64>,
    pub blocked_ms: Option<f64>,
    pub dns_ms: Option<f64>,
    pub connect_ms: Option<f64>,
    pub ssl_ms: Option<f64>,
    pub send_ms: Option<f64>,
    pub wait_ms: Option<f64>,
    pub receive_ms: Option<f64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserContext {
    pub resource_type: Option<String>,
    pub priority: Option<String>,
    pub connection_id: Option<String>,
    pub initiator: Option<InitiatorContext>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitiatorContext {
    pub top_function: Option<String>,
    pub top_url: Option<String>,
    pub top_line: Option<i64>,
    pub top_column: Option<i64>,
    pub stack_depth: Option<usize>,
}
