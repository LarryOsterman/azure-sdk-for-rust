//cspell: words amqp

use crate::amqp_client::messaging::{AmqpMessage, AmqpTarget};
use crate::amqp_client::sender::AmqpSenderOptions;
use azure_core::error::Result;

#[derive(Debug)]
pub(crate) struct Fe2o3AmqpSender {
    session: fe2o3_amqp::session::SessionHandle<()>,
    target: AmqpTarget,
    options: Option<AmqpSenderOptions>,
}

impl Fe2o3AmqpSender {
    pub(crate) fn new(
        session: fe2o3_amqp::session::SessionHandle<()>,
        target: AmqpTarget,
        options: Option<AmqpSenderOptions>,
    ) -> Self {
        Self {
            session,
            target,
            options,
        }
    }
    pub(crate) fn max_message_size(&self) -> Option<u64> {
        todo!()
    }

    pub(crate) async fn send(&self, _message: AmqpMessage) -> Result<()> {
        todo!()
    }
}
