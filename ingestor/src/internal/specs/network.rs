use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{
    har::HarCapture,
    timing::{BrowserContext, RequestTiming},
};

pub type HeaderMap = BTreeMap<String, Vec<String>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkRequest {
    pub schema: String,
    pub capture: HarCapture,
    pub identity: RequestIdentity,
    pub request: HttpRequestSpec,
    pub response: HttpResponseSpec,
    pub timing: RequestTiming,
    pub browser: BrowserContext,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestIdentity {
    pub request_id: Option<String>,
    pub entry_hash: String,
    pub dedupe_key: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpRequestSpec {
    pub method: String,
    pub url: String,
    pub url_sanitized: String,
    pub scheme: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub http_version: Option<String>,
    pub headers: HeaderMap,
    pub headers_size: Option<i64>,
    pub body_size: Option<i64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpResponseSpec {
    pub status: Option<i64>,
    pub status_text: Option<String>,
    pub http_version: Option<String>,
    pub mime_type: Option<String>,
    pub headers: HeaderMap,
    pub headers_size: Option<i64>,
    pub body_size: Option<i64>,
    pub from_cache: bool,
}
