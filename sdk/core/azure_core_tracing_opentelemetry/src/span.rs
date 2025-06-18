// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! OpenTelemetry implementation of typespec_client_core tracing traits.

use crate::attributes::AttributeValue as ConversionAttributeValue;
use azure_core::{
    tracing::{
        attributes::{AttributeArray, AttributeValue, KeyValue},
        Span, SpanStatus,
    },
    Result,
};
use opentelemetry::global::ObjectSafeSpan;
use std::error::Error as StdError;
use std::sync::Mutex;

/// OpenTelemetry implementation of Span
pub struct OpenTelemetrySpan {
    inner: Mutex<Box<dyn ObjectSafeSpan + Send + Sync>>,
}

impl Span for OpenTelemetrySpan {
    fn end(&self) {
        // Lock the inner span to ensure thread safety
        let mut inner = self.inner.lock().expect("Failed to lock span");
        inner.end();
    }

    fn add_event(&self, name: &str, attributes: Option<Vec<KeyValue>>) -> Result<()> {
        if let Some(attrs) = attributes {
            let otel_attrs: Vec<opentelemetry::KeyValue> =
                attrs.into_iter().map(|kv| kv.into()).collect();
            self.inner.lock()?.as_mut().add_event(name, otel_attrs);
        } else {
            self.inner.lock()?.as_mut().add_event(name, vec![]);
        }
        Ok(())
    }

    fn set_attribute(&self, key: &str, value: AttributeValue) {
        let otel_value = opentelemetry::Value::from(ConversionAttributeValue(value));
        self.inner
            .set_attribute(opentelemetry::KeyValue::new(key, otel_value));
    }

    fn record_error(&self, error: &dyn StdError) {
        self.inner.record_exception(error);
        self.set_status(SpanStatus::Error {
            description: error.to_string(),
        });
    }

    fn set_status(&self, status: SpanStatus) {
        let otel_status = match status {
            SpanStatus::Unset => Status::Unset,
            SpanStatus::Ok => Status::Ok,
            SpanStatus::Error { description } => Status::error(description),
        };
        self.inner.set_status(otel_status);
    }
}

impl OpenTelemetrySpan {
    pub fn new(inner: Box<dyn ObjectSafeSpan + Send + Sync>) -> Self {
        Self {
            inner: Mutex::new(inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
