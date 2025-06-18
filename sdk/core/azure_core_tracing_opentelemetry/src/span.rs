// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! OpenTelemetry implementation of typespec_client_core tracing traits.

use crate::attributes::{
    AttributeValue as ConversionAttributeValue, KeyValue as ConversionKeyValue,
};
use azure_core::{
    tracing::{
        attributes::{AttributeValue, KeyValue},
        Span, SpanStatus,
    },
    Error, Result,
};
use opentelemetry::global::ObjectSafeSpan;
use std::{error::Error as StdError, sync::Mutex, time::SystemTime};

/// OpenTelemetry implementation of Span
pub struct OpenTelemetrySpan {
    inner: Mutex<Box<dyn ObjectSafeSpan + Send + Sync>>,
}

impl Span for OpenTelemetrySpan {
    fn end(&self) -> Result<()> {
        // Lock the inner span to ensure thread safety
        let mut inner = self.inner.lock().map_err(|e| {
            Error::message(
                azure_core::error::ErrorKind::Other,
                format!("Failed to lock span: {e:?}"),
            )
        })?;
        inner.end();
        Ok(())
    }

    fn add_event(&self, name: String, attributes: Option<Vec<KeyValue>>) -> Result<()> {
        let mut span = self.inner.lock().map_err(|e| {
            Error::message(
                azure_core::error::ErrorKind::Other,
                format!("Failed to lock span: {e:?}"),
            )
        })?;
        if let Some(attrs) = attributes {
            let otel_attrs: Vec<opentelemetry::KeyValue> = attrs
                .into_iter()
                .map(|kv| ConversionKeyValue(kv).into())
                .collect();
            span.add_event_with_timestamp(name.into(), SystemTime::now(), otel_attrs);
        } else {
            span.add_event_with_timestamp(name.into(), SystemTime::now(), vec![]);
        }
        Ok(())
    }

    fn set_attribute(&self, key: String, value: AttributeValue) -> Result<()> {
        let otel_value = opentelemetry::Value::from(ConversionAttributeValue(value));
        let mut span = self.inner.lock().map_err(|e| {
            Error::message(
                azure_core::error::ErrorKind::Other,
                format!("Failed to lock span: {e:?}"),
            )
        })?;
        span.set_attribute(opentelemetry::KeyValue::new(key, otel_value));
        Ok(())
    }

    fn record_error(&self, error: &dyn StdError) -> Result<()> {
        let mut span = self.inner.lock().map_err(|e| {
            Error::message(
                azure_core::error::ErrorKind::Other,
                format!("Failed to lock span: {e:?}"),
            )
        })?;
        if span.is_recording() {
            span.add_event_with_timestamp(
                std::borrow::Cow::Borrowed("error"),
                SystemTime::now(),
                vec![opentelemetry::KeyValue::new(
                    "error.message",
                    opentelemetry::Value::String(error.to_string().into()),
                )],
            );
        }
        span.set_status(opentelemetry::trace::Status::Error {
            description: error.to_string().into(),
        });
        Ok(())
    }

    fn set_status(&self, status: SpanStatus) -> Result<()> {
        let otel_status = match status {
            SpanStatus::Unset => opentelemetry::trace::Status::Unset,
            SpanStatus::Ok => opentelemetry::trace::Status::Ok,
            SpanStatus::Error { description } => opentelemetry::trace::Status::error(description),
        };
        self.inner
            .lock()
            .map_err(|e| {
                Error::message(
                    azure_core::error::ErrorKind::Other,
                    format!("Failed to lock span: {e:?}"),
                )
            })?
            .as_mut()
            .set_status(otel_status);
        Ok(())
    }
}

impl OpenTelemetrySpan {
    pub fn new(inner: Box<dyn ObjectSafeSpan + Send + Sync>) -> Box<Self> {
        Box::new(Self {
            inner: Mutex::new(inner),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::telemetry::OpenTelemetryTracerProvider;
    use azure_core::tracing::attributes::AttributeValue;
    use azure_core::tracing::SpanStatus;
    use azure_core::tracing::TracerProvider;
    use opentelemetry_sdk::trace::{in_memory_exporter::InMemorySpanExporter, SdkTracerProvider};
    use std::io::{Error, ErrorKind};
    use std::sync::Arc;

    fn create_exportable_tracer_provider() -> (Arc<SdkTracerProvider>, InMemorySpanExporter) {
        let otel_exporter = InMemorySpanExporter::default();
        let otel_tracer_provider = SdkTracerProvider::builder()
            .with_simple_exporter(otel_exporter.clone())
            .build();
        let otel_tracer_provider = Arc::new(otel_tracer_provider);
        (otel_tracer_provider, otel_exporter)
    }

    #[test]
    fn test_open_telemetry_span_new() {
        let (otel_tracer_provider, otel_exporter) = create_exportable_tracer_provider();

        let tracer_provider = OpenTelemetryTracerProvider::new(otel_tracer_provider);
        assert!(tracer_provider.is_ok());
        let tracer = tracer_provider
            .unwrap()
            .get_tracer("test".to_string(), "0.1.0".to_string());
        let span = tracer.start_span("test_span".to_string());
        assert!(span.end().is_ok());

        let spans = otel_exporter.get_finished_spans().unwrap();
        assert_eq!(spans.len(), 1);
        for span in &spans {
            println!("Span: {:?}", span);
            assert_eq!(span.name, "test_span");
            assert_eq!(span.status, opentelemetry::trace::Status::Unset);
            assert!(span.attributes.is_empty());
        }
    }

    #[test]
    fn test_open_telemetry_span_add_event() {
        let (otel_tracer_provider, otel_exporter) = create_exportable_tracer_provider();
        let tracer_provider = OpenTelemetryTracerProvider::new(otel_tracer_provider);
        assert!(tracer_provider.is_ok());
        let tracer = tracer_provider
            .unwrap()
            .get_tracer("test".to_string(), "0.1.0".to_string());
        let span = tracer.start_span("test_span".to_string());
        assert!(span.add_event("test_event".to_string(), None).is_ok());
        assert!(span.end().is_ok());

        let spans = otel_exporter.get_finished_spans().unwrap();
        assert_eq!(spans.len(), 1);
        for span in &spans {
            println!("Span: {:?}", span);
            assert_eq!(span.name, "test_span");
            assert_eq!(span.status, opentelemetry::trace::Status::Unset);
            assert_eq!(span.events.len(), 1);
            assert_eq!(span.events[0].name, "test_event");
            assert!(span.events[0].attributes.is_empty());
        }
    }

    #[test]
    fn test_open_telemetry_span_set_attribute() {
        let (otel_tracer_provider, otel_exporter) = create_exportable_tracer_provider();
        let tracer_provider = OpenTelemetryTracerProvider::new(otel_tracer_provider);
        assert!(tracer_provider.is_ok());
        let tracer = tracer_provider
            .unwrap()
            .get_tracer("test".to_string(), "0.1.0".to_string());
        let span = tracer.start_span("test_span".to_string());

        assert!(span
            .set_attribute(
                "test_key".to_string(),
                AttributeValue::String("test_value".to_string())
            )
            .is_ok());
        assert!(span.end().is_ok());

        let spans = otel_exporter.get_finished_spans().unwrap();
        assert_eq!(spans.len(), 1);
        for span in &spans {
            assert_eq!(span.name, "test_span");
            assert_eq!(span.attributes.len(), 1);
            assert_eq!(span.attributes[0].key, "test_key".into());
            assert_eq!(
                format!("{:?}", span.attributes[0].value),
                "String(Owned(\"test_value\"))"
            );
        }
    }

    #[test]
    fn test_open_telemetry_span_record_error() {
        let (otel_tracer_provider, otel_exporter) = create_exportable_tracer_provider();
        let tracer_provider = OpenTelemetryTracerProvider::new(otel_tracer_provider);
        assert!(tracer_provider.is_ok());
        let tracer = tracer_provider
            .unwrap()
            .get_tracer("test".to_string(), "0.1.0".to_string());
        let span = tracer.start_span("test_span".to_string());

        let error = Error::new(ErrorKind::NotFound, "resource not found");
        assert!(span.record_error(&error).is_ok());
        assert!(span.end().is_ok());

        let spans = otel_exporter.get_finished_spans().unwrap();
        assert_eq!(spans.len(), 1);
        for span in &spans {
            assert_eq!(span.name, "test_span");
            assert_eq!(
                span.status,
                opentelemetry::trace::Status::error("resource not found")
            );
            assert_eq!(span.events.len(), 1);
            assert_eq!(span.events[0].name, "error");
            assert_eq!(span.events[0].attributes.len(), 1);
            assert_eq!(span.events[0].attributes[0].key, "error.message".into());
        }
    }

    #[test]
    fn test_open_telemetry_span_set_status() {
        let (otel_tracer_provider, otel_exporter) = create_exportable_tracer_provider();
        let tracer_provider = OpenTelemetryTracerProvider::new(otel_tracer_provider);
        assert!(tracer_provider.is_ok());
        let tracer = tracer_provider
            .unwrap()
            .get_tracer("test".to_string(), "0.1.0".to_string());

        // Test Ok status
        let span = tracer.start_span("test_span_ok".to_string());
        assert!(span.set_status(SpanStatus::Ok).is_ok());
        assert!(span.end().is_ok());

        // Test Error status
        let span = tracer.start_span("test_span_error".to_string());
        assert!(span
            .set_status(SpanStatus::Error {
                description: "test error".to_string()
            })
            .is_ok());
        assert!(span.end().is_ok());

        let spans = otel_exporter.get_finished_spans().unwrap();
        assert_eq!(spans.len(), 2);

        let ok_span = spans.iter().find(|s| s.name == "test_span_ok").unwrap();
        assert_eq!(ok_span.status, opentelemetry::trace::Status::Ok);

        let error_span = spans.iter().find(|s| s.name == "test_span_error").unwrap();
        assert_eq!(
            error_span.status,
            opentelemetry::trace::Status::error("test error")
        );
    }
}
