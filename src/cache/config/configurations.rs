/// Provides defaults suitable for a medium-to-high-latency dev environment. Permissive timeouts
/// and relaxed latency and throughput targets.
pub mod laptop {
    use std::time::Duration;

    use crate::cache::Configuration;
    use crate::config::grpc_configuration::GrpcConfiguration;
    use crate::config::transport_strategy::TransportStrategy;

    /// Latest recommended config for a laptop development environment.
    ///
    /// NOTE: this config may change in future releases to take advantage of improvements
    /// we identify for default configurations.
    pub fn latest() -> impl Into<Configuration> {
        Configuration::builder().transport_strategy(
            TransportStrategy::builder().grpc_configuration(
                GrpcConfiguration::builder()
                    .deadline(Duration::from_millis(15000))
                    .enable_keep_alives_with_defaults(),
            ),
        )
    }
}

/// Provides defaults suitable for an environment where your client is running in the same
/// region as the Momento service. It has more aggressive timeouts than the laptop config.
pub mod in_region {
    use std::time::Duration;

    use crate::cache::Configuration;
    use crate::config::grpc_configuration::GrpcConfiguration;
    use crate::config::transport_strategy::TransportStrategy;

    /// Latest recommended config for a typical in-region environment.
    ///
    /// NOTE: this config may change in future releases to take advantage of improvements
    /// we identify for default configurations.
    pub fn latest() -> impl Into<Configuration> {
        Configuration::builder().transport_strategy(
            TransportStrategy::builder().grpc_configuration(
                GrpcConfiguration::builder()
                    .deadline(Duration::from_millis(1100))
                    .enable_keep_alives_with_defaults(),
            ),
        )
    }
}

/// This config prioritizes keeping p99.9 latencies as low as possible, potentially sacrificing
/// some throughput to achieve this. Use this config if low latency is more important in
/// your application than cache availability.
pub mod low_latency {
    use std::time::Duration;

    use crate::cache::Configuration;
    use crate::config::grpc_configuration::GrpcConfiguration;
    use crate::config::transport_strategy::TransportStrategy;

    /// Latest recommended config for a low-latency environment.
    ///
    /// NOTE: this config may change in future releases to take advantage of improvements
    /// we identify for default configurations.
    pub fn latest() -> impl Into<Configuration> {
        Configuration::builder().transport_strategy(
            TransportStrategy::builder().grpc_configuration(
                GrpcConfiguration::builder()
                    .deadline(Duration::from_millis(500))
                    .enable_keep_alives_with_defaults(),
            ),
        )
    }
}

/// Provides defaults suitable for a typical lambda environment. It has more aggressive timeouts
/// than the laptop config and does not check connection health with a keep-alive.
pub mod lambda {
    use std::time::Duration;

    use crate::cache::Configuration;
    use crate::config::grpc_configuration::GrpcConfiguration;
    use crate::config::transport_strategy::TransportStrategy;

    /// Latest recommended config for a typical lambda environment.
    ///
    /// NOTE: this config may change in future releases to take advantage of improvements
    /// we identify for default configurations.
    pub fn latest() -> impl Into<Configuration> {
        Configuration::builder().transport_strategy(
            TransportStrategy::builder().grpc_configuration(
                GrpcConfiguration::builder().deadline(Duration::from_millis(1100)),
            ),
        )
    }
}
