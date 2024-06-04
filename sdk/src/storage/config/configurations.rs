use std::time::Duration;

use crate::config::grpc_configuration::GrpcConfiguration;
use crate::config::transport_strategy::TransportStrategy;
use crate::storage::Configuration;

/// Provides defaults suitable for a medium-to-high-latency dev environment. Permissive timeouts
/// and relaxed latency and throughput targets.
pub struct Laptop {}

impl Laptop {
    /// Returns the latest prebuilt configuration.
    /// This is the recommended configuration for most users.
    ///
    /// NOTE: this config may change in future releases to take advantage of improvements
    /// we identify for default configurations.
    #[allow(dead_code)]
    pub fn latest() -> impl Into<Configuration> {
        Laptop::v1()
    }

    /// Returns the v1 prebuilt configuration.
    ///
    /// Versioning the prebuilt configurations allows users to opt-in to changes in the default
    /// configurations. This is useful for users who want to ensure that their application's
    /// behavior does not change unexpectedly.
    pub fn v1() -> impl Into<Configuration> {
        Configuration::builder().transport_strategy(
            TransportStrategy::builder().grpc_configuration(
                GrpcConfiguration::builder()
                    .deadline(Duration::from_millis(15000))
                    .enable_keep_alives_with_defaults(),
            ),
        )
    }
}
