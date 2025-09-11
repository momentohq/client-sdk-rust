use std::time::Duration;

use crate::MomentoResult;
use crate::ProtosocketCacheClient;

/// A trait that allows Momento request types to define their interaction with the protosocket client.
pub trait MomentoProtosocketRequest {
    #[allow(missing_docs)]
    type Response;

    /// An internal fn that allows Momento request types to define their interaction with
    /// the protosocket client. You can impl this fn for your own types if you'd like to hand them
    /// to the Momento client directly, but that is not an explicitly supported scenario and
    /// this signature may change a little over time. If that's okay with you, impl away!
    #[doc(hidden)]
    fn send(
        self,
        cache_client: &ProtosocketCacheClient,
        timeout: Duration,
    ) -> impl std::future::Future<Output = MomentoResult<Self::Response>> + Send;
}
