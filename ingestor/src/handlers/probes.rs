use axum::Json;
use serde_json::json;

use crate::prelude::Result;

#[worker::send]
pub async fn livez() -> Result<Json<serde_json::Value>> {
    Ok(Json(json!({"status": "ok"})))
}
