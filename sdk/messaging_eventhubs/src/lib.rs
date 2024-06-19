// cspell: words amqp eventhubs

#![recursion_limit = "128"]

pub mod amqp_client;
pub(crate) mod common;
pub mod consumer;
pub mod error;
pub mod producer;

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
pub struct EventHubProperties {
    pub name: String,
    pub created_on: std::time::SystemTime,
    pub partition_ids: Vec<String>,
}
