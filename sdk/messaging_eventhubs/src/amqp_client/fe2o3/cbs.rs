// cspell:: words amqp servicebus sastoken

use std::borrow::BorrowMut;

use crate::amqp_client::fe2o3::error::AmqpManagementError;
use crate::amqp_client::AmqpClaimsBasedSecurity;
use async_trait::async_trait;
use azure_core::error::Result;
use fe2o3_amqp_cbs::token::CbsToken;
use fe2o3_amqp_types::primitives::Timestamp;
use tokio::sync::Mutex;

#[derive(Debug)]
pub(crate) struct Fe2o3ClaimsBasedSecurity {
    cbs: Box<Mutex<fe2o3_amqp_cbs::client::CbsClient>>,
    session: fe2o3_amqp::session::SessionHandle<()>,
}

impl Fe2o3ClaimsBasedSecurity {
    pub(crate) fn new(
        cbs: fe2o3_amqp_cbs::client::CbsClient,
        session: fe2o3_amqp::session::SessionHandle<()>,
    ) -> Self {
        Self {
            cbs: Box::new(Mutex::new(cbs)),
            session,
        }
    }
}

unsafe impl Send for Fe2o3ClaimsBasedSecurity {}
unsafe impl Sync for Fe2o3ClaimsBasedSecurity {}

#[async_trait]
impl AmqpClaimsBasedSecurity for Fe2o3ClaimsBasedSecurity {
    async fn authorize_path(&self, path: &str, secret: &str, expires_at: i64) -> Result<()> {
        let cbs_token = CbsToken::new(secret, "jwt", Some(Timestamp::from(expires_at)));
        self.cbs
            .lock()
            .await
            .borrow_mut()
            .put_token(path, cbs_token)
            .await
            .map_err(AmqpManagementError::from)?;
        Ok(())
    }
}
