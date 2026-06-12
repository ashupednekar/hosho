use serde_json::Value;

use crate::internal::{
    error::{IngestError, IngestResult},
    specs::console::{
        ConsoleBatchEnvelope, ConsoleCapture, ConsoleEventEnvelope, ConsoleRecord,
        CONSOLE_BATCH_SCHEMA, CONSOLE_EVENT_SCHEMA,
    },
};

pub fn normalize_console_payload(payload: Value) -> IngestResult<Vec<ConsoleRecord>> {
    match payload.get("schema").and_then(Value::as_str) {
        Some(CONSOLE_EVENT_SCHEMA) => event_envelope(payload),
        Some(CONSOLE_BATCH_SCHEMA) => batch_envelope(payload),
        _ if payload.get("level").is_some() => {
            Ok(vec![record_from_value(ConsoleCapture::default(), payload)])
        }
        _ => Err(IngestError::InvalidPayload("unsupported console payload")),
    }
}

fn event_envelope(payload: Value) -> IngestResult<Vec<ConsoleRecord>> {
    let envelope: ConsoleEventEnvelope = serde_json::from_value(payload)
        .map_err(|_| IngestError::InvalidPayload("invalid console event envelope"))?;
    Ok(vec![record_from_value(envelope.capture, envelope.event)])
}

fn batch_envelope(payload: Value) -> IngestResult<Vec<ConsoleRecord>> {
    let envelope: ConsoleBatchEnvelope = serde_json::from_value(payload)
        .map_err(|_| IngestError::InvalidPayload("invalid console batch envelope"))?;
    Ok(envelope
        .events
        .into_iter()
        .map(|event| record_from_value(envelope.capture.clone(), event))
        .collect())
}

fn record_from_value(capture: ConsoleCapture, event: Value) -> ConsoleRecord {
    let args = event
        .get("args")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    ConsoleRecord {
        level: event
            .get("level")
            .and_then(Value::as_str)
            .unwrap_or("log")
            .to_string(),
        body: console_body(&args),
        args,
        url: event.get("url").and_then(Value::as_str).map(str::to_string),
        tab_id: event
            .get("tabId")
            .and_then(Value::as_i64)
            .or(capture.tab_id),
        captured_at: event
            .get("capturedAt")
            .and_then(Value::as_str)
            .map(str::to_string),
        capture,
    }
}

fn console_body(args: &[Value]) -> String {
    args.iter()
        .map(|arg| {
            arg.as_str()
                .map(str::to_string)
                .unwrap_or_else(|| arg.to_string())
        })
        .collect::<Vec<_>>()
        .join(" ")
}
