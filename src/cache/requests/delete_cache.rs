use momento_protos::control_client;
use tonic::Request;

use crate::cache::requests::MomentoRequest;
use crate::{utils, CacheClient, MomentoResult};

/// Request to delete a cache
///
/// # Arguments
///
/// * `name` - The name of the cache to be deleted.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```no_run
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::requests::cache::delete_cache::DeleteCache;
/// use momento::requests::cache::delete_cache::DeleteCacheRequest;
/// use momento::requests::MomentoErrorCode;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
///
/// let delete_cache_request = DeleteCacheRequest::new(&cache_name);
///
/// match cache_client.send_request(delete_cache_request).await {
///     Ok(_) => println!("Cache deleted: {}", &cache_name),
///     Err(e) => if let MomentoErrorCode::NotFoundError = e.error_code {
///         println!("Cache not found: {}", &cache_name);
///     } else {
///         eprintln!("Error deleting cache {}: {}", &cache_name, e);
///     }
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct DeleteCacheRequest {
    pub cache_name: String,
}

impl DeleteCacheRequest {
    pub fn new(cache_name: impl Into<String>) -> Self {
        DeleteCacheRequest {
            cache_name: cache_name.into(),
        }
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

#[derive(Debug, PartialEq, Eq)]
pub struct DeleteCache {}
