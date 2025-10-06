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
    /// Optional availability zone ID to use for preferring connections to one az or another.
    pub(crate) az_id: Option<String>,
}

impl Configuration {
    /// First level of constructing a ProtosocketCacheClient configuration. Must provide a timeout duration to continue.
    pub fn builder() -> ConfigurationBuilder<NeedsTimeout> {
        ConfigurationBuilder {
            state: NeedsTimeout,
        }
    }

    /// Returns the duration the client will wait before terminating an RPC with a DeadlineExceeded error.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Returns the availability zone ID to use for preferring connections to one az or another.
    pub fn az_id(&self) -> &Option<String> {
        &self.az_id
    }

    /// Returns the number of connections to keep in the connection pool.
    pub fn connection_count(&self) -> usize {
        self.connection_count
    }

    /// Set the number of connections to keep in the connection pool.
    pub fn set_connection_count(&mut self, connection_count: usize) -> &mut Self {
        self.connection_count = connection_count;
        self
    }

    /// Set the availability zone id hint to use for preferring connections to one az or another.
    pub fn set_az_id(&mut self, az_id: Option<String>) -> &mut Self {
        self.az_id = az_id;
        self
    }
}

/// The initial state of the ConfigurationBuilder.
pub struct ConfigurationBuilder<State> {
    state: State,
}

/// The state of the ConfigurationBuilder when it is waiting for a timeout.
pub struct NeedsTimeout;

/// The state of the ConfigurationBuilder when it is waiting for a connection count.
pub struct NeedsConnectionCount {
    timeout: Duration,
}

/// The state of the ConfigurationBuilder when it is waiting for an optional availability zone ID.
pub struct NeedsAzId {
    timeout: Duration,
    connection_count: usize,
}

/// The state of the ConfigurationBuilder when it is ready to build a Configuration.
pub struct ReadyToBuild {
    timeout: Duration,
    connection_count: usize,
    az_id: Option<String>,
}

impl ConfigurationBuilder<NeedsTimeout> {
    /// Sets the transport strategy for the Configuration and returns
    /// the ConfigurationBuilder in the ReadyToBuild state.
    pub fn timeout(
        self,
        timeout: impl Into<Duration>,
    ) -> ConfigurationBuilder<NeedsConnectionCount> {
        ConfigurationBuilder {
            state: NeedsConnectionCount {
                timeout: timeout.into(),
            },
        }
    }
}

impl ConfigurationBuilder<NeedsConnectionCount> {
    /// Sets the transport strategy for the Configuration and returns
    /// the ConfigurationBuilder in the ReadyToBuild state.
    pub fn connection_count(self, connection_count: u32) -> ConfigurationBuilder<NeedsAzId> {
        ConfigurationBuilder {
            state: NeedsAzId {
                timeout: self.state.timeout,
                connection_count: connection_count as usize,
            },
        }
    }
}

impl ConfigurationBuilder<NeedsAzId> {
    /// Sets the transport strategy for the Configuration and returns
    /// the ConfigurationBuilder in the ReadyToBuild state.
    pub fn az_id(self, az_id: Option<String>) -> ConfigurationBuilder<ReadyToBuild> {
        ConfigurationBuilder {
            state: ReadyToBuild {
                timeout: self.state.timeout,
                connection_count: self.state.connection_count,
                az_id,
            },
        }
    }
}

impl ConfigurationBuilder<ReadyToBuild> {
    /// Constructs the Configuration with the given transport strategy.
    pub fn build(self) -> Configuration {
        let ReadyToBuild {
            timeout,
            connection_count,
            az_id,
        } = self.state;
        Configuration {
            timeout,
            connection_count,
            az_id,
        }
    }
}

impl From<ConfigurationBuilder<ReadyToBuild>> for Configuration {
    fn from(builder: ConfigurationBuilder<ReadyToBuild>) -> Configuration {
        builder.build()
    }
}
