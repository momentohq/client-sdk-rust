use momento_protos::control_client;
use tonic::Request;

use crate::requests::cache::MomentoRequest;
use crate::requests::status_to_error;
use crate::{utils, CacheClient, MomentoResult};

/// Request to create a cache.
///
/// # Arguments
///
/// * `cache_name` - The name of the cache to create.
///
/// # Example
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```no_run
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::requests::cache::create_cache::CreateCache;
/// use momento::requests::cache::create_cache::CreateCacheRequest;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
///
/// let create_cache_request = CreateCacheRequest::new(cache_name.to_string());
///
/// let create_cache_response = cache_client.send_request(create_cache_request).await?;
///
/// assert_eq!(create_cache_response, CreateCache {});
/// # Ok(())
/// # })
/// # }
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

        let result = cache_client
            .control_client
            .clone()
            .create_cache(request)
            .await;
        match result {
            Ok(_) => Ok(CreateCache::Created {}),
            Err(e) => {
                if e.code() == tonic::Code::AlreadyExists {
                    return Ok(CreateCache::AlreadyExists {});
                }
                Err(status_to_error(e))
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CreateCache {
    Created,
    AlreadyExists,
}
