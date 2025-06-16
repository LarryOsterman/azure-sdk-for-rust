use azure_core_tracing_opentelemetry::{
    create_azure_span, create_tracer, init_tracing,
    attributes::attribute_value,
    Span, SpanStatus, TracerProvider,
};
use opentelemetry::trace::SpanKind;

#[tokio::test]
async fn integration_test_azure_services() {
    // Initialize tracing
    init_tracing().expect("Failed to initialize tracing");

    // Create a tracer
    let tracer = create_tracer("integration-test");

    // Test Storage service spans
    let storage_span = create_azure_span("azure_storage_list_blobs")
        .with_kind(SpanKind::Client)
        .with_azure_service("storage")
        .with_azure_operation("list_blobs")
        .with_request_id("storage-req-001")
        .with_attribute("account.name", attribute_value::string("teststorage"))
        .with_attribute("container.name", attribute_value::string("data"))
        .build();

    // Test Cosmos DB service spans
    let cosmos_span = create_azure_span("azure_cosmos_query_documents")
        .with_kind(SpanKind::Client)
        .with_azure_service("cosmos")
        .with_azure_operation("query_documents")
        .with_request_id("cosmos-req-002")
        .with_attribute("database.name", attribute_value::string("testdb"))
        .with_attribute("collection.name", attribute_value::string("users"))
        .build();

    // Test Event Hubs service spans
    let eventhubs_span = create_azure_span("azure_eventhubs_send_batch")
        .with_kind(SpanKind::Producer)
        .with_azure_service("eventhubs")
        .with_azure_operation("send_batch")
        .with_request_id("eh-req-003")
        .with_attribute("namespace.name", attribute_value::string("testns"))
        .with_attribute("eventhub.name", attribute_value::string("events"))
        .build();

    // Execute operations
    {
        // Simulate storage operation
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        storage_span.set_attribute("operation.status", attribute_value::string("success"));
        storage_span.set_status(SpanStatus::Ok);
    }

    {
        // Simulate cosmos operation with error
        let error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
        cosmos_span.record_error(&error);
        cosmos_span.set_status(SpanStatus::Error {
            description: "Access denied".to_string(),
        });
    }

    {
        // Simulate event hub operation
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        eventhubs_span.set_attribute("batch.size", attribute_value::i64(10));
        eventhubs_span.set_status(SpanStatus::Ok);
    }

    // End all spans
    storage_span.end();
    cosmos_span.end();
    eventhubs_span.end();
}
