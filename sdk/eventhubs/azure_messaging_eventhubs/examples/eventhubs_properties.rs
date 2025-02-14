// Copyright (c) Microsoft Corporation. All Rights reserved
// Licensed under the MIT license.

use azure_core::Result;
use azure_identity::DefaultAzureCredential;
use azure_messaging_eventhubs::ProducerClient;

use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber from environment.
    tracing_subscriber::fmt().init();

    let host =
        env::var("EVENTHUBS_HOST").expect("Could not find EVENTHUBS_HOST environment variable.");
    let eventhub =
        env::var("EVENTHUB_NAME").expect("Could not find EVENTHUB_NAME environment variable.");

    let credential = DefaultAzureCredential::new()?;

    let result = ProducerClient::builder()
        .with_application_id("test_get_properties")
        .open(host.as_str(), eventhub.as_str(), credential.clone())
        .await;
    if let Err(err) = result {
        eprintln!("Error opening client: {err}");
        return Err(err);
    }
    let client = result?;
    let properties = client.get_eventhub_properties().await.unwrap();
    println!("Eventhub Properties for: {eventhub} {properties:?}");

    Ok(())
}
