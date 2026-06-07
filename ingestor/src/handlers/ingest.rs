use axum::Json;
use serde_json::Value;
use worker::console_log;

#[worker::send]
pub async fn ingest_har(har: Json<Value>) -> String {
    console_log!("har: {:?}", &har);
    "".into()
}

#[worker::send]
pub async fn ingest_console() -> String {
    "".into()
}
