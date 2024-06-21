use momento_protos::control_client;
use tonic::Request;

use crate::cache::messages::MomentoRequest;
use crate::{utils, CacheClient, MomentoResult};

/// Request to flush a cache of its data
///
/// # Arguments
///
/// * `name` - The name of the cache to be flushed.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::cache::{FlushCacheResponse, FlushCacheRequest};
/// use momento::MomentoErrorCode;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
///
/// let flush_cache_request = FlushCacheRequest::new(cache_name.to_string());
///
/// match cache_client.send_request(flush_cache_request).await {
///     Ok(_) => println!("Flushed cache: {}", cache_name),
///     Err(e) => {
///         if let MomentoErrorCode::CacheNotFoundError = e.error_code {
///             println!("Cache not found: {}", cache_name);
///         } else {
///            eprintln!("Error flushing cache: {}", e);
///         }
///     }
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct FlushCacheRequest {
    /// The name of the cache to be flushed
    pub cache_name: String,
}

impl FlushCacheRequest {
    /// Constructs a new FlushCacheRequest.
    pub fn new(cache_name: impl Into<String>) -> Self {
        FlushCacheRequest {
            cache_name: cache_name.into(),
        }
    }
}

impl MomentoRequest for FlushCacheRequest {
    type Response = FlushCacheResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<FlushCacheResponse> {
        let cache_name = &self.cache_name;

        utils::is_cache_name_valid(cache_name)?;
        let request = Request::new(control_client::FlushCacheRequest {
            cache_name: cache_name.to_string(),
        });

        let _ = cache_client
            .control_client
            .clone()
            .flush_cache(request)
            .await?;
        Ok(FlushCacheResponse {})
    }
}

/// The response type for a successful flush cache request
#[derive(Debug, PartialEq, Eq)]
pub struct FlushCacheResponse {}
