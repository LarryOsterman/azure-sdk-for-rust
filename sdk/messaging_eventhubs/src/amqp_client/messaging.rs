//cspell: words amqp

use crate::amqp_client::value::{AmqpOrderedMap, AmqpValue};

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
