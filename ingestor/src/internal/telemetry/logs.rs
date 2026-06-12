use serde_json::{json, Value};

use crate::internal::{
    normalize::time::unix_nano,
    specs::{
        console::ConsoleRecord,
        telemetry::{TelemetryAttribute, TelemetryValue},
    },
};

use super::{attributes::otlp_attributes, config::SCOPE_VERSION, resource::resource_attributes};

pub fn console_logs(records: &[ConsoleRecord]) -> Option<Value> {
    (!records.is_empty()).then(|| {
        json!({"resourceLogs": [{
            "resource": {"attributes": otlp_attributes(&resource_attributes(records.first().and_then(|r| r.capture.session_id.as_deref())))},
            "scopeLogs": [{"scope": {"name": "hosho.console", "version": SCOPE_VERSION}, "logRecords": log_records(records)}]
        }]})
    })
}

fn log_records(records: &[ConsoleRecord]) -> Vec<Value> {
    records.iter().map(log_record).collect()
}

fn log_record(record: &ConsoleRecord) -> Value {
    json!({
        "timeUnixNano": unix_nano(record.captured_at.as_deref()).unwrap_or_default().to_string(),
        "severityText": record.level.to_ascii_uppercase(),
        "severityNumber": severity_number(&record.level),
        "body": {"stringValue": record.body},
        "attributes": otlp_attributes(&log_attrs(record)),
    })
}

fn log_attrs(record: &ConsoleRecord) -> Vec<TelemetryAttribute> {
    [
        Some(TelemetryAttribute::new(
            "hosho.schema",
            TelemetryValue::String("hosho.console.record.v1".to_string()),
        )),
        record
            .url
            .as_ref()
            .map(|url| TelemetryAttribute::new("url.full", TelemetryValue::String(url.clone()))),
        record.tab_id.map(|tab_id| {
            TelemetryAttribute::new("hosho.browser.tab_id", TelemetryValue::Int(tab_id))
        }),
        record.capture.page_id.as_ref().map(|page_id| {
            TelemetryAttribute::new(
                "hosho.capture.page_id",
                TelemetryValue::String(page_id.clone()),
            )
        }),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn severity_number(level: &str) -> i64 {
    match level {
        "debug" => 5,
        "warn" => 13,
        "error" => 17,
        _ => 9,
    }
}
