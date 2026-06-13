use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;

use super::config::{SERVICE_NAME, SERVICE_NAMESPACE};

pub fn resource_attributes(instance_id: Option<&str>) -> Vec<KeyValue> {
    let mut attrs = vec![
        KeyValue::new("service.name", SERVICE_NAME),
        KeyValue::new("service.namespace", SERVICE_NAMESPACE),
        KeyValue::new("telemetry.sdk.name", "hosho"),
        KeyValue::new("telemetry.sdk.language", "webjs"),
    ];

    if let Some(instance_id) = instance_id {
        attrs.push(KeyValue::new(
            "service.instance.id",
            instance_id.to_string(),
        ));
    }

    Resource::builder_empty()
        .with_attributes(attrs)
        .build()
        .iter()
        .map(|(key, value)| KeyValue::new(key.to_string(), value.clone()))
        .collect()
}
