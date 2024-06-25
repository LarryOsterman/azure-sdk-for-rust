// cspell: words amqp sasl

#[cfg(any(feature = "enable-fe2o3-amqp"))]
use super::fe2o3::session::Fe2o3AmqpSession;

use super::{
    connection::AmqpConnection,
    messaging::AmqpTarget,
    sender::{AmqpSender, AmqpSenderOptions},
    value::{AmqpOrderedMap, AmqpSymbol, AmqpValue},
};
use azure_core::error::Result;
use serde::de;
use std::fmt::Debug;

#[derive(Debug)]
pub(crate) struct AmqpSessionOptions {
    pub(crate) next_outgoing_id: Option<u32>,
    pub(crate) incoming_window: Option<u32>,
    pub(crate) outgoing_window: Option<u32>,
    pub(crate) handle_max: Option<u32>,
    pub(crate) offered_capabilities: Option<Vec<AmqpSymbol>>,
    pub(crate) desired_capabilities: Option<Vec<AmqpSymbol>>,
    pub(crate) properties: Option<AmqpOrderedMap<AmqpSymbol, AmqpValue>>,
    pub(crate) buffer_size: Option<usize>,
}

impl AmqpSessionOptions {
    pub fn builder() -> builders::AmqpSessionOptionsBuilder {
        builders::AmqpSessionOptionsBuilder::new()
    }
}

#[allow(dead_code)]
pub(crate) trait AmqpSessionTrait {
    async fn begin(
        &self,
        connection: &AmqpConnection,
        options: Option<AmqpSessionOptions>,
    ) -> Result<()> {
        unimplemented!()
    }
    async fn end(&self) -> Result<()> {
        unimplemented!()
    }
    async fn create_sender(
        &self,
        target: impl Into<AmqpTarget>,
        options: Option<AmqpSenderOptions>,
    ) -> Result<AmqpSender> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub(crate) struct AmqpSessionImpl<T>(pub(crate) T);

impl<T> AmqpSessionImpl<T>
where
    T: AmqpSessionTrait,
{
    pub(crate) fn new(session: T) -> Self {
        Self(session)
    }
}

#[cfg(any(feature = "enable-fe2o3-amqp"))]
type SessionImplementation = super::fe2o3::session::Fe2o3AmqpSession;

#[cfg(not(any(feature = "enable-fe2o3-amqp")))]
type SessionImplementation = super::noop::NoopAmqpSession;

#[derive(Debug)]
pub(crate) struct AmqpSession(pub(crate) AmqpSessionImpl<SessionImplementation>);

impl AmqpSessionTrait for AmqpSession {
    async fn begin(
        &self,
        connection: &AmqpConnection,
        options: Option<AmqpSessionOptions>,
    ) -> Result<()> {
        self.0 .0.begin(&connection, options).await
    }

    async fn end(&self) -> Result<()> {
        self.0 .0.end().await
    }
    async fn create_sender(
        &self,
        target: impl Into<AmqpTarget>,
        options: Option<AmqpSenderOptions>,
    ) -> Result<AmqpSender> {
        Ok(self.0 .0.create_sender(target, options).await?)
    }
}

impl AmqpSession {
    pub(crate) fn new() -> Self {
        let inner = SessionImplementation::new();
        Self(AmqpSessionImpl::new(inner))
    }
}

pub mod builders {
    use super::*;

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
}
#[cfg(test)]
mod tests {
    use super::builders::*;
    use super::*;

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
        assert!(properties.contains_key("key"));
        assert_eq!(
            *properties.get("key").unwrap(),
            AmqpValue::String("value".to_string())
        );

        assert_eq!(session_options.buffer_size, Some(1024));
    }
}
