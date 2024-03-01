use std::time::Duration;

/// Low-level gRPC settings for communicating with Momento.
#[derive(Clone)]
pub struct GrpcConfiguration {
    /// The duration the client is willing to wait for an RPC to complete before it is terminated
    /// with a DeadlineExceeded error.
    pub deadline_millis: Duration,
    /// Indicates whether the client should send keep-alive pings.
    ///
    /// NOTE: keep-alives are very important for long-lived server environments where there may be
    /// periods of time when the connection is idle. However, they are very problematic for lambda
    /// environments where the lambda runtime is continuously frozen and unfrozen, because the
    /// lambda may be frozen before the "ACK" is received from the server. This can cause the
    /// keep-alive to timeout even though the connection is completely healthy. Therefore,
    /// keep-alives should be disabled in lambda and similar environments.
    pub keep_alive_while_idle: bool,
    /// The interval at which keep-alive pings are sent.
    pub keep_alive_interval: Duration,
    /// The duration the client is willing to wait for a keep-alive ping to be acknowledged before
    /// closing the connection.
    pub keep_alive_timeout: Duration,
}
