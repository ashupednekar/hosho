use crate::prelude::HttpResult;
use axum::Json;
use serde_json::json;

#[worker::send]
pub async fn livez() -> HttpResult<Json<serde_json::Value>> {
    Ok(Json(json!({"status": "ok"})))
}
