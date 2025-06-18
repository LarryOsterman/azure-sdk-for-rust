// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

use azure_core::tracing::Tracer;
use opentelemetry::global::ObjectSafeTracer;

pub struct OpenTelemetryTracer {
    inner: Box<dyn ObjectSafeTracer + Send + Sync>,
}

impl OpenTelemetryTracer {
    /// Creates a new OpenTelemetry tracer with the given inner tracer.
    pub(super) fn new(tracer: Box<dyn ObjectSafeTracer + Send + Sync>) -> Self {
        Self { inner: tracer }
    }
}

impl Tracer for OpenTelemetryTracer {
    fn start_span(&self, name: &str) -> Box<dyn azure_core::tracing::Span + Send + Sync> {
        self.inner.start_span(name).boxed()
    }

    fn end_span(&self, span: Box<dyn azure_core::tracing::Span + Send + Sync>) {
        span.end();
    }

    fn set_attribute(&self, key: &str, value: &str) {
        self.inner.set_attribute(key, value);
    }

    fn set_active_span(&self, span: Box<dyn azure_core::tracing::Span + Send + Sync>) {
        todo!()
    }
}
