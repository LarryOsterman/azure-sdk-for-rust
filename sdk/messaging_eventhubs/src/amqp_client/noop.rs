// cspell: words amqp

use std::sync::Arc;

#[derive(Debug)]
struct NoopAmqpConnectionBuilder;

pub(crate) fn noop_amqp_connection_builder() -> Arc<dyn crate::amqp_client::AmqpConnectionBuilder> {
    Arc::new(NoopAmqpConnectionBuilder)
}
impl crate::amqp_client::AmqpConnectionBuilder for NoopAmqpConnectionBuilder {
    async fn open(self) -> std::sync::Arc<dyn super::AmqpConnection> {
        todo!()
    }

    fn with_uri(self, uri: impl Into<String>) -> Self {
        todo!()
    }

    fn with_container_id(self, container_id: impl Into<String>) -> Self {
        todo!()
    }

    fn with_max_frame_size(self, max_frame_size: u32) -> Self {
        todo!()
    }

    fn with_channel_max(self, channel_max: u16) -> Self {
        todo!()
    }

    fn with_idle_timeout(self, idle_timeout: time::convert::Millisecond) -> Self {
        todo!()
    }

    fn with_outgoing_locales(self, outgoing_locales: Vec<String>) -> Self {
        todo!()
    }

    fn with_incoming_locales(self, incoming_locales: Vec<String>) -> Self {
        todo!()
    }

    fn with_offered_capabilities(self, offered_capabilities: Vec<String>) -> Self {
        todo!()
    }

    fn with_desired_capabilities(self, desired_capabilities: Vec<String>) -> Self {
        todo!()
    }

    fn with_properties(self, properties: Vec<(String, String)>) -> Self {
        todo!()
    }

    fn with_buffer_size(self, buffer_size: usize) -> Self {
        todo!()
    }
}
