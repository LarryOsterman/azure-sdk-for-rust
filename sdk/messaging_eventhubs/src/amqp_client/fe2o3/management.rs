// cspell: words amqp servicebus eventhub mgmt

use std::borrow::BorrowMut;

use crate::amqp_client::{
    fe2o3::error::AmqpManagementError,
    management::AmqpManagementTrait,
    value::{AmqpOrderedMap, AmqpValue},
};

use azure_core::error::Result;
use fe2o3_amqp_management::operations::ReadResponse;
use fe2o3_amqp_types::messaging::ApplicationProperties;
use log::trace;
use tokio::sync::Mutex;

use super::{error::AmqpManagementAttachError, session::Fe2o3AmqpSession};

#[derive(Debug)]
pub(crate) struct Fe2o3AmqpManagement {
    management: Mutex<fe2o3_amqp_management::MgmtClient>,
}

impl Fe2o3AmqpManagement {
    pub(crate) async fn new(
        session: &Fe2o3AmqpSession,
        client_node_name: impl Into<String>,
    ) -> Result<Self> {
        let mut session = session.get().lock().await;

        let management = fe2o3_amqp_management::client::MgmtClient::builder()
            .client_node_addr(client_node_name)
            //            .management_node_address("$management")
            .attach(session.borrow_mut())
            .await
            .map_err(AmqpManagementAttachError::from)?;
        Ok(Self {
            management: Mutex::new(management),
        })
    }
}

impl AmqpManagementTrait for Fe2o3AmqpManagement {
    async fn call(
        &self,
        operation_type: impl Into<String>,
        entity: impl Into<String>,
        application_properties: Option<AmqpOrderedMap<String, AmqpValue>>,
    ) -> Result<AmqpOrderedMap<String, AmqpValue>> {
        let mut management = self.management.lock().await;

        if application_properties.is_none() {
            let request = fe2o3_amqp_management::operations::ReadRequest::name(
                entity.into(),
                operation_type.into(),
                None,
            );

            trace!("Request: {:?}", request);

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
            let request = GetPartitionRequest::new(entity, partition_id);
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
    pub fn new(eventhub: impl Into<String>, partition_id: impl Into<String>) -> Self {
        Self {
            eventhub: eventhub.into(),
            partition_id: partition_id.into(),
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
