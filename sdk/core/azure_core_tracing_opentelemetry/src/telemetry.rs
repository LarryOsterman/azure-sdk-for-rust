use azure_core::tracing::TracerProvider;
use azure_core::Result;
use opentelemetry::{global::{self, ObjectSafeTracerProvider}, trace::Tracer};
use opentelemetry_sdk::{
    resource::{ResourceDetector, TelemetryResourceDetector},
    trace::TracerProvider as SdkTracerProvider,
    Resource,
};
use std::sync::Arc;

/// Enum to hold different OpenTelemetry tracer provider implementations.
pub struct OpenTelemetryTracerProvider {
    inner: SdkTracerProvider,
}

impl OpenTelemetryTracerProvider {
    /// Creates a new Azure telemetry provider with the given SDK tracer provider.
    pub fn new(provider: SdkTracerProvider) -> Result<Self> {
        global::set_tracer_provider(provider.clone());
        Ok(Self { inner: provider })
    }
}

impl TracerProvider for OpenTelemetryTracerProvider {
    fn get_tracer(&self, name: &str) -> Box<dyn azure_core::tracing::Tracer + Send + Sync> {
        self.inner.boxed_tracer(library)
        todo!()
    }
}
