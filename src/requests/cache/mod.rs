use crate::cache_client::CacheClient;
use crate::MomentoResult;

pub mod set_add_elements;

pub trait MomentoRequest {
    type Response;
}

pub trait MomentoSendableRequest<R: MomentoRequest> {
    fn send(
        self,
        cache_client: &CacheClient,
    ) -> impl std::future::Future<Output = MomentoResult<R::Response>> + Send;
}
