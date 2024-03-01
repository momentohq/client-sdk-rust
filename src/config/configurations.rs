/// Provides defaults suitable for a medium-to-high-latency dev environment. Permissive timeouts
/// and relaxed latency and throughput targets.
pub mod laptop {
    use crate::config::configuration::Configuration;
    use crate::config::grpc_configuration::GrpcConfiguration;
    use crate::config::transport_strategy::TransportStrategy;
    use std::time::Duration;

    /// Latest recommended config for a laptop development environment.
    ///
    /// NOTE: this config may change in future releases to take advantage of improvements
    /// we identify for default configurations.
    pub const LATEST: Configuration = V1;

    /// V1 config for a laptop development environment.
    ///
    /// This config is guaranteed not to change in future releases of the Momento Rust SDK.
    pub const V1: Configuration = Configuration {
        transport_strategy: TransportStrategy {
            grpc_configuration: GrpcConfiguration {
                deadline_millis: Duration::from_millis(5000),
                keep_alive_while_idle: true,
                keep_alive_interval: Duration::from_secs(5000),
                keep_alive_timeout: Duration::from_secs(1000),
            },
        },
    };
}

/// Provides defaults suitable for an environment where your client is running in the same
/// region as the Momento service. It has more aggressive timeouts than the laptop config.
pub mod in_region {
    use crate::config::configuration::Configuration;
    use crate::config::grpc_configuration::GrpcConfiguration;
    use crate::config::transport_strategy::TransportStrategy;
    use std::time::Duration;

    /// Latest recommended config for a typical in-region environment.
    ///
    /// NOTE: this config may change in future releases to take advantage of improvements
    /// we identify for default configurations.
    pub const LATEST: Configuration = V1;

    /// V1 config for a typical in-region environment.
    ///
    /// This config is guaranteed not to change in future releases of the Momento Rust SDK.
    pub const V1: Configuration = Configuration {
        transport_strategy: TransportStrategy {
            grpc_configuration: GrpcConfiguration {
                deadline_millis: Duration::from_millis(1100),
                keep_alive_while_idle: true,
                keep_alive_interval: Duration::from_secs(5000),
                keep_alive_timeout: Duration::from_secs(1000),
            },
        },
    };
}

/// This config prioritizes keeping p99.9 latencies as low as possible, potentially sacrificing
/// some throughput to achieve this. Use this config if low latency is more important in
/// your application than cache availability.
pub mod low_latency {
    use crate::config::configuration::Configuration;
    use crate::config::grpc_configuration::GrpcConfiguration;
    use crate::config::transport_strategy::TransportStrategy;
    use std::time::Duration;

    /// Latest recommended config for a low-latency environment.
    ///
    /// NOTE: this config may change in future releases to take advantage of improvements
    /// we identify for default configurations.
    pub const LATEST: Configuration = V1;

    /// V1 config for a low-latency environment.
    ///
    /// This config is guaranteed not to change in future releases of the Momento Rust SDK.
    pub const V1: Configuration = Configuration {
        transport_strategy: TransportStrategy {
            grpc_configuration: GrpcConfiguration {
                deadline_millis: Duration::from_millis(500),
                keep_alive_while_idle: true,
                keep_alive_interval: Duration::from_secs(5000),
                keep_alive_timeout: Duration::from_secs(1000),
            },
        },
    };
}

/// Provides defaults suitable for a typical lambda environment. It has more aggressive timeouts
/// than the laptop config and does not check connection health with a keep-alive.
pub mod lambda {
    use crate::config::configuration::Configuration;
    use crate::config::grpc_configuration::GrpcConfiguration;
    use crate::config::transport_strategy::TransportStrategy;
    use std::time::Duration;

    /// Latest recommended config for a typical lambda environment.
    ///
    /// NOTE: this config may change in future releases to take advantage of improvements
    /// we identify for default configurations.
    pub const LATEST: Configuration = V1;

    /// V1 config for a typical lambda environment.
    ///
    /// This config is guaranteed not to change in future releases of the Momento Rust SDK.
    pub const V1: Configuration = Configuration {
        transport_strategy: TransportStrategy {
            grpc_configuration: GrpcConfiguration {
                deadline_millis: Duration::from_millis(1100),
                keep_alive_while_idle: false,
                keep_alive_interval: Duration::MAX,
                keep_alive_timeout: Duration::MAX,
            },
        },
    };
}
