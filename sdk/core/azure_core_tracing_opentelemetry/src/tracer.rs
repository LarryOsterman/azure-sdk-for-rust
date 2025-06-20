// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

use crate::span::{OpenTelemetrySpan, OpenTelemetrySpanKind};
use azure_core::tracing::Tracer;
use opentelemetry::{
    global::BoxedTracer,
    trace::{TraceContextExt, Tracer as OpenTelemetryTracerTrait},
    Context,
};
use std::sync::Arc;
use typespec_client_core::tracing::SpanKind;

pub struct OpenTelemetryTracer {
    inner: BoxedTracer,
}

impl OpenTelemetryTracer {
    /// Creates a new OpenTelemetry tracer with the given inner tracer.
    pub(super) fn new(tracer: BoxedTracer) -> Self {
        Self { inner: tracer }
    }
}

impl Tracer for OpenTelemetryTracer {
    fn start_span(
        &self,
        name: String,
        kind: SpanKind,
    ) -> Arc<dyn azure_core::tracing::Span + Send + Sync> {
        let span_builder = opentelemetry::trace::SpanBuilder::from_name(name.to_string())
            .with_kind(OpenTelemetrySpanKind(kind).into());
        let context = Context::new();
        let span = self.inner.build_with_context(span_builder, &context);

        OpenTelemetrySpan::new(context.with_span(span))
    }
    fn start_span_with_parent(
        &self,
        name: String,
        kind: SpanKind,
        parent: Arc<dyn azure_core::tracing::Span + Send + Sync>,
    ) -> Arc<dyn azure_core::tracing::Span + Send + Sync> {
        let span_builder = opentelemetry::trace::SpanBuilder::from_name(name.to_string())
            .with_kind(OpenTelemetrySpanKind(kind).into());
        let parent_span = parent
            .as_any()
            .downcast_ref::<OpenTelemetrySpan>()
            .expect("Parent span must be an OpenTelemetrySpan");
        let context = parent_span.context().clone();
        let span = self.inner.build_with_context(span_builder, &context);

        OpenTelemetrySpan::new(context.with_span(span))
    }
}

#[cfg(test)]
mod tests {
    use crate::telemetry::OpenTelemetryTracerProvider;
    use azure_core::tracing::{SpanKind, TracerProvider};
    use opentelemetry::trace::noop::NoopTracerProvider;
    use opentelemetry_sdk::trace::SdkTracerProvider;
    use std::sync::Arc;

    #[test]
    fn test_create_tracer() {
        let noop_tracer = NoopTracerProvider::new();
        let otel_provider = OpenTelemetryTracerProvider::new(Arc::new(noop_tracer)).unwrap();
        let tracer = otel_provider.get_tracer("test_tracer".to_string(), "1.0.0".to_string());
        let span = tracer.start_span("test_span".to_string(), SpanKind::Internal);
        assert!(span.end().is_ok());
    }

    #[test]
    fn test_create_tracer_with_sdk_tracer() {
        let provider = SdkTracerProvider::builder().build();
        let otel_provider = OpenTelemetryTracerProvider::new(Arc::new(provider)).unwrap();
        let _tracer = otel_provider.get_tracer("test_tracer".to_string(), "1.0.0".to_string());
    }

    #[test]
    fn test_create_span_from_tracer() {
        let provider = SdkTracerProvider::builder().build();
        let otel_provider = OpenTelemetryTracerProvider::new(Arc::new(provider)).unwrap();
        let tracer = otel_provider.get_tracer("test_tracer".to_string(), "1.0.0".to_string());
        let _span = tracer.start_span("test_span".to_string(), SpanKind::Internal);
    }
}
