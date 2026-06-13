use crate::prelude::Result;
use axum::Json;
use serde_json::json;

#[worker::send]
pub async fn livez() -> Result<Json<serde_json::Value>> {
    Ok(Json(json!({"status": "ok"})))
}
