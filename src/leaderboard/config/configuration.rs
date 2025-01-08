use std::time::Duration;

use crate::config::transport_strategy::TransportStrategy;

/// Configuration for a Momento leaderboard client.
///
/// Static, versioned configurations are provided for different environments:
/// ```
/// use momento::leaderboard::configurations;
///
/// /// Use laptop for local development
/// let developer_config = configurations::Laptop::latest();
///
/// /// Use in_region for a typical server environment
/// let server_config = configurations::InRegion::latest();
/// ```
/// If you have specific requirements, configurations can also be constructed manually:
/// ```
/// use std::time::Duration;
/// use momento::leaderboard::Configuration;
/// use momento::config::grpc_configuration::{GrpcConfiguration, GrpcConfigurationBuilder};
/// use momento::config::transport_strategy::TransportStrategy;
///
/// let config = Configuration::builder()
///     .transport_strategy(
///         TransportStrategy::builder()
///             .grpc_configuration(
///                 GrpcConfiguration::builder()
///                     .deadline(Duration::from_millis(1000))
///             )
///     );

#[derive(Clone, Debug)]
pub struct Configuration {
    /// Low-level options for network interactions with Momento.
    pub(crate) transport_strategy: TransportStrategy,
}

impl Configuration {
    /// First level of constructing a CacheClient configuration. Must provide a [TransportStrategy] to continue.
    pub fn builder() -> ConfigurationBuilder<NeedsTransportStrategy> {
        ConfigurationBuilder(NeedsTransportStrategy(()))
    }

    /// Returns the duration the client will wait before terminating an RPC with a DeadlineExceeded error.
    pub fn deadline_millis(&self) -> Duration {
        self.transport_strategy.grpc_configuration.deadline
    }
}

/// The initial state of the ConfigurationBuilder.
pub struct ConfigurationBuilder<State>(State);

/// The state of the ConfigurationBuilder when it is waiting for a transport strategy.
pub struct NeedsTransportStrategy(());

/// The state of the ConfigurationBuilder when it is ready to build a Configuration.
pub struct ReadyToBuild {
    transport_strategy: TransportStrategy,
}

impl ConfigurationBuilder<NeedsTransportStrategy> {
    /// Sets the transport strategy for the Configuration and returns
    /// the ConfigurationBuilder in the ReadyToBuild state.
    pub fn transport_strategy(
        self,
        transport_strategy: impl Into<TransportStrategy>,
    ) -> ConfigurationBuilder<ReadyToBuild> {
        ConfigurationBuilder(ReadyToBuild {
            transport_strategy: transport_strategy.into(),
        })
    }
}

impl ConfigurationBuilder<ReadyToBuild> {
    /// Constructs the Configuration with the given transport strategy.
    pub fn build(self) -> Configuration {
        Configuration {
            transport_strategy: self.0.transport_strategy,
        }
    }
}

impl From<ConfigurationBuilder<ReadyToBuild>> for Configuration {
    fn from(builder: ConfigurationBuilder<ReadyToBuild>) -> Configuration {
        builder.build()
    }
}
