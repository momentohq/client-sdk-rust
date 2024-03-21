/// Provides defaults suitable for a medium-to-high-latency dev environment. Permissive timeouts
/// and relaxed latency and throughput targets.
pub mod laptop {
    use std::time::Duration;

    use crate::config::configuration::Configuration;
    use crate::config::grpc_configuration::GrpcConfiguration;
    use crate::config::transport_strategy::TransportStrategy;

    /// Latest recommended config for a laptop development environment.
    ///
    /// NOTE: this config may change in future releases to take advantage of improvements
    /// we identify for default configurations.
    pub fn latest() -> Configuration {
        v1()
    }

    /// V1 config for a laptop development environment.
    ///
    /// This config is guaranteed not to change in future releases of the Momento Rust SDK.
    pub fn v1() -> Configuration {
        Configuration::builder()
            .transport_strategy(
                TransportStrategy::builder()
                    .grpc_configuration(
                        GrpcConfiguration::builder()
                            .deadline(Duration::from_millis(15000))
                            .enable_keep_alives_with_defaults()
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}

/// Provides defaults suitable for an environment where your client is running in the same
/// region as the Momento service. It has more aggressive timeouts than the laptop config.
pub mod in_region {
    use std::time::Duration;

    use crate::config::configuration::Configuration;
    use crate::config::grpc_configuration::GrpcConfiguration;
    use crate::config::transport_strategy::TransportStrategy;

    /// Latest recommended config for a typical in-region environment.
    ///
    /// NOTE: this config may change in future releases to take advantage of improvements
    /// we identify for default configurations.
    pub fn latest() -> Configuration {
        v1()
    }

    /// V1 config for a typical in-region environment.
    ///
    /// This config is guaranteed not to change in future releases of the Momento Rust SDK.
    pub fn v1() -> Configuration {
        Configuration::builder()
            .transport_strategy(
                TransportStrategy::builder()
                    .grpc_configuration(
                        GrpcConfiguration::builder()
                            .deadline(Duration::from_millis(1100))
                            .enable_keep_alives_with_defaults()
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}

/// This config prioritizes keeping p99.9 latencies as low as possible, potentially sacrificing
/// some throughput to achieve this. Use this config if low latency is more important in
/// your application than cache availability.
pub mod low_latency {
    use std::time::Duration;

    use crate::config::configuration::Configuration;
    use crate::config::grpc_configuration::GrpcConfiguration;
    use crate::config::transport_strategy::TransportStrategy;

    /// Latest recommended config for a low-latency environment.
    ///
    /// NOTE: this config may change in future releases to take advantage of improvements
    /// we identify for default configurations.
    pub fn latest() -> Configuration {
        v1()
    }

    /// V1 config for a low-latency environment.
    ///
    /// This config is guaranteed not to change in future releases of the Momento Rust SDK.
    pub fn v1() -> Configuration {
        Configuration::builder()
            .transport_strategy(
                TransportStrategy::builder()
                    .grpc_configuration(
                        GrpcConfiguration::builder()
                            .deadline(Duration::from_millis(500))
                            .enable_keep_alives_with_defaults()
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}

/// Provides defaults suitable for a typical lambda environment. It has more aggressive timeouts
/// than the laptop config and does not check connection health with a keep-alive.
pub mod lambda {
    use std::time::Duration;

    use crate::config::configuration::Configuration;
    use crate::config::grpc_configuration::GrpcConfiguration;
    use crate::config::transport_strategy::TransportStrategy;

    /// Latest recommended config for a typical lambda environment.
    ///
    /// NOTE: this config may change in future releases to take advantage of improvements
    /// we identify for default configurations.
    pub fn latest() -> Configuration {
        v1()
    }

    /// V1 config for a typical lambda environment.
    ///
    /// This config is guaranteed not to change in future releases of the Momento Rust SDK.
    pub fn v1() -> Configuration {
        Configuration::builder()
            .transport_strategy(
                TransportStrategy::builder()
                    .grpc_configuration(
                        GrpcConfiguration::builder()
                            .deadline(Duration::from_millis(1100))
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
