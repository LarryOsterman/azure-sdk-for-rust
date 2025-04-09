// Copyright (c) Microsoft Corporation. All Rights reserved
// Licensed under the MIT license.

use crate::messaging::{AmqpMessage, AmqpTarget};
use crate::sender::{AmqpSendOptions, AmqpSendOutcome, AmqpSenderApis, AmqpSenderOptions};
use crate::session::AmqpSession;
use azure_core::Result;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct MockAmqpSender {
    is_attached: Arc<Mutex<bool>>,
    session: Arc<Mutex<Option<AmqpSession>>>,
    name: Arc<Mutex<Option<String>>>,
    target: Arc<Mutex<Option<AmqpTarget>>>,
    options: Arc<Mutex<Option<AmqpSenderOptions>>>,
    sent_messages: Arc<Mutex<Vec<AmqpMessage>>>,
    message_size: Arc<Mutex<Option<u64>>>,
    next_outcome: Arc<Mutex<Option<AmqpSendOutcome>>>,
}

impl MockAmqpSender {
    pub fn new() -> Self {
        Self {
            is_attached: Arc::new(Mutex::new(false)),
            session: Arc::new(Mutex::new(None)),
            name: Arc::new(Mutex::new(None)),
            target: Arc::new(Mutex::new(None)),
            options: Arc::new(Mutex::new(None)),
            sent_messages: Arc::new(Mutex::new(Vec::new())),
            message_size: Arc::new(Mutex::new(Some(1024))),
            next_outcome: Arc::new(Mutex::new(Some(AmqpSendOutcome::Accepted))),
        }
    }

    // Helper methods for testing
    pub fn is_attached(&self) -> bool {
        *self.is_attached.lock().unwrap()
    }

    pub fn get_sent_messages(&self) -> Vec<AmqpMessage> {
        self.sent_messages.lock().unwrap().clone()
    }

    pub fn set_max_message_size(&self, size: Option<u64>) {
        *self.message_size.lock().unwrap() = size;
    }

    pub fn set_next_outcome(&self, outcome: AmqpSendOutcome) {
        *self.next_outcome.lock().unwrap() = Some(outcome);
    }
}

impl AmqpSenderApis for MockAmqpSender {
    async fn attach(
        &self,
        session: &AmqpSession,
        name: String,
        target: impl Into<AmqpTarget>,
        options: Option<AmqpSenderOptions>,
    ) -> Result<()> {
        let target = target.into();
        *self.is_attached.lock().unwrap() = true;
        *self.session.lock().unwrap() = Some(session.clone());
        *self.name.lock().unwrap() = Some(name);
        *self.target.lock().unwrap() = Some(target);
        *self.options.lock().unwrap() = options;
        Ok(())
    }

    async fn detach(self) -> Result<()> {
        *self.is_attached.lock().unwrap() = false;
        Ok(())
    }

    async fn max_message_size(&self) -> Result<Option<u64>> {
        Ok(*self.message_size.lock().unwrap())
    }

    async fn send(
        &self,
        message: impl Into<AmqpMessage>,
        _options: Option<AmqpSendOptions>,
    ) -> Result<AmqpSendOutcome> {
        let message = message.into();
        self.sent_messages.lock().unwrap().push(message);

        // Return the configured outcome or default to Accepted
        Ok(self
            .next_outcome
            .lock()
            .unwrap()
            .as_ref()
            .cloned()
            .unwrap_or(AmqpSendOutcome::Accepted))
    }
}
