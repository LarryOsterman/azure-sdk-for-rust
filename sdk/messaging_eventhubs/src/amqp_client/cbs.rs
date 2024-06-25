// cspell: words amqp sasl

use azure_core::error::Result;

#[cfg(any(feature = "enable-fe2o3-amqp"))]
use super::fe2o3::cbs::Fe2o3ClaimsBasedSecurity;

#[cfg(not(any(feature = "enable-fe2o3-amqp")))]
use super::noop::NoopAmqpClaimsBasedSecurity;
use super::session::AmqpSession;

pub(crate) trait AmqpClaimsBasedSecurityTrait {
    async fn attach(&self) -> Result<()> {
        unimplemented!()
    }
    async fn authorize_path(
        &self,
        path: &String,
        secret: impl Into<String>,
        expires_on: time::OffsetDateTime,
    ) -> Result<()> {
        unimplemented!()
    }
}

#[derive(Debug)]
struct AmqpClaimsBasedSecurityImpl<T>(T);

impl<T> AmqpClaimsBasedSecurityImpl<T>
where
    T: AmqpClaimsBasedSecurityTrait,
{
    pub(crate) fn new(cbs: T) -> Self {
        Self(cbs)
    }
}

#[derive(Debug)]
#[cfg(any(feature = "enable-fe2o3-amqp"))]
pub(crate) struct AmqpClaimsBasedSecurity(AmqpClaimsBasedSecurityImpl<Fe2o3ClaimsBasedSecurity>);

#[cfg(not(any(feature = "enable-fe2o3-amqp")))]
pub(crate) struct AmqpClaimsBasedSecurity(
    AmqpClaimsBasedSecurityImpl<super::noop::NoopAmqpClaimsBasedSecurity>,
);

impl AmqpClaimsBasedSecurityTrait for AmqpClaimsBasedSecurity {
    async fn authorize_path(
        &self,
        path: &String,
        secret: impl Into<String>,
        expires_on: time::OffsetDateTime,
    ) -> Result<()> {
        self.0 .0.authorize_path(path, secret, expires_on).await?;
        Ok(())
    }

    async fn attach(&self) -> Result<()> {
        self.0 .0.attach().await?;
        Ok(())
    }
}

impl AmqpClaimsBasedSecurity {
    pub(crate) fn new(session: AmqpSession) -> Self {
        #[cfg(any(feature = "enable-fe2o3-amqp"))]
        let session = Fe2o3ClaimsBasedSecurity::new(session.0 .0);
        #[cfg(not(any(feature = "enable-fe2o3-amqp")))]
        let session = NoopAmqpClaimsBasedSecurity::new();

        Self(AmqpClaimsBasedSecurityImpl::new(session))
    }
}
