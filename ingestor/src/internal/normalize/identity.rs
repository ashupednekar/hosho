use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::internal::specs::network::{HttpRequestSpec, RequestIdentity};

pub fn identity_for_entry(entry: &Value, request: &HttpRequestSpec) -> RequestIdentity {
    let entry_hash = stable_hash("hosho.har.entry.v1", entry);
    let request_id = entry
        .get("_requestId")
        .and_then(Value::as_str)
        .map(str::to_string);
    let dedupe_key = hash_parts(
        "hosho.har.dedupe.v1",
        &[&entry_hash, &request.method, &request.url],
    );

    RequestIdentity {
        request_id,
        entry_hash,
        dedupe_key,
    }
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

fn stable_hash(prefix: &str, value: &Value) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prefix.as_bytes());
    hasher.update([0]);
    hasher.update(serde_json::to_vec(value).unwrap_or_default());
    format!("sha256:{}", hex::encode(hasher.finalize()))
}
