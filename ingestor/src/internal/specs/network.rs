use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

use super::{
    capture::{HarHeader, HarRecord, HarRequest, HarResponse},
    timing::RequestTiming,
};

const NETWORK_REQUEST_SCHEMA: &str = "hosho.network.request.v1";
const ALLOWED_HEADERS: &[&str] = &["accept", "content-type", "traceparent"];
const SENSITIVE_HEADERS: &[&str] = &[
    "authorization",
    "cookie",
    "proxy-authorization",
    "set-cookie",
];
pub type HeaderMap = BTreeMap<String, Vec<String>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkRequest {
    pub schema: String,
    pub identity: RequestIdentity,
    pub request: HttpRequestSpec,
    pub response: HttpResponseSpec,
    pub timing: RequestTiming,
    pub trace: TraceContext,
}

impl From<HarRecord> for NetworkRequest {
    fn from(payload: HarRecord) -> Self {
        let request = HttpRequestSpec::from(&payload.request);

        Self {
            schema: NETWORK_REQUEST_SCHEMA.to_string(),
            identity: RequestIdentity::from_payload(&payload, &request),
            request,
            response: HttpResponseSpec::from(&payload.response),
            timing: payload.timing,
            trace: payload.trace,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestIdentity {
    pub entry_hash: String,
    pub dedupe_key: String,
}

impl RequestIdentity {
    pub fn from_payload<T: Serialize>(payload: &T, request: &HttpRequestSpec) -> Self {
        let entry_hash = stable_hash("hosho.har.payload.v1", payload);
        let dedupe_key = hash_parts(
            "hosho.har.dedupe.v1",
            &[&entry_hash, &request.method, &request.url],
        );

        Self {
            entry_hash,
            dedupe_key,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpRequestSpec {
    pub method: String,
    pub url: String,
    pub scheme: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub http_version: Option<String>,
    pub headers: HeaderMap,
    pub headers_size: Option<i64>,
    pub body_size: Option<i64>,
}

impl From<&HarRequest> for HttpRequestSpec {
    fn from(request: &HarRequest) -> Self {
        let url = request.url.as_deref().unwrap_or_default();
        let parts = UrlParts::from(url);

        Self {
            method: request.method.as_deref().unwrap_or("_OTHER").to_string(),
            url: url.to_string(),
            scheme: parts.scheme,
            host: parts.host,
            port: parts.port,
            http_version: request.http_version.clone(),
            headers: HeaderMap::from_headers(&request.headers),
            headers_size: non_negative_i64(request.headers_size),
            body_size: non_negative_i64(request.body_size),
        }
    }
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
}

impl From<&HarResponse> for HttpResponseSpec {
    fn from(response: &HarResponse) -> Self {
        Self {
            status: non_negative_i64(response.status),
            status_text: response.status_text.clone(),
            http_version: response.http_version.clone(),
            mime_type: response
                .content
                .as_ref()
                .and_then(|content| content.mime_type.clone()),
            headers: HeaderMap::from_headers(&response.headers),
            headers_size: non_negative_i64(response.headers_size),
            body_size: non_negative_i64(response.body_size),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceContext {
    pub traceparent: Option<String>,
    pub trigger: Option<TraceTrigger>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceTrigger {
    pub function_name: Option<String>,
    pub url: Option<String>,
    pub line_number: Option<i64>,
    pub column_number: Option<i64>,
}

pub fn allowed_header_attributes(prefix: &str, headers: &HeaderMap) -> Vec<(String, Vec<String>)> {
    headers
        .iter()
        .filter(|(name, _)| ALLOWED_HEADERS.contains(&name.as_str()))
        .map(|(name, values)| (format!("{prefix}.{name}"), values.clone()))
        .collect()
}

pub fn hash_parts(prefix: &str, parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prefix.as_bytes());
    for part in parts {
        hasher.update([0]);
        hasher.update(part.as_bytes());
    }
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

fn stable_hash<T: Serialize>(prefix: &str, value: &T) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prefix.as_bytes());
    hasher.update([0]);
    hasher.update(serde_json::to_vec(value).unwrap_or_default());
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

trait HeaderMapExt {
    fn from_headers(items: &[HarHeader]) -> HeaderMap;
}

impl HeaderMapExt for HeaderMap {
    fn from_headers(items: &[HarHeader]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for item in items {
            insert_header(&mut headers, &item.name, &item.value);
        }
        headers
    }
}

struct UrlParts {
    scheme: Option<String>,
    host: Option<String>,
    port: Option<u16>,
}

impl From<&str> for UrlParts {
    fn from(raw: &str) -> Self {
        let Ok(url) = Url::parse(raw) else {
            return Self::default();
        };

        Self {
            scheme: Some(url.scheme().to_string()),
            host: url.host_str().map(str::to_string),
            port: url.port_or_known_default(),
        }
    }
}

impl Default for UrlParts {
    fn default() -> Self {
        Self {
            scheme: None,
            host: None,
            port: None,
        }
    }
}

fn insert_header(headers: &mut HeaderMap, name: &str, value: &str) {
    let name = name.to_ascii_lowercase();
    if !name.is_empty() && !SENSITIVE_HEADERS.contains(&name.as_str()) {
        headers.entry(name).or_default().push(value.to_string());
    }
}

fn non_negative_i64(value: Option<i64>) -> Option<i64> {
    value.filter(|number| *number >= 0)
}
