// cspell: words amqp servicebus

use crate::amqp_client::{
    fe2o3::error::AmqpManagementError,
    value::{AmqpOrderedMap, AmqpValue},
    AmqpManagement,
};

use async_trait::async_trait;
use azure_core::error::Result;
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

            return Ok(response.entity_attributes.into());
        }

        todo!();
    }
}
