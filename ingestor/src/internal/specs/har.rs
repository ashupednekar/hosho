use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const HAR_ENTRY_SCHEMA: &str = "hosho.har.entry.v1";
pub const HAR_BATCH_SCHEMA: &str = "hosho.har.batch.v1";
pub const NETWORK_REQUEST_SCHEMA: &str = "hosho.network.request.v1";

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HarCapture {
    pub source: Option<String>,
    pub session_id: Option<String>,
    pub page_id: Option<String>,
    pub tab_id: Option<i64>,
    pub page_url: Option<String>,
    pub extension_version: Option<String>,
    pub captured_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarEntryEnvelope {
    #[serde(default)]
    pub capture: HarCapture,
    pub entry: Value,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarBatchEnvelope {
    #[serde(default)]
    pub capture: HarCapture,
    #[serde(default)]
    pub entries: Vec<Value>,
}
