//cspell: words amqp eventhub amqps servicebus eventhubs

use crate::{
    amqp_client::{
        value::{AmqpOrderedMap, AmqpValue},
        AmqpConnection, AmqpConnectionBuilder, AmqpSender, AmqpSession, AmqpSessionOptions,
    },
    common::user_agent::{
        get_package_name, get_package_version, get_platform_info, get_user_agent,
    },
    error::ErrorKind,
    EventHubPartitionProperties, EventHubProperties,
};
use azure_core::RetryOptions;
use azure_core::{
    auth::AccessToken,
    error::{Error, Result},
};
use std::{boxed::Box, collections::HashMap};
use std::{
    sync::{Arc, OnceLock},
    time::SystemTime,
};
use time::UtcOffset;
use tokio::sync::Mutex;
use tracing::debug;
use url::Url;

pub struct ProducerClientOptions {
    application_id: Option<String>,
    retry_options: Option<RetryOptions>,
}

impl ProducerClientOptions {
    pub fn builder() -> ProducerClientOptionsBuilder {
        ProducerClientOptionsBuilder::new()
    }
}

pub struct ProducerClientOptionsBuilder {
    application_id: Option<String>,
    retry_options: Option<RetryOptions>,
}

impl ProducerClientOptionsBuilder {
    pub fn new() -> Self {
        Self {
            application_id: None,
            retry_options: None,
        }
    }

    pub fn with_application_id<T: Into<String>>(mut self, application_id: T) -> Self {
        self.application_id = Some(application_id.into());
        self
    }

    pub fn with_retry_options(mut self, retry_options: RetryOptions) -> Self {
        self.retry_options = Some(retry_options);
        self
    }

    pub fn build(self) -> ProducerClientOptions {
        ProducerClientOptions {
            application_id: self.application_id,
            retry_options: self.retry_options,
        }
    }
}

struct SenderInstance {
    #[allow(dead_code)]
    session: Box<dyn AmqpSession>,
    sender: Arc<Mutex<Box<dyn AmqpSender>>>,
}

#[derive(Debug)]
struct ManagementInstance {
    #[allow(dead_code)]
    session: Box<dyn AmqpSession>,
    management: Box<dyn crate::amqp_client::AmqpManagement>,
}

impl ManagementInstance {
    fn new(
        session: Box<dyn AmqpSession>,
        management: Box<dyn crate::amqp_client::AmqpManagement>,
    ) -> Self {
        Self {
            session,
            management,
        }
    }
}

pub struct ProducerClient {
    options: ProducerClientOptions,
    connection: OnceLock<Box<dyn AmqpConnection>>,
    credential: Box<dyn azure_core::auth::TokenCredential>,
    fully_qualified_namespace: String,
    eventhub: String,
    url: String,
    authorization_scopes: Mutex<HashMap<String, AccessToken>>,
    mgmt_client: Mutex<OnceLock<ManagementInstance>>,
}

impl ProducerClient {
    pub fn new(
        fully_qualified_namespace: impl Into<String>,
        eventhub: impl Into<String>,
        credential: impl azure_core::auth::TokenCredential + 'static,
        options: ProducerClientOptions,
    ) -> Result<Self> {
        let eventhub: String = eventhub.into();
        let fully_qualified_namespace: String = fully_qualified_namespace.into();
        Ok(Self {
            options,
            connection: OnceLock::new(),
            credential: Box::new(credential),
            url: format!("amqps://{}/{}", fully_qualified_namespace, eventhub),
            eventhub: eventhub,
            fully_qualified_namespace: fully_qualified_namespace,
            authorization_scopes: Mutex::new(HashMap::new()),
            mgmt_client: Mutex::new(OnceLock::new()),
        })
    }

    pub async fn open(&self) -> Result<()> {
        self.ensure_connection(&self.url).await?;
        Ok(())
    }

    pub async fn close(self) -> Result<()> {
        self.connection.get().unwrap().close().await?;
        Ok(())
    }

    pub async fn get_eventhub_properties(&self) -> Result<EventHubProperties> {
        self.authorize_path(&self.url).await?;
        self.ensure_management_client().await?;

        let response = self
            .mgmt_client
            .lock()
            .await
            .get()
            .unwrap()
            .management
            .call("com.microsoft:eventhub", self.eventhub.as_str(), None)
            .await?;

        if !response.contains_key("name".to_string())
            || !response.contains_key("type".to_string())
            || !response.contains_key("created_at".to_string())
            || !response.contains_key("partition_count".to_string())
            || !response.contains_key("partition_ids".to_string())
        {
            return Err(ErrorKind::InvalidManagementResponse.into());
        }
        let name: String = response.get("name".to_string()).unwrap().clone().into();
        let created_at: SystemTime =
            Into::<SystemTime>::into(response.get("created_at".to_string()).unwrap().clone());
        //        let partition_count: i32 =
        //            Into::<i32>::into(response.get("partition_count".to_string()).unwrap().clone());

        let partition_ids = response.get("partition_ids".to_string()).unwrap();
        let partition_ids = match partition_ids {
            AmqpValue::Array(partition_ids) => partition_ids
                .iter()
                .map(|id| match id {
                    AmqpValue::String(id) => Ok(id.clone()),
                    _ => Err(ErrorKind::InvalidManagementResponse.into()),
                })
                .collect::<Result<Vec<String>>>()?,
            _ => return Err(ErrorKind::InvalidManagementResponse.into()),
        };
        Ok(EventHubProperties {
            name,
            created_on: created_at,
            partition_ids,
        })
    }

    fn get_entity_property<T: TryFrom<AmqpValue>>(
        map: &AmqpOrderedMap<String, AmqpValue>,
        property_name: &str,
    ) -> T
    where
        <T as TryFrom<AmqpValue>>::Error: core::fmt::Debug,
    {
        TryInto::<T>::try_into(map.get(property_name.into()).unwrap().clone()).unwrap()
    }

    pub async fn get_partition_properties(
        &self,
        partition_id: impl Into<String>,
    ) -> Result<EventHubPartitionProperties> {
        self.authorize_path(&self.url).await?;
        self.ensure_management_client().await?;

        let partition_id: String = partition_id.into();

        let mut application_properties: AmqpOrderedMap<String, AmqpValue> = AmqpOrderedMap::new();
        application_properties.insert("partition_id".to_string(), partition_id.into());
        application_properties.insert("name".to_string(), self.eventhub.clone().into());

        let response = self
            .mgmt_client
            .lock()
            .await
            .get()
            .unwrap()
            .management
            .call(
                "com.microsoft:eventhub",
                self.eventhub.as_str(),
                Some(application_properties),
            )
            .await?;

        // Look for the required response properties
        if !response.contains_key("name".to_string())
            || !response.contains_key("type".to_string())
            || !response.contains_key("partition".to_string())
            || !response.contains_key("begin_sequence_number_epoch".to_string())
            || !response.contains_key("begin_sequence_number".to_string())
            || !response.contains_key("last_enqueued_sequence_number_epoch".to_string())
            || !response.contains_key("last_enqueued_sequence_number".to_string())
            || !response.contains_key("last_enqueued_offset".to_string())
            || !response.contains_key("last_enqueued_time_utc".to_string())
            || !response.contains_key("is_partition_empty".to_string())
        {
            return Err(ErrorKind::InvalidManagementResponse.into());
        }

        Ok(EventHubPartitionProperties {
            beginning_sequence_number: Self::get_entity_property(
                &response,
                "begin_sequence_number",
            ),
            id: Self::get_entity_property(&response, "partition"),
            eventhub: Self::get_entity_property(&response, "name"),
            last_enqueued_sequence_number: Self::get_entity_property(
                &response,
                "last_enqueued_sequence_number",
            ),
            last_enqueued_offset: Self::get_entity_property::<String>(
                &response,
                "last_enqueued_offset",
            )
            .parse()
            .unwrap(),
            last_enqueued_time_utc: Into::<SystemTime>::into(
                response
                    .get("last_enqueued_time_utc".to_string())
                    .unwrap()
                    .clone(),
            ),
            is_empty: Self::get_entity_property(&response, "is_partition_empty"),
        })
    }

    async fn ensure_management_client(&self) -> Result<()> {
        let mgmt_client = self.mgmt_client.lock().await;

        if mgmt_client.get().is_some() {
            return Ok(());
        }

        // Clients must call ensure_connection before calling ensure_management_client.
        if self.connection.get().is_none() {
            return Err(ErrorKind::MissingConnection.into());
        }

        let connection = self.connection.get().unwrap();
        let session = connection
            .create_session(AmqpSessionOptions::builder().build())
            .await?;
        let management = session.create_management("eventhub").await?;
        mgmt_client
            .set(ManagementInstance::new(session, management))
            .unwrap();
        Ok(())
    }

    async fn ensure_connection(&self, url: &String) -> Result<()> {
        if self.connection.get().is_none() {
            let connection = AmqpConnectionBuilder::new()
                .with_properties(vec![
                    ("user-agent", get_user_agent(&self.options.application_id)),
                    ("version", get_package_version().into()),
                    ("platform", get_platform_info().into()),
                    ("product", get_package_name().into()),
                ])
                .open(
                    self.options
                        .application_id
                        .clone()
                        .unwrap_or(uuid::Uuid::new_v4().to_string())
                        .as_str(),
                    Url::parse(url.as_str()).map_err(Error::from)?,
                )
                .await?;
            self.connection.set(connection).unwrap();
        }
        Ok(())
    }

    async fn authorize_path(&self, url: &str) -> Result<()> {
        debug!("Authorizing path: {}", url);
        let mut scopes = self.authorization_scopes.lock().await;
        if self.connection.get().is_none() {
            return Err(ErrorKind::MissingConnection.into());
        }
        if !scopes.contains_key(url) {
            let connection = self.connection.get().unwrap();
            let cbs = connection.create_claims_based_security().await?;

            debug!("Get Token.");
            let token = self
                .credential
                .get_token(&[&"https://eventhubs.azure.net/.default"])
                .await?;
            debug!("Got token: {:?}", token.token.secret());
            let expires_at = token
                .expires_on
                .to_offset(UtcOffset::UTC)
                .unix_timestamp()
                .checked_mul(1_000)
                .unwrap();
            cbs.authorize_path(url, token.token.secret(), expires_at)
                .await?;
            scopes.insert(url.to_string(), token);
        }
        Ok(())
    }

    async fn ensure_session(&self, partition_id: impl Into<String>) -> Result<()> {
        let connection = self.connection.get().unwrap();

        let session = connection
            .create_session(AmqpSessionOptions::builder().build())
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_producer_client_options_builder() {
        let options = ProducerClientOptions::builder()
            .with_application_id("application_id")
            .with_retry_options(RetryOptions::default())
            .build();
        assert_eq!(options.application_id.unwrap(), "application_id");
    }
}
