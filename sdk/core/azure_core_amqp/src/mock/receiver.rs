// Copyright (c) Microsoft Corporation. All Rights reserved
// Licensed under the MIT license.

use crate::messaging::{AmqpDelivery, AmqpSource};
use crate::mock::delivery::MockAmqpDelivery;
use crate::receiver::{AmqpReceiverApis, AmqpReceiverOptions, ReceiverCreditMode};
use crate::session::AmqpSession;
use async_trait::async_trait;
use azure_core::Result;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct MockAmqpReceiver {
    is_attached: Arc<Mutex<bool>>,
    current_credit_mode: Arc<Mutex<ReceiverCreditMode>>,
    session: Arc<Mutex<Option<AmqpSession>>>,
    source: Arc<Mutex<Option<AmqpSource>>>,
    options: Arc<Mutex<Option<AmqpReceiverOptions>>>,
    delivery_queue: Arc<Mutex<Vec<AmqpDelivery>>>,
}

impl MockAmqpReceiver {
    pub fn new() -> Self {
        Self {
            is_attached: Arc::new(Mutex::new(false)),
            current_credit_mode: Arc::new(Mutex::new(ReceiverCreditMode::default())),
            session: Arc::new(Mutex::new(None)),
            source: Arc::new(Mutex::new(None)),
            options: Arc::new(Mutex::new(None)),
            delivery_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    // Helper methods for testing
    pub fn is_attached(&self) -> bool {
        *self.is_attached.lock().unwrap()
    }

    pub fn get_options(&self) -> Option<AmqpReceiverOptions> {
        self.options.lock().unwrap().clone()
    }

    pub fn queue_delivery(&self, delivery: AmqpDelivery) {
        self.delivery_queue.lock().unwrap().push(delivery);
    }

    pub fn queue_mock_delivery(&self, data: Vec<u8>) {
        let delivery = MockAmqpDelivery::new_with_data(data);
        self.queue_delivery(AmqpDelivery(delivery));
    }
}

#[async_trait]
impl AmqpReceiverApis for MockAmqpReceiver {
    async fn attach(
        &self,
        session: &AmqpSession,
        source: impl Into<AmqpSource> + Send,
        options: Option<AmqpReceiverOptions>,
    ) -> Result<()> {
        let source = source.into();
        *self.is_attached.lock().unwrap() = true;
        *self.session.lock().unwrap() = Some(session.clone());
        *self.source.lock().unwrap() = Some(source);
        *self.options.lock().unwrap() = options;
        Ok(())
    }

    async fn detach(self) -> Result<()> {
        *self.is_attached.lock().unwrap() = false;
        Ok(())
    }

    async fn set_credit_mode(&self, credit_mode: ReceiverCreditMode) -> Result<()> {
        *self.current_credit_mode.lock().unwrap() = credit_mode;
        Ok(())
    }

    async fn credit_mode(&self) -> Result<ReceiverCreditMode> {
        Ok(self.current_credit_mode.lock().unwrap().clone())
    }

    async fn receive_delivery(&self) -> Result<AmqpDelivery> {
        let mut queue = self.delivery_queue.lock().unwrap();
        if queue.is_empty() {
            // Create a default delivery if none are queued
            Ok(AmqpDelivery(MockAmqpDelivery::new()))
        } else {
            Ok(queue.remove(0))
        }
    }

    async fn accept_delivery(&self, _delivery: &AmqpDelivery) -> Result<()> {
        Ok(())
    }

    async fn reject_delivery(&self, _delivery: &AmqpDelivery) -> Result<()> {
        Ok(())
    }

    async fn release_delivery(&self, _delivery: &AmqpDelivery) -> Result<()> {
        Ok(())
    }
}
