// cspell: words amqp

use super::management::Fe2o3AmqpManagement;
use crate::amqp_client::{
    fe2o3::error::AmqpManagementAttachError, value::AmqpValue, AmqpSenderOptions, AmqpSession,
};
use async_trait::async_trait;
use azure_core::Result;
use std::{borrow::BorrowMut, sync::OnceLock};
use tokio::sync::Mutex;

#[derive(Debug)]
pub(crate) struct Fe2o3AmqpSession {
    session: OnceLock<Mutex<fe2o3_amqp::session::SessionHandle<()>>>,
}

unsafe impl Sync for Fe2o3AmqpSession {}

#[async_trait]
impl AmqpSession for Fe2o3AmqpSession {
    async fn end(&self) -> Result<()> {
        todo!()
    }

    async fn create_sender(
        &self,
        target: AmqpValue,
        options: Option<AmqpSenderOptions>,
    ) -> Result<Box<dyn crate::amqp_client::AmqpSender>> {
        todo!()
    }

    async fn create_management(
        &self,
        client_node_name: &str,
    ) -> Result<Box<dyn crate::amqp_client::AmqpManagement>> {
        let mut session = self.session.get().unwrap().lock().await;

        let management = fe2o3_amqp_management::client::MgmtClient::builder()
            .client_node_addr(client_node_name)
            .management_node_address("$management")
            .attach(session.borrow_mut())
            .await
            .map_err(|err| AmqpManagementAttachError(err))?;
        Ok(Box::new(Fe2o3AmqpManagement::new(management)))
    }
}

impl Fe2o3AmqpSession {
    pub(crate) fn new(session: fe2o3_amqp::session::SessionHandle<()>) -> Self {
        let session_once_lock = OnceLock::new();
        session_once_lock.set(Mutex::new(session)).unwrap();
        Self {
            session: session_once_lock,
        }
    }
}
