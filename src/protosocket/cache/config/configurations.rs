use std::time::Duration;

use crate::protosocket::cache::Configuration;

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
        Configuration::builder()
            .timeout(Duration::from_millis(15000))
            .connection_count(default_connections())
            .az_id(None)
    }
}

/// Provides defaults suitable for an environment where your client is running in the same
/// region as the Momento service. It has more aggressive timeouts than the laptop config.
pub struct InRegion {}

impl InRegion {
    /// Returns the latest prebuilt configuration.
    /// This is the recommended configuration for most users.
    ///
    /// NOTE: this config may change in future releases to take advantage of improvements
    /// we identify for default configurations.
    #[allow(dead_code)]
    pub fn latest() -> impl Into<Configuration> {
        InRegion::v1()
    }

    /// Returns the v1 prebuilt configuration.
    ///
    /// Versioning the prebuilt configurations allows users to opt-in to changes in the default
    /// configurations. This is useful for users who want to ensure that their application's
    /// behavior does not change unexpectedly.
    ///
    /// You can set your availability zone ID by setting the `MOMENTO_AWS_AZ_ID` environment variable.
    /// See https://docs.aws.amazon.com/ram/latest/userguide/working-with-az-ids.html for more information
    /// about availability zone IDs.
    #[allow(dead_code)]
    pub fn v1() -> impl Into<Configuration> {
        Configuration::builder()
            .timeout(Duration::from_millis(1100))
            .connection_count(default_connections())
            .az_id(az_id_from_env())
    }
}

/// This config prioritizes keeping p99.9 latencies as low as possible, potentially sacrificing
/// some throughput to achieve this. Use this config if low latency is more important in
/// your application than cache availability.
pub struct LowLatency {}

impl LowLatency {
    /// Returns the latest prebuilt configuration.
    /// This is the recommended configuration for most users.
    ///
    /// NOTE: this config may change in future releases to take advantage of improvements
    /// we identify for default configurations.
    #[allow(dead_code)]
    pub fn latest() -> impl Into<Configuration> {
        LowLatency::v1()
    }

    /// Returns the v1 prebuilt configuration.
    ///
    /// Versioning the prebuilt configurations allows users to opt-in to changes in the default
    /// configurations. This is useful for users who want to ensure that their application's
    /// behavior does not change unexpectedly.
    ///
    /// You can set your availability zone ID by setting the `MOMENTO_AWS_AZ_ID` environment variable.
    /// See https://docs.aws.amazon.com/ram/latest/userguide/working-with-az-ids.html for more information
    /// about availability zone IDs.
    pub fn v1() -> impl Into<Configuration> {
        Configuration::builder()
            .timeout(Duration::from_millis(500))
            .connection_count(default_connections())
            .az_id(az_id_from_env())
    }
}

// TODO: confirm Lambda config makes sense for the protosocket client

/// This config optimizes for lambda environments.
///
/// In addition to the in region settings of [InRegion], this
/// disables keep-alives.
///
/// NOTE: keep-alives are very important for long-lived server environments where there may be periods of time
/// when the connection is idle. However, they are very problematic for lambda environments where the lambda
/// runtime is continuously frozen and unfrozen, because the lambda may be frozen before the "ACK" is received
/// from the server. This can cause the keep-alive to timeout even though the connection is completely healthy.
/// Therefore, keep-alives should be disabled in lambda and similar environments.
pub struct Lambda {}

impl Lambda {
    /// Latest recommended config for a typical lambda environment.
    ///
    /// NOTE: this config may change in future releases to take advantage of improvements
    /// we identify for default configurations.
    #[allow(dead_code)]
    pub fn latest() -> impl Into<Configuration> {
        Lambda::v1()
    }

    /// Returns the v1 prebuilt configuration.
    ///
    /// Versioning the prebuilt configurations allows users to opt-in to changes in the default
    /// configurations. This is useful for users who want to ensure that their application's
    /// behavior does not change unexpectedly.
    ///
    /// You can set your availability zone ID by setting the `MOMENTO_AWS_AZ_ID` environment variable.
    /// See https://docs.aws.amazon.com/ram/latest/userguide/working-with-az-ids.html for more information
    /// about availability zone IDs.
    pub fn v1() -> impl Into<Configuration> {
        Configuration::builder()
            .timeout(Duration::from_millis(1100))
            .connection_count(1)
            .az_id(az_id_from_env())
    }
}

fn az_id_from_env() -> Option<String> {
    std::env::var("MOMENTO_AWS_AZ_ID")
        .ok()
        .filter(|s| !s.is_empty())
}

fn default_connections() -> u32 {
    (std::thread::available_parallelism().map_or(1, |n| n.get()) / 2).clamp(1, 16) as u32
}
