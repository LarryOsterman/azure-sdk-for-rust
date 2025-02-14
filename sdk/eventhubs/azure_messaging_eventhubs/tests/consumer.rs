// Copyright (c) Microsoft Corporation. All Rights reserved
// Licensed under the MIT license.

use async_std::future::timeout;
use azure_core_test::recorded;
use azure_core_test::TestContext;
use azure_identity::DefaultAzureCredential;
use azure_messaging_eventhubs::{ConsumerClient, OpenReceiverOptions, StartPosition};
use futures::{pin_mut, StreamExt};
use std::time::Duration;
use tracing::{info, trace};
mod common;

#[recorded::test(live)]
async fn test_new(ctx: TestContext) -> Result<(), azure_core::Error> {
    common::setup();

    let recording = ctx.recording();

    let host = recording.var("EVENTHUBS_HOST", None);
    let eventhub = recording.var("EVENTHUB_NAME", None);
    let _client = ConsumerClient::builder()
        .with_application_id("test_new")
        .open(
            host.as_str(),
            eventhub.as_str(),
            DefaultAzureCredential::new()?,
        )
        .await?;

    Ok(())
}

#[recorded::test(live)]
async fn test_new_with_error(ctx: TestContext) -> Result<(), azure_core::Error> {
    common::setup();

    let recording = ctx.recording();

    trace!("test_new_with_error");
    let eventhub = recording.var("EVENTHUB_NAME", None);
    let result = ConsumerClient::builder()
        .with_application_id("test_new")
        .open(
            "invalid_host",
            eventhub.as_str(),
            DefaultAzureCredential::new()?,
        )
        .await;
    match result {
        Ok(_) => panic!("Expected error, but got Ok"),
        Err(e) => {
            info!("Error: {:?}", e);
            assert!(e.to_string().contains("Invalid URI"));
        }
    }

    Ok(())
}

#[recorded::test(live)]
async fn test_open(ctx: TestContext) -> Result<(), azure_core::Error> {
    common::setup();

    let recording = ctx.recording();
    let host = recording.var("EVENTHUBS_HOST", None);
    let eventhub = recording.var("EVENTHUB_NAME", None);
    let _client = ConsumerClient::builder()
        .with_application_id("test_open")
        .open(
            host.as_str(),
            eventhub.as_str(),
            azure_identity::DefaultAzureCredential::new()?,
        )
        .await?;

    Ok(())
}
#[recorded::test(live)]
async fn test_close(ctx: TestContext) -> Result<(), azure_core::Error> {
    common::setup();

    let recording = ctx.recording();
    let host = recording.var("EVENTHUBS_HOST", None);
    let eventhub = recording.var("EVENTHUB_NAME", None);
    let client = ConsumerClient::builder()
        .with_application_id("test_open")
        .open(
            host.as_str(),
            eventhub.as_str(),
            azure_identity::DefaultAzureCredential::new()?,
        )
        .await?;
    client.close().await?;

    Ok(())
}

#[recorded::test(live)]
async fn test_get_properties(ctx: TestContext) -> Result<(), azure_core::Error> {
    common::setup();
    let recording = ctx.recording();
    let host = recording.var("EVENTHUBS_HOST", None);
    let eventhub = recording.var("EVENTHUB_NAME", None);

    let credential = DefaultAzureCredential::new()?;

    let client = ConsumerClient::builder()
        .with_application_id("test_open")
        .open(host.as_str(), eventhub.as_str(), credential.clone())
        .await?;
    let properties = client.get_eventhub_properties().await?;
    info!("Properties: {:?}", properties);
    assert_eq!(properties.name, eventhub);

    Ok(())
}

#[recorded::test(live)]
async fn test_get_partition_properties(ctx: TestContext) -> Result<(), azure_core::Error> {
    common::setup();

    let recording = ctx.recording();

    let host = recording.var("EVENTHUBS_HOST", None);
    let eventhub = recording.var("EVENTHUB_NAME", None);

    let credential = DefaultAzureCredential::new()?;

    let client = ConsumerClient::builder()
        .with_application_id("test_open")
        .open(host.as_str(), eventhub.as_str(), credential.clone())
        .await?;
    let properties = client.get_eventhub_properties().await?;

    for partition_id in properties.partition_ids {
        let partition_properties = client
            .get_partition_properties(partition_id.as_str())
            .await?;
        info!("Partition properties: {:?}", partition_properties);
        assert_eq!(partition_properties.id, partition_id);
    }

    Ok(())
}

#[recorded::test(live)]
async fn receive_lots_of_events(ctx: TestContext) -> Result<(), azure_core::Error> {
    common::setup();

    let recording = ctx.recording();

    let host = recording.var("EVENTHUBS_HOST", None);
    let eventhub = recording.var("EVENTHUB_NAME", None);

    info!("Establishing credentials.");

    let credential = DefaultAzureCredential::new()?;

    info!("Creating client.");
    let client = ConsumerClient::builder()
        .with_application_id("test_open")
        .open(host.as_str(), eventhub.as_str(), credential.clone())
        .await?;

    let receiver = client
        .open_receiver_on_partition(
            "0",
            Some(OpenReceiverOptions {
                start_position: Some(StartPosition {
                    location: azure_messaging_eventhubs::StartLocation::Earliest,
                    ..Default::default()
                }),
                // Timeout for individual receive operations.
                receive_timeout: Some(Duration::from_secs(5)),
                ..Default::default()
            }),
        )
        .await?;

    info!("Creating event receive stream.");
    let event_stream = receiver.stream_events();

    pin_mut!(event_stream); // Needed for iteration.

    let mut count = 0;

    const TEST_DURATION: std::time::Duration = Duration::from_secs(10);
    info!("Receiving events for {:?}.", TEST_DURATION);

    // Read events from the stream for a bit of time.

    let result = timeout(TEST_DURATION, async {
        while let Some(event) = event_stream.next().await {
            match event {
                Ok(_event) => {
                    //                    info!("Received the following message:: {:?}", event);
                    count += 1;
                }
                Err(err) => {
                    info!("Error while receiving message: {:?}", err);
                }
            }
        }
    })
    .await;

    info!("Received {count} messages in {TEST_DURATION:?}. Timeout: {result:?}");

    Ok(())
}
