use crate::{Leaderboard, MomentoResult};

/// A trait that allows Momento request types to define their interaction with the gRPC client.
pub trait MomentoRequest {
    /// The response type for this request.
    type Response;

    /// An internal fn that allows Momento request types to define their interaction with
    /// the gRPC client.
    #[doc(hidden)]
    fn send(
        self,
        leaderboard: &Leaderboard,
    ) -> impl std::future::Future<Output = MomentoResult<Self::Response>> + Send;
}
