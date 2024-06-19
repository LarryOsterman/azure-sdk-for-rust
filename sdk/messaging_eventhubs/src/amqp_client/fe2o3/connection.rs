// cspell: words amqp widnow eventhubs

use crate::amqp_client::fe2o3::error::{
    AmqpBeginError, AmqpConnectionError, AmqpManagementAttachError,
};
use crate::amqp_client::AmqpConnection;
use crate::amqp_client::AmqpSessionOptions;

use async_trait::async_trait;
use azure_core::Result;
use fe2o3_amqp::connection::ConnectionHandle;
use std::{borrow::BorrowMut, sync::OnceLock};
use tokio::sync::Mutex;
use tracing::debug;

use crate::amqp_client::fe2o3::{cbs::Fe2o3ClaimsBasedSecurity, session::Fe2o3AmqpSession};

#[derive(Debug)]
pub(crate) struct Fe2o3AmqpConnection {
    connection: OnceLock<Mutex<ConnectionHandle<()>>>,
}

unsafe impl Sync for Fe2o3AmqpConnection {}

impl Fe2o3AmqpConnection {
    pub(crate) fn new(connection: fe2o3_amqp::connection::ConnectionHandle<()>) -> Self {
        let connection_cell = OnceLock::new();
        connection_cell.set(Mutex::new(connection)).unwrap();
        Self {
            connection: connection_cell,
        }
    }
}

#[async_trait]
impl AmqpConnection for Fe2o3AmqpConnection {
    async fn close(&self) -> Result<()> {
        let mut connection = self.connection.get().unwrap().lock().await;
        connection
            .borrow_mut()
            .close()
            .await
            .map_err(AmqpConnectionError::from)?;
        Ok(())
    }

    async fn create_session(
        &self,
        options: AmqpSessionOptions,
    ) -> Result<Box<dyn crate::amqp_client::AmqpSession>> {
        let mut session_builder = fe2o3_amqp::session::Session::builder();
        if let Some(incoming_window) = options.incoming_window {
            session_builder = session_builder.incoming_window(incoming_window);
        }
        if let Some(outgoing_window) = options.outgoing_window {
            session_builder = session_builder.outgoing_widnow(outgoing_window);
        }
        if let Some(handle_max) = options.handle_max {
            session_builder = session_builder.handle_max(handle_max);
        }
        if let Some(offered_capabilities) = options.offered_capabilities {
            for capability in offered_capabilities {
                let capability: fe2o3_amqp_types::primitives::Symbol = capability.into();
                session_builder = session_builder.add_offered_capabilities(capability);
            }
        }
        if let Some(desired_capabilities) = options.desired_capabilities {
            for capability in desired_capabilities {
                let capability: fe2o3_amqp_types::primitives::Symbol = capability.into();
                session_builder = session_builder.add_desired_capabilities(capability);
            }
        }
        if let Some(properties) = options.properties {
            let mut fields = fe2o3_amqp::types::definitions::Fields::new();
            for property in properties.iter() {
                debug!("Property: {:?}, Value: {:?}", property.0, property.1);
                let k: fe2o3_amqp_types::primitives::Symbol = property.0.into();
                let v: fe2o3_amqp_types::primitives::Value = property.1.into();
                debug!("Property: {:?}, Value: {:?}", k, v);

                fields.insert(k, v);
            }
            session_builder = session_builder.properties(fields);
        }
        if let Some(buffer_size) = options.buffer_size {
            session_builder = session_builder.buffer_size(buffer_size);
        }

        let session = session_builder
            .begin(self.connection.get().unwrap().lock().await.borrow_mut())
            .await
            .map_err(AmqpBeginError::from)?;
        Ok(Box::new(Fe2o3AmqpSession::new(session)))
    }

    async fn create_claims_based_security(
        &self,
    ) -> Result<Box<dyn crate::amqp_client::AmqpClaimsBasedSecurity>> {
        let mut connection = self.connection.get().unwrap().lock().await;
        let mut session = fe2o3_amqp::session::Session::begin(connection.borrow_mut())
            .await
            .map_err(AmqpBeginError::from)?;

        let cbs_client = fe2o3_amqp_cbs::client::CbsClient::builder()
            .client_node_addr("rust_eventhubs_cbs")
            .attach(&mut session)
            .await
            .map_err(AmqpManagementAttachError::from)?;

        Ok(Box::new(Fe2o3ClaimsBasedSecurity::new(cbs_client, session)))
    }
}
