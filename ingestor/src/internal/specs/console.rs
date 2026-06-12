use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const CONSOLE_EVENT_SCHEMA: &str = "hosho.console.event.v1";
pub const CONSOLE_BATCH_SCHEMA: &str = "hosho.console.batch.v1";

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleCapture {
    pub session_id: Option<String>,
    pub page_id: Option<String>,
    pub tab_id: Option<i64>,
    pub page_url: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleRecord {
    pub level: String,
    pub body: String,
    pub args: Vec<Value>,
    pub url: Option<String>,
    pub tab_id: Option<i64>,
    pub captured_at: Option<String>,
    pub capture: ConsoleCapture,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleEventEnvelope {
    #[serde(default)]
    pub capture: ConsoleCapture,
    pub event: Value,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleBatchEnvelope {
    #[serde(default)]
    pub capture: ConsoleCapture,
    #[serde(default)]
    pub events: Vec<Value>,
}
