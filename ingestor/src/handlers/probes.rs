use axum::Json;
use serde_json::json;

#[worker::send]
pub async fn livez() -> Json<serde_json::Value> {
    Json(json!({"status": "ok"}))
}
