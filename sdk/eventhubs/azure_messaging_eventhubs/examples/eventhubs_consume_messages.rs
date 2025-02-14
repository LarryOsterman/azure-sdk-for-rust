/// This sample demonstrates how to consume events from an Event Hub partition using the `ConsumerClient`.
///
use azure_core::Result;
use azure_identity::DefaultAzureCredential;
use azure_messaging_eventhubs::{
    ConsumerClient, OpenReceiverOptions, StartLocation, StartPosition,
};
use futures::{pin_mut, StreamExt};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Set up the Event Hub client
    let eventhub_namespace =
        env::var("EVENTHUBS_HOST").expect("Could not find EVENTHUBS_HOST environment variable.");
    let eventhub_name =
        env::var("EVENTHUB_NAME").expect("Could not find EVENTHUB_NAME environment variable.");
    let credential = DefaultAzureCredential::new()?;

    let consumer = ConsumerClient::builder()
        .open(
            eventhub_namespace.as_str(),
            eventhub_name.as_str(),
            credential.clone(),
        )
        .await?;

    println!("Opened consumer client");

    // Get the partition IDs
    let properties = consumer.get_eventhub_properties().await?;
    println!("EventHub Properties: {:?}", properties);

    // The default is to receive messages from the end of the partition, so specify a start position at the start of the partition.
    let receiver = consumer
        .open_receiver_on_partition(
            properties.partition_ids[0].as_str(),
            Some(OpenReceiverOptions {
                start_position: Some(StartPosition {
                    location: StartLocation::Earliest,
                    ..Default::default()
                }),
                receive_timeout: Some(std::time::Duration::from_secs(5)),
                ..Default::default()
            }),
        )
        .await?;

    println!("Created receiver");

    // Create a stream of events from the receiver
    let receive_stream = receiver.stream_events();

    println!("Created receive stream");

    // Pin the receive stream on the stack so that it can be polled
    pin_mut!(receive_stream);

    // Receive events until the receive_timeout has been reached.
    while let Some(event) = receive_stream.next().await {
        println!("Received raw AMQP message: {:?}", event?.raw_amqp_message());
    }

    consumer.close().await?;

    Ok(())
}
