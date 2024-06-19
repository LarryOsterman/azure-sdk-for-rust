// cspell: words amqp sasl

#[cfg(any(feature = "enable-fe2o3-amqp"))]
mod fe2o3;

#[cfg(not(any(feature = "enable-fe2o3-amqp")))]
mod noop;

pub mod error;
pub mod messaging;
pub mod value;

use crate::amqp_client::{
    messaging::AmqpSource,
    value::{AmqpOrderedMap, AmqpSymbol, AmqpValue},
};
use async_trait::async_trait;
use azure_core::error::Result;
use std::fmt::Debug;
use time::Duration;
use tracing::debug;

#[cfg(any(feature = "enable-fe2o3-amqp"))]
use fe2o3::error::AmqpOpenError;

#[cfg(any(feature = "enable-fe2o3-amqp"))]
use fe2o3::connection::Fe2o3AmqpConnection;

pub struct AmqpConnectionBuilder {
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

impl AmqpConnectionBuilder {
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
    pub async fn open(
        self,
        id: impl Into<String>,
        url: url::Url,
    ) -> Result<Box<dyn AmqpConnection>> {
        #[cfg(any(feature = "enable-fe2o3-amqp"))]
        {
            // All AMQP clients have a similar set of options.
            let mut builder = fe2o3_amqp::Connection::builder()
                .sasl_profile(fe2o3_amqp::sasl_profile::SaslProfile::Anonymous)
                .alt_tls_establishment(true)
                .container_id(id)
                .max_frame_size(65536);

            if self.max_frame_size.is_some() {
                builder = builder.max_frame_size(self.max_frame_size.unwrap());
            }
            if self.channel_max.is_some() {
                builder = builder.channel_max(self.channel_max.unwrap());
            }
            if self.idle_timeout.is_some() {
                builder =
                    builder.idle_time_out(self.idle_timeout.unwrap().whole_milliseconds() as u32);
            }
            if self.outgoing_locales.is_some() {
                for locale in self.outgoing_locales.as_ref().unwrap() {
                    builder = builder.add_outgoing_locales(locale.as_str());
                }
            }
            if self.incoming_locales.is_some() {
                for locale in self.incoming_locales.as_ref().unwrap() {
                    builder = builder.add_incoming_locales(locale.as_str());
                }
            }
            if self.offered_capabilities.is_some() {
                for capability in self.offered_capabilities.unwrap() {
                    let capability: fe2o3_amqp_types::primitives::Symbol = capability.into();
                    builder = builder.add_offered_capabilities(capability);
                }
            }
            if self.desired_capabilities.is_some() {
                for capability in self.desired_capabilities.unwrap() {
                    let capability: fe2o3_amqp_types::primitives::Symbol = capability.into();
                    builder = builder.add_desired_capabilities(capability);
                }
            }
            if self.properties.is_some() {
                let mut fields = fe2o3_amqp::types::definitions::Fields::new();
                for property in self.properties.unwrap().iter() {
                    debug!("Property: {:?}, Value: {:?}", property.0, property.1);
                    let k: fe2o3_amqp_types::primitives::Symbol = property.0.into();
                    let v: fe2o3_amqp_types::primitives::Value = property.1.into();
                    debug!("Property2: {:?}, Value: {:?}", k, v);

                    fields.insert(k, v);
                }
                builder = builder.properties(fields);
            }
            if self.buffer_size.is_some() {
                builder = builder.buffer_size(self.buffer_size.unwrap());
            }

            Ok(Box::new(Fe2o3AmqpConnection::new(
                builder.open(url).await.map_err(AmqpOpenError::from)?,
            )))
        }
        #[cfg(not(any(feature = "enable-fe2o3-amqp")))]
        {
            Ok(Arc::new(noop::NoopAmqpConnection))
        }
    }
    pub fn with_max_frame_size(self, max_frame_size: u32) -> Self {
        Self {
            max_frame_size: Some(max_frame_size),
            ..self
        }
    }
    pub fn with_channel_max(self, channel_max: u16) -> Self {
        Self {
            channel_max: Some(channel_max),
            ..self
        }
    }
    pub fn with_idle_timeout(self, idle_timeout: Duration) -> Self {
        Self {
            idle_timeout: Some(idle_timeout),
            ..self
        }
    }
    pub fn with_outgoing_locales(self, outgoing_locales: Vec<String>) -> Self {
        Self {
            outgoing_locales: Some(outgoing_locales),
            ..self
        }
    }
    pub fn with_incoming_locales(self, incoming_locales: Vec<String>) -> Self {
        Self {
            incoming_locales: Some(incoming_locales),
            ..self
        }
    }
    pub fn with_offered_capabilities(self, offered_capabilities: Vec<AmqpSymbol>) -> Self {
        Self {
            offered_capabilities: Some(offered_capabilities),
            ..self
        }
    }
    pub fn with_desired_capabilities(self, desired_capabilities: Vec<AmqpSymbol>) -> Self {
        Self {
            desired_capabilities: Some(desired_capabilities),
            ..self
        }
    }
    pub fn with_properties(
        self,
        properties: Vec<(impl Into<AmqpSymbol>, impl Into<AmqpValue>)>,
    ) -> Self {
        let properties_map: AmqpOrderedMap<AmqpSymbol, AmqpValue> = properties
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        Self {
            properties: Some(properties_map),
            ..self
        }
    }
    pub fn with_buffer_size(self, buffer_size: usize) -> Self {
        Self {
            buffer_size: Some(buffer_size),
            ..self
        }
    }
}

pub(crate) struct AmqpSessionOptions {
    next_outgoing_id: Option<u32>,
    incoming_window: Option<u32>,
    outgoing_window: Option<u32>,
    handle_max: Option<u32>,
    offered_capabilities: Option<Vec<AmqpSymbol>>,
    desired_capabilities: Option<Vec<AmqpSymbol>>,
    properties: Option<AmqpOrderedMap<AmqpSymbol, AmqpValue>>,
    buffer_size: Option<usize>,
}

impl AmqpSessionOptions {
    pub fn builder() -> AmqpSessionOptionsBuilder {
        AmqpSessionOptionsBuilder::new()
    }
}

pub(crate) struct AmqpSessionOptionsBuilder {
    next_outgoing_id: Option<u32>,
    incoming_window: Option<u32>,
    outgoing_window: Option<u32>,
    handle_max: Option<u32>,
    offered_capabilities: Option<Vec<AmqpSymbol>>,
    desired_capabilities: Option<Vec<AmqpSymbol>>,
    properties: Option<AmqpOrderedMap<AmqpSymbol, AmqpValue>>,
    buffer_size: Option<usize>,
}

impl AmqpSessionOptionsBuilder {
    pub fn new() -> Self {
        Self {
            next_outgoing_id: None,
            incoming_window: None,
            outgoing_window: None,
            handle_max: None,
            offered_capabilities: None,
            desired_capabilities: None,
            properties: None,
            buffer_size: None,
        }
    }
    pub fn build(self) -> AmqpSessionOptions {
        AmqpSessionOptions {
            next_outgoing_id: self.next_outgoing_id,
            incoming_window: self.incoming_window,
            outgoing_window: self.outgoing_window,
            handle_max: self.handle_max,
            offered_capabilities: self.offered_capabilities,
            desired_capabilities: self.desired_capabilities,
            properties: self.properties,
            buffer_size: self.buffer_size,
        }
    }
    pub fn with_next_outgoing_id(self, next_outgoing_id: u32) -> Self {
        Self {
            next_outgoing_id: Some(next_outgoing_id),
            ..self
        }
    }
    pub fn with_incoming_window(self, incoming_window: u32) -> Self {
        Self {
            incoming_window: Some(incoming_window),
            ..self
        }
    }
    pub fn with_outgoing_window(self, outgoing_window: u32) -> Self {
        Self {
            outgoing_window: Some(outgoing_window),
            ..self
        }
    }
    pub fn with_handle_max(self, handle_max: u32) -> Self {
        Self {
            handle_max: Some(handle_max),
            ..self
        }
    }
    pub fn with_offered_capabilities(self, offered_capabilities: Vec<AmqpSymbol>) -> Self {
        Self {
            offered_capabilities: Some(offered_capabilities),
            ..self
        }
    }
    pub fn with_desired_capabilities(self, desired_capabilities: Vec<AmqpSymbol>) -> Self {
        Self {
            desired_capabilities: Some(desired_capabilities),
            ..self
        }
    }
    pub fn with_properties(self, properties: Vec<(&str, &str)>) -> Self {
        let properties_map: AmqpOrderedMap<AmqpSymbol, AmqpValue> = properties
            .into_iter()
            .map(|(k, v)| (AmqpSymbol::from(k), AmqpValue::from(v)))
            .collect();
        Self {
            properties: Some(properties_map),
            ..self
        }
    }
    pub fn with_buffer_size(self, buffer_size: usize) -> Self {
        Self {
            buffer_size: Some(buffer_size),
            ..self
        }
    }
}

#[derive(Debug)]
pub(crate) struct AmqpSenderOptions {
    name: Option<String>,
    sender_settle_mode: Option<SenderSettleMode>,
    receiver_settle_mode: Option<ReceiverSettleMode>,
    source: Option<AmqpSource>,
    offered_capabilities: Option<Vec<AmqpSymbol>>,
    desired_capabilities: Option<Vec<AmqpSymbol>>,
    properties: Option<AmqpOrderedMap<AmqpSymbol, AmqpValue>>,
    buffer_size: Option<usize>,
    role: Option<AmqpValue>,
    initial_delivery_count: Option<u32>,
    max_message_size: Option<u64>,
}

impl AmqpSenderOptions {
    pub fn builder() -> AmqpSenderOptionsBuilder {
        AmqpSenderOptionsBuilder::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
enum SenderSettleMode {
    Unsettled = 0,
    Settled = 1,
    Mixed = 2,
}

#[derive(Debug, Clone, PartialEq)]
enum ReceiverSettleMode {
    First = 0,
    Second = 1,
}

pub(crate) struct AmqpSenderOptionsBuilder {
    name: Option<String>,
    sender_settle_mode: Option<SenderSettleMode>,
    receiver_settle_mode: Option<ReceiverSettleMode>,
    source: Option<AmqpSource>,
    offered_capabilities: Option<Vec<AmqpSymbol>>,
    desired_capabilities: Option<Vec<AmqpSymbol>>,
    properties: Option<AmqpOrderedMap<AmqpSymbol, AmqpValue>>,
    buffer_size: Option<usize>,
    role: Option<AmqpValue>,
    initial_delivery_count: Option<u32>,
    max_message_size: Option<u64>,
}

impl AmqpSenderOptionsBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            sender_settle_mode: None,
            receiver_settle_mode: None,
            source: None,
            offered_capabilities: None,
            desired_capabilities: None,
            properties: None,
            buffer_size: None,
            role: None,
            initial_delivery_count: None,
            max_message_size: None,
        }
    }
    pub fn with_name(self, name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..self
        }
    }
    pub fn with_sender_settle_mode(self, sender_settle_mode: SenderSettleMode) -> Self {
        Self {
            sender_settle_mode: Some(sender_settle_mode),
            ..self
        }
    }
    pub fn with_receiver_settle_mode(self, receiver_settle_mode: ReceiverSettleMode) -> Self {
        Self {
            receiver_settle_mode: Some(receiver_settle_mode),
            ..self
        }
    }
    pub fn with_source(self, source: AmqpSource) -> Self {
        Self {
            source: Some(source),
            ..self
        }
    }
    pub fn with_offered_capabilities(self, offered_capabilities: Vec<AmqpSymbol>) -> Self {
        Self {
            offered_capabilities: Some(offered_capabilities),
            ..self
        }
    }
    pub fn with_desired_capabilities(self, desired_capabilities: Vec<AmqpSymbol>) -> Self {
        Self {
            desired_capabilities: Some(desired_capabilities),
            ..self
        }
    }
    pub fn with_properties(self, properties: Vec<(&str, &str)>) -> Self {
        let properties_map: AmqpOrderedMap<AmqpSymbol, AmqpValue> = properties
            .into_iter()
            .map(|(k, v)| (AmqpSymbol::from(k), AmqpValue::from(v)))
            .collect();
        Self {
            properties: Some(properties_map),
            ..self
        }
    }
    pub fn with_buffer_size(self, buffer_size: usize) -> Self {
        Self {
            buffer_size: Some(buffer_size),
            ..self
        }
    }
    pub fn with_role(self, role: AmqpValue) -> Self {
        Self {
            role: Some(role),
            ..self
        }
    }
    pub fn with_initial_delivery_count(self, initial_delivery_count: u32) -> Self {
        Self {
            initial_delivery_count: Some(initial_delivery_count),
            ..self
        }
    }
    pub fn with_max_message_size(self, max_message_size: u64) -> Self {
        Self {
            max_message_size: Some(max_message_size),
            ..self
        }
    }

    pub fn build(self) -> AmqpSenderOptions {
        AmqpSenderOptions {
            name: self.name,
            sender_settle_mode: self.sender_settle_mode,
            receiver_settle_mode: self.receiver_settle_mode,
            source: self.source,
            offered_capabilities: self.offered_capabilities,
            desired_capabilities: self.desired_capabilities,
            properties: self.properties,
            buffer_size: self.buffer_size,
            role: self.role,
            initial_delivery_count: self.initial_delivery_count,
            max_message_size: self.max_message_size,
        }
    }
}

#[async_trait]
pub trait AmqpConnection: Send + Sync + Debug {
    async fn close(&self) -> Result<()>;
    async fn create_session(&self, options: AmqpSessionOptions) -> Result<Box<dyn AmqpSession>>;
    async fn create_claims_based_security(&self) -> Result<Box<dyn AmqpClaimsBasedSecurity>>;
}

#[async_trait]
pub trait AmqpSession: Send + Sync + Debug {
    //    async fn end_with_error(&self, error: impl Into<String>) -> Result<()>;
    async fn end(&self) -> Result<()>;
    async fn create_sender(
        &self,
        target: AmqpValue,
        options: Option<AmqpSenderOptions>,
    ) -> Result<Box<dyn AmqpSender>>;
    async fn create_management(&self, client_node_address: &str)
        -> Result<Box<dyn AmqpManagement>>;
}

#[async_trait]
pub trait AmqpClaimsBasedSecurity: Send + Sync + Debug {
    async fn authorize_path(&self, path: &str, secret: &str, expires_on: i64) -> Result<()>;
}

#[async_trait]
pub trait AmqpManagement: Send + Sync + Debug {
    async fn call(
        &self,
        operation_type: &str,
        entity: &str,
        application_properties: Option<AmqpOrderedMap<String, AmqpValue>>,
    ) -> Result<AmqpOrderedMap<String, AmqpValue>>;
}

#[async_trait]
pub trait AmqpSender: Send + Sync + Debug {}

#[cfg(test)]
mod tests {

    use fe2o3_amqp_types::messaging::IntoBody;

    use super::*;

    #[test]
    fn test_amqp_connection_builder() {
        let builder = AmqpConnectionBuilder::new()
            .with_max_frame_size(1024)
            .with_channel_max(16)
            .with_idle_timeout(time::Duration::seconds(60))
            .with_outgoing_locales(vec!["en-US".to_string()])
            .with_incoming_locales(vec!["en-US".to_string()])
            .with_offered_capabilities(vec!["capability".into()])
            .with_desired_capabilities(vec!["capability".into()])
            .with_properties(vec![("key", "value")])
            .with_buffer_size(1024);

        let _connection = builder.open("id", url::Url::parse("amqp://localhost").unwrap());
    }

    #[test]
    fn test_amqp_session_options_builder() {
        let builder = AmqpSessionOptions::builder()
            .with_next_outgoing_id(1)
            .with_incoming_window(1)
            .with_outgoing_window(1)
            .with_handle_max(1)
            .with_offered_capabilities(vec!["capability".into()])
            .with_desired_capabilities(vec!["capability".into()])
            .with_properties(vec![("key", "value")])
            .with_buffer_size(1024);

        let session_options = builder.build();
        assert_eq!(session_options.next_outgoing_id, Some(1));
        assert_eq!(session_options.incoming_window, Some(1));
        assert_eq!(session_options.outgoing_window, Some(1));
        assert_eq!(session_options.handle_max, Some(1));
        assert_eq!(
            session_options.offered_capabilities,
            Some(vec!["capability".into()])
        );
        assert_eq!(
            session_options.desired_capabilities,
            Some(vec!["capability".into()])
        );
        assert!(session_options.properties.is_some());
        let properties = session_options.properties.clone().unwrap();
        assert!(properties.contains_key("key".into()));
        assert_eq!(
            *properties.get("key".into()).unwrap(),
            AmqpValue::String("value".to_string())
        );

        assert_eq!(session_options.buffer_size, Some(1024));
    }

    #[test]
    fn test_amqp_sender_options_builder() {
        let builder = AmqpSenderOptions::builder()
            .with_name("name".to_string())
            .with_sender_settle_mode(SenderSettleMode::Mixed)
            .with_receiver_settle_mode(ReceiverSettleMode::First)
            .with_source(AmqpSource::builder().with_address("address").build())
            .with_offered_capabilities(vec!["capability".into()])
            .with_desired_capabilities(vec!["capability".into()])
            .with_properties(vec![("key", "value")])
            .with_buffer_size(1024)
            .with_initial_delivery_count(27)
            .with_max_message_size(1024)
            .with_role(AmqpValue::String("role".to_string()));

        let sender_options = builder.build();
        assert_eq!(sender_options.name, Some("name".to_string()));
        assert_eq!(
            sender_options.sender_settle_mode,
            Some(SenderSettleMode::Mixed)
        );
        assert_eq!(
            sender_options.receiver_settle_mode,
            Some(ReceiverSettleMode::First)
        );
        assert_eq!(
            sender_options.offered_capabilities,
            Some(vec!["capability".into()])
        );
        assert_eq!(
            sender_options.desired_capabilities,
            Some(vec!["capability".into()])
        );
        assert!(sender_options.properties.is_some());
        let properties = sender_options.properties.clone().unwrap();
        assert!(properties.contains_key("key".into()));
        assert_eq!(
            *properties.get("key".into()).unwrap(),
            AmqpValue::String("value".to_string())
        );

        assert_eq!(sender_options.initial_delivery_count, Some(27));
        assert_eq!(sender_options.max_message_size, Some(1024));

        assert_eq!(sender_options.buffer_size, Some(1024));
        assert_eq!(
            sender_options.role,
            Some(AmqpValue::String("role".to_string()))
        );
    }
}
