use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use standard_error::{Interpolate, StandardError, Status};

use super::{network::TraceContext, timing::RequestTiming};
use crate::prelude::Result;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleRecord {
    pub level: String,
    pub body: String,
    pub args: Vec<Value>,
    pub url: Option<String>,
    pub captured_at: Option<String>,
}

impl TryFrom<Value> for ConsoleRecord {
    type Error = StandardError;

    fn try_from(event: Value) -> Result<Self> {
        if event.get("level").is_none() {
            return Err(StandardError::new("ERR-CAPTURE-001").code(StatusCode::BAD_REQUEST));
        }

        let args = event
            .get("args")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        Ok(Self {
            level: event
                .get("level")
                .and_then(Value::as_str)
                .unwrap_or("log")
                .to_string(),
            body: args
                .iter()
                .map(|arg| {
                    arg.as_str()
                        .map(str::to_string)
                        .unwrap_or_else(|| arg.to_string())
                })
                .collect::<Vec<_>>()
                .join(" "),
            args,
            url: event.get("url").and_then(Value::as_str).map(str::to_string),
            captured_at: event
                .get("capturedAt")
                .and_then(Value::as_str)
                .map(str::to_string),
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HarRecord {
    pub request: HarRequest,
    pub response: HarResponse,
    pub timing: RequestTiming,
    #[serde(default)]
    pub trace: TraceContext,
}

impl TryFrom<Value> for HarRecord {
    type Error = StandardError;

    fn try_from(payload: Value) -> Result<Self> {
        serde_json::from_value(payload).map_err(|e| {
            StandardError::new("ERR-CAPTURE-002")
                .code(StatusCode::BAD_REQUEST)
                .interpolate_err(e.to_string())
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HarRequest {
    pub method: Option<String>,
    pub url: Option<String>,
    pub http_version: Option<String>,
    #[serde(default)]
    pub headers: Vec<HarHeader>,
    pub headers_size: Option<i64>,
    pub body_size: Option<i64>,
    pub post_data: Option<HarPostData>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HarResponse {
    pub status: Option<i64>,
    pub status_text: Option<String>,
    pub http_version: Option<String>,
    #[serde(default)]
    pub headers: Vec<HarHeader>,
    pub headers_size: Option<i64>,
    pub body_size: Option<i64>,
    pub content: Option<HarContent>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct HarHeader {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub value: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HarContent {
    pub mime_type: Option<String>,
    pub text: Option<String>,
    pub encoding: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HarPostData {
    pub mime_type: Option<String>,
    pub text: Option<String>,
}
