use momento_protos::control_client;
use tonic::Request;

use crate::requests::cache::MomentoRequest;
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
    pub fn new(cache_name: impl Into<String>) -> Self {
        CreateCacheRequest {
            cache_name: cache_name.into(),
        }
    }
}

impl MomentoRequest for CreateCacheRequest {
    type Response = CreateCache;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<CreateCache> {
        utils::is_cache_name_valid(&self.cache_name)?;
        let request = Request::new(control_client::CreateCacheRequest {
            cache_name: self.cache_name,
        });

        let _ = cache_client
            .control_client
            .clone()
            .create_cache(request)
            .await?;
        Ok(CreateCache {})
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CreateCache {}
