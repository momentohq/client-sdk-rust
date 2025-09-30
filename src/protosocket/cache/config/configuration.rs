use std::time::Duration;

/// Configuration for a Momento ProtosocketCacheClient.
///
/// Static, versioned configurations are provided for different environments:
/// ```
/// use momento::protosocket::cache::configurations;
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
/// use momento::protosocket::cache::Configuration;
///
/// let config = Configuration::builder().timeout(Duration::from_millis(1000));
/// ```

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Configuration {
    /// The duration the client will wait before terminating an RPC with a DeadlineExceeded error.
    pub(crate) timeout: Duration,
    /// The number of connections to keep in the connection pool.
    pub(crate) connection_count: usize,
}

impl Configuration {
    /// First level of constructing a ProtosocketCacheClient configuration. Must provide a timeout duration to continue.
    pub fn builder() -> ConfigurationBuilder<NeedsTimeout> {
        ConfigurationBuilder(NeedsTimeout(()))
    }

    /// Returns the duration the client will wait before terminating an RPC with a DeadlineExceeded error.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Returns the number of connections to keep in the connection pool.
    pub fn connection_count(&self) -> usize {
        self.connection_count
    }

    /// Set the number of connections to keep in the connection pool.
    pub fn set_connection_count(&self, connection_count: usize) -> Self {
        Self {
            connection_count,
            timeout: self.timeout,
        }
    }
}

/// The initial state of the ConfigurationBuilder.
pub struct ConfigurationBuilder<State>(State);

/// The state of the ConfigurationBuilder when it is waiting for a timeout.
pub struct NeedsTimeout(());

/// The state of the ConfigurationBuilder when it is waiting for a connection count.
pub struct NeedsConnectionCount {
    timeout: Duration,
}

/// The state of the ConfigurationBuilder when it is ready to build a Configuration.
pub struct ReadyToBuild {
    timeout: Duration,
    connection_count: usize,
}

impl ConfigurationBuilder<NeedsTimeout> {
    /// Sets the transport strategy for the Configuration and returns
    /// the ConfigurationBuilder in the ReadyToBuild state.
    pub fn timeout(
        self,
        timeout: impl Into<Duration>,
    ) -> ConfigurationBuilder<NeedsConnectionCount> {
        ConfigurationBuilder(NeedsConnectionCount {
            timeout: timeout.into(),
        })
    }
}

impl ConfigurationBuilder<NeedsConnectionCount> {
    /// Sets the transport strategy for the Configuration and returns
    /// the ConfigurationBuilder in the ReadyToBuild state.
    pub fn connection_count(self, connection_count: u32) -> ConfigurationBuilder<ReadyToBuild> {
        ConfigurationBuilder(ReadyToBuild {
            timeout: self.0.timeout,
            connection_count: connection_count as usize,
        })
    }
}

impl ConfigurationBuilder<ReadyToBuild> {
    /// Constructs the Configuration with the given transport strategy.
    pub fn build(self) -> Configuration {
        Configuration {
            timeout: self.0.timeout,
            connection_count: self.0.connection_count,
        }
    }
}

impl From<ConfigurationBuilder<ReadyToBuild>> for Configuration {
    fn from(builder: ConfigurationBuilder<ReadyToBuild>) -> Configuration {
        builder.build()
    }
}
