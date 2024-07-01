// cspell: words amqp eventhub

#![recursion_limit = "128"]

pub mod amqp_client;
pub(crate) mod common;
pub mod consumer;
pub mod error;
pub mod producer;

mod models {
    use fe2o3_amqp_types::messaging::properties;

    use crate::amqp_client::{
        messaging::{AmqpMessage, AmqpMessageProperties},
        value::AmqpValue,
    };
    use std::collections::HashMap;

    #[derive(Debug)]
    pub struct EventHubProperties {
        pub name: String,
        pub created_on: std::time::SystemTime,
        pub partition_ids: Vec<String>,
    }

    /// Represents the properties of an Event Hub partition.
    ///
    /// This struct provides detailed information about a specific partition within an Event Hub, including its unique identifier, the Event Hub it belongs to, sequence numbers for events, and more.
    ///
    /// # Fields
    ///
    /// - `id`: A `String` representing the unique identifier of the partition.
    /// - `eventhub`: A `String` representing the name of the Event Hub this partition belongs to.
    /// - `beginning_sequence_number`: An `i64` representing the sequence number of the earliest event that can be received from this partition.
    /// - `last_enqueued_sequence_number`: An `i64` representing the sequence number of the latest event that has been enqueued in this partition.
    /// - `last_enqueued_offset`: A `String` representing the offset of the latest event that has been enqueued in this partition. This can be used to start receiving from this event onwards.
    /// - `last_enqueued_time_utc`: A `std::time::SystemTime` representing the UTC time when the last event was enqueued in this partition.
    /// - `is_empty`: A `bool` indicating whether the partition is empty (i.e., contains no events).
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// use azure_messaging_eventhubs::models::EventHubPartitionProperties;
    /// use std::time::{SystemTime, UNIX_EPOCH};
    ///
    /// let partition_properties = EventHubPartitionProperties {
    ///     id: "0".to_string(),
    ///     eventhub: "example-hub".to_string(),
    ///     beginning_sequence_number: 0,
    ///     last_enqueued_sequence_number: 100,
    ///     last_enqueued_offset: "12345".to_string(),
    ///     last_enqueued_time_utc: SystemTime::now(),
    ///     is_empty: false,
    /// };
    /// ```
    ///
    /// Note: Replace `crate::models::EventHubPartitionProperties` with the actual path to `EventHubPartitionProperties` in your project.
    #[derive(Debug)]
    pub struct EventHubPartitionProperties {
        pub id: String,
        pub eventhub: String,
        pub beginning_sequence_number: i64,
        pub last_enqueued_sequence_number: i64,
        pub last_enqueued_offset: String,
        pub last_enqueued_time_utc: std::time::SystemTime,
        pub is_empty: bool,
    }

    #[derive(Debug)]
    pub struct EventData {
        body: Vec<u8>,
        content_type: Option<String>,
        correlation_id: Option<AmqpValue>,
        message_id: Option<String>,
        properties: HashMap<String, AmqpValue>,
    }

    impl EventData {
        pub fn builder() -> builders::EventDataBuilder {
            builders::EventDataBuilder::new()
        }

        pub fn new() -> Self {
            Self {
                body: Vec::new(),
                content_type: None,
                correlation_id: None,
                message_id: None,
                properties: HashMap::new(),
            }
        }

        // pub fn set_body(&mut self, body: Vec<u8>) -> &mut Self {
        //     self.body = body;
        //     self
        // }

        // pub fn set_content_type(&mut self, content_type: impl Into<String>) -> &mut Self {
        //     self.content_type = Some(content_type.into());
        //     self
        // }

        // pub fn set_correlation_id(&mut self, correlation_id: AmqpValue) -> &mut Self {
        //     self.correlation_id = Some(correlation_id);
        //     self
        // }

        // pub fn set_message_id(&mut self, message_id: impl Into<String>) -> &mut Self {
        //     self.message_id = Some(message_id.into());
        //     self
        // }

        // pub fn set_property(&mut self, key: impl Into<String>, value: AmqpValue) -> &mut Self {
        //     self.properties.insert(key.into(), value);
        //     self
        // }
    }

    impl From<Vec<u8>> for EventData {
        fn from(body: Vec<u8>) -> Self {
            Self {
                body,
                content_type: None,
                correlation_id: None,
                message_id: None,
                properties: HashMap::new(),
            }
        }
    }

    impl From<AmqpMessage> for EventData {
        fn from(message: AmqpMessage) -> Self {
            let mut event_data_builder = EventData::builder();

            if let Some(properties) = message.properties {
                if let Some(content_type) = properties.content_type() {
                    event_data_builder = event_data_builder
                        .with_content_type(Into::<String>::into(content_type.clone()));
                }
                if let Some(correlation_id) = properties.correlation_id() {
                    event_data_builder =
                        event_data_builder.with_correlation_id(correlation_id.clone());
                }
                if let Some(message_id) = properties.message_id() {
                    event_data_builder = event_data_builder.with_message_id(message_id.clone());
                }
            }
            if let Some(application_properties) = message.application_properties {
                for (key, value) in application_properties {
                    event_data_builder = event_data_builder.add_property(key, value);
                }
            }
            event_data_builder.build()
        }
    }
    impl From<EventData> for AmqpMessage {
        fn from(event_data: EventData) -> Self {
            let mut message_properties_builder = AmqpMessageProperties::builder();
            let mut message_builder = AmqpMessage::builder();
            if let Some(content_type) = event_data.content_type {
                message_properties_builder =
                    message_properties_builder.with_content_type(content_type.into());
            }
            if let Some(correlation_id) = event_data.correlation_id {
                message_properties_builder =
                    message_properties_builder.with_correlation_id(correlation_id);
            }
            if let Some(message_id) = event_data.message_id {
                message_properties_builder =
                    message_properties_builder.with_message_id(message_id.into());
            }
            message_builder = message_builder.with_properties(message_properties_builder.build());
            for (key, value) in event_data.properties {
                message_builder = message_builder.add_application_property(key, value);
            }
            message_builder.build()
        }
    }

    pub mod builders {
        use super::*;

        pub struct EventDataBuilder {
            event_data: EventData,
        }

        impl EventDataBuilder {
            pub fn new() -> Self {
                Self {
                    event_data: EventData::new(),
                }
            }

            pub fn with_body(mut self, body: Vec<u8>) -> Self {
                self.event_data.body = body;
                self
            }

            pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
                self.event_data.content_type = Some(content_type.into());
                self
            }

            pub fn with_correlation_id(mut self, correlation_id: AmqpValue) -> Self {
                self.event_data.correlation_id = Some(correlation_id);
                self
            }

            pub fn with_message_id(mut self, message_id: impl Into<String>) -> Self {
                self.event_data.message_id = Some(message_id.into());
                self
            }

            pub fn add_property(mut self, key: impl Into<String>, value: AmqpValue) -> Self {
                self.event_data.properties.insert(key.into(), value);
                self
            }

            pub fn build(self) -> EventData {
                self.event_data
            }
        }
    }
}
