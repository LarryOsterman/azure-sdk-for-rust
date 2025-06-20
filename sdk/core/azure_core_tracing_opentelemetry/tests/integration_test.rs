// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

use azure_core::tracing::{SpanKind, TracerProvider};
use azure_core_tracing_opentelemetry::OpenTelemetryTracerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use std::error::Error;
use std::sync::Arc;

#[tokio::test]
async fn test_span_creation() -> Result<(), Box<dyn Error>> {
    // Set up a tracer provider for testing
    let sdk_provider = Arc::new(SdkTracerProvider::builder().build());
    let azure_provider = OpenTelemetryTracerProvider::new(sdk_provider)?;

    // Get a tracer from the Azure provider
    let tracer = azure_provider.get_tracer("test_tracer".to_string(), "1.0.0".to_string());

    // Create a span using the Azure tracer
    let span = tracer.start_span("test_span".to_string(), SpanKind::Internal);

    // Add attributes to the span using individual set_attribute calls
    span.set_attribute(
        "test_key".to_string(),
        azure_core::tracing::attributes::AttributeValue::String("test_value".to_string()),
    )?;
    span.set_attribute(
        "service.name".to_string(),
        azure_core::tracing::attributes::AttributeValue::String("azure-test".to_string()),
    )?;

    // End the span
    span.end()?;

    Ok(())
}

#[tokio::test]
async fn test_tracer_provider_creation() -> Result<(), Box<dyn Error>> {
    // Create multiple tracer provider instances to test initialization
    let sdk_provider = Arc::new(SdkTracerProvider::builder().build());
    let azure_provider = OpenTelemetryTracerProvider::new(sdk_provider)?;

    // Get a tracer and verify it works
    let tracer = azure_provider.get_tracer("test_tracer".to_string(), "1.0.0".to_string());
    let span = tracer.start_span("test_span".to_string(), SpanKind::Internal);
    span.end()?;

    Ok(())
}

#[tokio::test]
async fn test_span_attributes() -> Result<(), Box<dyn Error>> {
    // Set up a tracer provider for testing
    let sdk_provider = Arc::new(SdkTracerProvider::builder().build());
    let azure_provider = OpenTelemetryTracerProvider::new(sdk_provider)?;

    // Get a tracer from the Azure provider
    let tracer = azure_provider.get_tracer("test_tracer".to_string(), "1.0.0".to_string());

    // Create span with multiple attributes
    let span = tracer.start_span("test_span".to_string(), SpanKind::Internal);

    // Add attributes using individual set_attribute calls
    span.set_attribute(
        "service.name".to_string(),
        azure_core::tracing::attributes::AttributeValue::String("test-service".to_string()),
    )?;
    span.set_attribute(
        "operation.name".to_string(),
        azure_core::tracing::attributes::AttributeValue::String("test-operation".to_string()),
    )?;
    span.set_attribute(
        "request.id".to_string(),
        azure_core::tracing::attributes::AttributeValue::String("req-123".to_string()),
    )?;

    // End the span
    span.end()?;

    Ok(())
}
