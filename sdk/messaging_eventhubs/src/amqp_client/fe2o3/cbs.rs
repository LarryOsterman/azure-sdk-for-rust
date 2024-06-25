// cspell:: words amqp servicebus sastoken

use super::error::AmqpManagementAttachError;
use super::session::Fe2o3AmqpSession;
use crate::amqp_client::{cbs::AmqpClaimsBasedSecurityTrait, fe2o3::error::AmqpManagementError};
use azure_core::error::Result;
use fe2o3_amqp_cbs::token::CbsToken;
use fe2o3_amqp_types::primitives::Timestamp;
use log::trace;
use std::borrow::BorrowMut;
use std::sync::OnceLock;
use tokio::sync::Mutex;

#[derive(Debug)]
pub(crate) struct Fe2o3ClaimsBasedSecurity {
    cbs: OnceLock<Mutex<fe2o3_amqp_cbs::client::CbsClient>>,
    session: Mutex<Fe2o3AmqpSession>,
}

impl Fe2o3ClaimsBasedSecurity {
    pub(crate) fn new(session: Fe2o3AmqpSession) -> Self {
        //     let mut connection = self.connection.get().unwrap().lock().await;
        //     let mut session = fe2o3_amqp::session::Session::begin(connection.borrow_mut())
        //         .await
        //         .map_err(AmqpBeginError::from)?;

        //     let cbs_client = fe2o3_amqp_cbs::client::CbsClient::builder()
        //         .client_node_addr("rust_eventhubs_cbs")
        //         .attach(&mut session)
        //         .await
        //         .map_err(AmqpManagementAttachError::from)?;

        Self {
            cbs: OnceLock::new(),
            session: Mutex::new(session),
        }
    }
}

unsafe impl Send for Fe2o3ClaimsBasedSecurity {}
unsafe impl Sync for Fe2o3ClaimsBasedSecurity {}

impl Fe2o3ClaimsBasedSecurity {}

impl AmqpClaimsBasedSecurityTrait for Fe2o3ClaimsBasedSecurity {
    async fn attach(&self) -> Result<()> {
        let session = self.session.lock().await;
        let cbs_client = fe2o3_amqp_cbs::client::CbsClient::builder()
            .client_node_addr("rust_eventhubs_cbs")
            .attach(session.get().lock().await.borrow_mut())
            .await
            .map_err(AmqpManagementAttachError::from)?;
        self.cbs.set(Mutex::new(cbs_client)).unwrap();
        Ok(())
    }

    async fn authorize_path(
        &self,
        path: &String,
        secret: impl Into<String>,
        expires_at: time::OffsetDateTime,
    ) -> Result<()> {
        trace!(
            "authorize_path: path: {:?}, expires_at: {:?}",
            path,
            expires_at
        );
        let cbs_token = CbsToken::new(
            secret.into(),
            "jwt",
            Some(Timestamp::from(
                expires_at
                    .to_offset(time::UtcOffset::UTC)
                    .unix_timestamp()
                    .checked_mul(1_000)
                    .unwrap(),
            )),
        );
        self.cbs
            .get()
            .unwrap()
            .lock()
            .await
            .borrow_mut()
            .put_token(path, cbs_token)
            .await
            .map_err(AmqpManagementError::from)?;
        trace!("Path authorized successfully.");
        Ok(())
    }
}
