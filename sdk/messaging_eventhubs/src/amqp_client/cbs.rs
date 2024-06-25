// cspell: words amqp sasl

use azure_core::error::Result;

#[cfg(any(feature = "enable-fe2o3-amqp"))]
use super::fe2o3::cbs::Fe2o3ClaimsBasedSecurity;

#[cfg(not(any(feature = "enable-fe2o3-amqp")))]
use super::noop::NoopAmqpClaimsBasedSecurity;

pub(crate) trait AmqpClaimsBasedSecurityTrait {
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
}

impl AmqpClaimsBasedSecurity {
    pub(crate) fn new(
        #[cfg(any(feature = "enable-fe2o3-amqp"))] inner: Fe2o3ClaimsBasedSecurity,
        #[cfg(not(any(feature = "enable-fe2o3-amqp")))] inner: NoopAmqpClaimsBasedSecurity,
    ) -> Self {
        Self(AmqpClaimsBasedSecurityImpl::new(inner))
    }
}
