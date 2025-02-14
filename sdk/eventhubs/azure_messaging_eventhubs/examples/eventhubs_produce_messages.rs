/// This sample demonstrates how to send AMQP messages to an Event Hub partition using the `ProducerClient`.
///
use azure_core::Result;
use azure_identity::DefaultAzureCredential;
use azure_messaging_eventhubs::{
    models::{AmqpMessage, AmqpValue},
    ProducerClient,
};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Set up the Event Hub client
    let eventhub_namespace =
        env::var("EVENTHUBS_HOST").expect("Could not find EVENTHUBS_HOST environment variable.");
    let eventhub_name =
        env::var("EVENTHUB_NAME").expect("Could not find EVENTHUB_NAME environment variable.");
    let credential = DefaultAzureCredential::new()?;

    let client = ProducerClient::builder()
        .open(
            eventhub_namespace.as_str(),
            eventhub_name.as_str(),
            credential.clone(),
        )
        .await?;

    println!("Created producer client.");

    // Send a message to an eventhub instance directly. The message will be sent to a random partition.
    client
        .send_message(
            AmqpMessage::builder()
                .with_body(AmqpValue::from("Hello, Event Hubs from AMQP!"))
                .build(),
            None,
        )
        .await?;

    // Send an AMQP message whose body is an array of bytes to a random partition of the Event Hubs instance.
    client
        .send_message(
            AmqpMessage::builder().with_body(vec![2, 13, 8, 16]).build(),
            None,
        )
        .await?;

    // Send an AMQP message whose body is an AMQP Value to a random partition.
    client
        .send_message(
            AmqpMessage::builder()
                .with_body(AmqpValue::from("String Value"))
                .build(),
            None,
        )
        .await?;

    println!("Sent messages. Closing client.");

    client.close().await?;
    Ok(())
}
