// Copyright (c) Microsoft Corporation. All Rights reserved
// Licensed under the MIT license.

use crate::messaging::{AmqpDeliveryApis, AmqpMessage, DeliveryNumber, DeliveryTag};

pub struct MockAmqpDelivery {
    message: AmqpMessage,
    delivery_id: DeliveryNumber,
    delivery_tag: DeliveryTag,
    message_format: Option<u32>,
}

impl Default for MockAmqpDelivery {
    fn default() -> Self {
        Self::new()
    }
}

impl MockAmqpDelivery {
    pub fn new() -> Self {
        Self {
            message: AmqpMessage::default(),
            delivery_id: 1,
            delivery_tag: vec![1, 2, 3],
            message_format: Some(0),
        }
    }

    pub fn new_with_data(data: Vec<u8>) -> Self {
        Self {
            message: AmqpMessage::from(data),
            delivery_id: 1,
            delivery_tag: vec![1, 2, 3],
            message_format: Some(0),
        }
    }

    pub fn with_id(mut self, id: DeliveryNumber) -> Self {
        self.delivery_id = id;
        self
    }

    pub fn with_tag(mut self, tag: DeliveryTag) -> Self {
        self.delivery_tag = tag;
        self
    }

    pub fn with_format(mut self, format: Option<u32>) -> Self {
        self.message_format = format;
        self
    }
}

impl AmqpDeliveryApis for MockAmqpDelivery {
    fn message(&self) -> &AmqpMessage {
        &self.message
    }

    fn delivery_id(&self) -> DeliveryNumber {
        self.delivery_id
    }

    fn delivery_tag(&self) -> &DeliveryTag {
        &self.delivery_tag
    }

    fn message_format(&self) -> &Option<u32> {
        &self.message_format
    }

    fn into_message(self) -> AmqpMessage {
        self.message
    }
}
