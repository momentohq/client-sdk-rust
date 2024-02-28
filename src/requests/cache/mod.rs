use crate::cache_client::CacheClient;
use crate::MomentoResult;

pub mod set_add_elements;

pub trait MomentoRequest {
    type Response;

    /// An internal fn that allows Momento request types to define their interaction with
    /// the gRPC client. You can impl this fn for your own types if you'd like to hand them
    /// to the Momento client directly, but that is not an explicitly supported scenario and
    /// this signature may change a little over time. If that's okay with you, impl away!
    #[doc(hidden)]
    fn send(
        self,
        cache_client: &CacheClient,
    ) -> impl std::future::Future<Output = MomentoResult<Self::Response>> + Send;
}
