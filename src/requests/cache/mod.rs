use crate::{MomentoResult};
use crate::cache_client::CacheClient;

pub mod set_add_elements;

pub trait MomentoResponse {}
pub trait MomentoRequest<R: MomentoResponse> {
    fn send(self: Self, cache_client: &CacheClient) -> impl std::future::Future<Output = MomentoResult<R>> + Send;
}
