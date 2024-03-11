use momento_protos::control_client;
use tonic::Request;

use crate::requests::cache::MomentoRequest;
use crate::{utils, CacheClient, MomentoResult};

pub struct CreateCacheRequest {
    pub cache_name: String,
}

impl CreateCacheRequest {
    pub fn new(cache_name: String) -> Self {
        CreateCacheRequest { cache_name }
    }
}

impl MomentoRequest for CreateCacheRequest {
    type Response = CreateCache;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<CreateCache> {
        let cache_name = &self.cache_name;

        utils::is_cache_name_valid(cache_name)?;
        let request = Request::new(control_client::CreateCacheRequest {
            cache_name: cache_name.to_string(),
        });

        let _ = cache_client
            .control_client
            .clone()
            .create_cache(request)
            .await?;
        Ok(CreateCache {})
    }
}

pub struct CreateCache {}
