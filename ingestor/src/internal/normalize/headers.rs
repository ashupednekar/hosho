use serde_json::Value;

use crate::internal::specs::network::HeaderMap;

const ALLOWED_HEADERS: &[&str] = &["accept", "content-type", "traceparent"];

pub fn har_headers(value: Option<&Value>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let Some(items) = value.and_then(Value::as_array) else {
        return headers;
    };

    for item in items {
        if let (Some(name), Some(header_value)) = (item.get("name"), item.get("value")) {
            insert_header(&mut headers, name, header_value);
        }
    }

    headers
}

pub fn allowed_header_attributes(prefix: &str, headers: &HeaderMap) -> Vec<(String, Vec<String>)> {
    headers
        .iter()
        .filter(|(name, _)| ALLOWED_HEADERS.contains(&name.as_str()))
        .map(|(name, values)| (format!("{prefix}.{name}"), values.clone()))
        .collect()
}

fn insert_header(headers: &mut HeaderMap, name: &Value, value: &Value) {
    let Some(name) = name.as_str().map(|header| header.to_ascii_lowercase()) else {
        return;
    };

    if !is_sensitive(&name) {
        headers
            .entry(name)
            .or_default()
            .push(value.as_str().unwrap_or_default().to_string());
    }
}

fn is_sensitive(name: &str) -> bool {
    matches!(
        name,
        "authorization" | "cookie" | "proxy-authorization" | "set-cookie"
    )
}
