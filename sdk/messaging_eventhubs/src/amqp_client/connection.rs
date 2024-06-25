// cspell: words amqp sasl

use super::value::{AmqpOrderedMap, AmqpSymbol, AmqpValue};
use azure_core::error::Result;
use log::debug;
use std::fmt::Debug;
use time::Duration;
use url::Url;

pub(crate) struct AmqpConnectionOptions {
    pub(crate) max_frame_size: Option<u32>,
    pub(crate) channel_max: Option<u16>,
    pub(crate) idle_timeout: Option<Duration>,
    pub(crate) outgoing_locales: Option<Vec<String>>,
    pub(crate) incoming_locales: Option<Vec<String>>,
    pub(crate) offered_capabilities: Option<Vec<AmqpSymbol>>,
    pub(crate) desired_capabilities: Option<Vec<AmqpSymbol>>,
    pub(crate) properties: Option<AmqpOrderedMap<AmqpSymbol, AmqpValue>>,
    pub(crate) buffer_size: Option<usize>,
}

impl AmqpConnectionOptions {
    pub(crate) fn builder() -> builders::AmqpConnectionOptionsBuilder {
        builders::AmqpConnectionOptionsBuilder::new()
    }
}

#[allow(dead_code)]
pub(crate) trait AmqpConnectionTrait {
    async fn open(
        &self,
        name: impl Into<String>,
        url: Url,
        options: Option<AmqpConnectionOptions>,
    ) -> Result<()> {
        unimplemented!();
    }
    async fn close(&self) -> Result<()> {
        unimplemented!();
    }
}

#[cfg(any(feature = "enable-fe2o3-amqp"))]
type ConnectionImplementation = super::fe2o3::connection::Fe2o3AmqpConnection;

#[cfg(not(any(feature = "enable-fe2o3-amqp")))]
type ConnectionImplementation = super::noop::NoopAmqpConnection;

#[derive(Debug)]
pub(crate) struct AmqpConnectionImpl<T>(pub(crate) T);

impl<T> AmqpConnectionImpl<T>
where
    T: AmqpConnectionTrait,
{
    pub(crate) fn new(connection: T) -> Self {
        Self(connection)
    }
}

#[derive(Debug)]
pub(crate) struct AmqpConnection(pub(crate) AmqpConnectionImpl<ConnectionImplementation>);

impl AmqpConnectionTrait for AmqpConnection {
    async fn open(
        &self,
        name: impl Into<String>,
        url: Url,
        options: Option<AmqpConnectionOptions>,
    ) -> Result<()> {
        self.0 .0.open(name, url, options).await
    }
    async fn close(&self) -> Result<()> {
        self.0 .0.close().await
    }
}

impl AmqpConnection {
    pub(crate) fn new() -> Self {
        let connection = ConnectionImplementation::new();

        Self(AmqpConnectionImpl::new(connection))
    }
}

pub mod builders {
    use super::*;
    pub(crate) struct AmqpConnectionOptionsBuilder {
        max_frame_size: Option<u32>,
        channel_max: Option<u16>,
        idle_timeout: Option<time::Duration>,
        outgoing_locales: Option<Vec<String>>,
        incoming_locales: Option<Vec<String>>,
        offered_capabilities: Option<Vec<AmqpSymbol>>,
        desired_capabilities: Option<Vec<AmqpSymbol>>,
        properties: Option<AmqpOrderedMap<AmqpSymbol, AmqpValue>>,
        buffer_size: Option<usize>,
    }

    impl AmqpConnectionOptionsBuilder {
        pub fn new() -> Self {
            Self {
                max_frame_size: None,
                channel_max: None,
                idle_timeout: None,
                outgoing_locales: None,
                incoming_locales: None,
                offered_capabilities: None,
                desired_capabilities: None,
                properties: None,
                buffer_size: None,
            }
        }
        pub fn build(&self) -> AmqpConnectionOptions {
            AmqpConnectionOptions {
                max_frame_size: self.max_frame_size,
                channel_max: self.channel_max,
                idle_timeout: self.idle_timeout,
                outgoing_locales: self.outgoing_locales.clone(),
                incoming_locales: self.incoming_locales.clone(),
                offered_capabilities: self.offered_capabilities.clone(),
                desired_capabilities: self.desired_capabilities.clone(),
                properties: self.properties.clone(),
                buffer_size: self.buffer_size,
            }
        }
        pub fn with_max_frame_size(&mut self, max_frame_size: u32) -> &mut Self {
            self.max_frame_size = Some(max_frame_size);
            self
        }
        pub fn with_channel_max(&mut self, channel_max: u16) -> &mut Self {
            self.channel_max = Some(channel_max);
            self
        }
        pub fn with_idle_timeout(&mut self, idle_timeout: Duration) -> &mut Self {
            self.idle_timeout = Some(idle_timeout);
            self
        }
        pub fn with_outgoing_locales(&mut self, outgoing_locales: Vec<String>) -> &mut Self {
            self.outgoing_locales = Some(outgoing_locales);
            self
        }
        pub fn with_incoming_locales(&mut self, incoming_locales: Vec<String>) -> &mut Self {
            self.incoming_locales = Some(incoming_locales);
            self
        }
        pub fn with_offered_capabilities(
            &mut self,
            offered_capabilities: Vec<AmqpSymbol>,
        ) -> &mut Self {
            self.offered_capabilities = Some(offered_capabilities);
            self
        }
        pub fn with_desired_capabilities(
            &mut self,
            desired_capabilities: Vec<AmqpSymbol>,
        ) -> &mut Self {
            self.desired_capabilities = Some(desired_capabilities);
            self
        }
        pub fn with_properties(
            &mut self,
            properties: Vec<(impl Into<AmqpSymbol>, impl Into<AmqpValue>)>,
        ) -> &mut Self {
            let properties_map: AmqpOrderedMap<AmqpSymbol, AmqpValue> = properties
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect();
            self.properties = Some(properties_map);
            self
        }
        pub fn with_buffer_size(&mut self, buffer_size: usize) -> &mut Self {
            self.buffer_size = Some(buffer_size);
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use builders::AmqpConnectionOptionsBuilder;

    use super::*;

    #[test]
    fn test_amqp_connection_builder() {
        let builder = AmqpConnectionOptionsBuilder::new()
            .with_max_frame_size(1024)
            .with_channel_max(16)
            .with_idle_timeout(time::Duration::seconds(60))
            .with_outgoing_locales(vec!["en-US".to_string()])
            .with_incoming_locales(vec!["en-US".to_string()])
            .with_offered_capabilities(vec!["capability".into()])
            .with_desired_capabilities(vec!["capability".into()])
            .with_properties(vec![("key", "value")])
            .with_buffer_size(1024)
            .build();
    }
}
