use std::time::Duration;

use crate::config::transport_strategy::TransportStrategy;

/// Configuration for a Momento cache client.
///
/// Static, versioned configurations are provided for different environments:
/// ```
/// use momento::config::configurations;
///
/// /// Use laptop for local development
/// let developer_config = configurations::laptop::latest();
/// /// Use in_region for a typical server environment
/// let server_config = configurations::in_region::v1();
/// ```
/// If you have specific requirements, configurations can also be constructed manually:
/// ```
/// use std::time::Duration;
/// use momento::config::configuration::Configuration;
/// use momento::config::grpc_configuration::{GrpcConfiguration, GrpcConfigurationBuilder};
/// use momento::config::transport_strategy::TransportStrategy;
///
/// let config = Configuration::builder(
///            TransportStrategy::builder(
///                GrpcConfiguration::builder(Duration::from_millis(1000)).build(),
///            )
///            .build(),
///        )
///        .build();
#[derive(Clone)]
pub struct Configuration {
    /// Low-level options for network interactions with Momento.
    pub(crate) transport_strategy: TransportStrategy,
}

impl Configuration {
    pub fn builder(transport_strategy: TransportStrategy) -> ConfigurationBuilder {
        ConfigurationBuilder { transport_strategy }
    }

    /// Returns the duration the client will wait before terminating an RPC with a DeadlineExceeded error.
    pub fn deadline_millis(&self) -> Duration {
        self.transport_strategy.grpc_configuration.deadline
    }
}

pub struct ConfigurationBuilder {
    transport_strategy: TransportStrategy,
}

impl ConfigurationBuilder {
    pub fn with_transport_strategy(mut self, transport_strategy: TransportStrategy) -> Self {
        self.transport_strategy = transport_strategy;
        self
    }

    pub fn build(self) -> Configuration {
        Configuration {
            transport_strategy: self.transport_strategy,
        }
    }
}
