use std::time::Duration;

use momento_protos::cache_client::update_ttl_request::UpdateTtl::IncreaseToMilliseconds;
use momento_protos::cache_client::update_ttl_response::{self};

use crate::MomentoError;
use crate::{
    cache::MomentoRequest, utils::prep_request_with_timeout, CacheClient, IntoBytes, MomentoResult,
};

/// Increase the ttl of an item in the cache.
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `key` - the key of the item for which ttl is requested
/// * `ttl` - The time-to-live that should overwrite the current ttl. Should be greater than the current ttl.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// use std::time::Duration;
/// use momento::cache::{IncreaseTtlResponse, IncreaseTtlRequest};
/// # cache_client.set(&cache_name, "key1", "value").await?;
///
/// let request = IncreaseTtlRequest::new(&cache_name, "key1", Duration::from_secs(5));
///
/// match(cache_client.send_request(request).await?) {
///     IncreaseTtlResponse::Set => println!("TTL updated"),
///     IncreaseTtlResponse::NotSet => println!("unable to increase TTL"),
///     IncreaseTtlResponse::Miss => return Err(anyhow::Error::msg("cache miss"))
/// };
/// # Ok(())
/// # })
/// # }
/// ```
pub struct IncreaseTtlRequest<K: IntoBytes> {
    cache_name: String,
    key: K,
    ttl: Duration,
}

impl<K: IntoBytes> IncreaseTtlRequest<K> {
    /// Constructs a new IncreaseTtlRequest.
    pub fn new(cache_name: impl Into<String>, key: K, ttl: Duration) -> Self {
        Self {
            cache_name: cache_name.into(),
            key,
            ttl,
        }
    }
}

impl<K: IntoBytes> MomentoRequest for IncreaseTtlRequest<K> {
    type Response = IncreaseTtlResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<IncreaseTtlResponse> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.configuration.deadline_millis(),
            momento_protos::cache_client::UpdateTtlRequest {
                cache_key: self.key.into_bytes(),
                update_ttl: Some(IncreaseToMilliseconds(
                    cache_client.expand_ttl_ms(Some(self.ttl))?,
                )),
            },
        )?;

        let response = cache_client
.next_data_client()
            .update_ttl(request)
            .await?
            .into_inner();

        match response.result {
            Some(update_ttl_response::Result::Missing(_)) => Ok(IncreaseTtlResponse::Miss),
            Some(update_ttl_response::Result::Set(_)) => Ok(IncreaseTtlResponse::Set),
            Some(update_ttl_response::Result::NotSet(_)) => Ok(IncreaseTtlResponse::NotSet),
            _ => Err(MomentoError::unknown_error(
                "IncreaseTtl",
                Some(format!("{:#?}", response)),
            )),
        }
    }
}

/// Response for an increase ttl operation.
#[derive(Debug, PartialEq, Eq)]
pub enum IncreaseTtlResponse {
    /// The ttl was updated successfully.
    Set,
    /// The ttl was not updated because a precondition was not met.
    NotSet,
    /// The item was not found in the cache.
    Miss,
}
