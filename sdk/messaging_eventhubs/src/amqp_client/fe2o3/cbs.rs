// cspell:: words amqp servicebus sastoken

use crate::amqp_client::{cbs::AmqpClaimsBasedSecurityTrait, fe2o3::error::AmqpManagementError};
use azure_core::error::Result;
use fe2o3_amqp_cbs::token::CbsToken;
use fe2o3_amqp_types::primitives::Timestamp;
use log::trace;
use std::borrow::BorrowMut;
use tokio::sync::Mutex;

#[derive(Debug)]
pub(crate) struct Fe2o3ClaimsBasedSecurity {
    cbs: Mutex<fe2o3_amqp_cbs::client::CbsClient>,
    session: fe2o3_amqp::session::SessionHandle<()>,
}

impl Fe2o3ClaimsBasedSecurity {
    pub(crate) fn new(
        cbs: fe2o3_amqp_cbs::client::CbsClient,
        session: fe2o3_amqp::session::SessionHandle<()>,
    ) -> Self {
        Self {
            cbs: Mutex::new(cbs),
            session,
        }
    }
}

unsafe impl Send for Fe2o3ClaimsBasedSecurity {}
unsafe impl Sync for Fe2o3ClaimsBasedSecurity {}

impl Fe2o3ClaimsBasedSecurity {}

impl AmqpClaimsBasedSecurityTrait for Fe2o3ClaimsBasedSecurity {
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
