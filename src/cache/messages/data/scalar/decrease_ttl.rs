use std::time::Duration;

use momento_protos::cache_client::update_ttl_request::UpdateTtl::DecreaseToMilliseconds;
use momento_protos::cache_client::update_ttl_response::{self};

use crate::MomentoError;
use crate::{
    cache::MomentoRequest, utils::prep_request_with_timeout, CacheClient, IntoBytes, MomentoResult,
};

/// Decrease the ttl of an item in the cache.
///
/// # Arguments
/// * `cache_name` - name of cache
/// * `key` - the key of the item for which ttl is requested
/// * `ttl` - The time-to-live that should overwrite the current ttl. Should be less than the current ttl.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// use std::time::Duration;
/// use momento::cache::{DecreaseTtlResponse, DecreaseTtlRequest};
/// # cache_client.set(&cache_name, "key1", "value").await?;
///
/// let request = DecreaseTtlRequest::new(&cache_name, "key1", Duration::from_secs(3));
///
/// match(cache_client.send_request(request).await?) {
///     DecreaseTtlResponse::Set => println!("TTL updated"),
///     DecreaseTtlResponse::NotSet => println!("unable to decrease TTL"),
///     DecreaseTtlResponse::Miss => return Err(anyhow::Error::msg("cache miss"))
/// };
/// # Ok(())
/// # })
/// # }
/// ```
pub struct DecreaseTtlRequest<K: IntoBytes> {
    cache_name: String,
    key: K,
    ttl: Duration,
}

impl<K: IntoBytes> DecreaseTtlRequest<K> {
    /// Constructs a new DecreaseTtlRequest.
    pub fn new(cache_name: impl Into<String>, key: K, ttl: Duration) -> Self {
        Self {
            cache_name: cache_name.into(),
            key,
            ttl,
        }
    }
}

impl<K: IntoBytes> MomentoRequest for DecreaseTtlRequest<K> {
    type Response = DecreaseTtlResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<DecreaseTtlResponse> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.deadline_millis(),
            momento_protos::cache_client::UpdateTtlRequest {
                cache_key: self.key.into_bytes(),
                update_ttl: Some(DecreaseToMilliseconds(
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
            Some(update_ttl_response::Result::Missing(_)) => Ok(DecreaseTtlResponse::Miss),
            Some(update_ttl_response::Result::Set(_)) => Ok(DecreaseTtlResponse::Set),
            Some(update_ttl_response::Result::NotSet(_)) => Ok(DecreaseTtlResponse::NotSet),
            _ => Err(MomentoError::unknown_error(
                "DecreaseTtl",
                Some(format!("{response:#?}")),
            )),
        }
    }
}

/// Response for a decrease ttl operation.
#[derive(Debug, PartialEq, Eq)]
pub enum DecreaseTtlResponse {
    /// The ttl was successfully decreased.
    Set,
    /// The ttl could not be decreased because a precondition was not met.
    NotSet,
    /// The item was not found in the cache.
    Miss,
}
