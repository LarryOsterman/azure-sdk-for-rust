//#![cfg(all(test, feature = "test_e2e"))] // to run this, do: `cargo test --features test_e2e`
//cspell: words eventhubs eventhub

use azure_identity::{DefaultAzureCredential, TokenCredentialOptions};
use azure_messaging_eventhubs::producer::{ProducerClient, ProducerClientOptions};
use log::info;
use std::env;

mod common;

#[tokio::test]
async fn test_new() {
    common::setup();
    let host = env::var("EVENTHUBS_HOST").unwrap();
    let eventhub = env::var("EVENTHUB_NAME").unwrap();
    let _client = ProducerClient::new(
        host,
        eventhub,
        DefaultAzureCredential::create(TokenCredentialOptions::default()).unwrap(),
        ProducerClientOptions::builder()
            .with_application_id("test_new")
            .build(),
    );
}

#[tokio::test]
async fn test_new_with_error() {
    common::setup();
    let eventhub = env::var("EVENTHUB_NAME").unwrap();
    let producer = ProducerClient::new(
        "invalid_host",
        eventhub,
        azure_identity::DefaultAzureCredential::create(TokenCredentialOptions::default()).unwrap(),
        ProducerClientOptions::builder()
            .with_application_id("test_new_with_error")
            .build(),
    )
    .unwrap();
    let result = producer.open().await;
    assert!(result.is_err());
    info!("Error: {:?}", result);
}

#[tokio::test]
async fn test_open() {
    common::setup();
    let host = env::var("EVENTHUBS_HOST").unwrap();
    let eventhub = env::var("EVENTHUB_NAME").unwrap();
    let client = ProducerClient::new(
        host,
        eventhub,
        azure_identity::DefaultAzureCredential::create(TokenCredentialOptions::default()).unwrap(),
        ProducerClientOptions::builder()
            .with_application_id("test_open")
            .build(),
    )
    .unwrap();
    client.open().await.unwrap();
}
#[tokio::test]
async fn test_close() {
    common::setup();
    let host = env::var("EVENTHUBS_HOST").unwrap();
    let eventhub = env::var("EVENTHUB_NAME").unwrap();
    let client = ProducerClient::new(
        host,
        eventhub,
        azure_identity::DefaultAzureCredential::create(TokenCredentialOptions::default()).unwrap(),
        ProducerClientOptions::builder()
            .with_application_id("test_close")
            .build(),
    )
    .unwrap();
    client.open().await.unwrap();
    client.close().await.unwrap();
}

#[tokio::test]
async fn test_get_properties() {
    common::setup();
    let host = env::var("EVENTHUBS_HOST").unwrap();
    let eventhub = env::var("EVENTHUB_NAME").unwrap();

    let credential = DefaultAzureCredential::create(TokenCredentialOptions::default()).unwrap();

    let client = ProducerClient::new(
        host,
        eventhub.clone(),
        credential,
        ProducerClientOptions::builder()
            .with_application_id("test_get_properties")
            .build(),
    )
    .unwrap();
    client.open().await.unwrap();
    let properties = client.get_eventhub_properties().await.unwrap();
    info!("Properties: {:?}", properties);
    assert_eq!(properties.name, eventhub);
}

#[tokio::test]
async fn test_get_partition_properties() {
    common::setup();
    let host = env::var("EVENTHUBS_HOST").unwrap();
    let eventhub = env::var("EVENTHUB_NAME").unwrap();

    let credential = DefaultAzureCredential::create(TokenCredentialOptions::default()).unwrap();

    let client = ProducerClient::new(
        host,
        eventhub.clone(),
        credential,
        ProducerClientOptions::builder()
            .with_application_id("test_get_properties")
            .build(),
    )
    .unwrap();
    client.open().await.unwrap();
    let properties = client.get_eventhub_properties().await.unwrap();

    for partition_id in properties.partition_ids {
        let partition_properties = client
            .get_partition_properties(partition_id.clone())
            .await
            .unwrap();
        info!("Partition properties: {:?}", partition_properties);
        assert_eq!(partition_properties.id, partition_id);
    }
}
