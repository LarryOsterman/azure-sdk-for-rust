use azure_core_tracing_opentelemetry::{
    create_azure_span, create_tracer, create_tracer_provider, init_tracing,
    attributes::{attribute_value, key_value},
    conventions, AttributeValue, KeyValue, Span, SpanStatus,
};
use opentelemetry::trace::SpanKind;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing with default configuration
    init_tracing()?;

    // Create a tracer provider
    let tracer_provider = create_tracer_provider();
    let tracer = tracer_provider.get_tracer("example-service");

    // Create spans using the tracer
    let span1 = tracer.start_span("operation1");
    span1.set_attribute("operation.type", attribute_value::string("read"));
    span1.set_attribute("operation.size", attribute_value::i64(1024));

    // Create spans using the Azure span builder
    let span2 = create_azure_span("azure_storage_operation")
        .with_kind(SpanKind::Client)
        .with_azure_service("storage")
        .with_azure_operation("list_blobs")
        .with_request_id("req-12345")
        .with_attribute("container.name", attribute_value::string("my-container"))
        .build();

    // Use spans to trace operations
    {
        println!("Executing operation 1...");

        // Record attributes on the span
        span1.set_attribute("status", attribute_value::string("in_progress"));

        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(100));

        span1.set_attribute("status", attribute_value::string("completed"));
        span1.set_status(SpanStatus::Ok);
    }

    {
        println!("Executing Azure storage operation...");

        // Simulate an error
        let error = std::io::Error::new(std::io::ErrorKind::NotFound, "Container not found");
        span2.record_error(&error);

        // Add Azure-specific attributes
        span2.set_azure_service("storage");
        span2.set_azure_operation("list_blobs");
        span2.set_azure_request_id("req-12345");
    }

    // End spans
    span1.end();
    span2.end();

    println!("Tracing example completed successfully!");

    Ok(())
}
