use momento_protos::control_client;
use tonic::Request;

use crate::requests::cache::MomentoRequest;
use crate::{utils, CacheClient, MomentoResult};

pub struct DeleteCacheRequest {
    pub cache_name: String,
}

impl DeleteCacheRequest {
    pub fn new(cache_name: String) -> Self {
        DeleteCacheRequest { cache_name }
    }
}

impl MomentoRequest for DeleteCacheRequest {
    type Response = DeleteCache;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<DeleteCache> {
        let cache_name = &self.cache_name;

        utils::is_cache_name_valid(cache_name)?;
        let request = Request::new(control_client::DeleteCacheRequest {
            cache_name: cache_name.to_string(),
        });

        let _ = cache_client
            .control_client
            .clone()
            .delete_cache(request)
            .await?;
        Ok(DeleteCache {})
    }
}

pub struct DeleteCache {}
