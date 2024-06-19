// cspell: words amqp servicebus eventhub mgmt

use crate::amqp_client::{
    fe2o3::error::AmqpManagementError,
    value::{AmqpOrderedMap, AmqpValue},
    AmqpManagement,
};

use async_trait::async_trait;
use azure_core::error::Result;
use fe2o3_amqp_management::operations::ReadResponse;
use fe2o3_amqp_types::messaging::ApplicationProperties;
use tokio::sync::Mutex;

#[derive(Debug)]
pub(crate) struct Fe2o3AmqpManagement {
    management: Mutex<fe2o3_amqp_management::MgmtClient>,
}

unsafe impl Send for Fe2o3AmqpManagement {}
unsafe impl Sync for Fe2o3AmqpManagement {}

impl Fe2o3AmqpManagement {
    pub(crate) fn new(management: fe2o3_amqp_management::MgmtClient) -> Self {
        Self {
            management: Mutex::new(management),
        }
    }
}

#[async_trait]
impl AmqpManagement for Fe2o3AmqpManagement {
    async fn call(
        &self,
        operation_type: &str,
        entity: &str,
        application_properties: Option<AmqpOrderedMap<String, AmqpValue>>,
    ) -> Result<AmqpOrderedMap<String, AmqpValue>> {
        let mut management = self.management.lock().await;

        if application_properties.is_none() {
            let request =
                fe2o3_amqp_management::operations::ReadRequest::name(entity, operation_type, None);

            let response = management
                .call(request)
                .await
                .map_err(AmqpManagementError::from)?;

            Ok(response.entity_attributes.into())
        } else {
            let partition_id: String = application_properties
                .unwrap()
                .get("partition_id".to_string())
                .unwrap()
                .to_owned()
                .into();
            let request = GetPartitionRequest::new(entity, partition_id.as_str());
            let response = management
                .call(request)
                .await
                .map_err(AmqpManagementError::from)?;
            Ok(response.entity_attributes.into())
        }
    }
}

struct GetPartitionRequest {
    eventhub: String,
    partition_id: String,
}

impl GetPartitionRequest {
    pub fn new(eventhub: &str, partition_id: &str) -> Self {
        Self {
            eventhub: eventhub.to_owned(),
            partition_id: partition_id.to_owned(),
        }
    }
}

impl fe2o3_amqp_management::Request for GetPartitionRequest {
    const OPERATION: &'static str = "READ";
    type Response = ReadResponse;
    type Body = ();

    fn manageable_entity_type(&mut self) -> Option<String> {
        Some("com.microsoft:partition".to_owned())
    }
    fn locales(&mut self) -> Option<String> {
        None
    }
    fn encode_application_properties(
        &mut self,
    ) -> Option<fe2o3_amqp_types::messaging::ApplicationProperties> {
        Some(
            ApplicationProperties::builder()
                .insert("name", self.eventhub.clone())
                .insert("partition", self.partition_id.clone())
                .build(),
        )
    }
    fn encode_body(self) -> Self::Body {}
}
