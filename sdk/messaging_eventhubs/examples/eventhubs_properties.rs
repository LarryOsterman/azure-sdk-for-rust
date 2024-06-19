// cspell: words eventhub eventhubs
use azure_core::auth::TokenCredential;
use azure_core::error::Result;
use azure_identity::{DefaultAzureCredential, TokenCredentialOptions};
use azure_messaging_eventhubs::producer::{ProducerClient, ProducerClientOptions};

use env_logger;
use std::env;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let host = env::var("EVENTHUBS_HOST").unwrap();
    let eventhub = env::var("EVENTHUB_NAME").unwrap();

    let credential = DefaultAzureCredential::create(TokenCredentialOptions::default()).unwrap();
    info!("Credential: {:?}", credential);
    let token = credential
        .get_token(&["https://eventhubs.azure.net/.default"])
        .await
        .unwrap();
    info!("Token: {:?}", token.token.secret());

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
    println!("Eventhub Properties for: {eventhub} {:?}", properties);
    Ok(())
}
