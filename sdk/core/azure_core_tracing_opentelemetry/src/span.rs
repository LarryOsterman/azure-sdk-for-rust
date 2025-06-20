// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! OpenTelemetry implementation of typespec_client_core tracing traits.

use crate::attributes::{
    AttributeValue as ConversionAttributeValue, KeyValue as ConversionKeyValue,
};
use azure_core::{
    tracing::{
        attributes::{AttributeValue, KeyValue},
        Span, SpanGuard, SpanStatus,
    },
    Error, Result,
};
use opentelemetry::{
    global::{BoxedSpan, ObjectSafeSpan},
    trace::TraceContextExt,
};
use std::{
    error::Error as StdError,
    sync::{Arc, Mutex},
    time::SystemTime,
};

struct OtelSpanState {
    span: BoxedSpan,
    context: opentelemetry::Context,
}

/// OpenTelemetry implementation of Span
pub(super) struct OpenTelemetrySpan {
    inner: Mutex<OtelSpanState>,
}

impl OpenTelemetrySpan {
    pub fn new(span: BoxedSpan, context: opentelemetry::Context) -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(OtelSpanState { span, context }),
        })
    }
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
        inner.span.end();
        Ok(())
    }

    fn add_event(&self, name: String, attributes: Option<Vec<KeyValue>>) -> Result<()> {
        let mut inner = self.inner.lock().map_err(|e| {
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
            inner
                .span
                .add_event_with_timestamp(name.into(), SystemTime::now(), otel_attrs);
        } else {
            inner
                .span
                .add_event_with_timestamp(name.into(), SystemTime::now(), vec![]);
        }
        Ok(())
    }

    fn set_attribute(&self, key: String, value: AttributeValue) -> Result<()> {
        let otel_value = opentelemetry::Value::from(ConversionAttributeValue(value));
        let mut inner = self.inner.lock().map_err(|e| {
            Error::message(
                azure_core::error::ErrorKind::Other,
                format!("Failed to lock span: {e:?}"),
            )
        })?;
        inner
            .span
            .set_attribute(opentelemetry::KeyValue::new(key, otel_value));
        Ok(())
    }

    fn record_error(&self, error: &dyn StdError) -> Result<()> {
        let mut inner = self.inner.lock().map_err(|e| {
            Error::message(
                azure_core::error::ErrorKind::Other,
                format!("Failed to lock span: {e:?}"),
            )
        })?;
        if inner.span.is_recording() {
            inner.span.add_event_with_timestamp(
                std::borrow::Cow::Borrowed("error"),
                SystemTime::now(),
                vec![opentelemetry::KeyValue::new(
                    "error.message",
                    opentelemetry::Value::String(error.to_string().into()),
                )],
            );
        }
        inner.span.set_status(opentelemetry::trace::Status::Error {
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
            .span
            .set_status(otel_status);
        Ok(())
    }

    fn set_current(
        &self,
        _context: &azure_core::http::Context,
    ) -> typespec_client_core::Result<Box<dyn SpanGuard>> {
        let inner = self.inner.lock().map_err(|e| {
            Error::message(
                azure_core::error::ErrorKind::Other,
                format!("Failed to lock span: {e:?}"),
            )
        })?;

        // Create a context with the current span
        let context_guard = inner.context.clone().attach();

        println!("Setting current OpenTelemetry span context");
        Ok(Box::new(OpenTelemetrySpanGuard {
            _inner: context_guard,
        }))
    }
}

struct OpenTelemetrySpanGuard {
    _inner: opentelemetry::ContextGuard,
}

impl SpanGuard for OpenTelemetrySpanGuard {
    fn end(self) -> Result<()> {
        // The span is ended when the guard is dropped, so no action needed here.
        Ok(())
    }
}

impl Drop for OpenTelemetrySpanGuard {
    fn drop(&mut self) {
        // The OpenTelemetry context guard will automatically end the span when dropped.
        println!(
            "Dropping OpenTelemetry span guard, ending span: {:?}",
            self._inner
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::telemetry::OpenTelemetryTracerProvider;
    use azure_core::http::Context as AzureContext;
    use azure_core::tracing::{attributes::AttributeValue, SpanStatus, TracerProvider};
    use opentelemetry::trace::TraceContextExt;
    use opentelemetry::{Context, Key, KeyValue, Value};
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

    #[tokio::test]
    async fn test_open_telemetry_span_futures() {
        let (otel_tracer_provider, otel_exporter) = create_exportable_tracer_provider();
        let tracer_provider = OpenTelemetryTracerProvider::new(otel_tracer_provider);
        assert!(tracer_provider.is_ok());
        let tracer = tracer_provider
            .unwrap()
            .get_tracer("test".to_string(), "0.1.0".to_string());

        let future = async {
            let context = Context::current();
            println!("In captured context: {:?}", context);
            context.span().add_event("name", vec![]);
            context.span().set_attribute(KeyValue::new(
                Key::from("test_key"),
                Value::from("test_value"),
            ));
            context.span().end();
            42
        };

        let span = tracer.start_span("test_span".to_string());

        let azure_context = AzureContext::new();
        let azure_context = azure_context.with_value(span.clone());

        let _guard = span.set_current(&azure_context).unwrap();

        let result = future.await;

        assert_eq!(result, 42);
        span.end().unwrap();

        let spans = otel_exporter.get_finished_spans().unwrap();
        assert_eq!(spans.len(), 1);
        for span in &spans {
            assert_eq!(span.name, "test_span");
            assert_eq!(span.events.len(), 1);
            assert_eq!(span.attributes.len(), 1);
            assert_eq!(span.attributes[0].key, "test_key".into());
            assert_eq!(
                format!("{:?}", span.attributes[0].value),
                "String(Owned(\"test_value\"))"
            );
        }
    }
}
