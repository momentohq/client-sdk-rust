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
/// use momento::config::grpc_configuration::GrpcConfigurationBuilder;
/// use momento::config::transport_strategy::TransportStrategy;
///
/// let config = Configuration {
///             transport_strategy: TransportStrategy {
///                 grpc_configuration: GrpcConfigurationBuilder::new(Duration::from_millis(1000))
///                     .build(),
///             },
///         };
#[derive(Clone)]
pub struct Configuration {
    /// Low-level options for network interactions with Momento.
    pub transport_strategy: TransportStrategy,
}

impl Configuration {
    pub fn new(transport_strategy: TransportStrategy) -> Self {
        Configuration { transport_strategy }
    }

    /// Returns the duration the client will wait before terminating an RPC with a DeadlineExceeded error.
    pub fn deadline_millis(&self) -> Duration {
        self.transport_strategy.grpc_configuration.deadline
    }
}
