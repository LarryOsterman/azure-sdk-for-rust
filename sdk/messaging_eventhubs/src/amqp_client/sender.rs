//cspell: words amqp

#[cfg(any(feature = "enable-fe2o3-amqp"))]
use super::fe2o3::sender::Fe2o3AmqpSender;

#[cfg(not(any(feature = "enable-fe2o3-amqp")))]
use super::noop::NoopAmqpSender;

use super::messaging::{AmqpMessage, AmqpSource};
use super::value::{AmqpOrderedMap, AmqpSymbol, AmqpValue};
use crate::amqp_client::{ReceiverSettleMode, SenderSettleMode};
use azure_core::error::Result;

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
    pub fn builder() -> builders::AmqpSenderOptionsBuilder {
        builders::AmqpSenderOptionsBuilder::new()
    }
}

pub(crate) struct AmqpSender {
    #[cfg(feature = "enable-fe2o3-amqp")]
    inner: Fe2o3AmqpSender,

    #[cfg(not(feature = "enable-fe2o3-amqp"))]
    inner: NoopAmqpSender,
}

impl AmqpSender {
    pub(crate) fn new(
        #[cfg(feature = "enable-fe2o3-amqp")] inner: Fe2o3AmqpSender,
        #[cfg(not(feature = "enable-fe2o3-amqp"))] inner: NoopAmqpSender,
    ) -> Self {
        Self { inner }
    }
    pub(crate) fn max_message_size(&self) -> Option<u64> {
        self.inner.max_message_size()
    }
    pub(crate) async fn send(&self, message: AmqpMessage) -> Result<()> {
        Ok(self.inner.send(message).await?)
    }
}

pub mod builders {
    use super::*;

    pub(crate) struct AmqpSenderOptionsBuilder {
        options: AmqpSenderOptions,
    }

    impl AmqpSenderOptionsBuilder {
        pub(super) fn new() -> Self {
            AmqpSenderOptionsBuilder {
                options: AmqpSenderOptions {
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
                },
            }
        }
        pub fn with_name(mut self, name: impl Into<String>) -> Self {
            self.options.name = Some(name.into());
            self
        }
        pub fn with_sender_settle_mode(mut self, sender_settle_mode: SenderSettleMode) -> Self {
            self.options.sender_settle_mode = Some(sender_settle_mode);
            self
        }
        pub fn with_receiver_settle_mode(
            mut self,
            receiver_settle_mode: ReceiverSettleMode,
        ) -> Self {
            self.options.receiver_settle_mode = Some(receiver_settle_mode);
            self
        }
        pub fn with_source(mut self, source: impl Into<AmqpSource>) -> Self {
            self.options.source = Some(source.into());
            self
        }
        pub fn with_offered_capabilities(mut self, offered_capabilities: Vec<AmqpSymbol>) -> Self {
            self.options.offered_capabilities = Some(offered_capabilities);
            self
        }
        pub fn with_desired_capabilities(mut self, desired_capabilities: Vec<AmqpSymbol>) -> Self {
            self.options.desired_capabilities = Some(desired_capabilities);
            self
        }
        pub fn with_properties(mut self, properties: Vec<(&str, &str)>) -> Self {
            let properties_map: AmqpOrderedMap<AmqpSymbol, AmqpValue> = properties
                .into_iter()
                .map(|(k, v)| (AmqpSymbol::from(k), AmqpValue::from(v)))
                .collect();

            self.options.properties = Some(properties_map);
            self
        }
        pub fn with_buffer_size(mut self, buffer_size: usize) -> Self {
            self.options.buffer_size = Some(buffer_size);
            self
        }
        pub fn with_role(mut self, role: AmqpValue) -> Self {
            self.options.role = Some(role);
            self
        }
        pub fn with_initial_delivery_count(mut self, initial_delivery_count: u32) -> Self {
            self.options.initial_delivery_count = Some(initial_delivery_count);
            self
        }
        pub fn with_max_message_size(mut self, max_message_size: u64) -> Self {
            self.options.max_message_size = Some(max_message_size);
            self
        }

        pub fn build(self) -> AmqpSenderOptions {
            self.options
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amqp_sender_options_builder() {
        let builder = AmqpSenderOptions::builder()
            .with_name("name".to_string())
            .with_sender_settle_mode(SenderSettleMode::Unsettled)
            .with_sender_settle_mode(SenderSettleMode::Settled)
            .with_sender_settle_mode(SenderSettleMode::Mixed)
            .with_receiver_settle_mode(ReceiverSettleMode::Second)
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
        assert!(properties.contains_key("key"));
        assert_eq!(
            *properties.get("key").unwrap(),
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
