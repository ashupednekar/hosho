use serde_json::Value;

use crate::internal::specs::timing::{BrowserContext, InitiatorContext};

pub fn browser_context(entry: &Value) -> BrowserContext {
    BrowserContext {
        resource_type: entry
            .get("_resourceType")
            .and_then(Value::as_str)
            .map(str::to_string),
        priority: entry
            .get("_priority")
            .and_then(Value::as_str)
            .map(str::to_string),
        connection_id: entry
            .get("_connectionId")
            .and_then(Value::as_str)
            .map(str::to_string),
        initiator: initiator_context(entry.get("_initiator")),
    }
}

fn initiator_context(value: Option<&Value>) -> Option<InitiatorContext> {
    let initiator = value?;
    let frame = top_frame(initiator);

    Some(InitiatorContext {
        top_function: frame
            .and_then(|v| v.get("functionName"))
            .and_then(Value::as_str)
            .map(str::to_string),
        top_url: frame
            .and_then(|v| v.get("url"))
            .and_then(Value::as_str)
            .map(str::to_string),
        top_line: frame
            .and_then(|v| v.get("lineNumber"))
            .and_then(Value::as_i64),
        top_column: frame
            .and_then(|v| v.get("columnNumber"))
            .and_then(Value::as_i64),
        stack_depth: initiator
            .pointer("/stack/callFrames")
            .and_then(Value::as_array)
            .map(Vec::len),
    })
}

fn top_frame(initiator: &Value) -> Option<&Value> {
    initiator.pointer("/stack/callFrames/0")
}
