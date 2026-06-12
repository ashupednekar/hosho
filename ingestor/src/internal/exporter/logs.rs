use std::collections::HashMap;

use opentelemetry::logs::{
    AnyValue as LogValue, LogRecord as _, Logger as _, LoggerProvider as _, Severity,
};
use opentelemetry::Key;
use serde::{Deserialize, Serialize};

use crate::internal::exporter::otel::logger_provider;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleLogPayload {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tab_id: Option<i64>,
    pub level: String,
    #[serde(default)]
    pub args: Vec<ConsoleArg>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub captured_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ConsoleArg {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<ConsoleArg>),
    Object(HashMap<String, ConsoleArg>),
}

pub async fn send_log(log: &ConsoleLogPayload) -> worker::Result<()> {
    let provider = logger_provider()?;
    let logger = provider.logger("hosho.console");
    let mut record = logger.create_log_record();
    let severity = Severity::from(log);

    record.set_event_name("console");
    record.set_target("browser.extension");
    record.set_severity_number(severity);
    record.set_severity_text(severity.name());
    record.set_body(LogValue::from(log));
    record.add_attributes(log_attributes(log));

    logger.emit(record);
    provider.force_flush().map_err(export_error)
}

impl From<&ConsoleLogPayload> for Severity {
    fn from(log: &ConsoleLogPayload) -> Self {
        match log.level.as_str() {
            "debug" => Severity::Debug,
            "warn" => Severity::Warn,
            "error" => Severity::Error,
            "info" | "log" => Severity::Info,
            _ => Severity::Info,
        }
    }
}

impl From<&ConsoleLogPayload> for LogValue {
    fn from(log: &ConsoleLogPayload) -> Self {
        let mut fields = HashMap::from([
            (
                Key::from_static_str("level"),
                LogValue::from(level_name(log)),
            ),
            (
                Key::from_static_str("args"),
                LogValue::ListAny(Box::new(log.args.iter().map(LogValue::from).collect())),
            ),
        ]);

        if let Some(tab_id) = log.tab_id {
            fields.insert(Key::from_static_str("tabId"), LogValue::Int(tab_id));
        }

        if let Some(url) = &log.url {
            fields.insert(
                Key::from_static_str("url"),
                LogValue::String(url.clone().into()),
            );
        }

        if let Some(captured_at) = &log.captured_at {
            fields.insert(
                Key::from_static_str("capturedAt"),
                LogValue::String(captured_at.clone().into()),
            );
        }

        LogValue::Map(Box::new(fields))
    }
}

impl From<&ConsoleArg> for LogValue {
    fn from(arg: &ConsoleArg) -> Self {
        match arg {
            ConsoleArg::Null => LogValue::String("null".into()),
            ConsoleArg::Bool(value) => LogValue::Boolean(*value),
            ConsoleArg::Number(value) => LogValue::Double(*value),
            ConsoleArg::String(value) => LogValue::String(value.clone().into()),
            ConsoleArg::Array(values) => {
                LogValue::ListAny(Box::new(values.iter().map(LogValue::from).collect()))
            }
            ConsoleArg::Object(values) => LogValue::Map(Box::new(
                values
                    .iter()
                    .map(|(key, value)| (Key::new(key.clone()), LogValue::from(value)))
                    .collect(),
            )),
        }
    }
}

fn log_attributes(log: &ConsoleLogPayload) -> Vec<(Key, LogValue)> {
    let mut attributes = vec![
        (
            Key::from_static_str("hosho.signal"),
            LogValue::from("console"),
        ),
        (
            Key::from_static_str("log.level"),
            LogValue::from(level_name(log)),
        ),
    ];

    if let Some(tab_id) = log.tab_id {
        attributes.push((
            Key::from_static_str("browser.tab.id"),
            LogValue::Int(tab_id),
        ));
    }

    if let Some(url) = &log.url {
        attributes.push((
            Key::from_static_str("url.full"),
            LogValue::String(url.clone().into()),
        ));
    }

    attributes
}

fn level_name(log: &ConsoleLogPayload) -> &'static str {
    match log.level.as_str() {
        "debug" => "debug",
        "warn" => "warn",
        "error" => "error",
        "info" => "info",
        "log" => "log",
        _ => "info",
    }
}

fn export_error(error: impl std::fmt::Display) -> worker::Error {
    worker::Error::RustError(format!("otel log export failed: {error}"))
}
