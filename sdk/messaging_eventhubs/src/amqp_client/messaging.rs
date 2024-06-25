//cspell: words amqp

use crate::amqp_client::value::{AmqpList, AmqpOrderedMap, AmqpValue};

use super::value::AmqpSymbol;

#[derive(Debug, Clone, PartialEq)]
pub enum TerminusDurability {
    None,
    Configuration,
    UnsettledState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TerminusExpiryPolicy {
    LinkDetach,
    SessionEnd,
    ConnectionClose,
    Never,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DistributionMode {
    Move,
    Copy,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AmqpOutcome {
    Accepted,
    Rejected,
    Released,
    Modified,
}

/// A target node in an AMQP message
#[derive(Debug, Clone, PartialEq)]
pub struct AmqpTarget {
    pub address: Option<String>,
    pub durable: Option<TerminusDurability>,
    pub expiry_policy: Option<TerminusExpiryPolicy>,
    pub timeout: Option<u32>,
    pub dynamic: Option<bool>,
    pub dynamic_node_properties: Option<AmqpOrderedMap<String, AmqpValue>>,
    pub capabilities: Option<Vec<AmqpValue>>,
}

impl AmqpTarget {
    pub fn builder() -> AmqpTargetBuilder {
        AmqpTargetBuilder::new()
    }
}

impl Into<String> for AmqpTarget {
    fn into(self) -> String {
        self.address.unwrap()
    }
}

impl Into<AmqpTarget> for String {
    fn into(self) -> AmqpTarget {
        AmqpTarget {
            address: Some(self),
            durable: None,
            expiry_policy: None,
            timeout: None,
            dynamic: None,
            dynamic_node_properties: None,
            capabilities: None,
        }
    }
}

pub struct AmqpTargetBuilder {
    address: Option<String>,
    durable: Option<TerminusDurability>,
    expiry_policy: Option<TerminusExpiryPolicy>,
    timeout: Option<u32>,
    dynamic: Option<bool>,
    dynamic_node_properties: Option<AmqpOrderedMap<String, AmqpValue>>,
    capabilities: Option<Vec<AmqpValue>>,
}

impl AmqpTargetBuilder {
    pub fn build(self) -> AmqpTarget {
        AmqpTarget {
            address: self.address,
            durable: self.durable,
            expiry_policy: self.expiry_policy,
            timeout: self.timeout,
            dynamic: self.dynamic,
            dynamic_node_properties: self.dynamic_node_properties,
            capabilities: self.capabilities,
        }
    }
    fn new() -> AmqpTargetBuilder {
        AmqpTargetBuilder {
            address: None,
            durable: None,
            expiry_policy: None,
            timeout: None,
            dynamic: None,
            dynamic_node_properties: None,
            capabilities: None,
        }
    }
    pub fn with_address(mut self, address: impl Into<String>) -> Self {
        self.address = Some(address.into());
        self
    }
    pub fn with_durable(mut self, durable: TerminusDurability) -> Self {
        self.durable = Some(durable);
        self
    }
    pub fn with_expiry_policy(mut self, expiry_policy: TerminusExpiryPolicy) -> Self {
        self.expiry_policy = Some(expiry_policy.into());
        self
    }
    pub fn with_timeout(mut self, timeout: u32) -> Self {
        self.timeout = Some(timeout);
        self
    }
    pub fn with_dynamic(mut self, dynamic: bool) -> Self {
        self.dynamic = Some(dynamic);
        self
    }
    pub fn with_dynamic_node_properties(
        mut self,
        dynamic_node_properties: impl Into<AmqpOrderedMap<String, AmqpValue>>,
    ) -> Self {
        self.dynamic_node_properties = Some(dynamic_node_properties.into());
        self
    }
    pub fn with_capabilities(mut self, capabilities: Vec<AmqpValue>) -> Self {
        self.capabilities = Some(capabilities);
        self
    }
}

/// A source node in an AMQP message
#[derive(Debug, Clone, PartialEq)]
pub struct AmqpSource {
    pub address: Option<String>,
    pub durable: Option<TerminusDurability>,
    pub expiry_policy: Option<TerminusExpiryPolicy>,
    pub timeout: Option<u32>,
    pub dynamic: Option<bool>,
    pub dynamic_node_properties: Option<AmqpOrderedMap<String, AmqpValue>>,
    pub distribution_mode: Option<DistributionMode>,
    pub filter: Option<AmqpOrderedMap<String, AmqpValue>>,
    pub default_outcome: Option<AmqpOutcome>,
    pub outcomes: Option<Vec<AmqpSymbol>>,
    pub capabilities: Option<Vec<AmqpValue>>,
}

impl AmqpSource {
    pub fn builder() -> AmqpSourceBuilder {
        AmqpSourceBuilder::new()
    }
}

pub struct AmqpSourceBuilder {
    address: Option<String>,
    durable: Option<TerminusDurability>,
    expiry_policy: Option<TerminusExpiryPolicy>,
    timeout: Option<u32>,
    dynamic: Option<bool>,
    dynamic_node_properties: Option<AmqpOrderedMap<String, AmqpValue>>,
    distribution_mode: Option<DistributionMode>,
    filter: Option<AmqpOrderedMap<String, AmqpValue>>,
    default_outcome: Option<AmqpOutcome>,
    outcomes: Option<Vec<AmqpSymbol>>,
    capabilities: Option<Vec<AmqpValue>>,
}

impl AmqpSourceBuilder {
    pub fn build(self) -> AmqpSource {
        AmqpSource {
            address: self.address,
            durable: self.durable,
            expiry_policy: self.expiry_policy,
            timeout: self.timeout,
            dynamic: self.dynamic,
            dynamic_node_properties: self.dynamic_node_properties,
            distribution_mode: self.distribution_mode,
            filter: self.filter,
            default_outcome: self.default_outcome,
            outcomes: self.outcomes,
            capabilities: self.capabilities,
        }
    }
    fn new() -> AmqpSourceBuilder {
        AmqpSourceBuilder {
            address: None,
            durable: None,
            expiry_policy: None,
            timeout: None,
            dynamic: None,
            dynamic_node_properties: None,
            distribution_mode: None,
            filter: None,
            default_outcome: None,
            outcomes: None,
            capabilities: None,
        }
    }
    pub fn with_address(mut self, address: impl Into<String>) -> Self {
        self.address = Some(address.into());
        self
    }
    pub fn with_durable(mut self, durable: TerminusDurability) -> Self {
        self.durable = Some(durable);
        self
    }
    pub fn with_expiry_policy(mut self, expiry_policy: TerminusExpiryPolicy) -> Self {
        self.expiry_policy = Some(expiry_policy.into());
        self
    }
    pub fn with_timeout(mut self, timeout: u32) -> Self {
        self.timeout = Some(timeout);
        self
    }
    pub fn with_dynamic(mut self, dynamic: bool) -> Self {
        self.dynamic = Some(dynamic);
        self
    }
    pub fn with_dynamic_node_properties(
        mut self,
        dynamic_node_properties: impl Into<AmqpOrderedMap<String, AmqpValue>>,
    ) -> Self {
        self.dynamic_node_properties = Some(dynamic_node_properties.into());
        self
    }
    pub fn with_distribution_mode(mut self, distribution_mode: DistributionMode) -> Self {
        self.distribution_mode = Some(distribution_mode);
        self
    }
    pub fn with_filter(mut self, filter: impl Into<AmqpOrderedMap<String, AmqpValue>>) -> Self {
        self.filter = Some(filter.into());
        self
    }
    pub fn with_default_outcome(mut self, default_outcome: AmqpOutcome) -> Self {
        self.default_outcome = Some(default_outcome);
        self
    }
    pub fn with_outcomes(mut self, outcomes: Vec<AmqpSymbol>) -> Self {
        self.outcomes = Some(outcomes);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AmqpHeader {
    pub durable: Option<bool>,
    pub priority: Option<u8>,
    pub time_to_live: Option<u64>,
    pub first_acquirer: Option<bool>,
    pub delivery_count: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AmqpProperties {
    pub message_id: Option<AmqpValue>,
    pub user_id: Option<Vec<u8>>,
    pub to: Option<AmqpValue>,
    pub subject: Option<String>,
    pub reply_to: Option<AmqpValue>,
    pub correlation_id: Option<AmqpValue>,
    pub content_type: Option<AmqpSymbol>,
    pub content_encoding: Option<AmqpSymbol>,
    pub absolute_expiry_time: Option<std::time::SystemTime>,
    pub creation_time: Option<std::time::SystemTime>,
    pub group_id: Option<String>,
    pub group_sequence: Option<u32>,
    pub reply_to_group_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AmqpBody {
    Binary(Vec<Vec<u8>>),
    Sequence(Vec<AmqpList>),
    Value(AmqpValue),
    Empty,
}

/// An AMQP message
/// This is a simplified version of the AMQP message
/// that is used in the Azure SDK for Event Hubs
/// and is not a complete implementation of the AMQP message
/// as defined in the AMQP specification
/// https://docs.oasis-open.org/amqp/core/v1.0/os/amqp-core-messaging-v1.0-os.html
///
#[derive(Debug, Clone, PartialEq)]
pub struct AmqpMessage {
    pub body: AmqpBody,
    pub header: Option<AmqpHeader>,
    pub application_properties: Option<AmqpOrderedMap<String, AmqpValue>>,
    pub message_annotations: Option<AmqpOrderedMap<AmqpValue, AmqpValue>>,
    pub delivery_annotations: Option<AmqpOrderedMap<AmqpValue, AmqpValue>>,
    pub properties: Option<AmqpProperties>,
    pub footer: Option<AmqpOrderedMap<AmqpValue, AmqpValue>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amqp_source_builder() {
        let source = AmqpSource::builder()
            .with_address("address")
            .with_durable(TerminusDurability::Configuration)
            .with_expiry_policy(TerminusExpiryPolicy::ConnectionClose)
            .with_timeout(10)
            .with_dynamic(true)
            .with_dynamic_node_properties(AmqpOrderedMap::new())
            .with_distribution_mode(DistributionMode::Copy)
            .with_filter(AmqpOrderedMap::new())
            .with_default_outcome(AmqpOutcome::Accepted)
            .with_outcomes(vec![AmqpSymbol::from("outcome")])
            .build();

        assert_eq!(source.address, Some("address".to_string()));
        assert_eq!(source.durable, Some(TerminusDurability::Configuration));
        assert_eq!(
            source.expiry_policy,
            Some(TerminusExpiryPolicy::ConnectionClose)
        );
        assert_eq!(source.timeout, Some(10));
        assert_eq!(source.dynamic, Some(true));
        assert_eq!(source.dynamic_node_properties, Some(AmqpOrderedMap::new()));
        assert_eq!(source.distribution_mode, Some(DistributionMode::Copy));
        assert_eq!(source.filter, Some(AmqpOrderedMap::new()));
        assert_eq!(source.default_outcome, Some(AmqpOutcome::Accepted));
        assert_eq!(source.outcomes, Some(vec![AmqpSymbol::from("outcome")]));
    }
}
