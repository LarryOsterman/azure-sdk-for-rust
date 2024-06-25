// cspell: words amqp sasl

#[cfg(any(feature = "enable-fe2o3-amqp"))]
use super::fe2o3::management::Fe2o3AmqpManagement;

use super::{
    session::AmqpSession,
    value::{AmqpOrderedMap, AmqpValue},
};
use azure_core::error::Result;
use std::fmt::Debug;

pub(crate) trait AmqpManagementTrait {
    async fn call(
        &self,
        operation_type: impl Into<String>,
        entity: impl Into<String>,
        application_properties: Option<AmqpOrderedMap<String, AmqpValue>>,
    ) -> Result<AmqpOrderedMap<String, AmqpValue>> {
        unimplemented!()
    }
}

#[derive(Debug)]
struct AmqpManagementImpl<T>(T);

impl<T> AmqpManagementImpl<T>
where
    T: AmqpManagementTrait,
{
    pub(crate) fn new(manager: T) -> Self {
        Self(manager)
    }
}

#[derive(Debug)]
#[cfg(any(feature = "enable-fe2o3-amqp"))]
pub struct AmqpManagement(AmqpManagementImpl<Fe2o3AmqpManagement>);

#[cfg(not(any(feature = "enable-fe2o3-amqp")))]
pub struct AmqpManagement(AmqpManagementImpl<super::noop::NoopAmqpManagement>);

impl AmqpManagementTrait for AmqpManagement {
    async fn call(
        &self,
        operation_type: impl Into<String>,
        entity: impl Into<String>,
        application_properties: Option<AmqpOrderedMap<String, AmqpValue>>,
    ) -> Result<AmqpOrderedMap<String, AmqpValue>> {
        self.0
             .0
            .call(operation_type, entity, application_properties)
            .await
    }
}

impl AmqpManagement {
    pub(crate) async fn new(
        session: &AmqpSession,
        client_node_name: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self(AmqpManagementImpl::new(
            #[cfg(any(feature = "enable-fe2o3-amqp"))]
            Fe2o3AmqpManagement::new(&session.0 .0, client_node_name).await?,
            #[cfg(not(any(feature = "enable-fe2o3-amqp")))]
            super::noop::NoopAmqpManagement::new(),
        )))
    }
}
