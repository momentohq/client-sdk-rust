use std::time::Duration;

/// Low-level gRPC settings for communicating with Momento.
#[derive(Clone, Debug)]
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
    pub(crate) keep_alive_while_idle: Option<bool>,
    /// The interval at which keep-alive pings are sent.
    pub(crate) keep_alive_interval: Option<Duration>,
    /// The duration the client is willing to wait for a keep-alive ping to be acknowledged before
    /// closing the connection.
    pub(crate) keep_alive_timeout: Option<Duration>,
}

impl GrpcConfiguration {
    pub fn builder() -> GrpcConfigurationBuilder<NeedsDeadline> {
        GrpcConfigurationBuilder(NeedsDeadline(()))
    }
}

pub struct GrpcConfigurationBuilder<State>(State);

pub struct NeedsDeadline(());

pub struct ReadyToBuild {
    deadline: Duration,
    keep_alive_while_idle: Option<bool>,
    keep_alive_interval: Option<Duration>,
    keep_alive_timeout: Option<Duration>,
}

impl GrpcConfigurationBuilder<NeedsDeadline> {
    pub fn deadline(self, deadline: Duration) -> GrpcConfigurationBuilder<ReadyToBuild> {
        GrpcConfigurationBuilder(ReadyToBuild {
            deadline,
            keep_alive_while_idle: None,
            keep_alive_interval: None,
            keep_alive_timeout: None,
        })
    }
}

impl GrpcConfigurationBuilder<ReadyToBuild> {
    pub(crate) fn enable_keep_alives_with_defaults(
        mut self,
    ) -> GrpcConfigurationBuilder<ReadyToBuild> {
        self.0.keep_alive_while_idle = Some(true);
        self.0.keep_alive_interval = Some(Duration::from_secs(5));
        self.0.keep_alive_timeout = Some(Duration::from_secs(1));
        self
    }

    pub fn keep_alive_while_idle(mut self, keep_alive_while_idle: bool) -> Self {
        self.0.keep_alive_while_idle = Some(keep_alive_while_idle);
        self
    }

    pub fn keep_alive_interval(mut self, keep_alive_interval: Duration) -> Self {
        self.0.keep_alive_interval = Some(keep_alive_interval);
        self
    }

    pub fn keep_alive_timeout(mut self, keep_alive_timeout: Duration) -> Self {
        self.0.keep_alive_timeout = Some(keep_alive_timeout);
        self
    }

    pub fn build(self) -> GrpcConfiguration {
        GrpcConfiguration {
            deadline: self.0.deadline,
            keep_alive_while_idle: self.0.keep_alive_while_idle,
            keep_alive_interval: self.0.keep_alive_interval,
            keep_alive_timeout: self.0.keep_alive_timeout,
        }
    }
}
