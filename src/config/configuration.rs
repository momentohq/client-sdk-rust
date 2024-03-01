use crate::config::grpc_configuration::GrpcConfiguration;
use crate::config::transport_strategy::TransportStrategy;
use std::time::Duration;

/// Configuration for a Momento cache client.
///
/// Static, versioned configurations are provided for different environments:
/// ```
/// use momento::config::configurations;
///
/// /// Use laptop for local development
/// let developer_config = configurations::laptop::LATEST;
/// /// Use in_region for a typical server environment
/// let server_config = configurations::in_region::V1;
/// ```
/// If you have specific requirements, configurations can also be constructed manually:
/// ```
/// use std::time::Duration;
/// use momento::config::configuration::Configuration;
/// use momento::config::grpc_configuration::GrpcConfiguration;
/// use momento::config::transport_strategy::TransportStrategy;
///
/// let config = Configuration {
///             transport_strategy: TransportStrategy {
///                 grpc_configuration: GrpcConfiguration {
///                     deadline_millis: Duration::from_millis(1000),
///                     keep_alive_while_idle: true,
///                     keep_alive_interval: Duration::from_secs(5000),
///                     keep_alive_timeout: Duration::from_secs(1000),
///                 },
///             },
///         };
#[derive(Clone)]
pub struct Configuration {
    /// Low-level options for network interactions with Momento.
    pub transport_strategy: TransportStrategy,
}

impl Configuration {
    pub fn new(
        deadline_millis: Duration,
        keep_alive_while_idle: bool,
        keep_alive_interval: Duration,
        keep_alive_timeout: Duration,
    ) -> Self {
        Configuration {
            transport_strategy: TransportStrategy {
                grpc_configuration: GrpcConfiguration {
                    deadline_millis,
                    keep_alive_while_idle,
                    keep_alive_interval,
                    keep_alive_timeout,
                },
            },
        }
    }

    /// Returns the duration the client will wait before terminating an RPC with a DeadlineExceeded error.
    pub fn deadline_millis(&self) -> Duration {
        self.transport_strategy.grpc_configuration.deadline_millis
    }
}
