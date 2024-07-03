// Copyright (c) Microsoft Corp. All Rights Reserved.

// cspell: words amqp servicebus eventhub mgmt

use fe2o3_amqp_types::messaging::annotations::OwnedKey;
use serde_amqp::Value;

use crate::amqp_client::{
    messaging::{
        AmqpAnnotationKey, AmqpAnnotations, AmqpApplicationProperties, AmqpMessage,
        AmqpMessageBody, AmqpMessageHeader, AmqpMessageId, AmqpMessageProperties,
    },
    value::{AmqpOrderedMap, AmqpValue},
};

impl From<fe2o3_amqp_types::messaging::MessageId> for AmqpMessageId {
    fn from(message_id: fe2o3_amqp_types::messaging::MessageId) -> Self {
        match message_id {
            fe2o3_amqp_types::messaging::MessageId::String(message_id) => {
                AmqpMessageId::String(message_id)
            }
            fe2o3_amqp_types::messaging::MessageId::Uuid(message_id) => {
                AmqpMessageId::Uuid(message_id.into())
            }
            fe2o3_amqp_types::messaging::MessageId::Binary(message_id) => {
                AmqpMessageId::Binary(message_id.to_vec())
            }
            fe2o3_amqp_types::messaging::MessageId::Ulong(message_id) => {
                AmqpMessageId::Ulong(message_id)
            }
        }
    }
}

impl From<AmqpMessageId> for fe2o3_amqp_types::messaging::MessageId {
    fn from(message_id: AmqpMessageId) -> Self {
        match message_id {
            AmqpMessageId::String(message_id) => {
                fe2o3_amqp_types::messaging::MessageId::String(message_id)
            }
            AmqpMessageId::Uuid(message_id) => fe2o3_amqp_types::messaging::MessageId::Uuid(
                fe2o3_amqp_types::primitives::Uuid::from(message_id),
            ),
            AmqpMessageId::Binary(message_id) => fe2o3_amqp_types::messaging::MessageId::Binary(
                serde_bytes::ByteBuf::from(message_id),
            ),
            AmqpMessageId::Ulong(message_id) => {
                fe2o3_amqp_types::messaging::MessageId::Ulong(message_id)
            }
        }
    }
}

impl From<fe2o3_amqp_types::messaging::ApplicationProperties>
    for crate::amqp_client::messaging::AmqpApplicationProperties
{
    fn from(application_properties: fe2o3_amqp_types::messaging::ApplicationProperties) -> Self {
        let mut properties = AmqpOrderedMap::<String, AmqpValue>::new();
        for (key, value) in application_properties.0 {
            properties.insert(key, value.into());
        }
        AmqpApplicationProperties(properties)
    }
}

impl From<fe2o3_amqp_types::messaging::Header> for AmqpMessageHeader {
    fn from(header: fe2o3_amqp_types::messaging::Header) -> Self {
        AmqpMessageHeader::builder()
            .with_durable(header.durable)
            .with_priority(header.priority.into())
            .with_time_to_live(header.ttl.unwrap_or(0))
            .with_first_acquirer(header.first_acquirer)
            .with_delivery_count(header.delivery_count)
            .build()
    }
}

impl From<crate::amqp_client::messaging::AmqpAnnotationKey> for OwnedKey {
    fn from(key: AmqpAnnotationKey) -> Self {
        match key {
            AmqpAnnotationKey::Ulong(key) => OwnedKey::Ulong(key),
            AmqpAnnotationKey::Symbol(key) => OwnedKey::Symbol(key.into()),
        }
    }
}

impl From<OwnedKey> for crate::amqp_client::messaging::AmqpAnnotationKey {
    fn from(key: OwnedKey) -> Self {
        match key {
            OwnedKey::Ulong(key) => crate::amqp_client::messaging::AmqpAnnotationKey::Ulong(key),
            OwnedKey::Symbol(key) => {
                crate::amqp_client::messaging::AmqpAnnotationKey::Symbol(key.into())
            }
        }
    }
}

impl From<crate::amqp_client::messaging::AmqpAnnotations>
    for fe2o3_amqp_types::messaging::Annotations
{
    fn from(annotations: AmqpAnnotations) -> Self {
        let mut message_annotations = fe2o3_amqp_types::messaging::Annotations::new();
        for (key, value) in annotations.0 {
            message_annotations.insert(key.into(), value.into());
        }
        message_annotations
    }
}

impl From<fe2o3_amqp_types::messaging::Annotations> for AmqpAnnotations {
    fn from(annotations: fe2o3_amqp_types::messaging::Annotations) -> Self {
        let mut amqp_annotations = AmqpOrderedMap::<AmqpAnnotationKey, AmqpValue>::new();
        for (key, value) in annotations {
            amqp_annotations.insert(key.into(), value.into());
        }
        AmqpAnnotations(amqp_annotations)
    }
}

impl From<fe2o3_amqp_types::messaging::Message<fe2o3_amqp_types::messaging::Body<Value>>>
    for AmqpMessage
{
    fn from(
        message: fe2o3_amqp_types::messaging::Message<fe2o3_amqp_types::messaging::Body<Value>>,
    ) -> Self {
        let mut amqp_message_builder = AmqpMessage::builder();

        if let Some(application_properties) = message.application_properties {
            amqp_message_builder =
                amqp_message_builder.with_application_properties(application_properties.into());
        }

        let body = message.body;
        if body.is_data() {
            let data = body.try_into_data().unwrap();
            let body = AmqpMessageBody::Binary(data.map(|x| x.to_vec()).collect());
            amqp_message_builder = amqp_message_builder.with_body(body);
        } else if body.is_value() {
            let value = body.try_into_value().unwrap();
            amqp_message_builder =
                amqp_message_builder.with_body(AmqpMessageBody::Value(value.into()));
        } else if body.is_sequence() {
            let sequence = body.try_into_sequence().unwrap();
            let body = AmqpMessageBody::Sequence(
                sequence
                    .map(|x| {
                        x.iter()
                            .map(|v| Into::<AmqpValue>::into(v.clone()))
                            .collect()
                    })
                    .collect(),
            );
            amqp_message_builder = amqp_message_builder.with_body(body);
        }

        if let Some(header) = message.header {
            amqp_message_builder = amqp_message_builder.with_header(header.into());
        }

        if let Some(properties) = message.properties {
            amqp_message_builder = amqp_message_builder.with_properties(properties.into());
        }

        if let Some(delivery_annotations) = message.delivery_annotations {
            amqp_message_builder =
                amqp_message_builder.with_delivery_annotations(delivery_annotations.0.into());
        }

        if let Some(message_annotations) = message.message_annotations {
            amqp_message_builder =
                amqp_message_builder.with_message_annotations(message_annotations.0.into());
        }

        if let Some(footer) = message.footer {
            amqp_message_builder = amqp_message_builder.with_footer(footer.0.into());
        }

        amqp_message_builder.build()
    }
}

impl From<fe2o3_amqp_types::messaging::Properties> for AmqpMessageProperties {
    fn from(properties: fe2o3_amqp_types::messaging::Properties) -> Self {
        let mut amqp_message_properties_builder = AmqpMessageProperties::builder();

        if let Some(message_id) = properties.message_id {
            amqp_message_properties_builder =
                amqp_message_properties_builder.with_message_id(message_id.into());
        }
        if let Some(user_id) = properties.user_id {
            amqp_message_properties_builder =
                amqp_message_properties_builder.with_user_id(user_id.to_vec());
        }
        if let Some(to) = properties.to {
            amqp_message_properties_builder = amqp_message_properties_builder.with_to(to);
        }
        if let Some(subject) = properties.subject {
            amqp_message_properties_builder = amqp_message_properties_builder.with_subject(subject);
        }
        if let Some(reply_to) = properties.reply_to {
            amqp_message_properties_builder =
                amqp_message_properties_builder.with_reply_to(reply_to);
        }
        if let Some(correlation_id) = properties.correlation_id {
            amqp_message_properties_builder =
                amqp_message_properties_builder.with_correlation_id(correlation_id.into());
        }
        if let Some(content_type) = properties.content_type {
            amqp_message_properties_builder =
                amqp_message_properties_builder.with_content_type(content_type.into());
        }
        if let Some(content_encoding) = properties.content_encoding {
            amqp_message_properties_builder =
                amqp_message_properties_builder.with_content_encoding(content_encoding.into());
        }
        if let Some(absolute_expiry_time) = properties.absolute_expiry_time {
            amqp_message_properties_builder = amqp_message_properties_builder
                .with_absolute_expiry_time(absolute_expiry_time.into());
        }
        if let Some(creation_time) = properties.creation_time {
            amqp_message_properties_builder =
                amqp_message_properties_builder.with_creation_time(creation_time.into());
        }
        if let Some(group_id) = properties.group_id {
            amqp_message_properties_builder =
                amqp_message_properties_builder.with_group_id(group_id);
        }
        if let Some(group_sequence) = properties.group_sequence {
            amqp_message_properties_builder =
                amqp_message_properties_builder.with_group_sequence(group_sequence);
        }
        if let Some(reply_to_group_id) = properties.reply_to_group_id {
            amqp_message_properties_builder =
                amqp_message_properties_builder.with_reply_to_group_id(reply_to_group_id);
        }
        amqp_message_properties_builder.build()
    }
}
