// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

use crate::span::{TracingSpan, TracingSpanKind};

use azure_core::tracing::{SpanKind, Tracer};
use std::{fmt::Debug, sync::Arc};

pub struct TracingTracer {
    namespace: Option<&'static str>,
    crate_name: &'static str,
    crate_version: Option<&'static str>,
}

impl TracingTracer {
    /// Creates a new OpenTelemetry tracer with the given inner tracer.
    pub(super) fn new(
        namespace: Option<&'static str>,
        crate_name: &'static str,
        crate_version: Option<&'static str>,
    ) -> Self {
        Self {
            namespace,
            crate_name,
            crate_version,
        }
    }
}

impl Debug for TracingTracer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TracingTracer")
            .field("namespace", &self.namespace)
            .finish_non_exhaustive()
    }
}

impl Tracer for TracingTracer {
    fn namespace(&self) -> Option<&'static str> {
        self.namespace
    }

    fn start_span(
        &self,
        name: &'static str,
        kind: SpanKind,
        attributes: Vec<azure_core::tracing::Attribute>,
    ) -> Arc<dyn azure_core::tracing::Span> {
        TracingSpan::new(name, kind, self.crate_name, self.crate_version, attributes)
    }

    fn start_span_with_parent(
        &self,
        _name: &'static str,
        _kind: SpanKind,
        _attributes: Vec<azure_core::tracing::Attribute>,
        _parent: Arc<dyn azure_core::tracing::Span>,
    ) -> Arc<dyn azure_core::tracing::Span> {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use azure_core::tracing::{SpanKind, TracerProvider};
    use std::sync::Arc;

    #[test]
    fn test_create_tracer() {
        todo!();
    }

    #[test]
    fn test_create_tracer_with_sdk_tracer() {
        todo!();
    }

    #[test]
    fn test_create_span_from_tracer() {
        todo!();
    }
}
