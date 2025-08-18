// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

use crate::tracer::TracingTracer;
use azure_core::tracing::TracerProvider;
use std::{fmt::Debug, sync::Arc};

/// Enum to hold different Tracing provider implementations.
pub struct TracingTracerProvider {}

impl TracingTracerProvider {
    /// Creates a new Azure telemetry provider with the given SDK tracer provider.
    ///
    /// # Arguments
    /// - `provider`: An `Arc` to an object-safe tracer provider that implements the
    ///   `ObjectSafeTracerProvider` trait.
    ///
    /// # Returns
    /// An `Arc` to the newly created `OpenTelemetryTracerProvider`.
    ///
    ///
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }
}

impl Debug for TracingTracerProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TracingTracerProvider")
            .finish_non_exhaustive()
    }
}

impl TracerProvider for TracingTracerProvider {
    fn get_tracer(
        &self,
        namespace: Option<&'static str>,
        crate_name: &'static str,
        crate_version: Option<&'static str>,
    ) -> Arc<dyn azure_core::tracing::Tracer> {
        Arc::new(TracingTracer::new(namespace, crate_name, crate_version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tracer_provider_sdk_tracer() {
        let _tracer_provider = TracingTracerProvider::new();
    }
}
