// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! OpenTelemetry implementation of typespec_client_core tracing traits.

use crate::attributes::{AttributeValue, KeyValue};
use azure_core::Error;
use opentelemetry::{
    global,
    trace::{Span as OtelSpan, SpanKind, Status, Tracer as OtelTracer},
    Context as OtelContext,
};
use opentelemetry_sdk as sdk;
use std::error::Error as StdError;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use typespec_client_core::tracing::{Span, SpanStatus, Tracer, TracerProvider, WithSpan};
/// OpenTelemetry implementation of TracerProvider
pub struct OpenTelemetryTracerProvider {
    inner: Arc<sdk::trace::TracerProvider>,
}

impl OpenTelemetryTracerProvider {
    /// Create a new OpenTelemetry tracer provider
    pub fn new() -> Self {
        Self {
            inner: Arc::new(global::tracer_provider()),
        }
    }
    /// Create a new OpenTelemetry tracer provider with a custom provider
    pub fn with_provider(provider: Arc<sdk::trace::TracerProvider>) -> Self {
        Self { inner: provider }
    }
}

impl Default for OpenTelemetryTracerProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TracerProvider for OpenTelemetryTracerProvider {
    fn get_tracer(&self, name: &str) -> Box<dyn Tracer + Send + Sync> {
        let tracer = self.inner.tracer(name);
        Box::new(OpenTelemetryTracer {
            inner: tracer,
            active_span: Arc::new(Mutex::new(None)),
        })
    }
}

/// OpenTelemetry implementation of Tracer
pub struct OpenTelemetryTracer {
    inner: Box<dyn OtelTracer + Send + Sync>,
    active_span: Arc<Mutex<Option<Box<dyn Span + Send + Sync>>>>,
}

impl Tracer for OpenTelemetryTracer {
    fn start_span(&self, name: &str) -> Box<dyn Span + Send + Sync> {
        let span = self.inner.start(name);
        Box::new(OpenTelemetrySpan {
            inner: Box::new(span),
        })
    }

    fn set_active_span(&self, span: Box<dyn Span + Send + Sync>) {
        let mut active = self.active_span.lock().unwrap();
        *active = Some(span);
    }
}

/// OpenTelemetry implementation of Span
pub struct OpenTelemetrySpan {
    inner: Box<dyn OtelSpan + Send + Sync>,
}

impl Span for OpenTelemetrySpan {
    fn end(&self) {
        self.inner.end();
    }

    fn add_event(&self, name: &str, attributes: Option<Vec<KeyValue>>) {
        if let Some(attrs) = attributes {
            let otel_attrs: Vec<opentelemetry::KeyValue> =
                attrs.into_iter().map(|kv| kv.into()).collect();
            self.inner.add_event(name, otel_attrs);
        } else {
            self.inner.add_event(name, vec![]);
        }
    }

    fn set_attribute(&self, key: &str, value: AttributeValue) {
        let otel_value = opentelemetry::Value::from(value);
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
    /// Record an Azure Core error on the span with additional Azure-specific context
    pub fn record_azure_error(&self, error: &Error) {
        self.inner.record_exception(error);

        // Add Azure-specific error attributes
        self.set_attribute(
            "error.type",
            crate::attributes::attribute_value::string(format!("{:?}", error.kind())),
        );

        if let Some(source) = error.source() {
            self.set_attribute(
                "error.source",
                crate::attributes::attribute_value::string(source.to_string()),
            );
        }

        self.set_status(SpanStatus::Error {
            description: error.to_string(),
        });
    }

    /// Set Azure service name attribute
    pub fn set_azure_service(&self, service_name: &str) {
        self.set_attribute(
            crate::conventions::AZURE_SERVICE_NAME,
            crate::attributes::attribute_value::string(service_name),
        );
    }

    /// Set Azure operation name attribute
    pub fn set_azure_operation(&self, operation_name: &str) {
        self.set_attribute(
            crate::conventions::AZURE_OPERATION_NAME,
            crate::attributes::attribute_value::string(operation_name),
        );
    }

    /// Set Azure request ID attribute
    pub fn set_azure_request_id(&self, request_id: &str) {
        self.set_attribute(
            crate::conventions::AZURE_REQUEST_ID,
            crate::attributes::attribute_value::string(request_id),
        );
    }

    /// Get the underlying OpenTelemetry span
    pub fn inner(&self) -> &dyn OtelSpan {
        self.inner.as_ref()
    }
}

/// A wrapper that implements WithSpan for futures
pub struct SpanFuture<F> {
    future: F,
    span: Box<dyn Span + Send + Sync>,
}

impl<F> Future for SpanFuture<F>
where
    F: Future + Send,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Safety: We never move the future
        let this = unsafe { self.get_unchecked_mut() };
        let future = unsafe { Pin::new_unchecked(&mut this.future) };

        // Set the span as active for the duration of this poll
        let _guard = this.span.inner.as_ref().span_context();

        future.poll(cx)
    }
}

impl<F> WithSpan for F
where
    F: Future + Send + Sync + 'static,
{
    fn with_span(
        self,
        span: Box<dyn Span + Send + Sync>,
    ) -> Box<dyn Future<Output = Self> + Send + Sync> {
        Box::new(SpanFuture { future: self, span })
    }
}

/// Builder for creating Azure-specific spans with OpenTelemetry integration
pub struct AzureSpanBuilder {
    name: String,
    kind: SpanKind,
    attributes: Vec<KeyValue>,
    tracer: Box<dyn OtelTracer + Send + Sync>,
}

impl AzureSpanBuilder {
    /// Create a new span builder
    pub fn new(name: impl Into<String>) -> Self {
        let tracer = global::tracer("azure-sdk-rust");
        Self {
            name: name.into(),
            kind: SpanKind::Internal,
            attributes: Vec::new(),
            tracer: Box::new(tracer),
        }
    }

    /// Set the span kind
    pub fn with_kind(mut self, kind: SpanKind) -> Self {
        self.kind = kind;
        self
    }

    /// Add an attribute
    pub fn with_attribute(mut self, key: impl Into<String>, value: AttributeValue) -> Self {
        self.attributes.push(KeyValue {
            key: key.into(),
            value,
        });
        self
    }

    /// Set Azure service name
    pub fn with_azure_service(self, service_name: impl Into<String>) -> Self {
        self.with_attribute(
            crate::conventions::AZURE_SERVICE_NAME,
            crate::attributes::attribute_value::string(service_name.into()),
        )
    }

    /// Set Azure operation name
    pub fn with_azure_operation(self, operation_name: impl Into<String>) -> Self {
        self.with_attribute(
            crate::conventions::AZURE_OPERATION_NAME,
            crate::attributes::attribute_value::string(operation_name.into()),
        )
    }

    /// Set Azure request ID
    pub fn with_request_id(self, request_id: impl Into<String>) -> Self {
        self.with_attribute(
            crate::conventions::AZURE_REQUEST_ID,
            crate::attributes::attribute_value::string(request_id.into()),
        )
    }

    /// Build the span
    pub fn build(self) -> OpenTelemetrySpan {
        let mut span_builder = self.tracer.span_builder(&self.name).with_kind(self.kind);

        for attr in self.attributes {
            let otel_attr: opentelemetry::KeyValue = attr.into();
            span_builder = span_builder.with_attributes(vec![otel_attr]);
        }

        let span = span_builder.start(&*self.tracer);
        OpenTelemetrySpan {
            inner: Box::new(span),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracer_provider_creates_tracer() {
        let provider = OpenTelemetryTracerProvider::new();
        let _tracer = provider.get_tracer("test");
    }

    #[test]
    fn tracer_creates_span() {
        let provider = OpenTelemetryTracerProvider::new();
        let tracer = provider.get_tracer("test");
        let _span = tracer.start_span("test-span");
    }

    #[test]
    fn span_builder_works() {
        let span = AzureSpanBuilder::new("test-span")
            .with_kind(SpanKind::Client)
            .with_azure_service("storage")
            .with_azure_operation("list_blobs")
            .with_request_id("req-123")
            .build();

        span.end();
    }

    #[test]
    fn span_records_attributes_and_events() {
        let span = AzureSpanBuilder::new("test-span").build();

        span.set_attribute(
            "test.key",
            crate::attributes::attribute_value::string("test.value"),
        );
        span.add_event(
            "test-event",
            Some(vec![KeyValue {
                key: "event.key".to_string(),
                value: crate::attributes::attribute_value::string("event.value"),
            }]),
        );

        span.end();
    }

    #[test]
    fn span_records_errors() {
        let span = AzureSpanBuilder::new("error-span").build();
        let error = std::io::Error::new(std::io::ErrorKind::NotFound, "Test error");

        span.record_error(&error);
        span.end();
    }

    #[test]
    fn span_records_azure_errors() {
        let span = AzureSpanBuilder::new("azure-error-span").build();
        let error = Error::message(azure_core::error::ErrorKind::Other, "Azure test error");

        span.record_azure_error(&error);
        span.end();
    }
}
