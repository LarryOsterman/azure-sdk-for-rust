//cspell: words amqp

use crate::amqp_client::{value::AmqpValue, AmqpSender, AmqpSenderOptions};

#[derive(Debug)]
pub(crate) struct Fe2o3AmqpSender {
    session: fe2o3_amqp::session::SessionHandle<()>,
    target: AmqpValue,
    options: Option<AmqpSenderOptions>,
}

impl Fe2o3AmqpSender {
    pub(crate) fn new(
        session: fe2o3_amqp::session::SessionHandle<()>,
        target: AmqpValue,
        options: Option<AmqpSenderOptions>,
    ) -> Self {
        Self {
            session,
            target,
            options,
        }
    }
}

impl AmqpSender for Fe2o3AmqpSender {
    // fn send(&self, _message: AmqpMessage) -> Result<()> {
    //     todo!()
    // }
}
