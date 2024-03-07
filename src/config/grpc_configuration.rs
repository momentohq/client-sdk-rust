use std::time::Duration;

/// Low-level gRPC settings for communicating with Momento.
#[derive(Clone)]
pub struct GrpcConfiguration {
    /// The duration the client is willing to wait for an RPC to complete before it is terminated
    /// with a DeadlineExceeded error.
    pub(crate) deadline: Duration,
    /// Indicates whether the client should send keep-alive pings.
    ///
    /// NOTE: keep-alives are very important for long-lived server environments where there may be
    /// periods of time when the connection is idle. However, they are very problematic for lambda
    /// environments where the lambda runtime is continuously frozen and unfrozen, because the
    /// lambda may be frozen before the "ACK" is received from the server. This can cause the
    /// keep-alive to timeout even though the connection is completely healthy. Therefore,
    /// keep-alives should be disabled in lambda and similar environments.
    pub(crate) keep_alive_while_idle: bool,
    /// The interval at which keep-alive pings are sent.
    pub(crate) keep_alive_interval: Duration,
    /// The duration the client is willing to wait for a keep-alive ping to be acknowledged before
    /// closing the connection.
    pub(crate) keep_alive_timeout: Duration,
}

impl GrpcConfiguration {
    pub fn builder(deadline: Duration) -> GrpcConfigurationBuilder {
        GrpcConfigurationBuilder {
            deadline,
            keep_alive_while_idle: true,
            keep_alive_interval: Duration::from_secs(5000),
            keep_alive_timeout: Duration::from_secs(1000),
        }
    }
}

/// Builder for `GrpcConfiguration`.
pub struct GrpcConfigurationBuilder {
    deadline: Duration,
    keep_alive_while_idle: bool,
    keep_alive_interval: Duration,
    keep_alive_timeout: Duration,
}

impl GrpcConfigurationBuilder {
    pub fn with_deadline(mut self, deadline: Duration) -> Self {
        self.deadline = deadline;
        self
    }

    pub fn with_keep_alive_while_idle(mut self, keep_alive_while_idle: bool) -> Self {
        self.keep_alive_while_idle = keep_alive_while_idle;
        self
    }

    pub fn with_keep_alive_interval(mut self, keep_alive_interval: Duration) -> Self {
        self.keep_alive_interval = keep_alive_interval;
        self
    }

    pub fn with_keep_alive_timeout(mut self, keep_alive_timeout: Duration) -> Self {
        self.keep_alive_timeout = keep_alive_timeout;
        self
    }

    pub fn build(self) -> GrpcConfiguration {
        GrpcConfiguration {
            deadline: self.deadline,
            keep_alive_while_idle: self.keep_alive_while_idle,
            keep_alive_interval: self.keep_alive_interval,
            keep_alive_timeout: self.keep_alive_timeout,
        }
    }
}
