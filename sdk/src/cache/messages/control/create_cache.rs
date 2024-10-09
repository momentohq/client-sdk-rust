use momento_protos::control_client;
use tonic::Request;

use crate::cache::messages::MomentoRequest;
use crate::status_to_error;
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
/// use momento::cache::{CreateCacheResponse, CreateCacheRequest};
/// # let (cache_client, cache_name) = create_doctest_cache_client();
///
/// let create_cache_request = CreateCacheRequest::new(&cache_name);
///
/// match cache_client.send_request(create_cache_request).await? {
///     CreateCacheResponse::Created => println!("Cache {} created", &cache_name),
///     CreateCacheResponse::AlreadyExists => println!("Cache {} already exists", &cache_name),
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct CreateCacheRequest {
    /// The name of the cache to create.
    pub cache_name: String,
}

impl CreateCacheRequest {
    /// Constructs a new CreateCacheRequest.
    pub fn new(cache_name: impl Into<String>) -> Self {
        CreateCacheRequest {
            cache_name: cache_name.into(),
        }
    }
}

impl MomentoRequest for CreateCacheRequest {
    type Response = CreateCacheResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<CreateCacheResponse> {
        utils::is_cache_name_valid(&self.cache_name)?;
        let request = Request::new(control_client::CreateCacheRequest {
            cache_name: self.cache_name,
        });

        let result = cache_client.control_client().create_cache(request).await;
        match result {
            Ok(_) => Ok(CreateCacheResponse::Created {}),
            Err(e) => {
                if e.code() == tonic::Code::AlreadyExists {
                    return Ok(CreateCacheResponse::AlreadyExists {});
                }
                Err(status_to_error(e))
            }
        }
    }
}

/// The response type for a successful create cache request
#[derive(Debug, PartialEq, Eq)]
pub enum CreateCacheResponse {
    /// The cache was created.
    Created,
    /// The cache already exists.
    AlreadyExists,
}
